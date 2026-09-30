//! Replaceable `XML` parsing and serialization boundary.

#[cfg(any(test, feature = "workbench"))]
mod input_transcoding;

pub(crate) mod names;

#[cfg(any(test, feature = "workbench"))]
pub(crate) mod quick_xml_experiment;
