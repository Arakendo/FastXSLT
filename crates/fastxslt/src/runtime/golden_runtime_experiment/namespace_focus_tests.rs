//! Compiled namespace focus controls; these fixtures are not corpus credit.

#[path = "namespace_copy_execution_tests.rs"]
mod namespace_copy;

#[path = "namespace_mixed_focus_tests.rs"]
mod namespace_mixed;

#[path = "namespace_conditional_focus_tests.rs"]
mod namespace_conditional;

#[path = "namespace_count_argument_tests.rs"]
mod namespace_count_arguments;

use super::super::{
    BTreeMap, Document, ExecutionFailure, InvocationControl, MultipleMatchPolicy, ParseLimits,
    StylesheetProgram, WhitespaceRepresentation, WorkDomain, execute_program_with_parameters_using,
    serialize_xml,
};
use super::ResultNode;
use crate::compile::golden_stylesheet_experiment::{CompileCategory, compile_stylesheet};
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::xml::quick_xml_experiment::parse_document;

fn document(identity: &str, xml: &str) -> Document {
    Document::from_parsed(
        parse_document(
            identity,
            xml.as_bytes(),
            ParseLimits {
                max_events: 256,
                max_depth: 16,
            },
        )
        .unwrap(),
    )
    .unwrap()
}

fn stylesheet(version: &str, body: &str) -> Document {
    document(
        "urn:namespace:style",
        &format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:strip-space elements="*"/><xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#,
        ),
    )
}

fn run(
    program: &StylesheetProgram,
    source: &Document,
    representation: WhitespaceRepresentation,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let result = execute_program_with_parameters_using(
        program,
        source,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        "namespace-focus",
        representation,
        None,
        None,
        control,
    )?;
    serialize_xml(&result, &program.output, "namespace-focus", 8192, control)
}

const BODY: &str = r#"<xsl:for-each select="/*/namespace::*"><xsl:value-of select="name()"/><xsl:text>:</xsl:text><xsl:value-of select="local-name()"/><xsl:text>:</xsl:text><xsl:value-of select="namespace-uri()"/><xsl:text>:</xsl:text><xsl:value-of select="."/><xsl:text>:</xsl:text><xsl:value-of select="position()"/><xsl:text>/</xsl:text><xsl:value-of select="last()"/><xsl:text>|</xsl:text></xsl:for-each>"#;

#[test]
fn namespace_focus_generated_identity_comparisons_use_owner_and_prefix_not_uri() {
    let source = document(
        "urn:namespace:identity",
        r#"<r xmlns:p="urn:=same" xmlns:q="urn:=same"><child/></r>"#,
    );
    for version in ["1.0", "3.0"] {
        for (left, right, same) in [
            ("r/namespace::p", "r/namespace::p", true),
            ("r/namespace::p", "r/namespace::q", false),
            ("r/namespace::p", "r/child/namespace::p", false),
            ("r/namespace::p", "r", false),
            ("r/namespace::missing", "r/namespace::absent", true),
            ("r/namespace::missing", "r/namespace::p", false),
            (
                "r/namespace::p[string(.)='urn:=same']",
                "r/namespace::p",
                true,
            ),
        ] {
            for (operator, expected) in [("=", same), ("!=", !same)] {
                let body = format!(
                    r#"<xsl:value-of select="generate-id({left}) {operator} generate-id({right})"/>"#
                );
                let program = compile_stylesheet(&stylesheet(version, &body)).unwrap();
                for representation in [
                    WhitespaceRepresentation::VisibilityView,
                    WhitespaceRepresentation::CompleteReference,
                ] {
                    assert_eq!(
                        run(
                            &program,
                            &source,
                            representation,
                            &mut InvocationControl::unbounded()
                        )
                        .unwrap(),
                        expected.to_string()
                    );
                }
            }
        }
    }
}

