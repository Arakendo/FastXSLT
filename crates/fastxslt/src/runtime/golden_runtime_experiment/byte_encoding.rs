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
    encode_single_byte_xml(value, request_id, "ISO-8859-1", |character| {
        u8::try_from(u32::from(character)).ok()
    })
}

pub(super) fn encode_iso_8859_2_xml(
    value: &str,
    request_id: &str,
) -> Result<Vec<u8>, ExecutionFailure> {
    encode_single_byte_xml(value, request_id, "ISO-8859-2", encode_iso_8859_2_character)
}

#[allow(
    clippy::too_many_lines,
    reason = "the cohesive XML lexical-context state machine keeps encoding decisions local"
)]
fn encode_single_byte_xml(
    value: &str,
    request_id: &str,
    encoding: &str,
    encode: fn(char) -> Option<u8>,
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
        encoding: &str,
        encode: fn(char) -> Option<u8>,
    ) -> Result<(), ExecutionFailure> {
        let codepoint = u32::from(character);
        if let Some(byte) = encode(character) {
            output.push(byte);
            Ok(())
        } else {
            Err(failure(
                "FXSR1006",
                FailureCategory::Unsupported,
                Some(request_id),
                format!("{encoding} cannot represent U+{codepoint:04X} in serialized {context}"),
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
            Context::Text | Context::Attribute(_) if encode(character).is_none() => {
                push_reference(&mut output, character);
            }
            Context::Cdata if encode(character).is_none() => {
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
                push_direct(
                    &mut output,
                    character,
                    request_id,
                    "comment",
                    encoding,
                    encode,
                )?;
            }
            Context::ProcessingInstruction => {
                push_direct(
                    &mut output,
                    character,
                    request_id,
                    "processing instruction",
                    encoding,
                    encode,
                )?;
            }
            Context::Declaration => {
                push_direct(
                    &mut output,
                    character,
                    request_id,
                    "declaration",
                    encoding,
                    encode,
                )?;
            }
            Context::Tag => {
                push_direct(
                    &mut output,
                    character,
                    request_id,
                    "markup name",
                    encoding,
                    encode,
                )?;
            }
            Context::Text | Context::Attribute(_) | Context::Cdata => {
                push_direct(
                    &mut output,
                    character,
                    request_id,
                    "content",
                    encoding,
                    encode,
                )?;
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

pub(super) fn encode_iso_8859_2_text(
    value: &str,
    request_id: &str,
) -> Result<Vec<u8>, ExecutionFailure> {
    value
        .chars()
        .map(|character| {
            encode_iso_8859_2_character(character).ok_or_else(|| {
                failure(
                    "FXSR1006",
                    FailureCategory::Unsupported,
                    Some(request_id),
                    format!(
                        "ISO-8859-2 cannot represent U+{:04X} in text output",
                        u32::from(character)
                    ),
                )
            })
        })
        .collect()
}

const ISO_8859_2_UPPER: [char; 96] = [
    '\u{00A0}', '\u{0104}', '\u{02D8}', '\u{0141}', '\u{00A4}', '\u{013D}', '\u{015A}', '\u{00A7}',
    '\u{00A8}', '\u{0160}', '\u{015E}', '\u{0164}', '\u{0179}', '\u{00AD}', '\u{017D}', '\u{017B}',
    '\u{00B0}', '\u{0105}', '\u{02DB}', '\u{0142}', '\u{00B4}', '\u{013E}', '\u{015B}', '\u{02C7}',
    '\u{00B8}', '\u{0161}', '\u{015F}', '\u{0165}', '\u{017A}', '\u{02DD}', '\u{017E}', '\u{017C}',
    '\u{0154}', '\u{00C1}', '\u{00C2}', '\u{0102}', '\u{00C4}', '\u{0139}', '\u{0106}', '\u{00C7}',
    '\u{010C}', '\u{00C9}', '\u{0118}', '\u{00CB}', '\u{011A}', '\u{00CD}', '\u{00CE}', '\u{010E}',
    '\u{0110}', '\u{0143}', '\u{0147}', '\u{00D3}', '\u{00D4}', '\u{0150}', '\u{00D6}', '\u{00D7}',
    '\u{0158}', '\u{016E}', '\u{00DA}', '\u{0170}', '\u{00DC}', '\u{00DD}', '\u{0162}', '\u{00DF}',
    '\u{0155}', '\u{00E1}', '\u{00E2}', '\u{0103}', '\u{00E4}', '\u{013A}', '\u{0107}', '\u{00E7}',
    '\u{010D}', '\u{00E9}', '\u{0119}', '\u{00EB}', '\u{011B}', '\u{00ED}', '\u{00EE}', '\u{010F}',
    '\u{0111}', '\u{0144}', '\u{0148}', '\u{00F3}', '\u{00F4}', '\u{0151}', '\u{00F6}', '\u{00F7}',
    '\u{0159}', '\u{016F}', '\u{00FA}', '\u{0171}', '\u{00FC}', '\u{00FD}', '\u{0163}', '\u{02D9}',
];

fn encode_iso_8859_2_character(character: char) -> Option<u8> {
    let codepoint = u32::from(character);
    if codepoint < 0xA0 {
        return u8::try_from(codepoint).ok();
    }
    ISO_8859_2_UPPER
        .iter()
        .position(|candidate| *candidate == character)
        .and_then(|index| u8::try_from(index + 0xA0).ok())
}

#[cfg(test)]
pub(crate) fn decode_iso_8859_2_byte(byte: u8) -> char {
    if byte < 0xA0 {
        return char::from(byte);
    }
    ISO_8859_2_UPPER[usize::from(byte - 0xA0)]
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

#[cfg(test)]
mod tests {
    use super::{decode_iso_8859_2_byte, encode_iso_8859_2_text, encode_iso_8859_2_xml};

    #[test]
    fn iso_8859_2_round_trips_representative_central_european_characters() {
        let encoded = encode_iso_8859_2_text("Ąčřůž", "encoding-test")
            .expect("representative ISO-8859-2 text should encode");
        assert_eq!(encoded, [0xA1, 0xE8, 0xF8, 0xF9, 0xBE]);
        assert_eq!(
            encoded
                .into_iter()
                .map(decode_iso_8859_2_byte)
                .collect::<String>(),
            "Ąčřůž"
        );
    }

    #[test]
    fn iso_8859_2_xml_references_unrepresentable_character_data() {
        assert_eq!(
            encode_iso_8859_2_xml("<out>€</out>", "encoding-test")
                .expect("XML content may use a character reference"),
            b"<out>&#x20AC;</out>"
        );
        assert!(encode_iso_8859_2_text("€", "encoding-test").is_err());
    }
}
