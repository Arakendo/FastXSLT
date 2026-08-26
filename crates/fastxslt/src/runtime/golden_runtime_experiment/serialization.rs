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
    if inferred_html
        || settings
            .method
            .as_deref()
            .is_some_and(|method| method != "xml")
    {
        return Err(failure(
            "FXSR1001",
            FailureCategory::Unsupported,
            Some(request_id),
            "the selected output method is outside the private XML serialization slice",
        ));
    }
    let mut output = BudgetedString::new(byte_limit, request_id, control);
    if !settings.omit_xml_declaration {
        output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    }
    for node in &result.children {
        serialize_node(node, &mut output)?;
    }
    Ok(output.finish())
}

fn serialize_node(node: &ResultNode, output: &mut BudgetedString) -> Result<(), ExecutionFailure> {
    match node {
        ResultNode::Text(value) => escape_text(value, output)?,
        ResultNode::Element { name, children } => {
            if name.namespace.is_some() {
                return Err(failure(
                    "FXSR1002",
                    FailureCategory::Unsupported,
                    Some(&output.request_id),
                    "namespaced result serialization is outside the private slice",
                ));
            }
            output.push('<')?;
            output.push_str(&name.local)?;
            output.push('>')?;
            for child in children {
                serialize_node(child, output)?;
            }
            output.push_str("</")?;
            output.push_str(&name.local)?;
            output.push('>')?;
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
