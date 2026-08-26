//! `XSLT` stylesheet and instruction semantics.

#[cfg(any(test, feature = "workbench"))]
pub(crate) mod golden_semantics_experiment;
#[cfg(test)]
mod semantic_inspection_experiment;
