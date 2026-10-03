//! Shallow recursive serializer oracle retained before the stack-safe repair.

use super::{
    BudgetedString, ExecutionFailure, HtmlMode, IndentationState, NamespaceScope, ResultNode,
    SerializationOptions, XhtmlMode, enter_element_namespace_scope, is_content_type_head,
    is_html_raw_text_element, is_html_void_element, is_replaced_content_type_meta,
    is_xhtml_void_element, serialize_cdata, serialize_content_type_if_needed,
    serialize_element_attribute, serialize_leaf, write_character_mapped, write_indentation,
    write_name,
};

pub(super) fn serialize_node<'a>(
    node: &'a ResultNode,
    namespace_scope: &mut NamespaceScope<'a>,
    options: SerializationOptions<'_>,
    depth: usize,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    if matches!(node, ResultNode::Element { .. }) {
        serialize_element(node, namespace_scope, options, depth, output)
    } else {
        serialize_leaf(node, options, output)
    }
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
            serialize_element_attribute(attribute, name, namespace_scope, options, output)?;
        }
        let html_void = options.html_mode != HtmlMode::None && is_html_void_element(name);
        if options.xml_empty_element_tag && children.is_empty() {
            return output.push_str("/>");
        }
        if html_void && children.is_empty() {
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
        if html_void {
            return Ok(());
        }
        output.push_str("</")?;
        write_name(prefix.as_deref(), &name.local, output)?;
        output.push('>')
    })();
    namespace_scope.exit(frame);
    result
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
