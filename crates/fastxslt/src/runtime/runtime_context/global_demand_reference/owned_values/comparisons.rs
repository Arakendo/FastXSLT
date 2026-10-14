//! Demand/completed-value comparison parity and bounded conversion tests.

use super::super::RuntimeVariables;
use super::tests::{document, driver};
use super::*;
use crate::compile::golden_stylesheet_experiment::compile_stylesheet;

#[test]
fn legacy_scalar_pair_equality_obeys_type_precedence() {
    use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType::*};
    for (left, right, equal) in [
        ((Integer, "2"), (String, "02"), true),
        ((String, "02"), (Integer, "2"), true),
        ((String, "02"), (String, "2"), false),
        ((Boolean, "true"), (String, "false"), true),
        ((Boolean, "false"), (String, ""), true),
        ((Boolean, "true"), (Integer, "2"), true),
        ((Boolean, "false"), (Double, "-0"), true),
        ((Integer, "0"), (String, "-0"), true),
        ((Integer, "2"), (String, "NaN"), false),
        ((Double, "NaN"), (Double, "NaN"), false),
        ((Double, "INF"), (Double, "INF"), true),
    ] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document("<r/>"),
            r#"
            <xsl:template match="/">
              <xsl:choose><xsl:when test="$a = $b">yes|</xsl:when><xsl:otherwise>no|</xsl:otherwise></xsl:choose>
              <xsl:value-of select="$a = $b"/><xsl:text>|</xsl:text><xsl:value-of select="$a != $b"/>
            </xsl:template>"#,
            &mut control,
        );
        let mut frame = RuntimeVariables::default();
        frame.bind_atomic(
            "a".into(),
            AtomicValue::from_validated_lexical(left.0, left.1),
        );
        frame.bind_atomic(
            "b".into(),
            AtomicValue::from_validated_lexical(right.0, right.1),
        );
        let tree = driver
            .execute_with_frame(
                &driver.program.root_template.as_ref().unwrap().body,
                &frame,
                &driver.source,
                &mut control,
            )
            .unwrap();
        assert_eq!(
            temporary_tree_string_value(&tree, "scalar-pair", &mut control).unwrap(),
            if equal {
                "yes|true|false"
            } else {
                "no|false|true"
            },
            "{left:?} / {right:?}"
        );
    }
}

