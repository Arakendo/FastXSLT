//! Private exact-rational arithmetic over typed location paths.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};

use super::path_experiment::{LocationPath, evaluate_location_path_controlled};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BinaryNumericOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericOperandSelection {
    FirstInDocumentOrder,
    ZeroOrOne,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BinaryNumericExpression {
    pub(crate) root: BinaryNumericNode,
    pub(crate) selection: NumericOperandSelection,
    pub(crate) location: SourceLocation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BinaryNumericNode {
    Path {
        path: LocationPath,
        negate: bool,
    },
    Literal(ExactRational),
    Variable(String),
    Negate(Box<Self>),
    Operation {
        left: Box<Self>,
        operator: BinaryNumericOperator,
        right: Box<Self>,
    },
}

impl BinaryNumericExpression {
    #[cfg(feature = "workbench")]
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        self.root.known_owned_capacity_bytes() + self.location.resource.capacity()
    }
}

impl BinaryNumericNode {
    #[cfg(feature = "workbench")]
    fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::Path { path, .. } => path.known_owned_capacity_bytes(),
            Self::Variable(name) => name.capacity(),
            Self::Literal(_) => 0,
            Self::Negate(operand) => operand.known_owned_capacity_bytes(),
            Self::Operation { left, right, .. } => {
                left.known_owned_capacity_bytes() + right.known_owned_capacity_bytes()
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BinaryNumericEvaluationFailure<VariableFailure = ()> {
    Control(ControlFailure),
    Cardinality,
    EmptyOperand,
    UnsupportedLexical,
    ZeroDivisor,
    NonIntegral,
    Overflow,
    Variable(VariableFailure),
}

pub(crate) fn split_paths(expression: &str) -> Option<(&str, BinaryNumericOperator, &str)> {
    let expression = strip_balanced_parentheses(expression.trim());
    let bytes = expression.as_bytes();
    let mut depth = 0usize;
    let mut additive_candidate = None;
    let mut multiplicative_candidate = None;
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
                    && !is_left_spaced_binary_minus(expression, index)
                    && !(bytes[..index].last().is_some_and(u8::is_ascii_digit)
                        && bytes[index + 1..].first().is_some_and(|byte| {
                            byte.is_ascii_alphanumeric() || matches!(byte, b'$' | b'_')
                        }))
                    && !(expression[..index].ends_with([')', ']'])
                        && expression[index + 1..].starts_with(['(', '[']))
                {
                    continue;
                }
                let candidate = (
                    index,
                    match character {
                        '+' => BinaryNumericOperator::Add,
                        '-' => BinaryNumericOperator::Subtract,
                        '*' => BinaryNumericOperator::Multiply,
                        _ => unreachable!(),
                    },
                    1,
                );
                if matches!(character, '+' | '-') {
                    additive_candidate = Some(candidate);
                } else {
                    multiplicative_candidate = Some(candidate);
                }
            }
            'd' if depth == 0
                && expression[index..].starts_with("div")
                && operator_keyword_left_boundary(&expression[..index])
                && expression[index + 3..].starts_with(char::is_whitespace) =>
            {
                multiplicative_candidate = Some((index, BinaryNumericOperator::Divide, 3));
            }
            'm' if depth == 0
                && expression[index..].starts_with("mod")
                && operator_keyword_left_boundary(&expression[..index])
                && expression[index + 3..].starts_with(char::is_whitespace) =>
            {
                multiplicative_candidate = Some((index, BinaryNumericOperator::Modulo, 3));
            }
            _ => {}
        }
    }
    if depth != 0 {
        return None;
    }
    let (index, operator, operator_width) = additive_candidate.or(multiplicative_candidate)?;
    let right_index = index + operator_width;
    let left = strip_balanced_parentheses(expression[..index].trim());
    let right = strip_balanced_parentheses(expression[right_index..].trim());
    (!left.is_empty() && !right.is_empty()).then_some((left, operator, right))
}

fn is_left_spaced_binary_minus(expression: &str, index: usize) -> bool {
    let left = &expression[..index];
    if !left.ends_with(char::is_whitespace) {
        return false;
    }
    left.trim_end()
        .chars()
        .next_back()
        .is_some_and(|character| !matches!(character, '+' | '-' | '*' | '/' | '('))
}

fn operator_keyword_left_boundary(left: &str) -> bool {
    left.ends_with(char::is_whitespace) || left.ends_with([')', ']'])
}

pub(crate) fn signed_path(expression: &str) -> Option<(&str, bool)> {
    let expression = expression.trim();
    let (expression, negate) = expression
        .strip_prefix('-')
        .map_or((expression, false), |path| (path.trim(), true));
    let expression = strip_balanced_parentheses(expression);
    (!expression.is_empty()).then_some((expression, negate))
}

