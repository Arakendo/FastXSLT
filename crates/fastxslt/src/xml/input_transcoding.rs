use std::borrow::Cow;

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