#[test]
fn demand_mixed_node_set_equality_uses_scalar_type_in_both_directions() {
    use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType::*};
    for (xml, kind, lexical, equal, unequal) in [
        ("<r><b>1</b><b>02</b></r>", Integer, "2", true, true),
        ("<r><b>1</b><b>02</b></r>", String, "2", false, true),
        ("<r><b>1</b><b>02</b></r>", String, "02", true, true),
        ("<r><b/></r>", Boolean, "true", true, false),
        ("<r><b/></r>", Boolean, "false", false, true),
        ("<r/>", Boolean, "false", true, false),
        ("<r/>", Boolean, "true", false, true),
        ("<r/>", String, "", false, false),
        ("<r/>", Integer, "0", false, false),
        ("<r><b>NaN</b></r>", Double, "NaN", false, true),
    ] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document(xml),
            r#"
            <xsl:template match="/">
              <xsl:choose><xsl:when test="$source = $scalar">yes|</xsl:when><xsl:otherwise>no|</xsl:otherwise></xsl:choose>
              <xsl:value-of select="$scalar = $source"/><xsl:text>|</xsl:text>
              <xsl:value-of select="$source != $scalar"/><xsl:text>|</xsl:text>
              <xsl:value-of select="$scalar != $source"/>
            </xsl:template>"#,
            &mut control,
        );
        let mut frame = RuntimeVariables::default();
        frame.bind_atomic(
            "scalar".into(),
            AtomicValue::from_validated_lexical(kind, lexical),
        );
        let tree = driver
            .execute_with_frame(
                &driver.program.root_template.as_ref().unwrap().body,
                &frame,
                &driver.source,
                &mut control,
            )
            .unwrap();
        assert_eq!(
            temporary_tree_string_value(&tree, "mixed-pair", &mut control).unwrap(),
            format!(
                "{}|{equal}|{unequal}|{unequal}",
                if equal { "yes" } else { "no" }
            ),
            "{xml} / {kind:?} / {lexical}"
        );
        assert_eq!(driver.evaluations("source"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_explicit_string_operand_preserves_the_other_operand_type() {
    use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType::*};
    for (xml, kind, lexical, expected) in [
        ("<r><b>1</b><b>2</b></r>", Integer, "2", "no|no|yes|yes"),
        ("<r><b>02</b></r>", Integer, "2", "yes|yes|no|no"),
        ("<r><b/></r>", Boolean, "true", "no|no|no|no"),
        ("<r/>", Boolean, "false", "yes|yes|no|no"),
        ("<r/>", String, "", "yes|yes|no|no"),
    ] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document(xml),
            r#"
            <xsl:template match="/">
              <xsl:choose><xsl:when test="string($source) = $scalar">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose><xsl:text>|</xsl:text>
              <xsl:choose><xsl:when test="$scalar = string($source)">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose><xsl:text>|</xsl:text>
              <xsl:choose><xsl:when test="$source = string($scalar)">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose><xsl:text>|</xsl:text>
              <xsl:choose><xsl:when test="string($scalar) = $source">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose>
            </xsl:template>"#,
            &mut control,
        );
        let mut frame = RuntimeVariables::default();
        frame.bind_atomic(
            "scalar".into(),
            AtomicValue::from_validated_lexical(kind, lexical),
        );
        let tree = driver
            .execute_with_frame(
                &driver.program.root_template.as_ref().unwrap().body,
                &frame,
                &driver.source,
                &mut control,
            )
            .unwrap();
        assert_eq!(
            temporary_tree_string_value(&tree, "explicit-mixed", &mut control).unwrap(),
            expected,
            "{xml} / {kind:?} / {lexical}"
        );
        assert_eq!(driver.evaluations("source"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_mixed_string_and_node_set_equality_keeps_conversion_direction() {
    for (xml, expected) in [
        ("<r><b>A</b><b>B</b><c>B</c></r>", "no|yes"),
        ("<r><b>B</b><c>A</c><c>B</c></r>", "yes|no"),
        ("<r><c/></r>", "yes|no"),
        ("<r><b/></r>", "no|yes"),
        ("<r/>", "no|no"),
    ] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document(xml),
            r#"
            <xsl:variable name="right" select="/r/c"/>
            <xsl:template match="/">
              <xsl:choose><xsl:when test="string($source) = $right">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose><xsl:text>|</xsl:text>
              <xsl:choose><xsl:when test="$source = string($right)">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose>
            </xsl:template>"#,
            &mut control,
        );
        assert_eq!(driver.run(&mut control).unwrap(), expected, "{xml}");
        assert_eq!(driver.evaluations("source"), 1);
        assert_eq!(driver.evaluations("right"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_source_pair_equality_and_inequality_are_independently_existential() {
    for (xml, expected) in [
        ("<r><b>A</b><b>B</b><c>B</c></r>", "yes|true|true"),
        ("<r><b>A</b><c>A</c></r>", "yes|true|false"),
        ("<r><b>A</b><c>C</c></r>", "no|false|true"),
        ("<r><b>A</b></r>", "no|false|false"),
        ("<r><c>A</c></r>", "no|false|false"),
        ("<r/>", "no|false|false"),
        ("<r><b>02</b><c>2</c></r>", "no|false|true"),
    ] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document(xml),
            r#"
            <xsl:variable name="right" select="/r/c"/>
            <xsl:template match="/">
              <xsl:choose><xsl:when test="$source = $right">yes|</xsl:when><xsl:otherwise>no|</xsl:otherwise></xsl:choose>
              <xsl:value-of select="$right = $source"/><xsl:text>|</xsl:text>
              <xsl:value-of select="$source != $right"/>
            </xsl:template>"#,
            &mut control,
        );
        assert_eq!(driver.run(&mut control).unwrap(), expected, "{xml}");
        assert_eq!(driver.evaluations("source"), 1);
        assert_eq!(driver.evaluations("right"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_source_equality_buffers_and_pair_work_are_bounded() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let mut setup = InvocationControl::unbounded();
    let driver = driver(
        document("<r><b>1</b><b>2</b><c>2</c></r>"),
        r#"
        <xsl:variable name="right" select="/r/c"/>
        <xsl:template match="/"><xsl:variable name="n" select="2"/>
          <xsl:if test="$source = $n"><xsl:if test="$source = $right">yes</xsl:if></xsl:if>
          <xsl:if test="$n = $source"/>
          <xsl:if test="string($n) = $source"/>
        </xsl:template>"#,
        &mut setup,
    );
    driver.read_value("source", &mut setup).unwrap();
    driver.read_value("right", &mut setup).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(driver.run(&mut baseline).unwrap(), "yes");
    let owners = Arc::strong_count(&driver.source);
    for domain in [
        WorkDomain::XdmNode,
        WorkDomain::XdmStringValueNode,
        WorkDomain::ResultTextByte,
        WorkDomain::XPathOperation,
    ] {
        let exact = baseline.consumed(domain);
        assert!(exact > 0);
        for (limit, cancel, succeeds) in [
            (exact, false, true),
            (exact - 1, false, false),
            (exact, true, false),
        ] {
            let mut limits = WorkLimits::unbounded();
            match domain {
                WorkDomain::XdmNode => limits.xdm_nodes = limit,
                WorkDomain::XdmStringValueNode => limits.xdm_string_value_nodes = limit,
                WorkDomain::ResultTextByte => limits.result_text_bytes = limit,
                WorkDomain::XPathOperation => limits.xpath_operations = limit,
                _ => unreachable!(),
            }
            let mut control = InvocationControl::new(CancellationToken::new(), limits);
            if cancel {
                control = control.cancelling_on_charge(domain, 1);
            }
            let result = driver.run(&mut control);
            assert_eq!(result.is_ok(), succeeds, "{domain:?}");
            if !succeeds {
                assert_eq!(
                    result.unwrap_err().code,
                    if cancel { "FXCT0001" } else { "FXCT0002" }
                );
            }
            assert_eq!(Arc::strong_count(&driver.source), owners);
            assert!(driver.resolver.stack.borrow().is_empty());
            assert_eq!(driver.evaluations("source"), 1);
            assert_eq!(driver.evaluations("right"), 1);
        }
    }
}

#[test]
fn demand_temporary_pair_equality_preserves_fragment_type() {
    use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType::*};
    for (scalar, text, expected) in [
        ((Boolean, "true"), "", "yes"),
        ((Boolean, "false"), "", "no"),
        ((Integer, "2"), "02", "yes"),
        ((String, "2"), "02", "no"),
    ] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document("<r/>"),
            &format!(
                r#"
            <xsl:template match="/">
            <xsl:choose><xsl:when test="$first = $scalar">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose></xsl:template>
            <xsl:template name="first_constructor"><v>{text}</v></xsl:template>"#
            ),
            &mut control,
        );
        let mut frame = RuntimeVariables::default();
        frame.bind_atomic(
            "scalar".into(),
            AtomicValue::from_validated_lexical(scalar.0, scalar.1),
        );
        let tree = driver
            .execute_with_frame(
                &driver.program.root_template.as_ref().unwrap().body,
                &frame,
                &driver.source,
                &mut control,
            )
            .unwrap();
        assert_eq!(
            temporary_tree_string_value(&tree, "fragment-pair", &mut control).unwrap(),
            expected
        );
        assert_eq!(driver.evaluations("first"), 1);
    }
}

#[test]
fn demand_explicit_string_pair_preserves_traversal_controls_and_cached_owner() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let mut setup = InvocationControl::unbounded();
    let driver = driver(
        document("<r><b>A</b><b>B</b></r>"),
        r#"<xsl:template match="/"><xsl:if test="string($source) = string($source)">yes</xsl:if></xsl:template>"#,
        &mut setup,
    );
    driver.read_value("source", &mut setup).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(driver.run(&mut baseline).unwrap(), "yes");
    let exact = baseline.consumed(WorkDomain::XdmStringValueNode);
    assert!(exact > 0);
    let owners = Arc::strong_count(&driver.source);
    for (limit, cancel, succeeds) in [
        (exact, false, true),
        (exact - 1, false, false),
        (exact, true, false),
    ] {
        let mut limits = WorkLimits::unbounded();
        limits.xdm_string_value_nodes = limit;
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        if cancel {
            control = control.cancelling_on_charge(WorkDomain::XdmStringValueNode, 1);
        }
        let result = driver.run(&mut control);
        assert_eq!(result.is_ok(), succeeds);
        if !succeeds {
            assert_eq!(
                result.unwrap_err().code,
                if cancel { "FXCT0001" } else { "FXCT0002" }
            );
        }
        assert_eq!(driver.evaluations("source"), 1);
        assert_eq!(Arc::strong_count(&driver.source), owners);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_scalar_equality_controls_do_not_poison_completed_bindings() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let mut setup = InvocationControl::unbounded();
    let driver = driver(
        document("<r/>"),
        r#"
        <xsl:template match="/"><xsl:variable name="n" select="2"/>
          <xsl:if test="$first = $n">yes</xsl:if></xsl:template>
        <xsl:template name="first_constructor"><v>02</v></xsl:template>"#,
        &mut setup,
    );
    driver.read_value("first", &mut setup).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(driver.run(&mut baseline).unwrap(), "yes");
    let exact = baseline.consumed(WorkDomain::XPathOperation);
    assert!(exact > 0);
    for (limit, cancel, succeeds) in [
        (exact, false, true),
        (exact - 1, false, false),
        (exact, true, false),
    ] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = limit;
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        if cancel {
            control = control.cancelling_on_charge(WorkDomain::XPathOperation, 1);
        }
        let result = driver.run(&mut control);
        assert_eq!(result.is_ok(), succeeds);
        if !succeeds {
            assert_eq!(
                result.unwrap_err().code,
                if cancel { "FXCT0001" } else { "FXCT0002" }
            );
        }
        assert!(driver.resolver.stack.borrow().is_empty());
        assert_eq!(driver.evaluations("first"), 1);
    }
    assert_eq!(driver.run(&mut setup).unwrap(), "yes");
    assert_eq!(driver.evaluations("first"), 1);
}

#[test]
fn demand_explicit_string_pair_uses_first_node_and_empty_string() {
    for (xml, text, expected) in [
        ("<r><b>A</b><b>B</b></r>", "A", "yes"),
        ("<r><b>A</b><b>B</b></r>", "B", "no"),
        ("<r/>", "", "yes"),
    ] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document(xml),
            &format!(
                r#"
            <xsl:template match="/"><xsl:choose>
              <xsl:when test="string($source) = string($first)">yes</xsl:when>
              <xsl:otherwise>no</xsl:otherwise>
            </xsl:choose></xsl:template>
            <xsl:template name="first_constructor"><v>{text}</v></xsl:template>"#
            ),
            &mut control,
        );
        for _ in 0..2 {
            assert_eq!(driver.run(&mut control).unwrap(), expected);
        }
        assert_eq!(driver.evaluations("source"), 1);
        assert_eq!(driver.evaluations("first"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn production_source_numeric_equality_golden_uses_complete_node_set() {
    use crate::runtime::golden_runtime_experiment::{execute_program, serialize_xml};
    let source = document(include_str!(
        "../../../../../../../corpus/golden/source-numeric-equality/input.xml"
    ));
    let stylesheet = document(include_str!(
        "../../../../../../../corpus/golden/source-numeric-equality/stylesheet.xsl"
    ));
    let program = compile_stylesheet(&stylesheet).unwrap();
    let mut control = InvocationControl::unbounded();
    let result = execute_program(&program, &source, "numeric-golden", &mut control).unwrap();
    let result = serialize_xml(
        &result,
        &program.output,
        "numeric-golden",
        4096,
        &mut control,
    )
    .unwrap();
    assert_eq!(
        result,
        include_str!("../../../../../../../corpus/golden/source-numeric-equality/expected.xml")
            .trim_end()
    );
}

#[test]
fn demand_owned_comparison_bridge_preserves_sequence_conversion() {
    for (xml, expression, expected, first) in [
        ("<r><b>A</b><b>B</b></r>", "$source = 'B'", "yes", "A"),
        ("<r><b>A</b><b>B</b></r>", "$source != 'A'", "yes", "A"),
        ("<r/>", "$source = ''", "no", ""),
        ("<r><b>1</b><b>2</b></r>", "$source = 2", "yes", "1"),
        ("<r/>", "$source = 0", "no", ""),
        ("<r><b>1</b><b>3</b></r>", "$source &gt; 2", "yes", "1"),
        ("<r><b>3</b><b>1</b></r>", "$source &lt; 2", "yes", "3"),
        ("<r><b>NaN</b><b>2</b></r>", "2 &lt;= $source", "yes", "NaN"),
        ("<r/>", "$source &gt;= 0", "no", ""),
        ("<r><b>3</b><b>2</b></r>", "$source &lt;= 2", "yes", "3"),
        ("<r><b>NaN</b></r>", "$source &gt; 0", "no", "NaN"),
        ("<r><b>NaN</b><b>2</b></r>", "$source = 2", "yes", "NaN"),
        (
            "<r><b>1</b><b>2147483648</b></r>",
            "$source = 2147483648",
            "yes",
            "1",
        ),
    ] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document(xml),
            &format!(
                r#"
      <xsl:template match="/"><xsl:choose><xsl:when test="{expression}">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose></xsl:template>"#
            ),
            &mut control,
        );
        assert_eq!(driver.run(&mut control).unwrap(), expected, "{expression}");
        assert_eq!(
            driver
                .read_string("source", &mut control)
                .unwrap()
                .as_deref(),
            Some(first)
        );
        assert_eq!(driver.evaluations("source"), 1);
        let value = driver.read_value("source", &mut control).unwrap().unwrap();
        let OwnedValue::Source(selection) = value.as_ref() else {
            panic!("source selection")
        };
        let mut frame = RuntimeVariables::default();
        frame.bind_retained_source_nodes(
            "source".into(),
            Arc::clone(&selection.owner),
            selection.nodes.clone(),
        );
        let result = driver
            .execute_with_frame(
                &driver.program.root_template.as_ref().unwrap().body,
                &frame,
                &driver.source,
                &mut control,
            )
            .unwrap();
        assert_eq!(
            temporary_tree_string_value(&result, "comparison", &mut control).unwrap(),
            expected
        );
    }
}

#[test]
fn demand_temporary_equality_reads_both_bindings_once() {
    for (right, expected) in [("same", "yes"), ("different", "no")] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document("<r/>"),
            &format!(
                r#"
            <xsl:template match="/"><xsl:choose><xsl:when test="$first = $second">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose></xsl:template>
            <xsl:template name="first_constructor"><v>same</v></xsl:template>
            <xsl:template name="second_constructor"><v>{right}</v></xsl:template>"#
            ),
            &mut control,
        );
        for _ in 0..2 {
            assert_eq!(driver.run(&mut control).unwrap(), expected);
        }
        assert_eq!(driver.evaluations("first"), 1);
        assert_eq!(driver.evaluations("second"), 1);
    }
}

