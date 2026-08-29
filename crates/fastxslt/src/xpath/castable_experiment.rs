//! Built-in atomic castability for the native `castable-001` through `004` slices.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};

use super::path_experiment::{
    LocationPath, PathFailure, evaluate_location_path_controlled, parse_location_path,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CastableExpression {
    operand: AtomicOperand,
    target: BuiltinAtomicType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AtomicOperand {
    Path(LocationPath),
    Variable(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CastExpression {
    operand: LocationPath,
    target: BuiltinAtomicType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CastableFailure {
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CastEvaluationFailure {
    Control(ControlFailure),
    InvalidValue,
}

pub(crate) fn parse(
    expression: &str,
    location: &SourceLocation,
) -> Result<CastableExpression, CastableFailure> {
    let normalized = expression.trim();
    let (operand, target) = normalized
        .split_once(" castable as xs:")
        .ok_or_else(|| unsupported(normalized, location))?;
    let target =
        BuiltinAtomicType::parse(target).ok_or_else(|| unsupported(normalized, location))?;
    let operand = if let Some(variable) = operand.strip_prefix('$') {
        if !is_ascii_ncname(variable) || !variable_target_is_admitted(target) {
            return Err(unsupported(normalized, location));
        }
        AtomicOperand::Variable(variable.to_owned())
    } else {
        AtomicOperand::Path(parse_path_operand(operand, location)?)
    };
    Ok(CastableExpression { operand, target })
}

pub(crate) fn parse_cast(
    expression: &str,
    location: &SourceLocation,
) -> Result<CastExpression, CastableFailure> {
    let normalized = expression.trim();
    let (operand, target) = normalized
        .split_once(" cast as xs:")
        .ok_or_else(|| unsupported(normalized, location))?;
    let target =
        BuiltinAtomicType::parse(target).ok_or_else(|| unsupported(normalized, location))?;
    if target == BuiltinAtomicType::UntypedAtomic {
        return Err(unsupported(normalized, location));
    }
    Ok(CastExpression {
        operand: parse_path_operand(operand, location)?,
        target,
    })
}

pub(crate) fn evaluate(
    expression: &CastableExpression,
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let AtomicOperand::Path(path) = &expression.operand else {
        unreachable!("variable castability is evaluated from the runtime variable frame")
    };
    let selected = evaluate_location_path_controlled(document, context, path, control)?;
    control.charge(WorkDomain::XPathOperation, 1)?;
    let [node] = selected.as_slice() else {
        return Ok(false);
    };
    let lexical = document.string_value_controlled(*node, control)?;
    Ok(castable_as(
        &AtomicValue::untyped(lexical),
        expression.target,
    ))
}

pub(crate) fn variable_name(expression: &CastableExpression) -> Option<&str> {
    match &expression.operand {
        AtomicOperand::Path(_) => None,
        AtomicOperand::Variable(name) => Some(name),
    }
}

pub(crate) fn evaluate_value(
    expression: &CastableExpression,
    value: &AtomicValue,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    control.charge(WorkDomain::XPathOperation, 1)?;
    Ok(castable_as(value, expression.target))
}

pub(crate) fn evaluate_cast(
    expression: &CastExpression,
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<AtomicValue, CastEvaluationFailure> {
    let selected =
        evaluate_location_path_controlled(document, context, &expression.operand, control)
            .map_err(CastEvaluationFailure::Control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(CastEvaluationFailure::Control)?;
    let [node] = selected.as_slice() else {
        return Err(CastEvaluationFailure::InvalidValue);
    };
    let lexical = document
        .string_value_controlled(*node, control)
        .map_err(CastEvaluationFailure::Control)?;
    cast_untyped(&lexical, expression.target).ok_or(CastEvaluationFailure::InvalidValue)
}

fn cast_untyped(value: &str, target: BuiltinAtomicType) -> Option<AtomicValue> {
    target
        .accepts(value)
        .then(|| AtomicValue::from_validated_lexical(target, value.trim().to_owned()))
}

fn castable_as(value: &AtomicValue, target: BuiltinAtomicType) -> bool {
    if matches!(
        target,
        BuiltinAtomicType::String | BuiltinAtomicType::UntypedAtomic
    ) {
        return true;
    }
    if value.atomic_type() == target {
        return true;
    }

    match (value.atomic_type(), target) {
        (BuiltinAtomicType::String | BuiltinAtomicType::UntypedAtomic, _) => {
            target.accepts(value.lexical())
        }
        (BuiltinAtomicType::Boolean, target) if target.is_numeric() => true,
        (source, target) if source.is_numeric() && target.is_numeric() => !matches!(
            (source, target, value.lexical().trim()),
            (
                BuiltinAtomicType::Float | BuiltinAtomicType::Double,
                BuiltinAtomicType::Integer | BuiltinAtomicType::Decimal,
                "INF" | "-INF" | "NaN"
            )
        ),
        (source, target) if source.is_duration() && target.is_duration() => true,
        _ => false,
    }
}

fn variable_target_is_admitted(target: BuiltinAtomicType) -> bool {
    matches!(
        target,
        BuiltinAtomicType::String | BuiltinAtomicType::UntypedAtomic
    ) || target.is_numeric()
        || target.is_duration()
}

impl BuiltinAtomicType {
    fn parse(local: &str) -> Option<Self> {
        Some(match local {
            "string" => Self::String,
            "untypedAtomic" => Self::UntypedAtomic,
            "boolean" => Self::Boolean,
            "integer" => Self::Integer,
            "decimal" => Self::Decimal,
            "float" => Self::Float,
            "double" => Self::Double,
            "duration" => Self::Duration,
            "dayTimeDuration" => Self::DayTimeDuration,
            "yearMonthDuration" => Self::YearMonthDuration,
            "dateTime" => Self::DateTime,
            "date" => Self::Date,
            "time" => Self::Time,
            _ => return None,
        })
    }

    fn accepts(self, lexical: &str) -> bool {
        let collapsed = lexical.trim();
        match self {
            Self::String | Self::UntypedAtomic => true,
            Self::Boolean => matches!(collapsed, "true" | "false" | "1" | "0"),
            Self::Integer => signed_digits(collapsed),
            Self::Decimal => decimal(collapsed),
            Self::Float | Self::Double => floating_point(collapsed),
            Self::Duration => duration(collapsed, DurationKind::General),
            Self::DayTimeDuration => duration(collapsed, DurationKind::DayTime),
            Self::YearMonthDuration => duration(collapsed, DurationKind::YearMonth),
            Self::DateTime => date_time(collapsed),
            Self::Date => date(collapsed),
            Self::Time => time(collapsed),
        }
    }

    fn is_numeric(self) -> bool {
        matches!(
            self,
            Self::Integer | Self::Decimal | Self::Float | Self::Double
        )
    }

    fn is_duration(self) -> bool {
        matches!(
            self,
            Self::Duration | Self::DayTimeDuration | Self::YearMonthDuration
        )
    }
}

fn parse_path_operand(
    operand: &str,
    location: &SourceLocation,
) -> Result<LocationPath, CastableFailure> {
    parse_location_path(operand, location.clone()).map_err(|failure| {
        let detail = match failure {
            PathFailure::Invalid { detail, .. } | PathFailure::Unsupported { detail, .. } => detail,
        };
        CastableFailure {
            detail,
            location: location.clone(),
        }
    })
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

fn signed_digits(value: &str) -> bool {
    let digits = value.strip_prefix(['+', '-']).unwrap_or(value);
    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
}

fn decimal(value: &str) -> bool {
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    let Some((whole, fractional)) = unsigned.split_once('.') else {
        return !unsigned.is_empty() && unsigned.bytes().all(|byte| byte.is_ascii_digit());
    };
    !(whole.is_empty() && fractional.is_empty())
        && whole.bytes().all(|byte| byte.is_ascii_digit())
        && fractional.bytes().all(|byte| byte.is_ascii_digit())
}

fn floating_point(value: &str) -> bool {
    if matches!(value, "INF" | "-INF" | "NaN") {
        return true;
    }
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    let (mantissa, exponent) = unsigned
        .split_once(['e', 'E'])
        .map_or((unsigned, None), |(mantissa, exponent)| {
            (mantissa, Some(exponent))
        });
    decimal(mantissa)
        && exponent.is_none_or(signed_digits)
        && unsigned.matches(['e', 'E']).count() <= 1
}

#[derive(Clone, Copy)]
enum DurationKind {
    General,
    DayTime,
    YearMonth,
}

fn duration(value: &str, kind: DurationKind) -> bool {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let Some(body) = unsigned.strip_prefix('P') else {
        return false;
    };
    if body.is_empty() || body.matches('T').count() > 1 {
        return false;
    }
    let (date_part, time_part) = body
        .split_once('T')
        .map_or((body, None), |(date, time)| (date, Some(time)));
    let Some(date_mask) = component_mask(date_part, &['Y', 'M', 'D'], false) else {
        return false;
    };
    let time_mask = match time_part {
        Some("") => return false,
        Some(part) => {
            let Some(mask) = component_mask(part, &['H', 'M', 'S'], true) else {
                return false;
            };
            mask
        }
        None => 0,
    };
    if date_mask == 0 && time_mask == 0 {
        return false;
    }
    match kind {
        DurationKind::General => true,
        DurationKind::DayTime => date_mask.trailing_zeros() >= 2,
        DurationKind::YearMonth => time_part.is_none() && date_mask & 0b100 == 0,
    }
}

fn component_mask(value: &str, designators: &[char], seconds_decimal: bool) -> Option<u8> {
    if value.is_empty() {
        return Some(0);
    }
    let mut remainder = value;
    let mut previous = None;
    let mut mask = 0_u8;
    while !remainder.is_empty() {
        let split = remainder.find(|character: char| character.is_ascii_alphabetic())?;
        let (number, suffix) = remainder.split_at(split);
        let designator = suffix.chars().next()?;
        let position = designators
            .iter()
            .position(|candidate| *candidate == designator)?;
        if previous.is_some_and(|previous| position <= previous)
            || number.is_empty()
            || !(seconds_decimal && designator == 'S' && decimal(number)
                || number.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return None;
        }
        mask |= 1 << position;
        previous = Some(position);
        remainder = &suffix[designator.len_utf8()..];
    }
    Some(mask)
}

fn date_time(value: &str) -> bool {
    let Some((date_part, time_part)) = value.split_once('T') else {
        return false;
    };
    !time_part.contains('T') && date(date_part) && time(time_part)
}

fn date(value: &str) -> bool {
    let (core, timezone) = split_timezone(value);
    if !timezone_valid(timezone) {
        return false;
    }
    let negative = core.starts_with('-');
    let unsigned = core.strip_prefix('-').unwrap_or(core);
    let parts: Vec<_> = unsigned.split('-').collect();
    if parts.len() != 3
        || parts[0].len() < 4
        || !parts
            .iter()
            .all(|part| part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return false;
    }
    let Ok(year) = parts[0].parse::<u64>() else {
        return false;
    };
    let (Ok(month), Ok(day)) = (parts[1].parse::<u8>(), parts[2].parse::<u8>()) else {
        return false;
    };
    year != 0 && valid_day(if negative { None } else { Some(year) }, month, day)
}

fn time(value: &str) -> bool {
    let (core, timezone) = split_timezone(value);
    if !timezone_valid(timezone) {
        return false;
    }
    let parts: Vec<_> = core.split(':').collect();
    if parts.len() != 3 || parts[0].len() != 2 || parts[1].len() != 2 {
        return false;
    }
    let (Ok(hour), Ok(minute)) = (parts[0].parse::<u8>(), parts[1].parse::<u8>()) else {
        return false;
    };
    let seconds = parts[2];
    if !decimal(seconds) {
        return false;
    }
    let whole_seconds = seconds.split('.').next().unwrap_or(seconds);
    let Ok(second) = whole_seconds.parse::<u8>() else {
        return false;
    };
    (hour < 24 && minute < 60 && second < 60)
        || (hour == 24 && minute == 0 && second == 0 && !seconds.contains('.'))
}

fn split_timezone(value: &str) -> (&str, Option<&str>) {
    if let Some(core) = value.strip_suffix('Z') {
        return (core, Some("Z"));
    }
    if value.len() >= 6 {
        let index = value.len() - 6;
        let candidate = &value[index..];
        if matches!(candidate.as_bytes().first(), Some(b'+' | b'-'))
            && candidate.as_bytes().get(3) == Some(&b':')
        {
            return (&value[..index], Some(candidate));
        }
    }
    (value, None)
}

fn timezone_valid(timezone: Option<&str>) -> bool {
    let Some(timezone) = timezone else {
        return true;
    };
    if timezone == "Z" {
        return true;
    }
    let (Ok(hour), Ok(minute)) = (timezone[1..3].parse::<u8>(), timezone[4..6].parse::<u8>())
    else {
        return false;
    };
    minute < 60 && (hour < 14 || hour == 14 && minute == 0)
}

fn valid_day(year: Option<u64>, month: u8, day: u8) -> bool {
    let leap = year.is_some_and(|year| year % 4 == 0 && (year % 100 != 0 || year % 400 == 0));
    let maximum = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    day != 0 && day <= maximum
}

fn unsupported(expression: &str, location: &SourceLocation) -> CastableFailure {
    CastableFailure {
        detail: format!(
            "the private castability slice supports a path castable as one admitted xs built-in type: {expression}"
        ),
        location: location.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{evaluate, evaluate_cast, evaluate_value, parse, parse_cast, variable_name};
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};
    use crate::xdm::atomic_value_experiment::BuiltinAtomicType;
    use crate::xdm::owned_tree_experiment::{Document, SourceLocation};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    #[test]
    fn recognizes_native_positive_and_negative_lexical_families() {
        let cases = [
            (BuiltinAtomicType::Boolean, "true", "abcd"),
            (BuiltinAtomicType::Integer, "43", "-1.23"),
            (BuiltinAtomicType::Decimal, "-1.23", "12.78e-2"),
            (BuiltinAtomicType::Float, "12.78e-2", "2006-05-16"),
            (BuiltinAtomicType::Duration, "P1Y2M3DT1H2M3S", "2006-05-16"),
            (BuiltinAtomicType::DayTimeDuration, "P3DT1H2M3S", "P1Y2M"),
            (BuiltinAtomicType::YearMonthDuration, "P1Y2M", "P3DT1H2M3S"),
            (
                BuiltinAtomicType::DateTime,
                "2002-10-10T12:00:00-05:00",
                "P1Y",
            ),
            (BuiltinAtomicType::Date, "2006-05-16", "23:17:00"),
            (BuiltinAtomicType::Time, "23:17:00", "2006-05-16"),
        ];
        for (target, accepted, rejected) in cases {
            assert!(
                target.accepts(accepted),
                "{target:?} should accept {accepted}"
            );
            assert!(
                !target.accepts(rejected),
                "{target:?} should reject {rejected}"
            );
        }
    }

    #[test]
    fn evaluates_one_atomized_path_and_charges_owned_domains() {
        let parsed = parse_document(
            "memory:castable",
            b"<root><value>43</value></root>",
            ParseLimits {
                max_events: 8,
                max_depth: 4,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let expression = parse(
            "//value castable as xs:integer",
            &SourceLocation {
                resource: "memory:stylesheet".to_owned(),
                span: 0..31,
            },
        )
        .expect("native castability shape should parse");
        let mut control = InvocationControl::unbounded();

        assert!(
            evaluate(
                &expression,
                &document,
                document.document_node(),
                &mut control
            )
            .expect("castability should evaluate")
        );
        assert_eq!(control.consumed(WorkDomain::XPathOperation), 1);
        assert!(control.consumed(WorkDomain::XPathNodeVisit) > 0);
        assert!(control.consumed(WorkDomain::XdmStringValueNode) > 0);
    }

    #[test]
    fn cast_values_retain_type_identity_for_variable_castability() {
        let parsed = parse_document(
            "memory:cast",
            b"<root><value>43</value></root>",
            ParseLimits {
                max_events: 8,
                max_depth: 4,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let location = SourceLocation {
            resource: "memory:stylesheet".to_owned(),
            span: 0..30,
        };
        let cast = parse_cast("//value cast as xs:integer", &location)
            .expect("native cast shape should parse");
        let mut control = InvocationControl::unbounded();
        let value = evaluate_cast(&cast, &document, document.document_node(), &mut control)
            .expect("valid integer cast should produce a value");
        let to_string = parse("$value castable as xs:string", &location)
            .expect("typed variable castability should parse");
        let to_untyped = parse("$value castable as xs:untypedAtomic", &location)
            .expect("untypedAtomic target should parse");

        assert_eq!(value.atomic_type(), BuiltinAtomicType::Integer);
        assert_eq!(value.lexical(), "43");
        assert_eq!(variable_name(&to_string), Some("value"));
        assert!(evaluate_value(&to_string, &value, &mut control).expect("castability"));
        assert!(evaluate_value(&to_untyped, &value, &mut control).expect("castability"));
    }

    #[test]
    fn applies_the_admitted_cross_numeric_castability_matrix() {
        let location = SourceLocation {
            resource: "memory:stylesheet".to_owned(),
            span: 0..40,
        };
        let sources = [
            ("boolean", BuiltinAtomicType::Boolean, "true"),
            ("integer", BuiltinAtomicType::Integer, "43"),
            ("decimal", BuiltinAtomicType::Decimal, "-1.23"),
            ("float", BuiltinAtomicType::Float, "12.78e-2"),
            ("double", BuiltinAtomicType::Double, "12.78e-2"),
        ];
        let targets = ["float", "double", "decimal", "integer"];
        let mut control = InvocationControl::unbounded();

        for (name, source_type, lexical) in sources {
            let value = crate::xdm::atomic_value_experiment::AtomicValue::from_validated_lexical(
                source_type,
                lexical,
            );
            for target in targets {
                let expression = parse(&format!("${name} castable as xs:{target}"), &location)
                    .expect("native numeric conversion edge should parse");
                assert!(
                    evaluate_value(&expression, &value, &mut control).expect("castability"),
                    "{source_type:?} should be castable as xs:{target}"
                );
            }
        }

        for lexical in ["INF", "-INF", "NaN"] {
            for source_type in [BuiltinAtomicType::Float, BuiltinAtomicType::Double] {
                let value =
                    crate::xdm::atomic_value_experiment::AtomicValue::from_validated_lexical(
                        source_type,
                        lexical,
                    );
                for target in ["decimal", "integer"] {
                    let expression = parse(&format!("$value castable as xs:{target}"), &location)
                        .expect("value-sensitive numeric conversion edge should parse");
                    assert!(
                        !evaluate_value(&expression, &value, &mut control).expect("castability")
                    );
                }
            }
        }

        for (source_type, lexical) in [
            (BuiltinAtomicType::String, "abcd"),
            (BuiltinAtomicType::Duration, "P1Y2M3DT1H2M3S"),
            (BuiltinAtomicType::DateTime, "2002-10-10T12:00:00-05:00"),
        ] {
            let value = crate::xdm::atomic_value_experiment::AtomicValue::from_validated_lexical(
                source_type,
                lexical,
            );
            let expression = parse("$value castable as xs:float", &location)
                .expect("native incompatible edge should parse");
            assert!(
                !evaluate_value(&expression, &value, &mut control).expect("castability"),
                "{source_type:?} should not be castable as xs:float"
            );
        }
    }

    #[test]
    fn applies_the_admitted_duration_family_castability_matrix() {
        let location = SourceLocation {
            resource: "memory:stylesheet".to_owned(),
            span: 0..50,
        };
        let mut control = InvocationControl::unbounded();
        for (source_type, lexical, target) in [
            (
                BuiltinAtomicType::Duration,
                "P1Y2M3DT1H2M3S",
                "dayTimeDuration",
            ),
            (
                BuiltinAtomicType::Duration,
                "P1Y2M3DT1H2M3S",
                "yearMonthDuration",
            ),
            (
                BuiltinAtomicType::DayTimeDuration,
                "P3DT1H2M3S",
                "yearMonthDuration",
            ),
            (
                BuiltinAtomicType::YearMonthDuration,
                "P1Y2M",
                "dayTimeDuration",
            ),
        ] {
            let value = crate::xdm::atomic_value_experiment::AtomicValue::from_validated_lexical(
                source_type,
                lexical,
            );
            let expression = parse(&format!("$value castable as xs:{target}"), &location)
                .expect("native duration conversion edge should parse");
            assert!(
                evaluate_value(&expression, &value, &mut control).expect("castability"),
                "{source_type:?} should be castable as xs:{target}"
            );
        }

        for (source_type, lexical, target) in [
            (BuiltinAtomicType::Date, "2006-05-16", "yearMonthDuration"),
            (BuiltinAtomicType::Time, "23:17:00", "dayTimeDuration"),
            (BuiltinAtomicType::Boolean, "true", "yearMonthDuration"),
            (BuiltinAtomicType::Integer, "43", "dayTimeDuration"),
        ] {
            let value = crate::xdm::atomic_value_experiment::AtomicValue::from_validated_lexical(
                source_type,
                lexical,
            );
            let expression = parse(&format!("$value castable as xs:{target}"), &location)
                .expect("native incompatible duration edge should parse");
            assert!(
                !evaluate_value(&expression, &value, &mut control).expect("castability"),
                "{source_type:?} should not be castable as xs:{target}"
            );
        }
    }
}
