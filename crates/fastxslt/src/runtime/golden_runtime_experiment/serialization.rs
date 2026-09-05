//! Private serialization of the golden slice's semantic result.

#[cfg(test)]
use super::byte_encoding::{encode_us_ascii_cdata, serialize_utf16_be};
use super::{
    ExecutionFailure, FailureCategory, ResultAttribute, ResultNode, SemanticResult,
    control_failure, failure,
};
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xslt::golden_semantics_experiment::OutputSettings;
use unicode_normalization::UnicodeNormalization;

mod namespace_scope;

use namespace_scope::{
    NamespaceFrame, NamespaceMode, NamespaceScope, complete_element_prefix as element_prefix,
    complete_namespace_scope as element_namespace_scope,
};

#[derive(Clone, Copy)]
struct SerializationOptions<'a> {
    cdata_section_elements: &'a [crate::xml::quick_xml_experiment::ExpandedName],
    suppress_indentation_elements: &'a [crate::xml::quick_xml_experiment::ExpandedName],
    character_map: &'a [(char, String)],
    xhtml_mode: XhtmlMode,
    content_type_media_type: Option<&'a str>,
    html_mode: HtmlMode,
    escape_uri_attributes: bool,
    normalization_form: NormalizationForm,
    xml_empty_element_tag: bool,
    indent: bool,
    indentation_state: IndentationState,
}

