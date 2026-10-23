//! Selected-processor argument eligibility and shared named-call lowering.
//! No runtime, module linking, or host policy belongs here.
#[cfg(test)]
use super::*;
#[cfg(not(test))]
use super::{
    CompileFailure, Document, Instruction, LocalVersion, NodeId, NodeKind, compile_apply_imports,
    compile_apply_templates, compiler, ensure_no_meaningful_children, invalid, is_ascii_ncname,
    is_xslt_element, lower_selected_value_of, meaningful_children, optional_attribute, refusal,
    require_probe_attribute_set, required_attribute, selected_keys, template_invocation_compiler,
    value_of, xpath_string_literal,
};

#[cfg(test)]
mod tests;

pub(super) fn compile(
    document: &Document,
    element: NodeId,
    version: LocalVersion,
    bindings: &[String],
) -> Result<Instruction, CompileFailure> {
    if is_xslt_element(document, element, "apply-imports") {
        // Legacy apply-imports has no parameters or body. Modern with-param
        // semantics must not enter through the shared invocation compiler.
        require_probe_attribute_set(document, element, "")?;
        ensure_no_meaningful_children(document, element, "xsl:apply-imports")?;
        return compile_apply_imports(document, element);
    }
    let named = is_xslt_element(document, element, "call-template");
    if !named
        && let Some(expression) = optional_attribute(document, element, None, "select")
        && let Some(lookup) =
            selected_keys::lookup(document, element, expression, version, bindings)?
    {
        require_probe_attribute_set(document, element, "select")?;
        ensure_no_meaningful_children(document, element, "xsl:apply-templates")?;
        return template_invocation_compiler::compile_apply_templates_with_selection(
            document,
            element,
            &mut |_, _, _, _| {
                Ok(
                    crate::xslt::golden_semantics_experiment::ApplySelection::Xslt10KeyLookup(
                        Box::new(lookup.clone()),
                    ),
                )
            },
        );
    }
    if named {
        require_probe_attribute_set(document, element, "name")?;
    } else {
        for attribute in document.attributes(element) {
            let name = document.name(*attribute).expect("attribute");
            if name.namespace.is_some()
                || !(name.local == "select"
                    || name.local == "mode"
                        && document.value(*attribute).is_some_and(is_ascii_ncname))
            {
                return Err(refusal(document, *attribute));
            }
        }
        if let Some(select) = optional_attribute(document, element, None, "select")
            && select != "*"
            && !select
                .strip_suffix("/*")
                .unwrap_or(select)
                .split('/')
                .all(is_ascii_ncname)
        {
            return Err(refusal(document, element));
        }
    }
    for child in meaningful_children(document, element) {
        if !is_xslt_element(document, child, "with-param") {
            return Err(refusal(document, child));
        }
        for attr in document.attributes(child) {
            let name = document.name(*attr).expect("attribute");
            if name.namespace.is_some() || !matches!(name.local.as_str(), "name" | "select") {
                return Err(refusal(document, *attr));
            }
        }
        let name = required_attribute(document, child, None, "name")?;
        // NCNames have identical argument normalization in both existing
        // shared branches. QName arguments and general constructors stay guarded.
        if !is_ascii_ncname(name) {
            return Err(refusal(document, child));
        }
        let Some(select) = optional_attribute(document, child, None, "select") else {
            let content =
                compiler::stylesheet_preprocessing::constructor_children(document, child)?;
            if content
                .iter()
                .any(|node| document.kind(*node) != NodeKind::Text)
                && !selected_argument_string_iteration(document, &content)?
                && !(named && selected_argument_value_node(document, &content).is_some())
            {
                return Err(refusal(document, child));
            }
            continue;
        };
        if let Some(variable) = select.strip_prefix('$').filter(|v| is_ascii_ncname(v)) {
            if !bindings.iter().any(|binding| binding == variable) {
                return Err(invalid(
                    "XPST0008",
                    "unbound template argument variable",
                    document.location(child),
                ));
            }
        } else if xpath_string_literal(select).is_none()
            && !matches!(select.trim(), "name()" | "name(.)")
            && !select.split('/').all(is_ascii_ncname)
            && !selected_integer_argument(select)
        {
            return Err(refusal(document, child));
        }
        ensure_no_meaningful_children(document, child, "xsl:with-param")?;
    }
    if named {
        template_invocation_compiler::compile_call_template_with_content(
            document,
            element,
            &mut |document, argument| compile_content(document, argument, version, bindings),
        )
    } else {
        compile_apply_templates(document, element)
    }
}

