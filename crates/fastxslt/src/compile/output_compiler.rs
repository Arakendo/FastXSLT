//! Private compilation of `xsl:output` declarations.

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use std::collections::BTreeSet;

use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::OutputSettings;

use super::{
    CompileFailure, compile_expanded_qname, ensure_no_meaningful_children, invalid,
    optional_attribute, unsupported,
};

const OUTPUT_ATTRIBUTES: &[&str] = &[
    "name",
    "method",
    "version",
    "encoding",
    "media-type",
    "doctype-system",
    "doctype-public",
    "escape-uri-attributes",
    "include-content-type",
    "byte-order-mark",
    "normalization-form",
    "use-character-maps",
    "undeclare-prefixes",
    "standalone",
    "cdata-section-elements",
    "suppress-indentation",
    "omit-xml-declaration",
    "indent",
    "html-version",
];

pub(in crate::compile) fn default_output_settings() -> OutputSettings {
    OutputSettings {
        method: None,
        version: None,
        html_version: None,
        encoding: None,
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        cdata_section_elements: Vec::new(),
        suppress_indentation_elements: Vec::new(),
        omit_xml_declaration: false,
        indent: None,
    }
}

pub(super) struct OutputDeclaration {
    pub(super) name: Option<ExpandedName>,
    pub(super) settings: OutputSettings,
    pub(super) character_map_names: Vec<ExpandedName>,
    pub(super) specified: BTreeSet<String>,
    pub(super) location: SourceLocation,
}

pub(super) fn compile_output(
    document: &Document,
    element: NodeId,
    declared_version: &str,
) -> Result<OutputDeclaration, CompileFailure> {
    ensure_output_attributes(document, element)?;
    ensure_no_meaningful_children(document, element, "xsl:output")?;
    let name = optional_attribute(document, element, None, "name")
        .map(|value| compile_expanded_qname(document, element, value, "xsl:output name"))
        .transpose()?;
    let html_version = compile_html_version(document, element)?;
    let method = optional_attribute(document, element, None, "method").map(str::trim);
    if method.is_some_and(|method| !matches!(method, "xml" | "text" | "xhtml" | "html")) {
        return Err(unsupported(
            "FXST1004",
            format!("unsupported output method: {}", method.unwrap_or_default()),
            document.location(element),
        ));
    }
    let encoding = optional_attribute(document, element, None, "encoding");
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
    let indent = compile_output_boolean_attribute(document, element, "indent", declared_version)?;
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
    let escape_uri_attributes =
        compile_bounded_escape_uri_attributes(document, element, method, declared_version)?;
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
    let normalization_form = optional_attribute(document, element, None, "normalization-form");
    let character_map_names = compile_character_map_names(document, element, method)?;
    let undeclare_prefixes = compile_output_boolean_attribute(
        document,
        element,
        "undeclare-prefixes",
        declared_version,
    )?;
    let standalone = optional_attribute(document, element, None, "standalone")
        .map(|value| parse_standalone(value, declared_version, document.location(element)))
        .transpose()?;
    let doctype_system = optional_attribute(document, element, None, "doctype-system");
    let doctype_public = optional_attribute(document, element, None, "doctype-public");
    validate_doctype_public(document, element, doctype_public)?;
    let version = compile_serialization_version(
        document,
        element,
        method,
        omit_xml_declaration && doctype_system.is_some(),
    )?;
    let settings = OutputSettings {
        method: method.map(str::to_owned),
        version: version.map(str::to_owned),
        html_version,
        encoding: encoding.map(str::to_owned),
        media_type: optional_attribute(document, element, None, "media-type").map(str::to_owned),
        doctype_system: doctype_system.map(str::to_owned),
        doctype_public: doctype_public.map(str::to_owned),
        include_content_type,
        escape_uri_attributes,
        byte_order_mark,
        normalization_form: normalization_form.map(str::to_owned),
        character_map: Vec::new(),
        undeclare_prefixes,
        standalone,
        cdata_section_elements: compile_cdata_section_elements(document, element)?,
        suppress_indentation_elements: compile_suppress_indentation_elements(document, element)?,
        omit_xml_declaration,
        indent,
    };
    let specified = compile_specified_properties(document, element);
    Ok(OutputDeclaration {
        name,
        settings,
        character_map_names,
        specified,
        location: document.location(element).clone(),
    })
}

