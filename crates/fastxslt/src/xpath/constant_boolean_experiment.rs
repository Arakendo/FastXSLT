//! Private source-free `XPath` boolean-expression slice.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

use super::deep_equal_atomic::{
    EffectiveBooleanValueFailure, parse_effective_boolean_value, parse_sequence,
    split_top_level_once,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BooleanExpression {
    Constant(bool),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Compare {
        left: Box<Self>,
        operator: BooleanComparison,
        right: Box<Self>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BooleanComparison {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BooleanParseFailure {
    InvalidArity,
    InvalidEffectiveBooleanValue,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BooleanEvaluationFailure {
    Control(ControlFailure),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ScalarExpression {
    Boolean(BooleanExpression),
    BooleanString(BooleanExpression),
    Concat(Box<Self>, Box<Self>),
    Contains(Box<Self>, Box<Self>),
    StringLength(Box<Self>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ScalarValue {
    Boolean(bool),
    String(String),
    Integer(usize),
}

pub(crate) fn parse(expression: &str) -> Result<BooleanExpression, BooleanParseFailure> {
    parse_inner(expression.trim())
}

pub(crate) fn parse_scalar(expression: &str) -> Result<ScalarExpression, BooleanParseFailure> {
    let expression = expression.trim();
    if let Some(inner) = function_argument(expression, &["string", "fn:string", "xs:string"]) {
        return parse(inner).map(ScalarExpression::BooleanString);
    }
    if let Ok(boolean) = parse(expression) {
        return Ok(ScalarExpression::Boolean(boolean));
    }
    if let Some(inner) = function_argument(expression, &["fn:concat"]) {
        let (left, right) = split_top_level(inner, ",").ok_or(BooleanParseFailure::Unsupported)?;
        return Ok(ScalarExpression::Concat(
            Box::new(parse_scalar(left)?),
            Box::new(parse_scalar(right)?),
        ));
    }
    if let Some(inner) = function_argument(expression, &["fn:contains"]) {
        let (left, right) = split_top_level(inner, ",").ok_or(BooleanParseFailure::Unsupported)?;
        return Ok(ScalarExpression::Contains(
            Box::new(parse_scalar(left)?),
            Box::new(parse_scalar(right)?),
        ));
    }
    if let Some(inner) = function_argument(expression, &["fn:string-length"]) {
        return Ok(ScalarExpression::StringLength(Box::new(parse_scalar(
            inner,
        )?)));
    }
    parse(expression).map(ScalarExpression::Boolean)
}

fn parse_inner(expression: &str) -> Result<BooleanExpression, BooleanParseFailure> {
    let expression = strip_balanced_parentheses(expression);
    if let Some((left, right)) = split_top_level(expression, " or ") {
        return Ok(BooleanExpression::Or(
            Box::new(parse_inner(left)?),
            Box::new(parse_inner(right)?),
        ));
    }
    if let Some((left, right)) = split_top_level(expression, " and ") {
        return Ok(BooleanExpression::And(
            Box::new(parse_inner(left)?),
            Box::new(parse_inner(right)?),
        ));
    }
    for (lexical, operator) in [
        (" <= ", BooleanComparison::LessThanOrEqual),
        (" >= ", BooleanComparison::GreaterThanOrEqual),
        (" != ", BooleanComparison::NotEqual),
        (" eq ", BooleanComparison::Equal),
        (" ne ", BooleanComparison::NotEqual),
        (" lt ", BooleanComparison::LessThan),
        (" le ", BooleanComparison::LessThanOrEqual),
        (" gt ", BooleanComparison::GreaterThan),
        (" ge ", BooleanComparison::GreaterThanOrEqual),
        (" = ", BooleanComparison::Equal),
        (" < ", BooleanComparison::LessThan),
        (" > ", BooleanComparison::GreaterThan),
    ] {
        if let Some((left, right)) = split_top_level(expression, lexical) {
            return Ok(BooleanExpression::Compare {
                left: Box::new(parse_inner(left)?),
                operator,
                right: Box::new(parse_inner(right)?),
            });
        }
    }
    if matches!(expression, "true()" | "fn:true()") {
        return Ok(BooleanExpression::Constant(true));
    }
    if matches!(expression, "false()" | "fn:false()") {
        return Ok(BooleanExpression::Constant(false));
    }
    if has_nonzero_arity(expression, "true")
        || has_nonzero_arity(expression, "fn:true")
        || has_nonzero_arity(expression, "false")
        || has_nonzero_arity(expression, "fn:false")
    {
        return Err(BooleanParseFailure::InvalidArity);
    }
    if let Some(inner) = function_argument(
        expression,
        &["not", "fn:not", "boolean", "fn:boolean", "xs:boolean"],
    ) {
        if inner.trim().is_empty() || split_top_level_once(inner).is_some() {
            return Err(BooleanParseFailure::InvalidArity);
        }
        let inner = match parse_effective_boolean_value(inner) {
            Some(value) => value.map(BooleanExpression::Constant).map_err(
                |EffectiveBooleanValueFailure::InvalidTypeOrCardinality| {
                    BooleanParseFailure::InvalidEffectiveBooleanValue
                },
            )?,
            None => parse_inner(inner)?,
        };
        return if expression.trim_start().starts_with("not(")
            || expression.trim_start().starts_with("fn:not(")
        {
            Ok(BooleanExpression::Not(Box::new(inner)))
        } else {
            Ok(inner)
        };
    }
    if let Some(inner) = function_argument(expression, &["empty", "fn:empty"]) {
        if inner.trim().is_empty() || split_top_level_once(inner).is_some() {
            return Err(BooleanParseFailure::InvalidArity);
        }
        return parse_sequence(inner)
            .map(|sequence| BooleanExpression::Constant(sequence.is_empty()))
            .ok_or(BooleanParseFailure::Unsupported);
    }
    if let Some(value) = parse_effective_boolean_value(expression) {
        return value.map(BooleanExpression::Constant).map_err(
            |EffectiveBooleanValueFailure::InvalidTypeOrCardinality| {
                BooleanParseFailure::InvalidEffectiveBooleanValue
            },
        );
    }
    Err(BooleanParseFailure::Unsupported)
}

pub(crate) fn evaluate(
    expression: &BooleanExpression,
    control: &mut InvocationControl,
) -> Result<bool, BooleanEvaluationFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(BooleanEvaluationFailure::Control)?;
    match expression {
        BooleanExpression::Constant(value) => Ok(*value),
        BooleanExpression::Not(inner) => evaluate(inner, control).map(|value| !value),
        BooleanExpression::And(left, right) => {
            let left = evaluate(left, control)?;
            if !left {
                return Ok(false);
            }
            evaluate(right, control)
        }
        BooleanExpression::Or(left, right) => {
            let left = evaluate(left, control)?;
            if left {
                return Ok(true);
            }
            evaluate(right, control)
        }
        BooleanExpression::Compare {
            left,
            operator,
            right,
        } => {
            let left = evaluate(left, control)?;
            let right = evaluate(right, control)?;
            Ok(compare(left, *operator, right))
        }
    }
}

pub(crate) fn evaluate_scalar(
    expression: &ScalarExpression,
    control: &mut InvocationControl,
) -> Result<ScalarValue, BooleanEvaluationFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(BooleanEvaluationFailure::Control)?;
    match expression {
        ScalarExpression::Boolean(boolean) => evaluate(boolean, control).map(ScalarValue::Boolean),
        ScalarExpression::BooleanString(boolean) => evaluate(boolean, control)
            .map(|value| ScalarValue::String(if value { "true" } else { "false" }.to_owned())),
        ScalarExpression::Concat(left, right) => {
            let mut left = evaluate_string(left, control)?;
            left.push_str(&evaluate_string(right, control)?);
            Ok(ScalarValue::String(left))
        }
        ScalarExpression::Contains(value, search) => {
            let value = evaluate_string(value, control)?;
            let search = evaluate_string(search, control)?;
            Ok(ScalarValue::Boolean(value.contains(&search)))
        }
        ScalarExpression::StringLength(value) => {
            evaluate_string(value, control).map(|value| ScalarValue::Integer(value.chars().count()))
        }
    }
}

fn evaluate_string(
    expression: &ScalarExpression,
    control: &mut InvocationControl,
) -> Result<String, BooleanEvaluationFailure> {
    match evaluate_scalar(expression, control)? {
        ScalarValue::String(value) => Ok(value),
        ScalarValue::Boolean(value) => Ok(if value { "true" } else { "false" }.to_owned()),
        ScalarValue::Integer(value) => Ok(value.to_string()),
    }
}

fn compare(left: bool, operator: BooleanComparison, right: bool) -> bool {
    match operator {
        BooleanComparison::Equal => left == right,
        BooleanComparison::NotEqual => left != right,
        BooleanComparison::LessThan => !left && right,
        BooleanComparison::LessThanOrEqual => !left || right,
        BooleanComparison::GreaterThan => left && !right,
        BooleanComparison::GreaterThanOrEqual => left || !right,
    }
}

fn has_nonzero_arity(expression: &str, name: &str) -> bool {
    expression
        .strip_prefix(name)
        .and_then(|tail| tail.strip_prefix('('))
        .and_then(|tail| tail.strip_suffix(')'))
        .is_some_and(|argument| !argument.trim().is_empty())
}

fn function_argument<'a>(expression: &'a str, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| {
        expression
            .strip_prefix(name)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
            .filter(|inner| balanced(inner))
    })
}

fn strip_balanced_parentheses(mut expression: &str) -> &str {
    loop {
        let Some(inner) = expression
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
        else {
            return expression;
        };
        if !balanced(inner) {
            return expression;
        }
        expression = inner.trim();
    }
}

fn balanced(expression: &str) -> bool {
    let mut depth = 0usize;
    for character in expression.chars() {
        match character {
            '(' => depth += 1,
            ')' => {
                let Some(next) = depth.checked_sub(1) else {
                    return false;
                };
                depth = next;
            }
            _ => {}
        }
    }
    depth == 0
}

fn split_top_level<'a>(expression: &'a str, operator: &str) -> Option<(&'a str, &'a str)> {
    let mut depth = 0usize;
    for (index, character) in expression.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            _ if depth == 0 && expression[index..].starts_with(operator) => {
                let right = index + operator.len();
                return Some((expression[..index].trim(), expression[right..].trim()));
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{BooleanParseFailure, ScalarValue, evaluate, evaluate_scalar, parse, parse_scalar};
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};

    #[test]
    fn evaluates_boolean_constants_composition_and_comparisons() {
        for (source, expected) in [
            ("fn:true()", true),
            ("not(false())", true),
            ("true() and false()", false),
            ("false() or true()", true),
            ("false() lt true()", true),
            ("true() >= false()", true),
            ("xs:boolean(fn:false())", false),
            ("fn:not(xs:int(\"0\"))", true),
            ("not(xs:double('NaN'))", true),
            ("not(xs:anyURI(\"example.com/\"))", false),
            ("fn:not(())", true),
            ("fn:boolean(xs:float('NaN'))", false),
            ("boolean(xs:untypedAtomic(\"string\"))", true),
            ("boolean(string(false()))", true),
            ("not(empty(((), 1, 2)))", true),
        ] {
            let expression = parse(source).expect("parse admitted boolean expression");
            let mut control = InvocationControl::unbounded();
            assert_eq!(
                evaluate(&expression, &mut control),
                Ok(expected),
                "{source}"
            );
            assert!(control.consumed(WorkDomain::XPathOperation) > 0);
        }
    }

    #[test]
    fn distinguishes_invalid_arity_from_unsupported_syntax() {
        assert_eq!(parse("true(1)"), Err(BooleanParseFailure::InvalidArity));
        assert_eq!(parse("not()"), Err(BooleanParseFailure::InvalidArity));
        assert_eq!(parse("not(1, 2)"), Err(BooleanParseFailure::InvalidArity));
        assert_eq!(
            parse("boolean((1, 2))"),
            Err(BooleanParseFailure::InvalidEffectiveBooleanValue)
        );
        assert_eq!(
            parse("boolean(xs:dateTime(\"1999-12-31T00:00:00\"))"),
            Err(BooleanParseFailure::InvalidEffectiveBooleanValue)
        );
        assert_eq!(
            parse("contains('a', 'a')"),
            Err(BooleanParseFailure::Unsupported)
        );
    }

    #[test]
    fn projects_boolean_constants_through_bounded_string_functions() {
        for (source, expected) in [
            (
                "fn:string(fn:true())",
                ScalarValue::String("true".to_owned()),
            ),
            (
                "fn:concat(xs:string(false()),xs:string(false()))",
                ScalarValue::String("falsefalse".to_owned()),
            ),
            (
                "fn:contains(xs:string(true()),xs:string(true()))",
                ScalarValue::Boolean(true),
            ),
            (
                "fn:string-length(xs:string(false()))",
                ScalarValue::Integer(5),
            ),
        ] {
            let expression = parse_scalar(source).expect("parse scalar projection");
            let mut control = InvocationControl::unbounded();
            assert_eq!(
                evaluate_scalar(&expression, &mut control),
                Ok(expected),
                "{source}"
            );
        }
    }
}
