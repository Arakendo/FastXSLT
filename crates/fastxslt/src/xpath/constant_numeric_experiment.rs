//! Checked exact-rational constants for the admitted conditional-expression slice.

use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConstantNumericFailure {
    Invalid,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntegralFunction {
    Floor,
    Ceiling,
    Round,
}

pub(crate) fn compare(left: &str, right: &str) -> Result<Ordering, ConstantNumericFailure> {
    let left = evaluate(left)?;
    let right = evaluate(right)?;
    left.numerator
        .checked_mul(right.denominator)
        .and_then(|left_scaled| {
            right
                .numerator
                .checked_mul(left.denominator)
                .map(|right_scaled| left_scaled.cmp(&right_scaled))
        })
        .ok_or(ConstantNumericFailure::Invalid)
}

pub(crate) fn fold_integral_function(expression: &str) -> Option<String> {
    let expression = expression.trim();
    integral_function_call(expression)?;
    let value = evaluate(expression).ok()?;
    (value.denominator == 1).then(|| value.numerator.to_string())
}

pub(crate) fn fold_exact_integral_arithmetic(expression: &str) -> Option<String> {
    let expression = expression.trim();
    contains_binary_arithmetic_operator(expression)?;
    let value = evaluate(expression).ok()?;
    (value.numerator.rem_euclid(value.denominator) == 0)
        .then(|| value.numerator.div_euclid(value.denominator).to_string())
}

pub(crate) fn fold_number_conversion(expression: &str) -> Option<String> {
    let argument = number_function_call(expression)?;
    if argument.is_empty() {
        return None;
    }
    if let Some(lexical) = xpath_string_literal(argument) {
        return evaluate_number_lexical(lexical).ok();
    }
    canonical_finite_decimal(argument.trim())
}

pub(crate) fn number_function_call(expression: &str) -> Option<&str> {
    let argument = expression
        .trim()
        .strip_prefix("number")?
        .trim_start_matches([' ', '\t', '\r', '\n'])
        .strip_prefix('(')?
        .strip_suffix(')')?
        .trim();
    Some(argument)
}

pub(crate) fn evaluate_number_lexical(lexical: &str) -> Result<String, ConstantNumericFailure> {
    let lexical = lexical.trim();
    if let Some(value) = canonical_finite_decimal(lexical) {
        return Ok(value);
    }
    let numeric_extension = lexical
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'+' | b'-' | b'.' | b'e' | b'E'))
        && (lexical.contains(['e', 'E']) || lexical.starts_with('.') || lexical.ends_with('.'));
    if numeric_extension
        || matches!(
            lexical,
            "NaN" | "INF" | "+INF" | "-INF" | "Infinity" | "+Infinity" | "-Infinity"
        )
    {
        Err(ConstantNumericFailure::Unsupported)
    } else {
        Ok("NaN".to_owned())
    }
}

fn xpath_string_literal(expression: &str) -> Option<&str> {
    for quote in ['\'', '"'] {
        if let Some(value) = expression
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
        {
            return (!value.contains(quote)).then_some(value);
        }
    }
    None
}

fn canonical_finite_decimal(lexical: &str) -> Option<String> {
    if !is_admitted_decimal_lexical(lexical) {
        return None;
    }
    let (negative, unsigned) = lexical
        .strip_prefix('-')
        .map_or((false, lexical), |value| (true, value));
    let unsigned = unsigned.strip_prefix('+').unwrap_or(unsigned);
    let (integer, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, ""), |parts| parts);
    let integer = integer.trim_start_matches('0');
    let integer = if integer.is_empty() { "0" } else { integer };
    let fraction = fraction.trim_end_matches('0');
    let is_zero = integer == "0" && fraction.is_empty();
    let sign = if negative && !is_zero { "-" } else { "" };
    if fraction.is_empty() {
        Some(format!("{sign}{integer}"))
    } else {
        Some(format!("{sign}{integer}.{fraction}"))
    }
}

fn contains_binary_arithmetic_operator(expression: &str) -> Option<()> {
    let first = expression
        .char_indices()
        .find_map(|(index, character)| (!character.is_ascii_whitespace()).then_some(index))?;
    expression
        .char_indices()
        .any(|(index, character)| {
            matches!(character, '+' | '*') || (character == '-' && index > first)
        })
        .then_some(())
        .or_else(|| {
            expression
                .split_ascii_whitespace()
                .any(|token| matches!(token, "div" | "mod"))
                .then_some(())
        })
}

