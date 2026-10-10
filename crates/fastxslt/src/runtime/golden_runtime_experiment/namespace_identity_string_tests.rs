//! Source-qualified identity strings share axis selection and the ordinary evaluator.

use super::*;
use std::fmt::Write;

#[test]
fn namespace_identity_strings_position_selection_normalizes_before_filtering() {
    let source = document(
        "urn:namespace:positions",
        r#"<r xmlns:p="urn:p"> <child/> </r>"#,
    );
    let mut body = String::new();
    for (position, path) in [
        (1, "r/namespace::p"),
        (2, "r/namespace::xml"),
        (3, "r/child/namespace::p"),
        (4, "r/child/namespace::xml"),
    ] {
        write!(body, r#"<xsl:variable name="pos{position}" select="{position}"/><xsl:variable name="id{position}" select="generate-id((descendant-or-self::*/namespace::*)[position()=$pos{position}])"/><xsl:value-of select="$id{position}"/><xsl:text>|</xsl:text><xsl:value-of select="generate-id({path})"/><xsl:text>|</xsl:text>"#).unwrap();
    }
    let program = compile_stylesheet(&stylesheet("1.0", &body)).unwrap();
    let expected = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let parts = expected.split('|').collect::<Vec<_>>();
    for index in (0..8).step_by(2) {
        assert!(!parts[index].is_empty());
        assert_eq!(parts[index], parts[index + 1]);
    }
    assert_ne!(parts[0], parts[4]);
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::CompleteReference,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
    std::thread::scope(|scope| {
        let handles = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    run(
                        &program,
                        &source,
                        WhitespaceRepresentation::VisibilityView,
                        &mut InvocationControl::unbounded(),
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            assert_eq!(handle.join().unwrap(), expected);
        }
    });
}

#[test]
fn namespace_identity_strings_position_selection_preserves_conversion_and_controls() {
    let source = document("urn:namespace:positions", r#"<r xmlns:p="urn:p"/>"#);
    for position in ["0", "-1", "1.5", "99", "'NaN'"] {
        let body = format!(
            r#"<xsl:variable name="pos" select="{position}"/><xsl:value-of select="generate-id((r/namespace::*)[position()=$pos])"/>"#
        );
        let program = compile_stylesheet(&stylesheet("1.0", &body)).unwrap();
        assert_eq!(
            run(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "",
            "{position}"
        );
    }
    let body = r#"<xsl:variable name="pos" select="'1'"/><xsl:value-of select="generate-id((r/namespace::*)[position()=$pos])"/>"#;
    let program = compile_stylesheet(&stylesheet("1.0", body)).unwrap();
    let mut measured = InvocationControl::unbounded();
    let expected = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut measured,
    )
    .unwrap();
    assert!(!expected.is_empty());
    let charge = measured.consumed(WorkDomain::XPathOperation);
    let mut limits = WorkLimits::unbounded();
    limits.xpath_operations = charge;
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::new(CancellationToken::new(), limits)
        )
        .unwrap(),
        expected
    );
    limits.xpath_operations -= 1;
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::new(CancellationToken::new(), limits)
        )
        .unwrap_err()
        .code,
        "FXCT0002"
    );
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 0);
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut cancelled
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
}

