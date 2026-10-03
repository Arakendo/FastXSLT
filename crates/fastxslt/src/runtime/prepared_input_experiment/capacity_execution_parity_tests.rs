//! AR-0027 execution parity over already admitted DTD and temporary-tree semantics.

use crate::resources::{ResolutionLimits, ResourceLimits, ResourceSetBuilder, SnapshotResolver};
use crate::xml::quick_xml_experiment::{
    AdmittedExternalSubset, ParsedDocument, parse_document_controlled,
    parse_document_controlled_with_single_external_subset,
};

use super::{
    Capacity, DOMAINS, Document, InternalSubsetLimits, InvocationControl, ParseLimits, SOURCE_ID,
    assert_visible_parity, compile_resource, execute_program, output, prepare, program,
    serialize_xml,
};

const EXTERNAL_SOURCE_ID: &str = "https://ar0027.invalid/input/source.xml";
const EXTERNAL_DTD_ID: &str = "https://ar0027.invalid/input/types.dtd";
const EXTERNAL_DTD: &[u8] =
    br#"<!ENTITY word "expanded"><!ATTLIST item key ID #IMPLIED note CDATA "outer">"#;

fn construct(parsed: ParsedDocument, lane: usize, control: &mut InvocationControl) -> Document {
    match lane {
        0 => Document::from_parsed_controlled(parsed, control),
        1 => Document::from_parsed_with_presized_node_capacity(parsed, control),
        2 => Document::from_parsed_with_frozen_node_capacity(parsed, control),
        3 => Document::from_parsed_with_capacity_polls(parsed, control, true, false, None),
        _ => Document::from_parsed_with_capacity_polls(parsed, control, false, true, None),
    }
    .unwrap()
}

#[test]
fn capacity_execution_preserves_sealed_external_defaults_ids_and_views() {
    for internal in ["", " [<!ATTLIST item note CDATA 'inner'>]"] {
        let source = format!(
            "<!DOCTYPE r SYSTEM 'types.dtd'{internal}><r xmlns:p='urn:p'> <item key='alpha'>&word;<p:leaf/></item> <item key='beta' note='authored'/> </r>"
        );
        let expected_note = if internal.is_empty() {
            "outer"
        } else {
            "inner"
        };
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
        resources
            .admit(EXTERNAL_SOURCE_ID, source.into_bytes())
            .unwrap();
        resources
            .admit(EXTERNAL_DTD_ID, EXTERNAL_DTD.to_vec())
            .unwrap();
        let snapshot = resources.seal();
        let limits = ParseLimits {
            max_events: 1000,
            max_depth: 32,
        };
        assert_eq!(
            parse_document_controlled(
                EXTERNAL_SOURCE_ID,
                snapshot.get(EXTERNAL_SOURCE_ID).unwrap(),
                limits,
                &mut InvocationControl::unbounded()
            )
            .unwrap_err()
            .dtd_reference_category(),
            "unexpected-default-denial"
        );
        let mut parsed_charges = None;
        let mut reference = None;
        let mut expected_outputs = None;
        for lane in 0..5 {
            let mut control = InvocationControl::unbounded();
            let mut resolver =
                SnapshotResolver::new(&snapshot, std::iter::empty(), ResolutionLimits::new(1));
            let external = resolver
                .resolve_from(EXTERNAL_SOURCE_ID, "types.dtd")
                .unwrap();
            assert_eq!(external.identity, EXTERNAL_DTD_ID);
            let parsed = parse_document_controlled_with_single_external_subset(
                EXTERNAL_SOURCE_ID,
                snapshot.get(EXTERNAL_SOURCE_ID).unwrap(),
                limits,
                InternalSubsetLimits {
                    declarations: 16,
                    nesting_depth: 8,
                    references: 100,
                    replacement_bytes: 8192,
                },
                AdmittedExternalSubset {
                    identity: &external.identity,
                    reference: "types.dtd",
                    bytes: external.bytes,
                    max_bytes: 8192,
                },
                &mut control,
            )
            .unwrap();
            let document = construct(parsed, lane, &mut control);
            let charges = DOMAINS.map(|domain| control.consumed(domain));
            if let Some(expected) = parsed_charges {
                assert_eq!(charges, expected);
            } else {
                parsed_charges = Some(charges);
            }
            assert_eq!(document.elements_with_id("alpha").len(), 1);
            let mut outputs = Vec::new();
            for strip in [false, true] {
                let compiled = program(
                    "urn:ar0027:external-style",
                    strip,
                    "<out><xsl:value-of select=\"id('alpha')/@note\"/><xsl:text>|</xsl:text><xsl:value-of select=\"id('alpha')\"/><xsl:text>|</xsl:text><xsl:value-of select='count(/r/item/..)'/><xsl:text>|</xsl:text><xsl:value-of select='count(/r/text())'/></out>",
                );
                let observed = output(&compiled, &document);
                assert_eq!(
                    observed.0,
                    format!(
                        "<out>{expected_note}|expanded|1|{}</out>",
                        if strip { 0 } else { 3 }
                    )
                );
                outputs.push(observed);
            }
            if let Some(expected) = &expected_outputs {
                assert_eq!(&outputs, expected);
            } else {
                expected_outputs = Some(outputs);
            }
            if let Some(reference) = &reference {
                assert_visible_parity(reference, &document);
            } else {
                reference = Some(document);
            }
        }
    }
}

#[test]
#[ignore = "open AR-0027 baseline blocker: temporary-path for-each reaches source-only unreachable dispatch"]
fn capacity_execution_preserves_temporary_focus_paths_and_result_retirement() {
    let stylesheet = br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:strip-space elements="*"/><xsl:template match="/"><xsl:variable name="t"><box><xsl:copy-of select="/r/item"/></box></xsl:variable><out><xsl:for-each select="$t/box/item"><v><xsl:value-of select="position()"/><xsl:text>/</xsl:text><xsl:value-of select="last()"/><xsl:text>:</xsl:text><xsl:value-of select="@key"/></v></xsl:for-each><xsl:text>|</xsl:text><xsl:value-of select="count(/r/item/..)"/></out></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources
        .admit("urn:ar0027:temporary-style", stylesheet.to_vec())
        .unwrap();
    let snapshot = resources.seal();
    let compiled = compile_resource(&snapshot, "urn:ar0027:temporary-style").unwrap();
    let reference = prepare(Capacity::Growth);
    let expected = output(&compiled, &reference);
    assert_eq!(expected.0, "<out><v>1/2:alpha</v><v>2/2:beta</v>|1</out>");
    for capacity in [Capacity::Presized, Capacity::Frozen] {
        let candidate = prepare(capacity);
        assert_eq!(output(&compiled, &candidate), expected);
        let mut control = InvocationControl::unbounded();
        let result = execute_program(&compiled, &candidate, "ar0027-parity", &mut control).unwrap();
        drop(candidate);
        let serialized = serialize_xml(
            &result,
            &compiled.output,
            "ar0027-parity",
            8192,
            &mut control,
        )
        .unwrap();
        assert_eq!(serialized, expected.0);
        assert_eq!(
            DOMAINS.map(|domain| control.consumed(domain)).to_vec(),
            expected.1
        );
    }
    assert_eq!(
        reference.location(reference.document_node()).resource,
        SOURCE_ID
    );
}
