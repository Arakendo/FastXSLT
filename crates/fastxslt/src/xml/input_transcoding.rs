use std::borrow::Cow;

use encoding_rs::{DecoderResult, ISO_2022_JP};

pub(super) struct ParserInput<'a> {
    bytes: Cow<'a, [u8]>,
    original_offsets: Option<Vec<usize>>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct TranscodeFailure {
    pub(super) offset: usize,
    pub(super) detail: String,
}

impl<'a> ParserInput<'a> {
    pub(super) fn new(input: &'a [u8]) -> Result<Self, TranscodeFailure> {
        if let Some(content) = input.strip_prefix(&[0xff, 0xfe]) {
            return transcode_utf16(input, content, Utf16Endianness::Little);
        }
        if let Some(content) = input.strip_prefix(&[0xfe, 0xff]) {
            return transcode_utf16(input, content, Utf16Endianness::Big);
        }
        if declared_encoding(input).is_some_and(|encoding| {
            encoding.eq_ignore_ascii_case(b"ISO-2022-JP")
                || encoding.eq_ignore_ascii_case(b"ISO_2022_JP")
        }) {
            return transcode_iso_2022_jp(input);
        }
        Ok(Self {
            bytes: Cow::Borrowed(input),
            original_offsets: None,
        })
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(super) fn transcoded_utf8(&self) -> Option<&str> {
        self.original_offsets.as_ref().map(|_| {
            std::str::from_utf8(&self.bytes)
                .expect("XML input transcoding always produces valid UTF-8")
        })
    }

    pub(super) fn original_offset(&self, parser_offset: usize) -> usize {
        self.original_offsets
            .as_ref()
            .map_or(parser_offset, |offsets| {
                offsets
                    .get(parser_offset)
                    .copied()
                    .unwrap_or_else(|| *offsets.last().unwrap_or(&usize::MAX))
            })
    }
}

fn declared_encoding(input: &[u8]) -> Option<&[u8]> {
    let declaration_end = input
        .windows(2)
        .position(|window| window == b"?>")?
        .checked_add(2)?;
    let declaration = input.get(..declaration_end)?;
    if !declaration
        .get(..5)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"<?xml"))
    {
        return None;
    }
    let encoding = declaration
        .windows(b"encoding".len())
        .position(|window| window.eq_ignore_ascii_case(b"encoding"))?;
    let remainder = declaration.get(encoding + b"encoding".len()..)?;
    let equals = remainder.iter().position(|byte| *byte == b'=')?;
    let remainder = remainder.get(equals + 1..)?;
    let first = remainder
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())?;
    let quote = *remainder.get(first)?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }
    let value = remainder.get(first + 1..)?;
    let close = value.iter().position(|byte| *byte == quote)?;
    value.get(..close)
}

fn transcode_iso_2022_jp(input: &[u8]) -> Result<ParserInput<'_>, TranscodeFailure> {
    const INPUT_CHUNK_BYTES: usize = 256;
    let mut decoder = ISO_2022_JP.new_decoder_without_bom_handling();
    let mut utf8 = String::with_capacity(input.len());
    let mut original_offsets = vec![0];
    let mut pending_start = 0usize;
    let mut consumed = 0usize;

    while consumed < input.len() {
        let chunk_end = consumed.saturating_add(INPUT_CHUNK_BYTES).min(input.len());
        let remaining = &input[consumed..chunk_end];
        let reserve = decoder
            .max_utf8_buffer_length_without_replacement(remaining.len())
            .ok_or_else(|| TranscodeFailure {
                offset: consumed,
                detail: "ISO-2022-JP decoded-size calculation overflowed".to_owned(),
            })?;
        utf8.reserve(reserve);
        let output_start = utf8.len();
        let (result, read) = decoder.decode_to_string_without_replacement(
            remaining,
            &mut utf8,
            chunk_end == input.len(),
        );
        consumed += read;
        if utf8.len() > output_start {
            append_original_offsets(
                &mut original_offsets,
                utf8.len() - output_start,
                pending_start,
                consumed,
            );
            pending_start = consumed;
        }
        match result {
            DecoderResult::InputEmpty => {
                debug_assert_eq!(consumed, chunk_end);
            }
            DecoderResult::OutputFull => {
                if read == 0 {
                    return Err(TranscodeFailure {
                        offset: consumed,
                        detail:
                            "ISO-2022-JP decoder made no progress despite reserved output capacity"
                                .to_owned(),
                    });
                }
            }
            DecoderResult::Malformed(malformed, consumed_after) => {
                let offset =
                    consumed.saturating_sub(usize::from(malformed) + usize::from(consumed_after));
                return Err(TranscodeFailure {
                    offset,
                    detail: "cannot decode input using ISO-2022-JP".to_owned(),
                });
            }
        }
    }
    Ok(ParserInput {
        bytes: Cow::Owned(utf8.into_bytes()),
        original_offsets: Some(original_offsets),
    })
}

