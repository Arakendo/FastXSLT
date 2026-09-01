//! Private serialization of the golden slice's semantic result.

use super::{
    ExecutionFailure, FailureCategory, ResultNode, SemanticResult, control_failure, failure,
};
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xslt::golden_semantics_experiment::OutputSettings;

#[derive(Clone, Copy)]
struct SerializationOptions<'a> {
    cdata_section_elements: &'a [crate::xml::quick_xml_experiment::ExpandedName],
    character_map: &'a [(char, String)],
    xhtml: bool,
    xhtml_media_type: Option<&'a str>,
    indent: bool,
}

pub(in crate::runtime) fn serialize_xml(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
    byte_limit: usize,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    validate_serialization_preconditions(result, settings, request_id)?;
    validate_normalization_form(settings, request_id)?;
    validate_string_encoding(settings, request_id)?;
    if settings.byte_order_mark == Some(true) {
        return Err(failure(
            "FXSR1005",
            FailureCategory::Unsupported,
            Some(request_id),
            "byte-order-mark=yes requires a future byte serialization result lane",
        ));
    }
    let first_significant = result.children.iter().find(|node| match node {
        ResultNode::Text(value) => !value.chars().all(char::is_whitespace),
        ResultNode::Element { .. } => true,
        ResultNode::ProcessingInstruction { .. } => false,
    });
    let unsupported_adaptive_html = settings.method.is_none()
        && matches!(
            first_significant,
            Some(ResultNode::Element { name, .. })
                if name.namespace.is_none() && name.local.eq_ignore_ascii_case("html")
        );
    if unsupported_adaptive_html {
        return Err(failure(
            "FXSR1001",
            FailureCategory::Unsupported,
            Some(request_id),
            "the selected output method is outside the private XML serialization slice",
        ));
    }
    let default_is_xhtml = settings.method.is_none()
        && matches!(
            first_significant,
            Some(ResultNode::Element { name, .. })
                if name.namespace.as_deref() == Some("http://www.w3.org/1999/xhtml")
                    && name.local.eq_ignore_ascii_case("html")
        );
    if settings.method.as_deref() == Some("text") {
        let mut output = BudgetedString::new(byte_limit, request_id, control);
        for node in &result.children {
            serialize_text_node(node, &settings.character_map, &mut output)?;
        }
        return Ok(output.finish());
    }
    if settings
        .method
        .as_deref()
        .is_some_and(|method| !matches!(method, "xml" | "xhtml"))
    {
        return Err(failure(
            "FXSR1001",
            FailureCategory::Unsupported,
            Some(request_id),
            "the selected output method is outside the private XML-compatible serialization slice",
        ));
    }
    let xhtml = settings.method.as_deref() == Some("xhtml") || default_is_xhtml;
    let mut output = BudgetedString::new(byte_limit, request_id, control);
    if !settings.omit_xml_declaration {
        output.push_str("<?xml version=\"")?;
        output.push_str(settings.version.as_deref().unwrap_or("1.0"))?;
        output.push_str("\" encoding=\"UTF-8\"")?;
        if let Some(standalone @ ("yes" | "no")) = settings.standalone.as_deref() {
            output.push_str(" standalone=\"")?;
            output.push_str(standalone)?;
            output.push('"')?;
        }
        output.push_str("?>")?;
    }
    serialize_xhtml_doctype(result, settings, xhtml, &mut output)?;
    let xhtml_media_type = (xhtml && settings.include_content_type != Some(false))
        .then(|| settings.media_type.as_deref().unwrap_or("text/html"));
    let options = SerializationOptions {
        cdata_section_elements: &settings.cdata_section_elements,
        character_map: &settings.character_map,
        xhtml,
        xhtml_media_type,
        indent: settings.indent == Some(true),
    };
    for node in &result.children {
        serialize_node(node, &[], options, 0, &mut output)?;
    }
    Ok(output.finish())
}