#[test]
fn demand_temporary_value_inequality_preserves_local_shadowing() {
    for (right, expected) in [("same", "false"), ("other", "true")] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document("<r/>"),
            &format!(
                r#"
            <xsl:template match="/"><xsl:variable name="first" select="'same'"/><xsl:value-of select="$first != $second"/></xsl:template>
            <xsl:template name="first_constructor"><xsl:message terminate="yes">must not run</xsl:message></xsl:template>
            <xsl:template name="second_constructor"><v>{right}</v></xsl:template>"#
            ),
            &mut control,
        );
        assert_eq!(driver.run(&mut control).unwrap(), expected);
        assert_eq!(driver.evaluations("first"), 0);
        assert_eq!(driver.evaluations("second"), 1);
        assert!(control.messages().is_empty());
    }
}

#[test]
fn demand_mixed_equality_preserves_source_and_cleans_recursive_cycle() {
    for (expression, first) in [
        ("$source = $second", "unused"),
        ("$first = $second", "cycle"),
    ] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document("<r><b>A</b><b>B</b></r>"),
            &format!(
                r#"
            <xsl:template match="/"><xsl:if test="{expression}">wrong</xsl:if></xsl:template>
            <xsl:template name="first_constructor"><xsl:if test="$first = $second">wrong</xsl:if></xsl:template>
            <xsl:template name="second_constructor">A</xsl:template>"#
            ),
            &mut control,
        );
        if first == "unused" {
            assert_eq!(driver.run(&mut control).unwrap(), "wrong");
        } else {
            assert_eq!(driver.run(&mut control).unwrap_err().code, "FXAR0030");
        }
        assert!(driver.resolver.stack.borrow().is_empty());
        assert_eq!(driver.evaluations("second"), usize::from(first == "unused"));
        assert_eq!(driver.evaluations("first"), usize::from(first == "cycle"));
        if first == "unused" {
            assert_eq!(
                driver
                    .read_string("source", &mut control)
                    .unwrap()
                    .as_deref(),
                Some("A")
            );
            assert_eq!(driver.evaluations("source"), 1);
        }
    }
}

