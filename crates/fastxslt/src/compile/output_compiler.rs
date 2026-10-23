//! Private compilation of `xsl:output` declarations.

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use std::collections::BTreeSet;

use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::OutputSettings;

use super::instruction_compiler::uses_xslt10_compatibility;
use super::{
    CompileFailure, compile_expanded_qname, ensure_no_meaningful_children, invalid,
    is_ignored_xslt10_extension_attribute, is_xslt10_forward_compatible_version,
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

/// A projection of the caller's compilation policy for Boolean output values.
/// No declared-version rewriting, source mutation or public profile selector.
#[derive(Clone, Copy)]
pub(super) struct OutputBooleanPolicy {
    modern_aliases: bool,
    ignore_invalid_optional: bool,
    ignore_invalid_omit_declaration: bool,
}

impl OutputBooleanPolicy {
    fn baseline(declared_version: &str) -> Self {
        Self {
            modern_aliases: declared_version == "3.0",
            ignore_invalid_optional: is_xslt10_forward_compatible_version(declared_version),
            ignore_invalid_omit_declaration: false,
        }
    }

    pub(super) const fn selected_legacy(forward: bool) -> Self {
        Self {
            modern_aliases: false,
            ignore_invalid_optional: forward,
            ignore_invalid_omit_declaration: forward,
        }
    }
}

pub(super) fn compile_output(
    document: &Document,
    element: NodeId,
    declared_version: &str,
    allow_superseded_xslt10_extension_method: bool,
) -> Result<OutputDeclaration, CompileFailure> {
    compile_output_with_boolean_policy(
        document,
        element,
        declared_version,
        allow_superseded_xslt10_extension_method,
        OutputBooleanPolicy::baseline(declared_version),
        &mut ensure_output_attributes,
    )
}

pub(super) fn compile_output_with_boolean_policy(
    document: &Document,
    element: NodeId,
    declared_version: &str,
    allow_superseded_xslt10_extension_method: bool,
    policy: OutputBooleanPolicy,
    validate_attributes: &mut impl FnMut(&Document, NodeId) -> Result<(), CompileFailure>,
) -> Result<OutputDeclaration, CompileFailure> {
    validate_attributes(document, element)?;
    ensure_no_meaningful_children(document, element, "xsl:output")?;
    let name = optional_attribute(document, element, None, "name")
        .map(|value| compile_expanded_qname(document, element, value, "xsl:output name"))
        .transpose()?;
    let html_version = compile_html_version(document, element)?;
    let method =
        compile_output_method(document, element, allow_superseded_xslt10_extension_method)?;
    let encoding = compile_output_encoding(document, element);
    let omit_value = compile_output_boolean_attribute(
        document,
        element,
        "omit-xml-declaration",
        OutputBooleanPolicy {
            ignore_invalid_optional: policy.ignore_invalid_omit_declaration,
            ..policy
        },
    )?;
    let omit_xml_declaration = omit_value.unwrap_or(false);
    let indent = compile_output_boolean_attribute(document, element, "indent", policy)?;
    let include_content_type = optional_attribute(document, element, None, "include-content-type")
        .map(|value| {
            parse_output_boolean(
                value,
                "include-content-type",
                policy,
                document.location(element),
            )
        })
        .transpose()?;
    let escape_uri_attributes =
        compile_bounded_escape_uri_attributes(document, element, method, policy)?;
    let byte_order_mark = optional_attribute(document, element, None, "byte-order-mark")
        .map(|value| {
            parse_output_boolean(value, "byte-order-mark", policy, document.location(element))
        })
        .transpose()?;
    let normalization_form = optional_attribute(document, element, None, "normalization-form");
    let character_map_names = compile_character_map_names(document, element, method)?;
    let undeclare_prefixes =
        compile_output_boolean_attribute(document, element, "undeclare-prefixes", policy)?;
    let standalone = compile_standalone(document, element, declared_version, omit_xml_declaration)?;
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
    let mut specified = compile_specified_properties(document, element);
    // These optional booleans can be ignored by forward-compatible validation.
    // Their lexical presence must not survive as a merge-conflict declaration.
    for (property, value) in [
        ("indent", settings.indent),
        ("undeclare-prefixes", settings.undeclare_prefixes),
        ("omit-xml-declaration", omit_value),
    ] {
        if value.is_none() {
            specified.remove(property);
        }
    }
    Ok(OutputDeclaration {
        name,
        settings,
        character_map_names,
        specified,
        location: document.location(element).clone(),
    })
}

fn compile_output_method(
    document: &Document,
    element: NodeId,
    allow_superseded_xslt10_extension_method: bool,
) -> Result<Option<&str>, CompileFailure> {
    let method = optional_attribute(document, element, None, "method").map(str::trim);
    let Some(extension) =
        method.filter(|method| !matches!(*method, "xml" | "text" | "xhtml" | "html"))
    else {
        return Ok(method);
    };
    let expanded = compile_expanded_qname(document, element, extension, "xsl:output method")?;
    if expanded.namespace.is_none() {
        return Err(invalid(
            "XTSE1570",
            format!("an extension output method must have a non-null namespace URI: {extension}"),
            document.location(element),
        ));
    }
    if allow_superseded_xslt10_extension_method {
        return Ok(method);
    }
    Err(unsupported(
        "FXST1004",
        format!("unsupported output method: {extension}"),
        document.location(element),
    ))
}

fn compile_output_encoding(document: &Document, element: NodeId) -> Option<&str> {
    let encoding = optional_attribute(document, element, None, "encoding").map(str::trim);
    if uses_xslt10_compatibility(document, element)
        && encoding.is_some_and(|value| value.eq_ignore_ascii_case("unicode"))
    {
        // The OASIS 1.0 corpus preserves the historical Microsoft/MSXML label
        // for UTF-16. Keep this compatibility spelling out of modern output
        // settings while routing the legacy profile through the existing
        // bounded UTF-16 byte serializer.
        Some("UTF-16")
    } else {
        encoding
    }
}

fn compile_standalone(
    document: &Document,
    element: NodeId,
    declared_version: &str,
    omit_xml_declaration: bool,
) -> Result<Option<String>, CompileFailure> {
    let standalone = optional_attribute(document, element, None, "standalone")
        .map(|value| parse_standalone(value, declared_version, document.location(element)))
        .transpose()?;
    if omit_xml_declaration && uses_xslt10_compatibility(document, element) {
        Ok(None)
    } else {
        Ok(standalone)
    }
}

fn compile_specified_properties(document: &Document, element: NodeId) -> BTreeSet<String> {
    document
        .attributes(element)
        .iter()
        .filter_map(|attribute| document.name(*attribute))
        .filter(|name| name.namespace.is_none() && name.local != "name")
        .filter(|name| is_known_output_attribute(&name.local))
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
            || is_ignored_xslt10_extension_attribute(document, element, *attribute)
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

pub(super) fn is_known_output_attribute(local: &str) -> bool {
    OUTPUT_ATTRIBUTES.contains(&local)
}

fn compile_bounded_escape_uri_attributes(
    document: &Document,
    element: NodeId,
    method: Option<&str>,
    policy: OutputBooleanPolicy,
) -> Result<Option<bool>, CompileFailure> {
    let Some(value) = optional_attribute(document, element, None, "escape-uri-attributes") else {
        return Ok(None);
    };
    let enabled = parse_output_boolean(
        value,
        "escape-uri-attributes",
        policy,
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
    recover_conflicts_by_declaration_order: bool,
) -> Result<OutputDeclaration, CompileFailure> {
    debug_assert!(existing.name.is_none() && next.name.is_none());
    let overlaps = existing
        .specified
        .intersection(&next.specified)
        .filter(|property| !repeat_is_compatible(&existing, &next, property))
        .cloned()
        .collect::<Vec<_>>();
    if !overlaps.is_empty() && !recover_conflicts_by_declaration_order {
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
    if next.specified.contains("omit-xml-declaration") {
        existing.settings.omit_xml_declaration = next.settings.omit_xml_declaration;
    }
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
    if recover_conflicts_by_declaration_order && existing.settings.omit_xml_declaration {
        existing.settings.standalone = None;
    }
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
        "omit-xml-declaration" => {
            existing.settings.omit_xml_declaration == next.settings.omit_xml_declaration
        }
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
    policy: OutputBooleanPolicy,
) -> Result<Option<bool>, CompileFailure> {
    let Some(value) = optional_attribute(document, element, None, local_name) else {
        return Ok(None);
    };
    match parse_output_boolean(value, local_name, policy, document.location(element)) {
        Ok(value) => Ok(Some(value)),
        Err(_) if policy.ignore_invalid_optional => Ok(None),
        Err(failure) => Err(failure),
    }
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
    if uses_xslt10_compatibility(document, element)
        && method == Some("html")
        && version.is_some_and(|value| value.trim().is_empty())
    {
        return Ok(None);
    }
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
    crate::xml::names::is_ncname(value)
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
    policy: OutputBooleanPolicy,
    location: &SourceLocation,
) -> Result<bool, CompileFailure> {
    match value {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ if policy.modern_aliases => match value.trim() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::golden_stylesheet_experiment::{CompileCategory, compile_stylesheet};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn compile(
        version: &str,
        outputs: &str,
    ) -> Result<crate::xslt::golden_semantics_experiment::StylesheetProgram, CompileFailure> {
        let xml = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}">{outputs}<xsl:template match="/"><out><child/></out></xsl:template></xsl:stylesheet>"#
        );
        let document = Document::from_parsed(
            parse_document(
                "urn:output-merge",
                xml.as_bytes(),
                ParseLimits {
                    max_events: 256,
                    max_depth: 32,
                },
            )
            .unwrap(),
        )
        .unwrap();
        compile_stylesheet(&document)
    }

    #[test]
    fn ignored_output_boolean_does_not_create_a_merge_conflict() {
        for property in ["indent", "undeclare-prefixes"] {
            for value in ["yes", "no"] {
                for invalid_first in [false, true] {
                    let valid = format!(r#"<xsl:output {property}="{value}"/>"#);
                    let ignored = format!(r#"<xsl:output {property}="sure"/>"#);
                    let outputs = if invalid_first {
                        format!("{ignored}{valid}")
                    } else {
                        format!("{valid}{ignored}")
                    };
                    let program = compile("1.5", &outputs).unwrap();
                    let actual = if property == "indent" {
                        program.output.indent
                    } else {
                        program.output.undeclare_prefixes
                    };
                    assert_eq!(actual, Some(value == "yes"));
                    if property == "indent" {
                        use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test as execute;
                        let source = Document::from_parsed(
                            parse_document(
                                "urn:source",
                                b"<r/>",
                                ParseLimits {
                                    max_events: 16,
                                    max_depth: 4,
                                },
                            )
                            .unwrap(),
                        )
                        .unwrap();
                        let reference = compile("1.5", &valid).unwrap();
                        assert_eq!(
                            execute(&program, &source, "merged").unwrap(),
                            execute(&reference, &source, "reference").unwrap()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn ignored_output_boolean_is_not_a_specified_property() {
        let program = compile(
            "1.5",
            r#"<xsl:output indent="sure" undeclare-prefixes="sure"/>"#,
        )
        .unwrap();
        assert_eq!(program.output.indent, None);
        assert_eq!(program.output.undeclare_prefixes, None);
        for property in ["indent", "undeclare-prefixes"] {
            assert!(
                !program
                    .output_specified_properties
                    .iter()
                    .any(|p| p == property)
            );
            assert!(
                !program
                    .output_same_precedence_properties
                    .iter()
                    .any(|p| p == property)
            );
        }
    }

    #[test]
    fn output_boolean_merge_keeps_actual_conflicts_and_invalid_nonforward_values() {
        let failure = compile(
            "1.5",
            r#"<xsl:output indent="yes"/><xsl:output indent="no"/>"#,
        )
        .unwrap_err();
        assert_eq!(failure.code, "FXST1018");
        for version in ["1.0", "2.0", "3.0"] {
            let failure = compile(version, r#"<xsl:output indent="sure"/>"#).unwrap_err();
            assert_eq!(failure.category, CompileCategory::Invalid);
            assert_eq!(failure.code, "XTSE0020");
            assert_eq!(failure.location.resource, "urn:output-merge");
        }
    }
}