fn validate_string_encoding(
    settings: &OutputSettings,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    if let Some(encoding) = settings
        .encoding
        .as_deref()
        .filter(|encoding| !encoding.eq_ignore_ascii_case("UTF-8"))
    {
        let (code, detail) = if encoding.eq_ignore_ascii_case("ISO-8859-1") {
            (
                "FXSR1004",
                "the private string serialization lane supports only UTF-8; use the bounded byte lane for ISO-8859-1".to_owned(),
            )
        } else {
            (
                "SESU0007",
                format!("the requested output encoding is not supported: {encoding}"),
            )
        };
        return Err(failure(
            code,
            FailureCategory::Unsupported,
            Some(request_id),
            detail,
        ));
    }
    Ok(())
}

fn validate_normalization_form(
    settings: &OutputSettings,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    if let Some(normalization_form) = settings
        .normalization_form
        .as_deref()
        .filter(|value| *value != "none")
    {
        return Err(failure(
            "SESU0011",
            FailureCategory::Unsupported,
            Some(request_id),
            format!("the requested normalization form is not supported: {normalization_form}"),
        ));
    }
    Ok(())
}

fn serialize_xhtml_doctype(
    result: &SemanticResult,
    settings: &OutputSettings,
    xhtml: bool,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    let Some(system) = settings.doctype_system.as_deref() else {
        return Ok(());
    };
    if !xhtml || !is_xhtml_html_document(result) {
        return Err(failure(
            "FXSR1007",
            FailureCategory::Unsupported,
            Some(&output.request_id),
            "DOCTYPE serialization is currently bounded to an XHTML html document element",
        ));
    }
    output.push_str("<!DOCTYPE html")?;
    if let Some(public) = settings.doctype_public.as_deref() {
        output.push_str(" PUBLIC ")?;
        serialize_external_identifier(public, output)?;
        output.push(' ')?;
    } else {
        output.push_str(" SYSTEM ")?;
    }
    serialize_external_identifier(system, output)?;
    output.push('>')
}

fn is_xhtml_html_document(result: &SemanticResult) -> bool {
    let mut root_seen = false;
    for node in &result.children {
        match node {
            ResultNode::Text(value) if value.chars().all(char::is_whitespace) => {}
            ResultNode::ProcessingInstruction { .. } => {}
            ResultNode::Element { name, .. }
                if !root_seen
                    && name.namespace.as_deref() == Some("http://www.w3.org/1999/xhtml")
                    && name.local == "html" =>
            {
                root_seen = true;
            }
            _ => return false,
        }
    }
    root_seen
}

fn serialize_external_identifier(
    value: &str,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    let delimiter = if !value.contains('"') {
        '"'
    } else if !value.contains('\'') {
        '\''
    } else {
        return Err(failure(
            "FXSR1008",
            FailureCategory::Unsupported,
            Some(&output.request_id),
            "DOCTYPE identifier containing both quote forms is outside the private slice",
        ));
    };
    output.push(delimiter)?;
    output.push_str(value)?;
    output.push(delimiter)
}

