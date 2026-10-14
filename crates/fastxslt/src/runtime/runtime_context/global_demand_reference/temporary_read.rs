//! Test-only owned temporary-value read bridge; no evaluation or copy semantics.

use super::super::{ExecutionFailure, InvocationControl, TemporaryTree};
use crate::xdm::owned_tree_experiment::SourceLocation;
use std::{rc::Rc, sync::Arc};

type Reader = dyn Fn(
    &str,
    &SourceLocation,
    &mut InvocationControl,
) -> Result<Option<Arc<TemporaryTree>>, ExecutionFailure>;

pub(in crate::runtime::golden_runtime_experiment) struct TemporaryReadHook(Rc<Reader>);

impl TemporaryReadHook {
    pub(super) fn new(
        reader: impl Fn(
            &str,
            &SourceLocation,
            &mut InvocationControl,
        ) -> Result<Option<Arc<TemporaryTree>>, ExecutionFailure>
        + 'static,
    ) -> Self {
        Self(Rc::new(reader))
    }

    pub(in crate::runtime::golden_runtime_experiment) fn read(
        &self,
        name: &str,
        location: &SourceLocation,
        control: &mut InvocationControl,
    ) -> Result<Option<Arc<TemporaryTree>>, ExecutionFailure> {
        (self.0)(name, location, control)
    }
}

impl std::fmt::Debug for TemporaryReadHook {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("TemporaryReadHook(test-only)")
    }
}