impl SerializationOptions<'_> {
    fn inherited_for(self, name: &crate::xml::quick_xml_experiment::ExpandedName) -> Self {
        Self {
            indentation_state: if self.indentation_state == IndentationState::Suppressed
                || self.suppress_indentation_elements.contains(name)
            {
                IndentationState::Suppressed
            } else {
                IndentationState::Enabled
            },
            ..self
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum XhtmlMode {
    None,
    PreservePrefixes,
    DefaultNamespace,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HtmlMode {
    None,
    Legacy,
    Five,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NormalizationForm {
    None,
    Nfc,
    Nfd,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum IndentationState {
    Enabled,
    Suppressed,
}

pub(in crate::runtime) fn serialize_xml(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
    byte_limit: usize,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    serialize_xml_with_namespace_mode(
        result,
        settings,
        request_id,
        byte_limit,
        control,
        NamespaceMode::ScopedStack,
    )
}

fn serialize_xml_with_namespace_mode(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
    byte_limit: usize,
    control: &mut InvocationControl,
    namespace_mode: NamespaceMode,
) -> Result<String, ExecutionFailure> {
    validate_serialization_preconditions(result, settings, request_id)?;
    let normalization_form = validate_normalization_form(settings, request_id)?;
    validate_string_encoding(settings, request_id)?;
    validate_html_version(settings, request_id)?;
    validate_string_byte_order_mark(settings, request_id)?;
    let first_significant = result
        .children
        .iter()
        .find(|node| is_significant_result_node(node));
    if settings.method.is_none()
        && matches!(first_significant, Some(ResultNode::Element { name, .. })
            if name.namespace.is_none() && name.local.eq_ignore_ascii_case("html"))
    {
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
        return serialize_text_result(
            result,
            settings,
            normalization_form,
            request_id,
            byte_limit,
            control,
        );
    }
    let html = settings.method.as_deref() == Some("html");
    if html {
        validate_html_processing_instructions(result, request_id)?;
        validate_bounded_html_result(result, settings, request_id)?;
    }
    if settings
        .method
        .as_deref()
        .is_some_and(|method| !matches!(method, "xml" | "xhtml" | "html"))
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
    if !settings.omit_xml_declaration && !html {
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
    let content_type_media_type = ((xhtml || html) && settings.include_content_type != Some(false))
        .then(|| settings.media_type.as_deref().unwrap_or("text/html"));
    let xhtml_mode = if settings.html_version.as_deref() == Some("5") && (xhtml || html) {
        XhtmlMode::DefaultNamespace
    } else if xhtml {
        XhtmlMode::PreservePrefixes
    } else {
        XhtmlMode::None
    };
    let options = SerializationOptions {
        cdata_section_elements: &settings.cdata_section_elements,
        suppress_indentation_elements: &settings.suppress_indentation_elements,
        character_map: &settings.character_map,
        xhtml_mode,
        content_type_media_type,
        html_mode: select_html_mode(settings, html),
        escape_uri_attributes: (xhtml || html) && settings.escape_uri_attributes.unwrap_or(true),
        normalization_form,
        xml_empty_element_tag: settings.doctype_system.is_some() && !xhtml && !html,
        indent: settings.indent == Some(true),
        indentation_state: IndentationState::Enabled,
    };
    let mut namespace_scope =
        NamespaceScope::new(output_namespace_mode(namespace_mode, xhtml_mode));
    let mut doctype_written = false;
    for node in &result.children {
        if !doctype_written && matches!(node, ResultNode::Element { .. }) {
            serialize_doctype(result, settings, xhtml, &mut output)?;
            doctype_written = true;
        }
        serialize_node(node, &mut namespace_scope, options, 0, &mut output)?;
    }
    Ok(output.finish())
}

fn output_namespace_mode(requested: NamespaceMode, xhtml_mode: XhtmlMode) -> NamespaceMode {
    if xhtml_mode == XhtmlMode::DefaultNamespace {
        NamespaceMode::CompleteClone
    } else {
        requested
    }
}

#[cfg(test)]
pub(in crate::runtime) fn serialize_xml_complete_namespace_reference(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
    byte_limit: usize,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    serialize_xml_with_namespace_mode(
        result,
        settings,
        request_id,
        byte_limit,
        control,
        NamespaceMode::CompleteClone,
    )
}

fn is_significant_result_node(node: &ResultNode) -> bool {
    match node {
        ResultNode::Text(value) => !value.chars().all(char::is_whitespace),
        ResultNode::Element { .. } => true,
        ResultNode::PendingAttribute(_)
        | ResultNode::ProcessingInstruction { .. }
        | ResultNode::Comment(_) => false,
    }
}

fn serialize_text_result(
    result: &SemanticResult,
    settings: &OutputSettings,
    normalization_form: NormalizationForm,
    request_id: &str,
    byte_limit: usize,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let mut output = BudgetedString::new(byte_limit, request_id, control);
    for node in &result.children {
        serialize_text_node(
            node,
            &settings.character_map,
            normalization_form,
            &mut output,
        )?;
    }
    Ok(output.finish())
}

fn select_html_mode(settings: &OutputSettings, html: bool) -> HtmlMode {
    if !html {
        HtmlMode::None
    } else if settings
        .version
        .as_deref()
        .is_some_and(is_html_version_five)
    {
        HtmlMode::Five
    } else {
        HtmlMode::Legacy
    }
}

fn validate_bounded_html_result(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    if settings.html_version.as_deref() == Some("5") && is_bounded_html5_document(&result.children)
    {
        return Ok(());
    }
    if settings.html_version.as_deref() == Some("5")
        && is_bounded_html5_input_document(&result.children)
    {
        return Ok(());
    }
    if settings.html_version.as_deref() == Some("5")
        && is_bounded_html5_suppressed_paragraph_document(&result.children)
    {
        return Ok(());
    }
    if settings.html_version.as_deref() == Some("5")
        && !settings.character_map.is_empty()
        && is_bounded_html5_character_map_document(&result.children)
    {
        return Ok(());
    }
    if settings
        .version
        .as_deref()
        .is_some_and(is_html_version_five)
        && is_bounded_html5_control_character_document(&result.children)
    {
        return Ok(());
    }
    if is_bounded_html_content_type_document(&result.children) {
        return Ok(());
    }
    if is_bounded_html_manual_script_document(&result.children) {
        return Ok(());
    }
    if is_bounded_html_raw_text_whitespace_document(&result.children) {
        return Ok(());
    }
    if is_bounded_html_ins_del_document(&result.children) {
        return Ok(());
    }
    if settings.escape_uri_attributes.unwrap_or(true)
        && is_bounded_html_uri_document(&result.children)
    {
        return Ok(());
    }
    let significant: Vec<_> = result
        .children
        .iter()
        .filter(|node| {
            !matches!(node, ResultNode::Text(value) if value.chars().all(char::is_whitespace))
        })
        .collect();
    let [root] = significant.as_slice() else {
        return Err(unsupported_html_result(request_id));
    };
    if settings.character_map.is_empty() || !is_bounded_html_node(root, true) {
        return Err(unsupported_html_result(request_id));
    }
    Ok(())
}

fn is_bounded_html5_input_document(nodes: &[ResultNode]) -> bool {
    let significant: Vec<_> = nodes
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    matches!(significant.as_slice(), [ResultNode::Element {
        name,
        namespaces,
        attributes,
        children,
    }] if name.namespace.is_none()
        && name.local == "input"
        && namespaces.is_empty()
        && attributes.len() == 2
        && children.is_empty()
        && attributes.iter().any(|attribute| {
            attribute.name.namespace.is_none()
                && attribute.name.local == "type"
                && attribute.value == "text"
        })
        && attributes.iter().any(|attribute| {
            attribute.name.namespace.is_none()
                && attribute.name.local == "value"
                && attribute.value == "✈"
        }))
}

fn is_bounded_html5_suppressed_paragraph_document(nodes: &[ResultNode]) -> bool {
    let significant: Vec<_> = nodes
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [html] = significant.as_slice() else {
        return false;
    };
    let Some(html_children) = plain_html_children(html, "html") else {
        return false;
    };
    let html_elements: Vec<_> = html_children
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [head, body] = html_elements.as_slice() else {
        return false;
    };
    exact_text_container_children(head, "head", ["title"])
        && exact_text_container_children(body, "body", ["h1", "p"])
}

fn exact_text_container_children<const N: usize>(
    node: &ResultNode,
    local: &str,
    child_locals: [&str; N],
) -> bool {
    let Some(children) = plain_html_children(node, local) else {
        return false;
    };
    let elements: Vec<_> = children
        .iter()
        .filter(|child| !is_whitespace_text(child))
        .collect();
    elements.len() == N
        && elements.iter().zip(child_locals).all(|(child, local)| {
            plain_html_children(child, local)
                .is_some_and(|children| matches!(children, [ResultNode::Text(_)]))
        })
}

fn is_bounded_html_ins_del_document(nodes: &[ResultNode]) -> bool {
    let significant: Vec<_> = nodes
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [html] = significant.as_slice() else {
        return false;
    };
    let Some(html_children) = plain_html_children(html, "html") else {
        return false;
    };
    let html_elements: Vec<_> = html_children
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [head, body] = html_elements.as_slice() else {
        return false;
    };
    let Some(head_children) = plain_html_children(head, "head") else {
        return false;
    };
    if !head_children
        .iter()
        .all(|node| matches!(node, ResultNode::Text(_)))
    {
        return false;
    }
    let Some(body_children) = plain_html_children(body, "body") else {
        return false;
    };
    let body_elements: Vec<_> = body_children
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [paragraph] = body_elements.as_slice() else {
        return false;
    };
    let Some(paragraph_children) = plain_html_children(paragraph, "p") else {
        return false;
    };
    let embedded: Vec<_> = paragraph_children
        .iter()
        .filter(|node| matches!(node, ResultNode::Element { .. }))
        .collect();
    matches!(embedded.as_slice(), [deleted, inserted]
        if plain_html_children(deleted, "del")
            .is_some_and(|children| children.iter().all(|node| matches!(node, ResultNode::Text(_))))
            && plain_html_children(inserted, "ins")
                .is_some_and(|children| children.iter().all(|node| matches!(node, ResultNode::Text(_)))))
        && paragraph_children.iter().all(|node| {
            matches!(node, ResultNode::Text(_))
                || embedded.iter().any(|child| std::ptr::eq(*child, node))
        })
}

fn is_bounded_html_uri_document(nodes: &[ResultNode]) -> bool {
    let significant: Vec<_> = nodes
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [html] = significant.as_slice() else {
        return false;
    };
    let Some(html_children) = plain_html_children(html, "html") else {
        return false;
    };
    let significant_html: Vec<_> = html_children
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [body] = significant_html.as_slice() else {
        return false;
    };
    let Some(body_children) = plain_html_children(body, "body") else {
        return false;
    };
    let significant_body: Vec<_> = body_children
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [division] = significant_body.as_slice() else {
        return false;
    };
    let Some(division_children) = plain_html_children(division, "div") else {
        return false;
    };
    let links: Vec<_> = division_children
        .iter()
        .filter(|node| matches!(node, ResultNode::Element { .. }))
        .collect();
    matches!(links.as_slice(), [link] if is_bounded_html_uri_link(link))
        && division_children.iter().all(|node| {
            matches!(node, ResultNode::Text(_))
                || links.iter().any(|link| std::ptr::eq(*link, node))
        })
}

fn is_bounded_html_uri_link(node: &ResultNode) -> bool {
    let ResultNode::Element {
        name,
        namespaces,
        attributes,
        children,
    } = node
    else {
        return false;
    };
    name.namespace.is_none()
        && name.local == "a"
        && namespaces.is_empty()
        && matches!(attributes.as_slice(), [attribute]
            if attribute.name.namespace.is_none() && attribute.name.local == "href")
        && children
            .iter()
            .all(|child| matches!(child, ResultNode::Text(_)))
}

fn is_bounded_html_manual_script_document(nodes: &[ResultNode]) -> bool {
    let significant: Vec<_> = nodes
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [html] = significant.as_slice() else {
        return false;
    };
    let Some(html_children) = plain_html_children(html, "html") else {
        return false;
    };
    let html_elements: Vec<_> = html_children
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [head, body] = html_elements.as_slice() else {
        return false;
    };
    let Some(head_children) = plain_html_children(head, "head") else {
        return false;
    };
    let significant_head: Vec<_> = head_children
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [script] = significant_head.as_slice() else {
        return false;
    };
    is_bounded_manual_script(script)
        && plain_html_children(body, "body")
            .is_some_and(|children| children.iter().all(is_whitespace_text))
}

fn is_bounded_manual_script(node: &ResultNode) -> bool {
    let ResultNode::Element {
        name,
        namespaces,
        attributes,
        children,
    } = node
    else {
        return false;
    };
    name.namespace.is_none()
        && name.local == "script"
        && namespaces.is_empty()
        && matches!(attributes.as_slice(), [attribute]
            if attribute.name.namespace.is_none()
                && attribute.name.local == "type"
                && attribute.value == "text/javascript")
        && matches!(children.as_slice(), [ResultNode::Text(_)])
}

fn is_bounded_html_raw_text_whitespace_document(nodes: &[ResultNode]) -> bool {
    let significant: Vec<_> = nodes
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [html] = significant.as_slice() else {
        return false;
    };
    let Some(html_children) = plain_html_children(html, "html") else {
        return false;
    };
    let html_elements: Vec<_> = html_children
        .iter()
        .filter(|node| !is_whitespace_text(node))
        .collect();
    let [head, body] = html_elements.as_slice() else {
        return false;
    };
    is_bounded_raw_text_head(head) && is_bounded_pre_textarea_body(body)
}

fn is_bounded_raw_text_head(node: &ResultNode) -> bool {
    let Some(children) = plain_html_children(node, "head") else {
        return false;
    };
    let elements: Vec<_> = children
        .iter()
        .filter(|child| !is_whitespace_text(child))
        .collect();
    matches!(elements.as_slice(), [script, style]
        if is_bounded_manual_script(script)
            && plain_html_children(style, "style")
                .is_some_and(|children| matches!(children, [ResultNode::Text(_)])))
}

fn is_bounded_pre_textarea_body(node: &ResultNode) -> bool {
    let Some(children) = plain_html_children(node, "body") else {
        return false;
    };
    let elements: Vec<_> = children
        .iter()
        .filter(|child| !is_whitespace_text(child))
        .collect();
    matches!(elements.as_slice(), [pre, textarea]
        if is_bounded_pre(pre) && is_bounded_textarea(textarea))
}

fn is_bounded_pre(node: &ResultNode) -> bool {
    let Some(children) = plain_html_children(node, "pre") else {
        return false;
    };
    let elements: Vec<_> = children
        .iter()
        .filter(|child| matches!(child, ResultNode::Element { .. }))
        .collect();
    matches!(elements.as_slice(), [bold]
        if plain_html_children(bold, "b")
            .is_some_and(|children| matches!(children, [ResultNode::Text(_)])))
        && children.iter().all(|child| {
            matches!(child, ResultNode::Text(_))
                || elements.iter().any(|element| std::ptr::eq(*element, child))
        })
}

fn is_bounded_textarea(node: &ResultNode) -> bool {
    let ResultNode::Element {
        name,
        namespaces,
        attributes,
        children,
    } = node
    else {
        return false;
    };
    name.namespace.is_none()
        && name.local == "textarea"
        && namespaces.is_empty()
        && attributes.len() == 2
        && attributes.iter().any(|attribute| {
            attribute.name.namespace.is_none()
                && attribute.name.local == "rows"
                && attribute.value == "2"
        })
        && attributes.iter().any(|attribute| {
            attribute.name.namespace.is_none()
                && attribute.name.local == "cols"
                && attribute.value == "20"
        })
        && matches!(children.as_slice(), [ResultNode::Text(_)])
}

fn plain_html_children<'a>(node: &'a ResultNode, local: &str) -> Option<&'a [ResultNode]> {
    let ResultNode::Element {
        name,
        namespaces,
        attributes,
        children,
    } = node
    else {
        return None;
    };
    (name.namespace.is_none()
        && name.local == local
        && namespaces.is_empty()
        && attributes.is_empty())
    .then_some(children)
}

fn is_whitespace_text(node: &ResultNode) -> bool {
    matches!(node, ResultNode::Text(value) if value.chars().all(char::is_whitespace))
}

fn is_bounded_html_content_type_document(nodes: &[ResultNode]) -> bool {
    let significant: Vec<_> = nodes
        .iter()
        .filter(|node| {
            !matches!(node, ResultNode::Text(value) if value.chars().all(char::is_whitespace))
        })
        .collect();
    let [root] = significant.as_slice() else {
        return false;
    };
    is_bounded_html_content_type_node(root, true)
}

fn is_bounded_html_content_type_node(node: &ResultNode, root: bool) -> bool {
    let ResultNode::Element {
        name,
        namespaces,
        attributes,
        children,
    } = node
    else {
        return matches!(node, ResultNode::Text(_));
    };
    name.namespace.is_none()
        && namespaces.is_empty()
        && attributes.is_empty()
        && if root {
            name.local.eq_ignore_ascii_case("html") && children.iter().all(|child| {
                matches!(child, ResultNode::Text(value) if value.chars().all(char::is_whitespace))
                    || is_bounded_html_content_type_node(child, false)
            })
        } else {
            matches!(name.local.to_ascii_lowercase().as_str(), "head" | "body")
                && children.iter().all(|child| {
                    matches!(child, ResultNode::Text(_))
                        || (name.local.eq_ignore_ascii_case("head")
                            && is_bounded_existing_html_content_type_meta(child))
                })
        }
}

fn is_bounded_existing_html_content_type_meta(node: &ResultNode) -> bool {
    let ResultNode::Element {
        name,
        namespaces,
        attributes,
        children,
    } = node
    else {
        return false;
    };
    name.namespace.is_none()
        && name.local.eq_ignore_ascii_case("meta")
        && namespaces.is_empty()
        && children.is_empty()
        && attributes.len() == 2
        && attributes.iter().any(|attribute| {
            attribute.name.namespace.is_none()
                && attribute.name.local.eq_ignore_ascii_case("http-equiv")
                && attribute.value.eq_ignore_ascii_case("Content-Type")
        })
        && attributes.iter().any(|attribute| {
            attribute.name.namespace.is_none()
                && attribute.name.local.eq_ignore_ascii_case("content")
                && attribute.value.starts_with("text/html")
        })
}

fn is_bounded_html5_control_character_document(nodes: &[ResultNode]) -> bool {
    let significant: Vec<_> = nodes
        .iter()
        .filter(|node| {
            !matches!(node, ResultNode::Text(value) if value.chars().all(char::is_whitespace))
        })
        .collect();
    matches!(
        significant.as_slice(),
        [ResultNode::Element {
            name,
            namespaces,
            attributes,
            children,
        }] if name.namespace.is_none()
            && name.local == "doc"
            && namespaces.is_empty()
            && attributes.is_empty()
            && matches!(children.as_slice(), [ResultNode::Text(value)]
                if value.chars().any(is_c1_control))
    )
}

fn is_c1_control(character: char) -> bool {
    ('\u{7f}'..='\u{9f}').contains(&character)
}

fn is_bounded_html5_character_map_document(nodes: &[ResultNode]) -> bool {
    let significant: Vec<_> = nodes
        .iter()
        .filter(|node| {
            !matches!(node, ResultNode::Text(value) if value.chars().all(char::is_whitespace))
        })
        .collect();
    let [
        ResultNode::Element {
            name,
            namespaces,
            attributes,
            children,
        },
    ] = significant.as_slice()
    else {
        return false;
    };
    name.namespace.is_none()
        && name.local == "doc"
        && namespaces.is_empty()
        && attributes.is_empty()
        && matches!(
            children.as_slice(),
            [ResultNode::Element {
                name,
                namespaces,
                attributes,
                children,
            }] if name.namespace.is_none()
                && name.local == "a"
                && namespaces.is_empty()
                && matches!(attributes.as_slice(), [attribute]
                    if attribute.name.namespace.is_none()
                        && attribute.name.local == "value")
                && matches!(children.as_slice(), [ResultNode::Text(_)])
        )
}

fn is_bounded_html5_document(nodes: &[ResultNode]) -> bool {
    let elements: Vec<_> = nodes
        .iter()
        .filter(|node| matches!(node, ResultNode::Element { .. }))
        .collect();
    let [root] = elements.as_slice() else {
        return false;
    };
    nodes.iter().all(|node| match node {
        ResultNode::Text(value) => value.chars().all(char::is_whitespace),
        ResultNode::Comment(_) | ResultNode::ProcessingInstruction { .. } => true,
        ResultNode::PendingAttribute(_) => false,
        ResultNode::Element { .. } => std::ptr::eq(node, *root),
    }) && is_bounded_html5_node(root, true)
}

fn is_bounded_html5_node(node: &ResultNode, root: bool) -> bool {
    match node {
        ResultNode::Text(_) | ResultNode::Comment(_) => true,
        ResultNode::PendingAttribute(_) | ResultNode::ProcessingInstruction { .. } => false,
        ResultNode::Element {
            name,
            namespaces,
            attributes,
            children,
        } => {
            is_bounded_html5_element_name(name, root)
                && namespaces.iter().all(|binding| {
                    is_xhtml5_default_namespace(Some(&binding.namespace))
                        || matches!(binding.namespace.as_str(), "NamespaceN" | "NamespaceM")
                })
                && attributes.iter().all(is_bounded_html5_attribute)
                && children
                    .iter()
                    .all(|child| is_bounded_html5_node(child, false))
        }
    }
}

fn is_bounded_html5_attribute(attribute: &ResultAttribute) -> bool {
    match attribute.name.namespace.as_deref() {
        None => matches!(
            attribute.name.local.as_str(),
            "width" | "height" | "fill" | "cx" | "cy" | "r" | "z"
        ),
        Some("http://www.w3.org/2000/svg") => {
            matches!(attribute.name.local.as_str(), "att" | "atZZZ")
        }
        Some("http://www.w3.org/1998/Math/MathML") => attribute.name.local == "att",
        Some("NamespaceM") => attribute.name.local == "zzz",
        Some(_) => false,
    }
}

fn is_bounded_html5_element_name(
    name: &crate::xml::quick_xml_experiment::ExpandedName,
    root: bool,
) -> bool {
    if root {
        return name.namespace.is_none() && name.local == "html";
    }
    match name.namespace.as_deref() {
        None => {
            matches!(name.local.as_str(), "head" | "title" | "body" | "p")
                || is_html_void_element(name)
        }
        Some("http://www.w3.org/2000/svg") => {
            matches!(name.local.as_str(), "svg" | "rect" | "circle")
        }
        Some("http://www.w3.org/1998/Math/MathML") => {
            matches!(
                name.local.as_str(),
                "math" | "mrow" | "mi" | "msup" | "mn" | "mo"
            )
        }
        Some("NamespaceN") => name.local == "zzz",
        Some(_) => false,
    }
}

fn validate_string_byte_order_mark(
    settings: &OutputSettings,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    if settings.byte_order_mark == Some(true) {
        return Err(failure(
            "FXSR1005",
            FailureCategory::Unsupported,
            Some(request_id),
            "byte-order-mark=yes requires a byte serialization result lane",
        ));
    }
    Ok(())
}

fn validate_html_processing_instructions(
    result: &SemanticResult,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    fn contains_forbidden_data(node: &ResultNode) -> bool {
        match node {
            ResultNode::ProcessingInstruction { value, .. } => value.contains('>'),
            ResultNode::Element { children, .. } => children.iter().any(contains_forbidden_data),
            ResultNode::PendingAttribute(_) | ResultNode::Text(_) | ResultNode::Comment(_) => false,
        }
    }

    if result.children.iter().any(contains_forbidden_data) {
        return Err(failure(
            "SERE0015",
            FailureCategory::Invalid,
            Some(request_id),
            "HTML processing-instruction data must not contain >",
        ));
    }
    Ok(())
}

fn is_bounded_html_node(node: &ResultNode, root: bool) -> bool {
    match node {
        ResultNode::Text(_) => true,
        ResultNode::PendingAttribute(_)
        | ResultNode::ProcessingInstruction { .. }
        | ResultNode::Comment(_) => false,
        ResultNode::Element {
            name,
            namespaces,
            attributes,
            children,
        } => {
            name.namespace.is_none()
                && attributes.is_empty()
                && namespaces.is_empty()
                && if root {
                    name.local == "html"
                } else {
                    matches!(name.local.as_str(), "body" | "p")
                }
                && children
                    .iter()
                    .all(|child| is_bounded_html_node(child, false))
        }
    }
}

fn unsupported_html_result(request_id: &str) -> ExecutionFailure {
    failure(
        "FXSR1001",
        FailureCategory::Unsupported,
        Some(request_id),
        "HTML serialization is limited to the admitted bounded result shapes",
    )
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
) -> Result<NormalizationForm, ExecutionFailure> {
    match settings.normalization_form.as_deref().unwrap_or("none") {
        "none" => Ok(NormalizationForm::None),
        "NFC" => Ok(NormalizationForm::Nfc),
        "NFD" => Ok(NormalizationForm::Nfd),
        normalization_form => Err(failure(
            "SESU0011",
            FailureCategory::Unsupported,
            Some(request_id),
            format!("the requested normalization form is not supported: {normalization_form}"),
        )),
    }
}

fn validate_html_version(
    settings: &OutputSettings,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    if settings.method.as_deref() != Some("html") {
        return Ok(());
    }
    let Some(version) = settings.version.as_deref() else {
        return Ok(());
    };
    if is_html_version_five(version) {
        return Ok(());
    }
    Err(failure(
        "SESU0013",
        FailureCategory::Unsupported,
        Some(request_id),
        format!("the requested HTML output version is not supported: {version}"),
    ))
}

fn is_html_version_five(version: &str) -> bool {
    matches!(version.trim().trim_start_matches('+'), "5" | "5.0")
}

fn serialize_doctype(
    result: &SemanticResult,
    settings: &OutputSettings,
    xhtml: bool,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    let automatic_xhtml5 = xhtml && settings.html_version.as_deref() == Some("5");
    let html_doctype_required =
        settings.method.as_deref() == Some("html") && settings.html_version.as_deref() == Some("5");
    let system = settings
        .doctype_system
        .as_deref()
        .filter(|value| !value.is_empty());
    if system.is_none() && !automatic_xhtml5 && !html_doctype_required {
        return Ok(());
    }
    let document_element = result
        .children
        .iter()
        .find(|node| matches!(node, ResultNode::Element { .. }));
    let Some(ResultNode::Element {
        name, namespaces, ..
    }) = document_element
    else {
        unreachable!("DOCTYPE preconditions require one document element")
    };
    let is_xhtml_html = name.namespace.as_deref() == Some("http://www.w3.org/1999/xhtml")
        && name.local.eq_ignore_ascii_case("html");
    if html_doctype_required && system.is_none() {
        output.push_str("<!DOCTYPE html>")?;
        return Ok(());
    }
    if automatic_xhtml5 && system.is_none() {
        if is_xhtml_html {
            output.push_str("<!DOCTYPE ")?;
            output.push_str(&name.local)?;
            return output.push('>');
        }
        return Ok(());
    }
    if xhtml && !is_xhtml_html {
        return Err(failure(
            "FXSR1007",
            FailureCategory::Unsupported,
            Some(&output.request_id),
            "XHTML DOCTYPE serialization requires an XHTML html document element",
        ));
    }
    let (in_scope, _) = element_namespace_scope(name, namespaces, &[]);
    let prefix = element_prefix(name.namespace.as_deref(), &in_scope, output)?;
    output.push_str("<!DOCTYPE ")?;
    write_name(prefix, &name.local, output)?;
    if let Some(public) = settings.doctype_public.as_deref() {
        output.push_str(" PUBLIC ")?;
        serialize_external_identifier(public, output)?;
        output.push(' ')?;
    } else {
        output.push_str(" SYSTEM ")?;
    }
    serialize_external_identifier(system.expect("system identifier branch"), output)?;
    output.push('>')
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
        return serialize_utf8_bytes(result, settings, request_id, byte_limit, control);
    }
    if encoding.eq_ignore_ascii_case("UTF-16") {
        return serialize_utf16_be(result, settings, request_id, byte_limit, control);
    }
    serialize_single_byte_bytes(result, settings, request_id, byte_limit, control, encoding)
}

#[cfg(test)]
fn serialize_utf8_bytes(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
    byte_limit: usize,
    control: &mut InvocationControl,
) -> Result<Vec<u8>, ExecutionFailure> {
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
    Ok(bytes)
}

#[cfg(test)]
fn serialize_single_byte_bytes(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
    byte_limit: usize,
    control: &mut InvocationControl,
    encoding: &str,
) -> Result<Vec<u8>, ExecutionFailure> {
    let us_ascii = encoding.eq_ignore_ascii_case("US-ASCII");
    if !us_ascii && !encoding.eq_ignore_ascii_case("ISO-8859-1") {
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
            "the bounded single-byte encoding lane does not emit a byte-order mark",
        ));
    }

    let declaration = if settings.omit_xml_declaration
        || matches!(settings.method.as_deref(), Some("text" | "html"))
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
    let body_len = body.len();
    let encoded_body = if us_ascii {
        encode_us_ascii_cdata(&body, request_id)?
    } else if body.is_ascii() {
        body
    } else {
        return Err(failure(
            "FXSR1006",
            FailureCategory::Unsupported,
            Some(request_id),
            "the bounded ISO-8859-1 lane currently admits only ASCII result characters",
        ));
    };
    let expansion_bytes = encoded_body.len().saturating_sub(body_len);
    if expansion_bytes > 0 {
        control
            .charge(WorkDomain::SerializedByte, expansion_bytes)
            .map_err(|failure| control_failure(failure, request_id))?;
    }
    if encoded_body.len() > body_limit {
        return Err(failure(
            "FXSR0002",
            FailureCategory::Limit,
            Some(request_id),
            format!(
                "serialized result requires {} bytes; limit is {byte_limit}",
                declaration.len() + encoded_body.len()
            ),
        ));
    }

    let mut bytes = declaration.into_bytes();
    bytes.extend_from_slice(encoded_body.as_bytes());
    Ok(bytes)
}

fn validate_serialization_preconditions(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    if contains_pending_attribute(&result.children) {
        return Err(failure(
            "XTDE0410",
            FailureCategory::Invalid,
            Some(request_id),
            "a result attribute escaped its containing element construction",
        ));
    }
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

fn contains_pending_attribute(nodes: &[ResultNode]) -> bool {
    nodes.iter().any(|node| match node {
        ResultNode::PendingAttribute(_) => true,
        ResultNode::Element { children, .. } => contains_pending_attribute(children),
        ResultNode::Text(_) | ResultNode::ProcessingInstruction { .. } | ResultNode::Comment(_) => {
            false
        }
    })
}

fn serialize_text_node(
    node: &ResultNode,
    character_map: &[(char, String)],
    normalization_form: NormalizationForm,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    match node {
        ResultNode::Text(value) => {
            write_character_mapped(value, character_map, normalization_form, output)
        }
        ResultNode::PendingAttribute(_)
        | ResultNode::ProcessingInstruction { .. }
        | ResultNode::Comment(_) => Ok(()),
        ResultNode::Element { children, .. } => {
            for child in children {
                serialize_text_node(child, character_map, normalization_form, output)?;
            }
            Ok(())
        }
    }
}

fn write_character_mapped(
    value: &str,
    character_map: &[(char, String)],
    normalization_form: NormalizationForm,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    write_character_expansion(
        value,
        character_map,
        normalization_form,
        |character, output| output.push(character),
        output,
    )
}

fn serialize_node<'a>(
    node: &'a ResultNode,
    namespace_scope: &mut NamespaceScope<'a>,
    options: SerializationOptions<'_>,
    depth: usize,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    match node {
        ResultNode::Text(value) => {
            escape_text(
                value,
                options.character_map,
                options.normalization_form,
                options.html_mode == HtmlMode::Five,
                output,
            )?;
        }
        ResultNode::ProcessingInstruction { target, value } => {
            serialize_processing_instruction(target, value, output)?;
        }
        ResultNode::Comment(value) => {
            output.push_str("<!--")?;
            output.push_str(value)?;
            output.push_str("-->")?;
        }
        ResultNode::Element { .. } => {
            serialize_element(node, namespace_scope, options, depth, output)?;
        }
        ResultNode::PendingAttribute(_) => {
            return Err(failure(
                "XTDE0410",
                FailureCategory::Invalid,
                None,
                "a result attribute escaped its containing element construction",
            ));
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

fn serialize_element<'a>(
    node: &'a ResultNode,
    namespace_scope: &mut NamespaceScope<'a>,
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
    let frame = enter_element_namespace_scope(
        namespace_scope,
        name,
        namespaces,
        attributes,
        options.xhtml_mode,
    );
    let result = (|| {
        let prefix = namespace_scope.element_prefix(name.namespace.as_deref(), output)?;
        output.push('<')?;
        write_name(prefix.as_deref(), &name.local, output)?;
        namespace_scope.write_declarations(&frame, output)?;
        for attribute in attributes {
            output.push(' ')?;
            let prefix =
                namespace_scope.attribute_prefix(attribute.name.namespace.as_deref(), output)?;
            write_name(prefix.as_deref(), &attribute.name.local, output)?;
            output.push_str("=\"")?;
            if options.escape_uri_attributes && is_uri_attribute(name, attribute) {
                escape_uri_attribute(&attribute.value, output)?;
            } else {
                escape_attribute_with_character_map(
                    &attribute.value,
                    options.character_map,
                    options.normalization_form,
                    output,
                )?;
            }
            output.push('"')?;
        }
        if options.xml_empty_element_tag && children.is_empty() {
            return output.push_str("/>");
        }
        if options.html_mode != HtmlMode::None && children.is_empty() && is_html_void_element(name)
        {
            return output.push('>');
        }
        if options.xhtml_mode != XhtmlMode::None
            && children.is_empty()
            && is_xhtml_void_element(name, options.xhtml_mode)
        {
            return output.push_str(" />");
        }
        output.push('>')?;
        let inject_content_type = options
            .content_type_media_type
            .is_some_and(|_| is_content_type_head(name, options));
        let options = options.inherited_for(name);
        let indent_children = options.indent
            && options.indentation_state == IndentationState::Enabled
            && (inject_content_type
                || children
                    .iter()
                    .any(|child| !is_replaced_content_type_meta(child, inject_content_type)))
            && children
                .iter()
                .filter(|child| !is_replaced_content_type_meta(child, inject_content_type))
                .all(|child| matches!(child, ResultNode::Element { .. }));
        serialize_content_type_if_needed(
            options,
            inject_content_type,
            indent_children,
            depth,
            output,
        )?;
        for child in children {
            if is_replaced_content_type_meta(child, inject_content_type) {
                continue;
            }
            serialize_element_child(
                child,
                name,
                namespace_scope,
                options,
                depth,
                indent_children,
                output,
            )?;
        }
        if indent_children {
            write_indentation(depth, output)?;
        }
        output.push_str("</")?;
        write_name(prefix.as_deref(), &name.local, output)?;
        output.push('>')
    })();
    namespace_scope.exit(frame);
    result
}

fn enter_element_namespace_scope<'a>(
    namespace_scope: &mut NamespaceScope<'a>,
    name: &crate::xml::quick_xml_experiment::ExpandedName,
    namespaces: &'a [crate::xml::quick_xml_experiment::NamespaceBinding],
    attributes: &[ResultAttribute],
    xhtml_mode: XhtmlMode,
) -> NamespaceFrame {
    if xhtml_mode == XhtmlMode::DefaultNamespace {
        let default_namespace = name
            .namespace
            .as_deref()
            .filter(|namespace| is_xhtml5_default_namespace(Some(namespace)));
        let normalized =
            normalize_xhtml5_namespace_bindings(default_namespace, namespaces, attributes);
        namespace_scope.enter_transient(name, &normalized)
    } else {
        namespace_scope.enter_borrowed(name, namespaces)
    }
}

fn serialize_element_child<'a>(
    child: &'a ResultNode,
    parent_name: &crate::xml::quick_xml_experiment::ExpandedName,
    namespace_scope: &mut NamespaceScope<'a>,
    options: SerializationOptions<'_>,
    depth: usize,
    indent: bool,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    if indent {
        write_indentation(depth + 1, output)?;
    }
    if options.cdata_section_elements.contains(parent_name)
        && let ResultNode::Text(value) = child
    {
        return serialize_cdata(value, options.normalization_form, output);
    }
    if options.html_mode != HtmlMode::None
        && is_html_raw_text_element(parent_name)
        && let ResultNode::Text(value) = child
    {
        return write_character_mapped(
            value,
            options.character_map,
            options.normalization_form,
            output,
        );
    }
    serialize_node(child, namespace_scope, options, depth + 1, output)
}

fn serialize_content_type_if_needed(
    options: SerializationOptions<'_>,
    inject: bool,
    indent: bool,
    depth: usize,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    let Some(media_type) = options.content_type_media_type.filter(|_| inject) else {
        return Ok(());
    };
    if indent {
        write_indentation(depth + 1, output)?;
    }
    serialize_content_type_meta(media_type, options.html_mode == HtmlMode::None, output)
}

fn is_html_void_element(name: &crate::xml::quick_xml_experiment::ExpandedName) -> bool {
    name.namespace.is_none()
        && matches!(
            name.local.as_str(),
            "area"
                | "base"
                | "br"
                | "col"
                | "command"
                | "embed"
                | "hr"
                | "img"
                | "input"
                | "keygen"
                | "link"
                | "meta"
                | "param"
                | "source"
                | "track"
                | "wbr"
        )
}

fn is_html_raw_text_element(name: &crate::xml::quick_xml_experiment::ExpandedName) -> bool {
    name.namespace.is_none() && matches!(name.local.as_str(), "script" | "style")
}

fn is_xhtml5_default_namespace(namespace: Option<&str>) -> bool {
    matches!(
        namespace,
        Some(
            "http://www.w3.org/1999/xhtml"
                | "http://www.w3.org/2000/svg"
                | "http://www.w3.org/1998/Math/MathML"
        )
    )
}

fn normalize_xhtml5_namespace_bindings(
    default_namespace: Option<&str>,
    namespaces: &[crate::xml::quick_xml_experiment::NamespaceBinding],
    attributes: &[ResultAttribute],
) -> Vec<crate::xml::quick_xml_experiment::NamespaceBinding> {
    let mut normalized = namespaces
        .iter()
        .filter(|binding| {
            !is_xhtml5_default_namespace(Some(&binding.namespace))
                || binding.prefix.is_some()
                    && attributes.iter().any(|attribute| {
                        attribute.name.namespace.as_deref() == Some(&binding.namespace)
                    })
        })
        .cloned()
        .collect::<Vec<_>>();
    if let Some(default_namespace) = default_namespace {
        normalized.push(crate::xml::quick_xml_experiment::NamespaceBinding {
            prefix: None,
            namespace: default_namespace.to_owned(),
        });
    }
    normalized
}

fn is_xhtml_void_element(
    name: &crate::xml::quick_xml_experiment::ExpandedName,
    mode: XhtmlMode,
) -> bool {
    (name.namespace.as_deref() == Some("http://www.w3.org/1999/xhtml")
        || (mode == XhtmlMode::DefaultNamespace && name.namespace.is_none()))
        && matches!(
            name.local.as_str(),
            "area"
                | "base"
                | "basefont"
                | "br"
                | "col"
                | "embed"
                | "frame"
                | "hr"
                | "img"
                | "input"
                | "isindex"
                | "link"
                | "meta"
                | "param"
                | "source"
                | "track"
                | "wbr"
        )
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
            } if matches!(name.namespace.as_deref(), None | Some("http://www.w3.org/1999/xhtml"))
                && name.local.eq_ignore_ascii_case("meta")
                && attributes.iter().any(|attribute| {
                    attribute.name.namespace.is_none()
                        && attribute.name.local.eq_ignore_ascii_case("http-equiv")
                        && attribute.value.eq_ignore_ascii_case("Content-Type")
                })
        )
}

