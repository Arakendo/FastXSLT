//! Private compilation of `xsl:output` declarations.

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use std::collections::BTreeSet;

use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::OutputSettings;

use super::{
    CompileFailure, ensure_no_meaningful_children, ensure_only_attributes, invalid,
    optional_attribute, unsupported,
};

const OUTPUT_ATTRIBUTES: &[&str] = &[
    "method",
    "version",
    "encoding",
    "media-type",
    "doctype-system",
    "doctype-public",
    "include-content-type",
    "byte-order-mark",
    "normalization-form",
    "standalone",
    "cdata-section-elements",
    "omit-xml-declaration",
    "indent",
];

pub(in crate::compile) fn default_output_settings() -> OutputSettings {
    OutputSettings {
        method: None,
        version: None,
        encoding: None,
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        byte_order_mark: None,
        normalization_form: None,
        standalone: None,
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: false,
        indent: None,
    }
}

pub(super) struct OutputDeclaration {
    pub(super) settings: OutputSettings,
    specified: BTreeSet<String>,
    location: SourceLocation,
}

pub(super) fn compile_output(
    document: &Document,
    element: NodeId,
    declared_version: &str,
) -> Result<OutputDeclaration, CompileFailure> {
    ensure_only_attributes(document, element, OUTPUT_ATTRIBUTES, "xsl:output")?;
    ensure_no_meaningful_children(document, element, "xsl:output")?;
    let method = optional_attribute(document, element, None, "method");
    if method.is_some_and(|method| !matches!(method, "xml" | "text" | "xhtml")) {
        return Err(unsupported(
            "FXST1004",
            format!("unsupported output method: {}", method.unwrap_or_default()),
            document.location(element),
        ));
    }
    let encoding = compile_encoding(document, element)?;
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
    let normalization_form = compile_normalization_form(document, element)?;
    let standalone = optional_attribute(document, element, None, "standalone")
        .map(|value| parse_standalone(value, declared_version, document.location(element)))
        .transpose()?;
    let settings = OutputSettings {
        method: method.map(str::to_owned),
        version: compile_serialization_version(document, element)?.map(str::to_owned),
        encoding: encoding.map(str::to_owned),
        media_type: optional_attribute(document, element, None, "media-type").map(str::to_owned),
        doctype_system: optional_attribute(document, element, None, "doctype-system")
            .map(str::to_owned),
        doctype_public: optional_attribute(document, element, None, "doctype-public")
            .map(str::to_owned),
        include_content_type,
        byte_order_mark,
        normalization_form: normalization_form.map(str::to_owned),
        standalone,
        cdata_section_elements: compile_cdata_section_elements(document, element)?,
        omit_xml_declaration,
        indent,
    };
    let specified = document
        .attributes(element)
        .iter()
        .filter_map(|attribute| document.name(*attribute))
        .filter(|name| name.namespace.is_none())
        .map(|name| name.local.clone())
        .collect();
    Ok(OutputDeclaration {
        settings,
        specified,
        location: document.location(element).clone(),
    })
}

pub(super) fn merge_output(
    mut existing: OutputDeclaration,
    next: OutputDeclaration,
) -> Result<OutputDeclaration, CompileFailure> {
    let overlaps = existing
        .specified
        .intersection(&next.specified)
        .filter(|property| property.as_str() != "cdata-section-elements")
        .cloned()
        .collect::<Vec<_>>();
    if !overlaps.is_empty() {
        return Err(unsupported(
            "FXST1018",
            format!(
                "merging repeated output properties is outside the private slice: {}",
                overlaps.join(", ")
            ),
            &next.location,
        ));
    }
    merge_optional(&mut existing.settings.method, next.settings.method);
    merge_optional(&mut existing.settings.version, next.settings.version);
    merge_optional(&mut existing.settings.encoding, next.settings.encoding);
    merge_optional(&mut existing.settings.media_type, next.settings.media_type);
    merge_optional(
        &mut existing.settings.doctype_system,
        next.settings.doctype_system,
    );
    merge_optional(
        &mut existing.settings.doctype_public,
        next.settings.doctype_public,
    );
    merge_optional(
        &mut existing.settings.include_content_type,
        next.settings.include_content_type,
    );
    merge_optional(
        &mut existing.settings.byte_order_mark,
        next.settings.byte_order_mark,
    );
    merge_optional(
        &mut existing.settings.normalization_form,
        next.settings.normalization_form,
    );
    merge_optional(&mut existing.settings.standalone, next.settings.standalone);
    merge_optional(&mut existing.settings.indent, next.settings.indent);
    existing.settings.omit_xml_declaration |= next.settings.omit_xml_declaration;
    for name in next.settings.cdata_section_elements {
        if !existing.settings.cdata_section_elements.contains(&name) {
            existing.settings.cdata_section_elements.push(name);
        }
    }
    existing.specified.extend(next.specified);
    Ok(existing)
}

