use std::collections::HashSet;
use std::ops::Range;

use quick_xml::XmlVersion;
use quick_xml::events::{BytesStart, Event};
use quick_xml::name::ResolveResult;
use quick_xml::reader::NsReader;

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
    pub(crate) value: String,
    pub(crate) span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OwnedXmlEvent {
    Start {
        name: ExpandedName,
        attributes: Vec<XmlAttribute>,
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
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct LocatedFailure {
    resource: String,
    failure: ParseFailure,
}

pub(crate) fn parse_document(
    resource: &str,
    input: &[u8],
    limits: ParseLimits,
) -> Result<ParsedDocument, LocatedFailure> {
    parse_bytes(input, limits)
        .map(|mut document| {
            document.resource = resource.to_owned();
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
fn parse_bytes(input: &[u8], limits: ParseLimits) -> Result<ParsedDocument, ParseFailure> {
    let mut reader = NsReader::from_reader(input);
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
        let start = usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX);
        let event = reader
            .read_event()
            .map_err(|error| ParseFailure::Malformed {
                offset: usize::try_from(reader.error_position()).unwrap_or(usize::MAX),
                detail: error.to_string(),
            })?;
        let end = usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX);
        let span = start..end;

        if !matches!(event, Event::Eof) {
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
                let attributes = resolve_attributes(&reader, &element, start)?;
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
                    attributes,
                    span,
                });
            }
            Event::Empty(element) => {
                let name = resolve_element_name(&reader, &element, start)?;
                let attributes = resolve_attributes(&reader, &element, start)?;
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
                    attributes,
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
                if depth == 0 && !text.as_ref().iter().all(u8::is_ascii_whitespace) {
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
                if reference.resolve_char_ref().ok().flatten().is_none()
                    && !matches!(
                        reference.as_ref(),
                        b"lt" | b"gt" | b"amp" | b"apos" | b"quot"
                    )
                {
                    return Err(ParseFailure::UnknownEntity {
                        offset: start,
                        name: reference.as_ref().to_vec(),
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
                let target = decode_name(instruction.target(), start)?;
                let value = std::str::from_utf8(instruction.content())
                    .map_err(|error| malformed(start, error))?
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
    expanded_name(namespace, local.as_ref(), offset)
}

fn resolve_end_name(
    reader: &NsReader<&[u8]>,
    name: &[u8],
    offset: usize,
) -> Result<ExpandedName, ParseFailure> {
    let qualified = quick_xml::name::QName(name);
    let (namespace, local) = reader.resolver().resolve_element(qualified);
    expanded_name(namespace, local.as_ref(), offset)
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
        let name = expanded_name(namespace, local.as_ref(), offset)?;
        if !expanded_names.insert(name.clone()) {
            return Err(ParseFailure::Malformed {
                offset,
                detail: format!("duplicate expanded attribute name: {name:?}"),
            });
        }
        names.push(XmlAttribute {
            name,
            value,
            span: offset..usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX),
        });
    }
    Ok(names)
}

fn expanded_name(
    namespace: ResolveResult<'_>,
    local: &[u8],
    offset: usize,
) -> Result<ExpandedName, ParseFailure> {
    let namespace = match namespace {
        ResolveResult::Unbound => None,
        ResolveResult::Bound(namespace) => Some(decode_name(namespace.as_ref(), offset)?),
        ResolveResult::Unknown(prefix) => {
            return Err(ParseFailure::UnknownNamespacePrefix { offset, prefix });
        }
    };
    Ok(ExpandedName {
        namespace,
        local: decode_name(local, offset)?,
    })
}

fn decode_name(bytes: &[u8], offset: usize) -> Result<String, ParseFailure> {
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|error| ParseFailure::Malformed {
            offset,
            detail: format!("the private XML experiment accepts UTF-8 names only: {error}"),
        })
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
    let character = match reference.as_ref() {
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
    use super::{ExpandedName, LocatedFailure, ParseFailure, ParseLimits, parse_document};

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