fn append_original_offsets(
    offsets: &mut Vec<usize>,
    encoded_width: usize,
    original_start: usize,
    original_end: usize,
) {
    offsets.extend(
        std::iter::repeat_n(original_start, encoded_width.saturating_sub(1))
            .chain(std::iter::once(original_end)),
    );
}

fn transcode_utf16<'a>(
    input: &'a [u8],
    content: &[u8],
    endianness: Utf16Endianness,
) -> Result<ParserInput<'a>, TranscodeFailure> {
    if content.len() % 2 != 0 {
        return Err(TranscodeFailure {
            offset: input.len() - 1,
            detail: "UTF-16 input ends with an incomplete code unit".to_owned(),
        });
    }

    let code_units = content.chunks_exact(2).map(|bytes| match endianness {
        Utf16Endianness::Little => u16::from_le_bytes([bytes[0], bytes[1]]),
        Utf16Endianness::Big => u16::from_be_bytes([bytes[0], bytes[1]]),
    });
    let mut utf8 = Vec::with_capacity(content.len() / 2);
    let mut original_offsets = vec![2];
    let mut original_offset = 2usize;
    for decoded in char::decode_utf16(code_units) {
        let character = decoded.map_err(|error| TranscodeFailure {
            offset: original_offset,
            detail: format!("invalid UTF-16 surrogate: {error}"),
        })?;
        let original_width = if character.len_utf16() == 2 { 4 } else { 2 };
        let mut encoded = [0u8; 4];
        let encoded = character.encode_utf8(&mut encoded).as_bytes();
        utf8.extend_from_slice(encoded);
        original_offsets.extend(
            std::iter::repeat_n(original_offset, encoded.len().saturating_sub(1))
                .chain(std::iter::once(original_offset + original_width)),
        );
        original_offset += original_width;
    }
    Ok(ParserInput {
        bytes: Cow::Owned(utf8),
        original_offsets: Some(original_offsets),
    })
}

#[derive(Clone, Copy)]
enum Utf16Endianness {
    Little,
    Big,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcodes_declared_iso_2022_jp_and_retains_monotonic_source_offsets() {
        let xml = "<?xml version=\"1.0\" encoding=\"ISO-2022-JP\"?><root>日本語</root>";
        let (encoded, _, had_errors) = ISO_2022_JP.encode(xml);
        assert!(!had_errors);

        let input = ParserInput::new(&encoded).expect("ISO-2022-JP should transcode");
        assert_eq!(input.transcoded_utf8(), Some(xml));
        let offsets = input
            .original_offsets
            .as_ref()
            .expect("transcoded input retains an offset map");
        assert_eq!(offsets.len(), xml.len() + 1);
        assert_eq!(offsets.first(), Some(&0));
        assert_eq!(offsets.last(), Some(&encoded.len()));
        assert!(offsets.windows(2).all(|pair| pair[0] <= pair[1]));
    }

    #[test]
    fn leaves_iso_2022_jp_text_without_a_declaration_on_the_parser_lane() {
        let input = ParserInput::new(b"<root>plain ASCII</root>").expect("ASCII XML");
        assert!(input.transcoded_utf8().is_none());
        assert_eq!(input.bytes(), b"<root>plain ASCII</root>");
    }
}