pub(crate) fn evaluate_with_variables<VariableFailure>(
    expression: &BinaryNumericExpression,
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
    mut resolve_variable: impl FnMut(&str, &mut InvocationControl) -> Result<String, VariableFailure>,
) -> Result<String, BinaryNumericEvaluationFailure<VariableFailure>> {
    let value = evaluate_node(
        &expression.root,
        document,
        context,
        expression.selection,
        control,
        &mut resolve_variable,
    )?;
    value.format_decimal().map_err(lift_failure)
}

fn lift_failure<VariableFailure>(
    failure: BinaryNumericEvaluationFailure,
) -> BinaryNumericEvaluationFailure<VariableFailure> {
    match failure {
        BinaryNumericEvaluationFailure::Control(failure) => {
            BinaryNumericEvaluationFailure::Control(failure)
        }
        BinaryNumericEvaluationFailure::Cardinality => BinaryNumericEvaluationFailure::Cardinality,
        BinaryNumericEvaluationFailure::EmptyOperand => {
            BinaryNumericEvaluationFailure::EmptyOperand
        }
        BinaryNumericEvaluationFailure::UnsupportedLexical => {
            BinaryNumericEvaluationFailure::UnsupportedLexical
        }
        BinaryNumericEvaluationFailure::ZeroDivisor => BinaryNumericEvaluationFailure::ZeroDivisor,
        BinaryNumericEvaluationFailure::NonIntegral => BinaryNumericEvaluationFailure::NonIntegral,
        BinaryNumericEvaluationFailure::Overflow => BinaryNumericEvaluationFailure::Overflow,
        BinaryNumericEvaluationFailure::Variable(()) => {
            unreachable!("a variable-free numeric operation cannot report variable failure")
        }
    }
}

fn evaluate_node<VariableFailure>(
    node: &BinaryNumericNode,
    document: &Document,
    context: NodeId,
    selection: NumericOperandSelection,
    control: &mut InvocationControl,
    resolve_variable: &mut impl FnMut(&str, &mut InvocationControl) -> Result<String, VariableFailure>,
) -> Result<ExactRational, BinaryNumericEvaluationFailure<VariableFailure>> {
    match node {
        BinaryNumericNode::Path { path, negate } => {
            operand_value(document, context, path, *negate, selection, control)
                .map_err(lift_failure)
        }
        BinaryNumericNode::Literal(value) => Ok(*value),
        BinaryNumericNode::Variable(name) => {
            let lexical = resolve_variable(name, control)
                .map_err(BinaryNumericEvaluationFailure::Variable)?;
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(BinaryNumericEvaluationFailure::Control)?;
            ExactRational::parse_decimal(&lexical)
                .ok_or(BinaryNumericEvaluationFailure::UnsupportedLexical)
        }
        BinaryNumericNode::Negate(operand) => evaluate_node(
            operand,
            document,
            context,
            selection,
            control,
            resolve_variable,
        )?
        .checked_negate()
        .map_err(lift_failure),
        BinaryNumericNode::Operation {
            left,
            operator,
            right,
        } => {
            let left = evaluate_node(
                left,
                document,
                context,
                selection,
                control,
                resolve_variable,
            )?;
            let right = evaluate_node(
                right,
                document,
                context,
                selection,
                control,
                resolve_variable,
            )?;
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(BinaryNumericEvaluationFailure::Control)?;
            apply_operator(left, *operator, right).map_err(lift_failure)
        }
    }
}

