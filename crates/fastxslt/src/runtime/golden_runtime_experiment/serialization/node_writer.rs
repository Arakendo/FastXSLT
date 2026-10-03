//! Private result-node traversal and element emission.

use super::{
    BudgetedString, ExecutionFailure, FailureCategory, HtmlMode, HtmlRawTextContext,
    IndentationState, NamespaceFrame, NamespaceScope, ResultAttribute, ResultNode,
    SerializationOptions, XhtmlMode, escape_attribute_with_character_map,
    escape_html_attribute_with_character_map, escape_legacy_html_uri_attribute, escape_text,
    escape_uri_attribute, failure, is_content_type_head, is_html_raw_text_element,
    is_html_void_element, is_minimized_html_boolean_attribute, is_replaced_content_type_meta,
    is_uri_attribute, is_xhtml_void_element, is_xhtml5_default_namespace,
    normalize_xhtml5_namespace_bindings, serialize_cdata, serialize_content_type_if_needed,
    write_character_mapped, write_indentation, write_name,
};

#[cfg(test)]
mod recursive_reference;
#[cfg(test)]
mod tests;

struct ElementCursor<'a, 'o> {
    name: &'a crate::xml::quick_xml_experiment::ExpandedName,
    children: std::slice::Iter<'a, ResultNode>,
    namespace_frame: NamespaceFrame,
    prefix: Option<String>,
    options: SerializationOptions<'o>,
    depth: usize,
    inject_content_type: bool,
    indent_children: bool,
    html_void: bool,
}

/// Keep only active ancestors, not a pending work item for every descendant.
/// The result already owns the tree; cursors borrow it and restore namespace
/// scopes in reverse order on both ordinary completion and any output failure.
pub(super) fn serialize_node<'a>(
    node: &'a ResultNode,
    namespace_scope: &mut NamespaceScope<'a>,
    options: SerializationOptions<'_>,
    depth: usize,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    let mut ancestors = Vec::new();
    let result = (|| {
        let mut next = Some((node, options, depth));
        loop {
            if let Some((node, options, depth)) = next.take() {
                if matches!(node, ResultNode::Element { .. }) {
                    if let Some(cursor) =
                        begin_element(node, namespace_scope, options, depth, output)?
                    {
                        ancestors.push(cursor);
                    }
                } else {
                    serialize_leaf(node, options, output)?;
                }
            }
            loop {
                let Some(cursor) = ancestors.last_mut() else {
                    return Ok(());
                };
                let child = cursor.children.find(|child| {
                    !is_replaced_content_type_meta(child, cursor.inject_content_type)
                });
                if let Some(child) = child {
                    if !serialize_child_prefix(
                        child,
                        cursor.name,
                        cursor.options,
                        cursor.depth,
                        cursor.indent_children,
                        output,
                    )? {
                        next = Some((child, cursor.options, cursor.depth + 1));
                        break;
                    }
                } else {
                    let cursor = ancestors.pop().expect("active cursor");
                    let finished = finish_element(&cursor, output);
                    namespace_scope.exit(cursor.namespace_frame);
                    finished?;
                }
            }
        }
    })();
    while let Some(cursor) = ancestors.pop() {
        namespace_scope.exit(cursor.namespace_frame);
    }
    result
}

fn serialize_leaf(
    node: &ResultNode,
    options: SerializationOptions<'_>,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    match node {
        ResultNode::Text(value) => {
            if options.html_raw_text_context == HtmlRawTextContext::Active {
                write_character_mapped(
                    value,
                    options.character_map,
                    options.normalization_form,
                    output,
                )?;
            } else {
                escape_text(
                    value,
                    options.character_map,
                    options.normalization_form,
                    options.html_mode == HtmlMode::Five,
                    output,
                )?;
            }
        }
        ResultNode::Xslt10DisableOutputEscapingText(value) => {
            write_character_mapped(
                value,
                options.character_map,
                options.normalization_form,
                output,
            )?;
        }
        ResultNode::ProcessingInstruction { target, value } => {
            serialize_processing_instruction(
                target,
                value,
                options.html_mode != HtmlMode::None,
                output,
            )?;
        }
        ResultNode::Comment(value) => {
            output.push_str("<!--")?;
            output.push_str(value)?;
            output.push_str("-->")?;
        }
        ResultNode::Element { .. } => {
            unreachable!("elements are handled by the traversal cursor");
        }
        ResultNode::PendingNamespace { .. } => {
            return Err(failure(
                "XTDE0420",
                FailureCategory::Invalid,
                None,
                "a result namespace escaped its containing element construction",
            ));
        }
        ResultNode::PendingAttribute(_) => {
            return Err(failure(
                "XTDE0410",
                FailureCategory::Invalid,
                None,
                "a result attribute escaped its containing element construction",
            ));
        }
        ResultNode::Xslt10RecoverableAttribute(_) => {}
    }
    Ok(())
}