pub(crate) fn integral_function_call(expression: &str) -> Option<(IntegralFunction, &str)> {
    let expression = expression.trim();
    for (name, function) in [
        ("floor", IntegralFunction::Floor),
        ("ceiling", IntegralFunction::Ceiling),
        ("round", IntegralFunction::Round),
    ] {
        let Some(remainder) = expression.strip_prefix(name) else {
            continue;
        };
        let remainder = remainder.trim_start_matches([' ', '\t', '\r', '\n']);
        let argument = remainder.strip_prefix('(')?.strip_suffix(')')?.trim();
        if !argument.is_empty() {
            return Some((function, argument));
        }
    }
    None
}

pub(crate) fn evaluate_integral_lexical(
    function: IntegralFunction,
    lexical: &str,
) -> Result<String, ConstantNumericFailure> {
    let lexical = lexical.trim();
    if !is_admitted_decimal_lexical(lexical) {
        return Err(
            if lexical.contains(['e', 'E'])
                || matches!(lexical, "NaN" | "INF" | "+INF" | "-INF")
                || lexical.starts_with('.')
                || lexical.ends_with('.')
            {
                ConstantNumericFailure::Unsupported
            } else {
                ConstantNumericFailure::Invalid
            },
        );
    }
    let value = evaluate(lexical)?;
    let value = match function {
        IntegralFunction::Floor => value.floor(),
        IntegralFunction::Ceiling => value.ceiling()?,
        IntegralFunction::Round => value.round()?,
    };
    Ok(value.numerator.to_string())
}

fn is_admitted_decimal_lexical(lexical: &str) -> bool {
    let unsigned = lexical.strip_prefix(['+', '-']).unwrap_or(lexical);
    let mut parts = unsigned.split('.');
    let Some(integer) = parts.next() else {
        return false;
    };
    let fraction = parts.next();
    parts.next().is_none()
        && !integer.is_empty()
        && integer.bytes().all(|byte| byte.is_ascii_digit())
        && fraction.is_none_or(|fraction| {
            !fraction.is_empty() && fraction.bytes().all(|byte| byte.is_ascii_digit())
        })
}

pub(crate) fn fold_integral_equality(expression: &str) -> Option<bool> {
    let (left, right) = expression.split_once('=')?;
    if right.contains('=')
        || left.trim_end().ends_with('!')
        || !contains_integral_function(expression)
    {
        return None;
    }
    compare(left.trim(), right.trim())
        .ok()
        .map(std::cmp::Ordering::is_eq)
}

