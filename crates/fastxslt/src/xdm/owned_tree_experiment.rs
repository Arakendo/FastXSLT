use std::ops::Range;

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xml::quick_xml_experiment::{ExpandedName, OwnedXmlEvent, ParsedDocument};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct NodeId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeKind {
    Document,
    Element,
    Attribute,
    Text,
    Comment,
    ProcessingInstruction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceLocation {
    pub(crate) resource: String,
    pub(crate) span: Range<usize>,
}

#[derive(Debug)]
struct Node {
    kind: NodeKind,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    attributes: Vec<NodeId>,
    name: Option<ExpandedName>,
    value: Option<String>,
    location: SourceLocation,
    document_order: Option<usize>,
}

#[derive(Debug)]
pub(crate) struct Document {
    nodes: Vec<Node>,
    document: NodeId,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BuildFailure {
    UnexpectedEnd,
    UnclosedElement,
    Control(ControlFailure),
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StringValueVisitFailure<SinkFailure> {
    Control(ControlFailure),
    Sink(SinkFailure),
}

impl Document {
    #[allow(
        clippy::too_many_lines,
        reason = "keeping the private event-to-tree state machine together makes ownership auditable"
    )]
    pub(crate) fn from_parsed(parsed: ParsedDocument) -> Result<Self, BuildFailure> {
        let mut control = InvocationControl::unbounded();
        Self::from_parsed_controlled(parsed, &mut control)
    }

    #[allow(
        clippy::too_many_lines,
        reason = "keeping the private event-to-tree state machine together makes ownership auditable"
    )]
    pub(crate) fn from_parsed_controlled(
        parsed: ParsedDocument,
        control: &mut InvocationControl,
    ) -> Result<Self, BuildFailure> {
        let document_end = parsed
            .events
            .iter()
            .map(event_span)
            .map(|span| span.end)
            .max()
            .unwrap_or(0);
        control
            .charge(WorkDomain::XdmNode, 1)
            .map_err(BuildFailure::Control)?;
        let mut result = Self {
            nodes: vec![Node {
                kind: NodeKind::Document,
                parent: None,
                children: Vec::new(),
                attributes: Vec::new(),
                name: None,
                value: None,
                location: SourceLocation {
                    resource: parsed.resource.clone(),
                    span: 0..document_end,
                },
                document_order: None,
            }],
            document: NodeId(0),
        };
        let mut ancestors = vec![result.document];

        for event in parsed.events {
            match event {
                OwnedXmlEvent::Start {
                    name,
                    attributes,
                    span,
                } => {
                    control
                        .charge(WorkDomain::XdmNode, 1)
                        .map_err(BuildFailure::Control)?;
                    let parent = *ancestors.last().expect("document ancestor is retained");
                    let element = result.push_child(
                        parent,
                        NodeKind::Element,
                        Some(name),
                        None,
                        &parsed.resource,
                        span.clone(),
                    );
                    for attribute in attributes {
                        control
                            .charge(WorkDomain::XdmNode, 1)
                            .map_err(BuildFailure::Control)?;
                        let attribute_id = result.push_node(Node {
                            kind: NodeKind::Attribute,
                            parent: Some(element),
                            children: Vec::new(),
                            attributes: Vec::new(),
                            name: Some(attribute.name),
                            value: Some(attribute.value),
                            location: SourceLocation {
                                resource: parsed.resource.clone(),
                                span: attribute.span,
                            },
                            document_order: None,
                        });
                        result.nodes[element.0].attributes.push(attribute_id);
                    }
                    ancestors.push(element);
                }
                OwnedXmlEvent::End { .. } => {
                    if ancestors.len() == 1 {
                        return Err(BuildFailure::UnexpectedEnd);
                    }
                    ancestors.pop();
                }
                OwnedXmlEvent::Text { value, span } => {
                    let parent = *ancestors.last().expect("document ancestor is retained");
                    if let Some(last) = result.nodes[parent.0].children.last().copied()
                        && result.nodes[last.0].kind == NodeKind::Text
                    {
                        result.nodes[last.0]
                            .value
                            .as_mut()
                            .expect("text nodes carry values")
                            .push_str(&value);
                        result.nodes[last.0].location.span.end = span.end;
                    } else {
                        control
                            .charge(WorkDomain::XdmNode, 1)
                            .map_err(BuildFailure::Control)?;
                        result.push_child(
                            parent,
                            NodeKind::Text,
                            None,
                            Some(value),
                            &parsed.resource,
                            span,
                        );
                    }
                }
                OwnedXmlEvent::Comment { value, span } => {
                    control
                        .charge(WorkDomain::XdmNode, 1)
                        .map_err(BuildFailure::Control)?;
                    let parent = *ancestors.last().expect("document ancestor is retained");
                    result.push_child(
                        parent,
                        NodeKind::Comment,
                        None,
                        Some(value),
                        &parsed.resource,
                        span,
                    );
                }
                OwnedXmlEvent::ProcessingInstruction {
                    target,
                    value,
                    span,
                } => {
                    control
                        .charge(WorkDomain::XdmNode, 1)
                        .map_err(BuildFailure::Control)?;
                    let parent = *ancestors.last().expect("document ancestor is retained");
                    result.push_child(
                        parent,
                        NodeKind::ProcessingInstruction,
                        Some(ExpandedName {
                            namespace: None,
                            local: target,
                        }),
                        Some(value),
                        &parsed.resource,
                        span,
                    );
                }
            }
        }

        if ancestors.len() != 1 {
            return Err(BuildFailure::UnclosedElement);
        }
        result.assign_document_order();
        Ok(result)
    }

    fn push_child(
        &mut self,
        parent: NodeId,
        kind: NodeKind,
        name: Option<ExpandedName>,
        value: Option<String>,
        resource: &str,
        span: Range<usize>,
    ) -> NodeId {
        let id = self.push_node(Node {
            kind,
            parent: Some(parent),
            children: Vec::new(),
            attributes: Vec::new(),
            name,
            value,
            location: SourceLocation {
                resource: resource.to_owned(),
                span,
            },
            document_order: None,
        });
        self.nodes[parent.0].children.push(id);
        id
    }

    fn push_node(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        id
    }

    fn assign_document_order(&mut self) {
        let mut ordered = Vec::with_capacity(self.nodes.len());
        self.collect_document_order(self.document, &mut ordered);
        for (rank, id) in ordered.into_iter().enumerate() {
            self.nodes[id.0].document_order = Some(rank);
        }
    }

    fn collect_document_order(&self, id: NodeId, ordered: &mut Vec<NodeId>) {
        ordered.push(id);
        for attribute in &self.nodes[id.0].attributes {
            ordered.push(*attribute);
        }
        for child in &self.nodes[id.0].children {
            self.collect_document_order(*child, ordered);
        }
    }

    pub(crate) fn document_node(&self) -> NodeId {
        self.document
    }

    pub(crate) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn owned_capacity_bytes(&self) -> usize {
        let node_storage = self.nodes.capacity() * std::mem::size_of::<Node>();
        let nested_storage: usize = self
            .nodes
            .iter()
            .map(|node| {
                let relationships = (node.children.capacity() + node.attributes.capacity())
                    * std::mem::size_of::<NodeId>();
                let name_bytes = node.name.as_ref().map_or(0, |name| {
                    name.local.capacity()
                        + name
                            .namespace
                            .as_ref()
                            .map_or(0, std::string::String::capacity)
                });
                let value_bytes = node.value.as_ref().map_or(0, std::string::String::capacity);
                relationships + name_bytes + value_bytes + node.location.resource.capacity()
            })
            .sum();
        std::mem::size_of::<Self>() + node_storage + nested_storage
    }

    pub(crate) fn kind(&self, id: NodeId) -> NodeKind {
        self.nodes[id.0].kind
    }

    pub(crate) fn name(&self, id: NodeId) -> Option<&ExpandedName> {
        self.nodes[id.0].name.as_ref()
    }

    pub(crate) fn children(&self, id: NodeId) -> &[NodeId] {
        &self.nodes[id.0].children
    }

    pub(crate) fn attributes(&self, id: NodeId) -> &[NodeId] {
        &self.nodes[id.0].attributes
    }

    pub(crate) fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.nodes[id.0].parent
    }

    pub(crate) fn value(&self, id: NodeId) -> Option<&str> {
        self.nodes[id.0].value.as_deref()
    }

    pub(crate) fn location(&self, id: NodeId) -> &SourceLocation {
        &self.nodes[id.0].location
    }

    pub(crate) fn string_value(&self, id: NodeId) -> String {
        let mut control = InvocationControl::unbounded();
        self.string_value_controlled(id, &mut control)
            .expect("unbounded private control cannot reject string-value work")
    }

    pub(crate) fn string_value_controlled(
        &self,
        id: NodeId,
        control: &mut InvocationControl,
    ) -> Result<String, ControlFailure> {
        let mut value = String::new();
        self.visit_string_value_controlled(id, control, &mut |part, _| {
            value.push_str(part);
            Ok::<_, std::convert::Infallible>(())
        })
        .map_err(|failure| match failure {
            StringValueVisitFailure::Control(failure) => failure,
            StringValueVisitFailure::Sink(never) => match never {},
        })?;
        Ok(value)
    }

    pub(crate) fn visit_string_value_controlled<SinkFailure>(
        &self,
        id: NodeId,
        control: &mut InvocationControl,
        sink: &mut impl FnMut(&str, &mut InvocationControl) -> Result<(), SinkFailure>,
    ) -> Result<(), StringValueVisitFailure<SinkFailure>> {
        control
            .charge(WorkDomain::XdmStringValueNode, 1)
            .map_err(StringValueVisitFailure::Control)?;
        let node = &self.nodes[id.0];
        match node.kind {
            NodeKind::Text
            | NodeKind::Attribute
            | NodeKind::Comment
            | NodeKind::ProcessingInstruction => {
                if let Some(value) = node.value.as_deref() {
                    sink(value, control).map_err(StringValueVisitFailure::Sink)?;
                }
            }
            NodeKind::Document | NodeKind::Element => {
                for child in &node.children {
                    self.visit_string_value_controlled(*child, control, sink)?;
                }
            }
        }
        Ok(())
    }
}

