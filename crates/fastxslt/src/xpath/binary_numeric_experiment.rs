//! Private checked-integer arithmetic over two typed location paths.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};

use super::path_experiment::{LocationPath, evaluate_location_path_controlled};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BinaryNumericOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericOperandSelection {
    FirstInDocumentOrder,
    ZeroOrOne,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BinaryNumericExpression {
    pub(crate) left: LocationPath,
    pub(crate) negate_left: bool,
    pub(crate) operator: BinaryNumericOperator,
    pub(crate) right: LocationPath,
    pub(crate) negate_right: bool,
    pub(crate) selection: NumericOperandSelection,
    pub(crate) location: SourceLocation,
}

impl BinaryNumericExpression {
    #[cfg(feature = "workbench")]
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        self.left.known_owned_capacity_bytes()
            + self.right.known_owned_capacity_bytes()
            + self.location.resource.capacity()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BinaryNumericEvaluationFailure {
    Control(ControlFailure),
    Cardinality,
    EmptyOperand,
    UnsupportedLexical,
    DivisionByZero,
    NonIntegral,
    Overflow,
}

pub(crate) fn split_paths(expression: &str) -> Option<(&str, BinaryNumericOperator, &str)> {
    let expression = expression.trim();
    let mut depth = 0usize;
    let mut candidate = None;
    for (index, character) in expression.char_indices() {
        match character {
            '(' => depth = depth.checked_add(1)?,
            ')' => depth = depth.checked_sub(1)?,
            '+' | '-' | '*' if depth == 0 => {
                if character == '*'
                    && (expression[..index].ends_with('/')
                        || expression[index + 1..].starts_with('/'))
                {
                    continue;
                }
                if character == '-'
                    && !(expression[..index].ends_with(char::is_whitespace)
                        && (expression[index + 1..].starts_with(char::is_whitespace)
                            || expression[index + 1..].starts_with('-')))
                {
                    continue;
                }
                if candidate.is_some() {
                    return None;
                }
                candidate = Some((
                    index,
                    match character {
                        '+' => BinaryNumericOperator::Add,
                        '-' => BinaryNumericOperator::Subtract,
                        '*' => BinaryNumericOperator::Multiply,
                        _ => unreachable!(),
                    },
                    1,
                ));
            }
            'd' if depth == 0
                && expression[index..].starts_with("div")
                && expression[..index].ends_with(char::is_whitespace)
                && expression[index + 3..].starts_with(char::is_whitespace) =>
            {
                if candidate.is_some() {
                    return None;
                }
                candidate = Some((index, BinaryNumericOperator::Divide, 3));
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    let (index, operator, operator_width) = candidate?;
    let right_index = index + operator_width;
    let left = strip_balanced_parentheses(expression[..index].trim());
    let right = strip_balanced_parentheses(expression[right_index..].trim());
    (!left.is_empty() && !right.is_empty()).then_some((left, operator, right))
}

pub(crate) fn signed_path(expression: &str) -> Option<(&str, bool)> {
    let expression = expression.trim();
    let (expression, negate) = expression
        .strip_prefix('-')
        .map_or((expression, false), |path| (path.trim(), true));
    let expression = strip_balanced_parentheses(expression);
    (!expression.is_empty()).then_some((expression, negate))
}

pub(crate) fn evaluate(
    expression: &BinaryNumericExpression,
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<String, BinaryNumericEvaluationFailure> {
    let left = operand_value(
        document,
        context,
        &expression.left,
        expression.negate_left,
        expression.selection,
        control,
    )?;
    let right = operand_value(
        document,
        context,
        &expression.right,
        expression.negate_right,
        expression.selection,
        control,
    )?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(BinaryNumericEvaluationFailure::Control)?;
    let value = apply_operator(left, expression.operator, right)?;
    Ok(value.to_string())
}

fn apply_operator(
    left: i128,
    operator: BinaryNumericOperator,
    right: i128,
) -> Result<i128, BinaryNumericEvaluationFailure> {
    match operator {
        BinaryNumericOperator::Add => left
            .checked_add(right)
            .ok_or(BinaryNumericEvaluationFailure::Overflow),
        BinaryNumericOperator::Subtract => left
            .checked_sub(right)
            .ok_or(BinaryNumericEvaluationFailure::Overflow),
        BinaryNumericOperator::Multiply => left
            .checked_mul(right)
            .ok_or(BinaryNumericEvaluationFailure::Overflow),
        BinaryNumericOperator::Divide if right == 0 => {
            Err(BinaryNumericEvaluationFailure::DivisionByZero)
        }
        BinaryNumericOperator::Divide => {
            let remainder = left
                .checked_rem(right)
                .ok_or(BinaryNumericEvaluationFailure::Overflow)?;
            if remainder != 0 {
                return Err(BinaryNumericEvaluationFailure::NonIntegral);
            }
            left.checked_div(right)
                .ok_or(BinaryNumericEvaluationFailure::Overflow)
        }
    }
}

fn operand_value(
    document: &Document,
    context: NodeId,
    path: &LocationPath,
    negate: bool,
    selection: NumericOperandSelection,
    control: &mut InvocationControl,
) -> Result<i128, BinaryNumericEvaluationFailure> {
    let selected = evaluate_location_path_controlled(document, context, path, control)
        .map_err(BinaryNumericEvaluationFailure::Control)?;
    if selection == NumericOperandSelection::ZeroOrOne && selected.len() > 1 {
        return Err(BinaryNumericEvaluationFailure::Cardinality);
    }
    let node = selected
        .first()
        .copied()
        .ok_or(BinaryNumericEvaluationFailure::EmptyOperand)?;
    let lexical = document
        .string_value_controlled(node, control)
        .map_err(BinaryNumericEvaluationFailure::Control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(BinaryNumericEvaluationFailure::Control)?;
    let value = lexical
        .trim()
        .parse::<i128>()
        .map_err(|_| BinaryNumericEvaluationFailure::UnsupportedLexical)?;
    if negate {
        value
            .checked_neg()
            .ok_or(BinaryNumericEvaluationFailure::Overflow)
    } else {
        Ok(value)
    }
}

fn strip_balanced_parentheses(mut expression: &str) -> &str {
    loop {
        let Some(inner) = expression
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
        else {
            return expression;
        };
        let mut depth = 0usize;
        let balanced = inner.chars().all(|character| match character {
            '(' => {
                depth += 1;
                true
            }
            ')' => {
                let Some(next) = depth.checked_sub(1) else {
                    return false;
                };
                depth = next;
                true
            }
            _ => true,
        }) && depth == 0;
        if !balanced {
            return expression;
        }
        expression = inner.trim();
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BinaryNumericEvaluationFailure, BinaryNumericOperator, apply_operator, signed_path,
        split_paths,
    };

    #[test]
    fn splits_only_one_bounded_top_level_operator() {
        assert_eq!(
            split_paths("n1+n2"),
            Some(("n1", BinaryNumericOperator::Add, "n2"))
        );
        assert_eq!(
            split_paths("(n1/@a) * (n2/@a)"),
            Some(("n1/@a", BinaryNumericOperator::Multiply, "n2/@a"))
        );
        assert_eq!(
            split_paths("n-2 - n-1"),
            Some(("n-2", BinaryNumericOperator::Subtract, "n-1"))
        );
        assert_eq!(
            split_paths("div div mod"),
            Some(("div", BinaryNumericOperator::Divide, "mod"))
        );
        assert_eq!(
            split_paths("-n-2 --n-1"),
            Some(("-n-2", BinaryNumericOperator::Subtract, "-n-1"))
        );
        for expression in ["n1", "n1+n2+n3", "(n1+n2)", "n1/ *", "/*/"] {
            assert_eq!(split_paths(expression), None, "{expression}");
        }
    }

    #[test]
    fn separates_unary_signs_from_hyphenated_path_names() {
        assert_eq!(signed_path("n-2"), Some(("n-2", false)));
        assert_eq!(signed_path("-n-2"), Some(("n-2", true)));
        assert_eq!(signed_path("-(n-2/@a)"), Some(("n-2/@a", true)));
    }

    #[test]
    fn exact_division_refuses_fractional_and_zero_results() {
        assert_eq!(apply_operator(8, BinaryNumericOperator::Divide, 4), Ok(2));
        assert_eq!(
            apply_operator(7, BinaryNumericOperator::Divide, 4),
            Err(BinaryNumericEvaluationFailure::NonIntegral)
        );
        assert_eq!(
            apply_operator(7, BinaryNumericOperator::Divide, 0),
            Err(BinaryNumericEvaluationFailure::DivisionByZero)
        );
    }
}
