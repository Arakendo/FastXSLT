//! Test-only source-value handoff. Comparison semantics remain in the shared evaluator.

use super::super::{ExecutionFailure, InvocationControl};
use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use std::{rc::Rc, sync::Arc};

type Selection = (Arc<Document>, Vec<NodeId>);
type Reader = dyn Fn(
    &str,
    Option<&SourceLocation>,
    &mut InvocationControl,
) -> Result<Option<Selection>, ExecutionFailure>;

pub(in crate::runtime::golden_runtime_experiment) struct SourceReadHook(Rc<Reader>);

impl SourceReadHook {
    pub(super) fn new(
        reader: impl Fn(
            &str,
            Option<&SourceLocation>,
            &mut InvocationControl,
        ) -> Result<Option<Selection>, ExecutionFailure>
        + 'static,
    ) -> Self {
        Self(Rc::new(reader))
    }

    pub(in crate::runtime) fn read(
        &self,
        name: &str,
        control: &mut InvocationControl,
    ) -> Result<Option<Selection>, ExecutionFailure> {
        self.read_at(name, None, control)
    }

    pub(in crate::runtime) fn read_at(
        &self,
        name: &str,
        location: Option<&SourceLocation>,
        control: &mut InvocationControl,
    ) -> Result<Option<Selection>, ExecutionFailure> {
        (self.0)(name, location, control)
    }
}

impl std::fmt::Debug for SourceReadHook {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SourceReadHook(test-only)")
    }
}
