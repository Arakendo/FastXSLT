//! Private scalar literal-document lookup compilation.
//! Static names and paths only; no resource acquisition or runtime values.

use super::{
    CompileFailure, Document, ExpandedName, NodeId, SourceLocation, ValueExpression, invalid,
    is_ascii_ncname, map_path_failure, namespace_for_prefix, parse_qualified_child_path,
    parse_xslt10_location_path, unsupported,
};

pub(super) fn compile_xslt10_literal_document_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ValueExpression>, CompileFailure> {
    let expression = expression.trim();
    let Some(argument_and_tail) = expression.strip_prefix("document(") else {
        return Ok(None);
    };
    let Some(quote) = argument_and_tail.as_bytes().first().copied() else {
        return Ok(None);
    };
    if !matches!(quote, b'\'' | b'"') {
        return Ok(None);
    }
    let Some(closing_quote) = argument_and_tail.as_bytes()[1..]
        .iter()
        .position(|byte| *byte == quote)
        .map(|position| position + 1)
    else {
        return Ok(None);
    };
    let reference = &argument_and_tail[1..closing_quote];
    let Some(tail) = argument_and_tail[closing_quote + 1..].strip_prefix(')') else {
        return Ok(None);
    };
    if !tail.is_empty() && !tail.starts_with('/') {
        return Ok(None);
    }
    let path_expression = if tail.is_empty() { "/" } else { tail };
    let (path_expression, outer_attribute_filter) = if let Some((path, predicate)) =
        path_expression.rsplit_once("[@")
        && let Some(predicate) = predicate.strip_suffix(']')
        && let Some((candidate, outer)) = predicate.split_once('=')
        && let Some(outer) = outer.trim().strip_prefix("current()/@")
    {
        let resolve = |name: &str| -> Result<ExpandedName, CompileFailure> {
            let name = name.trim();
            let (prefix, local) = name
                .split_once(':')
                .map_or((None, name), |(prefix, local)| (Some(prefix), local));
            if !is_ascii_ncname(local) || prefix.is_some_and(|prefix| !is_ascii_ncname(prefix)) {
                return Err(unsupported(
                    "FXXP1001",
                    "this current-attribute lookup name is not admitted",
                    location,
                ));
            }
            let namespace = prefix
                .map(|prefix| {
                    namespace_for_prefix(document, element, prefix)
                        .map(str::to_owned)
                        .ok_or_else(|| {
                            invalid(
                                "XPST0081",
                                format!("unbound current-attribute lookup prefix: {prefix}"),
                                location,
                            )
                        })
                })
                .transpose()?;
            Ok(ExpandedName {
                namespace,
                local: local.to_owned(),
            })
        };
        (
            path,
            Some(
                crate::xslt::golden_semantics_experiment::Xslt10OuterAttributeFilter {
                    candidate: resolve(candidate)?,
                    outer: resolve(outer)?,
                },
            ),
        )
    } else {
        (path_expression, None)
    };
    if outer_attribute_filter.is_some() && path_expression.contains('[') {
        return Err(unsupported(
            "FXXP1001",
            "current-attribute document lookup admits only one terminal predicate",
            location,
        ));
    }
    let path = if outer_attribute_filter.is_some() && path_expression.contains(':') {
        parse_qualified_child_path(path_expression, location.clone(), |prefix| {
            namespace_for_prefix(document, element, prefix).map(str::to_owned)
        })
    } else {
        parse_xslt10_location_path(path_expression, location.clone())
    }
    .map_err(map_path_failure)?;
    Ok(Some(ValueExpression::Xslt10LiteralDocumentPath {
        reference: crate::xslt::golden_semantics_experiment::DocumentRootReference {
            base: location.resource.clone(),
            reference: reference.to_owned(),
            descendant_name: None,
        },
        path,
        outer_attribute_filter,
    }))
}
