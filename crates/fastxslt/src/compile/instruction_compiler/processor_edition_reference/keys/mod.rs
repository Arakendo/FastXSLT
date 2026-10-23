//! Selected key admission only; indexing, lookup and dispatch remain shared.
#[cfg(test)]
use super::*;
#[cfg(not(test))]
use super::{
    CompileFailure, Document, LocalVersion, NodeId, ensure_no_meaningful_children, is_ascii_ncname,
    refusal, required_attribute, validate_value_binding, value_expression_compiler,
    xpath_string_literal,
};
use crate::xslt::golden_semantics_experiment::Xslt10KeyLookup;

pub(super) fn validate_declaration(
    document: &Document,
    element: NodeId,
) -> Result<(), CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document.name(*attribute).expect("attribute");
        if name.namespace.is_some() || !matches!(name.local.as_str(), "name" | "match" | "use") {
            return Err(refusal(document, *attribute));
        }
    }
    if !is_ascii_ncname(required_attribute(document, element, None, "name")?)
        || !is_ascii_ncname(required_attribute(document, element, None, "match")?)
        || !required_attribute(document, element, None, "use")?
            .strip_prefix('@')
            .is_some_and(is_ascii_ncname)
    {
        return Err(refusal(document, element));
    }
    ensure_no_meaningful_children(document, element, "xsl:key")
}

pub(super) fn lookup(
    document: &Document,
    element: NodeId,
    expression: &str,
    version: LocalVersion,
    bindings: &[String],
) -> Result<Option<Xslt10KeyLookup>, CompileFailure> {
    if matches!(version, LocalVersion::ModernWitness(_)) {
        return Ok(None);
    }
    let Some((key, value)) = expression
        .trim()
        .strip_prefix("key(")
        .and_then(|value| value.strip_suffix(')'))
        .and_then(|args| args.split_once(','))
    else {
        return Ok(None);
    };
    if !xpath_string_literal(key.trim()).is_some_and(is_ascii_ncname) {
        return Ok(None);
    }
    let value = value.trim();
    if let Some(name) = value.strip_prefix('$').filter(|name| is_ascii_ncname(name)) {
        validate_value_binding(document, element, name, bindings)?;
    } else if !value.strip_prefix('@').is_some_and(is_ascii_ncname) {
        return Ok(None);
    }
    value_expression_compiler::compile_xslt10_literal_key_lookup(
        document,
        element,
        expression,
        document.location(element),
    )
    .map(Some)
}
