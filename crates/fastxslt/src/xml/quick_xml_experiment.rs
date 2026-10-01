use std::collections::HashSet;
use std::ops::Range;

use quick_xml::XmlVersion;
use quick_xml::encoding::Decoder;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::{QName, ResolveResult};
use quick_xml::reader::NsReader;

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xml::input_transcoding::ParserInput;
use crate::xml::internal_subset::{
    DeclaredAttributeType, InternalEntities, InternalSubsetFailure, InternalSubsetLimits,
    parse_internal_subset,
};

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
    pub(crate) is_id: bool,
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
    DtdUnsupported { span: Range<usize>, detail: String },
    DtdLimit { span: Range<usize>, detail: String },
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
            | ParseFailure::DtdUnsupported { span, .. }
            | ParseFailure::DtdLimit { span, .. }
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
            ParseFailure::DtdLimit { detail, .. } => Some(detail.clone()),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(crate) const fn dtd_reference_category(&self) -> &'static str {
        match self.failure {
            ParseFailure::DtdUnsupported { .. } => "unsupported-declaration-semantics",
            ParseFailure::DtdLimit { .. } => "dtd-limit",
            ParseFailure::Malformed { .. } => "malformed-xml-or-dtd",
            ParseFailure::UnknownEntity { .. } => "unknown-entity",
            ParseFailure::Control(_) => "control",
            ParseFailure::DtdForbidden { .. } => "unexpected-default-denial",
            ParseFailure::UnknownNamespacePrefix { .. }
            | ParseFailure::MultipleRoots { .. }
            | ParseFailure::MissingRoot
            | ParseFailure::ContentOutsideRoot { .. }
            | ParseFailure::EventLimit { .. }
            | ParseFailure::DepthLimit { .. } => "other-xml-failure",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum DtdPolicy {
    Deny,
    #[cfg_attr(
        not(test),
        allow(
            dead_code,
            reason = "AR-0025 reference path is not admitted by production callers"
        )
    )]
    BoundedInternalSubset(InternalSubsetLimits),
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
    parse_bytes(input, limits, DtdPolicy::Deny, control)
        .map(|mut document| {
            resource.clone_into(&mut document.resource);
            document
        })
        .map_err(|failure| LocatedFailure {
            resource: resource.to_owned(),
            failure,
        })
}

pub(crate) fn parse_document_with_internal_subset(
    resource: &str,
    input: &[u8],
    limits: ParseLimits,
    dtd_limits: InternalSubsetLimits,
) -> Result<ParsedDocument, LocatedFailure> {
    let mut control = InvocationControl::unbounded();
    parse_document_controlled_with_internal_subset(
        resource,
        input,
        limits,
        dtd_limits,
        &mut control,
    )
}

pub(crate) fn parse_document_controlled_with_internal_subset(
    resource: &str,
    input: &[u8],
    limits: ParseLimits,
    dtd_limits: InternalSubsetLimits,
    control: &mut InvocationControl,
) -> Result<ParsedDocument, LocatedFailure> {
    parse_bytes(
        input,
        limits,
        DtdPolicy::BoundedInternalSubset(dtd_limits),
        control,
    )
    .map(|mut document| {
        resource.clone_into(&mut document.resource);
        document
    })
    .map_err(|failure| LocatedFailure {
        resource: resource.to_owned(),
        failure,
    })
}

