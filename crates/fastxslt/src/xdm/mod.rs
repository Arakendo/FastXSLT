//! Engine-owned `XDM` semantics.

#[cfg(any(test, feature = "workbench"))]
pub(crate) mod atomic_value_experiment;
#[cfg(any(test, feature = "workbench"))]
pub(crate) mod owned_tree_experiment;

#[cfg(any(test, feature = "workbench"))]
pub(crate) mod qualified_nodes;

#[cfg(test)]
pub(crate) mod namespace_binding_reference;
#[cfg(test)]
mod namespace_nodes_reference;
