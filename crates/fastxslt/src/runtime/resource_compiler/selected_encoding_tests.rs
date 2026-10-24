use super::*;
use crate::execution_control_experiment::WorkDomain;
use crate::runtime::golden_runtime_experiment::{execute_program, serialize_xml_bytes};

fn program(
    version: &str,
    encoding: &str,
    method: &str,
    body: &str,
) -> Result<StylesheetProgram, ExecutionFailure> {
    selected(&[(
        PRINCIPAL,
        &format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="{method}" encoding="{encoding}" omit-xml-declaration="yes"/><xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#
        ),
    )])
}

fn source() -> Document {
    Document::from_parsed(
        parse_document(
            "urn:encoding-source",
            b"<r/>",
            super::super::super::XML_LIMITS,
        )
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn selected_latin1_bytes_and_guarded_encodings() {
    for version in ["1.0", "2.0", "3.0"] {
        for encoding in ["ISO-8859-1", "iso-8859-1"] {
            let compiled = program(version, encoding, "xml", "<out>é ‰</out>").unwrap();
            let result = execute_program(
                &compiled,
                &source(),
                "latin1",
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let bytes = serialize_xml_bytes(
                &result,
                &compiled.output,
                "latin1",
                4096,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert_eq!(bytes, b"<out>\xE9 &#x2030;</out>");
            // This slice does not relabel encoded bytes as a UTF-8 string.
            assert!(execute_compiled_stylesheet_for_test(&compiled, &source(), "string").is_err());
        }
        for encoding in [
            "UTF-16LE",
            "UTF-16BE",
            "ISO-8859-2",
            "US-ASCII",
            "unicode",
            "latin1",
        ] {
            let failure = program(version, encoding, "xml", "<out/>").unwrap_err();
            assert_eq!(failure.category, FailureCategory::Unsupported);
            assert_eq!(failure.location.unwrap().resource, PRINCIPAL);
        }
    }
}

#[test]
fn selected_latin1_unrepresentable_content_and_reuse() {
    for (method, body) in [
        ("text", "<xsl:text>‰</xsl:text>"),
        ("xml", "<out><xsl:comment>‰</xsl:comment></out>"),
    ] {
        let compiled = program("2.0", "ISO-8859-1", method, body).unwrap();
        for _ in 0..2 {
            let result = execute_program(
                &compiled,
                &source(),
                "unrepresentable",
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert!(
                serialize_xml_bytes(
                    &result,
                    &compiled.output,
                    "unrepresentable",
                    4096,
                    &mut InvocationControl::unbounded()
                )
                .is_err()
            );
        }
    }
}

#[test]
fn selected_latin1_output_limits_cancel_and_reuse() {
    // ASCII keeps intermediate UTF-8 and final encoded lengths equal. The
    // existing serializer bounds both; this admission does not change that.
    let compiled = program("1.0", "ISO-8859-1", "xml", "<out>A</out>").unwrap();
    let result = execute_program(
        &compiled,
        &source(),
        "bounded",
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let expected = serialize_xml_bytes(
        &result,
        &compiled.output,
        "bounded",
        4096,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    for ceiling in [expected.len() - 1, expected.len()] {
        let actual = serialize_xml_bytes(
            &result,
            &compiled.output,
            "bounded",
            ceiling,
            &mut InvocationControl::unbounded(),
        );
        if ceiling == expected.len() {
            assert_eq!(actual.unwrap(), expected);
        } else {
            assert!(actual.is_err());
        }
    }
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::SerializedByte, 1);
    assert_eq!(
        serialize_xml_bytes(&result, &compiled.output, "cancel", 4096, &mut cancelled)
            .unwrap_err()
            .code,
        "FXCT0001"
    );
    assert_eq!(
        serialize_xml_bytes(
            &result,
            &compiled.output,
            "reuse",
            4096,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
}

#[test]
#[ignore = "manual B03 UTF-16 serializer ceiling observation"]
fn selected_encoding_utf16_ceiling_observation() {
    // Compile only currently admitted UTF-8. The private settings clone probes
    // the existing byte serializer, not a newly admitted processor profile.
    let body = format!("<out>{}</out>", "x".repeat(8192));
    let compiled = program("1.0", "UTF-8", "xml", &body).unwrap();
    let result = execute_program(
        &compiled,
        &source(),
        "probe",
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let mut settings = compiled.output.clone();
    settings.encoding = Some("UTF-16".to_owned());
    let mut control = InvocationControl::unbounded();
    let failure = serialize_xml_bytes(&result, &settings, "probe", 16, &mut control).unwrap_err();
    assert_eq!(failure.code, "FXSR0002");
    println!(
        "UTF16_CEILING_OBSERVATION ceiling=16 payload=8192 serialized_work={} failure={}",
        control.consumed(WorkDomain::SerializedByte),
        failure.code
    );
    // Do not assert late work as required behavior: a repair should reduce it.
    assert!(program("1.0", "UTF-16LE", "xml", "<out/>").is_err());
}

#[test]
fn selected_utf16_byte_admission_is_case_insensitive_and_not_string_output() {
    let expected: Vec<u8> = [0xfe, 0xff]
        .into_iter()
        .chain("<out>é中😀</out>".encode_utf16().flat_map(u16::to_be_bytes))
        .collect();
    for version in ["1.0", "2.0", "3.0"] {
        for encoding in ["UTF-16", "utf-16"] {
            let compiled = program(version, encoding, "xml", "<out>é中😀</out>").unwrap();
            let result = execute_program(
                &compiled,
                &source(),
                "utf16",
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert_eq!(
                serialize_xml_bytes(
                    &result,
                    &compiled.output,
                    "utf16",
                    expected.len(),
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                expected
            );
            assert!(execute_compiled_stylesheet_for_test(&compiled, &source(), "string").is_err());
        }
    }
}

#[test]
fn shared_utf16_bounded_staging_preserves_unicode_and_limits() {
    for (method, body, expected_body) in [
        ("xml", "<out>Aé中😀</out>", "<out>Aé中😀</out>"),
        ("text", "<xsl:text>中中中😀</xsl:text>", "中中中😀"),
        ("text", "<xsl:text/>", ""),
    ] {
        let compiled = program("1.0", "UTF-8", method, body).unwrap();
        let result = execute_program(
            &compiled,
            &source(),
            "utf16",
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        for omit in [false, true] {
            let mut settings = compiled.output.clone();
            settings.encoding = Some("UTF-16".to_owned());
            settings.omit_xml_declaration = omit;
            let declaration = if omit || method == "text" {
                ""
            } else {
                "<?xml version=\"1.0\" encoding=\"UTF-16\"?>"
            };
            let expected: Vec<u8> = [0xfe, 0xff]
                .into_iter()
                .chain(
                    format!("{declaration}{expected_body}")
                        .encode_utf16()
                        .flat_map(u16::to_be_bytes),
                )
                .collect();
            for ceiling in [0, 1, expected.len() - 1, expected.len(), usize::MAX] {
                let actual = serialize_xml_bytes(
                    &result,
                    &settings,
                    "utf16",
                    ceiling,
                    &mut InvocationControl::unbounded(),
                );
                if ceiling < expected.len() {
                    assert_eq!(actual.unwrap_err().code, "FXSR0002");
                } else {
                    assert_eq!(actual.unwrap(), expected);
                }
            }
            let mut cancelled =
                InvocationControl::unbounded().cancelling_on_charge(WorkDomain::SerializedByte, 1);
            assert_eq!(
                serialize_xml_bytes(&result, &settings, "cancel", expected.len(), &mut cancelled,)
                    .unwrap_err()
                    .code,
                "FXCT0001"
            );
            let mut measured = InvocationControl::unbounded();
            serialize_xml_bytes(&result, &settings, "charge", expected.len(), &mut measured)
                .unwrap();
            let charged = measured.consumed(WorkDomain::SerializedByte);
            for budget in [charged - 1, charged] {
                let mut limits = crate::execution_control_experiment::WorkLimits::unbounded();
                limits.serialized_bytes = budget;
                let mut bounded = InvocationControl::new(
                    crate::execution_control_experiment::CancellationToken::new(),
                    limits,
                );
                let actual =
                    serialize_xml_bytes(&result, &settings, "budget", expected.len(), &mut bounded);
                if budget == charged {
                    assert_eq!(actual.unwrap(), expected);
                } else {
                    assert_eq!(actual.unwrap_err().category, FailureCategory::Limit);
                }
            }
            assert_eq!(
                serialize_xml_bytes(
                    &result,
                    &settings,
                    "reuse",
                    expected.len(),
                    &mut InvocationControl::unbounded(),
                )
                .unwrap(),
                expected
            );
        }
    }
}

#[test]
fn shared_utf16_small_ceiling_stops_large_body_before_retention() {
    let compiled = program(
        "1.0",
        "UTF-8",
        "xml",
        &format!("<out>{}</out>", "x".repeat(8192)),
    )
    .unwrap();
    let result = execute_program(
        &compiled,
        &source(),
        "bounded",
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let mut settings = compiled.output.clone();
    settings.encoding = Some("UTF-16".to_owned());
    let mut control = InvocationControl::unbounded();
    assert_eq!(
        serialize_xml_bytes(&result, &settings, "bounded", 16, &mut control)
            .unwrap_err()
            .code,
        "FXSR0002"
    );
    // This fixture emits character-sized chunks: bounded staging now stops
    // near the ceiling, rather than serializing all 8 KiB before rejection.
    assert!(control.consumed(WorkDomain::SerializedByte) < 64);
}
