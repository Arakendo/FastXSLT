use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xml::quick_xml_experiment::{
    ExpandedName, NamespaceBinding, OwnedXmlEvent, ParsedDocument,
};

#[path = "whitespace_view.rs"]
mod whitespace_view;

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

#[derive(Debug, Clone)]
struct Node {
    kind: NodeKind,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    attributes: Vec<NodeId>,
    namespaces: Vec<NamespaceBinding>,
    name: Option<ExpandedName>,
    prefix: Option<String>,
    value: Option<String>,
    location: SourceLocation,
    document_order: Option<usize>,
}

#[derive(Debug)]
pub(crate) struct Document {
    nodes: Arc<Vec<Node>>,
    root: NodeId,
    child_overrides: Option<HashMap<NodeId, Box<[NodeId]>>>,
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
            nodes: Arc::new(vec![Node {
                kind: NodeKind::Document,
                parent: None,
                children: Vec::new(),
                attributes: Vec::new(),
                namespaces: Vec::new(),
                name: None,
                prefix: None,
                value: None,
                location: SourceLocation {
                    resource: parsed.resource.clone(),
                    span: 0..document_end,
                },
                document_order: None,
            }]),
            root: NodeId(0),
            child_overrides: None,
        };
        let mut ancestors = vec![result.root];

        for event in parsed.events {
            match event {
                OwnedXmlEvent::Start {
                    name,
                    prefix,
                    attributes,
                    namespaces,
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
                    result.nodes_mut()[element.0].prefix = prefix;
                    result.nodes_mut()[element.0].namespaces = namespaces;
                    for attribute in attributes {
                        control
                            .charge(WorkDomain::XdmNode, 1)
                            .map_err(BuildFailure::Control)?;
                        let attribute_id = result.push_node(Node {
                            kind: NodeKind::Attribute,
                            parent: Some(element),
                            children: Vec::new(),
                            attributes: Vec::new(),
                            namespaces: Vec::new(),
                            name: Some(attribute.name),
                            prefix: None,
                            value: Some(attribute.value),
                            location: SourceLocation {
                                resource: parsed.resource.clone(),
                                span: attribute.span,
                            },
                            document_order: None,
                        });
                        result.nodes_mut()[element.0].attributes.push(attribute_id);
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
                        result.nodes_mut()[last.0]
                            .value
                            .as_mut()
                            .expect("text nodes carry values")
                            .push_str(&value);
                        result.nodes_mut()[last.0].location.span.end = span.end;
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
            namespaces: Vec::new(),
            name,
            prefix: None,
            value,
            location: SourceLocation {
                resource: resource.to_owned(),
                span,
            },
            document_order: None,
        });
        self.nodes_mut()[parent.0].children.push(id);
        id
    }

    fn push_node(&mut self, node: Node) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes_mut().push(node);
        id
    }

    fn nodes_mut(&mut self) -> &mut Vec<Node> {
        Arc::get_mut(&mut self.nodes).expect("XDM construction retains the only node-storage owner")
    }

    fn assign_document_order(&mut self) {
        let mut ordered = Vec::with_capacity(self.nodes.len());
        self.collect_document_order(self.root, &mut ordered);
        for (rank, id) in ordered.into_iter().enumerate() {
            self.nodes_mut()[id.0].document_order = Some(rank);
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
        self.root
    }

    #[cfg(test)]
    pub(crate) fn derive_stripping_all_element_whitespace(
        &self,
        control: &mut InvocationControl,
    ) -> Result<Self, ControlFailure> {
        let mut nodes = Vec::new();
        for node in self.nodes.iter() {
            control.charge(WorkDomain::XdmNode, 1)?;
            nodes.push(node.clone());
        }
        let mut derived = Self {
            nodes: Arc::new(nodes),
            root: self.root,
            child_overrides: None,
        };
        for index in 0..self.nodes.len() {
            if self.nodes[index].kind != NodeKind::Element {
                continue;
            }
            derived.nodes_mut()[index].children.retain(|child| {
                let child = &self.nodes[child.0];
                child.kind != NodeKind::Text
                    || !child.value.as_deref().is_some_and(is_xml_whitespace_only)
            });
        }
        Ok(derived)
    }

    #[cfg(test)]
    pub(crate) fn node_count(&self) -> usize {
        self.nodes.len()
    }

    #[cfg(test)]
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
                let prefix_bytes = node.prefix.as_ref().map_or(0, String::capacity);
                let namespace_bytes = node.namespaces.capacity()
                    * std::mem::size_of::<NamespaceBinding>()
                    + node
                        .namespaces
                        .iter()
                        .map(|binding| {
                            binding.prefix.as_ref().map_or(0, String::capacity)
                                + binding.namespace.capacity()
                        })
                        .sum::<usize>();
                relationships
                    + name_bytes
                    + prefix_bytes
                    + value_bytes
                    + namespace_bytes
                    + node.location.resource.capacity()
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

    pub(crate) fn prefix(&self, id: NodeId) -> Option<&str> {
        self.nodes[id.0].prefix.as_deref()
    }

    pub(crate) fn children(&self, id: NodeId) -> &[NodeId] {
        if let Some(children) = self
            .child_overrides
            .as_ref()
            .and_then(|overrides| overrides.get(&id))
        {
            return children;
        }
        &self.nodes[id.0].children
    }

    pub(crate) fn attributes(&self, id: NodeId) -> &[NodeId] {
        &self.nodes[id.0].attributes
    }

    pub(crate) fn namespace_declarations(&self, id: NodeId) -> &[NamespaceBinding] {
        &self.nodes[id.0].namespaces
    }

    pub(crate) fn in_scope_namespaces(&self, id: NodeId) -> Vec<NamespaceBinding> {
        let mut lineage = Vec::new();
        let mut current = Some(id);
        while let Some(node) = current {
            lineage.push(node);
            current = self.parent(node);
        }

        let mut in_scope = Vec::new();
        for node in lineage.into_iter().rev() {
            for binding in self.namespace_declarations(node) {
                in_scope.retain(|candidate: &NamespaceBinding| candidate.prefix != binding.prefix);
                in_scope.push(binding.clone());
            }
        }
        in_scope
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

    pub(crate) fn document_order(&self, id: NodeId) -> usize {
        self.nodes[id.0]
            .document_order
            .expect("owned XDM nodes have assigned document order")
    }

    pub(crate) fn has_xml_space_declaration(
        &self,
        control: &mut InvocationControl,
    ) -> Result<bool, ControlFailure> {
        const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
        for node in self.nodes.iter() {
            control.charge(WorkDomain::XdmNode, 1)?;
            if node.kind != NodeKind::Element {
                continue;
            }
            if node.attributes.iter().any(|attribute| {
                self.name(*attribute).is_some_and(|name| {
                    name.namespace.as_deref() == Some(XML_NAMESPACE) && name.local == "space"
                })
            }) {
                return Ok(true);
            }
        }
        Ok(false)
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
                for child in self.children(id) {
                    self.visit_string_value_controlled(*child, control, sink)?;
                }
            }
        }
        Ok(())
    }
}

