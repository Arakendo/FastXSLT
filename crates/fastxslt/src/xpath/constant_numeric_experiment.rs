//! Checked exact-rational constants for the admitted conditional-expression slice.

use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConstantNumericFailure {
    Invalid,
    Unsupported,
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
        if self.numerator < 0 || self.denominator <= 0 {
            return Err(ConstantNumericFailure::Unsupported);
        }
        let doubled = self
            .numerator
            .checked_mul(2)
            .and_then(|value| value.checked_add(self.denominator))
            .ok_or(ConstantNumericFailure::Invalid)?;
        let divisor = self
            .denominator
            .checked_mul(2)
            .ok_or(ConstantNumericFailure::Invalid)?;
        Ok(Self::integer(doubled / divisor))
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
            .is_some_and(u8::is_ascii_whitespace)
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

    use super::{ConstantNumericFailure, compare};

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
    fn rejects_unadmitted_numeric_functions_and_invalid_division() {
        assert_eq!(
            compare("ceiling(3.7)", "3"),
            Err(ConstantNumericFailure::Unsupported)
        );
        assert_eq!(
            compare("1 div 0", "0"),
            Err(ConstantNumericFailure::Invalid)
        );
    }
}
