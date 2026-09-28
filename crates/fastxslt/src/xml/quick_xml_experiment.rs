use std::collections::HashSet;
use std::ops::Range;

use quick_xml::XmlVersion;
use quick_xml::encoding::Decoder;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::reader::NsReader;

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ParseLimits {
    pub(crate) max_events: usize,
    pub(crate) max_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ExpandedName {
    pub(crate) namespace: Option<String>,
    pub(crate) local: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct XmlAttribute {
    pub(crate) name: ExpandedName,
    pub(crate) prefix: Option<String>,
    pub(crate) value: String,
    pub(crate) span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NamespaceBinding {
    pub(crate) prefix: Option<String>,
    pub(crate) namespace: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OwnedXmlEvent {
    Start {
        name: ExpandedName,
        prefix: Option<String>,
        attributes: Vec<XmlAttribute>,
        namespaces: Vec<NamespaceBinding>,
        span: Range<usize>,
    },
    End {
        name: ExpandedName,
        span: Range<usize>,
    },
    Text {
        value: String,
        span: Range<usize>,
    },
    Comment {
        value: String,
        span: Range<usize>,
    },
    ProcessingInstruction {
        target: String,
        value: String,
        span: Range<usize>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct ParsedDocument {
    pub(crate) resource: String,
    pub(crate) events: Vec<OwnedXmlEvent>,
    root: ExpandedName,
    root_span: Range<usize>,
    root_attributes: Vec<ExpandedName>,
    element_count: usize,
    comment_count: usize,
    processing_instruction_count: usize,
}

impl ExpandedName {
    fn owned_capacity_bytes(&self) -> usize {
        self.local.capacity()
            + self
                .namespace
                .as_ref()
                .map_or(0, std::string::String::capacity)
    }
}

impl XmlAttribute {
    fn owned_capacity_bytes(&self) -> usize {
        self.name.owned_capacity_bytes()
            + self.prefix.as_ref().map_or(0, String::capacity)
            + self.value.capacity()
    }
}

impl OwnedXmlEvent {
    fn nested_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Start {
                name,
                prefix,
                attributes,
                namespaces,
                ..
            } => {
                name.owned_capacity_bytes()
                    + prefix.as_ref().map_or(0, String::capacity)
                    + attributes.capacity() * std::mem::size_of::<XmlAttribute>()
                    + attributes
                        .iter()
                        .map(XmlAttribute::owned_capacity_bytes)
                        .sum::<usize>()
                    + namespaces.capacity() * std::mem::size_of::<NamespaceBinding>()
                    + namespaces
                        .iter()
                        .map(|binding| {
                            binding.prefix.as_ref().map_or(0, String::capacity)
                                + binding.namespace.capacity()
                        })
                        .sum::<usize>()
            }
            Self::End { name, .. } => name.owned_capacity_bytes(),
            Self::Text { value, .. } | Self::Comment { value, .. } => value.capacity(),
            Self::ProcessingInstruction { target, value, .. } => {
                target.capacity() + value.capacity()
            }
        }
    }
}

impl ParsedDocument {
    pub(crate) fn owned_capacity_bytes(&self) -> usize {
        let event_storage = self.events.capacity() * std::mem::size_of::<OwnedXmlEvent>();
        let event_payloads = self
            .events
            .iter()
            .map(OwnedXmlEvent::nested_owned_capacity_bytes)
            .sum::<usize>();
        let root_attribute_storage =
            self.root_attributes.capacity() * std::mem::size_of::<ExpandedName>();
        let root_attribute_payloads = self
            .root_attributes
            .iter()
            .map(ExpandedName::owned_capacity_bytes)
            .sum::<usize>();

        std::mem::size_of::<Self>()
            + self.resource.capacity()
            + event_storage
            + event_payloads
            + self.root.owned_capacity_bytes()
            + root_attribute_storage
            + root_attribute_payloads
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ParseFailure {
    Malformed { offset: usize, detail: String },
    DtdForbidden { span: Range<usize> },
    UnknownNamespacePrefix { offset: usize, prefix: Vec<u8> },
    UnknownEntity { offset: usize, name: Vec<u8> },
    MultipleRoots { span: Range<usize> },
    MissingRoot,
    ContentOutsideRoot { span: Range<usize> },
    EventLimit { limit: usize, offset: usize },
    DepthLimit { limit: usize, span: Range<usize> },
    Control(ControlFailure),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LocatedFailure {
    resource: String,
    failure: ParseFailure,
}

impl LocatedFailure {
    pub(crate) fn control_failure(&self) -> Option<&ControlFailure> {
        match &self.failure {
            ParseFailure::Control(failure) => Some(failure),
            _ => None,
        }
    }

    pub(crate) fn source_span(&self) -> Option<Range<usize>> {
        match &self.failure {
            ParseFailure::Malformed { offset, .. }
            | ParseFailure::UnknownNamespacePrefix { offset, .. }
            | ParseFailure::UnknownEntity { offset, .. }
            | ParseFailure::EventLimit { offset, .. } => Some(*offset..*offset),
            ParseFailure::DtdForbidden { span }
            | ParseFailure::MultipleRoots { span }
            | ParseFailure::ContentOutsideRoot { span }
            | ParseFailure::DepthLimit { span, .. } => Some(span.clone()),
            ParseFailure::MissingRoot => Some(0..0),
            ParseFailure::Control(_) => None,
        }
    }

    pub(crate) fn structural_limit_detail(&self) -> Option<String> {
        match &self.failure {
            ParseFailure::EventLimit { limit, .. } => Some(format!("XML event limit is {limit}")),
            ParseFailure::DepthLimit { limit, .. } => Some(format!("XML depth limit is {limit}")),
            _ => None,
        }
    }
}

pub(crate) fn parse_document(
    resource: &str,
    input: &[u8],
    limits: ParseLimits,
) -> Result<ParsedDocument, LocatedFailure> {
    let mut control = InvocationControl::unbounded();
    parse_document_controlled(resource, input, limits, &mut control)
}

pub(crate) fn parse_document_controlled(
    resource: &str,
    input: &[u8],
    limits: ParseLimits,
    control: &mut InvocationControl,
) -> Result<ParsedDocument, LocatedFailure> {
    parse_bytes(input, limits, control)
        .map(|mut document| {
            resource.clone_into(&mut document.resource);
            document
        })
        .map_err(|failure| LocatedFailure {
            resource: resource.to_owned(),
            failure,
        })
}

enum ParserInput<'a> {
    Original(&'a [u8]),
    Utf16 {
        utf8: Vec<u8>,
        original_offsets: Vec<usize>,
    },
}

impl<'a> ParserInput<'a> {
    fn new(input: &'a [u8]) -> Result<Self, ParseFailure> {
        let (endianness, content) = if let Some(content) = input.strip_prefix(&[0xff, 0xfe]) {
            (Utf16Endianness::Little, content)
        } else if let Some(content) = input.strip_prefix(&[0xfe, 0xff]) {
            (Utf16Endianness::Big, content)
        } else {
            return Ok(Self::Original(input));
        };
        if content.len() % 2 != 0 {
            return Err(ParseFailure::Malformed {
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
            let character = decoded.map_err(|error| ParseFailure::Malformed {
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
        Ok(Self::Utf16 {
            utf8,
            original_offsets,
        })
    }

    fn bytes(&self) -> &[u8] {
        match self {
            Self::Original(bytes) => bytes,
            Self::Utf16 { utf8, .. } => utf8,
        }
    }

    fn original_offset(&self, parser_offset: usize) -> usize {
        match self {
            Self::Original(_) => parser_offset,
            Self::Utf16 {
                original_offsets, ..
            } => original_offsets
                .get(parser_offset)
                .copied()
                .unwrap_or_else(|| *original_offsets.last().unwrap_or(&usize::MAX)),
        }
    }
}

#[derive(Clone, Copy)]
enum Utf16Endianness {
    Little,
    Big,
}

#[allow(
    clippy::too_many_lines,
    reason = "keeping the experimental event loop together makes parser behavior auditable"
)]
fn parse_bytes(
    input: &[u8],
    limits: ParseLimits,
    control: &mut InvocationControl,
) -> Result<ParsedDocument, ParseFailure> {
    let parser_input = ParserInput::new(input)?;
    let mut reader = NsReader::from_reader(parser_input.bytes());
    reader.config_mut().enable_all_checks(true);

    let mut depth = 0_usize;
    let mut event_count = 0_usize;
    let mut element_count = 0_usize;
    let mut root = None;
    let mut root_span = None;
    let mut root_attributes = Vec::new();
    let mut comment_count = 0_usize;
    let mut processing_instruction_count = 0_usize;
    let mut events = Vec::new();

    loop {
        let parser_start = usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX);
        let event = reader
            .read_event()
            .map_err(|error| ParseFailure::Malformed {
                offset: parser_input.original_offset(
                    usize::try_from(reader.error_position()).unwrap_or(usize::MAX),
                ),
                detail: error.to_string(),
            })?;
        let parser_end = usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX);
        let start = parser_input.original_offset(parser_start);
        let end = parser_input.original_offset(parser_end);
        let span = start..end;

        if !matches!(event, Event::Eof) {
            control
                .charge(WorkDomain::XmlEvent, 1)
                .map_err(ParseFailure::Control)?;
            event_count = event_count.saturating_add(1);
            if event_count > limits.max_events {
                return Err(ParseFailure::EventLimit {
                    limit: limits.max_events,
                    offset: start,
                });
            }
        }

        match event {
            Event::Start(element) => {
                let name = resolve_element_name(&reader, &element, start)?;
                let prefix = lexical_prefix(reader.decoder(), element.name().as_ref(), start)?;
                let attributes = resolve_attributes(&reader, &element, start)?;
                let namespaces = resolve_namespace_declarations(&reader, &element, start)?;
                if depth == 0 {
                    if root.is_some() {
                        return Err(ParseFailure::MultipleRoots { span });
                    }
                    root = Some(name.clone());
                    root_span = Some(span.clone());
                    root_attributes = attributes
                        .iter()
                        .map(|attribute| attribute.name.clone())
                        .collect();
                }
                if depth >= limits.max_depth {
                    return Err(ParseFailure::DepthLimit {
                        limit: limits.max_depth,
                        span,
                    });
                }
                depth += 1;
                element_count += 1;
                events.push(OwnedXmlEvent::Start {
                    name,
                    prefix,
                    attributes,
                    namespaces,
                    span,
                });
            }
            Event::Empty(element) => {
                let name = resolve_element_name(&reader, &element, start)?;
                let prefix = lexical_prefix(reader.decoder(), element.name().as_ref(), start)?;
                let attributes = resolve_attributes(&reader, &element, start)?;
                let namespaces = resolve_namespace_declarations(&reader, &element, start)?;
                if depth == 0 {
                    if root.is_some() {
                        return Err(ParseFailure::MultipleRoots { span });
                    }
                    root = Some(name.clone());
                    root_span = Some(span.clone());
                    root_attributes = attributes
                        .iter()
                        .map(|attribute| attribute.name.clone())
                        .collect();
                }
                if depth >= limits.max_depth {
                    return Err(ParseFailure::DepthLimit {
                        limit: limits.max_depth,
                        span: start..end,
                    });
                }
                element_count += 1;
                events.push(OwnedXmlEvent::Start {
                    name: name.clone(),
                    prefix,
                    attributes,
                    namespaces,
                    span: span.clone(),
                });
                events.push(OwnedXmlEvent::End { name, span });
            }
            Event::End(element) => {
                let name = resolve_end_name(&reader, element.name().as_ref(), start)?;
                depth = depth.saturating_sub(1);
                events.push(OwnedXmlEvent::End { name, span });
            }
            Event::Text(text) => {
                let value = text
                    .xml10_content()
                    .map_err(|error| malformed(start, error))?
                    .into_owned();
                if depth == 0 && !text.iter().all(u8::is_ascii_whitespace) {
                    return Err(ParseFailure::ContentOutsideRoot { span });
                }
                if depth > 0 {
                    events.push(OwnedXmlEvent::Text { value, span });
                }
            }
            Event::CData(text) => {
                let value = text
                    .xml10_content()
                    .map_err(|error| malformed(start, error))?
                    .into_owned();
                if depth == 0 {
                    return Err(ParseFailure::ContentOutsideRoot { span });
                }
                events.push(OwnedXmlEvent::Text { value, span });
            }
            Event::GeneralRef(reference) => {
                if depth == 0 {
                    return Err(ParseFailure::ContentOutsideRoot { span });
                }
                let reference_bytes: &[u8] = reference.as_ref();
                if reference.resolve_char_ref().ok().flatten().is_none()
                    && !matches!(reference_bytes, b"lt" | b"gt" | b"amp" | b"apos" | b"quot")
                {
                    return Err(ParseFailure::UnknownEntity {
                        offset: start,
                        name: reference_bytes.to_vec(),
                    });
                }
                events.push(OwnedXmlEvent::Text {
                    value: resolve_reference(&reference, start)?,
                    span,
                });
            }
            Event::DocType(_) => return Err(ParseFailure::DtdForbidden { span }),
            Event::Comment(comment) => {
                let value = comment
                    .xml10_content()
                    .map_err(|error| malformed(start, error))?
                    .into_owned();
                comment_count += 1;
                events.push(OwnedXmlEvent::Comment { value, span });
            }
            Event::PI(instruction) => {
                let target = decode_name(reader.decoder(), instruction.target(), start)?;
                let value = reader
                    .decoder()
                    .decode(instruction.content())
                    .map_err(|error| malformed(start, error))?
                    .trim_start_matches([' ', '\t', '\r', '\n'])
                    .to_owned();
                processing_instruction_count += 1;
                events.push(OwnedXmlEvent::ProcessingInstruction {
                    target,
                    value,
                    span,
                });
            }
            Event::Eof => break,
            Event::Decl(declaration) => {
                std::str::from_utf8(declaration.as_ref())
                    .map_err(|error| malformed(start, error))?;
            }
        }
    }

    let root = root.ok_or(ParseFailure::MissingRoot)?;
    Ok(ParsedDocument {
        resource: String::new(),
        events,
        root,
        root_span: root_span.expect("a root event always records its span"),
        root_attributes,
        element_count,
        comment_count,
        processing_instruction_count,
    })
}

fn resolve_element_name(
    reader: &NsReader<&[u8]>,
    element: &BytesStart<'_>,
    offset: usize,
) -> Result<ExpandedName, ParseFailure> {
    let (namespace, local) = reader.resolver().resolve_element(element.name());
    expanded_name(reader.decoder(), namespace, local.as_ref(), offset)
}

fn lexical_prefix(
    decoder: Decoder,
    name: &[u8],
    offset: usize,
) -> Result<Option<String>, ParseFailure> {
    name.iter()
        .position(|byte| *byte == b':')
        .map(|separator| decode_name(decoder, &name[..separator], offset))
        .transpose()
}

fn resolve_end_name(
    reader: &NsReader<&[u8]>,
    name: &[u8],
    offset: usize,
) -> Result<ExpandedName, ParseFailure> {
    let qualified = quick_xml::name::QName(name);
    let (namespace, local) = reader.resolver().resolve_element(qualified);
    expanded_name(reader.decoder(), namespace, local.as_ref(), offset)
}

fn resolve_attributes(
    reader: &NsReader<&[u8]>,
    element: &BytesStart<'_>,
    offset: usize,
) -> Result<Vec<XmlAttribute>, ParseFailure> {
    let mut names = Vec::new();
    let mut expanded_names = HashSet::new();

    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| ParseFailure::Malformed {
            offset,
            detail: error.to_string(),
        })?;
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
            .map_err(|error| malformed(offset, error))?
            .into_owned();
        if attribute.key.as_ref() == b"xmlns" || attribute.key.as_ref().starts_with(b"xmlns:") {
            continue;
        }
        let (namespace, local) = reader.resolver().resolve_attribute(attribute.key);
        let name = expanded_name(reader.decoder(), namespace, local.as_ref(), offset)?;
        let prefix = lexical_prefix(reader.decoder(), attribute.key.as_ref(), offset)?;
        if !expanded_names.insert(name.clone()) {
            return Err(ParseFailure::Malformed {
                offset,
                detail: format!("duplicate expanded attribute name: {name:?}"),
            });
        }
        names.push(XmlAttribute {
            name,
            prefix,
            value,
            span: offset..usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX),
        });
    }
    Ok(names)
}

fn resolve_namespace_declarations(
    reader: &NsReader<&[u8]>,
    element: &BytesStart<'_>,
    offset: usize,
) -> Result<Vec<NamespaceBinding>, ParseFailure> {
    let mut bindings = Vec::new();
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| ParseFailure::Malformed {
            offset,
            detail: error.to_string(),
        })?;
        let key = attribute.key.as_ref();
        let prefix = if key == b"xmlns" {
            None
        } else if let Some(prefix) = key.strip_prefix(b"xmlns:") {
            Some(decode_name(reader.decoder(), prefix, offset)?)
        } else {
            continue;
        };
        let namespace = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
            .map_err(|error| malformed(offset, error))?
            .into_owned();
        bindings.push(NamespaceBinding { prefix, namespace });
    }
    Ok(bindings)
}

fn expanded_name(
    decoder: Decoder,
    namespace: ResolveResult<'_>,
    local: &[u8],
    offset: usize,
) -> Result<ExpandedName, ParseFailure> {
    let namespace = match namespace {
        ResolveResult::Unbound => None,
        ResolveResult::Bound(namespace) => Some(decode_resolved_namespace(
            decoder,
            namespace.as_ref(),
            offset,
        )?),
        ResolveResult::Unknown(prefix) => {
            return Err(ParseFailure::UnknownNamespacePrefix { offset, prefix });
        }
    };
    Ok(ExpandedName {
        namespace,
        local: decode_name(decoder, local, offset)?,
    })
}

fn decode_resolved_namespace(
    decoder: Decoder,
    namespace: &[u8],
    offset: usize,
) -> Result<String, ParseFailure> {
    let raw = decoder
        .decode(namespace)
        .map_err(|error| malformed(offset, error))?;
    let mut normalized = String::with_capacity(raw.len());
    let mut characters = raw.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\r' => {
                if characters.peek() == Some(&'\n') {
                    characters.next();
                }
                normalized.push(' ');
            }
            '\n' | '\t' => normalized.push(' '),
            _ => normalized.push(character),
        }
    }
    quick_xml::escape::unescape(&normalized)
        .map(std::borrow::Cow::into_owned)
        .map_err(|error| malformed(offset, error))
}

