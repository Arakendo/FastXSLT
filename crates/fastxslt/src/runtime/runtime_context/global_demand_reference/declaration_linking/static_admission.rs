//! Bounded straight-line static admission, not a second expression evaluator.
//! Unknown language shapes fail closed; refusal is not a standards diagnostic.

use super::*;
use crate::compile::golden_stylesheet_experiment::global_constructor_reference::CompiledReference;
use crate::xslt::golden_semantics_experiment::ValueExpression;
use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Eq)]
enum Refusal {
    OutsideSubset,
    RecursiveTemplate,
    DependencyCycle,
    Limit,
}

struct Plan {
    order: Vec<usize>,
    dependencies: Vec<BTreeSet<usize>>,
}

pub(super) fn assert_outside_subset(compiled: &CompiledReference) {
    assert!(matches!(plan(compiled, 1024), Err(Refusal::OutsideSubset)));
}

fn plan(compiled: &CompiledReference, mut steps: usize) -> Result<Plan, Refusal> {
    let bindings = &compiled.program.global_bindings;
    if bindings.len() > 32 || compiled.program.named_templates.len() > 32 {
        return Err(Refusal::Limit);
    }
    let mut dependencies = Vec::new();
    for binding in bindings {
        // Parameter defaults may be replaced by a host value. This first static
        // candidate refuses them rather than inventing unconditional edges.
        if binding.kind != crate::xslt::golden_semantics_experiment::GlobalBindingKind::Variable {
            return Err(Refusal::OutsideSubset);
        }
        let mut edges = BTreeSet::new();
        scan(
            compiled,
            &compiled.bodies[&binding.name],
            &mut Vec::new(),
            &mut steps,
            &mut edges,
        )?;
        dependencies.push(edges);
    }
    let mut order = Vec::new();
    while order.len() < bindings.len() {
        let Some(ready) = (0..bindings.len()).find(|index| {
            !order.contains(index) && dependencies[*index].iter().all(|edge| order.contains(edge))
        }) else {
            return Err(Refusal::DependencyCycle);
        };
        order.push(ready);
    }
    Ok(Plan {
        order,
        dependencies,
    })
}

fn scan(
    compiled: &CompiledReference,
    body: &[Instruction],
    active: &mut Vec<usize>,
    steps: &mut usize,
    edges: &mut BTreeSet<usize>,
) -> Result<(), Refusal> {
    for instruction in body {
        *steps = steps.checked_sub(1).ok_or(Refusal::Limit)?;
        match instruction {
            Instruction::Text { .. } => {}
            Instruction::ValueOf {
                select: ValueExpression::Xslt10VariableString(name),
                ..
            } => {
                let ordinal = compiled
                    .program
                    .global_bindings
                    .iter()
                    .position(|binding| binding.name == *name)
                    .ok_or(Refusal::OutsideSubset)?;
                edges.insert(ordinal);
            }
            Instruction::CallTemplate {
                name, arguments, ..
            } if arguments.is_empty() => {
                let ordinal = compiled
                    .program
                    .named_templates
                    .iter()
                    .position(|template| template.name == *name)
                    .ok_or(Refusal::OutsideSubset)?;
                let template = &compiled.program.named_templates[ordinal].template;
                if !template.parameters.is_empty() {
                    return Err(Refusal::OutsideSubset);
                }
                if active.contains(&ordinal) {
                    return Err(Refusal::RecursiveTemplate);
                }
                if active.len() == 32 {
                    return Err(Refusal::Limit);
                }
                active.push(ordinal);
                let result = scan(compiled, &template.body, active, steps, edges);
                active.pop();
                result?;
            }
            // Includes conditional reads, dynamic dispatch and lexical bindings.
            // Do not scan their children as if execution were unconditional.
            _ => return Err(Refusal::OutsideSubset),
        }
    }
    Ok(())
}

fn execute(compiled: CompiledReference, static_plan: Option<&Plan>) -> String {
    let mut control = InvocationControl::unbounded();
    let driver = Driver::with_constructor_bodies(
        Arc::new(compiled.program),
        document("urn:static-admission:source", "<r><b/></r>"),
        &BTreeMap::new(),
        false,
        compiled.bodies,
        &mut control,
    )
    .unwrap();
    if let Some(plan) = static_plan {
        for &ordinal in &plan.order {
            for &dependency in &plan.dependencies[ordinal] {
                assert_eq!(driver.evaluations.borrow()[dependency], 1);
            }
            let before = driver.evaluations.borrow().clone();
            driver
                .read_value(&driver.program.global_bindings[ordinal].name, &mut control)
                .unwrap();
            // Shared executor must not discover an unplanned dependency while
            // executing an admitted straight-line constructor.
            for (index, previous) in before.iter().enumerate() {
                assert_eq!(
                    driver.evaluations.borrow()[index],
                    previous + usize::from(index == ordinal)
                );
            }
        }
    }
    let output = driver.run(&mut control).unwrap();
    assert!(driver.resolver.stack.borrow().is_empty());
    output
}

