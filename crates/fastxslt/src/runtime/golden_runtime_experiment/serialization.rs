//! Private serialization of the golden slice's semantic result.

use super::{
    ExecutionFailure, FailureCategory, ResultNode, SemanticResult, control_failure, failure,
};
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xslt::golden_semantics_experiment::OutputSettings;

pub(in crate::runtime) fn serialize_xml(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
    byte_limit: usize,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    if settings
        .encoding
        .as_deref()
        .is_some_and(|encoding| !encoding.eq_ignore_ascii_case("UTF-8"))
    {
        return Err(failure(
            "FXSR1004",
            FailureCategory::Unsupported,
            Some(request_id),
            "the private string serialization lane supports only UTF-8",
        ));
    }
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
    });
    let inferred_html = settings.method.is_none()
        && matches!(
            first_significant,
            Some(ResultNode::Element { name, .. })
                if name.namespace.is_none() && name.local.eq_ignore_ascii_case("html")
        );
    if inferred_html {
        return Err(failure(
            "FXSR1001",
            FailureCategory::Unsupported,
            Some(request_id),
            "the selected output method is outside the private XML serialization slice",
        ));
    }
    if settings.method.as_deref() == Some("text") {
        let mut output = BudgetedString::new(byte_limit, request_id, control);
        for node in &result.children {
            serialize_text_node(node, &mut output)?;
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
    if settings.indent == Some(true) {
        return Err(failure(
            "FXSR1003",
            FailureCategory::Unsupported,
            Some(request_id),
            "indenting XML serialization is outside the private serialization slice",
        ));
    }
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
    for node in &result.children {
        serialize_node(node, &[], &settings.cdata_section_elements, &mut output)?;
    }
    Ok(output.finish())
}

#[cfg(test)]
pub(in crate::runtime) fn serialize_xml_bytes(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
    byte_limit: usize,
    control: &mut InvocationControl,
) -> Result<Vec<u8>, ExecutionFailure> {
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
            "FXSR1004",
            FailureCategory::Unsupported,
            Some(request_id),
            format!("unsupported byte serialization encoding: {encoding}"),
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

fn serialize_text_node(
    node: &ResultNode,
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    match node {
        ResultNode::Text(value) => output.push_str(value),
        ResultNode::Element { children, .. } => {
            for child in children {
                serialize_text_node(child, output)?;
            }
            Ok(())
        }
    }
}

fn serialize_node(
    node: &ResultNode,
    inherited_namespaces: &[crate::xml::quick_xml_experiment::NamespaceBinding],
    cdata_section_elements: &[crate::xml::quick_xml_experiment::ExpandedName],
    output: &mut BudgetedString,
) -> Result<(), ExecutionFailure> {
    match node {
        ResultNode::Text(value) => escape_text(value, output)?,
        ResultNode::Element {
            name,
            namespaces,
            attributes,
            children,
        } => {
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
                let prefix =
                    attribute_prefix(attribute.name.namespace.as_deref(), &in_scope, output)?;
                write_name(prefix, &attribute.name.local, output)?;
                output.push_str("=\"")?;
                escape_attribute(&attribute.value, output)?;
                output.push('"')?;
            }
            output.push('>')?;
            for child in children {
                if cdata_section_elements.contains(name) {
                    if let ResultNode::Text(value) = child {
                        serialize_cdata(value, output)?;
                        continue;
                    }
                }
                serialize_node(child, &in_scope, cdata_section_elements, output)?;
            }
            output.push_str("</")?;
            write_name(prefix, &name.local, output)?;
            output.push('>')?;
        }
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
        match character {
            '&' => output.push_str("&amp;")?,
            '<' => output.push_str("&lt;")?,
            '"' => output.push_str("&quot;")?,
            '\t' => output.push_str("&#x9;")?,
            '\n' => output.push_str("&#xA;")?,
            '\r' => output.push_str("&#xD;")?,
            _ => output.push(character)?,
        }
    }
    Ok(())
}

fn escape_text(value: &str, output: &mut BudgetedString) -> Result<(), ExecutionFailure> {
    for character in value.chars() {
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