fn merge_optional<T>(target: &mut Option<T>, value: Option<T>) {
    if value.is_some() {
        *target = value;
    }
}

fn compile_encoding(document: &Document, element: NodeId) -> Result<Option<&str>, CompileFailure> {
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
    Ok(encoding)
}

fn compile_serialization_version(
    document: &Document,
    element: NodeId,
) -> Result<Option<&str>, CompileFailure> {
    let version = optional_attribute(document, element, None, "version");
    if version.is_some_and(|value| value != "1.0") {
        return Err(unsupported(
            "FXST1021",
            format!(
                "unsupported output serialization version: {}",
                version.unwrap_or_default()
            ),
            document.location(element),
        ));
    }
    Ok(version)
}

fn compile_normalization_form(
    document: &Document,
    element: NodeId,
) -> Result<Option<&str>, CompileFailure> {
    let value = optional_attribute(document, element, None, "normalization-form");
    if value.is_some_and(|value| value != "none") {
        return Err(unsupported(
            "FXST1017",
            format!(
                "unsupported output normalization form: {}",
                value.unwrap_or_default()
            ),
            document.location(element),
        ));
    }
    Ok(value)
}

fn compile_cdata_section_elements(
    document: &Document,
    element: NodeId,
) -> Result<Vec<ExpandedName>, CompileFailure> {
    let Some(value) = optional_attribute(document, element, None, "cdata-section-elements") else {
        return Ok(Vec::new());
    };
    value
        .split_whitespace()
        .map(|lexical| {
            let (prefix, local) = lexical
                .split_once(':')
                .map_or((None, lexical), |(prefix, local)| (Some(prefix), local));
            if !is_ascii_ncname(local) || prefix.is_some_and(|prefix| !is_ascii_ncname(prefix)) {
                return Err(invalid(
                    "FXST1019",
                    format!("invalid cdata-section-elements QName: {lexical}"),
                    document.location(element),
                ));
            }
            let namespace = namespace_for_prefix(document, element, prefix).ok_or_else(|| {
                invalid(
                    "FXST1020",
                    format!("unbound cdata-section-elements prefix: {lexical}"),
                    document.location(element),
                )
            })?;
            Ok(ExpandedName {
                namespace: (!namespace.is_empty()).then(|| namespace.to_owned()),
                local: local.to_owned(),
            })
        })
        .collect()
}

fn namespace_for_prefix<'a>(
    document: &'a Document,
    element: NodeId,
    prefix: Option<&str>,
) -> Option<&'a str> {
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(binding) = document
            .namespace_declarations(node)
            .iter()
            .find(|binding| binding.prefix.as_deref() == prefix)
        {
            return Some(binding.namespace.as_str());
        }
        current = document.parent(node);
    }
    prefix.is_none().then_some("")
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut chars = value.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.'))
}

fn parse_standalone(
    value: &str,
    declared_version: &str,
    location: &SourceLocation,
) -> Result<String, CompileFailure> {
    match value {
        "yes" | "no" | "omit" => Ok(value.to_owned()),
        _ if declared_version == "3.0" => match value.trim() {
            "true" | "1" => Ok("yes".to_owned()),
            "false" | "0" => Ok("no".to_owned()),
            _ => Err(invalid(
                "FXST0005",
                "standalone has an invalid XSLT 3.0 value",
                location,
            )),
        },
        _ => Err(invalid(
            "FXST0005",
            "standalone must be 'yes', 'no', or 'omit'",
            location,
        )),
    }
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
