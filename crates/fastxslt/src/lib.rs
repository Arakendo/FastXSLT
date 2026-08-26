//! `FastXSLT` is a Rust-native XSLT engine.
//!
//! `FastXSLT` is a pre-stability Rust-native XSLT engine prototype with a private
//! executable implementation. It does not yet expose a supported public API or
//! claim broad standards conformance.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod compile;
mod diagnostics;
#[cfg(any(test, feature = "workbench"))]
mod execution_control_experiment;
#[cfg(any(test, feature = "workbench"))]
mod resources;
mod runtime;
#[cfg(test)]
mod verification_ledger_conservation_experiment;
#[cfg(test)]
mod verification_ledger_experiment;
mod xdm;
mod xml;
mod xpath;
mod xslt;

/// Explicitly unstable facade used only by the ASP.NET boundary workbench.
#[cfg(feature = "workbench")]
#[doc(hidden)]
pub mod workbench {
    pub use crate::runtime::workbench_experiment::{
        ExperimentalEngine, WorkbenchCancellation, WorkbenchFailure, WorkbenchLimits,
    };
}
