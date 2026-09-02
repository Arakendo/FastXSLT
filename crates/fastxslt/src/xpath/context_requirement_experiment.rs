//! Bounded recognition of source-free expressions that require a dynamic context item.

use crate::xdm::owned_tree_experiment::SourceLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MissingContextFailure {
    pub(crate) standard_code: &'static str,
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
}

pub(crate) fn classify(
    expression: &str,
    location: SourceLocation,
) -> Option<MissingContextFailure> {
    requires_context(expression.trim()).then(|| MissingContextFailure {
        standard_code: "XPDY0002",
        detail: "the expression requires a context item, but none was supplied".to_owned(),
        location,
    })
}

fn requires_context(expression: &str) -> bool {
    let expression = strip_positional_filter(expression);
    let expression = strip_enclosing_parentheses(expression).unwrap_or(expression);
    let Some(members) = split_top_level_members(expression) else {
        return false;
    };
    members.into_iter().any(member_requires_context)
}

fn member_requires_context(member: &str) -> bool {
    let member = member.trim();
    member == "/"
        || member.strip_suffix("* /").is_some_and(|left| {
            left.trim()
                .strip_prefix(['+', '-'])
                .unwrap_or(left.trim())
                .bytes()
                .all(|byte| byte.is_ascii_digit())
        })
        || is_ascii_ncname(member)
}

fn strip_positional_filter(expression: &str) -> &str {
    let Some((base, predicate)) = expression.rsplit_once('[') else {
        return expression;
    };
    let Some(predicate) = predicate.strip_suffix(']') else {
        return expression;
    };
    let predicate = predicate.trim();
    if predicate == "last()"
        || (!predicate.is_empty() && predicate.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return base.trim();
    }
    expression
}

fn strip_enclosing_parentheses(expression: &str) -> Option<&str> {
    let inner = expression.strip_prefix('(')?.strip_suffix(')')?;
    let mut depth = 0_u32;
    let mut quote = None;
    for character in inner.chars() {
        match character {
            '\'' | '"' if quote.is_none() => quote = Some(character),
            character if quote == Some(character) => quote = None,
            '(' if quote.is_none() => depth += 1,
            ')' if quote.is_none() => depth = depth.checked_sub(1)?,
            _ => {}
        }
    }
    (depth == 0 && quote.is_none()).then_some(inner)
}

fn split_top_level_members(expression: &str) -> Option<Vec<&str>> {
    let mut members = Vec::new();
    let mut depth = 0_u32;
    let mut quote = None;
    let mut start = 0;
    for (offset, character) in expression.char_indices() {
        match character {
            '\'' | '"' if quote.is_none() => quote = Some(character),
            character if quote == Some(character) => quote = None,
            '(' if quote.is_none() => depth += 1,
            ')' if quote.is_none() => depth = depth.checked_sub(1)?,
            ',' if quote.is_none() && depth == 0 => {
                members.push(&expression[start..offset]);
                start = offset + 1;
            }
            _ => {}
        }
    }
    if depth != 0 || quote.is_some() {
        return None;
    }
    members.push(&expression[start..]);
    Some(members)
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}

#[cfg(test)]
mod tests {
    use super::classify;
    use crate::xdm::owned_tree_experiment::SourceLocation;

    fn location(expression: &str) -> SourceLocation {
        SourceLocation {
            resource: "memory:expression".to_owned(),
            span: 0..expression.len(),
        }
    }

    #[test]
    fn recognizes_only_the_bounded_context_dependent_grammar() {
        for expression in [
            "declare",
            "xquery, version, encoding",
            "(1, /)[1]",
            "(/, 1)[2]",
            "(1, 5 * /)[1]",
        ] {
            let failure = classify(expression, location(expression)).expect("context required");
            assert_eq!(failure.standard_code, "XPDY0002");
        }
        for expression in ["1", "(1, 2)[1]", "'/'", "foo + bar"] {
            assert!(classify(expression, location(expression)).is_none());
        }
    }
}
