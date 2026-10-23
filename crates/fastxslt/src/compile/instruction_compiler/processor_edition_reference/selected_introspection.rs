//! Private selected-1.0 numeric processor reporting; not production promotion.
//! `QName` parsing stays in the shared introspection owner. No availability table
//! or declaration-derived processor identity is imported into this route.
#[cfg(test)]
use super::*;
#[cfg(not(test))]
use super::{
    BooleanExpression, Document, LocalVersion, NodeId, XSLT_NAMESPACE,
    xslt10_static_introspection_compiler,
};

#[cfg(test)]
#[path = "introspection_boundary_tests.rs"]
mod boundary_tests;

pub(super) fn version_number(
    document: &Document,
    element: NodeId,
    expression: &str,
    version: LocalVersion,
) -> Option<f64> {
    if matches!(version, LocalVersion::ModernWitness(_)) {
        return None;
    }
    xslt10_static_introspection_compiler::fold_with_policy(
        document,
        element,
        expression,
        |function, namespace, local| {
            (function == "system-property"
                && namespace == Some(XSLT_NAMESPACE)
                && local == "version")
                .then_some(1.0)
        },
    )
}

#[allow(
    clippy::float_cmp,
    reason = "XPath 1.0 numeric equality is exact IEEE-754 equality"
)]
pub(super) fn boolean_leaf(
    document: &Document,
    element: NodeId,
    expression: &str,
    version: LocalVersion,
) -> Option<BooleanExpression> {
    if let Some(number) = version_number(document, element, expression, version) {
        return Some(BooleanExpression::Constant(
            number != 0.0 && !number.is_nan(),
        ));
    }
    for operator in [">=", "<=", "!=", ">", "<", "="] {
        let Some((left, right)) = expression.split_once(operator) else {
            continue;
        };
        let left = version_number(document, element, left.trim(), version)?;
        // This slice admits a numeric literal, not strings, booleans, arithmetic
        // or dynamic conversions that would need their own typed lowering.
        let right =
            crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(right.trim())?;
        return Some(BooleanExpression::Constant(match operator {
            ">=" => left >= right,
            "<=" => left <= right,
            "!=" => left != right,
            ">" => left > right,
            "<" => left < right,
            "=" => left == right,
            _ => unreachable!(),
        }));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn fixture() -> (Document, NodeId) {
        let document = Document::from_parsed(parse_document("urn:version-leaf", br#"<x:stylesheet xmlns:x="http://www.w3.org/1999/XSL/Transform" xmlns:p="http://www.w3.org/1999/XSL/Transform" xmlns:q="urn:other" version="2.0"/>"#, ParseLimits { max_events: 20, max_depth: 4 }).unwrap()).unwrap();
        let root = compiler::document_element(&document).unwrap();
        (document, root)
    }

    #[test]
    fn selected_numeric_version_leaf_uses_qnames_and_numeric_comparisons() {
        let (document, root) = fixture();
        for version in [LocalVersion::LegacyNormal, LocalVersion::LegacyForward] {
            let number =
                version_number(&document, root, "system-property('p:version')", version).unwrap();
            assert_eq!(number.to_bits(), 1.0_f64.to_bits());
            for (suffix, expected) in [
                ("", true),
                (" >= 1", true),
                (" <= .5", false),
                (" != 1.0", false),
                (" > -0", true),
                (" < 2", true),
                (" = .999999999999999999999", true),
            ] {
                let expression = format!("system-property('p:version'){suffix}");
                assert!(
                    matches!(boolean_leaf(&document, root, &expression, version), Some(BooleanExpression::Constant(value)) if value == expected),
                    "{expression}"
                );
            }
        }
    }

    #[test]
    fn selected_numeric_version_leaf_does_not_admit_other_properties_or_types() {
        let (document, root) = fixture();
        assert!(
            version_number(
                &document,
                root,
                "system-property('p:version')",
                LocalVersion::ModernWitness(DeclaredVersion::Two)
            )
            .is_none()
        );
        for expression in [
            "system-property('q:version')",
            "system-property('missing:version')",
            "system-property('version')",
            "system-property('p:vendor')",
            "element-available('p:version')",
            "system-property('p:version') = '1'",
            "system-property('p:version') = true()",
            "system-property('p:version') > $v",
            "system-property('p:version') + 1",
            "2 > system-property('p:version')",
            "system-property('p:version') = 1 = 1",
        ] {
            assert!(
                boolean_leaf(&document, root, expression, LocalVersion::LegacyForward).is_none(),
                "{expression}"
            );
        }
    }

    #[test]
    fn selected_version_composition_preserves_deferred_and_static_errors() {
        use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
        for (comparison, fails) in [("&gt;= 2", false), ("&lt; 2", true)] {
            let xml = format!(
                r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:output method="text"/><xsl:template match="/"><xsl:if test="system-property('xsl:version') {comparison} and system-property()"><xsl:text>wrong</xsl:text></xsl:if><xsl:text>ok</xsl:text></xsl:template></xsl:stylesheet>"#
            );
            let document = Document::from_parsed(
                parse_document(
                    "urn:version-timing",
                    xml.as_bytes(),
                    ParseLimits {
                        max_events: 100,
                        max_depth: 10,
                    },
                )
                .unwrap(),
            )
            .unwrap();
            let program = compile(
                &document,
                Edition::Legacy,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let result = execute_compiled_stylesheet_for_test(&program, &document, "timing");
            if fails {
                let error = result.unwrap_err();
                assert!(error.contains("XPST0017") && error.contains("urn:version-timing"));
            } else {
                assert_eq!(result.unwrap(), "ok");
            }
            let root = compiler::document_element(&document).unwrap();
            for expression in [
                "system-property('xsl:version') >= 2 and $missing",
                "system-property('xsl:version') < 2 or $missing",
            ] {
                let error = boolean_call_timing::condition(
                    expression,
                    LocalVersion::LegacyForward,
                    document.location(root),
                    &[],
                    &|leaf| boolean_leaf(&document, root, leaf, LocalVersion::LegacyForward),
                )
                .unwrap_err();
                assert_eq!(error.code, "XPST0008");
            }
        }
    }

    #[test]
    fn selected_numeric_reporting_preserves_shared_controls_and_cleanup() {
        use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
        use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;
        let sheet = |select: &str| {
            let xml = format!(
                r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="{select}"/></xsl:template></xsl:stylesheet>"#
            );
            Document::from_parsed(
                parse_document(
                    "urn:reporting-control",
                    xml.as_bytes(),
                    ParseLimits {
                        max_events: 100,
                        max_depth: 10,
                    },
                )
                .unwrap(),
            )
            .unwrap()
        };
        let selected = compile(
            &sheet("system-property('xsl:version')"),
            Edition::Legacy,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let reference = compiler::compile_stylesheet(&sheet("'1'")).unwrap();
        let (source, _) = fixture();
        let mut actual = InvocationControl::unbounded();
        let mut expected = InvocationControl::unbounded();
        assert_eq!(
            execute(&selected, &source, "selected", &mut actual).unwrap(),
            "1"
        );
        assert_eq!(
            execute(&reference, &source, "reference", &mut expected).unwrap(),
            "1"
        );
        for domain in [
            WorkDomain::XPathNodeVisit,
            WorkDomain::XPathOperation,
            WorkDomain::XsltInstruction,
            WorkDomain::XsltTemplateCandidate,
            WorkDomain::ResultNode,
            WorkDomain::ResultTextByte,
            WorkDomain::SerializedByte,
        ] {
            assert_eq!(
                actual.consumed(domain),
                expected.consumed(domain),
                "{domain:?}"
            );
        }
        for allowance in [0, 1] {
            let mut limits = WorkLimits::unbounded();
            limits.result_text_bytes = allowance;
            let run = || {
                let result = execute(
                    &selected,
                    &source,
                    "budget",
                    &mut InvocationControl::new(CancellationToken::new(), limits),
                );
                if allowance == 0 {
                    assert!(result.unwrap_err().contains("FXCT0002"));
                } else {
                    assert_eq!(result.unwrap(), "1");
                }
            };
            run();
            #[cfg(feature = "allocation-observation")]
            {
                let observed = allocation_counter::measure(run);
                assert_eq!(observed.bytes_current, 0);
                assert_eq!(observed.count_current, 0);
            }
        }
        let mut cancelled =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::ResultTextByte, 0);
        assert!(
            execute(&selected, &source, "cancel", &mut cancelled)
                .unwrap_err()
                .contains("FXCT0001")
        );
        assert_eq!(
            execute(
                &selected,
                &source,
                "reuse",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "1"
        );
    }
}