#[cfg(test)]
pub(in crate::runtime) fn serialize_xml_bytes(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
    byte_limit: usize,
    control: &mut InvocationControl,
) -> Result<Vec<u8>, ExecutionFailure> {
    validate_serialization_preconditions(result, settings, request_id)?;
    validate_normalization_form(settings, request_id)?;
    let encoding = settings.encoding.as_deref().unwrap_or("UTF-8");
    if encoding.eq_ignore_ascii_case("UTF-8") {
        let bom = settings.byte_order_mark == Some(true);
        let body_limit = byte_limit
            .checked_sub(usize::from(bom) * 3)
            .ok_or_else(|| {
                failure(
                    "FXSR0002",
                    FailureCategory::Limit,
                    Some(request_id),
                    format!("serialized result requires at least 3 bytes; limit is {byte_limit}"),
                )
            })?;
        if bom {
            control
                .charge(WorkDomain::SerializedByte, 3)
                .map_err(|failure| control_failure(failure, request_id))?;
        }
        let mut body_settings = settings.clone();
        body_settings.byte_order_mark = Some(false);
        let body = serialize_xml(result, &body_settings, request_id, body_limit, control)?;
        let mut bytes = Vec::with_capacity(usize::from(bom) * 3 + body.len());
        if bom {
            bytes.extend_from_slice(&[0xef, 0xbb, 0xbf]);
        }
        bytes.extend_from_slice(body.as_bytes());
        return Ok(bytes);
    }
    if !encoding.eq_ignore_ascii_case("ISO-8859-1") {
        return Err(failure(
            "SESU0007",
            FailureCategory::Unsupported,
            Some(request_id),
            format!("the requested output encoding is not supported: {encoding}"),
        ));
    }
    if settings.byte_order_mark == Some(true) {
        return Err(failure(
            "FXSR1005",
            FailureCategory::Unsupported,
            Some(request_id),
            "the bounded ISO-8859-1 lane does not emit a byte-order mark",
        ));
    }

    let declaration = if settings.omit_xml_declaration || settings.method.as_deref() == Some("text")
    {
        String::new()
    } else {
        format!("<?xml version=\"1.0\" encoding=\"{encoding}\"?>")
    };
    let body_limit = byte_limit.checked_sub(declaration.len()).ok_or_else(|| {
        failure(
            "FXSR0002",
            FailureCategory::Limit,
            Some(request_id),
            format!(
                "serialized result requires at least {} bytes; limit is {byte_limit}",
                declaration.len()
            ),
        )
    })?;
    if !declaration.is_empty() {
        control
            .charge(WorkDomain::SerializedByte, declaration.len())
            .map_err(|failure| control_failure(failure, request_id))?;
    }

    let mut body_settings = settings.clone();
    body_settings.encoding = Some("UTF-8".to_owned());
    body_settings.omit_xml_declaration = true;
    body_settings.standalone = None;
    body_settings.version = Some("1.0".to_owned());
    let body = serialize_xml(result, &body_settings, request_id, body_limit, control)?;
    if !body.is_ascii() {
        return Err(failure(
            "FXSR1006",
            FailureCategory::Unsupported,
            Some(request_id),
            "the bounded ISO-8859-1 lane currently admits only ASCII result characters",
        ));
    }

    let mut bytes = declaration.into_bytes();
    bytes.extend_from_slice(body.as_bytes());
    Ok(bytes)
}

fn validate_serialization_preconditions(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    if settings.method.as_deref() == Some("xml")
        && settings.undeclare_prefixes == Some(true)
        && settings.version.as_deref().unwrap_or("1.0") == "1.0"
    {
        return Err(failure(
            "SEPM0010",
            FailureCategory::Invalid,
            Some(request_id),
            "undeclare-prefixes=yes is inconsistent with XML output version 1.0",
        ));
    }
    let standalone_requires_declaration = settings.omit_xml_declaration
        && matches!(settings.standalone.as_deref(), Some("yes" | "no"));
    let doctype_requires_xml_10 = settings.omit_xml_declaration
        && settings
            .version
            .as_deref()
            .is_some_and(|version| version != "1.0")
        && settings.doctype_system.is_some();
    if standalone_requires_declaration || doctype_requires_xml_10 {
        return Err(failure(
            "SEPM0009",
            FailureCategory::Invalid,
            Some(request_id),
            "the selected serialization parameters are internally inconsistent",
        ));
    }
    let document_element_required = matches!(settings.standalone.as_deref(), Some("yes" | "no"))
        || settings.doctype_system.is_some();
    let top_level_element_count = result
        .children
        .iter()
        .filter(|node| matches!(node, ResultNode::Element { .. }))
        .count();
    if document_element_required && top_level_element_count != 1 {
        return Err(failure(
            "SEPM0004",
            FailureCategory::Invalid,
            Some(request_id),
            "the selected serialization parameters require exactly one top-level element",
        ));
    }
    Ok(())
}

fn serialize_text_node(
    node: &ResultNode,
    character_map: &[(char, String)],
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    match node {
        ResultNode::Text(value) => write_character_mapped(value, character_map, output),
        ResultNode::ProcessingInstruction { .. } => Ok(()),
        ResultNode::Element { children, .. } => {
            for child in children {
                serialize_text_node(child, character_map, output)?;
            }
            Ok(())
        }
    }
}

