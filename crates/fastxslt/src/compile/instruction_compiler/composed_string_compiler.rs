//! Bounded XSLT 1.0 string composition, shared by output and concatenation.

use super::{
    CompileFailure, Document, NodeId, SourceLocation, boolean_expression_compiler,
    effective_xpath_default_namespace, map_path_failure, parse_location_path,
};
use crate::xslt::golden_semantics_experiment::{
    StringComparison, Xslt10ComposedString, Xslt10StringBound,
};

pub(super) fn compile(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<Xslt10ComposedString>, CompileFailure> {
    // Preserve the existing constant-bound path implementation as the reference.
    if !expression.trim_start().starts_with("substring(") {
        return Ok(None);
    }
    if let Some(args) = arguments(expression, "substring(", 3)
        && args.len() >= 2
        && !args[0].contains('(')
        && args[1..].iter().all(|value| {
            crate::xpath::static_string_experiment::parse_finite_number(value).is_some()
        })
    {
        return Ok(None);
    }
    let mut remaining = 128;
    parse_string(document, element, expression, location, 0, &mut remaining)
}

fn arguments<'a>(expression: &'a str, name: &str, maximum: usize) -> Option<Vec<&'a str>> {
    expression
        .trim()
        .strip_prefix(name)?
        .strip_suffix(')')
        .and_then(|value| crate::xpath::static_string_experiment::split_arguments(value, maximum))
}

fn enter(depth: usize, remaining: &mut usize) -> bool {
    if depth >= 16 || *remaining == 0 {
        return false;
    }
    *remaining -= 1;
    true
}

fn parse_string(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
    depth: usize,
    remaining: &mut usize,
) -> Result<Option<Xslt10ComposedString>, CompileFailure> {
    if !enter(depth, remaining) {
        return Ok(None);
    }
    if let Some(args) = arguments(expression, "substring(", 3) {
        let [source, start, rest @ ..] = args.as_slice() else {
            return Ok(None);
        };
        let Some(source) = parse_string(document, element, source, location, depth + 1, remaining)?
        else {
            return Ok(None);
        };
        let Some(start) = parse_bound(document, element, start, location, depth + 1, remaining)?
        else {
            return Ok(None);
        };
        let length = if let Some(value) = rest.first() {
            let Some(value) =
                parse_bound(document, element, value, location, depth + 1, remaining)?
            else {
                return Ok(None);
            };
            Some(Box::new(value))
        } else {
            None
        };
        return Ok(Some(Xslt10ComposedString::Substring {
            source: Box::new(source),
            start: Box::new(start),
            length,
        }));
    }
    if let Some(args) = arguments(expression, "substring-after(", 2) {
        let [source, delimiter] = args.as_slice() else {
            return Ok(None);
        };
        let Some(delimiter) = super::xpath_string_literal(delimiter) else {
            return Ok(None);
        };
        let Some(source) = parse_string(document, element, source, location, depth + 1, remaining)?
        else {
            return Ok(None);
        };
        return Ok(Some(Xslt10ComposedString::After {
            source: Box::new(source),
            delimiter: delimiter.to_owned(),
        }));
    }
    // Variables, qualified document paths and arbitrary functions stay unadmitted.
    if expression.contains('(') || expression.contains('$') {
        return Ok(None);
    }
    let mut path =
        parse_location_path(expression.trim(), location.clone()).map_err(map_path_failure)?;
    if let Some(namespace) = effective_xpath_default_namespace(document, element) {
        for step in &mut path.steps {
            if let crate::xpath::path_experiment::PathStep::ChildNamed(local) = step {
                *step = crate::xpath::path_experiment::PathStep::ChildExpandedName(
                    crate::xml::quick_xml_experiment::ExpandedName {
                        namespace: Some(namespace.to_owned()),
                        local: local.clone(),
                    },
                );
            }
        }
    }
    Ok(Some(Xslt10ComposedString::Path(path)))
}

