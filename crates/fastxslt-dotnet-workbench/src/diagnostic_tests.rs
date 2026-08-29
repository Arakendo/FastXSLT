use super::{
    OUTCOME_FAILURE, OUTCOME_RESULT, fastxslt_workbench_v0_create,
    fastxslt_workbench_v0_create_with_stylesheet_dependency, fastxslt_workbench_v0_engine_release,
    fastxslt_workbench_v0_outcome_copy, fastxslt_workbench_v0_outcome_kind,
    fastxslt_workbench_v0_outcome_length, fastxslt_workbench_v0_outcome_release,
    fastxslt_workbench_v0_outcome_take_engine, fastxslt_workbench_v0_transform,
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

    for (deny, expected_code, expected_category) in [
        (0, "FXRS0002", "missing-resource"),
        (1, "FXRS0003", "denied"),
    ] {
        let outcome = fastxslt_workbench_v0_create_with_stylesheet_dependency(
            SOURCE_ID.as_ptr(),
            SOURCE_ID.len(),
            b"<source/>".as_ptr(),
            b"<source/>".len(),
            STYLESHEET_ID.as_ptr(),
            STYLESHEET_ID.len(),
            stylesheet.as_ptr(),
            stylesheet.len(),
            DEPENDENCY_ID.as_ptr(),
            DEPENDENCY_ID.len(),
            std::ptr::null(),
            0,
            0,
            deny,
        );
        assert_eq!(fastxslt_workbench_v0_outcome_kind(outcome), OUTCOME_FAILURE);
        let fields = failure_fields(outcome);
        assert_eq!(fields[0], expected_code);
        assert_eq!(fields[1], expected_category);
        assert_eq!(fields[3], STYLESHEET_ID);
        assert!(fields[6].contains(DEPENDENCY_ID));
        assert_eq!(fastxslt_workbench_v0_outcome_release(outcome), 1);
    }
}

#[test]
fn native_dependency_initialization_executes_admitted_module() {
    const SOURCE_ID: &str = "urn:fastxslt:native-dependency:source";
    const STYLESHEET_ID: &str = "https://example.invalid/styles/main.xsl";
    const DEPENDENCY_ID: &str = "https://example.invalid/styles/dependency.xsl";
    let source = b"<source/>";
    let stylesheet = br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:include href="dependency.xsl"/><xsl:variable name="greeting">hello</xsl:variable></xsl:stylesheet>"#;
    let dependency = br#"<out xsl:version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:value-of select="$greeting"/></out>"#;
    let creation = fastxslt_workbench_v0_create_with_stylesheet_dependency(
        SOURCE_ID.as_ptr(),
        SOURCE_ID.len(),
        source.as_ptr(),
        source.len(),
        STYLESHEET_ID.as_ptr(),
        STYLESHEET_ID.len(),
        stylesheet.as_ptr(),
        stylesheet.len(),
        DEPENDENCY_ID.as_ptr(),
        DEPENDENCY_ID.len(),
        dependency.as_ptr(),
        dependency.len(),
        1,
        0,
    );
    let engine = fastxslt_workbench_v0_outcome_take_engine(creation);
    assert_ne!(engine, 0);
    let request = b"native-dependency";
    let result = fastxslt_workbench_v0_transform(engine, request.as_ptr(), request.len());
    assert_eq!(fastxslt_workbench_v0_outcome_kind(result), OUTCOME_RESULT);
    let length = fastxslt_workbench_v0_outcome_length(result);
    let mut bytes = vec![0_u8; length];
    assert_eq!(
        fastxslt_workbench_v0_outcome_copy(result, bytes.as_mut_ptr(), bytes.len()),
        0
    );
    assert_eq!(
        bytes,
        b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>hello</out>"
    );
    assert_eq!(fastxslt_workbench_v0_outcome_release(result), 1);
    assert_eq!(fastxslt_workbench_v0_engine_release(engine), 1);
}

#[test]
fn native_dependency_initialization_rejects_invalid_framing() {
    let source = b"<source/>";
    let stylesheet = b"<out/>";
    let dependency_id = b"urn:fastxslt:invalid-framing:dependency";
    for (dependency, admitted, expected_code) in [
        (b"".as_slice(), 2, "FXFFI0012"),
        (b"bytes".as_slice(), 0, "FXFFI0013"),
    ] {
        let outcome = fastxslt_workbench_v0_create_with_stylesheet_dependency(
            b"urn:fastxslt:invalid-framing:source".as_ptr(),
            b"urn:fastxslt:invalid-framing:source".len(),
            source.as_ptr(),
            source.len(),
            b"urn:fastxslt:invalid-framing:stylesheet".as_ptr(),
            b"urn:fastxslt:invalid-framing:stylesheet".len(),
            stylesheet.as_ptr(),
            stylesheet.len(),
            dependency_id.as_ptr(),
            dependency_id.len(),
            dependency.as_ptr(),
            dependency.len(),
            admitted,
            0,
        );
        assert_eq!(fastxslt_workbench_v0_outcome_kind(outcome), OUTCOME_FAILURE);
        assert_eq!(failure_fields(outcome)[0], expected_code);
        assert_eq!(fastxslt_workbench_v0_outcome_release(outcome), 1);
    }
}
