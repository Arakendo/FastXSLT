//! Explicit effective-owner retention for principal globals and lexical aliases.
//! Legacy local-ID consumers must prove their current effective source first.

use super::{
    Arc, Document, ExecutionFailure, FailureCategory, NodeId, RuntimeGlobals, RuntimeVariables,
    SequenceInputs, failure,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(super) struct RetainedSourceSelection {
    pub(super) owner: Arc<Document>,
    pub(super) nodes: Vec<NodeId>,
}

impl RuntimeVariables {
    pub(in super::super) fn qualify_local_sources(
        &self,
        source: &Document,
        request_id: &str,
        control: &mut crate::execution_control_experiment::InvocationControl,
    ) -> Result<Self, ExecutionFailure> {
        let mut qualified = self.clone();
        if self.source_nodes.is_empty() {
            return Ok(qualified);
        }
        let owner = source
            .retain_effective_owner(control)
            .map_err(|failure| super::super::control_failure(failure, request_id))?;
        for (name, nodes) in self.source_nodes.iter() {
            control
                .charge(
                    crate::execution_control_experiment::WorkDomain::XdmNode,
                    nodes.len(),
                )
                .map_err(|failure| super::super::control_failure(failure, request_id))?;
            qualified.bind_retained_source_nodes(name.clone(), owner.clone(), nodes.clone());
        }
        Ok(qualified)
    }

    pub(in super::super) fn bind_retained_source_nodes(
        &mut self,
        name: String,
        owner: Arc<Document>,
        nodes: Vec<NodeId>,
    ) {
        self.bind_retained_source(name, RetainedSourceSelection { owner, nodes });
    }
    pub(in super::super) fn retained_source<'a>(
        &'a self,
        globals: &'a RuntimeGlobals,
        name: &str,
    ) -> Option<(&'a Arc<Document>, &'a Vec<NodeId>)> {
        if let Some(value) = self
            .retained_sources
            .as_ref()
            .and_then(|values| values.get(name))
        {
            return Some((&value.owner, &value.nodes));
        }
        if !self.allows_global_fallback(name) {
            return None;
        }
        if let Some(value) = globals.retained_sources.get(name) {
            return Some((&value.owner, &value.nodes));
        }
        Some((globals.principal_owner.as_ref()?, globals.nodes.get(name)?))
    }

    pub(super) fn bind_retained_source(&mut self, name: String, value: RetainedSourceSelection) {
        self.clear_value_kinds(&name);
        let values = self
            .retained_sources
            .get_or_insert_with(|| Arc::new(BTreeMap::new()));
        Arc::make_mut(values).insert(name, value);
    }

    pub(in super::super) fn source_nodes_in_context<'a>(
        &'a self,
        inputs: &'a SequenceInputs<'_>,
        name: &str,
    ) -> Result<Option<&'a Vec<NodeId>>, ExecutionFailure> {
        if let Some((owner, nodes)) = self.retained_source(inputs.globals, name) {
            if !nodes.is_empty()
                && inputs
                    .source
                    .is_none_or(|source| !source.same_effective_storage(owner))
            {
                return Err(failure(
                    "FXRT1017",
                    FailureCategory::Unsupported,
                    Some(inputs.request_id),
                    format!("this consumer requires source-qualified navigation for ${name}"),
                ));
            }
            return Ok(Some(nodes));
        }
        // Bare local selections remain eligible only within their lexical
        // effective-source context. External transfer guards are retained.
        Ok(self.source_nodes(inputs.globals, name))
    }
}

#[cfg(test)]
mod tests {
    use super::super::InvocationControl;
    use super::*;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    #[test]
    fn retained_source_frames_preserve_owner_shadowing_and_release() {
        let source = Document::from_parsed(
            parse_document(
                "urn:retained-frame",
                b"<r/>",
                ParseLimits {
                    max_events: 8,
                    max_depth: 4,
                },
            )
            .unwrap(),
        )
        .unwrap();
        let owner = source
            .retain_effective_owner(&mut InvocationControl::unbounded())
            .unwrap();
        let weak = Arc::downgrade(&owner);
        let mut globals = RuntimeGlobals::default();
        globals
            .nodes
            .insert("principal".to_owned(), vec![owner.document_node()]);
        globals.principal_owner = Some(owner.clone());
        let mut parent = RuntimeVariables::default();
        assert!(parent.retained_sources.is_none());
        assert!(parent.bind_alias("alias".to_owned(), "principal", &globals));
        for complete in [false, true] {
            let mut child = if complete {
                parent.clone_complete_for_sequence()
            } else {
                parent.clone()
            };
            assert_eq!(
                Arc::ptr_eq(
                    parent.retained_sources.as_ref().unwrap(),
                    child.retained_sources.as_ref().unwrap()
                ),
                !complete
            );
            assert!(child.bind_alias("kept".to_owned(), "alias", &globals));
            child.bind_source_nodes("alias".to_owned(), Vec::new());
            assert!(child.retained_source(&globals, "alias").is_none());
            assert!(parent.retained_source(&globals, "alias").is_some());
            assert!(Arc::ptr_eq(
                child.retained_source(&globals, "kept").unwrap().0,
                &owner
            ));
            child.bind_source_nodes("principal".to_owned(), Vec::new());
            assert!(child.retained_source(&globals, "principal").is_none());
        }
        let complete = parent.clone_complete_for_sequence();
        drop(owner);
        drop(globals);
        drop(parent);
        assert!(weak.upgrade().is_some());
        drop(complete);
        assert!(weak.upgrade().is_none());
    }
}
