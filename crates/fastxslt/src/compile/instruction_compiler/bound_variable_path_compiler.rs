//! Bounded grouped variable-rooted child filters; no runtime values or owners.

use super::{
    CompileFailure, Document, NodeId, SourceLocation, is_ascii_ncname, value_expression_compiler,
};
use crate::xslt::golden_semantics_experiment::{BoundVariableChild, BoundVariablePath};

pub(super) fn compile_current_lookup(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<crate::xslt::golden_semantics_experiment::CurrentVariableLookup>, CompileFailure>
{
    let Some((variable, tail)) = expression
        .trim()
        .strip_prefix('$')
        .and_then(|v| v.split_once('/'))
    else {
        return Ok(None);
    };
    let Some((child, predicate)) = tail.split_once("[@") else {
        return Ok(None);
    };
    let Some((predicate, result)) = predicate.split_once("]/@") else {
        return Ok(None);
    };
    let Some((candidate, outer)) = predicate.split_once('=') else {
        return Ok(None);
    };
    let outer = outer.trim();
    let Some(outer) = outer
        .strip_prefix("current()/@")
        .or_else(|| outer.strip_prefix("(current())/@"))
    else {
        return Ok(None);
    };
    let qname = |value: &str| {
        let mut parts = value.split(':');
        parts.next().is_some_and(is_ascii_ncname)
            && parts.next().is_none_or(is_ascii_ncname)
            && parts.next().is_none()
    };
    if !is_ascii_ncname(variable)
        || ![child, candidate.trim(), outer.trim(), result]
            .into_iter()
            .all(qname)
    {
        return Ok(None);
    }
    let path = |tail: &str| -> Result<_, CompileFailure> {
        if tail.contains(':') {
            return super::parse_qualified_child_path(tail, location.clone(), |prefix| {
                super::namespace_for_prefix(document, element, prefix).map(str::to_owned)
            })
            .map_err(super::map_path_failure);
        }
        let (_, path) = value_expression_compiler::compile_xslt10_variable_path(
            document,
            element,
            &format!("$lookup/{tail}"),
            location,
        )?
        .expect("validated lookup path");
        Ok(path)
    };
    Ok(Some(
        crate::xslt::golden_semantics_experiment::CurrentVariableLookup {
            variable: variable.to_owned(),
            candidates: path(child)?,
            candidate_attribute: path(&format!("@{}", candidate.trim()))?,
            outer_attribute: path(&format!("@{}", outer.trim()))?,
            result_attribute: path(&format!("@{result}"))?,
        },
    ))
}

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
        for expression in [
            "$v/n[@key=current()/@key]/@out",
            "$v/n[@key=(current())/@key]/@out",
        ] {
            assert!(
                compile_current_lookup(&document, element, expression, location)
                    .unwrap()
                    .is_some()
            );
        }
        for expression in [
            "$v//n[@key=current()/@key]/@out",
            "$v/n[@key=current()/@key][1]/@out",
            "$v/n[@key=current()/@key]/@out/..",
            "$v/n[@key=current()/@key or true()]/@out",
            "$v/n[@key=current()/@key]/@out | /r",
            "$v/n[@key=current()/@key]/@",
            "$v/n[@key=current()/@key]/@a:b:c",
        ] {
            assert!(
                compile_current_lookup(&document, element, expression, location)
                    .unwrap()
                    .is_none(),
                "{expression}"
            );
        }
        let unbound = compile_current_lookup(
            &document,
            element,
            "$v/p:n[@key=current()/@key]/@out",
            location,
        )
        .unwrap_err();
        assert_eq!(unbound.code, "XPST0081");
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