#[allow(
    clippy::too_many_lines,
    reason = "keeping the experimental event loop together makes parser behavior auditable"
)]
fn parse_bytes(
    input: &[u8],
    limits: ParseLimits,
    dtd_policy: DtdPolicy,
    control: &mut InvocationControl,
) -> Result<ParsedDocument, ParseFailure> {
    let parser_input = ParserInput::new(input).map_err(|failure| ParseFailure::Malformed {
        offset: failure.offset,
        detail: failure.detail,
    })?;
    // `from_str` fixes quick-xml's decoder to UTF-8 for the transcoded lane.
    // Otherwise a retained `encoding="UTF-16"` XML declaration would make the
    // reader reinterpret the already-transcoded buffer as UTF-16.
    let mut reader = if let Some(utf8) = parser_input.transcoded_utf8() {
        NsReader::from_str(utf8)
    } else {
        NsReader::from_reader(parser_input.bytes())
    };
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
    let mut internal_entities = None;

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
                let attributes = resolve_attributes(
                    &reader,
                    &element,
                    start,
                    internal_entities.as_ref(),
                    control,
                )?;
                let namespaces = resolve_namespace_declarations(
                    &reader,
                    &element,
                    start,
                    internal_entities.as_ref(),
                    control,
                )?;
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
                let attributes = resolve_attributes(
                    &reader,
                    &element,
                    start,
                    internal_entities.as_ref(),
                    control,
                )?;
                let namespaces = resolve_namespace_declarations(
                    &reader,
                    &element,
                    start,
                    internal_entities.as_ref(),
                    control,
                )?;
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
                let value = if reference.resolve_char_ref().ok().flatten().is_some()
                    || matches!(reference_bytes, b"lt" | b"gt" | b"amp" | b"apos" | b"quot")
                {
                    resolve_reference(&reference, start)?
                } else if let Some(entities) = internal_entities.as_ref() {
                    let name = reader
                        .decoder()
                        .decode(reference_bytes)
                        .map_err(|error| malformed(start, error))?;
                    let before = entities.reference_count();
                    let value = entities
                        .resolve(&name)
                        .map_err(|failure| map_dtd_failure(failure, span.clone(), start))?
                        .to_owned();
                    charge_entity_references(
                        control,
                        entities
                            .reference_count()
                            .saturating_sub(before)
                            .saturating_sub(1),
                    )?;
                    value
                } else {
                    return Err(ParseFailure::UnknownEntity {
                        offset: start,
                        name: reference_bytes.to_vec(),
                    });
                };
                events.push(OwnedXmlEvent::Text { value, span });
            }
            Event::DocType(doctype) => match dtd_policy {
                DtdPolicy::Deny => return Err(ParseFailure::DtdForbidden { span }),
                DtdPolicy::BoundedInternalSubset(dtd_limits) => {
                    if internal_entities.is_some() {
                        return Err(ParseFailure::Malformed {
                            offset: start,
                            detail: "multiple DOCTYPE declarations are not permitted".to_owned(),
                        });
                    }
                    let declaration = reader
                        .decoder()
                        .decode(doctype.as_ref())
                        .map_err(|error| malformed(start, error))?;
                    let entities = parse_internal_subset(&declaration, dtd_limits)
                        .map_err(|failure| map_dtd_failure(failure, span, start))?;
                    charge_entity_references(control, entities.declaration_work())?;
                    internal_entities = Some(entities);
                }
            },
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
    entities: Option<&InternalEntities>,
    control: &mut InvocationControl,
) -> Result<Vec<XmlAttribute>, ParseFailure> {
    let mut names = Vec::new();
    let mut expanded_names = HashSet::new();
    let mut lexical_names = HashSet::new();
    let lexical_element_name = decode_name(reader.decoder(), element.name().as_ref(), offset)?;

    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| ParseFailure::Malformed {
            offset,
            detail: error.to_string(),
        })?;
        let lexical_attribute_name = decode_name(reader.decoder(), attribute.key.as_ref(), offset)?;
        lexical_names.insert(lexical_attribute_name.clone());
        let mut value = resolve_attribute_value(reader, &attribute, offset, entities, control)?;
        if attribute.key.as_ref() == b"xmlns" || attribute.key.as_ref().starts_with(b"xmlns:") {
            continue;
        }
        let (namespace, local) = reader.resolver().resolve_attribute(attribute.key);
        let name = expanded_name(reader.decoder(), namespace, local.as_ref(), offset)?;
        let prefix = lexical_prefix(reader.decoder(), attribute.key.as_ref(), offset)?;
        let declared_type = entities.and_then(|entities| {
            entities.attribute_type(&lexical_element_name, &lexical_attribute_name)
        });
        if matches!(
            declared_type,
            Some(DeclaredAttributeType::Id | DeclaredAttributeType::IdRef)
        ) {
            value = normalize_xml_tokenized_attribute(&value);
        }
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
            is_id: declared_type == Some(DeclaredAttributeType::Id),
            span: offset..usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX),
        });
    }
    if let Some(entities) = entities {
        for attribute in entities
            .default_attributes(&lexical_element_name, &lexical_names)
            .map_err(|failure| {
                map_dtd_failure(
                    failure,
                    offset..usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX),
                    offset,
                )
            })?
        {
            if attribute.name == "xmlns" || attribute.name.starts_with("xmlns:") {
                return Err(ParseFailure::DtdUnsupported {
                    span: offset..usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX),
                    detail: "DTD-derived namespace declarations require pre-tokenization namespace semantics"
                        .to_owned(),
                });
            }
            let qualified = QName(attribute.name.as_bytes());
            let (namespace, local) = reader.resolver().resolve_attribute(qualified);
            let name = expanded_name(reader.decoder(), namespace, local.as_ref(), offset)?;
            if !expanded_names.insert(name.clone()) {
                return Err(ParseFailure::Malformed {
                    offset,
                    detail: format!("duplicate expanded attribute name: {name:?}"),
                });
            }
            names.push(XmlAttribute {
                name,
                prefix: attribute
                    .name
                    .split_once(':')
                    .map(|(prefix, _)| prefix.to_owned()),
                value: attribute.value,
                is_id: attribute.declared_type == DeclaredAttributeType::Id,
                span: offset..usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX),
            });
        }
    }
    Ok(names)
}

