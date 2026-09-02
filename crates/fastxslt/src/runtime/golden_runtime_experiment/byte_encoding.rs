//! Private physical encoders used by the test-only serialization byte lane.

use std::fmt::Write as _;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xslt::golden_semantics_experiment::OutputSettings;

use super::serialization::serialize_xml;
use super::{ExecutionFailure, FailureCategory, SemanticResult, control_failure, failure};

pub(super) fn encode_us_ascii_cdata(
    value: &str,
    request_id: &str,
) -> Result<String, ExecutionFailure> {
    let mut output = String::with_capacity(value.len());
    let mut remaining = value;
    let mut in_cdata = false;
    while !remaining.is_empty() {
        if remaining.starts_with("<![CDATA[") {
            output.push_str("<![CDATA[");
            remaining = &remaining[9..];
            in_cdata = true;
            continue;
        }
        if remaining.starts_with("]]>") {
            output.push_str("]]>");
            remaining = &remaining[3..];
            in_cdata = false;
            continue;
        }
        let character = remaining.chars().next().expect("nonempty remainder");
        remaining = &remaining[character.len_utf8()..];
        if character.is_ascii() {
            output.push(character);
        } else if in_cdata {
            output.push_str("]]>&#x");
            write!(&mut output, "{:X}", u32::from(character))
                .expect("writing to a String cannot fail");
            output.push_str(";<![CDATA[");
        } else {
            return Err(failure(
                "FXSR1009",
                FailureCategory::Unsupported,
                Some(request_id),
                "the bounded US-ASCII lane admits non-ASCII characters only inside selected CDATA text",
            ));
        }
    }
    Ok(output)
}

pub(super) fn encode_utf16_be(value: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(value.encode_utf16().count() * 2);
    for unit in value.encode_utf16() {
        bytes.extend_from_slice(&unit.to_be_bytes());
    }
    bytes
}

pub(super) fn serialize_utf16_be(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
    byte_limit: usize,
    control: &mut InvocationControl,
) -> Result<Vec<u8>, ExecutionFailure> {
    let declaration = if settings.omit_xml_declaration
        || matches!(settings.method.as_deref(), Some("text" | "html"))
    {
        String::new()
    } else {
        "<?xml version=\"1.0\" encoding=\"UTF-16\"?>".to_owned()
    };
    let mut body_settings = settings.clone();
    body_settings.encoding = Some("UTF-8".to_owned());
    body_settings.omit_xml_declaration = true;
    body_settings.standalone = None;
    body_settings.version = Some("1.0".to_owned());
    let body = serialize_xml(result, &body_settings, request_id, usize::MAX, control)?;
    let mut characters = String::with_capacity(declaration.len() + body.len());
    characters.push_str(&declaration);
    characters.push_str(&body);
    let encoded = encode_utf16_be(&characters);
    let required = encoded.len().checked_add(2).ok_or_else(|| {
        failure(
            "FXSR0002",
            FailureCategory::Limit,
            Some(request_id),
            "serialized UTF-16 result byte count overflowed",
        )
    })?;
    if required > byte_limit {
        return Err(failure(
            "FXSR0002",
            FailureCategory::Limit,
            Some(request_id),
            format!("serialized result requires {required} bytes; limit is {byte_limit}"),
        ));
    }
    control
        .charge(
            WorkDomain::SerializedByte,
            required.saturating_sub(body.len()),
        )
        .map_err(|failure| control_failure(failure, request_id))?;
    let mut bytes = Vec::with_capacity(required);
    bytes.extend_from_slice(&[0xfe, 0xff]);
    bytes.extend_from_slice(&encoded);
    Ok(bytes)
}
