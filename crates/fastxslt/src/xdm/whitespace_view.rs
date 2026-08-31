use std::collections::HashMap;

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

use super::{Document, NodeId, NodeKind, is_xml_whitespace_only};

impl Document {
    pub(crate) fn view_stripping_all_element_whitespace(
        &self,
        control: &mut InvocationControl,
    ) -> Result<Self, ControlFailure> {
        let mut child_overrides = HashMap::new();

        for index in 0..self.nodes.len() {
            control.charge(WorkDomain::XdmNode, 1)?;
            let parent = NodeId(index);
            if self.kind(parent) != NodeKind::Element {
                continue;
            }

            let children = self.children(parent);
            let visible: Vec<_> = children
                .iter()
                .copied()
                .filter(|child| {
                    self.kind(*child) != NodeKind::Text
                        || !self.value(*child).is_some_and(is_xml_whitespace_only)
                })
                .collect();
            if visible.len() != children.len() {
                child_overrides.insert(parent, visible.into_boxed_slice());
            }
        }

        Ok(Self {
            nodes: self.nodes.clone(),
            root: self.root,
            child_overrides: (!child_overrides.is_empty()).then_some(child_overrides),
        })
    }

    #[cfg(test)]
    pub(crate) fn shares_node_storage_with(&self, other: &Self) -> bool {
        std::sync::Arc::ptr_eq(&self.nodes, &other.nodes)
    }

    #[cfg(test)]
    pub(crate) fn child_override_count(&self) -> usize {
        self.child_overrides.as_ref().map_or(0, HashMap::len)
    }

    #[cfg(test)]
    pub(crate) fn exclusive_view_capacity_bytes(&self) -> usize {
        self.child_overrides.as_ref().map_or(0, |overrides| {
            overrides.capacity() * std::mem::size_of::<(NodeId, Box<[NodeId]>)>()
                + overrides
                    .values()
                    .map(|children| std::mem::size_of_val(children.as_ref()))
                    .sum::<usize>()
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::execution_control_experiment::{
        CancellationToken, ControlFailure, InvocationControl, WorkDomain, WorkLimits,
    };
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    use super::Document;

    const LIMITS: ParseLimits = ParseLimits {
        max_events: 256,
        max_depth: 32,
    };

    fn source() -> Document {
        let parsed = parse_document(
            "memory:strip-view.xml",
            b"<root>  <kept> value </kept>\n<empty/>tail</root>",
            LIMITS,
        )
        .expect("strip view fixture should parse");
        Document::from_parsed(parsed).expect("strip view XDM should build")
    }

    #[test]
    fn view_matches_complete_reference_without_cloning_node_storage() {
        let source = source();
        let mut reference_control = InvocationControl::unbounded();
        let reference = source
            .derive_stripping_all_element_whitespace(&mut reference_control)
            .expect("safe reference should derive");
        let mut view_control = InvocationControl::unbounded();
        let view = source
            .view_stripping_all_element_whitespace(&mut view_control)
            .expect("visibility view should derive");

        assert!(!reference.shares_node_storage_with(&source));
        assert!(view.shares_node_storage_with(&source));
        assert_eq!(view.child_override_count(), 1);
        assert_eq!(view.document_node(), reference.document_node());

        for index in 0..source.node_count() {
            let node = super::NodeId(index);
            assert_eq!(view.kind(node), reference.kind(node));
            assert_eq!(view.name(node), reference.name(node));
            assert_eq!(view.prefix(node), reference.prefix(node));
            assert_eq!(view.children(node), reference.children(node));
            assert_eq!(view.attributes(node), reference.attributes(node));
            assert_eq!(
                view.namespace_declarations(node),
                reference.namespace_declarations(node)
            );
            assert_eq!(view.parent(node), reference.parent(node));
            assert_eq!(view.value(node), reference.value(node));
            assert_eq!(view.location(node), reference.location(node));
            assert_eq!(view.string_value(node), reference.string_value(node));
        }
    }

    #[test]
    fn view_construction_is_bounded_and_cancellable() {
        let source = source();
        let mut limits = WorkLimits::unbounded();
        limits.xdm_nodes = 0;
        let mut bounded = InvocationControl::new(CancellationToken::new(), limits);
        assert!(matches!(
            source.view_stripping_all_element_whitespace(&mut bounded),
            Err(ControlFailure::BudgetExhausted {
                domain: WorkDomain::XdmNode,
                ..
            })
        ));

        let mut cancelling =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 0);
        assert!(matches!(
            source.view_stripping_all_element_whitespace(&mut cancelling),
            Err(ControlFailure::Cancelled {
                domain: WorkDomain::XdmNode,
            })
        ));
    }

    #[test]
    fn view_retains_only_relationship_overrides_beyond_prepared_storage() {
        let source = source();
        let mut reference_control = InvocationControl::unbounded();
        let reference = source
            .derive_stripping_all_element_whitespace(&mut reference_control)
            .expect("safe reference should derive");
        let mut view_control = InvocationControl::unbounded();
        let view = source
            .view_stripping_all_element_whitespace(&mut view_control)
            .expect("visibility view should derive");

        assert!(view.exclusive_view_capacity_bytes() > 0);
        assert!(view.exclusive_view_capacity_bytes() < reference.owned_capacity_bytes());
    }
}