fn serialize_processing_instruction(
    target: &str,
    value: &str,
    html: bool,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    output.push_str("<?")?;
    output.push_str(target)?;
    if !value.is_empty() {
        output.push(' ')?;
        output.push_str(value)?;
    }
    output.push_str(if html { ">" } else { "?>" })
}

fn begin_element<'a, 'o>(
    node: &'a ResultNode,
    namespace_scope: &mut NamespaceScope<'a>,
    options: SerializationOptions<'o>,
    depth: usize,
    output: &mut BudgetedString,
) -> Result<Option<ElementCursor<'a, 'o>>, ExecutionFailure> {
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
            serialize_element_attribute(attribute, name, namespace_scope, options, output)?;
        }
        let html_void = options.html_mode != HtmlMode::None && is_html_void_element(name);
        if options.xml_empty_element_tag && children.is_empty() {
            output.push_str("/>")?;
            return Ok(None);
        }
        if html_void && children.is_empty() {
            output.push('>')?;
            return Ok(None);
        }
        if options.xhtml_mode != XhtmlMode::None
            && children.is_empty()
            && is_xhtml_void_element(name, options.xhtml_mode)
        {
            output.push_str(" />")?;
            return Ok(None);
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
        Ok(Some((
            prefix,
            options,
            inject_content_type,
            indent_children,
            html_void,
        )))
    })();
    match result {
        Ok(Some((prefix, options, inject_content_type, indent_children, html_void))) => {
            Ok(Some(ElementCursor {
                name,
                children: children.iter(),
                namespace_frame: frame,
                prefix,
                options,
                depth,
                inject_content_type,
                indent_children,
                html_void,
            }))
        }
        result => {
            namespace_scope.exit(frame);
            result.map(|_| None)
        }
    }
}

fn finish_element(
    cursor: &ElementCursor<'_, '_>,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    if cursor.indent_children {
        write_indentation(cursor.depth, output)?;
    }
    if cursor.html_void {
        return Ok(());
    }
    output.push_str("</")?;
    write_name(cursor.prefix.as_deref(), &cursor.name.local, output)?;
    output.push('>')
}

fn serialize_element_attribute(
    attribute: &ResultAttribute,
    element_name: &crate::xml::quick_xml_experiment::ExpandedName,
    namespace_scope: &mut NamespaceScope<'_>,
    options: SerializationOptions<'_>,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    output.push(' ')?;
    let prefix = namespace_scope.attribute_prefix(attribute.name.namespace.as_deref(), output)?;
    write_name(prefix.as_deref(), &attribute.name.local, output)?;
    let html_attribute = uses_html_attribute_rules(element_name, options.html_mode);
    let uri_attribute_rules = html_attribute
        || options.xhtml_mode != XhtmlMode::None
            && element_name.namespace.as_deref() == Some("http://www.w3.org/1999/xhtml");
    let legacy_html_attribute =
        options.html_mode == HtmlMode::Legacy && element_name.namespace.is_none();
    if options.html_mode != HtmlMode::None
        && element_name.namespace.is_none()
        && prefix.is_none()
        && is_minimized_html_boolean_attribute(element_name, attribute)
    {
        return Ok(());
    }
    output.push_str("=\"")?;
    if uri_attribute_rules
        && options.escape_uri_attributes
        && is_uri_attribute(element_name, attribute)
    {
        if legacy_html_attribute {
            escape_legacy_html_uri_attribute(&attribute.value, output)?;
        } else {
            escape_uri_attribute(&attribute.value, output)?;
        }
    } else if html_attribute {
        escape_html_attribute_with_character_map(
            &attribute.value,
            options.character_map,
            options.normalization_form,
            output,
        )?;
    } else {
        escape_attribute_with_character_map(
            &attribute.value,
            options.character_map,
            options.normalization_form,
            output,
        )?;
    }
    output.push('"')
}

fn uses_html_attribute_rules(
    element: &crate::xml::quick_xml_experiment::ExpandedName,
    html_mode: HtmlMode,
) -> bool {
    match html_mode {
        HtmlMode::None => false,
        HtmlMode::Legacy => element.namespace.is_none(),
        HtmlMode::Five => matches!(
            element.namespace.as_deref(),
            None | Some("http://www.w3.org/1999/xhtml")
        ),
    }
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

fn serialize_child_prefix(
    child: &ResultNode,
    parent_name: &crate::xml::quick_xml_experiment::ExpandedName,
    options: SerializationOptions<'_>,
    depth: usize,
    indent: bool,
    output: &mut BudgetedString,
) -> Result<bool, ExecutionFailure> {
    if indent {
        write_indentation(depth + 1, output)?;
    }
    if options.cdata_section_elements.contains(parent_name)
        && let ResultNode::Text(value) = child
    {
        serialize_cdata(value, options.normalization_form, output)?;
        return Ok(true);
    }
    if options.html_mode != HtmlMode::None
        && is_html_raw_text_element(parent_name)
        && let ResultNode::Text(value) = child
    {
        write_character_mapped(
            value,
            options.character_map,
            options.normalization_form,
            output,
        )?;
        return Ok(true);
    }
    Ok(false)
}
