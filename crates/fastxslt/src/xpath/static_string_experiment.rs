//! Bounded constant folding for static string concatenation.

pub(crate) fn fold(expression: &str) -> Option<String> {
    let mut value = String::new();
    for term in expression.split("||") {
        let term = term.trim();
        if let Some(literal) = single_quoted_literal(term) {
            value.push_str(literal);
        } else {
            value.push(parse_single_codepoint(term)?);
        }
    }
    Some(value)
}

fn single_quoted_literal(term: &str) -> Option<&str> {
    let literal = term.strip_prefix('\'')?.strip_suffix('\'')?;
    (!literal.contains('\'')).then_some(literal)
}

fn parse_single_codepoint(term: &str) -> Option<char> {
    let lexical = term
        .strip_prefix("codepoints-to-string(")?
        .strip_suffix(')')?
        .trim();
    let character = char::from_u32(lexical.parse().ok()?)?;
    is_xml_10_character(character).then_some(character)
}

fn is_xml_10_character(character: char) -> bool {
    matches!(character, '\u{9}' | '\u{a}' | '\u{d}')
        || ('\u{20}'..='\u{d7ff}').contains(&character)
        || ('\u{e000}'..='\u{fffd}').contains(&character)
        || ('\u{10000}'..='\u{10ffff}').contains(&character)
}

#[cfg(test)]
mod tests {
    use super::fold;

    #[test]
    fn folds_literal_concatenation_with_one_xml_codepoint() {
        assert_eq!(
            fold("'[' || codepoints-to-string(13) || ']'").as_deref(),
            Some("[\r]")
        );
        assert_eq!(fold("codepoints-to-string(0)"), None);
        assert_eq!(fold("$dynamic"), None);
    }
}
