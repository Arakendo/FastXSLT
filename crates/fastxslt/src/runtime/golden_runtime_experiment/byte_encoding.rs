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

pub(super) fn encode_iso_8859_1_xml(
    value: &str,
    request_id: &str,
) -> Result<Vec<u8>, ExecutionFailure> {
    #[derive(Clone, Copy)]
    enum Context {
        Text,
        Tag,
        Attribute(char),
        Comment,
        ProcessingInstruction,
        Cdata,
        Declaration,
    }

    fn push_direct(
        output: &mut Vec<u8>,
        character: char,
        request_id: &str,
        context: &str,
    ) -> Result<(), ExecutionFailure> {
        let codepoint = u32::from(character);
        if let Ok(byte) = u8::try_from(codepoint) {
            output.push(byte);
            Ok(())
        } else {
            Err(failure(
                "FXSR1006",
                FailureCategory::Unsupported,
                Some(request_id),
                format!("ISO-8859-1 cannot represent U+{codepoint:04X} in serialized {context}"),
            ))
        }
    }

    fn push_reference(output: &mut Vec<u8>, character: char) {
        output.extend_from_slice(format!("&#x{:X};", u32::from(character)).as_bytes());
    }

    let mut output = Vec::with_capacity(value.len());
    let mut remaining = value;
    let mut context = Context::Text;
    while !remaining.is_empty() {
        let transition = match context {
            Context::Text if remaining.starts_with("<![CDATA[") => {
                Some(("<![CDATA[", Context::Cdata))
            }
            Context::Text if remaining.starts_with("<!--") => Some(("<!--", Context::Comment)),
            Context::Text if remaining.starts_with("<?") => {
                Some(("<?", Context::ProcessingInstruction))
            }
            Context::Text if remaining.starts_with("<!") => Some(("<!", Context::Declaration)),
            Context::Text if remaining.starts_with('<') => Some(("<", Context::Tag)),
            Context::Comment if remaining.starts_with("-->") => Some(("-->", Context::Text)),
            Context::ProcessingInstruction if remaining.starts_with("?>") => {
                Some(("?>", Context::Text))
            }
            Context::Cdata if remaining.starts_with("]]>") => Some(("]]>", Context::Text)),
            Context::Declaration | Context::Tag if remaining.starts_with('>') => {
                Some((">", Context::Text))
            }
            _ => None,
        };
        if let Some((syntax, next)) = transition {
            output.extend_from_slice(syntax.as_bytes());
            remaining = &remaining[syntax.len()..];
            context = next;
            continue;
        }

        let character = remaining.chars().next().expect("nonempty remainder");
        remaining = &remaining[character.len_utf8()..];
        match context {
            Context::Text | Context::Attribute(_) if u32::from(character) > 0xff => {
                push_reference(&mut output, character);
            }
            Context::Cdata if u32::from(character) > 0xff => {
                output.extend_from_slice(b"]]>");
                push_reference(&mut output, character);
                output.extend_from_slice(b"<![CDATA[");
            }
            Context::Tag if matches!(character, '\'' | '"') => {
                output.push(character as u8);
                context = Context::Attribute(character);
            }
            Context::Attribute(delimiter) if character == delimiter => {
                output.push(character as u8);
                context = Context::Tag;
            }
            Context::Comment => {
                push_direct(&mut output, character, request_id, "comment")?;
            }
            Context::ProcessingInstruction => {
                push_direct(&mut output, character, request_id, "processing instruction")?;
            }
            Context::Declaration => {
                push_direct(&mut output, character, request_id, "declaration")?;
            }
            Context::Tag => {
                push_direct(&mut output, character, request_id, "markup name")?;
            }
            Context::Text | Context::Attribute(_) | Context::Cdata => {
                push_direct(&mut output, character, request_id, "content")?;
            }
        }
    }
    Ok(output)
}

pub(super) fn encode_iso_8859_1_text(
    value: &str,
    request_id: &str,
) -> Result<Vec<u8>, ExecutionFailure> {
    value
        .chars()
        .map(|character| {
            u8::try_from(u32::from(character)).map_err(|_| {
                failure(
                    "FXSR1006",
                    FailureCategory::Unsupported,
                    Some(request_id),
                    format!(
                        "ISO-8859-1 cannot represent U+{:04X} in text output",
                        u32::from(character)
                    ),
                )
            })
        })
        .collect()
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