#[test]
fn static_admission_orders_forward_named_calls_with_shared_execution() {
    let body = r#"<xsl:variable name="a:first"><xsl:call-template name="outer"/></xsl:variable>
      <xsl:variable name="second">B</xsl:variable>
      <xsl:template name="outer"><xsl:call-template name="inner"/></xsl:template>
      <xsl:template name="inner"><xsl:value-of select="$second"/></xsl:template>
      <xsl:template match="/"><xsl:value-of select="$b:first"/></xsl:template>"#;
    let compiled = declaration_program(body).unwrap();
    let planned = plan(&compiled, 64).unwrap();
    assert_eq!(planned.order, [1, 0]);
    assert_eq!(execute(compiled, Some(&planned)), "B");
    assert_eq!(execute(declaration_program(body).unwrap(), None), "B");
}

#[test]
fn static_admission_refuses_dynamic_reads_without_inventing_cycles() {
    for constructor in [
        r#"<xsl:if test="false()"><xsl:value-of select="$first"/></xsl:if>A"#,
        r#"<xsl:apply-templates select="//b" mode="chosen"/>"#,
        r#"<xsl:variable name="first" select="'local'"/><xsl:value-of select="$first"/>"#,
    ] {
        let body = format!(
            r#"<xsl:variable name="first">{constructor}</xsl:variable>
          <xsl:template match="b" mode="chosen">A</xsl:template>
          <xsl:template match="b" mode="other"><xsl:value-of select="$first"/></xsl:template>
          <xsl:template match="/"><xsl:value-of select="$first"/></xsl:template>"#
        );
        let compiled = declaration_program(&body).unwrap();
        assert!(matches!(plan(&compiled, 64), Err(Refusal::OutsideSubset)));
        assert!(matches!(execute(compiled, None).as_str(), "A" | "local"));
    }
}

#[test]
fn static_admission_distinguishes_cycle_recursion_and_limits() {
    let cycle = declaration_program(
        r#"<xsl:variable name="first"><xsl:value-of select="$second"/></xsl:variable>
      <xsl:variable name="second"><xsl:value-of select="$first"/></xsl:variable>"#,
    )
    .unwrap();
    assert!(matches!(plan(&cycle, 64), Err(Refusal::DependencyCycle)));
    let recursive = declaration_program(
        r#"<xsl:variable name="first"><xsl:call-template name="loop"/></xsl:variable>
      <xsl:template name="loop"><xsl:call-template name="loop"/></xsl:template>"#,
    )
    .unwrap();
    assert!(matches!(
        plan(&recursive, 64),
        Err(Refusal::RecursiveTemplate)
    ));
    let literal = declaration_program(r#"<xsl:variable name="first">A</xsl:variable>"#).unwrap();
    assert!(plan(&literal, 1).is_ok());
    assert!(matches!(plan(&literal, 0), Err(Refusal::Limit)));
    let param = declaration_program(r#"<xsl:param name="first">A</xsl:param>"#).unwrap();
    assert!(matches!(plan(&param, 64), Err(Refusal::OutsideSubset)));
}

#[test]
fn static_admission_uses_shared_import_winners_not_losing_bodies() {
    let principal = r#"<xsl:import href="sealed.xsl"/>
      <xsl:variable name="b:first"><xsl:call-template name="helper"/></xsl:variable>
      <xsl:template name="helper">winner</xsl:template>
      <xsl:template match="/"><xsl:value-of select="$second"/></xsl:template>"#;
    let imported = r#"<xsl:variable name="a:first"><xsl:value-of select="$second"/></xsl:variable>
      <xsl:variable name="second"><xsl:value-of select="$b:first"/></xsl:variable>
      <xsl:template name="helper"><xsl:value-of select="$second"/></xsl:template>"#;
    for strict in [false, true] {
        let compiled = dependency_program(principal, imported, strict, true).unwrap();
        let planned = plan(&compiled, 64).unwrap();
        let first = compiled
            .program
            .global_bindings
            .iter()
            .position(|binding| binding.name != "second")
            .unwrap();
        let second = compiled
            .program
            .global_bindings
            .iter()
            .position(|binding| binding.name == "second")
            .unwrap();
        assert_eq!(planned.order, [first, second]);
        assert_eq!(execute(compiled, Some(&planned)), "winner");
        assert_eq!(
            execute(
                dependency_program(principal, imported, strict, true).unwrap(),
                None
            ),
            "winner"
        );
    }
}

#[test]
fn static_admission_enforces_binding_and_template_caps() {
    use std::fmt::Write;
    for count in [32, 33] {
        let mut body = String::new();
        for index in 0..count {
            write!(body, r#"<xsl:variable name="v{index}">A</xsl:variable>"#).unwrap();
        }
        let compiled = declaration_program(&body).unwrap();
        assert_eq!(plan(&compiled, 64).is_ok(), count == 32);
        let mut body = String::from(r#"<xsl:variable name="first">A</xsl:variable>"#);
        for index in 0..count {
            write!(body, r#"<xsl:template name="t{index}">T</xsl:template>"#).unwrap();
        }
        let compiled = declaration_program(&body).unwrap();
        assert_eq!(plan(&compiled, 64).is_ok(), count == 32);
    }
}
