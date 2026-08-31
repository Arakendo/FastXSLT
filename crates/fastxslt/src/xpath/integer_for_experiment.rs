//! Narrow ordered integer `for` expression used by native XSLT30 `for-002`.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::SourceLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntegerForExpression {
    first_values: Vec<i64>,
    second_values: Vec<i64>,
}

#[cfg(feature = "workbench")]
impl IntegerForExpression {
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        (self.first_values.capacity() + self.second_values.capacity()) * std::mem::size_of::<i64>()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntegerForFailure {
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
}

pub(crate) fn parse(
    expression: &str,
    location: SourceLocation,
) -> Result<IntegerForExpression, IntegerForFailure> {
    let normalized = expression.split_whitespace().collect::<Vec<_>>().join(" ");
    let after_for = normalized
        .strip_prefix("for ")
        .ok_or_else(|| unsupported(&normalized, &location))?;
    let (first_name, first_values, remainder) = parse_binding(after_for, &location)?;
    let remainder = remainder
        .strip_prefix(", ")
        .ok_or_else(|| unsupported(&normalized, &location))?;
    let (second_name, second_values, remainder) = parse_binding(remainder, &location)?;
    let sum = remainder
        .strip_prefix(" return (")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| unsupported(&normalized, &location))?;
    let (left, right) = sum
        .split_once(" + ")
        .ok_or_else(|| unsupported(&normalized, &location))?;
    if left != format!("${first_name}") || right != format!("${second_name}") {
        return Err(unsupported(&normalized, &location));
    }
    if first_values.iter().any(|first| {
        second_values
            .iter()
            .any(|second| first.checked_add(*second).is_none())
    }) {
        return Err(IntegerForFailure {
            detail: "integer addition exceeds the private signed-64-bit value domain".to_owned(),
            location,
        });
    }
    Ok(IntegerForExpression {
        first_values,
        second_values,
    })
}

pub(crate) fn evaluate(
    expression: &IntegerForExpression,
    control: &mut InvocationControl,
) -> Result<Vec<i64>, ControlFailure> {
    let mut values = Vec::new();
    for first in &expression.first_values {
        for second in &expression.second_values {
            control.charge(WorkDomain::XPathOperation, 1)?;
            values.push(
                first
                    .checked_add(*second)
                    .expect("compiled integer for expressions have checked sums"),
            );
        }
    }
    Ok(values)
}

fn parse_binding<'a>(
    input: &'a str,
    location: &SourceLocation,
) -> Result<(String, Vec<i64>, &'a str), IntegerForFailure> {
    let after_dollar = input
        .strip_prefix('$')
        .ok_or_else(|| unsupported(input, location))?;
    let (name, after_name) = after_dollar
        .split_once(" in (")
        .ok_or_else(|| unsupported(input, location))?;
    if !is_ascii_ncname(name) {
        return Err(unsupported(input, location));
    }
    let (values, remainder) = after_name
        .split_once(')')
        .ok_or_else(|| unsupported(input, location))?;
    let values: Vec<i64> = values
        .split(',')
        .map(str::trim)
        .map(str::parse)
        .collect::<Result<_, _>>()
        .map_err(|_| unsupported(input, location))?;
    if values.is_empty() {
        return Err(unsupported(input, location));
    }
    Ok((name.to_owned(), values, remainder))
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    characters.next().is_some_and(|first| {
        (first.is_ascii_alphabetic() || first == '_')
            && characters.all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
            })
    })
}

fn unsupported(expression: &str, location: &SourceLocation) -> IntegerForFailure {
    IntegerForFailure {
        detail: format!(
            "the private slice supports two integer bindings and one ordered addition return: {expression}"
        ),
        location: location.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{evaluate, parse};
    use crate::execution_control_experiment::{
        CancellationToken, ControlFailure, InvocationControl, WorkDomain, WorkLimits,
    };
    use crate::xdm::owned_tree_experiment::SourceLocation;

    #[test]
    fn evaluates_two_bindings_in_input_order_and_charges_each_addition() {
        let expression = parse(
            "for $left in (10, 20), $right in (1, 2) return ($left + $right)",
            SourceLocation {
                resource: "memory:integer-for".to_owned(),
                span: 0..69,
            },
        )
        .expect("admitted integer for-expression should parse");
        let mut control = InvocationControl::unbounded();

        let values = evaluate(&expression, &mut control).expect("expression should evaluate");

        assert_eq!(values, [11, 12, 21, 22]);
        assert_eq!(control.consumed(WorkDomain::XPathOperation), 4);

        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = 3;
        let mut bounded = InvocationControl::new(CancellationToken::new(), limits);
        assert_eq!(
            evaluate(&expression, &mut bounded),
            Err(ControlFailure::BudgetExhausted {
                domain: WorkDomain::XPathOperation,
                limit: 3,
                consumed: 3,
                attempted: 1,
            })
        );
    }
}
