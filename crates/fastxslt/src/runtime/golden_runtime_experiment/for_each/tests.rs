//! Hand-authored temporary-path golden behavior, not a corpus conformance claim.

use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

use super::super::{FailureCategory, compile_resource, execute_program, serialize_xml};

const STYLE_ID: &str = "urn:golden:temporary-path-for-each:style";
const STYLE: &[u8] =
    include_bytes!("../../../../../../corpus/golden/temporary-path-for-each/stylesheet.xsl");
const EXPECTED: &str =
    include_str!("../../../../../../corpus/golden/temporary-path-for-each/expected.xml");

fn program(bytes: &[u8]) -> StylesheetProgram {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    builder.admit(STYLE_ID, bytes.to_vec()).unwrap();
    compile_resource(&builder.seal(), STYLE_ID).unwrap()
}

fn source() -> Document {
    Document::from_parsed(
        parse_document(
            "urn:golden:temporary-path-for-each:source",
            include_bytes!("../../../../../../corpus/golden/temporary-path-for-each/input.xml"),
            ParseLimits {
                max_events: 100,
                max_depth: 16,
            },
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn temporary_path_for_each_resets_nested_focus_and_preserves_empty_selection() {
    let program = program(STYLE);
    let source = source();
    let mut control = InvocationControl::unbounded();
    let result = execute_program(&program, &source, "temporary-path", &mut control).unwrap();
    drop(source);
    assert_eq!(
        serialize_xml(
            &result,
            &program.output,
            "temporary-path",
            8192,
            &mut control
        )
        .unwrap(),
        EXPECTED.trim()
    );
}

#[test]
fn temporary_path_for_each_reports_sorted_boundary_with_provenance() {
    let style = std::str::from_utf8(STYLE).unwrap().replace(
        "<xsl:for-each select=\"$g/box/item\">",
        "<xsl:for-each select=\"$g/box/item\"><xsl:sort select=\".\"/>",
    );
    let failure = execute_program(
        &program(style.as_bytes()),
        &source(),
        "sorted",
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(failure.code, "FXRT0007");
    assert_eq!(failure.category, FailureCategory::Unsupported);
    assert_eq!(failure.request_id.as_deref(), Some("sorted"));
    assert_eq!(failure.location.unwrap().resource, STYLE_ID);
}

#[test]
fn temporary_path_for_each_charges_visits_and_recovers_after_exhaustion() {
    let program = program(STYLE);
    let source = source();
    let mut baseline = InvocationControl::unbounded();
    execute_program(&program, &source, "baseline", &mut baseline).unwrap();
    let visits = baseline.consumed(WorkDomain::XPathNodeVisit);
    assert!(visits > 0);
    for limit in [0, visits - 1] {
        let mut control = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xpath_node_visits: limit,
                ..WorkLimits::unbounded()
            },
        );
        let failure = execute_program(&program, &source, "limited", &mut control).unwrap_err();
        assert_eq!(failure.code, "FXCT0002");
        assert_eq!(failure.work_domain, Some(WorkDomain::XPathNodeVisit));
    }
    let mut exact = InvocationControl::new(
        CancellationToken::new(),
        WorkLimits {
            xpath_node_visits: visits,
            ..WorkLimits::unbounded()
        },
    );
    execute_program(&program, &source, "exact", &mut exact).unwrap();
    let token = CancellationToken::new();
    token.cancel();
    let failure = execute_program(
        &program,
        &source,
        "cancelled",
        &mut InvocationControl::new(token, WorkLimits::unbounded()),
    )
    .unwrap_err();
    assert_eq!(failure.code, "FXCT0001");
    execute_program(
        &program,
        &source,
        "recovered",
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
}

#[test]
fn temporary_attribute_paths_match_source_values_without_aliasing_namespaces() {
    let document = Document::from_parsed(parse_document(
        "urn:temporary-attributes:source",
        br#"<r><item xmlns:p="urn:one" xmlns:q="urn:two" key="alpha" p:key="beta" q:key="gamma"/><item/></r>"#,
        ParseLimits { max_events: 100, max_depth: 16 },
    ).unwrap()).unwrap();
    for (path, expected) in [
        ("@key", "alpha"),
        ("@p:key", "beta"),
        ("@q:key", "gamma"),
        ("@*", "alpha;beta;gamma"),
        ("@missing", ""),
    ] {
        let body = format!("<out><xsl:value-of select='{path}' separator=';'/></out>");
        let stylesheet = format!(
            "<xsl:stylesheet version='2.0' xmlns:xsl='http://www.w3.org/1999/XSL/Transform' xmlns:p='urn:one' xmlns:q='urn:two' exclude-result-prefixes='p q'><xsl:output omit-xml-declaration='yes'/><xsl:template match='/'><xsl:variable name='t'><box><xsl:copy-of select='/r/item'/></box></xsl:variable><xsl:for-each select='SELECT'>{body}</xsl:for-each></xsl:template></xsl:stylesheet>"
        );
        let mut outputs = Vec::new();
        for selection in ["/r/item", "$t/box/item"] {
            let compiled = program(stylesheet.replace("SELECT", selection).as_bytes());
            let mut control = InvocationControl::unbounded();
            let result =
                execute_program(&compiled, &document, "attribute-parity", &mut control).unwrap();
            outputs.push(
                serialize_xml(
                    &result,
                    &compiled.output,
                    "attribute-parity",
                    8192,
                    &mut control,
                )
                .unwrap(),
            );
        }
        assert_eq!(outputs[0], outputs[1], "{path}");
        assert_eq!(
            outputs[1],
            if expected.is_empty() {
                "<out></out><out></out>".to_owned()
            } else {
                format!("<out>{expected}</out><out></out>")
            }
        );
    }
}

#[test]
fn temporary_attribute_paths_reject_predicates_with_path_provenance() {
    let style = std::str::from_utf8(STYLE).unwrap().replace(
        "<xsl:value-of select=\"position()\"/>",
        "<xsl:value-of select=\"@key[1]\"/>",
    );
    let failure = execute_program(
        &program(style.as_bytes()),
        &source(),
        "predicate",
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(failure.code, "FXRT0007");
    assert_eq!(failure.category, FailureCategory::Unsupported);
    assert_eq!(failure.location.unwrap().resource, STYLE_ID);
}

#[test]
fn temporary_attribute_paths_preserve_exact_budgets_and_mid_selection_cancellation() {
    let style = br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:variable name="t"><box><item key="alpha" other="beta"/></box></xsl:variable><out><xsl:for-each select="$t/box/item"><xsl:value-of select="@*" separator=";"/></xsl:for-each></out></xsl:template></xsl:stylesheet>"#;
    let compiled = program(style);
    let document = source();
    let mut baseline = InvocationControl::unbounded();
    execute_program(&compiled, &document, "baseline", &mut baseline).unwrap();
    for domain in [
        WorkDomain::XPathNodeVisit,
        WorkDomain::XdmStringValueNode,
        WorkDomain::ResultTextByte,
    ] {
        let total = baseline.consumed(domain);
        assert!(total > 0);
        for (limit, succeeds) in [(total - 1, false), (total, true)] {
            let mut limits = WorkLimits::unbounded();
            match domain {
                WorkDomain::XPathNodeVisit => limits.xpath_node_visits = limit,
                WorkDomain::XdmStringValueNode => limits.xdm_string_value_nodes = limit,
                WorkDomain::ResultTextByte => limits.result_text_bytes = limit,
                _ => unreachable!(),
            }
            let result = execute_program(
                &compiled,
                &document,
                "controlled",
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            if succeeds {
                result.unwrap();
            } else {
                let failure = result.unwrap_err();
                assert_eq!(failure.code, "FXCT0002");
                assert_eq!(failure.work_domain, Some(domain));
            }
        }
    }
    let visits = baseline.consumed(WorkDomain::XPathNodeVisit);
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, visits - 1);
    let failure = execute_program(&compiled, &document, "cancelled", &mut cancelled).unwrap_err();
    assert_eq!(failure.code, "FXCT0001");
    assert_eq!(failure.work_domain, Some(WorkDomain::XPathNodeVisit));
    let mut fresh = InvocationControl::unbounded();
    let result = execute_program(&compiled, &document, "fresh", &mut fresh).unwrap();
    assert_eq!(
        serialize_xml(&result, &compiled.output, "fresh", 8192, &mut fresh).unwrap(),
        "<out>alpha;beta</out>"
    );
}

#[test]
fn temporary_attribute_paths_do_not_admit_instruction_version_override() {
    let style = br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:variable name="t"><box><item key="alpha" other="beta"/></box></xsl:variable><out><xsl:for-each select="$t/box/item"><xsl:value-of version="1.0" select="@*"/></xsl:for-each></out></xsl:template></xsl:stylesheet>"#;
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    builder.admit(STYLE_ID, style.to_vec()).unwrap();
    let failure = compile_resource(&builder.seal(), STYLE_ID).unwrap_err();
    assert_eq!(failure.code, "FXST1009");
    assert_eq!(failure.category, FailureCategory::Unsupported);
    assert_eq!(failure.location.unwrap().resource, STYLE_ID);
}