fn compile_specified_properties(document: &Document, element: NodeId) -> BTreeSet<String> {
    document
        .attributes(element)
        .iter()
        .filter_map(|attribute| document.name(*attribute))
        .filter(|name| name.namespace.is_none() && name.local != "name")
        .map(|name| name.local.clone())
        .collect()
}

fn compile_html_version(
    document: &Document,
    element: NodeId,
) -> Result<Option<String>, CompileFailure> {
    let Some(value) = optional_attribute(document, element, None, "html-version") else {
        return Ok(None);
    };
    if !is_positive_decimal(value.trim()) {
        return Err(invalid(
            "XTSE0020",
            "html-version must be a positive decimal",
            document.location(element),
        ));
    }
    if is_decimal_five(value.trim()) {
        Ok(Some("5".to_owned()))
    } else {
        Err(unsupported(
            "FXST1049",
            "only XHTML html-version 5 is admitted by the private serializer",
            document.location(element),
        ))
    }
}

fn is_decimal_five(value: &str) -> bool {
    let value = value.strip_prefix('+').unwrap_or(value);
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    whole.trim_start_matches('0') == "5" && fraction.chars().all(|character| character == '0')
}

fn is_positive_decimal(value: &str) -> bool {
    let value = value.strip_prefix('+').unwrap_or(value);
    if value.is_empty() || value.starts_with('-') {
        return false;
    }
    let mut point_seen = false;
    let mut digit_seen = false;
    let mut nonzero_seen = false;
    for character in value.chars() {
        if character == '.' && !point_seen {
            point_seen = true;
        } else if character.is_ascii_digit() {
            digit_seen = true;
            nonzero_seen |= character != '0';
        } else {
            return false;
        }
    }
    digit_seen && nonzero_seen
}

fn compile_character_map_names(
    document: &Document,
    element: NodeId,
    method: Option<&str>,
) -> Result<Vec<ExpandedName>, CompileFailure> {
    let Some(value) = optional_attribute(document, element, None, "use-character-maps") else {
        return Ok(Vec::new());
    };
    let names: Vec<_> = value.split_whitespace().collect();
    if method.is_some_and(|method| !matches!(method, "xml" | "xhtml" | "html" | "text"))
        || names.is_empty()
    {
        return Err(unsupported(
            "FXST1048",
            "the admitted character-map slice requires QName names on XML, XHTML, bounded HTML, text, or inferred output",
            document.location(element),
        ));
    }
    names
        .into_iter()
        .map(|name| {
            compile_expanded_qname(document, element, name, "xsl:output use-character-maps")
        })
        .collect()
}

fn ensure_output_attributes(document: &Document, element: NodeId) -> Result<(), CompileFailure> {
    const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
    for attribute in document.attributes(element) {
        let name = document
            .name(*attribute)
            .expect("attribute nodes have expanded names");
        if (name.namespace.is_none() && OUTPUT_ATTRIBUTES.contains(&name.local.as_str()))
            || (name.namespace.as_deref() == Some(XML_NAMESPACE) && name.local == "space")
        {
            continue;
        }
        return Err(unsupported(
            "FXST1009",
            format!(
                "unsupported attribute on xsl:output: {{{}}}{}",
                name.namespace.as_deref().unwrap_or(""),
                name.local
            ),
            document.location(*attribute),
        ));
    }
    Ok(())
}

fn compile_bounded_escape_uri_attributes(
    document: &Document,
    element: NodeId,
    method: Option<&str>,
    declared_version: &str,
) -> Result<Option<bool>, CompileFailure> {
    let Some(value) = optional_attribute(document, element, None, "escape-uri-attributes") else {
        return Ok(None);
    };
    let enabled = parse_output_boolean(
        value,
        "escape-uri-attributes",
        declared_version,
        document.location(element),
    )?;
    if !matches!(method, Some("xml" | "xhtml" | "html")) {
        return Err(unsupported(
            "FXST1036",
            "escape-uri-attributes is admitted only for explicit XML, XHTML, or bounded HTML output",
            document.location(element),
        ));
    }
    Ok(Some(enabled))
}

