//! Bounded legacy grouped selections lowered to existing shared typed consumers.

use super::{
    CompileFailure, Document, NodeId, PathFailure, SourceLocation, ValueExpression, invalid,
    map_path_failure, matching_outer_parenthesis, namespace_for_prefix, parse_qualified_child_path,
    parse_xslt10_location_path, split_top_level_union,
};

use crate::xpath::path_experiment::qualified_nodes::{self, QualifiedLocationPath};
use crate::xslt::golden_semantics_experiment::{
    Xslt10GeneratedIdentitySelection, Xslt10GeneratedUnionPosition,
};

pub(super) fn compile_grouped_variable(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ValueExpression>, CompileFailure> {
    let expression = expression.trim();
    let Some(close) = matching_outer_parenthesis(expression) else {
        return Ok(None);
    };
    if close + 1 != expression.len() {
        return Ok(None);
    }
    let Some(variable) = expression[1..close].trim().strip_prefix('$') else {
        return Ok(None);
    };
    let is_name = crate::xml::names::is_ncname;
    if !variable.split_once(':').map_or_else(
        || is_name(variable),
        |(prefix, local)| is_name(prefix) && is_name(local),
    ) {
        return Ok(None);
    }
    // Only a transparent variable group is admitted; filters, operators and
    // nested general expressions keep their own typed eligibility checks.
    let name = super::normalize_variable_qname(document, element, variable).map_err(|_| {
        invalid(
            "FXXP0002",
            format!("invalid grouped variable reference: {expression}"),
            location,
        )
    })?;
    Ok(Some(ValueExpression::Xslt10VariableString(name)))
}

pub(in crate::compile) fn compile_generated_identity(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<Xslt10GeneratedUnionPosition>, CompileFailure> {
    let Some(argument) = expression
        .trim()
        .strip_prefix("generate-id(")
        .and_then(|tail| tail.strip_suffix(')'))
    else {
        return Ok(None);
    };
    let argument = argument.trim();
    let Some(close) = matching_outer_parenthesis(argument) else {
        return Ok(None);
    };
    let Some(predicate) = argument[close + 1..]
        .trim()
        .strip_prefix('[')
        .and_then(|tail| tail.strip_suffix(']'))
    else {
        return Ok(None);
    };
    let Some((variable, true)) = super::parse_variable_position_predicate(predicate) else {
        return Ok(None);
    };
    let group = &argument[..=close];
    let selection = if qualified_nodes::recognizes_namespace_axis(group) {
        let path = qualified_nodes::parse(&argument[1..close], location.clone())
            .map_err(map_path_failure)?;
        if !matches!(path, QualifiedLocationPath::Namespace { .. }) {
            return Err(super::unsupported(
                "FXXP1001",
                "positional namespace identity requires a terminal namespace selection",
                location,
            ));
        }
        Xslt10GeneratedIdentitySelection::Qualified(Box::new(path))
    } else {
        let Some(ValueExpression::Xslt10FirstNodePathUnion(paths)) =
            compile(document, element, group, location)?
        else {
            return Ok(None);
        };
        Xslt10GeneratedIdentitySelection::SourcePaths(paths)
    };
    Ok(Some(Xslt10GeneratedUnionPosition {
        selection,
        variable: variable.to_owned(),
        location: location.clone(),
    }))
}

pub(super) fn compile(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ValueExpression>, CompileFailure> {
    let expression = expression.trim();
    let Some(close) = matching_outer_parenthesis(expression) else {
        return Ok(None);
    };
    let take_last = expression[close + 1..].trim() == "[last()]";
    let expression = if take_last {
        expression[1..close].trim()
    } else {
        expression
    };
    let (group, suffix) = match matching_outer_parenthesis(expression) {
        Some(close) => (&expression[1..close], expression[close + 1..].trim()),
        None if take_last => (expression, ""),
        None => return Ok(None),
    };
    // Post-union predicates other than whole-set last(), descendant suffixes,
    // nested grouped branches, and namespace unions are not admitted here.
    if !suffix.is_empty() && (!suffix.starts_with('/') || suffix.starts_with("//")) {
        return Ok(None);
    }
    let Some(alternatives) = split_top_level_union(group) else {
        return Ok(None);
    };
    let mut paths = Vec::new();
    for alternative in alternatives {
        if alternative.trim().is_empty() {
            return Err(invalid(
                "XPST0003",
                "grouped path union contains an empty alternative",
                location,
            ));
        }
        let expanded = format!("{}{suffix}", alternative.trim());
        let path = parse_xslt10_location_path(&expanded, location.clone())
            .or_else(|failure| match failure {
                PathFailure::Unsupported { .. }
                    if expanded.contains(':') && !expanded.contains("::") =>
                {
                    parse_qualified_child_path(&expanded, location.clone(), |prefix| {
                        namespace_for_prefix(document, element, prefix).map(str::to_owned)
                    })
                }
                failure => Err(failure),
            })
            .map_err(map_path_failure)?;
        paths.push(path);
    }
    Ok(Some(if take_last {
        ValueExpression::Xslt10LastNodePathUnion(paths)
    } else {
        ValueExpression::Xslt10FirstNodePathUnion(paths)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    #[test]
    fn grouped_variable_scalar_only_recognizes_a_complete_variable_group() {
        let document = Document::from_parsed(parse_document("urn:grouped-variable:style", br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:variables"/>"#, ParseLimits { max_events: 8, max_depth: 4 }).unwrap()).unwrap();
        let element = document.children(document.document_node())[0];
        let location = document.location(element);
        for expression in ["($value)", " ( $value ) ", "($p:value)"] {
            assert!(matches!(
                compile_grouped_variable(&document, element, expression, location).unwrap(),
                Some(ValueExpression::Xslt10VariableString(_))
            ));
        }
        for expression in [
            "($a | $b)",
            "($a/child)",
            "($a)[1]",
            "($a) + 1",
            "(($a))",
            "('literal')",
            "($a",
            "($a))",
            "($a b)",
        ] {
            assert!(
                compile_grouped_variable(&document, element, expression, location)
                    .unwrap()
                    .is_none(),
                "{expression}"
            );
        }
        let error =
            compile_grouped_variable(&document, element, "($missing:value)", location).unwrap_err();
        assert_eq!(error.code, "FXXP0002");
        assert_eq!(error.location.resource, "urn:grouped-variable:style");
    }
}
