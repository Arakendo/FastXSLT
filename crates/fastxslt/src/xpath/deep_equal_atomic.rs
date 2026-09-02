//! Private atomic parsing and value-comparison semantics for the admitted
//! `deep-equal` experiment slice.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AtomicSequence(Vec<AtomicValue>);

const MAX_FOLDED_LITERAL_RANGE_ITEMS: usize = 1_024;

impl AtomicSequence {
    pub(super) fn len(&self) -> usize {
        self.0.len()
    }

    pub(super) fn item_equals(
        &self,
        other: &Self,
        index: usize,
        collation: AtomicCollation,
    ) -> bool {
        atomic_values_equal_with_collation(&self.0[index], &other.0[index], collation)
    }

    pub(super) fn supports_collation(&self, collation: AtomicCollation) -> bool {
        match collation {
            AtomicCollation::Codepoint => true,
            AtomicCollation::HtmlAsciiCaseInsensitive => self
                .0
                .iter()
                .all(|value| matches!(value, AtomicValue::String(_))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AtomicCollation {
    Codepoint,
    HtmlAsciiCaseInsensitive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AtomicValue {
    Integer(i128),
    Decimal(ExactDecimal),
    Float(u32),
    Double(u64),
    Boolean(bool),
    String(String),
    AnyUri(String),
    QName {
        namespace_uri: String,
        local_name: String,
    },
    HexBinary(Vec<u8>),
    Base64Binary(Vec<u8>),
    Date(DateValue),
    DateTime {
        date: DateValue,
        time: TimeValue,
    },
    Time(TimeValue),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DateValue {
    year: u16,
    month: u8,
    day: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TimeValue {
    hour: u8,
    minute: u8,
    second: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ExactDecimal {
    coefficient: i128,
    scale: u32,
}

pub(super) fn split_top_level_once(value: &str) -> Option<(&str, &str)> {
    let mut depth = 0_u32;
    let mut in_string = false;
    for (index, character) in value.char_indices() {
        match character {
            '"' => in_string = !in_string,
            '(' if !in_string => depth = depth.checked_add(1)?,
            ')' if !in_string => depth = depth.checked_sub(1)?,
            ',' if !in_string && depth == 0 => return Some((&value[..index], &value[index + 1..])),
            _ => {}
        }
    }
    None
}

pub(super) fn parse_sequence(expression: &str) -> Option<AtomicSequence> {
    parse_atomic_sequence(expression).map(AtomicSequence)
}

fn parse_atomic_sequence(expression: &str) -> Option<Vec<AtomicValue>> {
    let expression = expression.trim();
    if let Some(indexes) = parse_literal_index_of(expression) {
        return Some(indexes);
    }
    if let Some(values) = parse_literal_reverse(expression) {
        return Some(values);
    }
    if let Some(values) = parse_literal_integer_range(expression) {
        return Some(values);
    }
    if let Some(inner) = strip_outer_parentheses(expression) {
        let inner = inner.trim();
        if inner.is_empty() {
            return Some(Vec::new());
        }
        return parse_atomic_sequence_items(inner);
    }
    parse_atomic_value(expression).map(|value| vec![value])
}

fn parse_literal_reverse(expression: &str) -> Option<Vec<AtomicValue>> {
    let inner = expression.strip_prefix("reverse(")?.strip_suffix(')')?;
    let mut values = parse_atomic_sequence(inner.trim())?;
    values.reverse();
    Some(values)
}

fn parse_literal_integer_range(expression: &str) -> Option<Vec<AtomicValue>> {
    let (start, end) = expression.split_once(" to ")?;
    if end.contains(" to ") {
        return None;
    }
    let start = start.trim().parse::<i128>().ok()?;
    let end = end.trim().parse::<i128>().ok()?;
    if start > end {
        return Some(Vec::new());
    }
    let item_count = end.checked_sub(start)?.checked_add(1)?;
    let item_count = usize::try_from(item_count).ok()?;
    if item_count > MAX_FOLDED_LITERAL_RANGE_ITEMS {
        return None;
    }
    (0..item_count)
        .map(|offset| {
            i128::try_from(offset)
                .ok()
                .and_then(|offset| start.checked_add(offset))
                .map(AtomicValue::Integer)
        })
        .collect()
}

fn parse_literal_index_of(expression: &str) -> Option<Vec<AtomicValue>> {
    let inner = expression.strip_prefix("index-of(")?.strip_suffix(')')?;
    let (input, sought) = split_top_level_once(inner)?;
    let input = parse_atomic_sequence(input.trim())?;
    let sought = parse_atomic_sequence(sought.trim())?;
    let sought = (sought.len() == 1).then(|| sought.first())??;
    input
        .iter()
        .enumerate()
        .filter(|(_, item)| atomic_values_equal(item, sought))
        .map(|(index, _)| {
            i128::try_from(index.checked_add(1)?)
                .ok()
                .map(AtomicValue::Integer)
        })
        .collect()
}

fn parse_atomic_sequence_items(expression: &str) -> Option<Vec<AtomicValue>> {
    if let Some((left, right)) = split_top_level_once(expression) {
        let mut values = parse_atomic_sequence(left)?;
        values.extend(parse_atomic_sequence_items(right)?);
        Some(values)
    } else {
        parse_atomic_sequence(expression)
    }
}

fn strip_outer_parentheses(expression: &str) -> Option<&str> {
    if !expression.starts_with('(') || !expression.ends_with(')') {
        return None;
    }
    let mut depth = 0_u32;
    let mut in_string = false;
    let last = expression.len() - 1;
    for (index, character) in expression.char_indices() {
        match character {
            '"' => in_string = !in_string,
            '(' if !in_string => depth = depth.checked_add(1)?,
            ')' if !in_string => {
                depth = depth.checked_sub(1)?;
                if depth == 0 && index != last {
                    return None;
                }
            }
            _ => {}
        }
    }
    (!in_string && depth == 0).then_some(&expression[1..last])
}

fn parse_atomic_value(expression: &str) -> Option<AtomicValue> {
    if let Some((namespace_uri, local_name)) = parse_qname_constructor(expression) {
        return Some(AtomicValue::QName {
            namespace_uri,
            local_name,
        });
    }
    if let Some(value) = constructor_lexical(expression, "xs:hexBinary").and_then(parse_hex_binary)
    {
        return Some(AtomicValue::HexBinary(value));
    }
    if let Some(value) =
        constructor_lexical(expression, "xs:base64Binary").and_then(parse_base64_binary)
    {
        return Some(AtomicValue::Base64Binary(value));
    }
    if let Some(value) = expression
        .strip_prefix("xs:anyURI(\"")
        .and_then(|value| value.strip_suffix("\")"))
    {
        return (!value.contains('"')).then(|| AtomicValue::AnyUri(value.to_owned()));
    }
    if let Some(value) = expression
        .strip_prefix("xs:string(\"")
        .and_then(|value| value.strip_suffix("\")"))
    {
        return (!value.contains('"')).then(|| AtomicValue::String(value.to_owned()));
    }
    if let Some(value) =
        constructor_lexical(expression, "xs:NCName").filter(|value| is_ascii_ncname(value))
    {
        return Some(AtomicValue::String(value.to_owned()));
    }
    if let Some(value) = expression
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return (!value.contains('"')).then(|| AtomicValue::String(value.to_owned()));
    }
    if let Some(value) =
        constructor_lexical(expression, "xs:integer").and_then(|value| value.parse::<i128>().ok())
    {
        return Some(AtomicValue::Integer(value));
    }
    if let Some(value) =
        constructor_lexical(expression, "xs:decimal").and_then(parse_decimal_lexical)
    {
        return Some(AtomicValue::Decimal(value));
    }
    if let Some(value) = constructor_lexical(expression, "xs:float")
        .and_then(parse_float)
        .map(f32::to_bits)
    {
        return Some(AtomicValue::Float(value));
    }
    if let Some(value) = constructor_lexical(expression, "xs:double")
        .and_then(parse_double)
        .map(f64::to_bits)
    {
        return Some(AtomicValue::Double(value));
    }
    if let Some(value) = constructor_lexical(expression, "xs:boolean").and_then(parse_boolean) {
        return Some(AtomicValue::Boolean(value));
    }
    if let Some(value) = constructor_lexical(expression, "xs:date").and_then(parse_date) {
        return Some(AtomicValue::Date(value));
    }
    if let Some((date, time)) = constructor_lexical(expression, "xs:dateTime")
        .and_then(|value| value.split_once('T'))
        .and_then(|(date, time)| Some((parse_date(date)?, parse_time(time)?)))
    {
        return Some(AtomicValue::DateTime { date, time });
    }
    if let Some(value) = constructor_lexical(expression, "xs:time").and_then(parse_time) {
        return Some(AtomicValue::Time(value));
    }
    match expression {
        "true()" => return Some(AtomicValue::Boolean(true)),
        "false()" => return Some(AtomicValue::Boolean(false)),
        _ => {}
    }
    if expression.contains(['e', 'E']) {
        return expression
            .parse::<f64>()
            .ok()
            .map(f64::to_bits)
            .map(AtomicValue::Double);
    }
    expression.parse::<i128>().ok().map(AtomicValue::Integer)
}

fn parse_qname_constructor(expression: &str) -> Option<(String, String)> {
    let inner = expression.strip_prefix("QName(")?.strip_suffix(')')?;
    let (namespace_uri, lexical_qname) = split_top_level_once(inner)?;
    let namespace_uri = parse_quoted_string(namespace_uri.trim())?;
    let lexical_qname = parse_quoted_string(lexical_qname.trim())?;
    let (prefix, local_name) = lexical_qname
        .split_once(':')
        .map_or((None, lexical_qname), |(prefix, local_name)| {
            (Some(prefix), local_name)
        });
    if !is_ascii_ncname(local_name)
        || prefix.is_some_and(|value| !is_ascii_ncname(value))
        || (namespace_uri.is_empty() && prefix.is_some())
        || local_name.contains(':')
    {
        return None;
    }
    Some((namespace_uri.to_owned(), local_name.to_owned()))
}

fn parse_quoted_string(value: &str) -> Option<&str> {
    let value = value.strip_prefix('"')?.strip_suffix('"')?;
    (!value.contains('"')).then_some(value)
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}

fn parse_hex_binary(lexical: &str) -> Option<Vec<u8>> {
    let bytes = lexical.as_bytes();
    if bytes.len() % 2 != 0 {
        return None;
    }
    bytes
        .chunks_exact(2)
        .map(|pair| Some((hex_digit(pair[0])? << 4) | hex_digit(pair[1])?))
        .collect()
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn parse_base64_binary(lexical: &str) -> Option<Vec<u8>> {
    let bytes = lexical.as_bytes();
    if bytes.len() % 4 != 0 {
        return None;
    }
    let chunk_count = bytes.len() / 4;
    let mut decoded = Vec::with_capacity(chunk_count.checked_mul(3)?);
    for (index, chunk) in bytes.chunks_exact(4).enumerate() {
        let last = index + 1 == chunk_count;
        let first = base64_digit(chunk[0])?;
        let second = base64_digit(chunk[1])?;
        let third = (chunk[2] != b'=').then(|| base64_digit(chunk[2])).flatten();
        let fourth = (chunk[3] != b'=').then(|| base64_digit(chunk[3])).flatten();
        if (!last && (third.is_none() || fourth.is_none()))
            || (third.is_none() && fourth.is_some())
            || (third.is_none() && second & 0x0f != 0)
            || (third.is_some() && fourth.is_none() && third? & 0x03 != 0)
        {
            return None;
        }
        let third = third.unwrap_or(0);
        let fourth = fourth.unwrap_or(0);
        decoded.push((first << 2) | (second >> 4));
        if chunk[2] != b'=' {
            decoded.push((second << 4) | (third >> 2));
        }
        if chunk[3] != b'=' {
            decoded.push((third << 6) | fourth);
        }
    }
    Some(decoded)
}

fn base64_digit(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn constructor_lexical<'a>(expression: &'a str, name: &str) -> Option<&'a str> {
    let inner = expression
        .strip_prefix(name)?
        .strip_prefix('(')?
        .strip_suffix(')')?;
    if let Some(quoted) = inner
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
    {
        return (!quoted.contains('"')).then_some(quoted);
    }
    Some(inner)
}

fn parse_float(lexical: &str) -> Option<f32> {
    match lexical {
        "INF" => Some(f32::INFINITY),
        "-INF" => Some(f32::NEG_INFINITY),
        "NaN" => Some(f32::NAN),
        value => value.parse().ok(),
    }
}

fn parse_double(lexical: &str) -> Option<f64> {
    match lexical {
        "INF" => Some(f64::INFINITY),
        "-INF" => Some(f64::NEG_INFINITY),
        "NaN" => Some(f64::NAN),
        value => value.parse().ok(),
    }
}

fn parse_boolean(lexical: &str) -> Option<bool> {
    match lexical {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}

fn parse_date(lexical: &str) -> Option<DateValue> {
    let mut fields = lexical.split('-');
    let year = parse_fixed_digits(fields.next()?, 4)?;
    let month = u8::try_from(parse_fixed_digits(fields.next()?, 2)?).ok()?;
    let day = u8::try_from(parse_fixed_digits(fields.next()?, 2)?).ok()?;
    if fields.next().is_some() || !(1..=12).contains(&month) {
        return None;
    }
    let maximum_day = match month {
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 31,
    };
    (day > 0 && day <= maximum_day).then_some(DateValue { year, month, day })
}

fn parse_time(lexical: &str) -> Option<TimeValue> {
    let mut fields = lexical.split(':');
    let hour = u8::try_from(parse_fixed_digits(fields.next()?, 2)?).ok()?;
    let minute = u8::try_from(parse_fixed_digits(fields.next()?, 2)?).ok()?;
    let second = u8::try_from(parse_fixed_digits(fields.next()?, 2)?).ok()?;
    if fields.next().is_some() || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some(TimeValue {
        hour,
        minute,
        second,
    })
}

fn parse_fixed_digits(value: &str, width: usize) -> Option<u16> {
    (value.len() == width && value.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| value.parse().ok())
        .flatten()
}

fn is_leap_year(year: u16) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub(super) fn parse_decimal(expression: &str) -> Option<ExactDecimal> {
    let lexical = expression
        .strip_prefix("(xs:decimal(\"")
        .and_then(|value| value.strip_suffix("\"))"))?;
    parse_decimal_lexical(lexical)
}

fn parse_decimal_lexical(lexical: &str) -> Option<ExactDecimal> {
    let (negative, magnitude) = lexical
        .strip_prefix('-')
        .map_or((false, lexical), |value| (true, value));
    let (integer, fraction) = magnitude.split_once('.').unwrap_or((magnitude, ""));
    if integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let mut scale = u32::try_from(fraction.len()).ok()?;
    let digits = format!("{integer}{fraction}");
    let mut coefficient = digits.parse::<i128>().ok()?;
    while scale > 0 && coefficient % 10 == 0 {
        coefficient /= 10;
        scale -= 1;
    }
    if negative {
        coefficient = coefficient.checked_neg()?;
    }
    Some(ExactDecimal { coefficient, scale })
}

pub(super) fn parse_integer(expression: &str) -> Option<i128> {
    let int = expression
        .strip_prefix("(xs:int(\"")
        .and_then(|value| value.strip_suffix("\"))"))
        .and_then(|value| value.parse::<i32>().ok())
        .map(i128::from);
    let long = expression
        .strip_prefix("(xs:long(\"")
        .and_then(|value| value.strip_suffix("\"))"))
        .and_then(|value| value.parse::<i64>().ok())
        .map(i128::from);
    let short = expression
        .strip_prefix("(xs:short(\"")
        .and_then(|value| value.strip_suffix("\"))"))
        .and_then(|value| value.parse::<i16>().ok())
        .map(i128::from);
    let unsigned_short = expression
        .strip_prefix("(xs:unsignedShort(\"")
        .and_then(|value| value.strip_suffix("\"))"))
        .and_then(|value| value.parse::<u16>().ok())
        .map(i128::from);
    let unsigned_long = expression
        .strip_prefix("(xs:unsignedLong(\"")
        .and_then(|value| value.strip_suffix("\"))"))
        .and_then(|value| value.parse::<u64>().ok())
        .map(i128::from);
    let negative_integer = expression
        .strip_prefix("(xs:negativeInteger(\"")
        .and_then(|value| value.strip_suffix("\"))"))
        .and_then(|value| value.parse::<i128>().ok())
        .filter(|value| *value < 0);
    let positive_integer = expression
        .strip_prefix("(xs:positiveInteger(\"")
        .and_then(|value| value.strip_suffix("\"))"))
        .and_then(|value| value.parse::<i128>().ok())
        .filter(|value| *value > 0);
    let non_positive_integer = expression
        .strip_prefix("(xs:nonPositiveInteger(\"")
        .and_then(|value| value.strip_suffix("\"))"))
        .and_then(|value| value.parse::<i128>().ok())
        .filter(|value| *value <= 0);
    let non_negative_integer = expression
        .strip_prefix("(xs:nonNegativeInteger(\"")
        .and_then(|value| value.strip_suffix("\"))"))
        .and_then(|value| value.parse::<i128>().ok())
        .filter(|value| *value >= 0);
    int.or(long)
        .or(short)
        .or(unsigned_short)
        .or(unsigned_long)
        .or(negative_integer)
        .or(positive_integer)
        .or(non_positive_integer)
        .or(non_negative_integer)
        .or_else(|| {
            expression
                .strip_prefix("(xs:integer(\"")
                .and_then(|value| value.strip_suffix("\"))"))
                .and_then(|value| value.parse::<i128>().ok())
        })
}

fn atomic_values_equal(left: &AtomicValue, right: &AtomicValue) -> bool {
    atomic_values_equal_with_collation(left, right, AtomicCollation::Codepoint)
}

// XPath numeric promotion requires exact equality after the specified lossy
// conversion; an epsilon comparison would change the language semantics.
#[allow(clippy::cast_precision_loss, clippy::float_cmp)]
fn atomic_values_equal_with_collation(
    left: &AtomicValue,
    right: &AtomicValue,
    collation: AtomicCollation,
) -> bool {
    if atomic_is_nan(left) && atomic_is_nan(right) {
        return true;
    }
    match (left, right) {
        (AtomicValue::String(left), AtomicValue::String(right)) => match collation {
            AtomicCollation::Codepoint => left == right,
            AtomicCollation::HtmlAsciiCaseInsensitive => left.eq_ignore_ascii_case(right),
        },
        (AtomicValue::Integer(left), AtomicValue::Decimal(right))
        | (AtomicValue::Decimal(right), AtomicValue::Integer(left)) => {
            right.scale == 0 && right.coefficient == *left
        }
        (AtomicValue::String(left), AtomicValue::AnyUri(right))
        | (AtomicValue::AnyUri(right), AtomicValue::String(left)) => left == right,
        (AtomicValue::Integer(left), AtomicValue::Float(right))
        | (AtomicValue::Float(right), AtomicValue::Integer(left)) => {
            (*left as f32) == f32::from_bits(*right)
        }
        (AtomicValue::Integer(left), AtomicValue::Double(right))
        | (AtomicValue::Double(right), AtomicValue::Integer(left)) => {
            (*left as f64) == f64::from_bits(*right)
        }
        (AtomicValue::Decimal(left), AtomicValue::Float(right))
        | (AtomicValue::Float(right), AtomicValue::Decimal(left)) => {
            decimal_as_f32(*left) == f32::from_bits(*right)
        }
        (AtomicValue::Decimal(left), AtomicValue::Double(right))
        | (AtomicValue::Double(right), AtomicValue::Decimal(left)) => {
            decimal_as_f64(*left) == f64::from_bits(*right)
        }
        (AtomicValue::Float(left), AtomicValue::Double(right))
        | (AtomicValue::Double(right), AtomicValue::Float(left)) => {
            f64::from(f32::from_bits(*left)) == f64::from_bits(*right)
        }
        (AtomicValue::Float(left), AtomicValue::Float(right)) => {
            f32::from_bits(*left) == f32::from_bits(*right)
        }
        (AtomicValue::Double(left), AtomicValue::Double(right)) => {
            f64::from_bits(*left) == f64::from_bits(*right)
        }
        _ => left == right,
    }
}

fn atomic_is_nan(value: &AtomicValue) -> bool {
    match value {
        AtomicValue::Float(bits) => f32::from_bits(*bits).is_nan(),
        AtomicValue::Double(bits) => f64::from_bits(*bits).is_nan(),
        _ => false,
    }
}

#[allow(clippy::cast_precision_loss)]
fn decimal_as_f32(value: ExactDecimal) -> f32 {
    let scale = i32::try_from(value.scale).unwrap_or(i32::MAX);
    (value.coefficient as f32) / 10_f32.powi(scale)
}

#[allow(clippy::cast_precision_loss)]
fn decimal_as_f64(value: ExactDecimal) -> f64 {
    let scale = i32::try_from(value.scale).unwrap_or(i32::MAX);
    (value.coefficient as f64) / 10_f64.powi(scale)
}

#[cfg(test)]
mod tests {
    use super::{ExactDecimal, parse_decimal, parse_integer, parse_sequence};

    fn sequences_equal(left: &str, right: &str) -> Option<bool> {
        let left = parse_sequence(left)?;
        let right = parse_sequence(right)?;
        if left.len() != right.len() {
            return Some(false);
        }
        Some(
            (0..left.len())
                .all(|index| left.item_equals(&right, index, super::AtomicCollation::Codepoint)),
        )
    }

    #[test]
    fn compares_admitted_string_derived_ncname_by_value() {
        assert_eq!(sequences_equal("\"a\"", "xs:NCName(\"a\")"), Some(true));
        assert_eq!(sequences_equal("\"a\"", "xs:NCName(\"b\")"), Some(false));
        assert!(parse_sequence("xs:NCName(\"1bad\")").is_none());
    }

    #[test]
    fn enforces_admitted_integer_constructor_value_spaces() {
        assert_eq!(
            parse_integer("(xs:int(\"-2147483648\"))"),
            Some(-2_147_483_648)
        );
        assert_eq!(
            parse_integer("(xs:long(\"9223372036854775807\"))"),
            Some(9_223_372_036_854_775_807)
        );
        assert_eq!(
            parse_integer("(xs:unsignedLong(\"18446744073709551615\"))"),
            Some(18_446_744_073_709_551_615)
        );
        assert_eq!(parse_integer("(xs:positiveInteger(\"0\"))"), None);
        assert_eq!(parse_integer("(xs:unsignedShort(\"65536\"))"), None);
    }

    #[test]
    fn normalizes_exact_decimals_without_binary_floating_point() {
        assert_eq!(
            parse_decimal("(xs:decimal(\"1.00\"))"),
            Some(ExactDecimal {
                coefficient: 1,
                scale: 0,
            })
        );
        assert_eq!(
            parse_decimal("(xs:decimal(\"-12.30\"))"),
            Some(ExactDecimal {
                coefficient: -123,
                scale: 1,
            })
        );
        assert_eq!(
            sequences_equal("xs:decimal(\"1\")", "xs:decimal(1.0)"),
            Some(true)
        );
        assert_eq!(
            sequences_equal("xs:integer(\"1\")", "xs:integer(1)"),
            Some(true)
        );
    }

    #[test]
    fn flattens_nested_atomic_sequences_and_preserves_order() {
        assert_eq!(sequences_equal("((), (1, 2))", "(1, 2)"), Some(true));
        assert_eq!(sequences_equal("(1, 2)", "(2, 1)"), Some(false));
        assert_eq!(sequences_equal("()", "(())"), Some(true));
        assert_eq!(sequences_equal("(1, 2, 3)", "(1, (2, 3))"), Some(true));
        assert!(parse_sequence("1, 2, 3").is_none());
    }

    #[test]
    fn applies_only_the_admitted_atomic_promotions() {
        assert_eq!(
            sequences_equal("xs:anyURI(\"urn:example\")", "xs:string(\"urn:example\")"),
            Some(true)
        );
        assert_eq!(
            sequences_equal("xs:integer(1)", "xs:decimal(1.0)"),
            Some(true)
        );
        assert_eq!(
            sequences_equal("xs:integer(1)", "xs:decimal(1.01)"),
            Some(false)
        );
        assert_eq!(
            sequences_equal("xs:float(\"NaN\")", "xs:double(\"NaN\")"),
            Some(true)
        );
    }

    #[test]
    fn validates_boolean_and_calendar_lexicals_locally() {
        assert_eq!(sequences_equal("xs:boolean(\"1\")", "true()"), Some(true));
        assert!(parse_sequence("xs:boolean(\"yes\")").is_none());
        assert_eq!(
            sequences_equal("xs:date(\"2000-02-29\")", "xs:date(\"2000-02-29\")"),
            Some(true)
        );
        assert!(parse_sequence("xs:date(\"1993-02-29\")").is_none());
        assert!(parse_sequence("xs:time(\"24:01:00\")").is_none());
        assert!(parse_sequence("xs:dateTime(\"1972-13-01T00:00:00\")").is_none());
    }

    #[test]
    fn retains_qname_expanded_names_without_prefix_identity() {
        assert_eq!(
            sequences_equal(
                "QName(\"urn:example\", \"first:name\")",
                "QName(\"urn:example\", \"second:name\")"
            ),
            Some(true)
        );
        assert_eq!(
            sequences_equal(
                "QName(\"urn:example\", \"name\")",
                "QName(\"urn:other\", \"name\")"
            ),
            Some(false)
        );
        assert_eq!(
            sequences_equal("QName(\"urn:example\", \"name\")", "3e2"),
            Some(false)
        );
        assert!(parse_sequence("QName(\"\", \"prefix:name\")").is_none());
        assert!(parse_sequence("QName(\"urn:example\", \"1name\")").is_none());
    }

    #[test]
    fn decodes_binary_values_before_comparison() {
        assert_eq!(
            sequences_equal("xs:hexBinary(\"ff00\")", "xs:hexBinary(\"FF00\")"),
            Some(true)
        );
        assert_eq!(
            sequences_equal(
                "xs:base64Binary(\"ZmFzdA==\")",
                "xs:base64Binary(\"ZmFzdA==\")"
            ),
            Some(true)
        );
        assert!(parse_sequence("xs:hexBinary(\"0\")").is_none());
        assert!(parse_sequence("xs:base64Binary(\"A===\")").is_none());
    }

    #[test]
    fn folds_literal_index_of_results_in_position_order() {
        assert_eq!(
            sequences_equal("index-of((20, 40, 20), 20)", "(1, 3)"),
            Some(true)
        );
        assert_eq!(sequences_equal("index-of((20), 40)", "()"), Some(true));
        assert!(parse_sequence("index-of((20), ())").is_none());
    }

    #[test]
    fn bounds_literal_range_and_reverse_folding() {
        assert_eq!(sequences_equal("0 to -5", "()"), Some(true));
        assert_eq!(sequences_equal("reverse(1 to 3)", "(3, 2, 1)"), Some(true));
        assert!(parse_sequence("1 to 1025").is_none());
    }
}
