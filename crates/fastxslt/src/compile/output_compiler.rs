//! Private compilation of `xsl:output` declarations.

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xslt::golden_semantics_experiment::OutputSettings;

use super::{
    CompileFailure, ensure_no_meaningful_children, ensure_only_attributes, invalid,
    optional_attribute, unsupported,
};

pub(in crate::compile) fn default_output_settings() -> OutputSettings {
    OutputSettings {
        method: None,
        encoding: None,
        media_type: None,
        include_content_type: None,
        byte_order_mark: None,
        omit_xml_declaration: false,
        indent: None,
    }
}

pub(super) fn compile_output(
    document: &Document,
    element: NodeId,
    declared_version: &str,
) -> Result<OutputSettings, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &[
            "method",
            "encoding",
            "media-type",
            "include-content-type",
            "byte-order-mark",
            "omit-xml-declaration",
            "indent",
        ],
        "xsl:output",
    )?;
    ensure_no_meaningful_children(document, element, "xsl:output")?;
    let method = optional_attribute(document, element, None, "method");
    if method.is_some_and(|method| !matches!(method, "xml" | "text" | "xhtml")) {
        return Err(unsupported(
            "FXST1004",
            format!("unsupported output method: {}", method.unwrap_or_default()),
            document.location(element),
        ));
    }
    let encoding = optional_attribute(document, element, None, "encoding");
    if encoding.is_some_and(|value| {
        !value.eq_ignore_ascii_case("UTF-8") && !value.eq_ignore_ascii_case("ISO-8859-1")
    }) {
        return Err(unsupported(
            "FXST1016",
            format!(
                "unsupported output encoding: {}",
                encoding.unwrap_or_default()
            ),
            document.location(element),
        ));
    }
    let omit_xml_declaration = optional_attribute(document, element, None, "omit-xml-declaration")
        .map(|value| {
            parse_output_boolean(
                value,
                "omit-xml-declaration",
                declared_version,
                document.location(element),
            )
        })
        .transpose()?
        .unwrap_or(false);
    let indent = optional_attribute(document, element, None, "indent")
        .map(|value| {
            parse_output_boolean(
                value,
                "indent",
                declared_version,
                document.location(element),
            )
        })
        .transpose()?;
    let include_content_type = optional_attribute(document, element, None, "include-content-type")
        .map(|value| {
            parse_output_boolean(
                value,
                "include-content-type",
                declared_version,
                document.location(element),
            )
        })
        .transpose()?;
    let byte_order_mark = optional_attribute(document, element, None, "byte-order-mark")
        .map(|value| {
            parse_output_boolean(
                value,
                "byte-order-mark",
                declared_version,
                document.location(element),
            )
        })
        .transpose()?;
    Ok(OutputSettings {
        method: method.map(str::to_owned),
        encoding: encoding.map(str::to_owned),
        media_type: optional_attribute(document, element, None, "media-type").map(str::to_owned),
        include_content_type,
        byte_order_mark,
        omit_xml_declaration,
        indent,
    })
}

fn parse_output_boolean(
    value: &str,
    attribute: &str,
    declared_version: &str,
    location: &SourceLocation,
) -> Result<bool, CompileFailure> {
    match value {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ if declared_version == "3.0" => match value.trim() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(invalid(
                "FXST0005",
                format!("{attribute} has an invalid XSLT 3.0 boolean value"),
                location,
            )),
        },
        _ => Err(invalid(
            "FXST0005",
            format!("{attribute} must be 'yes' or 'no'"),
            location,
        )),
    }
}