pub(super) fn merge_output(
    mut existing: OutputDeclaration,
    next: OutputDeclaration,
) -> Result<OutputDeclaration, CompileFailure> {
    debug_assert!(existing.name.is_none() && next.name.is_none());
    let overlaps = existing
        .specified
        .intersection(&next.specified)
        .filter(|property| !repeat_is_compatible(&existing, &next, property))
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
    merge_optional(
        &mut existing.settings.html_version,
        next.settings.html_version,
    );
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
        &mut existing.settings.escape_uri_attributes,
        next.settings.escape_uri_attributes,
    );
    merge_optional(
        &mut existing.settings.byte_order_mark,
        next.settings.byte_order_mark,
    );
    merge_optional(
        &mut existing.settings.normalization_form,
        next.settings.normalization_form,
    );
    existing
        .character_map_names
        .extend(next.character_map_names);
    merge_optional(
        &mut existing.settings.undeclare_prefixes,
        next.settings.undeclare_prefixes,
    );
    merge_optional(&mut existing.settings.standalone, next.settings.standalone);
    merge_optional(&mut existing.settings.indent, next.settings.indent);
    existing.settings.omit_xml_declaration |= next.settings.omit_xml_declaration;
    for name in next.settings.cdata_section_elements {
        if !existing.settings.cdata_section_elements.contains(&name) {
            existing.settings.cdata_section_elements.push(name);
        }
    }
    for name in next.settings.suppress_indentation_elements {
        if !existing
            .settings
            .suppress_indentation_elements
            .contains(&name)
        {
            existing.settings.suppress_indentation_elements.push(name);
        }
    }
    existing.specified.extend(next.specified);
    Ok(existing)
}

fn repeat_is_compatible(
    existing: &OutputDeclaration,
    next: &OutputDeclaration,
    property: &str,
) -> bool {
    match property {
        "cdata-section-elements" | "use-character-maps" => true,
        "method" => existing.settings.method == next.settings.method,
        "html-version" => existing.settings.html_version == next.settings.html_version,
        "encoding" => existing.settings.encoding == next.settings.encoding,
        "indent" => existing.settings.indent == next.settings.indent,
        _ => false,
    }
}

fn merge_optional<T>(target: &mut Option<T>, value: Option<T>) {
    if value.is_some() {
        *target = value;
    }
}

fn compile_output_boolean_attribute(
    document: &Document,
    element: NodeId,
    local_name: &str,
    declared_version: &str,
) -> Result<Option<bool>, CompileFailure> {
    optional_attribute(document, element, None, local_name)
        .map(|value| {
            parse_output_boolean(
                value,
                local_name,
                declared_version,
                document.location(element),
            )
        })
        .transpose()
}

fn is_xml_public_identifier_char(value: char) -> bool {
    value.is_ascii_alphanumeric()
        || matches!(
            value,
            ' ' | '\r'
                | '\n'
                | '-'
                | '\''
                | '('
                | ')'
                | '+'
                | ','
                | '.'
                | '/'
                | ':'
                | '='
                | '?'
                | ';'
                | '!'
                | '*'
                | '#'
                | '@'
                | '$'
                | '_'
                | '%'
        )
}

fn validate_doctype_public(
    document: &Document,
    element: NodeId,
    value: Option<&str>,
) -> Result<(), CompileFailure> {
    if value.is_some_and(|value| !value.chars().all(is_xml_public_identifier_char)) {
        return Err(invalid(
            "XTSE0020",
            "doctype-public contains a character outside the XML public identifier set",
            document.location(element),
        ));
    }
    Ok(())
}

fn compile_serialization_version<'a>(
    document: &'a Document,
    element: NodeId,
    method: Option<&str>,
    admitted_inconsistent_error_path: bool,
) -> Result<Option<&'a str>, CompileFailure> {
    let version = optional_attribute(document, element, None, "version");
    if version.is_some_and(|value| value != "1.0")
        && method != Some("html")
        && !admitted_inconsistent_error_path
    {
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

fn compile_suppress_indentation_elements(
    document: &Document,
    element: NodeId,
) -> Result<Vec<ExpandedName>, CompileFailure> {
    let Some(value) = optional_attribute(document, element, None, "suppress-indentation") else {
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
                    "FXST1021",
                    format!("invalid suppress-indentation QName: {lexical}"),
                    document.location(element),
                ));
            }
            let namespace = namespace_for_prefix(document, element, prefix).ok_or_else(|| {
                invalid(
                    "FXST1022",
                    format!("unbound suppress-indentation prefix: {lexical}"),
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
                "XTSE0020",
                "standalone has an invalid XSLT 3.0 value",
                location,
            )),
        },
        _ => Err(invalid(
            "XTSE0020",
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
                "XTSE0020",
                format!("{attribute} has an invalid XSLT 3.0 boolean value"),
                location,
            )),
        },
        _ => Err(invalid(
            "XTSE0020",
            format!("{attribute} must be 'yes' or 'no'"),
            location,
        )),
    }
}