fn normalize_xml_tokenized_attribute(value: &str) -> String {
    value
        .split([' ', '\t', '\r', '\n'])
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn resolve_namespace_declarations(
    reader: &NsReader<&[u8]>,
    element: &BytesStart<'_>,
    offset: usize,
    entities: Option<&InternalEntities>,
    control: &mut InvocationControl,
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
        let namespace = resolve_attribute_value(reader, &attribute, offset, entities, control)?;
        bindings.push(NamespaceBinding { prefix, namespace });
    }
    Ok(bindings)
}

fn resolve_attribute_value(
    reader: &NsReader<&[u8]>,
    attribute: &quick_xml::events::attributes::Attribute<'_>,
    offset: usize,
    entities: Option<&InternalEntities>,
    control: &mut InvocationControl,
) -> Result<String, ParseFailure> {
    let Some(entities) = entities else {
        return attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
            .map(std::borrow::Cow::into_owned)
            .map_err(|error| malformed(offset, error));
    };
    let before = entities.reference_count();
    let mut entity_failure = None;
    let value = attribute.decoded_and_normalized_value_with(
        XmlVersion::Implicit1_0,
        reader.decoder(),
        usize::MAX,
        |name| match entities.resolve(name) {
            Ok(value) => Some(value),
            Err(failure) => {
                entity_failure = Some(failure);
                None
            }
        },
    );
    if let Some(failure) = entity_failure {
        return Err(map_dtd_failure(failure, offset..offset, offset));
    }
    let value = value
        .map_err(|error| malformed(offset, error))?
        .into_owned();
    charge_entity_references(control, entities.reference_count().saturating_sub(before))?;
    Ok(value)
}

fn charge_entity_references(
    control: &mut InvocationControl,
    references: usize,
) -> Result<(), ParseFailure> {
    control
        .charge(WorkDomain::XmlEvent, references)
        .map_err(ParseFailure::Control)
}

