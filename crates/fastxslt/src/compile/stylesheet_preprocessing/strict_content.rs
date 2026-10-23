//! Private strict preprocessing component for R0 non-test migration.
//! The full profile remains unpromoted; the existing visibility reference is
//! retained under tests for differential semantics and control accounting.

use super::{
    Document, NodeId, NodeKind, StrictPreprocessingFailure, StylesheetPreprocessingProfile,
    effective_xml_space_preserved, preserves_literal_text,
};
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use std::collections::HashSet;

pub(crate) fn prepare(
    document: &Document,
    root: NodeId,
    control: &mut InvocationControl,
) -> Result<Document, StrictPreprocessingFailure> {
    control.charge(WorkDomain::XdmNode, 1)?;
    let mut pending = vec![root];
    let mut parents = HashSet::new();
    let mut retained = HashSet::new();
    while let Some(parent) = pending.pop() {
        control.charge(WorkDomain::XdmNode, 1)?;
        parents.insert(parent);
        let explicit = super::super::is_xslt_element(document, parent, "text")
            || effective_xml_space_preserved(document, parent)?;
        let children = document.children(parent);
        for (index, child) in children.iter().copied().enumerate() {
            control.charge(WorkDomain::XdmNode, 1)?;
            let keep = match document.kind(child) {
                NodeKind::Comment | NodeKind::ProcessingInstruction => false,
                NodeKind::Text => preserves_literal_text(
                    document,
                    children,
                    index,
                    explicit,
                    StylesheetPreprocessingProfile::StrictXslt10,
                ),
                _ => true,
            };
            if keep {
                retained.insert(child);
            }
            if document.kind(child) == NodeKind::Element {
                pending.push(child);
            }
        }
    }
    document
        .view_filtering_children(
            |parent, child| !parents.contains(&parent) || retained.contains(&child),
            control,
        )
        .map_err(StrictPreprocessingFailure::Control)
}
