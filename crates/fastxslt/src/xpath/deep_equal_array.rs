//! Safe private reference representation for the admitted XDM-array
//! `deep-equal` literal slice.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ValueSequence(Vec<Value>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Integer(i128),
    Array(Vec<ValueSequence>),
}

const MAX_LITERAL_ARRAY_DEPTH: usize = 64;
const MAX_LITERAL_ARRAY_MEMBERS: usize = 1_024;

pub(super) fn recognizes(left: &str, right: &str) -> bool {
    left.trim_start().starts_with('[') || right.trim_start().starts_with('[')
}

pub(super) fn parse(expression: &str) -> Option<ValueSequence> {
    parse_at_depth(expression, 0)
}

fn parse_at_depth(expression: &str, depth: usize) -> Option<ValueSequence> {
    let expression = expression.trim();
    if expression == "()" {
        return Some(ValueSequence(Vec::new()));
    }
    if let Ok(value) = expression.parse::<i128>() {
        return Some(ValueSequence(vec![Value::Integer(value)]));
    }
    if depth >= MAX_LITERAL_ARRAY_DEPTH {
        return None;
    }
    let body = expression.strip_prefix('[')?.strip_suffix(']')?;
    let members = if body.trim().is_empty() {
        Vec::new()
    } else {
        let members = split_members(body)?;
        if members.len() > MAX_LITERAL_ARRAY_MEMBERS {
            return None;
        }
        members
            .into_iter()
            .map(|member| parse_at_depth(member, depth + 1))
            .collect::<Option<Vec<_>>>()?
    };
    Some(ValueSequence(vec![Value::Array(members)]))
}

fn split_members(value: &str) -> Option<Vec<&str>> {
    let mut members = Vec::new();
    let mut square_depth = 0_u32;
    let mut parenthesis_depth = 0_u32;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        match character {
            '[' => square_depth += 1,
            ']' => square_depth = square_depth.checked_sub(1)?,
            '(' => parenthesis_depth += 1,
            ')' => parenthesis_depth = parenthesis_depth.checked_sub(1)?,
            ',' if square_depth == 0 && parenthesis_depth == 0 => {
                members.push(&value[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if square_depth != 0 || parenthesis_depth != 0 {
        return None;
    }
    members.push(&value[start..]);
    Some(members)
}

pub(super) fn equals(
    left: &ValueSequence,
    right: &ValueSequence,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    control.charge(WorkDomain::XPathOperation, 1)?;
    if left.0.len() != right.0.len() {
        return Ok(false);
    }
    for (left, right) in left.0.iter().zip(&right.0) {
        control.charge(WorkDomain::XPathOperation, 1)?;
        match (left, right) {
            (Value::Integer(left), Value::Integer(right)) if left == right => {}
            (Value::Array(left), Value::Array(right)) => {
                control.charge(WorkDomain::XPathOperation, 1)?;
                if left.len() != right.len() {
                    return Ok(false);
                }
                for (left, right) in left.iter().zip(right) {
                    if !equals(left, right, control)? {
                        return Ok(false);
                    }
                }
            }
            _ => return Ok(false),
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::{equals, parse};
    use crate::execution_control_experiment::InvocationControl;

    #[test]
    fn distinguishes_array_members_from_member_sequences() {
        let mut control = InvocationControl::unbounded();
        assert!(
            equals(
                &parse("[[]]").unwrap(),
                &parse("[[]]").unwrap(),
                &mut control
            )
            .unwrap()
        );
        assert!(!equals(&parse("[]").unwrap(), &parse("[()]").unwrap(), &mut control).unwrap());
        assert!(!equals(&parse("[1]").unwrap(), &parse("1").unwrap(), &mut control).unwrap());
    }

    #[test]
    fn malformed_or_excessively_nested_arrays_are_not_admitted() {
        assert!(parse("[]]").is_none());
        let nested = format!("{}1{}", "[".repeat(65), "]".repeat(65));
        assert!(parse(&nested).is_none());
    }
}