fn map_dtd_failure(
    failure: InternalSubsetFailure,
    span: Range<usize>,
    offset: usize,
) -> ParseFailure {
    match failure {
        InternalSubsetFailure::Malformed(detail) => ParseFailure::Malformed { offset, detail },
        InternalSubsetFailure::Unsupported(detail) => ParseFailure::DtdUnsupported { span, detail },
        InternalSubsetFailure::Limit(detail) => ParseFailure::DtdLimit { span, detail },
        InternalSubsetFailure::UnknownEntity(name) => ParseFailure::UnknownEntity {
            offset,
            name: name.into_bytes(),
        },
    }
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
        parse_document_controlled_with_internal_subset, parse_document_with_internal_subset,
    };
    use crate::execution_control_experiment::{
        CancellationToken, ControlFailure, InvocationControl, WorkDomain, WorkLimits,
    };
    use crate::xml::internal_subset::InternalSubsetLimits;

    const LIMITS: ParseLimits = ParseLimits {
        max_events: 64,
        max_depth: 8,
    };
    const DTD_LIMITS: InternalSubsetLimits = InternalSubsetLimits {
        declarations: 8,
        nesting_depth: 4,
        references: 16,
        replacement_bytes: 128,
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
        let xml = "<?xml version=\"1.0\" encoding=\"UTF-16\"?><résumé>olé</résumé>";
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
    fn bounded_internal_subset_expands_text_and_attributes() {
        let xml = br#"<!DOCTYPE root [
                <!ENTITY greeting "Hello">
                <!ENTITY subject "world">
            ]>
            <root message="&greeting;, &subject;!">&greeting; &subject;</root>"#;
        let document = parse_document_with_internal_subset(
            "memory:internal-subset.xml",
            xml,
            LIMITS,
            DTD_LIMITS,
        )
        .expect("bounded character-data entities should be admitted explicitly");

        assert_eq!(document.root.namespace, None);
        assert!(document.events.iter().any(|event| {
            matches!(
                event,
                OwnedXmlEvent::Start { attributes, .. }
                    if attributes.iter().any(|attribute| {
                        attribute.name.local == "message"
                            && attribute.value == "Hello, world!"
                    })
            )
        }));
        let text = document
            .events
            .iter()
            .filter_map(|event| match event {
                OwnedXmlEvent::Text { value, .. } => Some(value.as_str()),
                _ => None,
            })
            .collect::<String>();
        assert_eq!(text, "Hello world");
        let final_reference = xml
            .windows(b"&subject;".len())
            .rposition(|window| window == b"&subject;")
            .expect("authored reference");
        assert!(document.events.iter().any(|event| {
            matches!(
                event,
                OwnedXmlEvent::Text { value, span }
                    if value == "world"
                        && span == &(final_reference..final_reference + b"&subject;".len())
            )
        }));
    }

    #[test]
    fn bounded_internal_subset_marks_ids_and_normalizes_tokenized_values() {
        let document = parse_document_with_internal_subset(
            "memory:typed-attributes.xml",
            br#"<!DOCTYPE root [
                <!ELEMENT root EMPTY>
                <!ATTLIST root id ID #REQUIRED ref IDREF #IMPLIED note CDATA #IMPLIED>
            ]><root id="  target&#x9;" ref="&#xA; target  " note="  unchanged  "/>"#,
            LIMITS,
            DTD_LIMITS,
        )
        .expect("bounded typed attributes should be admitted explicitly");

        let OwnedXmlEvent::Start { attributes, .. } = &document.events[0] else {
            panic!("root start event");
        };
        let id = attributes
            .iter()
            .find(|attribute| attribute.name.local == "id")
            .expect("ID attribute");
        assert!(id.is_id);
        assert_eq!(id.value, "target");
        let reference = attributes
            .iter()
            .find(|attribute| attribute.name.local == "ref")
            .expect("IDREF attribute");
        assert!(!reference.is_id);
        assert_eq!(reference.value, "target");
        let cdata = attributes
            .iter()
            .find(|attribute| attribute.name.local == "note")
            .expect("CDATA attribute");
        assert!(!cdata.is_id);
        assert_eq!(cdata.value, "  unchanged  ");
    }

    #[test]
    fn bounded_internal_subset_injects_defaults_without_overriding_authored_attributes() {
        let document = parse_document_with_internal_subset(
            "memory:default-attributes.xml",
            br#"<!DOCTYPE root [
                <!ELEMENT root (item,item)>
                <!ELEMENT item EMPTY>
                <!ATTLIST item value CDATA "default" fixed CDATA #FIXED "fixed">
            ]><root><item/><item value="authored"/></root>"#,
            LIMITS,
            DTD_LIMITS,
        )
        .expect("bounded default attributes should be admitted explicitly");

        let item_attributes = document.events.iter().filter_map(|event| match event {
            OwnedXmlEvent::Start {
                name, attributes, ..
            } if name.local == "item" => Some(attributes),
            _ => None,
        });
        let values = item_attributes
            .map(|attributes| {
                (
                    attributes
                        .iter()
                        .find(|attribute| attribute.name.local == "value")
                        .map(|attribute| attribute.value.as_str()),
                    attributes
                        .iter()
                        .find(|attribute| attribute.name.local == "fixed")
                        .map(|attribute| attribute.value.as_str()),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            values,
            vec![
                (Some("default"), Some("fixed")),
                (Some("authored"), Some("fixed"))
            ]
        );
    }

    #[test]
    fn bounded_internal_subset_charges_each_injected_default_value() {
        let failure = parse_document_with_internal_subset(
            "memory:bounded-defaults.xml",
            br#"<!DOCTYPE root [
                <!ELEMENT root (item,item)>
                <!ELEMENT item EMPTY>
                <!ATTLIST item value CDATA "xx">
            ]><root><item/><item/></root>"#,
            LIMITS,
            InternalSubsetLimits {
                replacement_bytes: 3,
                ..DTD_LIMITS
            },
        )
        .expect_err("two injected defaults must exceed the cumulative byte limit");

        assert_eq!(
            failure.structural_limit_detail().as_deref(),
            Some("DTD replacement-byte limit is 3")
        );
    }

    #[test]
    fn bounded_internal_subset_still_denies_external_authority() {
        assert!(matches!(
            parse_document_with_internal_subset(
                "memory:no-authority.xml",
                b"<!DOCTYPE root SYSTEM 'file:///secret'><root/>",
                LIMITS,
                DTD_LIMITS,
            ),
            Err(LocatedFailure {
                failure: ParseFailure::DtdUnsupported { .. },
                ..
            })
        ));
    }

    #[test]
    fn bounded_internal_subset_reports_expansion_limits_at_original_reference() {
        let xml = b"<!DOCTYPE root [<!ENTITY value '0123456789'>]><root>&value;</root>";
        let failure = parse_document_with_internal_subset(
            "memory:bounded.xml",
            xml,
            LIMITS,
            InternalSubsetLimits {
                replacement_bytes: 4,
                ..DTD_LIMITS
            },
        )
        .expect_err("replacement must be rejected before XDM construction");

        assert!(matches!(
            &failure,
            LocatedFailure {
                failure: ParseFailure::DtdLimit { .. },
                ..
            }
        ));
        assert!(failure.source_span().is_some());
        assert_eq!(
            failure.structural_limit_detail().as_deref(),
            Some("DTD replacement-byte limit is 4")
        );
    }

    #[test]
    fn bounded_internal_subset_observes_existing_xml_cancellation() {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let mut control = InvocationControl::new(cancellation, WorkLimits::unbounded());

        assert!(matches!(
            parse_document_controlled_with_internal_subset(
                "memory:cancelled.xml",
                b"<!DOCTYPE root [<!ENTITY value 'content'>]><root>&value;</root>",
                LIMITS,
                DTD_LIMITS,
                &mut control,
            ),
            Err(LocatedFailure {
                failure: ParseFailure::Control(ControlFailure::Cancelled {
                    domain: WorkDomain::XmlEvent,
                }),
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
