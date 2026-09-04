//! Compiled `fn:default-collation` expressions for the production XSLT path.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

pub(crate) const CODEPOINT_COLLATION: &str =
    "http://www.w3.org/2005/xpath-functions/collation/codepoint";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DefaultCollationExpression {
    Value,
    Equals(String),
    Count,
    Boolean,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DefaultCollationValue {
    Boolean(bool),
    Integer(usize),
    String(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DefaultCollationParseFailure {
    InvalidArity,
    Unsupported,
}

pub(crate) fn recognizes(expression: &str) -> bool {
    expression.contains("default-collation")
}

pub(crate) fn parse(
    expression: &str,
) -> Result<DefaultCollationExpression, DefaultCollationParseFailure> {
    let expression = expression.trim();
    if is_value_call(expression) {
        return Ok(DefaultCollationExpression::Value);
    }
    if let Some(argument) = wrapper_argument(expression, &["count", "fn:count"]) {
        return if is_value_call(argument.trim()) {
            Ok(DefaultCollationExpression::Count)
        } else {
            Err(DefaultCollationParseFailure::Unsupported)
        };
    }
    if let Some(argument) = wrapper_argument(expression, &["boolean", "fn:boolean"]) {
        return if is_value_call(argument.trim()) {
            Ok(DefaultCollationExpression::Boolean)
        } else {
            Err(DefaultCollationParseFailure::Unsupported)
        };
    }
    if let Some((left, right)) = expression.split_once(" eq ") {
        if is_value_call(left.trim()) {
            let expected = parse_quoted_string(right.trim())
                .ok_or(DefaultCollationParseFailure::Unsupported)?;
            return Ok(DefaultCollationExpression::Equals(expected));
        }
    }
    if value_call_argument(expression).is_some() {
        return Err(DefaultCollationParseFailure::InvalidArity);
    }
    Err(DefaultCollationParseFailure::Unsupported)
}

pub(crate) fn evaluate(
    expression: &DefaultCollationExpression,
    control: &mut InvocationControl,
) -> Result<DefaultCollationValue, ControlFailure> {
    control.charge(WorkDomain::XPathOperation, 1)?;
    Ok(match expression {
        DefaultCollationExpression::Value => {
            DefaultCollationValue::String(CODEPOINT_COLLATION.to_owned())
        }
        DefaultCollationExpression::Equals(expected) => {
            DefaultCollationValue::Boolean(CODEPOINT_COLLATION == expected)
        }
        DefaultCollationExpression::Count => DefaultCollationValue::Integer(1),
        DefaultCollationExpression::Boolean => DefaultCollationValue::Boolean(true),
    })
}

fn is_value_call(expression: &str) -> bool {
    matches!(expression, "default-collation()" | "fn:default-collation()")
}

fn value_call_argument(expression: &str) -> Option<&str> {
    wrapper_argument(expression, &["default-collation", "fn:default-collation"])
}

fn wrapper_argument<'a>(expression: &'a str, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| {
        expression
            .strip_prefix(name)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
    })
}

fn parse_quoted_string(expression: &str) -> Option<String> {
    for quote in ['"', '\''] {
        if let Some(value) = expression
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
            .filter(|value| !value.contains(quote))
        {
            return Some(value.to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};

    use super::{
        CODEPOINT_COLLATION, DefaultCollationExpression, DefaultCollationParseFailure,
        DefaultCollationValue, evaluate, parse,
    };

    #[test]
    fn parses_and_evaluates_the_complete_admitted_expression_shapes() {
        for (source, expected) in [
            (
                "fn:default-collation()",
                DefaultCollationValue::String(CODEPOINT_COLLATION.to_owned()),
            ),
            (
                "default-collation() eq \"http://www.w3.org/2005/xpath-functions/collation/codepoint\"",
                DefaultCollationValue::Boolean(true),
            ),
            (
                "fn:count(fn:default-collation())",
                DefaultCollationValue::Integer(1),
            ),
            (
                "fn:boolean(fn:default-collation())",
                DefaultCollationValue::Boolean(true),
            ),
        ] {
            let compiled = parse(source).expect("expression should compile");
            let mut control = InvocationControl::unbounded();
            assert_eq!(evaluate(&compiled, &mut control), Ok(expected), "{source}");
            assert_eq!(control.consumed(WorkDomain::XPathOperation), 1);
        }
    }

    #[test]
    fn rejects_every_nonzero_arity_value_call() {
        for source in [
            "fn:default-collation(\"An Argument\")",
            "default-collation(.)",
            "default-collation(1, 2)",
        ] {
            assert_eq!(
                parse(source),
                Err(DefaultCollationParseFailure::InvalidArity),
                "{source}"
            );
        }
    }

    #[test]
    fn retains_the_typed_compiled_variants() {
        assert_eq!(
            parse("fn:count(fn:default-collation())"),
            Ok(DefaultCollationExpression::Count)
        );
        assert_eq!(
            parse("fn:boolean(fn:default-collation())"),
            Ok(DefaultCollationExpression::Boolean)
        );
    }
}