fn is_content_type_head(
    name: &crate::xml::quick_xml_experiment::ExpandedName,
    options: SerializationOptions<'_>,
) -> bool {
    if options.html_mode == HtmlMode::None {
        is_xhtml_head(name)
    } else {
        name.namespace.is_none() && name.local.eq_ignore_ascii_case("head")
    }
}

fn serialize_content_type_meta(
    media_type: &str,
    xhtml: bool,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    output.push_str("<meta http-equiv=\"Content-Type\" content=\"")?;
    escape_attribute(media_type, output)?;
    output.push_str(if xhtml {
        "; charset=UTF-8\" />"
    } else {
        "; charset=UTF-8\">"
    })
}

fn write_indentation(depth: usize, output: &mut BudgetedString) -> Result<(), ExecutionFailure> {
    output.push('\n')?;
    for _ in 0..depth {
        output.push_str("  ")?;
    }
    Ok(())
}

fn serialize_cdata(
    value: &str,
    normalization_form: NormalizationForm,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    output.push_str("<![CDATA[")?;
    let normalized = normalize_to_string(value, normalization_form);
    output.push_str(&normalized.replace("]]>", "]]]]><![CDATA[>"))?;
    output.push_str("]]>")
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
        '>' => output.push_str("&gt;"),
        '"' => output.push_str("&quot;"),
        '\t' => output.push_str("&#x9;"),
        '\n' => output.push_str("&#xA;"),
        '\r' => output.push_str("&#xD;"),
        _ if is_c1_control(character) => {
            output.push_str(&format!("&#x{:X};", u32::from(character)))
        }
        _ => output.push(character),
    }
}