fn decode_name(decoder: Decoder, bytes: &[u8], offset: usize) -> Result<String, ParseFailure> {
    decoder
        .decode(bytes)
        .map(std::borrow::Cow::into_owned)
        .map_err(|error| malformed(offset, error))
}

fn malformed(offset: usize, error: impl std::fmt::Display) -> ParseFailure {
    ParseFailure::Malformed {
        offset,
        detail: error.to_string(),
    }
}

fn resolve_reference(
    reference: &quick_xml::events::BytesRef<'_>,
    offset: usize,
) -> Result<String, ParseFailure> {
    if let Some(character) = reference
        .resolve_char_ref()
        .map_err(|error| malformed(offset, error))?
    {
        return Ok(character.to_string());
    }
    let reference_bytes: &[u8] = reference.as_ref();
    let character = match reference_bytes {
        b"lt" => '<',
        b"gt" => '>',
        b"amp" => '&',
        b"apos" => '\'',
        b"quot" => '"',
        name => {
            return Err(ParseFailure::UnknownEntity {
                offset,
                name: name.to_vec(),
            });
        }
    };
    Ok(character.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        ExpandedName, LocatedFailure, OwnedXmlEvent, ParseFailure, ParseLimits, parse_document,
    };

    const LIMITS: ParseLimits = ParseLimits {
        max_events: 64,
        max_depth: 8,
    };

    #[test]
    fn resolves_element_and_attribute_namespaces_and_retains_root_span() {
        let xml =
            br#"<root xmlns="urn:default" xmlns:p="urn:p" plain="x" p:item="y"><p:child/></root>"#;

        let document =
            parse_document("memory:source.xml", xml, LIMITS).expect("namespaced XML should parse");

        assert_eq!(document.resource, "memory:source.xml");
        assert_eq!(
            document.root,
            ExpandedName {
                namespace: Some("urn:default".to_owned()),
                local: "root".to_owned(),
            }
        );
        assert_eq!(document.root_span, 0..63);
        assert_eq!(
            document.root_attributes,
            vec![
                ExpandedName {
                    namespace: None,
                    local: "plain".to_owned(),
                },
                ExpandedName {
                    namespace: Some("urn:p".to_owned()),
                    local: "item".to_owned(),
                },
            ]
        );
        assert_eq!(document.element_count, 2);
    }

    #[test]
    fn normalizes_namespace_declaration_values_before_name_resolution() {
        let document = parse_document(
            "memory:namespace-normalization.xml",
            b"<root xmlns=\"urn:a&amp;b\r\nc\"><child/></root>",
            LIMITS,
        )
        .expect("namespace declaration should parse");

        let expected = ExpandedName {
            namespace: Some("urn:a&b c".to_owned()),
            local: "root".to_owned(),
        };
        assert_eq!(document.root, expected);
        assert!(document.events.iter().any(|event| {
            matches!(
                event,
                OwnedXmlEvent::Start { name, .. } if name == &ExpandedName {
                    namespace: Some("urn:a&b c".to_owned()),
                    local: "child".to_owned(),
                }
            )
        }));
    }

    #[test]
    fn decodes_declared_single_byte_xml_without_losing_names_or_offsets() {
        let xml = b"<?xml version=\"1.0\" encoding=\"windows-1252\"?><r\xE9sum\xE9 attr=\"caf\xE9\">ol\xE9</r\xE9sum\xE9>";

        let document = parse_document("memory:windows-1252.xml", xml, LIMITS)
            .expect("declared Windows-1252 XML should parse");

        assert_eq!(document.root.local, "résumé");
        assert_eq!(document.root_span, 45..65);
        assert!(document.events.iter().any(|event| {
            matches!(
                event,
                OwnedXmlEvent::Start { attributes, .. }
                    if attributes.iter().any(|attribute| {
                        attribute.name.local == "attr" && attribute.value == "café"
                    })
            )
        }));
        assert!(document.events.iter().any(|event| {
            matches!(event, OwnedXmlEvent::Text { value, .. } if value == "olé")
        }));
    }

    #[test]
    fn decodes_bom_selected_utf16_and_preserves_original_byte_offsets() {
        let xml = "<?xml version=\"1.0\"?><résumé>olé</résumé>";
        let root_start = xml[..xml.find("<résumé>").expect("root start")]
            .encode_utf16()
            .count();
        let root_end = root_start + "<résumé>".encode_utf16().count();

        for (bom, units) in [
            (
                [0xff, 0xfe],
                xml.encode_utf16()
                    .flat_map(u16::to_le_bytes)
                    .collect::<Vec<_>>(),
            ),
            (
                [0xfe, 0xff],
                xml.encode_utf16()
                    .flat_map(u16::to_be_bytes)
                    .collect::<Vec<_>>(),
            ),
        ] {
            let mut bytes = bom.to_vec();
            bytes.extend(units);
            let document = parse_document("memory:utf16.xml", &bytes, LIMITS)
                .expect("BOM-selected UTF-16 XML should parse");

            assert_eq!(document.root.local, "résumé");
            assert_eq!(document.root_span, 2 + root_start * 2..2 + root_end * 2);
            assert!(document.events.iter().any(|event| {
                matches!(event, OwnedXmlEvent::Text { value, .. } if value == "olé")
            }));
        }
    }

    #[test]
    fn rejects_truncated_or_unpaired_utf16_before_xml_parsing() {
        assert!(matches!(
            parse_document("memory:truncated.xml", &[0xff, 0xfe, b'<'], LIMITS),
            Err(LocatedFailure {
                failure: ParseFailure::Malformed { offset: 2, .. },
                ..
            })
        ));
        assert!(matches!(
            parse_document("memory:surrogate.xml", &[0xff, 0xfe, 0x00, 0xd8], LIMITS),
            Err(LocatedFailure {
                failure: ParseFailure::Malformed { offset: 2, .. },
                ..
            })
        ));
    }

    #[test]
    fn rejects_malformed_structure_and_duplicate_expanded_attributes() {
        assert!(matches!(
            parse_document("memory:bad.xml", b"<root><child></root>", LIMITS),
            Err(LocatedFailure {
                failure: ParseFailure::Malformed { .. },
                ..
            })
        ));
        assert!(matches!(
            parse_document(
                "memory:bad.xml",
                br#"<root xmlns:a="urn:same" xmlns:b="urn:same" a:value="1" b:value="2"/>"#,
                LIMITS
            ),
            Err(LocatedFailure {
                failure: ParseFailure::Malformed { .. },
                ..
            })
        ));
        assert!(matches!(
            parse_document("memory:bad.xml", b"<one/><two/>", LIMITS),
            Err(LocatedFailure {
                failure: ParseFailure::MultipleRoots { .. },
                ..
            })
        ));
    }

    #[test]
    fn rejects_dtds_unknown_entities_and_unknown_namespace_prefixes() {
        assert_eq!(
            parse_document(
                "memory:hostile.xml",
                b"<!DOCTYPE root SYSTEM 'file:///secret'><root/>",
                LIMITS
            ),
            Err(LocatedFailure {
                resource: "memory:hostile.xml".to_owned(),
                failure: ParseFailure::DtdForbidden { span: 0..39 },
            })
        );
        assert!(matches!(
            parse_document("memory:hostile.xml", b"<root>&secret;</root>", LIMITS),
            Err(LocatedFailure {
                failure: ParseFailure::UnknownEntity { .. },
                ..
            })
        ));
        assert!(matches!(
            parse_document("memory:hostile.xml", b"<root value='&secret;'/>", LIMITS),
            Err(LocatedFailure {
                failure: ParseFailure::Malformed { .. },
                ..
            })
        ));
        assert!(matches!(
            parse_document("memory:hostile.xml", b"<missing:root/>", LIMITS),
            Err(LocatedFailure {
                failure: ParseFailure::UnknownNamespacePrefix { .. },
                ..
            })
        ));
    }

    #[test]
    fn preserves_comments_and_processing_instructions_as_semantic_pressure() {
        let document = parse_document(
            "memory:nodes.xml",
            b"<?before work?><root><!--inside--><?nested work?></root><!--after-->",
            LIMITS,
        )
        .expect("comments and processing instructions are legal document nodes");

        assert_eq!(document.comment_count, 2);
        assert_eq!(document.processing_instruction_count, 2);
        let processing_instruction_values: Vec<_> = document
            .events
            .iter()
            .filter_map(|event| match event {
                OwnedXmlEvent::ProcessingInstruction { value, .. } => Some(value.as_str()),
                _ => None,
            })
            .collect();
        assert_eq!(processing_instruction_values, ["work", "work"]);
    }

    #[test]
    fn enforces_event_and_depth_limits() {
        assert_eq!(
            parse_document(
                "memory:deep.xml",
                b"<root><one/><two/></root>",
                ParseLimits {
                    max_events: 2,
                    max_depth: 8,
                },
            ),
            Err(LocatedFailure {
                resource: "memory:deep.xml".to_owned(),
                failure: ParseFailure::EventLimit {
                    limit: 2,
                    offset: 12,
                },
            })
        );
        assert!(matches!(
            parse_document(
                "memory:deep.xml",
                b"<root><one><two/></one></root>",
                ParseLimits {
                    max_events: 64,
                    max_depth: 2,
                },
            ),
            Err(LocatedFailure {
                failure: ParseFailure::DepthLimit { limit: 2, .. },
                ..
            })
        ));
    }
}