#[test]
fn namespace_identity_strings_position_selection_keeps_explicit_boundaries() {
    let source = document("urn:namespace:positions", r#"<r att="1" xmlns:p="urn:p"/>"#);
    for (binding, code) in [
        ("", "FXRT0002"),
        (r#"<xsl:variable name="pos" select="r/@att"/>"#, "FXRT0007"),
    ] {
        let body = format!(
            r#"{binding}<xsl:value-of select="generate-id((r/namespace::*)[position()=$pos])"/>"#
        );
        let program = compile_stylesheet(&stylesheet("1.0", &body)).unwrap();
        let error = run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded(),
        )
        .unwrap_err();
        assert_eq!(error.code, code);
        assert_eq!(error.location.unwrap().resource, "urn:namespace:style");
    }
    let body = r#"<xsl:variable name="pos" select="1"/><xsl:value-of select="generate-id((r/namespace::*)[position()=$pos])"/>"#;
    assert_eq!(
        compile_stylesheet(&stylesheet("3.0", body))
            .unwrap_err()
            .category,
        CompileCategory::Unsupported
    );
    let mixed = r#"<xsl:variable name="pos" select="1"/><xsl:value-of select="generate-id((r | r/namespace::*)[position()=$pos])"/>"#;
    assert_eq!(
        compile_stylesheet(&stylesheet("1.0", mixed))
            .unwrap_err()
            .category,
        CompileCategory::Unsupported
    );
}

#[test]
fn namespace_identity_strings_preserve_owner_prefix_and_tree_disjointness() {
    let source = document(
        "urn:namespace:ids",
        r#"<r xmlns:p="urn:same" xmlns:q="urn:same" xmlns:é="urn:same"> <child/> </r>"#,
    );
    let mut body = String::new();
    for path in [
        "r/namespace::p",
        "r/namespace::p",
        "r/namespace::q",
        "r/child/namespace::p",
        "r/namespace::é",
        "r",
        "r/namespace::absent",
        "r | r/namespace::p",
    ] {
        write!(
            body,
            r#"<xsl:value-of select="generate-id({path})"/><xsl:text>|</xsl:text>"#
        )
        .unwrap();
    }
    let program = compile_stylesheet(&stylesheet("1.0", &body)).unwrap();
    let expected = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let parts = expected.split('|').collect::<Vec<_>>();
    assert_eq!(parts[0], parts[1]);
    for left in [0, 2, 3, 4, 5] {
        for right in [0, 2, 3, 4, 5] {
            assert_eq!(parts[left] == parts[right], left == right);
        }
        assert!(parts[left].bytes().all(|byte| byte.is_ascii_alphanumeric()));
        assert!(parts[left].starts_with(|character: char| character.is_ascii_alphabetic()));
    }
    assert_eq!(parts[6], "");
    assert_eq!(parts[7], parts[5]);
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::CompleteReference,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
    std::thread::scope(|scope| {
        let handles = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    run(
                        &program,
                        &source,
                        WhitespaceRepresentation::VisibilityView,
                        &mut InvocationControl::unbounded(),
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            assert_eq!(handle.join().unwrap(), expected);
        }
    });
}

#[test]
fn namespace_identity_strings_keep_legacy_first_node_and_modern_cardinality() {
    let source = document("urn:namespace:ids", r#"<r xmlns:p="urn:p"/>"#);
    let body = r#"<xsl:value-of select="generate-id(r/namespace::*)"/>"#;
    let legacy = compile_stylesheet(&stylesheet("1.0", body)).unwrap();
    let single = compile_stylesheet(&stylesheet(
        "3.0",
        r#"<xsl:value-of select="generate-id(r/namespace::p)"/>"#,
    ))
    .unwrap();
    let expected = run(
        &single,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(
        run(
            &legacy,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
    let modern = compile_stylesheet(&stylesheet("3.0", body)).unwrap();
    let error = run(
        &modern,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(error.code, "XPTY0004");
    assert_eq!(error.location.unwrap().resource, "urn:namespace:style");
    let local = compile_stylesheet(&stylesheet(
        "3.0",
        &format!(r#"<legacy xsl:version="1.0">{body}</legacy>"#),
    ))
    .unwrap();
    assert_eq!(
        run(
            &local,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
}

#[test]
fn namespace_identity_strings_preserve_charges_cancellation_and_recovery() {
    let source = document("urn:namespace:ids", r#"<r xmlns:p="urn:p"/>"#);
    let program = compile_stylesheet(&stylesheet(
        "1.0",
        r#"<xsl:value-of select="generate-id(r/namespace::p)"/>"#,
    ))
    .unwrap();
    let mut measured = InvocationControl::unbounded();
    let expected = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut measured,
    )
    .unwrap();
    for domain in [
        WorkDomain::XPathOperation,
        WorkDomain::XPathNodeVisit,
        WorkDomain::XdmNode,
        WorkDomain::ResultTextByte,
    ] {
        let charge = measured.consumed(domain);
        assert!(charge > 0);
        let mut limits = WorkLimits::unbounded();
        match domain {
            WorkDomain::XPathOperation => limits.xpath_operations = charge,
            WorkDomain::XPathNodeVisit => limits.xpath_node_visits = charge,
            WorkDomain::XdmNode => limits.xdm_nodes = charge,
            WorkDomain::ResultTextByte => limits.result_text_bytes = charge,
            _ => unreachable!(),
        }
        assert_eq!(
            run(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::new(CancellationToken::new(), limits)
            )
            .unwrap(),
            expected
        );
        match domain {
            WorkDomain::XPathOperation => limits.xpath_operations -= 1,
            WorkDomain::XPathNodeVisit => limits.xpath_node_visits -= 1,
            WorkDomain::XdmNode => limits.xdm_nodes -= 1,
            WorkDomain::ResultTextByte => limits.result_text_bytes -= 1,
            _ => unreachable!(),
        }
        let error = run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        )
        .unwrap_err();
        assert_eq!(error.code, "FXCT0002");
        assert_eq!(error.work_domain, Some(domain));
        let mut cancelled = InvocationControl::unbounded().cancelling_on_charge(domain, 0);
        assert_eq!(
            run(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut cancelled
            )
            .unwrap_err()
            .code,
            "FXCT0001"
        );
    }
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
}

#[test]
fn namespace_identity_strings_compose_without_weakening_unsupported_contexts() {
    let source = document("urn:namespace:ids", r#"<r xmlns:p="urn:p"><child/></r>"#);
    let body = r#"<xsl:value-of select="contains(concat('+',generate-id(r/namespace::p),'+'),concat('+',generate-id(r/namespace::p),'+'))"/><xsl:text>|</xsl:text><xsl:value-of select="contains(concat('+',generate-id(r/namespace::p),'+'),concat('+',generate-id(r/child/namespace::p),'+'))"/>"#;
    let program = compile_stylesheet(&stylesheet("1.0", body)).unwrap();
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "true|false"
    );
    let temporary = compile_stylesheet(&stylesheet("1.0", r#"<xsl:variable name="tree"><leaf/></xsl:variable><xsl:for-each select="$tree/leaf"><xsl:value-of select="generate-id(namespace::xml)"/></xsl:for-each>"#)).unwrap();
    assert_eq!(
        run(
            &temporary,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .category,
        super::super::FailureCategory::Unsupported
    );
    let temporary_concat = compile_stylesheet(&stylesheet("1.0", r#"<xsl:variable name="tree"><leaf/></xsl:variable><xsl:for-each select="$tree/leaf"><xsl:value-of select="concat('+',generate-id(namespace::xml),'+')"/></xsl:for-each>"#)).unwrap();
    assert_eq!(
        run(
            &temporary_concat,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .category,
        super::super::FailureCategory::Unsupported
    );
}

#[test]
fn namespace_identity_strings_reject_secondary_origins_without_aliasing_principal_ids() {
    use crate::resources::{ResourceLimits, ResourceSetBuilder};
    let source = document("urn:namespace:ids", r#"<r xmlns:p="urn:p"/>"#);
    let body = r#"<xsl:value-of select="generate-id(r/namespace::p)"/><xsl:for-each select="document('urn:namespace:external')/r"><xsl:value-of select="generate-id(namespace::p)"/></xsl:for-each>"#;
    let program = compile_stylesheet(&stylesheet("1.0", body)).unwrap();
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources
        .admit(
            "urn:namespace:external",
            br#"<r xmlns:p="urn:p"/>"#.to_vec(),
        )
        .unwrap();
    let snapshot = resources.seal();
    let error = execute_program_with_parameters_using(
        &program,
        &source,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        "namespace-secondary",
        WhitespaceRepresentation::VisibilityView,
        Some(&snapshot),
        None,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(error.code, "FXRT0007");
    assert_eq!(error.category, super::super::FailureCategory::Unsupported);
    assert_eq!(error.request_id.as_deref(), Some("namespace-secondary"));
    assert_eq!(error.location.unwrap().resource, "urn:namespace:style");
}