fn escape_attribute_with_character_map(
    value: &str,
    character_map: &[(char, String)],
    normalization_form: NormalizationForm,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    write_character_expansion(
        value,
        character_map,
        normalization_form,
        escape_attribute_character,
        output,
    )
}

fn is_uri_attribute(
    element: &crate::xml::quick_xml_experiment::ExpandedName,
    attribute: &ResultAttribute,
) -> bool {
    attribute.name.namespace.is_none()
        && attribute.name.local == "href"
        && matches!(
            element.namespace.as_deref(),
            None | Some("http://www.w3.org/1999/xhtml")
        )
}

fn escape_uri_attribute(value: &str, output: &mut BudgetedString) -> Result<(), ExecutionFailure> {
    for character in value.nfc() {
        if character.is_ascii() {
            escape_attribute_character(character, output)?;
        } else {
            let mut encoded = [0_u8; 4];
            for byte in character.encode_utf8(&mut encoded).as_bytes() {
                output.push_str(&format!("%{byte:02X}"))?;
            }
        }
    }
    Ok(())
}

fn escape_text(
    value: &str,
    character_map: &[(char, String)],
    normalization_form: NormalizationForm,
    html5: bool,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    if character_map.is_empty() && normalization_form == NormalizationForm::None {
        return escape_text_in_bounded_runs(value, html5, output);
    }
    write_character_expansion(
        value,
        character_map,
        normalization_form,
        |character, output| {
            match character {
                '&' => output.push_str("&amp;")?,
                '<' => output.push_str("&lt;")?,
                '>' => output.push_str("&gt;")?,
                _ if html5 && is_c1_control(character) => {
                    output.push_str(&format!("&#x{:X};", u32::from(character)))?;
                }
                _ => output.push(character)?,
            }
            Ok(())
        },
        output,
    )
}

