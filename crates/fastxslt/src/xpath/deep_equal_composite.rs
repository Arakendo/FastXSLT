//! Safe private reference representation for admitted XDM composite-value
//! `deep-equal` literal slices.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ValueSequence(Vec<Value>);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    Number(Number),
    Boolean(bool),
    String(String),
    Array(Vec<ValueSequence>),
    Map(Vec<(Number, ValueSequence)>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Number {
    Finite { coefficient: i128, scale: u32 },
    NaN,
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
    if let Some(value) = parse_number(expression) {
        return Some(ValueSequence(vec![Value::Number(value)]));
    }
    if matches!(expression, "true()" | "false()") {
        return Some(ValueSequence(vec![Value::Boolean(expression == "true()")]));
    }
    if expression.starts_with(['\'', '"']) {
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
    if expression.starts_with('[') && expression.ends_with(']') {
        return parse_array_constructor(expression, depth);
    }
    if expression.contains("=> array:") {
        return parse_array_update(expression, depth);
    }
    if expression.starts_with("map:remove(") {
        return parse_map_remove(expression, depth);
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
                    Some((parse_number(key.trim())?, parse_at_depth(value, depth + 1)?))
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
    None
}

fn parse_array_constructor(expression: &str, depth: usize) -> Option<ValueSequence> {
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

fn parse_array_update(expression: &str, depth: usize) -> Option<ValueSequence> {
    let (base, operation) = expression.split_once("=>")?;
    let mut base = parse_at_depth(base, depth + 1)?.0;
    if base.len() != 1 {
        return None;
    }
    let Value::Array(mut members) = base.pop()? else {
        return None;
    };
    let operation = operation.trim();
    if let Some(arguments) = operation
        .strip_prefix("array:put(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let arguments = split_members(arguments)?;
        if arguments.len() != 2 {
            return None;
        }
        let position = arguments[0].trim().parse::<usize>().ok()?.checked_sub(1)?;
        *members.get_mut(position)? = parse_at_depth(arguments[1], depth + 1)?;
    } else if let Some(position) = operation
        .strip_prefix("array:remove(")
        .and_then(|value| value.strip_suffix(')'))
        .and_then(|value| value.trim().parse::<usize>().ok())
        .and_then(|value| value.checked_sub(1))
    {
        if position >= members.len() {
            return None;
        }
        members.remove(position);
    } else {
        return None;
    }
    Some(ValueSequence(vec![Value::Array(members)]))
}

fn parse_map_remove(expression: &str, depth: usize) -> Option<ValueSequence> {
    let arguments = expression.strip_prefix("map:remove(")?.strip_suffix(')')?;
    let arguments = split_members(arguments)?;
    if arguments.len() != 2 {
        return None;
    }
    let mut base = parse_at_depth(arguments[0], depth + 1)?.0;
    if base.len() != 1 {
        return None;
    }
    let Value::Map(mut entries) = base.pop()? else {
        return None;
    };
    let key = parse_number(arguments[1].trim())?;
    let position = entries
        .iter()
        .position(|(candidate, _)| *candidate == key)?;
    entries.remove(position);
    Some(ValueSequence(vec![Value::Map(entries)]))
}

fn split_map_entry(entry: &str) -> Option<(&str, &str)> {
    let mut parenthesis_depth = 0_u32;
    let mut square_depth = 0_u32;
    let mut curly_depth = 0_u32;
    let mut in_string = false;
    for (index, character) in entry.char_indices() {
        match character {
            '\'' => in_string = !in_string,
            '(' if !in_string => parenthesis_depth = parenthesis_depth.checked_add(1)?,
            ')' if !in_string => parenthesis_depth = parenthesis_depth.checked_sub(1)?,
            '[' if !in_string => square_depth = square_depth.checked_add(1)?,
            ']' if !in_string => square_depth = square_depth.checked_sub(1)?,
            '{' if !in_string => curly_depth = curly_depth.checked_add(1)?,
            '}' if !in_string => curly_depth = curly_depth.checked_sub(1)?,
            ':' if !in_string
                && parenthesis_depth == 0
                && square_depth == 0
                && curly_depth == 0
                && parse_number(entry[..index].trim()).is_some() =>
            {
                return Some((&entry[..index], &entry[index + 1..]));
            }
            _ => {}
        }
    }
    None
}

fn parse_number(expression: &str) -> Option<Number> {
    if matches!(expression, "xs:double('NaN')" | "xs:float('NaN')") {
        return Some(Number::NaN);
    }
    let (mantissa, exponent) = expression
        .split_once(['e', 'E'])
        .map_or(Some((expression, 0_i32)), |(mantissa, exponent)| {
            Some((mantissa, exponent.parse::<i32>().ok()?))
        })?;
    let negative = mantissa.starts_with('-');
    let mantissa = mantissa.strip_prefix(['-', '+']).unwrap_or(mantissa);
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    if whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let digits = format!("{whole}{fraction}");
    let mut coefficient = digits.parse::<i128>().ok()?;
    if negative {
        coefficient = coefficient.checked_neg()?;
    }
    let mut scale = i32::try_from(fraction.len()).ok()?.checked_sub(exponent)?;
    if scale < 0 {
        coefficient = coefficient.checked_mul(10_i128.checked_pow(scale.unsigned_abs())?)?;
        scale = 0;
    }
    let mut scale = u32::try_from(scale).ok()?;
    while scale > 0 && coefficient % 10 == 0 {
        coefficient /= 10;
        scale -= 1;
    }
    Some(Number::Finite { coefficient, scale })
}

fn parse_string_literal(expression: &str) -> Option<String> {
    let quote = expression.chars().next()?;
    if !matches!(quote, '\'' | '"') {
        return None;
    }
    let body = expression.strip_prefix(quote)?.strip_suffix(quote)?;
    let mut characters = body.chars().peekable();
    let mut value = String::new();
    while let Some(character) = characters.next() {
        if character == quote {
            if characters.next() != Some(quote) {
                return None;
            }
            value.push(quote);
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
    let mut quote = None;
    let mut start = 0;
    for (index, character) in value.char_indices() {
        match character {
            '\'' | '"' if quote.is_none() => quote = Some(character),
            character if quote == Some(character) => quote = None,
            '[' if quote.is_none() => square_depth += 1,
            ']' if quote.is_none() => square_depth = square_depth.checked_sub(1)?,
            '(' if quote.is_none() => parenthesis_depth += 1,
            ')' if quote.is_none() => parenthesis_depth = parenthesis_depth.checked_sub(1)?,
            '{' if quote.is_none() => curly_depth += 1,
            '}' if quote.is_none() => curly_depth = curly_depth.checked_sub(1)?,
            ',' if quote.is_none()
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
    if quote.is_some() || square_depth != 0 || parenthesis_depth != 0 || curly_depth != 0 {
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
            (Value::Number(left), Value::Number(right)) if left == right => {}
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

    #[test]
    fn numeric_keys_normalize_exactly_and_nan_uses_same_key_semantics() {
        let mut control = InvocationControl::unbounded();
        for (left, right) in [
            ("map{1:true()}", "map{1.0:true()}"),
            ("map{1:true()}", "map{1.0e0:true()}"),
            (
                "map{xs:double('NaN'):true()}",
                "map{xs:float('NaN'):true()}",
            ),
        ] {
            assert!(equals(&parse(left).unwrap(), &parse(right).unwrap(), &mut control).unwrap());
        }
        assert!(parse("map{1e999999999:true()}").is_none());
    }

    #[test]
    fn folds_bounded_literal_composite_updates() {
        assert!(parse("['a', 'b', 'd'] => array:put(3, 'c')").is_some());
        assert!(parse("['a', 'b', 'c', 'd'] => array:remove(4)").is_some());
        assert!(parse("map:remove(map{1:\"A\", 2:\"B\"}, 2)").is_some());
        assert!(parse("([['a', 'b', 'd'] => array:put(3, 'c')], [], [1])").is_some());
    }
}