fn is_xml_whitespace_only(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
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
        let greeting = document.nodes[document.root.0].children[0];
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
        let root = document.nodes[document.root.0].children[0];
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
        let root = document.nodes[document.root.0].children[0];

        assert_eq!(document.nodes[root.0].children.len(), 1);
        assert_eq!(document.string_value(root), "one&twothree");
    }

    #[test]
    fn namespace_declarations_remain_owned_separately_from_attributes() {
        let parsed = parse_document(
            "memory:namespaces.xml",
            b"<root xmlns:p='urn:p' value='x'><child xmlns:q='urn:q'/></root>",
            LIMITS,
        )
        .expect("namespace fixture should parse");
        let document = Document::from_parsed(parsed).expect("owned XDM should build");
        let root = document.children(document.document_node())[0];
        let child = document.children(root)[0];

        assert_eq!(document.attributes(root).len(), 1);
        assert_eq!(document.namespace_declarations(root).len(), 1);
        assert_eq!(
            document.namespace_declarations(root)[0].prefix.as_deref(),
            Some("p")
        );
        assert_eq!(document.namespace_declarations(root)[0].namespace, "urn:p");
        assert_eq!(
            document.namespace_declarations(child)[0].prefix.as_deref(),
            Some("q")
        );
    }

    #[test]
    fn element_prefixes_remain_distinct_from_expanded_name_identity() {
        let parsed = parse_document(
            "memory:prefixes.xml",
            b"<root xmlns:one='urn:same' xmlns:two='urn:same'><one:item/><two:item/></root>",
            LIMITS,
        )
        .expect("prefix fixture should parse");
        let document = Document::from_parsed(parsed).expect("owned XDM should build");
        let root = document.children(document.document_node())[0];
        let children = document.children(root);

        assert_eq!(document.name(children[0]), document.name(children[1]));
        assert_eq!(document.prefix(children[0]), Some("one"));
        assert_eq!(document.prefix(children[1]), Some("two"));
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
        let root = document.nodes[document.root.0].children[0];
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

    #[test]
    fn derived_strip_all_reference_preserves_visible_identity_and_filters_relationships() {
        let parsed = parse_document(
            "memory:strip-reference.xml",
            b"<root>  <kept> value </kept>\n<empty/>tail</root>",
            LIMITS,
        )
        .expect("strip reference fixture should parse");
        let document = Document::from_parsed(parsed).expect("owned XDM should build");
        let root = document.children(document.document_node())[0];
        let original_children = document.children(root).to_vec();
        let kept = original_children
            .iter()
            .copied()
            .find(|node| {
                document
                    .name(*node)
                    .is_some_and(|name| name.local == "kept")
            })
            .expect("visible kept element");
        let mut control = crate::execution_control_experiment::InvocationControl::unbounded();

        let derived = document
            .derive_stripping_all_element_whitespace(&mut control)
            .expect("safe reference derivation should succeed");

        assert_eq!(derived.children(derived.document_node())[0], root);
        assert!(derived.children(root).contains(&kept));
        assert_eq!(derived.location(kept), document.location(kept));
        assert_eq!(document.children(root).len(), 5);
        assert_eq!(derived.children(root).len(), 3);
        assert_eq!(document.string_value(root), "   value \ntail");
        assert_eq!(derived.string_value(root), " value tail");
    }

    #[test]
    fn derived_strip_all_reference_is_bounded_and_cancellable() {
        let parsed = parse_document(
            "memory:strip-control.xml",
            b"<root>  <child/>  </root>",
            LIMITS,
        )
        .expect("strip control fixture should parse");
        let document = Document::from_parsed(parsed).expect("owned XDM should build");

        let mut limits = crate::execution_control_experiment::WorkLimits::unbounded();
        limits.xdm_nodes = 0;
        let mut bounded = crate::execution_control_experiment::InvocationControl::new(
            crate::execution_control_experiment::CancellationToken::new(),
            limits,
        );
        let failure = document
            .derive_stripping_all_element_whitespace(&mut bounded)
            .expect_err("zero XDM-node budget should stop reference construction");
        assert!(matches!(
            failure,
            crate::execution_control_experiment::ControlFailure::BudgetExhausted {
                domain: crate::execution_control_experiment::WorkDomain::XdmNode,
                ..
            }
        ));

        let mut cancelling = crate::execution_control_experiment::InvocationControl::unbounded()
            .cancelling_on_charge(crate::execution_control_experiment::WorkDomain::XdmNode, 0);
        let failure = document
            .derive_stripping_all_element_whitespace(&mut cancelling)
            .expect_err("cancellation should stop reference construction");
        assert_eq!(
            failure,
            crate::execution_control_experiment::ControlFailure::Cancelled {
                domain: crate::execution_control_experiment::WorkDomain::XdmNode,
            }
        );
    }
}
