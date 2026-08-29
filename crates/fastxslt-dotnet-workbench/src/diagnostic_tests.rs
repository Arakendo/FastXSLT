use fastxslt::workbench::{ExperimentalEngine, WorkbenchLimits, WorkbenchStylesheetResources};

use super::{
    OUTCOME_FAILURE, engine_failure, fastxslt_workbench_v0_create,
    fastxslt_workbench_v0_outcome_copy, fastxslt_workbench_v0_outcome_kind,
    fastxslt_workbench_v0_outcome_length, fastxslt_workbench_v0_outcome_release, state,
};

fn failure_fields(outcome: u64) -> Vec<String> {
    let length = fastxslt_workbench_v0_outcome_length(outcome);
    let mut bytes = vec![0_u8; length];
    assert_eq!(
        fastxslt_workbench_v0_outcome_copy(outcome, bytes.as_mut_ptr(), bytes.len()),
        0
    );
    let mut offset = 0;
    let mut fields = Vec::new();
    for _ in 0..7 {
        let length = u32::from_le_bytes(
            bytes[offset..offset + 4]
                .try_into()
                .expect("failure length field"),
        ) as usize;
        offset += 4;
        fields.push(
            String::from_utf8(bytes[offset..offset + length].to_vec())
                .expect("UTF-8 failure field"),
        );
        offset += length;
    }
    assert_eq!(offset, bytes.len());
    fields
}

#[test]
fn native_failure_envelope_preserves_structured_location() {
    let source_identity = b"urn:fastxslt:diagnostic:source";
    let source = b"<order/>";
    let stylesheet_identity = b"urn:fastxslt:diagnostic:unsupported-stylesheet";
    let stylesheet = br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/"><xsl:message/></xsl:template></xsl:stylesheet>"#;
    let outcome = fastxslt_workbench_v0_create(
        source_identity.as_ptr(),
        source_identity.len(),
        source.as_ptr(),
        source.len(),
        stylesheet_identity.as_ptr(),
        stylesheet_identity.len(),
        stylesheet.as_ptr(),
        stylesheet.len(),
    );
    assert_eq!(fastxslt_workbench_v0_outcome_kind(outcome), OUTCOME_FAILURE);
    assert_eq!(
        failure_fields(outcome),
        [
            "FXST1006",
            "unsupported",
            "",
            "urn:fastxslt:diagnostic:unsupported-stylesheet",
            "103",
            "117",
            "unsupported XSLT instruction: xsl:message at urn:fastxslt:diagnostic:unsupported-stylesheet:103..117",
        ]
    );
    assert_eq!(fastxslt_workbench_v0_outcome_release(outcome), 1);
}

#[test]
fn native_failure_envelope_preserves_resource_authority_categories() {
    const SOURCE_ID: &str = "urn:fastxslt:native-resource-diagnostic:source";
    const STYLESHEET_ID: &str = "https://example.invalid/styles/main.xsl";
    const DEPENDENCY_ID: &str = "https://example.invalid/styles/dependency.xsl";
    let stylesheet = br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:include href="dependency.xsl"/></xsl:stylesheet>"#;

    for (policy, expected_code, expected_category) in [
        (
            WorkbenchStylesheetResources::default(),
            "FXRS0002",
            "missing-resource",
        ),
        (
            WorkbenchStylesheetResources {
                dependencies: Vec::new(),
                denied_identities: vec![DEPENDENCY_ID.to_owned()],
            },
            "FXRS0003",
            "denied",
        ),
    ] {
        let Err(failure) = ExperimentalEngine::new_with_stylesheet_resources(
            SOURCE_ID,
            b"<source/>".to_vec(),
            STYLESHEET_ID,
            stylesheet.to_vec(),
            policy,
            WorkbenchLimits::default(),
        ) else {
            panic!("resource authority probe must fail during compilation");
        };
        let outcome = state().insert_outcome(engine_failure(&failure));
        let fields = failure_fields(outcome);
        assert_eq!(fields[0], expected_code);
        assert_eq!(fields[1], expected_category);
        assert_eq!(fields[3], STYLESHEET_ID);
        assert!(fields[6].contains(DEPENDENCY_ID));
        assert_eq!(fastxslt_workbench_v0_outcome_release(outcome), 1);
    }
}