#[test]
fn demand_boolean_relations_preserve_whole_node_set_conversion() {
    use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
    for (xml, boolean, expected) in [
        ("<r><b>-9</b><b>9</b></r>", "true", "no|yes|no|yes"),
        ("<r><b>NaN</b></r>", "false", "yes|yes|yes|yes"),
        ("<r/>", "true", "no|no|no|no"),
        ("<r/>", "false", "no|yes|no|yes"),
    ] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document(xml),
            r#"
          <xsl:template match="/">
            <xsl:choose><xsl:when test="$source &gt; $flag">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose><xsl:text>|</xsl:text>
            <xsl:choose><xsl:when test="$source &gt;= $flag">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose><xsl:text>|</xsl:text>
            <xsl:choose><xsl:when test="$flag &lt; $source">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose><xsl:text>|</xsl:text>
            <xsl:choose><xsl:when test="$flag &lt;= $source">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose>
          </xsl:template>"#,
            &mut control,
        );
        let mut frame = RuntimeVariables::default();
        frame.bind_atomic(
            "flag".into(),
            AtomicValue::from_validated_lexical(BuiltinAtomicType::Boolean, boolean),
        );
        let tree = driver
            .execute_with_frame(
                &driver.program.root_template.as_ref().unwrap().body,
                &frame,
                &driver.source,
                &mut control,
            )
            .unwrap();
        assert_eq!(
            temporary_tree_string_value(&tree, "boolean-relation", &mut control).unwrap(),
            expected,
            "{xml} / {boolean}"
        );
        assert_eq!(driver.evaluations("source"), 1);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn production_boolean_relation_golden_preserves_controls() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    use crate::runtime::golden_runtime_experiment::runtime_context::InvocationParameter;
    use crate::runtime::golden_runtime_experiment::{
        MultipleMatchPolicy, execute_program_with_parameters, serialize_xml,
    };
    use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
    let source = document(include_str!(
        "../../../../../../../corpus/golden/boolean-variable-relations/input.xml"
    ));
    let stylesheet = document(include_str!(
        "../../../../../../../corpus/golden/boolean-variable-relations/stylesheet.xsl"
    ));
    let program = compile_stylesheet(&stylesheet).unwrap();
    let parameters = std::collections::BTreeMap::from([(
        "flag".into(),
        InvocationParameter {
            value: AtomicValue::from_validated_lexical(BuiltinAtomicType::Boolean, "true").into(),
            tunnel: false,
        },
    )]);
    let execute = |control: &mut InvocationControl| {
        let result = execute_program_with_parameters(
            &program,
            &source,
            &parameters,
            MultipleMatchPolicy::UseLast,
            "boolean-golden",
            control,
        )?;
        serialize_xml(&result, &program.output, "boolean-golden", 4096, control)
    };
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(
        execute(&mut baseline).unwrap(),
        include_str!("../../../../../../../corpus/golden/boolean-variable-relations/expected.xml")
            .trim_end()
    );
    let exact = baseline.consumed(WorkDomain::XPathOperation);
    for (limit, cancelled, succeeds) in [
        (exact, false, true),
        (exact - 1, false, false),
        (exact, true, false),
    ] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = limit;
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        if cancelled {
            control = control.cancelling_on_charge(WorkDomain::XPathOperation, 1);
        }
        let result = execute(&mut control);
        assert_eq!(result.is_ok(), succeeds);
        if !succeeds {
            assert_eq!(
                result.unwrap_err().code,
                if cancelled { "FXCT0001" } else { "FXCT0002" }
            );
        }
    }
}