const MAX_SAFE_TEXT_RUN_BYTES: usize = 4_096;

fn escape_text_in_bounded_runs(
    value: &str,
    html5: bool,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    let mut run_start = 0;
    for (index, character) in value.char_indices() {
        let replacement = match character {
            '&' => Some("&amp;"),
            '<' => Some("&lt;"),
            '>' => Some("&gt;"),
            _ if html5 && is_c1_control(character) => None,
            _ => {
                let prospective_end = index + character.len_utf8();
                if prospective_end.saturating_sub(run_start) > MAX_SAFE_TEXT_RUN_BYTES
                    && run_start < index
                {
                    output.push_safe_run(&value[run_start..index])?;
                    run_start = index;
                }
                continue;
            }
        };
        if run_start < index {
            output.push_safe_run(&value[run_start..index])?;
        }
        if let Some(replacement) = replacement {
            output.push_str(replacement)?;
        } else {
            output.push_str(&format!("&#x{:X};", u32::from(character)))?;
        }
        run_start = index + character.len_utf8();
    }
    if run_start < value.len() {
        output.push_safe_run(&value[run_start..])?;
    }
    Ok(())
}

fn write_character_expansion(
    value: &str,
    character_map: &[(char, String)],
    normalization_form: NormalizationForm,
    mut write_character: impl FnMut(char, &mut BudgetedString) -> Result<(), ExecutionFailure>,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    if normalization_form == NormalizationForm::None {
        for character in value.chars() {
            if let Some(replacement) = character_map_replacement(character_map, character) {
                output.push_str(replacement)?;
            } else {
                write_character(character, output)?;
            }
        }
        return Ok(());
    }

    let mut unmapped = String::new();
    for character in value.chars() {
        if let Some(replacement) = character_map_replacement(character_map, character) {
            write_normalized_characters(
                &unmapped,
                normalization_form,
                &mut write_character,
                output,
            )?;
            unmapped.clear();
            output.push_str(replacement)?;
        } else {
            unmapped.push(character);
        }
    }
    write_normalized_characters(&unmapped, normalization_form, &mut write_character, output)
}

