//! Private checked-integer arithmetic over two typed location paths.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};

use super::path_experiment::{LocationPath, evaluate_location_path_controlled};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BinaryNumericOperator {
    Add,
    Multiply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericOperandSelection {
    FirstInDocumentOrder,
    ZeroOrOne,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BinaryNumericExpression {
    pub(crate) left: LocationPath,
    pub(crate) operator: BinaryNumericOperator,
    pub(crate) right: LocationPath,
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
            '+' | '*' if depth == 0 => {
                if character == '*'
                    && (expression[..index].ends_with('/')
                        || expression[index + 1..].starts_with('/'))
                {
                    continue;
                }
                if candidate.is_some() {
                    return None;
                }
                candidate = Some((
                    index,
                    if character == '+' {
                        BinaryNumericOperator::Add
                    } else {
                        BinaryNumericOperator::Multiply
                    },
                ));
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    let (index, operator) = candidate?;
    let right_index = index + expression[index..].chars().next()?.len_utf8();
    let left = strip_balanced_parentheses(expression[..index].trim());
    let right = strip_balanced_parentheses(expression[right_index..].trim());
    (!left.is_empty() && !right.is_empty()).then_some((left, operator, right))
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
        expression.selection,
        control,
    )?;
    let right = operand_value(
        document,
        context,
        &expression.right,
        expression.selection,
        control,
    )?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(BinaryNumericEvaluationFailure::Control)?;
    let value = match expression.operator {
        BinaryNumericOperator::Add => left.checked_add(right),
        BinaryNumericOperator::Multiply => left.checked_mul(right),
    }
    .ok_or(BinaryNumericEvaluationFailure::Overflow)?;
    Ok(value.to_string())
}

fn operand_value(
    document: &Document,
    context: NodeId,
    path: &LocationPath,
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
    lexical
        .trim()
        .parse::<i128>()
        .map_err(|_| BinaryNumericEvaluationFailure::UnsupportedLexical)
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
    use super::{BinaryNumericOperator, split_paths};

    #[test]
    fn splits_only_one_top_level_addition_or_multiplication() {
        assert_eq!(
            split_paths("n1+n2"),
            Some(("n1", BinaryNumericOperator::Add, "n2"))
        );
        assert_eq!(
            split_paths("(n1/@a) * (n2/@a)"),
            Some(("n1/@a", BinaryNumericOperator::Multiply, "n2/@a"))
        );
        for expression in ["n1", "n1+n2+n3", "(n1+n2)", "n1/ *", "/*/"] {
            assert_eq!(split_paths(expression), None, "{expression}");
        }
    }
}