#[test]
fn legacy_scalar_relations_use_typed_numbers_and_booleans() {
    use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType::*};
    for (left, right, expected) in [
        ((Boolean, "true"), (Integer, "0"), "greater"),
        ((Boolean, "1"), (Boolean, "0"), "greater"),
        ((Boolean, "false"), (String, "-1"), "greater"),
        ((Boolean, "false"), (String, "true"), "unordered"),
        ((Double, "INF"), (Integer, "2"), "greater"),
        ((Double, "-INF"), (Integer, "2"), "less"),
        ((Double, "NaN"), (Boolean, "true"), "unordered"),
        ((Double, "-0"), (Boolean, "false"), "equal"),
        ((String, "INF"), (Integer, "2"), "unordered"),
    ] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document("<r/>"),
            r#"<xsl:template match="/"><xsl:choose>
          <xsl:when test="$a &gt; $b">greater</xsl:when><xsl:when test="$a &lt; $b">less</xsl:when>
          <xsl:when test="$a &lt;= $b">equal</xsl:when><xsl:otherwise>unordered</xsl:otherwise>
        </xsl:choose></xsl:template>"#,
            &mut control,
        );
        let mut frame = RuntimeVariables::default();
        frame.bind_atomic(
            "a".into(),
            AtomicValue::from_validated_lexical(left.0, left.1),
        );
        frame.bind_atomic(
            "b".into(),
            AtomicValue::from_validated_lexical(right.0, right.1),
        );
        let tree = driver
            .execute_with_frame(
                &driver.program.root_template.as_ref().unwrap().body,
                &frame,
                &driver.source,
                &mut control,
            )
            .unwrap();
        assert_eq!(
            temporary_tree_string_value(&tree, "scalar-relation", &mut control).unwrap(),
            expected,
            "{left:?} / {right:?}"
        );
    }
}

