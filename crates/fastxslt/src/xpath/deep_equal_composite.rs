//! Safe private reference representation for admitted XDM composite-value
//! `deep-equal` literal slices.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ValueSequence(Vec<Value>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Integer(i128),
    Boolean(bool),
    String(String),
    Array(Vec<ValueSequence>),
    Map(Vec<(i128, ValueSequence)>),
}

const MAX_LITERAL_COMPOSITE_DEPTH: usize = 64;
const MAX_LITERAL_COMPOSITE_ITEMS: usize = 1_024;

pub(super) fn recognizes(left: &str, right: &str) -> bool {
    begins_with_array_item(left) || begins_with_array_item(right)
}

fn begins_with_array_item(expression: &str) -> bool {
    let expression = expression.trim_start();
    expression.starts_with('[')
        || expression.starts_with("map{")
        || expression.strip_prefix('(').is_some_and(|body| {
            let body = body.trim_start();
            body.starts_with('[') || body.starts_with("map{")
        })
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
    if matches!(expression, "true()" | "false()") {
        return Some(ValueSequence(vec![Value::Boolean(expression == "true()")]));
    }
    if expression.starts_with('\'') {
        return Some(ValueSequence(vec![Value::String(parse_string_literal(
            expression,
        )?)]));
    }
    if depth >= MAX_LITERAL_COMPOSITE_DEPTH {
        return None;
    }
    if let Some(body) = expression
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        let items = split_members(body)?;
        if items.len() > MAX_LITERAL_COMPOSITE_ITEMS {
            return None;
        }
        let mut values = Vec::new();
        for item in items {
            values.extend(parse_at_depth(item, depth + 1)?.0);
        }
        return Some(ValueSequence(values));
    }
    if let Some(body) = expression
        .strip_prefix("map{")
        .and_then(|value| value.strip_suffix('}'))
    {
        let entries = if body.trim().is_empty() {
            Vec::new()
        } else {
            let entries = split_members(body)?;
            if entries.len() > MAX_LITERAL_COMPOSITE_ITEMS {
                return None;
            }
            entries
                .into_iter()
                .map(|entry| {
                    let (key, value) = split_map_entry(entry)?;
                    Some((
                        key.trim().parse::<i128>().ok()?,
                        parse_at_depth(value, depth + 1)?,
                    ))
                })
                .collect::<Option<Vec<_>>>()?
        };
        if entries
            .iter()
            .enumerate()
            .any(|(index, (key, _))| entries[..index].iter().any(|(prior, _)| prior == key))
        {
            return None;
        }
        return Some(ValueSequence(vec![Value::Map(entries)]));
    }
    let body = expression.strip_prefix('[')?.strip_suffix(']')?;
    let members = if body.trim().is_empty() {
        Vec::new()
    } else {
        let members = split_members(body)?;
        if members.len() > MAX_LITERAL_COMPOSITE_ITEMS {
            return None;
        }
        members
            .into_iter()
            .map(|member| parse_at_depth(member, depth + 1))
            .collect::<Option<Vec<_>>>()?
    };
    Some(ValueSequence(vec![Value::Array(members)]))
}

fn split_map_entry(entry: &str) -> Option<(&str, &str)> {
    let (key, value) = entry.split_once(':')?;
    if value.contains(':') {
        return None;
    }
    Some((key, value))
}

fn parse_string_literal(expression: &str) -> Option<String> {
    let body = expression.strip_prefix('\'')?.strip_suffix('\'')?;
    let mut characters = body.chars().peekable();
    let mut value = String::new();
    while let Some(character) = characters.next() {
        if character == '\'' {
            if characters.next() != Some('\'') {
                return None;
            }
            value.push('\'');
        } else {
            value.push(character);
        }
    }
    Some(value)
}

fn split_members(value: &str) -> Option<Vec<&str>> {
    let mut members = Vec::new();
    let mut square_depth = 0_u32;
    let mut parenthesis_depth = 0_u32;
    let mut curly_depth = 0_u32;
    let mut in_string = false;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        match character {
            '\'' => in_string = !in_string,
            '[' if !in_string => square_depth += 1,
            ']' if !in_string => square_depth = square_depth.checked_sub(1)?,
            '(' if !in_string => parenthesis_depth += 1,
            ')' if !in_string => parenthesis_depth = parenthesis_depth.checked_sub(1)?,
            '{' if !in_string => curly_depth += 1,
            '}' if !in_string => curly_depth = curly_depth.checked_sub(1)?,
            ',' if !in_string
                && square_depth == 0
                && parenthesis_depth == 0
                && curly_depth == 0 =>
            {
                members.push(&value[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if in_string || square_depth != 0 || parenthesis_depth != 0 || curly_depth != 0 {
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
            (Value::Boolean(left), Value::Boolean(right)) if left == right => {}
            (Value::String(left), Value::String(right)) if left == right => {}
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
            (Value::Map(left), Value::Map(right)) => {
                control.charge(WorkDomain::XPathOperation, 1)?;
                if left.len() != right.len() {
                    return Ok(false);
                }
                for (key, left_value) in left {
                    control.charge(WorkDomain::XPathOperation, 1)?;
                    let Some((_, right_value)) =
                        right.iter().find(|(candidate, _)| candidate == key)
                    else {
                        return Ok(false);
                    };
                    if !equals(left_value, right_value, control)? {
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
    use super::{equals, parse, recognizes};
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
        assert!(parse("['a'b']").is_none());
        let nested = format!("{}1{}", "[".repeat(65), "]".repeat(65));
        assert!(parse(&nested).is_none());
    }

    #[test]
    fn path_predicates_are_not_array_constructor_evidence() {
        assert!(!recognizes("//a[1]/@a", "//a[2]/@a"));
        assert!(recognizes("([1], [])", "([1], [])"));
    }

    #[test]
    fn map_entry_order_is_not_semantic() {
        let mut control = InvocationControl::unbounded();
        assert!(
            equals(
                &parse("map{1:true(), 2:false()}").unwrap(),
                &parse("map{2:false(), 1:true()}").unwrap(),
                &mut control
            )
            .unwrap()
        );
        assert!(parse("map{1:true(), 1:false()}").is_none());
    }
}
