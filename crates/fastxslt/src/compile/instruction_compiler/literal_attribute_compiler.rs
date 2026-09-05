//! Literal result-attribute compilation for the admitted private AVT slice.

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xslt::golden_semantics_experiment::{LiteralAttribute, LiteralAttributeValue};

use super::{CompileFailure, XSLT_NAMESPACE, invalid, is_ascii_ncname, unsupported};

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
    if lexical == "{local-name()}" {
        return Ok(LiteralAttributeValue::ContextLocalName);
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
        if let Some(text) = unescape_static_braces(lexical) {
            return Ok(LiteralAttributeValue::Text(text));
        }
        return Err(unsupported(
            "FXST1031",
            format!("unsupported attribute value template: {lexical}"),
            location,
        ));
    }
    Ok(LiteralAttributeValue::Text(lexical.to_owned()))
}

fn unescape_static_braces(lexical: &str) -> Option<String> {
    let mut characters = lexical.chars().peekable();
    let mut result = String::with_capacity(lexical.len());
    while let Some(character) = characters.next() {
        if matches!(character, '{' | '}') {
            characters.next_if_eq(&character)?;
        }
        result.push(character);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use crate::xdm::owned_tree_experiment::SourceLocation;
    use crate::xslt::golden_semantics_experiment::LiteralAttributeValue;

    use super::parse_literal_attribute_value;

    fn location() -> SourceLocation {
        SourceLocation {
            resource: "memory:stylesheet.xsl".to_owned(),
            span: 10..20,
        }
    }

    #[test]
    fn unescapes_only_paired_static_avt_braces() {
        assert_eq!(
            parse_literal_attribute_value("{{font:helvetica}}", &location())
                .expect("paired braces should be static text"),
            LiteralAttributeValue::Text("{font:helvetica}".to_owned())
        );
        assert!(parse_literal_attribute_value("before{.}after", &location()).is_err());
        assert!(parse_literal_attribute_value("{{broken}", &location()).is_err());
    }
}