#[test]
fn demand_source_pair_relation_uses_later_pairs_and_empty_sets() {
    for (xml, expression, expected) in [
        (
            "<r><b>1</b><b>4</b><c>3</c></r>",
            "$source &gt; $right",
            "yes",
        ),
        (
            "<r><b>2</b><c>3</c><c>1</c></r>",
            "$source &gt; $right",
            "yes",
        ),
        ("<r><b>2</b></r>", "$source &gt; $right", "no"),
        ("<r><c>2</c></r>", "$source &lt; $right", "no"),
        ("<r><b>NaN</b><c>2</c></r>", "$source &lt;= $right", "no"),
    ] {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document(xml),
            &format!(
                r#"<xsl:variable name="right" select="/r/c"/><xsl:template match="/"><xsl:choose><xsl:when test="{expression}">yes</xsl:when><xsl:otherwise>no</xsl:otherwise></xsl:choose></xsl:template>"#
            ),
            &mut control,
        );
        assert_eq!(driver.run(&mut control).unwrap(), expected, "{xml}");
        assert_eq!(driver.evaluations("source"), 1);
        assert_eq!(driver.evaluations("right"), 1);
    }
}

#[test]
fn demand_pair_comparison_work_and_retention_are_bounded() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let mut setup = InvocationControl::unbounded();
    let driver = driver(
        document("<r><b>1</b><b>2</b><c>3</c><c>4</c></r>"),
        r#"<xsl:variable name="right" select="/r/c"/><xsl:template match="/"><xsl:if test="$source &gt; $right">unexpected</xsl:if></xsl:template>"#,
        &mut setup,
    );
    driver.read_value("source", &mut setup).unwrap();
    driver.read_value("right", &mut setup).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(driver.run(&mut baseline).unwrap(), "");
    let exact = baseline.consumed(WorkDomain::XPathOperation);
    assert!(exact >= 4);
    let owners = Arc::strong_count(&driver.source);
    for (limit, retain, cancel, succeeds) in [
        (exact, usize::MAX, false, true),
        (exact - 1, usize::MAX, false, false),
        (exact, 0, false, false),
        (exact, usize::MAX, true, false),
    ] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = limit;
        limits.xdm_nodes = retain;
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        if cancel {
            control = control.cancelling_on_charge(WorkDomain::XPathOperation, 1);
        }
        let result = driver.run(&mut control);
        assert_eq!(result.is_ok(), succeeds);
        if !succeeds {
            assert_eq!(
                result.unwrap_err().code,
                if cancel { "FXCT0001" } else { "FXCT0002" }
            );
        }
        assert_eq!(Arc::strong_count(&driver.source), owners);
        assert_eq!(driver.evaluations("source"), 1);
        assert_eq!(driver.evaluations("right"), 1);
    }
}

