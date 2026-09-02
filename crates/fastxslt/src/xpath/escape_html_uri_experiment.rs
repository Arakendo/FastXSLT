//! Bounded constant folding for the `XPath` `escape-html-uri` function.

pub(crate) fn fold_literal(expression: &str) -> Option<String> {
    let argument = expression
        .trim()
        .strip_prefix("escape-html-uri(")?
        .strip_suffix(')')?
        .trim();
    let literal = argument.strip_prefix('\'')?.strip_suffix('\'')?;
    if literal.contains('\'') {
        return None;
    }
    let mut escaped = String::with_capacity(literal.len());
    for character in literal.chars() {
        if ('\u{20}'..='\u{7e}').contains(&character) {
            escaped.push(character);
        } else {
            let mut encoded = [0_u8; 4];
            for byte in character.encode_utf8(&mut encoded).as_bytes() {
                escaped.push('%');
                escaped.push(hex_digit(byte >> 4));
                escaped.push(hex_digit(byte & 0x0f));
            }
        }
    }
    Some(escaped)
}

fn hex_digit(value: u8) -> char {
    char::from(if value < 10 {
        b'0' + value
    } else {
        b'A' + value - 10
    })
}

#[cfg(test)]
mod tests {
    use super::fold_literal;

    #[test]
    fn escapes_non_ascii_utf8_without_unicode_normalization() {
        assert_eq!(
            fold_literal("escape-html-uri('http://example/\u{fb4f}/\u{e5}/a\u{30a}')").as_deref(),
            Some("http://example/%EF%AD%8F/%C3%A5/a%CC%8A")
        );
        assert_eq!(fold_literal("escape-html-uri($uri)"), None);
    }
}
