//! `FastXSLT` is a Rust-native XSLT engine.
//!
//! This crate is currently a buildable project scaffold. It does not yet expose
//! a transformation API or claim support for an XSLT standards profile.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod compile;
mod diagnostics;
mod runtime;
mod xdm;
mod xml;
mod xpath;
mod xslt;
