//! Bounded constant folding for static string concatenation.

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StaticStringFunctionValue {
    String(String),
    Boolean(bool),
}

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

pub(crate) fn fold_concat_literals(expression: &str) -> Option<String> {
    const MAX_ARGUMENTS: usize = 4_096;
    let expression = expression.trim();
    let arguments = ["concat", "fn:concat"].iter().find_map(|name| {
        expression
            .strip_prefix(name)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
    })?;
    let arguments = split_arguments(arguments, MAX_ARGUMENTS)?;
    if arguments.len() < 2 {
        return None;
    }
    let mut result = String::new();
    for argument in arguments {
        result.push_str(&static_atom_string(argument)?);
    }
    Some(result)
}

pub(crate) fn fold_binary_literal_function(expression: &str) -> Option<StaticStringFunctionValue> {
    let expression = expression.trim();
    let (function, arguments) = [
        "substring-before",
        "fn:substring-before",
        "substring-after",
        "fn:substring-after",
        "starts-with",
        "fn:starts-with",
        "contains",
        "fn:contains",
    ]
    .iter()
    .find_map(|name| {
        expression
            .strip_prefix(name)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
            .map(|arguments| (*name, arguments))
    })?;
    let arguments = split_arguments(arguments, 2)?;
    let [value, search] = arguments.as_slice() else {
        return None;
    };
    let value = quoted_literal(value)?;
    let search = quoted_literal(search)?;
    match function.strip_prefix("fn:").unwrap_or(function) {
        "contains" => Some(StaticStringFunctionValue::Boolean(value.contains(search))),
        "starts-with" => Some(StaticStringFunctionValue::Boolean(
            value.starts_with(search),
        )),
        "substring-before" => Some(StaticStringFunctionValue::String(
            value
                .find(search)
                .map_or_else(String::new, |index| value[..index].to_owned()),
        )),
        "substring-after" => Some(StaticStringFunctionValue::String(
            value.find(search).map_or_else(String::new, |index| {
                value[index + search.len()..].to_owned()
            }),
        )),
        _ => unreachable!("function list and folding dispatch must agree"),
    }
}

fn split_arguments(source: &str, max_arguments: usize) -> Option<Vec<&str>> {
    let mut arguments = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut quote = None;
    for (index, character) in source.char_indices() {
        if let Some(active) = quote {
            if character == active {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => {
                arguments.push(source[start..index].trim());
                if arguments.len() >= max_arguments {
                    return None;
                }
                start = index + character.len_utf8();
            }
            _ => {}
        }
    }
    if quote.is_some() || depth != 0 {
        return None;
    }
    arguments.push(source[start..].trim());
    (arguments.len() <= max_arguments && arguments.iter().all(|value| !value.is_empty()))
        .then_some(arguments)
}

fn static_atom_string(source: &str) -> Option<String> {
    let source = source.trim();
    if let Some(value) = quoted_literal(source) {
        return Some(value.to_owned());
    }
    match source {
        "true()" | "fn:true()" => return Some("true".to_owned()),
        "false()" | "fn:false()" => return Some("false".to_owned()),
        _ => {}
    }
    if let Some(argument) = ["string", "fn:string"].iter().find_map(|name| {
        source
            .strip_prefix(name)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
    }) {
        return static_atom_string(argument);
    }
    parse_integer_literal(source).map(str::to_owned)
}

fn quoted_literal(source: &str) -> Option<&str> {
    for quote in ['\'', '"'] {
        if let Some(value) = source
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
            .filter(|value| !value.contains(quote))
        {
            return Some(value);
        }
    }
    None
}

fn parse_integer_literal(source: &str) -> Option<&str> {
    let unsigned = source.strip_prefix('-').unwrap_or(source);
    (!unsigned.is_empty() && unsigned.chars().all(|character| character.is_ascii_digit()))
        .then_some(source)
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
    use super::{
        StaticStringFunctionValue, fold, fold_binary_literal_function, fold_concat_literals,
    };

    #[test]
    fn folds_literal_concatenation_with_one_xml_codepoint() {
        assert_eq!(
            fold("'[' || codepoints-to-string(13) || ']'").as_deref(),
            Some("[\r]")
        );
        assert_eq!(fold("codepoints-to-string(0)"), None);
        assert_eq!(fold("$dynamic"), None);
    }

    #[test]
    fn folds_bounded_xpath_concat_over_static_atomic_arguments() {
        assert_eq!(
            fold_concat_literals("concat('a,b', false(), string(34), \"c\")").as_deref(),
            Some("a,bfalse34c")
        );
        assert_eq!(fold_concat_literals("concat('one')"), None);
        assert_eq!(fold_concat_literals("concat('a', $dynamic)"), None);
        assert_eq!(fold_concat_literals("concat('a', path)"), None);
        let excessive = format!("concat({})", vec!["'x'"; 4_097].join(","));
        assert_eq!(fold_concat_literals(&excessive), None);
    }

    #[test]
    fn folds_binary_literal_string_functions_with_empty_string_rules() {
        for (source, expected) in [
            (
                "contains('ENCYCLOPEDIA', 'CYCL')",
                StaticStringFunctionValue::Boolean(true),
            ),
            (
                "starts-with('abc', '')",
                StaticStringFunctionValue::Boolean(true),
            ),
            (
                "substring-before('1999/04/01', '/')",
                StaticStringFunctionValue::String("1999".to_owned()),
            ),
            (
                "substring-after('1999/04/01', '/')",
                StaticStringFunctionValue::String("04/01".to_owned()),
            ),
            (
                "substring-after('abc', 'z')",
                StaticStringFunctionValue::String(String::new()),
            ),
            (
                "substring-after('é😀', 'é')",
                StaticStringFunctionValue::String("😀".to_owned()),
            ),
        ] {
            assert_eq!(
                fold_binary_literal_function(source),
                Some(expected),
                "{source}"
            );
        }
        assert_eq!(fold_binary_literal_function("contains(path, 'x')"), None);
        assert_eq!(fold_binary_literal_function("contains('x')"), None);
    }
}