fn apply_operator(
    left: ExactRational,
    operator: BinaryNumericOperator,
    right: ExactRational,
) -> Result<ExactRational, BinaryNumericEvaluationFailure> {
    match operator {
        BinaryNumericOperator::Add => left.checked_add(right),
        BinaryNumericOperator::Subtract => left.checked_subtract(right),
        BinaryNumericOperator::Multiply => left.checked_multiply(right),
        BinaryNumericOperator::Divide | BinaryNumericOperator::Modulo if right.numerator == 0 => {
            Err(BinaryNumericEvaluationFailure::ZeroDivisor)
        }
        BinaryNumericOperator::Divide => left.checked_divide(right),
        BinaryNumericOperator::Modulo => {
            if left.denominator != 1 || right.denominator != 1 {
                return Err(BinaryNumericEvaluationFailure::NonIntegral);
            }
            left.numerator
                .checked_rem(right.numerator)
                .map(ExactRational::integer)
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
) -> Result<ExactRational, BinaryNumericEvaluationFailure> {
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
    let value = ExactRational::parse_decimal(&lexical)
        .ok_or(BinaryNumericEvaluationFailure::UnsupportedLexical)?;
    if negate {
        value.checked_negate()
    } else {
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ExactRational {
    numerator: i128,
    denominator: i128,
}

impl ExactRational {
    const fn integer(value: i128) -> Self {
        Self {
            numerator: value,
            denominator: 1,
        }
    }

    pub(crate) fn parse_decimal(value: &str) -> Option<Self> {
        let value = value.trim();
        let (negative, unsigned) = value.strip_prefix('-').map_or_else(
            || {
                value
                    .strip_prefix('+')
                    .map_or((false, value), |rest| (false, rest))
            },
            |rest| (true, rest),
        );
        let (whole, fractional) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        if (whole.is_empty() && fractional.is_empty())
            || !whole.bytes().all(|byte| byte.is_ascii_digit())
            || !fractional.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        let numerator =
            whole
                .bytes()
                .chain(fractional.bytes())
                .try_fold(0_i128, |value, digit| {
                    value
                        .checked_mul(10)?
                        .checked_add(i128::from(digit.checked_sub(b'0')?))
                })?;
        let numerator = if negative {
            numerator.checked_neg()?
        } else {
            numerator
        };
        let scale = u32::try_from(fractional.len()).ok()?;
        Self::normalized(numerator, 10_i128.checked_pow(scale)?)
    }

    fn normalized(numerator: i128, denominator: i128) -> Option<Self> {
        if denominator == 0 {
            return None;
        }
        let (numerator, denominator) = if denominator < 0 {
            (numerator.checked_neg()?, denominator.checked_neg()?)
        } else {
            (numerator, denominator)
        };
        let denominator = u128::try_from(denominator).ok()?;
        let divisor = greatest_common_divisor(numerator.unsigned_abs(), denominator);
        let divisor = i128::try_from(divisor).ok()?;
        let numerator = numerator.checked_div(divisor)?;
        let denominator = i128::try_from(denominator).ok()?.checked_div(divisor)?;
        Some(Self {
            numerator,
            denominator,
        })
    }

    fn checked_add(self, right: Self) -> Result<Self, BinaryNumericEvaluationFailure> {
        let numerator = self
            .numerator
            .checked_mul(right.denominator)
            .and_then(|left| {
                right
                    .numerator
                    .checked_mul(self.denominator)
                    .and_then(|right| left.checked_add(right))
            })
            .ok_or(BinaryNumericEvaluationFailure::Overflow)?;
        let denominator = self
            .denominator
            .checked_mul(right.denominator)
            .ok_or(BinaryNumericEvaluationFailure::Overflow)?;
        Self::normalized(numerator, denominator).ok_or(BinaryNumericEvaluationFailure::Overflow)
    }

    fn checked_subtract(self, right: Self) -> Result<Self, BinaryNumericEvaluationFailure> {
        self.checked_add(right.checked_negate()?)
    }

    pub(crate) fn checked_multiply(
        self,
        right: Self,
    ) -> Result<Self, BinaryNumericEvaluationFailure> {
        let numerator = self
            .numerator
            .checked_mul(right.numerator)
            .ok_or(BinaryNumericEvaluationFailure::Overflow)?;
        let denominator = self
            .denominator
            .checked_mul(right.denominator)
            .ok_or(BinaryNumericEvaluationFailure::Overflow)?;
        Self::normalized(numerator, denominator).ok_or(BinaryNumericEvaluationFailure::Overflow)
    }

    pub(crate) fn checked_divide(
        self,
        right: Self,
    ) -> Result<Self, BinaryNumericEvaluationFailure> {
        let numerator = self
            .numerator
            .checked_mul(right.denominator)
            .ok_or(BinaryNumericEvaluationFailure::Overflow)?;
        let denominator = self
            .denominator
            .checked_mul(right.numerator)
            .ok_or(BinaryNumericEvaluationFailure::Overflow)?;
        Self::normalized(numerator, denominator).ok_or(BinaryNumericEvaluationFailure::Overflow)
    }

    fn checked_negate(self) -> Result<Self, BinaryNumericEvaluationFailure> {
        Ok(Self {
            numerator: self
                .numerator
                .checked_neg()
                .ok_or(BinaryNumericEvaluationFailure::Overflow)?,
            denominator: self.denominator,
        })
    }

    pub(crate) fn format_decimal(self) -> Result<String, BinaryNumericEvaluationFailure> {
        if self.denominator == 1 {
            return Ok(self.numerator.to_string());
        }
        let mut denominator = self.denominator;
        let mut scale = 0_u32;
        while denominator % 2 == 0 {
            denominator /= 2;
            scale += 1;
        }
        while denominator % 5 == 0 {
            denominator /= 5;
            scale += 1;
        }
        if denominator != 1 {
            return Err(BinaryNumericEvaluationFailure::NonIntegral);
        }
        let power = 10_i128
            .checked_pow(scale)
            .ok_or(BinaryNumericEvaluationFailure::Overflow)?;
        let units = self
            .numerator
            .checked_mul(power / self.denominator)
            .ok_or(BinaryNumericEvaluationFailure::Overflow)?;
        let negative = units.is_negative();
        let magnitude = units
            .checked_abs()
            .ok_or(BinaryNumericEvaluationFailure::Overflow)?;
        let mut digits = magnitude.to_string();
        let scale = usize::try_from(scale).map_err(|_| BinaryNumericEvaluationFailure::Overflow)?;
        if digits.len() <= scale {
            digits.insert_str(0, &"0".repeat(scale + 1 - digits.len()));
        }
        let split = digits.len() - scale;
        let (whole, fractional) = digits.split_at(split);
        let fractional = fractional.trim_end_matches('0');
        let mut formatted = format!("{}{whole}", if negative { "-" } else { "" });
        if !fractional.is_empty() {
            formatted.push('.');
            formatted.push_str(fractional);
        }
        Ok(formatted)
    }
}

fn greatest_common_divisor(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left.max(1)
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
        BinaryNumericEvaluationFailure, BinaryNumericOperator, ExactRational, apply_operator,
        signed_path, split_paths,
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
            split_paths("div mod mod"),
            Some(("div", BinaryNumericOperator::Modulo, "mod"))
        );
        assert_eq!(
            split_paths("(n1*n2)div n3"),
            Some(("n1*n2", BinaryNumericOperator::Divide, "n3"))
        );
        assert_eq!(
            split_paths("-n-2 --n-1"),
            Some(("-n-2", BinaryNumericOperator::Subtract, "-n-1"))
        );
        assert_eq!(
            split_paths("n1*n2*n3"),
            Some(("n1*n2", BinaryNumericOperator::Multiply, "n3"))
        );
        assert_eq!(
            split_paths("n1*n2+n3*n4"),
            Some(("n1*n2", BinaryNumericOperator::Add, "n3*n4"))
        );
        assert_eq!(
            split_paths("4-6"),
            Some(("4", BinaryNumericOperator::Subtract, "6"))
        );
        assert_eq!(
            split_paths("(n1+n2)-(n3+n4)"),
            Some(("n1+n2", BinaryNumericOperator::Subtract, "n3+n4"))
        );
        assert_eq!(
            split_paths("100-n6 -4-n1 -1-11"),
            Some(("100-n6 -4-n1 -1", BinaryNumericOperator::Subtract, "11"))
        );
        assert_eq!(
            split_paths("$anum*5-4*n2+n6*n1 -n3*3"),
            Some((
                "$anum*5-4*n2+n6*n1",
                BinaryNumericOperator::Subtract,
                "n3*3"
            ))
        );
        for expression in ["n1", "(n1)", "n1/ *", "/*/"] {
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
    fn exact_division_retains_fractional_values_and_refuses_zero() {
        let integer = ExactRational::integer;
        assert_eq!(
            apply_operator(integer(8), BinaryNumericOperator::Divide, integer(4)),
            Ok(integer(2))
        );
        assert_eq!(
            apply_operator(integer(7), BinaryNumericOperator::Divide, integer(4))
                .and_then(ExactRational::format_decimal),
            Ok("1.75".to_owned())
        );
        assert_eq!(
            apply_operator(integer(7), BinaryNumericOperator::Divide, integer(0)),
            Err(BinaryNumericEvaluationFailure::ZeroDivisor)
        );
        assert_eq!(
            apply_operator(integer(7), BinaryNumericOperator::Modulo, integer(4)),
            Ok(integer(3))
        );
        assert_eq!(
            apply_operator(integer(7), BinaryNumericOperator::Modulo, integer(0)),
            Err(BinaryNumericEvaluationFailure::ZeroDivisor)
        );
    }

    #[test]
    fn parses_and_formats_exact_decimal_lexicals() {
        for (lexical, formatted) in [
            (".125", "0.125"),
            (".5", "0.5"),
            (".2", "0.2"),
            ("-01.2500", "-1.25"),
        ] {
            assert_eq!(
                ExactRational::parse_decimal(lexical).and_then(|value| value.format_decimal().ok()),
                Some(formatted.to_owned())
            );
        }
        for lexical in ["", ".", "-+1", "+-1", "1.2.3", "NaN"] {
            assert_eq!(ExactRational::parse_decimal(lexical), None, "{lexical}");
        }
    }
}
