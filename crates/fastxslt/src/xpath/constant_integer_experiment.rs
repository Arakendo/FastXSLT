//! Checked constant-integer expression parsing for the private `XPath` slice.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConstantIntegerFailure {
    Invalid,
    Unsupported,
}

pub(crate) fn evaluate(expression: &str) -> Result<i64, ConstantIntegerFailure> {
    let mut parser = Parser {
        input: expression.as_bytes(),
        offset: 0,
    };
    let value = parser.additive()?;
    parser.whitespace();
    if parser.offset == parser.input.len() {
        Ok(value)
    } else {
        Err(ConstantIntegerFailure::Unsupported)
    }
}

struct Parser<'a> {
    input: &'a [u8],
    offset: usize,
}

impl Parser<'_> {
    fn additive(&mut self) -> Result<i64, ConstantIntegerFailure> {
        let mut value = self.multiplicative()?;
        loop {
            self.whitespace();
            if self.consume(b'+') {
                value = value
                    .checked_add(self.multiplicative()?)
                    .ok_or(ConstantIntegerFailure::Invalid)?;
            } else if self.consume(b'-') {
                value = value
                    .checked_sub(self.multiplicative()?)
                    .ok_or(ConstantIntegerFailure::Invalid)?;
            } else {
                return Ok(value);
            }
        }
    }

    fn multiplicative(&mut self) -> Result<i64, ConstantIntegerFailure> {
        let mut value = self.primary()?;
        loop {
            self.whitespace();
            if self.consume(b'*') {
                value = value
                    .checked_mul(self.primary()?)
                    .ok_or(ConstantIntegerFailure::Invalid)?;
            } else if self.consume_keyword(b"div") {
                let divisor = self.primary()?;
                if divisor == 0 {
                    return Err(ConstantIntegerFailure::Invalid);
                }
                if value % divisor != 0 {
                    return Err(ConstantIntegerFailure::Unsupported);
                }
                value = value
                    .checked_div(divisor)
                    .ok_or(ConstantIntegerFailure::Invalid)?;
            } else if self.consume_keyword(b"mod") {
                let divisor = self.primary()?;
                if value < 0 || divisor <= 0 {
                    return Err(ConstantIntegerFailure::Unsupported);
                }
                value %= divisor;
            } else {
                return Ok(value);
            }
        }
    }

    fn primary(&mut self) -> Result<i64, ConstantIntegerFailure> {
        self.whitespace();
        if self.consume(b'(') {
            let value = self.additive()?;
            self.whitespace();
            if !self.consume(b')') {
                return Err(ConstantIntegerFailure::Invalid);
            }
            return Ok(value);
        }
        let start = self.offset;
        while self.input.get(self.offset).is_some_and(u8::is_ascii_digit) {
            self.offset += 1;
        }
        if self.offset == start {
            return Err(ConstantIntegerFailure::Unsupported);
        }
        std::str::from_utf8(&self.input[start..self.offset])
            .expect("ASCII digits are valid UTF-8")
            .parse()
            .map_err(|_| ConstantIntegerFailure::Invalid)
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
    use super::{ConstantIntegerFailure, evaluate};

    #[test]
    fn applies_xpath_operator_precedence_and_parentheses_for_path_007() {
        assert_eq!(evaluate("(((((2*10)-4)+9) div 5) mod 3 )"), Ok(2));
        assert_eq!(evaluate("2 + 3 * 4"), Ok(14));
        assert_eq!(evaluate("(2 + 3) * 4"), Ok(20));
    }

    #[test]
    fn rejects_unadmitted_numeric_semantics_instead_of_approximating_them() {
        assert_eq!(
            evaluate("1 div 2"),
            Err(ConstantIntegerFailure::Unsupported)
        );
        assert_eq!(evaluate("1 div 0"), Err(ConstantIntegerFailure::Invalid));
        assert_eq!(
            evaluate("floor(2)"),
            Err(ConstantIntegerFailure::Unsupported)
        );
        assert_eq!(evaluate("(2 + 3"), Err(ConstantIntegerFailure::Invalid));
    }
}