fn contains_integral_function(expression: &str) -> bool {
    ["floor", "ceiling", "round"].iter().any(|name| {
        expression.match_indices(name).any(|(index, _)| {
            let before_is_name =
                index > 0 && expression.as_bytes()[index - 1].is_ascii_alphanumeric();
            let remainder = &expression[index + name.len()..];
            !before_is_name
                && remainder
                    .trim_start_matches([' ', '\t', '\r', '\n'])
                    .starts_with('(')
        })
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rational {
    numerator: i128,
    denominator: i128,
}

impl Rational {
    const fn integer(value: i128) -> Self {
        Self {
            numerator: value,
            denominator: 1,
        }
    }

    fn add(self, right: Self) -> Result<Self, ConstantNumericFailure> {
        let numerator = self
            .numerator
            .checked_mul(right.denominator)
            .and_then(|left| {
                right
                    .numerator
                    .checked_mul(self.denominator)
                    .and_then(|right| left.checked_add(right))
            })
            .ok_or(ConstantNumericFailure::Invalid)?;
        let denominator = self
            .denominator
            .checked_mul(right.denominator)
            .ok_or(ConstantNumericFailure::Invalid)?;
        Ok(Self {
            numerator,
            denominator,
        })
    }

    fn subtract(self, right: Self) -> Result<Self, ConstantNumericFailure> {
        self.add(Self {
            numerator: right
                .numerator
                .checked_neg()
                .ok_or(ConstantNumericFailure::Invalid)?,
            denominator: right.denominator,
        })
    }

    fn multiply(self, right: Self) -> Result<Self, ConstantNumericFailure> {
        Ok(Self {
            numerator: self
                .numerator
                .checked_mul(right.numerator)
                .ok_or(ConstantNumericFailure::Invalid)?,
            denominator: self
                .denominator
                .checked_mul(right.denominator)
                .ok_or(ConstantNumericFailure::Invalid)?,
        })
    }

    fn divide(self, right: Self) -> Result<Self, ConstantNumericFailure> {
        if right.numerator == 0 {
            return Err(ConstantNumericFailure::Invalid);
        }
        let mut numerator = self
            .numerator
            .checked_mul(right.denominator)
            .ok_or(ConstantNumericFailure::Invalid)?;
        let mut denominator = self
            .denominator
            .checked_mul(right.numerator)
            .ok_or(ConstantNumericFailure::Invalid)?;
        if denominator < 0 {
            numerator = numerator
                .checked_neg()
                .ok_or(ConstantNumericFailure::Invalid)?;
            denominator = denominator
                .checked_neg()
                .ok_or(ConstantNumericFailure::Invalid)?;
        }
        Ok(Self {
            numerator,
            denominator,
        })
    }

    fn modulo(self, right: Self) -> Result<Self, ConstantNumericFailure> {
        if self.denominator != 1 || right.denominator != 1 || right.numerator <= 0 {
            return Err(ConstantNumericFailure::Unsupported);
        }
        Ok(Self::integer(self.numerator % right.numerator))
    }

    fn round(self) -> Result<Self, ConstantNumericFailure> {
        let doubled = self
            .numerator
            .checked_mul(2)
            .and_then(|value| value.checked_add(self.denominator))
            .ok_or(ConstantNumericFailure::Invalid)?;
        let divisor = self
            .denominator
            .checked_mul(2)
            .ok_or(ConstantNumericFailure::Invalid)?;
        Ok(Self::integer(doubled.div_euclid(divisor)))
    }

    fn floor(self) -> Self {
        Self::integer(self.numerator.div_euclid(self.denominator))
    }

    fn ceiling(self) -> Result<Self, ConstantNumericFailure> {
        let floor = self.numerator.div_euclid(self.denominator);
        let value = if self.numerator.rem_euclid(self.denominator) == 0 {
            floor
        } else {
            floor
                .checked_add(1)
                .ok_or(ConstantNumericFailure::Invalid)?
        };
        Ok(Self::integer(value))
    }
}

fn evaluate(expression: &str) -> Result<Rational, ConstantNumericFailure> {
    let mut parser = Parser {
        input: expression.as_bytes(),
        offset: 0,
    };
    let value = parser.additive()?;
    parser.whitespace();
    if parser.offset == parser.input.len() {
        Ok(value)
    } else {
        Err(ConstantNumericFailure::Unsupported)
    }
}

struct Parser<'a> {
    input: &'a [u8],
    offset: usize,
}

impl Parser<'_> {
    fn additive(&mut self) -> Result<Rational, ConstantNumericFailure> {
        let mut value = self.multiplicative()?;
        loop {
            self.whitespace();
            if self.consume(b'+') {
                value = value.add(self.multiplicative()?)?;
            } else if self.consume(b'-') {
                value = value.subtract(self.multiplicative()?)?;
            } else {
                return Ok(value);
            }
        }
    }

    fn multiplicative(&mut self) -> Result<Rational, ConstantNumericFailure> {
        let mut value = self.primary()?;
        loop {
            self.whitespace();
            if self.consume(b'*') {
                value = value.multiply(self.primary()?)?;
            } else if self.consume_keyword(b"div") {
                value = value.divide(self.primary()?)?;
            } else if self.consume_keyword(b"mod") {
                value = value.modulo(self.primary()?)?;
            } else {
                return Ok(value);
            }
        }
    }

    fn primary(&mut self) -> Result<Rational, ConstantNumericFailure> {
        self.whitespace();
        if self.consume(b'+') {
            return self.primary();
        }
        if self.consume(b'-') {
            let value = self.primary()?;
            return Ok(Rational {
                numerator: value
                    .numerator
                    .checked_neg()
                    .ok_or(ConstantNumericFailure::Invalid)?,
                denominator: value.denominator,
            });
        }
        if self.consume_keyword(b"floor") {
            self.whitespace();
            if !self.consume(b'(') {
                return Err(ConstantNumericFailure::Invalid);
            }
            let value = self.additive()?;
            self.whitespace();
            if !self.consume(b')') {
                return Err(ConstantNumericFailure::Invalid);
            }
            return Ok(value.floor());
        }
        if self.consume_keyword(b"ceiling") {
            self.whitespace();
            if !self.consume(b'(') {
                return Err(ConstantNumericFailure::Invalid);
            }
            let value = self.additive()?;
            self.whitespace();
            if !self.consume(b')') {
                return Err(ConstantNumericFailure::Invalid);
            }
            return value.ceiling();
        }
        if self.consume_keyword(b"round") {
            self.whitespace();
            if !self.consume(b'(') {
                return Err(ConstantNumericFailure::Invalid);
            }
            let value = self.additive()?;
            self.whitespace();
            if !self.consume(b')') {
                return Err(ConstantNumericFailure::Invalid);
            }
            return value.round();
        }
        if self.consume(b'(') {
            let value = self.additive()?;
            self.whitespace();
            if !self.consume(b')') {
                return Err(ConstantNumericFailure::Invalid);
            }
            return Ok(value);
        }
        let integer_start = self.offset;
        while self.input.get(self.offset).is_some_and(u8::is_ascii_digit) {
            self.offset += 1;
        }
        if self.offset == integer_start {
            return Err(ConstantNumericFailure::Unsupported);
        }
        let integer = std::str::from_utf8(&self.input[integer_start..self.offset])
            .expect("ASCII digits are valid UTF-8")
            .parse()
            .map_err(|_| ConstantNumericFailure::Invalid)?;
        if !self.consume(b'.') {
            return Ok(Rational::integer(integer));
        }
        let fraction_start = self.offset;
        while self.input.get(self.offset).is_some_and(u8::is_ascii_digit) {
            self.offset += 1;
        }
        if self.offset == fraction_start {
            return Err(ConstantNumericFailure::Invalid);
        }
        let fraction: i128 = std::str::from_utf8(&self.input[fraction_start..self.offset])
            .expect("ASCII digits are valid UTF-8")
            .parse()
            .map_err(|_| ConstantNumericFailure::Invalid)?;
        let fraction_digits = u32::try_from(self.offset - fraction_start)
            .map_err(|_| ConstantNumericFailure::Invalid)?;
        let denominator = 10_i128
            .checked_pow(fraction_digits)
            .ok_or(ConstantNumericFailure::Invalid)?;
        let numerator = integer
            .checked_mul(denominator)
            .and_then(|value| value.checked_add(fraction))
            .ok_or(ConstantNumericFailure::Invalid)?;
        Ok(Rational {
            numerator,
            denominator,
        })
    }

    fn whitespace(&mut self) {
        while self
            .input
            .get(self.offset)
            .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
        {
            self.offset += 1;
        }
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.input.get(self.offset) == Some(&expected) {
            self.offset += 1;
            true
        } else {
            false
        }
    }

    fn consume_keyword(&mut self, expected: &[u8]) -> bool {
        if self.input[self.offset..].starts_with(expected) {
            self.offset += expected.len();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::{
        ConstantNumericFailure, IntegralFunction, compare, evaluate_integral_lexical,
        evaluate_number_lexical, fold_exact_integral_arithmetic, fold_integral_equality,
        fold_integral_function, fold_number_conversion, integral_function_call,
        number_function_call,
    };

    #[test]
    fn compares_checked_exact_rational_constants() {
        assert_eq!(
            compare("(((((2*10)-4)+9) div 5) div 2)", "2"),
            Ok(Ordering::Greater)
        );
        assert_eq!(compare("9 mod 3", "0"), Ok(Ordering::Equal));
    }

    #[test]
    fn rounds_admitted_nonnegative_exact_decimals() {
        assert_eq!(compare("round(3.7)", "4"), Ok(Ordering::Equal));
        assert_eq!(compare("round(3.5)", "4"), Ok(Ordering::Equal));
        assert_eq!(compare("round(3.4)", "3"), Ok(Ordering::Equal));
    }

    #[test]
    fn folds_exact_integral_functions_with_xpath_rounding_direction() {
        assert_eq!(fold_integral_function("floor(3.7)"), Some("3".to_owned()));
        assert_eq!(fold_integral_function("floor(-1.5)"), Some("-2".to_owned()));
        assert_eq!(fold_integral_function("ceiling(3.1)"), Some("4".to_owned()));
        assert_eq!(
            fold_integral_function("ceiling(-1.5)"),
            Some("-1".to_owned())
        );
        assert_eq!(fold_integral_function("round(2.5)"), Some("3".to_owned()));
        assert_eq!(fold_integral_function("round(-2.5)"), Some("-2".to_owned()));
        assert_eq!(fold_integral_function("floor(source)"), None);
        assert_eq!(fold_integral_function("not-floor(1.5)"), None);
        assert_eq!(fold_integral_equality("floor(1.9) = 1"), Some(true));
        assert_eq!(fold_integral_equality("round(-1.5) = -1"), Some(true));
        assert_eq!(fold_integral_equality("ceiling(1.1) = 1"), Some(false));
        assert_eq!(fold_integral_equality("floor(1.9) != 1"), None);
        assert_eq!(
            integral_function_call("floor(source)"),
            Some((IntegralFunction::Floor, "source"))
        );
        assert_eq!(
            evaluate_integral_lexical(IntegralFunction::Round, " -2.5 "),
            Ok("-2".to_owned())
        );
        assert_eq!(
            evaluate_integral_lexical(IntegralFunction::Floor, "1+2"),
            Err(ConstantNumericFailure::Invalid)
        );
        assert_eq!(
            evaluate_integral_lexical(IntegralFunction::Floor, "1e2"),
            Err(ConstantNumericFailure::Unsupported)
        );
    }

    #[test]
    fn compares_integral_functions_and_rejects_invalid_division() {
        assert_eq!(compare("ceiling(3.7)", "3"), Ok(Ordering::Greater));
        assert_eq!(compare("floor(-1.5)", "-2"), Ok(Ordering::Equal));
        assert_eq!(compare("round(-1.5)", "-1"), Ok(Ordering::Equal));
        assert_eq!(
            compare("1 div 0", "0"),
            Err(ConstantNumericFailure::Invalid)
        );
    }

    #[test]
    fn folds_only_exact_integral_binary_arithmetic() {
        assert_eq!(fold_exact_integral_arithmetic("2*3"), Some("6".to_owned()));
        assert_eq!(
            fold_exact_integral_arithmetic("3 + 6"),
            Some("9".to_owned())
        );
        assert_eq!(
            fold_exact_integral_arithmetic("6 div 2"),
            Some("3".to_owned())
        );
        assert_eq!(
            fold_exact_integral_arithmetic("5 mod 2"),
            Some("1".to_owned())
        );
        assert_eq!(fold_exact_integral_arithmetic("1 div 2"), None);
        assert_eq!(fold_exact_integral_arithmetic("7"), None);
        assert_eq!(fold_exact_integral_arithmetic("1 div 0"), None);
    }

    #[test]
    fn folds_admitted_number_conversions_and_nan_results() {
        assert_eq!(fold_number_conversion("number(2)"), Some("2".to_owned()));
        assert_eq!(
            fold_number_conversion("number( '003.500' )"),
            Some("3.5".to_owned())
        );
        assert_eq!(fold_number_conversion("number(-0)"), Some("0".to_owned()));
        assert_eq!(
            fold_number_conversion("number('abc')"),
            Some("NaN".to_owned())
        );
        assert_eq!(fold_number_conversion("number('NaN')"), None);
        assert_eq!(fold_number_conversion("number(source)"), None);
        assert_eq!(fold_number_conversion("number(1 div 2)"), None);
        assert_eq!(fold_number_conversion("not-number(2)"), None);
        assert_eq!(number_function_call("number(source)"), Some("source"));
        assert_eq!(number_function_call("number()"), Some(""));
        assert_eq!(evaluate_number_lexical(" 001.2500 "), Ok("1.25".to_owned()));
        assert_eq!(evaluate_number_lexical("abc"), Ok("NaN".to_owned()));
    }
}