fn event_span(event: &OwnedXmlEvent) -> Range<usize> {
    match event {
        OwnedXmlEvent::Start { span, .. }
        | OwnedXmlEvent::End { span, .. }
        | OwnedXmlEvent::Text { span, .. }
        | OwnedXmlEvent::Comment { span, .. }
        | OwnedXmlEvent::ProcessingInstruction { span, .. } => span.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    use super::{Document, NodeKind};

    const LIMITS: ParseLimits = ParseLimits {
        max_events: 256,
        max_depth: 32,
    };

    #[test]
    fn golden_source_becomes_an_owned_document_with_identity_order_and_provenance() {
        let input = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/golden/hello/input.xml"
        ))
        .to_vec();
        let parsed = parse_document("golden:hello/input.xml", &input, LIMITS)
            .expect("golden source should parse");
        drop(input);

        let document = Document::from_parsed(parsed).expect("owned XDM should build");
        let greeting = document.nodes[document.document.0].children[0];
        let name = document.nodes[greeting.0]
            .children
            .iter()
            .copied()
            .find(|id| {
                document.nodes[id.0].kind == NodeKind::Element
                    && document.nodes[id.0]
                        .name
                        .as_ref()
                        .map(|name| name.local.as_str())
                        == Some("name")
            })
            .expect("name element should exist");

        assert_ne!(greeting, name);
        assert!(document.nodes[greeting.0].document_order < document.nodes[name.0].document_order);
        assert_eq!(document.parent(name), Some(greeting));
        assert_eq!(document.string_value(name), "FastXSLT");
        assert_eq!(
            document.nodes[name.0].location.resource,
            "golden:hello/input.xml"
        );
        assert!(!document.nodes[name.0].location.span.is_empty());
    }

    #[test]
    fn equal_nodes_keep_distinct_identity_and_attributes_are_not_children() {
        let parsed = parse_document(
            "memory:identity.xml",
            b"<root><same value='x'/><same value='x'/></root>",
            LIMITS,
        )
        .expect("identity fixture should parse");
        let document = Document::from_parsed(parsed).expect("owned XDM should build");
        let root = document.nodes[document.document.0].children[0];
        let elements: Vec<_> = document.nodes[root.0]
            .children
            .iter()
            .copied()
            .filter(|id| document.nodes[id.0].kind == NodeKind::Element)
            .collect();

        assert_ne!(elements[0], elements[1]);
        assert_eq!(document.nodes[elements[0].0].attributes.len(), 1);
        assert!(document.nodes[elements[0].0].children.is_empty());
        assert_eq!(
            document.string_value(document.nodes[elements[0].0].attributes[0]),
            "x"
        );
    }

    #[test]
    fn adjacent_text_and_references_coalesce_without_losing_semantics() {
        let parsed = parse_document(
            "memory:text.xml",
            b"<root>one&amp;<![CDATA[two]]>three</root>",
            LIMITS,
        )
        .expect("text fixture should parse");
        let document = Document::from_parsed(parsed).expect("owned XDM should build");
        let root = document.nodes[document.document.0].children[0];

        assert_eq!(document.nodes[root.0].children.len(), 1);
        assert_eq!(document.string_value(root), "one&twothree");
    }

    #[test]
    fn controlled_string_value_sink_preserves_fragment_order_without_an_intermediate_value() {
        let parsed = parse_document(
            "memory:string-value-parts.xml",
            b"<root>one<part>two</part>three</root>",
            LIMITS,
        )
        .expect("fragment fixture should parse");
        let document = Document::from_parsed(parsed).expect("owned XDM should build");
        let root = document.nodes[document.document.0].children[0];
        let mut control = crate::execution_control_experiment::InvocationControl::unbounded();
        let mut fragments = Vec::new();

        document
            .visit_string_value_controlled(root, &mut control, &mut |part, _| {
                fragments.push(part.to_owned());
                Ok::<_, std::convert::Infallible>(())
            })
            .expect("unbounded fragment sink should succeed");

        assert_eq!(fragments, ["one", "two", "three"]);
        assert_eq!(fragments.concat(), document.string_value(root));
    }
}