fn parse_bound(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
    depth: usize,
    remaining: &mut usize,
) -> Result<Option<Xslt10StringBound>, CompileFailure> {
    if !enter(depth, remaining) {
        return Ok(None);
    }
    if let Some(value) = crate::xpath::static_string_experiment::parse_finite_number(expression) {
        return Ok(Some(Xslt10StringBound::Literal(value.to_bits())));
    }
    if let Some((
        left,
        crate::xpath::binary_numeric_experiment::BinaryNumericOperator::Multiply,
        right,
    )) = crate::xpath::binary_numeric_experiment::split_paths(expression)
    {
        let Some(left) = parse_bound(document, element, left, location, depth + 1, remaining)?
        else {
            return Ok(None);
        };
        let Some(right) = parse_bound(document, element, right, location, depth + 1, remaining)?
        else {
            return Ok(None);
        };
        return Ok(Some(Xslt10StringBound::Multiply(
            Box::new(left),
            Box::new(right),
        )));
    }
    if let Some(args) = arguments(expression, "string-length(", 1) {
        let [source] = args.as_slice() else {
            return Ok(None);
        };
        return Ok(
            parse_string(document, element, source, location, depth + 1, remaining)?
                .map(|source| Xslt10StringBound::Length(Box::new(source))),
        );
    }
    if let Some(args) = arguments(expression, "number(", 1) {
        let [boolean] = args.as_slice() else {
            return Ok(None);
        };
        // Limit this operand to already admitted boolean compilation, without
        // variables or recursive calls back into composed strings.
        if !boolean_shape(boolean, 0) {
            return Ok(None);
        }
        return boolean_expression_compiler::compile(
            document,
            element,
            boolean,
            location,
            StringComparison::Codepoint,
            true,
        )
        .map(|value| Some(Xslt10StringBound::Boolean(Box::new(value))));
    }
    Ok(None)
}

fn boolean_shape(expression: &str, depth: usize) -> bool {
    if depth >= 16 {
        return false;
    }
    if let Some(args) = arguments(expression, "not(", 1) {
        return args.len() == 1 && boolean_shape(args[0], depth + 1);
    }
    ["starts-with(", "contains("].iter().any(|name| {
        arguments(expression, name, 2).is_some_and(|args| {
            args.len() == 2
                && !args[0].contains('(')
                && !args[0].contains('$')
                && super::xpath_string_literal(args[1]).is_some()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composed_string_grammar_is_bounded_and_does_not_convert_arbitrary_numbers_as_booleans() {
        let parsed = crate::xml::quick_xml_experiment::parse_document(
            "urn:composed-grammar",
            b"<style/>",
            crate::xml::quick_xml_experiment::ParseLimits {
                max_events: 8,
                max_depth: 4,
            },
        )
        .unwrap();
        let document = Document::from_parsed(parsed).unwrap();
        let element = document.children(document.document_node())[0];
        let parse = |value: &str| compile(&document, element, value, document.location(element));
        for value in [
            "substring(name,1,string-length(name))",
            "substring(substring-after(name,','),1,2)",
            "substring(name,1,number(not(starts-with(.,'Re: ')))*string-length(name))",
        ] {
            assert!(parse(value).unwrap().is_some(), "{value}");
        }
        for value in [
            "substring(name,1,$n)",
            "substring(name,1,number(5))",
            "substring(name,1,number(name))",
            "substring(name,1,string-length($v))",
            "substring(name,1,unknown())",
            "substring(name,1,2,3)",
        ] {
            assert!(parse(value).unwrap().is_none(), "{value}");
        }
        let mut nested = "name".to_owned();
        for _ in 0..20 {
            nested = format!("substring({nested},1,2)");
        }
        assert!(parse(&nested).unwrap().is_none());
        let mut remaining = 0;
        assert!(
            parse_string(
                &document,
                element,
                "name",
                document.location(element),
                0,
                &mut remaining
            )
            .unwrap()
            .is_none()
        );
    }
}
