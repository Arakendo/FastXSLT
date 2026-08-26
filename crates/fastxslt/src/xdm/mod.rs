//! Engine-owned `XDM` semantics.

#[cfg(any(test, feature = "workbench"))]
pub(crate) mod atomic_value_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod owned_tree_experiment;