fn write_character_mapped(
    value: &str,
    character_map: &[(char, String)],
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    for character in value.chars() {
        if let Some((_, replacement)) = character_map
            .iter()
            .find(|(candidate, _)| *candidate == character)
        {
            output.push_str(replacement)?;
        } else {
            output.push(character)?;
        }
    }
    Ok(())
}

fn serialize_node(
    node: &ResultNode,
    inherited_namespaces: &[crate::xml::quick_xml_experiment::NamespaceBinding],
    options: SerializationOptions<'_>,
    depth: usize,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    match node {
        ResultNode::Text(value) => escape_text(value, options.character_map, output)?,
        ResultNode::ProcessingInstruction { target, value } => {
            serialize_processing_instruction(target, value, output)?;
        }
        ResultNode::Element { .. } => {
            serialize_element(node, inherited_namespaces, options, depth, output)?;
        }
    }
    Ok(())
}

fn serialize_processing_instruction(
    target: &str,
    value: &str,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    output.push_str("<?")?;
    output.push_str(target)?;
    if !value.is_empty() {
        output.push(' ')?;
        output.push_str(value)?;
    }
    output.push_str("?>")
}

fn serialize_element(
    node: &ResultNode,
    inherited_namespaces: &[crate::xml::quick_xml_experiment::NamespaceBinding],
    options: SerializationOptions<'_>,
    depth: usize,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    let ResultNode::Element {
        name,
        namespaces,
        attributes,
        children,
    } = node
    else {
        unreachable!("serialize_element receives an element")
    };
    let (in_scope, declarations) = element_namespace_scope(name, namespaces, inherited_namespaces);
    let prefix = element_prefix(name.namespace.as_deref(), &in_scope, output)?;
    output.push('<')?;
    write_name(prefix, &name.local, output)?;
    for binding in &declarations {
        output.push_str(" xmlns")?;
        if let Some(prefix) = &binding.prefix {
            output.push(':')?;
            output.push_str(prefix)?;
        }
        output.push_str("=\"")?;
        escape_attribute(&binding.namespace, output)?;
        output.push('"')?;
    }
    for attribute in attributes {
        output.push(' ')?;
        let prefix = attribute_prefix(attribute.name.namespace.as_deref(), &in_scope, output)?;
        write_name(prefix, &attribute.name.local, output)?;
        output.push_str("=\"")?;
        escape_attribute_with_character_map(&attribute.value, options.character_map, output)?;
        output.push('"')?;
    }
    if options.xhtml && children.is_empty() && is_xhtml_void_element(name) {
        return output.push_str(" />");
    }
    output.push('>')?;
    let inject_content_type = options
        .xhtml_media_type
        .is_some_and(|_| is_xhtml_head(name));
    let indent_children = options.indent
        && (inject_content_type
            || children
                .iter()
                .any(|child| !is_replaced_content_type_meta(child, inject_content_type)))
        && children
            .iter()
            .filter(|child| !is_replaced_content_type_meta(child, inject_content_type))
            .all(|child| matches!(child, ResultNode::Element { .. }));
    if let Some(media_type) = options.xhtml_media_type.filter(|_| inject_content_type) {
        if indent_children {
            write_indentation(depth + 1, output)?;
        }
        serialize_xhtml_content_type_meta(media_type, output)?;
    }
    for child in children {
        if is_replaced_content_type_meta(child, inject_content_type) {
            continue;
        }
        if indent_children {
            write_indentation(depth + 1, output)?;
        }
        if options.cdata_section_elements.contains(name)
            && let ResultNode::Text(value) = child
        {
            serialize_cdata(value, output)?;
            continue;
        }
        serialize_node(child, &in_scope, options, depth + 1, output)?;
    }
    if indent_children {
        write_indentation(depth, output)?;
    }
    output.push_str("</")?;
    write_name(prefix, &name.local, output)?;
    output.push('>')
}