#[test]
fn demand_source_relation_charges_later_node_visits_and_preserves_cached_value() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let mut setup = InvocationControl::unbounded();
    let driver = driver(
        document("<r><b>1</b><b>3</b></r>"),
        r#"<xsl:template match="/"><xsl:if test="$source &gt; 2">yes</xsl:if></xsl:template>"#,
        &mut setup,
    );
    driver.read_value("source", &mut setup).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(driver.run(&mut baseline).unwrap(), "yes");
    let exact = baseline.consumed(WorkDomain::XdmStringValueNode);
    assert!(exact > 0);
    let owners = Arc::strong_count(&driver.source);
    for (limit, cancel, succeeds) in [
        (exact, false, true),
        (exact - 1, false, false),
        (exact, true, false),
    ] {
        let mut limits = WorkLimits::unbounded();
        limits.xdm_string_value_nodes = limit;
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        if cancel {
            control = control.cancelling_on_charge(WorkDomain::XdmStringValueNode, 1);
        }
        let result = driver.run(&mut control);
        assert_eq!(result.is_ok(), succeeds);
        if !succeeds {
            assert_eq!(
                result.unwrap_err().code,
                if cancel { "FXCT0001" } else { "FXCT0002" }
            );
        }
        assert_eq!(driver.evaluations("source"), 1);
        assert_eq!(Arc::strong_count(&driver.source), owners);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
}