fn character_map_replacement(character_map: &[(char, String)], character: char) -> Option<&str> {
    character_map
        .binary_search_by_key(&character, |(candidate, _)| *candidate)
        .ok()
        .map(|index| character_map[index].1.as_str())
}

fn write_normalized_characters(
    value: &str,
    normalization_form: NormalizationForm,
    write_character: &mut impl FnMut(char, &mut BudgetedString) -> Result<(), ExecutionFailure>,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    match normalization_form {
        NormalizationForm::None => value
            .chars()
            .try_for_each(|character| write_character(character, output)),
        NormalizationForm::Nfc => value
            .nfc()
            .try_for_each(|character| write_character(character, output)),
        NormalizationForm::Nfd => value
            .nfd()
            .try_for_each(|character| write_character(character, output)),
    }
}

fn normalize_to_string(value: &str, normalization_form: NormalizationForm) -> String {
    match normalization_form {
        NormalizationForm::None => value.to_owned(),
        NormalizationForm::Nfc => value.nfc().collect(),
        NormalizationForm::Nfd => value.nfd().collect(),
    }
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

    fn push_safe_run(&mut self, value: &str) -> Result<(), ExecutionFailure> {
        let remaining = self.byte_limit.saturating_sub(self.value.len());
        if value.len() <= remaining {
            return self.push_str(value);
        }

        let split = value
            .char_indices()
            .map(|(index, _)| index)
            .take_while(|index| *index <= remaining)
            .last()
            .unwrap_or(0);
        if split > 0 {
            self.push_str(&value[..split])?;
        }
        let first_unwritten = value[split..]
            .chars()
            .next()
            .expect("overflowing safe run must retain one character");
        self.push(first_unwritten)
    }

    fn finish(self) -> String {
        self.value
    }
}