fn is_xhtml_void_element(name: &crate::xml::quick_xml_experiment::ExpandedName) -> bool {
    name.namespace.as_deref() == Some("http://www.w3.org/1999/xhtml")
        && matches!(
            name.local.as_str(),
            "area"
                | "base"
                | "basefont"
                | "br"
                | "col"
                | "frame"
                | "hr"
                | "img"
                | "input"
                | "isindex"
                | "link"
                | "meta"
                | "param"
        )
}

fn element_namespace_scope(
    name: &crate::xml::quick_xml_experiment::ExpandedName,
    namespaces: &[crate::xml::quick_xml_experiment::NamespaceBinding],
    inherited_namespaces: &[crate::xml::quick_xml_experiment::NamespaceBinding],
) -> (
    Vec<crate::xml::quick_xml_experiment::NamespaceBinding>,
    Vec<crate::xml::quick_xml_experiment::NamespaceBinding>,
) {
    let mut in_scope = inherited_namespaces.to_vec();
    let mut declarations = Vec::new();
    for binding in namespaces {
        let inherited = in_scope.iter().position(|candidate| {
            candidate.prefix == binding.prefix && candidate.namespace == binding.namespace
        });
        if inherited.is_none() {
            declarations.push(binding.clone());
        }
        in_scope.retain(|candidate| candidate.prefix != binding.prefix);
        in_scope.push(binding.clone());
    }
    if name.namespace.is_none()
        && in_scope
            .iter()
            .any(|binding| binding.prefix.is_none() && !binding.namespace.is_empty())
    {
        let undeclaration = crate::xml::quick_xml_experiment::NamespaceBinding {
            prefix: None,
            namespace: String::new(),
        };
        declarations.push(undeclaration.clone());
        in_scope.retain(|binding| binding.prefix.is_some());
        in_scope.push(undeclaration);
    }
    (in_scope, declarations)
}

fn is_xhtml_head(name: &crate::xml::quick_xml_experiment::ExpandedName) -> bool {
    name.namespace.as_deref() == Some("http://www.w3.org/1999/xhtml") && name.local == "head"
}

fn is_replaced_content_type_meta(node: &ResultNode, replace: bool) -> bool {
    replace
        && matches!(
            node,
            ResultNode::Element {
                name,
                attributes,
                ..
            } if name.namespace.as_deref() == Some("http://www.w3.org/1999/xhtml")
                && name.local == "meta"
                && attributes.iter().any(|attribute| {
                    attribute.name.namespace.is_none()
                        && attribute.name.local.eq_ignore_ascii_case("http-equiv")
                        && attribute.value.eq_ignore_ascii_case("Content-Type")
                })
        )
}

fn serialize_xhtml_content_type_meta(
    media_type: &str,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    output.push_str("<meta http-equiv=\"Content-Type\" content=\"")?;
    escape_attribute(media_type, output)?;
    output.push_str("; charset=UTF-8\" />")
}

fn write_indentation(depth: usize, output: &mut BudgetedString) -> Result<(), ExecutionFailure> {
    output.push('\n')?;
    for _ in 0..depth {
        output.push_str("  ")?;
    }
    Ok(())
}

fn serialize_cdata(value: &str, output: &mut BudgetedString) -> Result<(), ExecutionFailure> {
    output.push_str("<![CDATA[")?;
    output.push_str(&value.replace("]]>", "]]]]><![CDATA[>"))?;
    output.push_str("]]>")
}

fn attribute_prefix<'a>(
    namespace: Option<&str>,
    in_scope: &'a [crate::xml::quick_xml_experiment::NamespaceBinding],
    output: &BudgetedString,
) -> Result<Option<&'a str>, ExecutionFailure> {
    let Some(namespace) = namespace else {
        return Ok(None);
    };
    if namespace == "http://www.w3.org/XML/1998/namespace" {
        return Ok(Some("xml"));
    }
    in_scope
        .iter()
        .find(|binding| binding.prefix.is_some() && binding.namespace == namespace)
        .and_then(|binding| binding.prefix.as_deref())
        .map(Some)
        .ok_or_else(|| {
            failure(
                "FXSR1002",
                FailureCategory::Unsupported,
                Some(&output.request_id),
                format!("result attribute namespace has no retained prefix binding: {namespace}"),
            )
        })
}

