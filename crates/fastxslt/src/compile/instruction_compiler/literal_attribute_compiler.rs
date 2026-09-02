//! Literal result-attribute compilation for the admitted private AVT slice.

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xslt::golden_semantics_experiment::{LiteralAttribute, LiteralAttributeValue};

use super::{CompileFailure, XSLT_NAMESPACE, invalid, is_ascii_ncname, unsupported};

const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";

pub(crate) fn compile_literal_result_attributes(
    document: &Document,
    element: NodeId,
) -> Result<Vec<LiteralAttribute>, CompileFailure> {
    let mut attributes = Vec::new();
    for attribute in document.attributes(element) {
        let name = document
            .name(*attribute)
            .expect("attribute nodes have expanded names");
        if name.namespace.as_deref() == Some(XSLT_NAMESPACE) {
            continue;
        }
        if name
            .namespace
            .as_deref()
            .is_some_and(|namespace| namespace != XML_NAMESPACE)
        {
            return Err(unsupported(
                "FXST1007",
                "namespaced literal result attributes are outside the private slice",
                document.location(*attribute),
            ));
        }
        let lexical = document.string_value(*attribute);
        let value = parse_literal_attribute_value(&lexical, document.location(*attribute))?;
        attributes.push(LiteralAttribute {
            name: name.clone(),
            value,
            location: document.location(*attribute).clone(),
        });
    }
    Ok(attributes)
}

fn parse_literal_attribute_value(
    lexical: &str,
    location: &SourceLocation,
) -> Result<LiteralAttributeValue, CompileFailure> {
    if lexical == "{position()}" {
        return Ok(LiteralAttributeValue::ContextPosition);
    }
    if lexical == "{last()}" {
        return Ok(LiteralAttributeValue::ContextSize);
    }
    if let Some(variable) = lexical
        .strip_prefix("{$")
        .and_then(|value| value.strip_suffix('}'))
    {
        if is_ascii_ncname(variable) {
            return Ok(LiteralAttributeValue::Variable(variable.to_owned()));
        }
        return Err(invalid(
            "FXST0031",
            format!("invalid variable-only attribute value template: {lexical}"),
            location,
        ));
    }
    if lexical.contains(['{', '}']) {
        return Err(unsupported(
            "FXST1031",
            format!("unsupported attribute value template: {lexical}"),
            location,
        ));
    }
    Ok(LiteralAttributeValue::Text(lexical.to_owned()))
}
