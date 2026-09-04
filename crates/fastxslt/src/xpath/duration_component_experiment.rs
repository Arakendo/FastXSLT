//! Bounded typed duration-component semantics used by QT3 evidence.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DurationValue {
    Boolean(bool),
    Duration(Duration),
    Empty,
    Integer(i128),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Duration {
    months: i128,
    whole_seconds: i128,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DurationKind {
    General,
    YearMonth,
    DayTime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DurationFailure {
    Control(ControlFailure),
    InvalidArity,
    Unsupported,
}

pub(crate) fn evaluate(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<DurationValue, DurationFailure> {
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(DurationFailure::Control)?;
    evaluate_value(expression.trim(), control)
}

fn evaluate_value(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<DurationValue, DurationFailure> {
    let expression = strip_outer_parentheses(expression.trim());
    if let Some(operand) = expression.strip_suffix(" instance of xs:integer?") {
        return Ok(DurationValue::Boolean(matches!(
            evaluate_value(operand, control)?,
            DurationValue::Empty | DurationValue::Integer(_)
        )));
    }
    for (operator, comparison) in [
        (" eq ", integer_eq as fn(i128, i128) -> bool),
        (" ne ", integer_ne),
        (" lt ", integer_lt),
        (" le ", integer_le),
        (" gt ", integer_gt),
        (" ge ", integer_ge),
    ] {
        if let Some((left, right)) = split_top_level(expression, operator) {
            return Ok(DurationValue::Boolean(comparison(
                require_integer(evaluate_value(left, control)?)?,
                require_integer(evaluate_value(right, control)?)?,
            )));
        }
    }
    for (operator, operation) in [
        (" + ", i128::checked_add as fn(i128, i128) -> Option<i128>),
        (" - ", i128::checked_sub),
    ] {
        if let Some((left, right)) = split_top_level(expression, operator) {
            return checked_binary(left, right, control, operation);
        }
    }
    for (operator, operation) in [
        (" * ", i128::checked_mul as fn(i128, i128) -> Option<i128>),
        (" div ", i128::checked_div),
        (" idiv ", i128::checked_div),
        (" mod ", i128::checked_rem),
    ] {
        if let Some((left, right)) = split_top_level(expression, operator) {
            return checked_binary(left, right, control, operation);
        }
    }
    if let Some(operand) = expression.strip_prefix('+') {
        return evaluate_value(operand, control);
    }
    if let Some(operand) = expression.strip_prefix('-') {
        return Ok(DurationValue::Integer(
            require_integer(evaluate_value(operand, control)?)?
                .checked_neg()
                .ok_or(DurationFailure::Unsupported)?,
        ));
    }
    if let Ok(value) = expression.parse::<i128>() {
        return Ok(DurationValue::Integer(value));
    }

    let (name, argument) = parse_function(expression).ok_or(DurationFailure::Unsupported)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(DurationFailure::Control)?;
    evaluate_function(name, argument, control)
}

fn integer_eq(left: i128, right: i128) -> bool {
    left == right
}

fn integer_ne(left: i128, right: i128) -> bool {
    left != right
}

fn integer_lt(left: i128, right: i128) -> bool {
    left < right
}

fn integer_le(left: i128, right: i128) -> bool {
    left <= right
}

fn integer_gt(left: i128, right: i128) -> bool {
    left > right
}

fn integer_ge(left: i128, right: i128) -> bool {
    left >= right
}

fn evaluate_function(
    name: &str,
    argument: &str,
    control: &mut InvocationControl,
) -> Result<DurationValue, DurationFailure> {
    match name {
        "years-from-duration" | "fn:years-from-duration" => {
            require_one_argument(argument)?;
            if argument.trim() == "()" {
                return Ok(DurationValue::Empty);
            }
            let duration = require_duration(evaluate_value(argument, control)?)?;
            Ok(DurationValue::Integer(duration.months / 12))
        }
        "months-from-duration" | "fn:months-from-duration" => {
            require_one_argument(argument)?;
            if argument.trim() == "()" {
                return Ok(DurationValue::Empty);
            }
            let duration = require_duration(evaluate_value(argument, control)?)?;
            Ok(DurationValue::Integer(duration.months % 12))
        }
        "days-from-duration" | "fn:days-from-duration" => {
            require_one_argument(argument)?;
            if argument.trim() == "()" {
                return Ok(DurationValue::Empty);
            }
            let duration = require_duration(evaluate_value(argument, control)?)?;
            Ok(DurationValue::Integer(duration.whole_seconds / 86_400))
        }
        "hours-from-duration" | "fn:hours-from-duration" => {
            require_one_argument(argument)?;
            if argument.trim() == "()" {
                return Ok(DurationValue::Empty);
            }
            let duration = require_duration(evaluate_value(argument, control)?)?;
            Ok(DurationValue::Integer(
                (duration.whole_seconds / 3_600) % 24,
            ))
        }
        "minutes-from-duration" | "fn:minutes-from-duration" => {
            require_one_argument(argument)?;
            if argument.trim() == "()" {
                return Ok(DurationValue::Empty);
            }
            let duration = require_duration(evaluate_value(argument, control)?)?;
            Ok(DurationValue::Integer((duration.whole_seconds / 60) % 60))
        }
        "xs:yearMonthDuration" | "xs:duration" | "xs:dayTimeDuration" => {
            require_one_argument(argument)?;
            let lexical = parse_quoted(argument).ok_or(DurationFailure::Unsupported)?;
            let kind = match name {
                "xs:yearMonthDuration" => DurationKind::YearMonth,
                "xs:dayTimeDuration" => DurationKind::DayTime,
                "xs:duration" => DurationKind::General,
                _ => unreachable!(),
            };
            Ok(DurationValue::Duration(parse_duration(&lexical, kind)?))
        }
        "count" | "fn:count" => {
            require_one_argument(argument)?;
            Ok(DurationValue::Integer(i128::from(!matches!(
                evaluate_value(argument, control)?,
                DurationValue::Empty
            ))))
        }
        "empty" | "fn:empty" => {
            require_one_argument(argument)?;
            Ok(DurationValue::Boolean(matches!(
                evaluate_value(argument, control)?,
                DurationValue::Empty
            )))
        }
        "avg" | "fn:avg" => {
            require_one_argument(argument)?;
            let sequence = strip_outer_parentheses(argument.trim());
            let (left, right) =
                split_top_level(sequence, ",").ok_or(DurationFailure::Unsupported)?;
            let left = require_integer(evaluate_value(left, control)?)?;
            let right = require_integer(evaluate_value(right, control)?)?;
            Ok(DurationValue::Integer((left + right) / 2))
        }
        _ => Err(DurationFailure::Unsupported),
    }
}

fn checked_binary(
    left: &str,
    right: &str,
    control: &mut InvocationControl,
    operation: fn(i128, i128) -> Option<i128>,
) -> Result<DurationValue, DurationFailure> {
    let left = require_integer(evaluate_value(left, control)?)?;
    let right = require_integer(evaluate_value(right, control)?)?;
    Ok(DurationValue::Integer(
        operation(left, right).ok_or(DurationFailure::Unsupported)?,
    ))
}

fn parse_duration(lexical: &str, kind: DurationKind) -> Result<Duration, DurationFailure> {
    let (sign, lexical) = lexical
        .strip_prefix('-')
        .map_or((1_i128, lexical), |value| (-1, value));
    let body = lexical
        .strip_prefix('P')
        .ok_or(DurationFailure::Unsupported)?;
    if body.is_empty() || body.matches('T').count() > 1 {
        return Err(DurationFailure::Unsupported);
    }
    let (date, time) = body.split_once('T').map_or((body, ""), |parts| parts);
    if body.contains('T') && time.is_empty() {
        return Err(DurationFailure::Unsupported);
    }
    match kind {
        DurationKind::YearMonth if body.contains('T') || date.contains('D') => {
            return Err(DurationFailure::Unsupported);
        }
        DurationKind::DayTime if date.contains('Y') || date.contains('M') => {
            return Err(DurationFailure::Unsupported);
        }
        DurationKind::General | DurationKind::YearMonth | DurationKind::DayTime => {}
    }
    let (years, after_years) = take_component(date, 'Y')?;
    let (months, after_months) = take_component(after_years, 'M')?;
    let (days, after_days) = take_component(after_months, 'D')?;
    if !after_days.is_empty() {
        return Err(DurationFailure::Unsupported);
    }
    let (hours, after_hours) = take_component(time, 'H')?;
    let (minutes, after_minutes) = take_component(after_hours, 'M')?;
    let seconds = take_whole_seconds(after_minutes)?;
    let has_component = date.contains(['Y', 'M', 'D']) || time.contains(['H', 'M', 'S']);
    if !has_component {
        return Err(DurationFailure::Unsupported);
    }
    let months = years
        .checked_mul(12)
        .and_then(|value| value.checked_add(months))
        .ok_or(DurationFailure::Unsupported)?;
    let whole_seconds = days
        .checked_mul(86_400)
        .and_then(|value| {
            hours
                .checked_mul(3_600)
                .and_then(|hours| value.checked_add(hours))
        })
        .and_then(|value| {
            minutes
                .checked_mul(60)
                .and_then(|minutes| value.checked_add(minutes))
        })
        .and_then(|value| value.checked_add(seconds))
        .ok_or(DurationFailure::Unsupported)?;
    Ok(Duration {
        months: sign
            .checked_mul(months)
            .ok_or(DurationFailure::Unsupported)?,
        whole_seconds: sign
            .checked_mul(whole_seconds)
            .ok_or(DurationFailure::Unsupported)?,
    })
}

fn take_whole_seconds(input: &str) -> Result<i128, DurationFailure> {
    let Some(seconds) = input.strip_suffix('S') else {
        return if input.is_empty() {
            Ok(0)
        } else {
            Err(DurationFailure::Unsupported)
        };
    };
    let whole = if let Some((whole, fraction)) = seconds.split_once('.') {
        if whole.is_empty()
            || fraction.is_empty()
            || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(DurationFailure::Unsupported);
        }
        whole
    } else {
        seconds
    };
    whole
        .parse::<i128>()
        .map_err(|_| DurationFailure::Unsupported)
}

fn take_component(input: &str, marker: char) -> Result<(i128, &str), DurationFailure> {
    let Some(index) = input.find(marker) else {
        return Ok((0, input));
    };
    let value = input[..index]
        .parse::<i128>()
        .map_err(|_| DurationFailure::Unsupported)?;
    Ok((value, &input[index + marker.len_utf8()..]))
}

fn require_duration(value: DurationValue) -> Result<Duration, DurationFailure> {
    match value {
        DurationValue::Duration(value) => Ok(value),
        _ => Err(DurationFailure::Unsupported),
    }
}

fn require_integer(value: DurationValue) -> Result<i128, DurationFailure> {
    match value {
        DurationValue::Integer(value) => Ok(value),
        _ => Err(DurationFailure::Unsupported),
    }
}

fn require_one_argument(argument: &str) -> Result<(), DurationFailure> {
    if argument.trim().is_empty() || split_top_level(argument, ",").is_some() {
        Err(DurationFailure::InvalidArity)
    } else {
        Ok(())
    }
}

fn parse_function(expression: &str) -> Option<(&str, &str)> {
    let open = expression.find('(')?;
    let name = expression[..open].trim();
    if name.is_empty() || !expression.ends_with(')') {
        return None;
    }
    let argument = &expression[open + 1..expression.len() - 1];
    balanced(argument).then_some((name, argument))
}

fn parse_quoted(expression: &str) -> Option<String> {
    for quote in ['"', '\''] {
        if let Some(value) = expression
            .trim()
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
            .filter(|value| !value.contains(quote))
        {
            return Some(value.to_owned());
        }
    }
    None
}

fn strip_outer_parentheses(mut expression: &str) -> &str {
    while expression.starts_with('(')
        && expression.ends_with(')')
        && balanced(&expression[1..expression.len() - 1])
    {
        expression = expression[1..expression.len() - 1].trim();
    }
    expression
}

fn split_top_level<'a>(expression: &'a str, separator: &str) -> Option<(&'a str, &'a str)> {
    let mut depth = 0usize;
    let mut quote = None;
    for (index, character) in expression.char_indices() {
        if matches!(character, '"' | '\'') {
            quote = if quote == Some(character) {
                None
            } else if quote.is_none() {
                Some(character)
            } else {
                quote
            };
            continue;
        }
        if quote.is_some() {
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            _ if depth == 0 && expression[index..].starts_with(separator) => {
                let right = index + separator.len();
                return Some((expression[..index].trim(), expression[right..].trim()));
            }
            _ => {}
        }
    }
    None
}

fn balanced(expression: &str) -> bool {
    let mut depth = 0usize;
    let mut quote = None;
    for character in expression.chars() {
        if matches!(character, '"' | '\'') {
            quote = if quote == Some(character) {
                None
            } else if quote.is_none() {
                Some(character)
            } else {
                quote
            };
            continue;
        }
        if quote.is_some() {
            continue;
        }
        match character {
            '(' => depth += 1,
            ')' => {
                let Some(next_depth) = depth.checked_sub(1) else {
                    return false;
                };
                depth = next_depth;
            }
            _ => {}
        }
    }
    depth == 0 && quote.is_none()
}

#[cfg(test)]
mod tests {
    use super::{DurationFailure, DurationValue, evaluate};
    use crate::execution_control_experiment::InvocationControl;

    #[test]
    fn normalizes_signed_year_month_components() {
        for (source, expected) in [
            (
                "years-from-duration(xs:yearMonthDuration(\"P2030Y12M\"))",
                2031,
            ),
            ("years-from-duration(xs:duration(\"-P3Y4M4DT1H\"))", -3),
            ("years-from-duration(xs:dayTimeDuration(\"P1D\"))", 0),
        ] {
            assert_eq!(
                evaluate(source, &mut InvocationControl::unbounded()),
                Ok(DurationValue::Integer(expected))
            );
        }
    }

    #[test]
    fn extracts_signed_normalized_month_components() {
        for (source, expected) in [
            ("months-from-duration(xs:yearMonthDuration(\"P20Y15M\"))", 3),
            ("months-from-duration(xs:duration(\"-P3Y4M4DT1H\"))", -4),
            ("months-from-duration(xs:dayTimeDuration(\"P1D\"))", 0),
        ] {
            assert_eq!(
                evaluate(source, &mut InvocationControl::unbounded()),
                Ok(DurationValue::Integer(expected))
            );
        }
    }

    #[test]
    fn extracts_signed_normalized_day_components() {
        for (source, expected) in [
            ("days-from-duration(xs:dayTimeDuration(\"P3DT55H\"))", 5),
            (
                "days-from-duration(xs:duration(\"-P3Y4M8DT1H23M2.34S\"))",
                -8,
            ),
            ("days-from-duration(xs:yearMonthDuration(\"P1Y\"))", 0),
        ] {
            assert_eq!(
                evaluate(source, &mut InvocationControl::unbounded()),
                Ok(DurationValue::Integer(expected))
            );
        }
    }

    #[test]
    fn extracts_signed_normalized_hour_and_minute_components() {
        for (source, expected) in [
            ("hours-from-duration(xs:dayTimeDuration(\"PT123H\"))", 3),
            (
                "hours-from-duration(xs:duration(\"-P3Y4M8DT1H23M2.34S\"))",
                -1,
            ),
            (
                "minutes-from-duration(xs:dayTimeDuration(\"P21DT10H65M\"))",
                5,
            ),
            (
                "minutes-from-duration(xs:duration(\"-P3Y4M8DT1H23M2.34S\"))",
                -23,
            ),
        ] {
            assert_eq!(
                evaluate(source, &mut InvocationControl::unbounded()),
                Ok(DurationValue::Integer(expected))
            );
        }
    }

    #[test]
    fn rejects_subtype_contamination_and_incomplete_lexicals() {
        for source in [
            "years-from-duration(xs:yearMonthDuration(\"P1D\"))",
            "days-from-duration(xs:dayTimeDuration(\"P1Y\"))",
            "days-from-duration(xs:duration(\"P1Dgarbage\"))",
            "days-from-duration(xs:duration(\"P\"))",
            "hours-from-duration(xs:duration(\"PT\"))",
            "hours-from-duration(xs:duration(\"PT1.S\"))",
        ] {
            assert_eq!(
                evaluate(source, &mut InvocationControl::unbounded()),
                Err(DurationFailure::Unsupported),
                "{source}"
            );
        }
    }
}