fn element_prefix<'a>(
    namespace: Option<&str>,
    in_scope: &'a [crate::xml::quick_xml_experiment::NamespaceBinding],
    output: &BudgetedString,
) -> Result<Option<&'a str>, ExecutionFailure> {
    let Some(namespace) = namespace else {
        return Ok(None);
    };
    in_scope
        .iter()
        .filter(|binding| binding.namespace == namespace)
        .min_by_key(|binding| usize::from(binding.prefix.is_some()))
        .map(|binding| binding.prefix.as_deref())
        .ok_or_else(|| {
            failure(
                "FXSR1002",
                FailureCategory::Unsupported,
                Some(&output.request_id),
                format!("result namespace has no retained prefix binding: {namespace}"),
            )
        })
}

fn write_name(
    prefix: Option<&str>,
    local: &str,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    if let Some(prefix) = prefix {
        output.push_str(prefix)?;
        output.push(':')?;
    }
    output.push_str(local)
}

fn escape_attribute(value: &str, output: &mut BudgetedString) -> Result<(), ExecutionFailure> {
    for character in value.chars() {
        escape_attribute_character(character, output)?;
    }
    Ok(())
}

fn escape_attribute_character(
    character: char,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    match character {
        '&' => output.push_str("&amp;"),
        '<' => output.push_str("&lt;"),
        '"' => output.push_str("&quot;"),
        '\t' => output.push_str("&#x9;"),
        '\n' => output.push_str("&#xA;"),
        '\r' => output.push_str("&#xD;"),
        _ => output.push(character),
    }
}

fn escape_attribute_with_character_map(
    value: &str,
    character_map: &[(char, String)],
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    for character in value.chars() {
        if let Some((_, replacement)) = character_map
            .iter()
            .find(|(candidate, _)| *candidate == character)
        {
            output.push_str(replacement)?;
        } else {
            escape_attribute_character(character, output)?;
        }
    }
    Ok(())
}

fn escape_text(
    value: &str,
    character_map: &[(char, String)],
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    for character in value.chars() {
        if let Some((_, replacement)) = character_map
            .iter()
            .find(|(candidate, _)| *candidate == character)
        {
            output.push_str(replacement)?;
            continue;
        }
        match character {
            '&' => output.push_str("&amp;")?,
            '<' => output.push_str("&lt;")?,
            '>' => output.push_str("&gt;")?,
            _ => output.push(character)?,
        }
    }
    Ok(())
}

struct BudgetedString<'a> {
    value: String,
    byte_limit: usize,
    request_id: String,
    control: &'a mut InvocationControl,
}

impl<'a> BudgetedString<'a> {
    fn new(byte_limit: usize, request_id: &str, control: &'a mut InvocationControl) -> Self {
        Self {
            value: String::new(),
            byte_limit,
            request_id: request_id.to_owned(),
            control,
        }
    }

    fn push_str(&mut self, value: &str) -> Result<(), ExecutionFailure> {
        self.control
            .charge(WorkDomain::SerializedByte, value.len())
            .map_err(|failure| control_failure(failure, &self.request_id))?;
        let attempted = self.value.len().checked_add(value.len()).ok_or_else(|| {
            failure(
                "FXSR0001",
                FailureCategory::Limit,
                Some(&self.request_id),
                "serialized result byte count overflowed",
            )
        })?;
        if attempted > self.byte_limit {
            return Err(failure(
                "FXSR0002",
                FailureCategory::Limit,
                Some(&self.request_id),
                format!(
                    "serialized result requires at least {attempted} bytes; limit is {}",
                    self.byte_limit
                ),
            ));
        }
        self.value.push_str(value);
        Ok(())
    }

    fn push(&mut self, character: char) -> Result<(), ExecutionFailure> {
        let mut encoded = [0_u8; 4];
        self.push_str(character.encode_utf8(&mut encoded))
    }

    fn finish(self) -> String {
        self.value
    }
}