#[cfg(test)]
mod scaling_measurement_tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::execution_control_experiment::{InvocationControl, WorkDomain};
    use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
    use crate::xslt::golden_semantics_experiment::OutputSettings;

    use super::{
        BudgetedString, NamespaceMode, NormalizationForm, ResultAttribute, ResultNode,
        SemanticResult, escape_text_in_bounded_runs, serialize_xml_with_namespace_mode,
        write_character_expansion,
    };

    fn xml_settings() -> OutputSettings {
        OutputSettings {
            method: Some("xml".to_owned()),
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
            omit_xml_declaration: true,
            indent: None,
        }
    }

    fn namespace_binding(prefix: Option<&str>, namespace: &str) -> NamespaceBinding {
        NamespaceBinding {
            prefix: prefix.map(str::to_owned),
            namespace: namespace.to_owned(),
        }
    }

    fn expanded_name(namespace: Option<&str>, local: &str) -> ExpandedName {
        ExpandedName {
            namespace: namespace.map(str::to_owned),
            local: local.to_owned(),
        }
    }

    fn namespace_scope_result() -> SemanticResult {
        let unnamespaced_grandchild = ResultNode::Element {
            name: expanded_name(None, "plain"),
            namespaces: Vec::new(),
            attributes: Vec::new(),
            children: Vec::new(),
        };
        let shadowing_child = ResultNode::Element {
            name: expanded_name(Some("urn:shadow"), "child"),
            namespaces: vec![
                namespace_binding(Some("q"), "urn:shared"),
                namespace_binding(Some("p"), "urn:shadow"),
            ],
            attributes: vec![ResultAttribute {
                name: expanded_name(Some("urn:shadow"), "value"),
                value: "one".to_owned(),
            }],
            children: vec![unnamespaced_grandchild],
        };
        let restored_sibling = ResultNode::Element {
            name: expanded_name(Some("urn:shared"), "sibling"),
            namespaces: Vec::new(),
            attributes: vec![ResultAttribute {
                name: expanded_name(Some("urn:shared"), "value"),
                value: "two".to_owned(),
            }],
            children: Vec::new(),
        };
        SemanticResult {
            children: vec![ResultNode::Element {
                name: expanded_name(Some("urn:shared"), "root"),
                namespaces: vec![
                    namespace_binding(Some("p"), "urn:shared"),
                    namespace_binding(Some("q"), "urn:shared"),
                    namespace_binding(None, "urn:default"),
                ],
                attributes: vec![ResultAttribute {
                    name: expanded_name(Some("urn:shared"), "value"),
                    value: "root".to_owned(),
                }],
                children: vec![shadowing_child, restored_sibling],
            }],
        }
    }

    #[test]
    fn scoped_namespace_stack_matches_complete_clone_for_shadowing_and_siblings() {
        let result = namespace_scope_result();
        let settings = xml_settings();
        let mut scoped_control = InvocationControl::unbounded();
        let scoped = serialize_xml_with_namespace_mode(
            &result,
            &settings,
            "namespace-scope",
            4_096,
            &mut scoped_control,
            NamespaceMode::ScopedStack,
        )
        .expect("scoped namespace serialization");
        let mut complete_control = InvocationControl::unbounded();
        let complete = serialize_xml_with_namespace_mode(
            &result,
            &settings,
            "namespace-scope",
            4_096,
            &mut complete_control,
            NamespaceMode::CompleteClone,
        )
        .expect("complete namespace serialization");

        assert_eq!(scoped, complete);
        assert_eq!(
            scoped,
            "<p:root xmlns:p=\"urn:shared\" xmlns:q=\"urn:shared\" xmlns=\"urn:default\" p:value=\"root\"><p:child xmlns:p=\"urn:shadow\" p:value=\"one\"><plain xmlns=\"\"></plain></p:child><p:sibling p:value=\"two\"></p:sibling></p:root>"
        );
        assert_eq!(
            scoped_control.consumed(WorkDomain::SerializedByte),
            complete_control.consumed(WorkDomain::SerializedByte)
        );
    }

    #[test]
    fn scoped_namespace_stack_matches_complete_clone_at_the_byte_limit() {
        let result = namespace_scope_result();
        let settings = xml_settings();
        let mut scoped_control = InvocationControl::unbounded();
        let scoped = serialize_xml_with_namespace_mode(
            &result,
            &settings,
            "namespace-limit",
            71,
            &mut scoped_control,
            NamespaceMode::ScopedStack,
        )
        .expect_err("scoped serialization should exceed the byte limit");
        let mut complete_control = InvocationControl::unbounded();
        let complete = serialize_xml_with_namespace_mode(
            &result,
            &settings,
            "namespace-limit",
            71,
            &mut complete_control,
            NamespaceMode::CompleteClone,
        )
        .expect_err("complete serialization should exceed the byte limit");

        assert_eq!(scoped, complete);
        assert_eq!(
            scoped_control.consumed(WorkDomain::SerializedByte),
            complete_control.consumed(WorkDomain::SerializedByte)
        );
    }

    #[test]
    fn scoped_namespace_stack_matches_complete_clone_at_cancellation() {
        let result = namespace_scope_result();
        let settings = xml_settings();
        let mut scoped_control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::SerializedByte, 12);
        let scoped = serialize_xml_with_namespace_mode(
            &result,
            &settings,
            "namespace-cancel",
            4_096,
            &mut scoped_control,
            NamespaceMode::ScopedStack,
        )
        .expect_err("scoped serialization should observe cancellation");
        let mut complete_control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::SerializedByte, 12);
        let complete = serialize_xml_with_namespace_mode(
            &result,
            &settings,
            "namespace-cancel",
            4_096,
            &mut complete_control,
            NamespaceMode::CompleteClone,
        )
        .expect_err("complete serialization should observe cancellation");

        assert_eq!(scoped, complete);
        assert_eq!(
            scoped_control.consumed(WorkDomain::SerializedByte),
            complete_control.consumed(WorkDomain::SerializedByte)
        );
    }

    #[test]
    fn bounded_text_runs_match_character_wise_xml_and_html_output() {
        let input = format!("{}<&>🚀\u{85}tail", "a".repeat(4_097));
        for html5 in [false, true] {
            let mut optimized_control = InvocationControl::unbounded();
            let mut optimized =
                BudgetedString::new(input.len() * 2, "bounded-text-runs", &mut optimized_control);
            escape_text_in_bounded_runs(&input, html5, &mut optimized)
                .expect("bounded run serialization");

            let mut reference_control = InvocationControl::unbounded();
            let mut reference = BudgetedString::new(
                input.len() * 2,
                "character-reference",
                &mut reference_control,
            );
            write_character_expansion(
                &input,
                &[],
                NormalizationForm::None,
                |character, output| match character {
                    '&' => output.push_str("&amp;"),
                    '<' => output.push_str("&lt;"),
                    '>' => output.push_str("&gt;"),
                    _ if html5 && super::is_c1_control(character) => {
                        output.push_str(&format!("&#x{:X};", u32::from(character)))
                    }
                    _ => output.push(character),
                },
                &mut reference,
            )
            .expect("character reference serialization");
            assert_eq!(optimized.finish(), reference.finish());
        }
    }

    #[test]
    fn bounded_text_runs_observe_cancellation_before_the_next_chunk_mutation() {
        let input = "a".repeat(12_288);
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::SerializedByte, 1);
        let mut output = BudgetedString::new(input.len(), "bounded-cancel", &mut control);
        let failure = escape_text_in_bounded_runs(&input, false, &mut output)
            .expect_err("second bounded write should observe cancellation");
        assert_eq!(failure.code, "FXCT0001");
        assert_eq!(failure.work_domain, Some(WorkDomain::SerializedByte));
        let written = output.value.len();
        drop(output);
        assert_eq!(written, 4_096);
        assert_eq!(control.consumed(WorkDomain::SerializedByte), 4_096);
    }

    #[test]
    fn bounded_text_runs_preserve_character_wise_byte_limit_failure() {
        let input = format!("{}🚀tail", "a".repeat(4_097));
        let byte_limit = 4_099;
        let mut optimized_control = InvocationControl::unbounded();
        let mut optimized =
            BudgetedString::new(byte_limit, "bounded-limit", &mut optimized_control);
        let optimized_failure = escape_text_in_bounded_runs(&input, false, &mut optimized)
            .expect_err("rocket should cross the byte limit");
        let optimized_value = optimized.value.clone();
        drop(optimized);

        let mut reference_control = InvocationControl::unbounded();
        let mut reference =
            BudgetedString::new(byte_limit, "bounded-limit", &mut reference_control);
        let reference_failure = write_character_expansion(
            &input,
            &[],
            NormalizationForm::None,
            |character, output| output.push(character),
            &mut reference,
        )
        .expect_err("reference rocket should cross the byte limit");
        let reference_value = reference.value.clone();
        drop(reference);

        assert_eq!(optimized_failure, reference_failure);
        assert_eq!(optimized_value, reference_value);
        assert_eq!(
            optimized_control.consumed(WorkDomain::SerializedByte),
            reference_control.consumed(WorkDomain::SerializedByte)
        );
    }

    #[test]
    #[ignore = "manual release-mode character-map serialization scaling measurement"]
    fn measure_absent_character_map_lookup_scaling() {
        const INPUT_CHARACTERS: usize = 10_000;
        let input = "z".repeat(INPUT_CHARACTERS);
        for entry_count in [100_usize, 1_000, 5_000, 10_000] {
            let character_map = (0..entry_count)
                .map(|offset| {
                    let scalar = u32::try_from(offset).expect("measurement size fits u32") + 0x1000;
                    (
                        char::from_u32(scalar).expect("measurement scalar should be valid"),
                        format!("replacement-{offset}"),
                    )
                })
                .collect::<Vec<_>>();
            let mut control = InvocationControl::unbounded();
            let mut output =
                BudgetedString::new(INPUT_CHARACTERS, "character-map-scale", &mut control);
            let started = Instant::now();
            write_character_expansion(
                black_box(&input),
                black_box(&character_map),
                NormalizationForm::None,
                |character, output| output.push(character),
                &mut output,
            )
            .expect("measurement serialization should succeed");
            let elapsed = started.elapsed();
            assert_eq!(output.finish(), input);
            eprintln!(
                "character-map-serialize entries={entry_count} input_chars={INPUT_CHARACTERS} elapsed_us={}",
                elapsed.as_micros()
            );
        }
    }
}
