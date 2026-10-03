//! Bounded legacy grouped path unions lowered to the shared charged union plan.

use super::{
    CompileFailure, Document, NodeId, PathFailure, SourceLocation, ValueExpression, invalid,
    map_path_failure, matching_outer_parenthesis, namespace_for_prefix, parse_qualified_child_path,
    parse_xslt10_location_path, split_top_level_union,
};

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
