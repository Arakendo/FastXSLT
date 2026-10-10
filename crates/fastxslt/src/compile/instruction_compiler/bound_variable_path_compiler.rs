//! Bounded grouped variable-rooted child filters; no runtime values or owners.

use super::{
    CompileFailure, Document, NodeId, SourceLocation, is_ascii_ncname, value_expression_compiler,
};
use crate::xslt::golden_semantics_experiment::{BoundVariableChild, BoundVariablePath};

pub(super) fn compile(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<(String, BoundVariablePath)>, CompileFailure> {
    let Some(body) = expression
        .trim()
        .strip_prefix('(')
        .and_then(|body| body.strip_suffix(")[1]"))
    else {
        return Ok(None);
    };
    let Some((source, tail)) = body
        .trim()
        .strip_prefix('$')
        .and_then(|body| body.split_once('/'))
    else {
        return Ok(None);
    };
    if !is_ascii_ncname(source) {
        return Ok(None);
    }
    let mut steps = Vec::new();
    for step in tail.split('/') {
        let (name, filter) = if let Some((name, predicate)) = step.split_once("[@") {
            let Some((attribute, variable)) = predicate
                .strip_suffix(']')
                .and_then(|predicate| predicate.split_once('='))
            else {
                return Ok(None);
            };
            let Some(variable) = variable
                .trim()
                .strip_prefix('$')
                .filter(|variable| is_ascii_ncname(variable))
            else {
                return Ok(None);
            };
            let attribute = attribute.trim();
            if !is_ascii_ncname(attribute) {
                return Ok(None);
            }
            let (_, path) = value_expression_compiler::compile_xslt10_variable_path(
                document,
                element,
                &format!("$root/@{attribute}"),
                location,
            )?
            .expect("validated attribute path");
            (name.trim(), Some((path, variable.to_owned())))
        } else {
            (step.trim(), None)
        };
        if !is_ascii_ncname(name) {
            return Ok(None);
        }
        let (_, path) = value_expression_compiler::compile_xslt10_variable_path(
            document,
            element,
            &format!("$root/{name}"),
            location,
        )?
        .expect("validated child path");
        steps.push(BoundVariableChild { path, filter });
    }
    if steps.is_empty() {
        return Ok(None);
    }
    Ok(Some((
        source.to_owned(),
        BoundVariablePath::FirstFilteredChildren(steps),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    #[test]
    fn grouped_variable_filters_reject_unadmitted_grammar_without_rewriting_it() {
        let document = Document::from_parsed(parse_document("urn:bound-path-compile", br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"/>"#, ParseLimits { max_events: 8, max_depth: 4 }).unwrap()).unwrap();
        let element = document.children(document.document_node())[0];
        let location = document.location(element);
        for expression in ["($doc/a[@key=$value]/b)[1]", "($doc/a/b)[1]"] {
            assert!(
                compile(&document, element, expression, location)
                    .unwrap()
                    .is_some()
            );
        }
        for expression in [
            "($doc/a[@key=$value]/b)[2]",
            "($doc//a)[1]",
            "($doc/a|b)[1]",
            "($doc/*)[1]",
            "($doc/a[@key=$value][@other=$second])[1]",
            "($doc/a[@key='literal'])[1]",
            "($doc/a[@key=$val ue])[1]",
            "($doc/a/../b)[1]",
            "($doc/a[@key=$value]/b)",
        ] {
            assert!(
                compile(&document, element, expression, location)
                    .unwrap()
                    .is_none(),
                "{expression}"
            );
        }
    }
}