fn compile_content(
    document: &Document,
    argument: NodeId,
    version: LocalVersion,
    bindings: &[String],
) -> Result<crate::xslt::golden_semantics_experiment::TemplateArgumentValue, CompileFailure> {
    let content = compiler::stylesheet_preprocessing::constructor_children(document, argument)?;
    if let Some(node) = selected_argument_value_node(document, &content) {
        require_probe_attribute_set(document, node, "select")?;
        ensure_no_meaningful_children(document, node, "xsl:value-of")?;
        let expression = required_attribute(document, node, None, "select")?;
        let lowered = if let Some((left, right)) = variable_sum(expression) {
            for name in [left, right] {
                if !bindings.iter().any(|binding| binding == name) {
                    return Err(invalid(
                        "XPST0008",
                        format!("unbound argument variable: ${name}"),
                        document.location(node),
                    ));
                }
            }
            lower_selected_value_of(document, node, version, false)?
        } else {
            value_of(document, node, version, bindings)?
        };
        let Instruction::ValueOf { select: value, .. } = lowered else {
            return Err(refusal(document, node));
        };
        return Ok(
            crate::xslt::golden_semantics_experiment::TemplateArgumentValue::Xslt10Content(
                Box::new(
                    crate::xslt::golden_semantics_experiment::Xslt10ContentArgument {
                        bindings: Vec::new(),
                        value: Box::new(value),
                    },
                ),
            ),
        );
    }
    template_invocation_compiler::compile_content_argument_value(document, argument)
}

fn selected_argument_value_node(document: &Document, content: &[NodeId]) -> Option<NodeId> {
    let [node] = content else {
        return None;
    };
    if !is_xslt_element(document, *node, "value-of") {
        return None;
    }
    let select = optional_attribute(document, *node, None, "select")?;
    (select
        .strip_prefix('@')
        .or_else(|| select.strip_prefix('$'))
        .is_some_and(is_ascii_ncname)
        || variable_sum(select).is_some())
    .then_some(*node)
}

fn variable_sum(expression: &str) -> Option<(&str, &str)> {
    let (left, operator, right) = crate::xpath::binary_numeric_experiment::split_paths(expression)?;
    if operator != crate::xpath::binary_numeric_experiment::BinaryNumericOperator::Add {
        return None;
    }
    let left = left
        .trim()
        .strip_prefix('$')
        .filter(|name| is_ascii_ncname(name))?;
    let right = right
        .trim()
        .strip_prefix('$')
        .filter(|name| is_ascii_ncname(name))?;
    Some((left, right))
}

pub(super) fn selected_integer_argument(expression: &str) -> bool {
    // This bounded exact domain shares the existing argument representation.
    // Do not inherit Rust's optional '+' lexical extension into XPath 1.0.
    let digits = expression.strip_prefix('-').unwrap_or(expression);
    !digits.is_empty()
        && digits.bytes().all(|byte| byte.is_ascii_digit())
        && expression.parse::<i32>().is_ok()
}

// This shared leaf has no version-sensitive expression compilation: a plain
// child path is evaluated, then each selected node's string value is appended.
// Keep arbitrary constructors, local version changes and escaping outside it.
fn selected_argument_string_iteration(
    document: &Document,
    content: &[NodeId],
) -> Result<bool, CompileFailure> {
    let [for_each] = content else {
        return Ok(false);
    };
    if !is_xslt_element(document, *for_each, "for-each") {
        return Ok(false);
    }
    require_probe_attribute_set(document, *for_each, "select")?;
    let path = required_attribute(document, *for_each, None, "select")?;
    if !path.split('/').all(is_ascii_ncname) {
        return Ok(false);
    }
    let children = compiler::stylesheet_preprocessing::constructor_children(document, *for_each)?;
    let [value_of] = children.as_slice() else {
        return Ok(false);
    };
    if !is_xslt_element(document, *value_of, "value-of") {
        return Ok(false);
    }
    require_probe_attribute_set(document, *value_of, "select")?;
    ensure_no_meaningful_children(document, *value_of, "xsl:value-of")?;
    Ok(required_attribute(document, *value_of, None, "select")? == ".")
}
