//! Typed `XPath` 1.0 ID lookup compilation, independent of runtime and acquisition.

use super::{
    CompileFailure, SourceLocation, ValueExpression, Xslt10IdArgument, Xslt10IdLookup, invalid,
    is_ascii_ncname, map_path_failure, parse_xslt10_location_path, xpath_string_literal,
};

pub(super) fn compile_node_name(
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ValueExpression>, CompileFailure> {
    for function in ["name", "local-name", "namespace-uri"] {
        let Some(argument) = expression
            .trim()
            .strip_prefix(function)
            .map(str::trim_start)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
        else {
            continue;
        };
        let Some(lookup) = compile_xslt10_id_lookup(argument, location)? else {
            return Ok(None);
        };
        let lookup = Box::new(lookup);
        return Ok(Some(match function {
            "name" => ValueExpression::Xslt10NameIdLookup(lookup),
            "local-name" => ValueExpression::Xslt10LocalNameIdLookup(lookup),
            "namespace-uri" => ValueExpression::Xslt10NamespaceUriIdLookup(lookup),
            _ => unreachable!("bounded node-name function dispatch"),
        }));
    }
    Ok(None)
}

pub(in crate::compile) fn compile_xslt10_id_lookup(
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<Xslt10IdLookup>, CompileFailure> {
    let expression = expression.trim();
    let Some(arguments_and_tail) = expression.strip_prefix("id(") else {
        return Ok(None);
    };
    let close = matching_id_parenthesis(arguments_and_tail)
        .ok_or_else(|| invalid("XPST0003", "id() requires a closed argument", location))?;
    let argument = arguments_and_tail[..close].trim();
    if argument.is_empty() {
        return Err(invalid(
            "XPST0017",
            "id() requires exactly one argument",
            location,
        ));
    }
    let argument = if let Some(literal) = xpath_string_literal(argument) {
        Xslt10IdArgument::Literal(literal.to_owned())
    } else if let Some(variable) = argument
        .strip_prefix('$')
        .filter(|name| is_ascii_ncname(name))
    {
        Xslt10IdArgument::Variable(variable.to_owned())
    } else {
        Xslt10IdArgument::Path(
            parse_xslt10_location_path(argument, location.clone()).map_err(map_path_failure)?,
        )
    };
    let tail = arguments_and_tail[close + 1..].trim();
    let tail = tail.trim();
    let relative_path = if tail.is_empty() {
        None
    } else {
        let Some(relative) = tail.strip_prefix('/') else {
            return Err(invalid(
                "XPST0003",
                "id() may be followed only by a relative path",
                location,
            ));
        };
        if relative.is_empty() {
            return Err(invalid(
                "XPST0003",
                "id() path must contain a step after '/'",
                location,
            ));
        }
        Some(if let Some(descendant) = relative.strip_prefix('/') {
            parse_xslt10_location_path(&format!(".//{descendant}"), location.clone())
                .map_err(map_path_failure)?
        } else {
            parse_xslt10_location_path(relative, location.clone()).map_err(map_path_failure)?
        })
    };
    Ok(Some(Xslt10IdLookup {
        argument,
        relative_path,
    }))
}

fn matching_id_parenthesis(arguments_and_tail: &str) -> Option<usize> {
    let mut quote = None;
    let mut depth = 0_usize;
    for (offset, character) in arguments_and_tail.char_indices() {
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            }
            continue;
        }
        if quote.is_some() {
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(offset),
            ')' => depth = depth.checked_sub(1)?,
            _ => {}
        }
    }
    None
}