#[test]
fn namespace_focus_generated_identity_preserves_cardinality_controls_and_recovery() {
    let source = document("urn:namespace:identity", r#"<r xmlns:p="urn:p"/>"#);
    let body =
        r#"<xsl:value-of select="generate-id(r/namespace::*) = generate-id(r/namespace::p)"/>"#;
    let legacy = compile_stylesheet(&stylesheet("1.0", body)).unwrap();
    assert_eq!(
        run(
            &legacy,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "true"
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
    for version in ["1.0", "3.0"] {
        let program = compile_stylesheet(&stylesheet(
            version,
            r#"<xsl:value-of select="generate-id(r/namespace::p) != generate-id(r)"/>"#,
        ))
        .unwrap();
        for domain in [
            WorkDomain::XPathOperation,
            WorkDomain::XPathNodeVisit,
            WorkDomain::ResultTextByte,
        ] {
            let mut control = InvocationControl::unbounded()
                .cancelling_on_charge(domain, usize::from(domain != WorkDomain::ResultTextByte));
            assert_eq!(
                run(
                    &program,
                    &source,
                    WhitespaceRepresentation::VisibilityView,
                    &mut control
                )
                .unwrap_err()
                .code,
                "FXCT0001"
            );
        }
        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = 2;
        let error = run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        )
        .unwrap_err();
        assert_eq!(error.code, "FXCT0002");
        assert_eq!(error.work_domain, Some(WorkDomain::XPathOperation));
        assert_eq!(
            run(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "true"
        );
    }
}

#[test]
fn namespace_focus_literal_attribute_avts_preserve_actual_focus_and_owned_values() {
    for version in ["1.0", "3.0"] {
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            let result = {
                let source = document(
                    "urn:namespace:literal-avts",
                    r#"<r xmlns="urn:d" xmlns:p="urn:p"><child xmlns:p="urn:q"/></r>"#,
                );
                let body = r#"<xsl:for-each select="/*/*/namespace::*"><out n="{name(.)}" local="{local-name()}" value="{.}" pos="{position()}" size="{last()}" fixed="literal"/></xsl:for-each>"#;
                let program = compile_stylesheet(&stylesheet(version, body)).unwrap();
                execute_program_with_parameters_using(
                    &program,
                    &source,
                    &BTreeMap::new(),
                    MultipleMatchPolicy::UseLast,
                    "namespace-focus",
                    representation,
                    None,
                    None,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap()
            };
            assert_eq!(result.children.len(), 3);
            for (index, (item, (prefix, uri))) in result
                .children
                .iter()
                .zip([
                    ("", "urn:d"),
                    ("p", "urn:q"),
                    ("xml", "http://www.w3.org/XML/1998/namespace"),
                ])
                .enumerate()
            {
                let ResultNode::Element {
                    attributes,
                    children,
                    ..
                } = item
                else {
                    panic!("literal result element")
                };
                assert!(children.is_empty());
                assert_eq!(attributes.len(), 6);
                for (name, expected) in [
                    ("n", prefix.to_owned()),
                    ("local", prefix.to_owned()),
                    ("value", uri.to_owned()),
                    ("pos", (index + 1).to_string()),
                    ("size", "3".to_owned()),
                    ("fixed", "literal".to_owned()),
                ] {
                    let attribute = attributes
                        .iter()
                        .find(|attr| attr.name.local == name)
                        .unwrap();
                    assert_eq!(attribute.name.namespace, None);
                    assert_eq!(attribute.value, expected);
                }
            }
        }
    }
}

#[test]
fn namespace_focus_literal_name_avts_charge_before_copy_and_recover_after_failure() {
    let source = document("urn:namespace:literal-avts", r#"<r xmlns:p="urn:p"/>"#);
    for version in ["1.0", "3.0"] {
        let program = compile_stylesheet(&stylesheet(version,
            r#"<xsl:for-each select="r/namespace::p"><out n="{name()}" local="{local-name()}"/><xsl:value-of select="name()"/></xsl:for-each>"#,
        )).unwrap();
        for domain in [WorkDomain::ResultTextByte, WorkDomain::XPathNodeVisit] {
            let mut control = InvocationControl::unbounded().cancelling_on_charge(domain, 0);
            assert_eq!(
                run(
                    &program,
                    &source,
                    WhitespaceRepresentation::VisibilityView,
                    &mut control
                )
                .unwrap_err()
                .code,
                "FXCT0001"
            );
        }
        let mut limits = WorkLimits::unbounded();
        limits.result_text_bytes = 1;
        let error = run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        )
        .unwrap_err();
        assert_eq!(error.code, "FXCT0002");
        assert_eq!(error.work_domain, Some(WorkDomain::ResultTextByte));
        assert_eq!(
            run(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "p"
        );
    }
}

#[test]
fn namespace_focus_constructs_ordinary_attributes_with_owned_names_and_values() {
    for version in ["1.0", "3.0"] {
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            let result = {
                let source = document("urn:namespace:attributes", r#"<r xmlns:p="urn:p"/>"#);
                let body = r#"<out><xsl:for-each select="r/namespace::node()"><xsl:sort select="name()"/><xsl:attribute name="{name(.)}"><xsl:value-of select="."/></xsl:attribute></xsl:for-each></out>"#;
                let program = compile_stylesheet(&stylesheet(version, body)).unwrap();
                execute_program_with_parameters_using(
                    &program,
                    &source,
                    &BTreeMap::new(),
                    MultipleMatchPolicy::UseLast,
                    "namespace-focus",
                    representation,
                    None,
                    None,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap()
            };
            let ResultNode::Element {
                attributes,
                children,
                ..
            } = &result.children[0]
            else {
                panic!("result element")
            };
            assert!(children.is_empty());
            assert_eq!(attributes.len(), 2);
            assert_eq!(attributes[0].name.local, "p");
            assert_eq!(attributes[0].name.namespace, None);
            assert_eq!(attributes[0].value, "urn:p");
            assert_eq!(attributes[1].name.local, "xml");
            assert_eq!(attributes[1].name.namespace, None);
            assert_eq!(attributes[1].value, "http://www.w3.org/XML/1998/namespace");
        }
    }
}

#[test]
fn namespace_focus_attribute_errors_and_control_failures_preserve_shared_rules() {
    let source = document("urn:namespace:attributes", r#"<r xmlns:p="urn:p"/>"#);
    let body = r#"<out><xsl:for-each select="r/namespace::p"><xsl:attribute name="{name()}"><xsl:value-of select="."/></xsl:attribute></xsl:for-each></out>"#;
    for version in ["1.0", "3.0"] {
        let program = compile_stylesheet(&stylesheet(version, body)).unwrap();
        for domain in [
            WorkDomain::ResultNode,
            WorkDomain::XdmStringValueNode,
            WorkDomain::ResultTextByte,
        ] {
            let mut cancelled = InvocationControl::unbounded().cancelling_on_charge(domain, 1);
            // Use the first string visit, but fail after the wrapper's result-node charge.
            if domain != WorkDomain::ResultNode {
                cancelled = InvocationControl::unbounded().cancelling_on_charge(domain, 0);
            }
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
        let mut limits = WorkLimits::unbounded();
        limits.result_nodes = 1;
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
        assert!(
            run(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::unbounded()
            )
            .is_ok()
        );
        let mut limits = WorkLimits::unbounded();
        limits.result_text_bytes = 5;
        let error = run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        )
        .unwrap_err();
        assert_eq!(error.code, "FXCT0002");
        assert_eq!(error.work_domain, Some(WorkDomain::ResultTextByte));
        let mut control = InvocationControl::unbounded();
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut control,
        )
        .unwrap();
        assert_eq!(control.consumed(WorkDomain::ResultTextByte), 6);
    }
    let body = r#"<out>child<xsl:for-each select="r/namespace::p"><xsl:attribute name="{name()}"><xsl:value-of select="."/></xsl:attribute></xsl:for-each></out>"#;
    let program = compile_stylesheet(&stylesheet("3.0", body)).unwrap();
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "XTDE0410"
    );
    let source = document("urn:namespace:attributes", r#"<r xmlns="urn:default"/>"#);
    let body = r#"<out><xsl:for-each select="/*/namespace::*[1]"><xsl:attribute name="{name()}">v</xsl:attribute></xsl:for-each></out>"#;
    let program = compile_stylesheet(&stylesheet("3.0", body)).unwrap();
    let error = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(error.code, "XTDE0850");
    assert_eq!(error.location.unwrap().resource, "urn:namespace:style");
}

#[test]
fn namespace_focus_sorting_uses_shared_controls_stability_and_sorted_focus() {
    let source = document("urn:namespace:sort", r#"<r xmlns:p="2" xmlns:q="1"/>"#);
    for version in ["1.0", "3.0"] {
        for (sort, expected) in [
            (
                r#"<xsl:sort select="name(.)" order="descending"/>"#,
                "xml:1/3;q:2/3;p:3/3;",
            ),
            (
                r#"<xsl:sort select="local-name()"/>"#,
                "p:1/3;q:2/3;xml:3/3;",
            ),
            (
                r#"<xsl:sort select="position()" order="descending"/>"#,
                "xml:1/3;q:2/3;p:3/3;",
            ),
            (r#"<xsl:sort select="last()"/>"#, "p:1/3;q:2/3;xml:3/3;"),
            (r#"<xsl:sort select="'same'"/>"#, "p:1/3;q:2/3;xml:3/3;"),
            (
                r#"<xsl:sort select="." data-type="number"/>"#,
                "xml:1/3;q:2/3;p:3/3;",
            ),
            (
                r#"<xsl:sort select="last()"/><xsl:sort select="name()" order="descending"/>"#,
                "xml:1/3;q:2/3;p:3/3;",
            ),
        ] {
            let body = format!(
                r#"<xsl:for-each select="r/namespace::*">{sort}<xsl:value-of select="name()"/>:<xsl:value-of select="position()"/>/<xsl:value-of select="last()"/>;</xsl:for-each>"#
            );
            let program = compile_stylesheet(&stylesheet(version, &body)).unwrap();
            for representation in [
                WhitespaceRepresentation::VisibilityView,
                WhitespaceRepresentation::CompleteReference,
            ] {
                assert_eq!(
                    run(
                        &program,
                        &source,
                        representation,
                        &mut InvocationControl::unbounded()
                    )
                    .unwrap(),
                    expected
                );
            }
        }
    }
}

#[test]
fn namespace_focus_sort_and_construct_failures_discard_results_and_recover() {
    let source = document("urn:namespace:sort", r#"<r xmlns:p="urn:p"/>"#);
    let body = r#"<xsl:for-each select="r/namespace::*"><xsl:sort select="name()"/><xsl:element name="{name()}"><xsl:value-of select="."/></xsl:element></xsl:for-each>"#;
    let program = compile_stylesheet(&stylesheet("1.0", body)).unwrap();
    for domain in [
        WorkDomain::XPathOperation,
        WorkDomain::ResultNode,
        WorkDomain::XdmStringValueNode,
    ] {
        let mut cancelled = InvocationControl::unbounded().cancelling_on_charge(domain, 0);
        let error = run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut cancelled,
        )
        .unwrap_err();
        assert_eq!(error.code, "FXCT0001");
        assert_eq!(error.work_domain, Some(domain));
    }
    let mut limits = WorkLimits::unbounded();
    limits.result_nodes = 1;
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
    assert!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .is_ok()
    );
    let source = document("urn:namespace:empty-prefix", r#"<r xmlns="urn:default"/>"#);
    let body = r#"<xsl:for-each select="/*/namespace::*[1]"><xsl:element name="{name()}"/></xsl:for-each>"#;
    let program = compile_stylesheet(&stylesheet("3.0", body)).unwrap();
    let error = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(error.code, "XTDE0820");
    assert_eq!(error.location.unwrap().resource, "urn:namespace:style");
}

#[test]
fn namespace_focus_result_elements_preserve_namespace_focus_and_own_the_result() {
    let body = r#"<xsl:for-each select="r/namespace::p"><wrapper><xsl:element name="{name(.)}"><xsl:value-of select="."/>|<xsl:value-of select="position()"/>/<xsl:value-of select="last()"/></xsl:element><xsl:value-of select="name()"/></wrapper></xsl:for-each>"#;
    for version in ["1.0", "3.0"] {
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            let result = {
                let source = document("urn:namespace:construct", r#"<r xmlns:p="urn:p"><a/></r>"#);
                let program = compile_stylesheet(&stylesheet(version, body)).unwrap();
                execute_program_with_parameters_using(
                    &program,
                    &source,
                    &BTreeMap::new(),
                    MultipleMatchPolicy::UseLast,
                    "namespace-focus",
                    representation,
                    None,
                    None,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap()
            };
            let ResultNode::Element { name, children, .. } = &result.children[0] else {
                panic!("literal wrapper")
            };
            assert_eq!(name.local, "wrapper");
            let ResultNode::Element { name, children, .. } = &children[0] else {
                panic!("computed element")
            };
            assert_eq!(name.local, "p");
            assert_eq!(name.namespace, None);
            assert!(matches!(&children[0], ResultNode::Text(value) if value == "urn:p|1/1"));
        }
    }
}

#[test]
fn namespace_focus_integer_positions_are_per_owner_after_the_name_test() {
    let source = document(
        "urn:namespace:positions",
        r#"<r xmlns:p="urn:p"><a xmlns="urn:default"/><a/></r>"#,
    );
    let body = r#"<xsl:for-each select="/*/*/namespace::*[1]"><xsl:value-of select="name()"/>:<xsl:value-of select="position()"/>/<xsl:value-of select="last()"/>;</xsl:for-each>|<xsl:for-each select="/*/*/namespace::xml[1]"><xsl:value-of select="name()"/>;</xsl:for-each>|<xsl:value-of select="count(/*/*/namespace::xml[2])"/>|<xsl:value-of select="count(/*/*/namespace::*[0])"/>|<xsl:value-of select="count(/*/*/namespace::*[999])"/>|<xsl:value-of select="name(/*/namespace::*[01])"/>"#;
    for version in ["1.0", "3.0"] {
        let program = compile_stylesheet(&stylesheet(version, body)).unwrap();
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            assert_eq!(
                run(
                    &program,
                    &source,
                    representation,
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                ":1/2;p:2/2;|xml;xml;|0|0|0|p"
            );
        }
    }
}

#[test]
fn namespace_focus_integer_predicates_propagate_control_failure_and_recover() {
    let source = document("urn:namespace:positions", r#"<r xmlns:p="urn:p"/>"#);
    let body = r#"<xsl:for-each select="r/namespace::*[1]"><xsl:value-of select="name()"/></xsl:for-each>"#;
    let program = compile_stylesheet(&stylesheet("1.0", body)).unwrap();
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 0);
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
    let mut limits = WorkLimits::unbounded();
    limits.xpath_operations = 0;
    let error = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::new(CancellationToken::new(), limits),
    )
    .unwrap_err();
    assert_eq!(error.code, "FXCT0002");
    assert_eq!(error.work_domain, Some(WorkDomain::XPathOperation));
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "p"
    );
}

#[test]
fn namespace_focus_names_values_positions_and_views_use_the_actual_node() {
    let source = document(
        "urn:namespace:source",
        r#"<r xmlns="urn:default" xmlns:p="urn:p"> <child/> </r>"#,
    );
    for version in ["1.0", "3.0"] {
        let program = compile_stylesheet(&stylesheet(version, BODY)).unwrap();
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            assert_eq!(
                run(
                    &program,
                    &source,
                    representation,
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                ":::urn:default:1/3|p:p::urn:p:2/3|xml:xml::http://www.w3.org/XML/1998/namespace:3/3|"
            );
        }
    }
}

#[test]
fn namespace_focus_scalar_paths_and_filters_preserve_names_not_binding_uris() {
    let body = r#"<xsl:value-of select="count(/*/namespace::*[name(.)!='xml'])"/>|<xsl:value-of select="name(/*/namespace::*[string()='urn:p/with/path'])"/>|<xsl:value-of select="local-name(/*/namespace::*[string()='urn:default'])"/>|<xsl:value-of select="namespace-uri(/*/namespace::*[string()='urn:p/with/path'])"/>|<xsl:value-of select="name(/*/namespace::missing)"/>|<xsl:for-each select="/*/namespace::*[namespace-uri(.)='']"><xsl:value-of select="name()"/>:<xsl:value-of select="position()"/>/<xsl:value-of select="last()"/>;</xsl:for-each>"#;
    let source = document(
        "urn:namespace:source",
        r#"<r xmlns="urn:default" xmlns:p="urn:p/with/path"> <child/> </r>"#,
    );
    for version in ["1.0", "3.0"] {
        let program = compile_stylesheet(&stylesheet(version, body)).unwrap();
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            assert_eq!(
                run(
                    &program,
                    &source,
                    representation,
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                "2|p||||:1/3;p:2/3;xml:3/3;"
            );
        }
    }
}

#[test]
fn namespace_focus_modern_cardinality_and_legacy_first_node_are_distinct() {
    let source = document("urn:namespace:source", r#"<r xmlns:p="urn:p"/>"#);
    for function in ["name", "local-name", "namespace-uri"] {
        let body = format!(r#"<xsl:value-of select="{function}(r/namespace::*)"/>"#);
        let modern = compile_stylesheet(&stylesheet("3.0", &body)).unwrap();
        let error = run(
            &modern,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded(),
        )
        .unwrap_err();
        assert_eq!(error.code, "XPTY0004");
        assert_eq!(
            error.location.as_ref().unwrap().resource,
            "urn:namespace:style"
        );
        let legacy = compile_stylesheet(&stylesheet("1.0", &body)).unwrap();
        assert_eq!(
            run(
                &legacy,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            if function == "namespace-uri" { "" } else { "p" }
        );
    }
}

#[test]
fn namespace_focus_predicate_control_failures_are_not_false_matches() {
    let program = compile_stylesheet(&stylesheet(
        "1.0",
        r#"<xsl:value-of select="count(r/namespace::*[string()='urn:p'])"/>"#,
    ))
    .unwrap();
    let source = document("urn:namespace:source", r#"<r xmlns:p="urn:p"/>"#);
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmStringValueNode, 0);
    let error = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut cancelled,
    )
    .unwrap_err();
    assert_eq!(error.code, "FXCT0001");
    let mut limits = WorkLimits::unbounded();
    limits.xdm_string_value_nodes = 0;
    let error = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::new(CancellationToken::new(), limits),
    )
    .unwrap_err();
    assert_eq!(error.code, "FXCT0002");
    assert_eq!(error.work_domain, Some(WorkDomain::XdmStringValueNode));
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "1"
    );
}

#[test]
fn namespace_focus_named_selection_string_function_and_empty_axis() {
    let body = r#"<xsl:for-each select="r/namespace::p"><xsl:value-of select="string(.)"/></xsl:for-each><xsl:for-each select="r/@a/namespace::*"><xsl:text>wrong</xsl:text></xsl:for-each><xsl:value-of select="name()"/>"#;
    for version in ["1.0", "3.0"] {
        let program = compile_stylesheet(&stylesheet(version, body)).unwrap();
        let source = document("urn:namespace:source", r#"<r xmlns:p="urn:p" a="value"/>"#);
        assert_eq!(
            run(
                &program,
                &source,
                WhitespaceRepresentation::VisibilityView,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "urn:p"
        );
    }
}

#[test]
fn namespace_focus_parent_scalars_preserve_owner_names_effective_values_and_outer_focus() {
    let body = r#"<xsl:for-each select="/*/*/namespace::p"><xsl:value-of select="name()"/>|<xsl:value-of select="name(..)"/>|<xsl:value-of select="local-name(..)"/>|<xsl:value-of select="namespace-uri(..)"/>|<xsl:value-of select=".."/>|<xsl:value-of select="string(..)"/>|<xsl:value-of select="parent::node()"/>|<xsl:value-of select="name()"/>|<xsl:value-of select="position()"/>/<xsl:value-of select="last()"/></xsl:for-each>"#;
    for version in ["1.0", "3.0"] {
        let program = compile_stylesheet(&stylesheet(version, body)).unwrap();
        for (xml, value) in [
            (
                r#"<r xmlns:o="urn:owner"><o:leaf xmlns:p="urn:binding"> <b>A</b> <b>B</b> </o:leaf></r>"#,
                "AB",
            ),
            (
                r#"<r xmlns:o="urn:owner"><o:leaf xmlns:p="urn:binding" xml:space="preserve"> <b>A</b> <b>B</b> </o:leaf></r>"#,
                " A B ",
            ),
        ] {
            let source = document("urn:namespace:parent-source", xml);
            for representation in [
                WhitespaceRepresentation::VisibilityView,
                WhitespaceRepresentation::CompleteReference,
            ] {
                assert_eq!(
                    run(
                        &program,
                        &source,
                        representation,
                        &mut InvocationControl::unbounded()
                    )
                    .unwrap(),
                    format!("p|o:leaf|leaf|urn:owner|{value}|{value}|{value}|p|1/1")
                );
            }
        }
    }
}

#[test]
fn namespace_focus_parent_scalar_charge_failures_do_not_mutate_prepared_input() {
    let body =
        r#"<xsl:for-each select="r/namespace::p"><xsl:value-of select=".."/></xsl:for-each>"#;
    let program = compile_stylesheet(&stylesheet("1.0", body)).unwrap();
    let source = document(
        "urn:namespace:parent-source",
        r#"<r xmlns:p="urn:p">value</r>"#,
    );
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmStringValueNode, 0);
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
    let mut limits = WorkLimits::unbounded();
    limits.xdm_string_value_nodes = 0;
    let error = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut InvocationControl::new(CancellationToken::new(), limits),
    )
    .unwrap_err();
    assert_eq!(error.code, "FXCT0002");
    assert_eq!(error.work_domain, Some(WorkDomain::XdmStringValueNode));
    assert_eq!(
        run(
            &program,
            &source,
            WhitespaceRepresentation::VisibilityView,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "value"
    );
}

#[test]
fn namespace_focus_unmigrated_consumers_stay_explicitly_unsupported() {
    for body in [
        r#"<xsl:sort select="../@n"/>"#,
        r#"<xsl:copy-of select="."/>"#,
        r#"<xsl:variable name="n" select="."/>"#,
        r#"<xsl:for-each select=".."><xsl:value-of select="name()"/></xsl:for-each>"#,
        r#"<xsl:value-of select="../*"/>"#,
        r#"<out name="{../@name}"/>"#,
    ] {
        let style = stylesheet(
            "1.0",
            &format!(r#"<xsl:for-each select="r/namespace::*">{body}</xsl:for-each>"#),
        );
        let error = compile_stylesheet(&style).unwrap_err();
        assert_eq!(error.category, CompileCategory::Unsupported, "{body}");
        assert_eq!(error.location.resource, "urn:namespace:style");
    }
}

#[test]
fn namespace_focus_budget_cancellation_cleanup_and_concurrent_reuse() {
    let source = document("urn:namespace:source", r#"<r xmlns:p="urn:p"/>"#);
    let program = compile_stylesheet(&stylesheet("1.0", BODY)).unwrap();
    let mut limits = WorkLimits::unbounded();
    limits.xpath_node_visits = 1;
    let mut bounded = InvocationControl::new(CancellationToken::new(), limits);
    let error = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut bounded,
    )
    .unwrap_err();
    assert_eq!(error.code, "FXCT0002");
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmStringValueNode, 0);
    let error = run(
        &program,
        &source,
        WhitespaceRepresentation::VisibilityView,
        &mut cancelled,
    )
    .unwrap_err();
    assert_eq!(error.code, "FXCT0001");
    assert_eq!(error.request_id.as_deref(), Some("namespace-focus"));
    std::thread::scope(|scope| {
        let jobs: Vec<_> = (0..4)
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
            .collect();
        for job in jobs {
            assert_eq!(
                job.join().unwrap(),
                "p:p::urn:p:1/2|xml:xml::http://www.w3.org/XML/1998/namespace:2/2|"
            );
        }
    });
}