#[test]
fn demand_source_comparison_preserves_shadowing_and_foreign_owner_guard() {
    let mut control = InvocationControl::unbounded();
    let driver = driver(
        document("<r><b>A</b><b>B</b></r>"),
        r#"<xsl:template match="/"><xsl:if test="$source = 'B'">yes</xsl:if></xsl:template>"#,
        &mut control,
    );
    let body = &driver.program.root_template.as_ref().unwrap().body;
    let mut frame = RuntimeVariables::default();
    frame.bind_retained_source_nodes("source".into(), Arc::clone(&driver.source), Vec::new());
    for complete in [false, true] {
        let frame = if complete {
            frame.clone_complete_for_sequence()
        } else {
            frame.clone()
        };
        let result = driver
            .execute_with_frame(body, &frame, &driver.source, &mut control)
            .unwrap();
        assert_eq!(
            temporary_tree_string_value(&result, "shadow", &mut control).unwrap(),
            ""
        );
    }
    assert_eq!(driver.evaluations("source"), 0);
    let foreign = document("<r><b>A</b><b>B</b></r>");
    let error = driver
        .execute_with_frame(body, &RuntimeVariables::default(), &foreign, &mut control)
        .unwrap_err();
    assert_eq!(error.code, "FXRT1017");
    assert_eq!(driver.evaluations("source"), 1);
    assert_eq!(driver.run(&mut control).unwrap(), "yes");
    assert_eq!(driver.evaluations("source"), 1);
}

#[test]
fn demand_source_comparison_handoff_failure_keeps_completed_value() {
    use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
    let mut initial = InvocationControl::unbounded();
    let driver = driver(
        document("<r><b>A</b><b>B</b></r>"),
        r#"<xsl:template match="/"><xsl:if test="$source = 'B'">yes</xsl:if></xsl:template>"#,
        &mut initial,
    );
    driver.read_value("source", &mut initial).unwrap();
    let owners = Arc::strong_count(&driver.source);
    for cancel in [false, true] {
        let mut limits = WorkLimits::unbounded();
        if !cancel {
            limits.xdm_nodes = 1;
        }
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        if cancel {
            control = control.cancelling_on_charge(WorkDomain::XdmNode, 1);
        }
        let error = driver.run(&mut control).unwrap_err();
        assert_eq!(error.code, if cancel { "FXCT0001" } else { "FXCT0002" });
        assert_eq!(driver.evaluations("source"), 1);
        assert_eq!(Arc::strong_count(&driver.source), owners);
        assert!(driver.resolver.stack.borrow().is_empty());
    }
    assert_eq!(driver.run(&mut initial).unwrap(), "yes");
    assert_eq!(driver.evaluations("source"), 1);
}
