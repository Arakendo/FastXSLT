//! Bounded static recognition of path expressions whose context is provably atomic.

use crate::xdm::owned_tree_experiment::SourceLocation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PathOperandTypeFailure {
    pub(crate) standard_code: &'static str,
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
}

pub(crate) fn classify(
    expression: &str,
    location: SourceLocation,
) -> Option<PathOperandTypeFailure> {
    let expression = expression.trim();
    if let Some(context) = expression.strip_suffix("[..]")
        && is_integer_literal(context.trim())
    {
        return Some(non_node_context("parent axis", location));
    }
    if let Some(context) = expression.strip_suffix("[element()]")
        && is_integer_literal(context.trim())
    {
        return Some(non_node_context("child axis", location));
    }

    let slash = top_level_slash(expression)?;
    let left = expression[..slash].trim();
    if !is_statically_atomic_sequence(left) {
        return None;
    }
    Some(PathOperandTypeFailure {
        standard_code: "XPTY0019",
        detail:
            "the left operand of a path expression is statically known to contain an atomic item"
                .to_owned(),
        location,
    })
}

fn non_node_context(axis: &str, location: SourceLocation) -> PathOperandTypeFailure {
    PathOperandTypeFailure {
        standard_code: "XPTY0020",
        detail: format!("the context item for the {axis} is statically known to be atomic"),
        location,
    }
}

fn top_level_slash(expression: &str) -> Option<usize> {
    let mut parentheses = 0_u32;
    let mut brackets = 0_u32;
    let mut quote = None;
    for (offset, character) in expression.char_indices() {
        match character {
            '\'' | '"' if quote.is_none() => quote = Some(character),
            character if quote == Some(character) => quote = None,
            '(' if quote.is_none() => parentheses += 1,
            ')' if quote.is_none() => parentheses = parentheses.checked_sub(1)?,
            '[' if quote.is_none() => brackets += 1,
            ']' if quote.is_none() => brackets = brackets.checked_sub(1)?,
            '/' if quote.is_none() && parentheses == 0 && brackets == 0 => return Some(offset),
            _ => {}
        }
    }
    None
}

fn is_statically_atomic_sequence(expression: &str) -> bool {
    if is_integer_literal(expression) {
        return true;
    }
    if let Some(inner) = expression
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        return is_integer_sequence(inner);
    }
    let Some((sequence, predicate)) = expression.rsplit_once('[') else {
        return false;
    };
    let Some(predicate) = predicate.strip_suffix(']') else {
        return false;
    };
    matches!(predicate.trim(), "1" | "last()")
        && sequence
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
            .is_some_and(is_integer_sequence)
}

fn is_integer_sequence(expression: &str) -> bool {
    let mut found = false;
    for member in expression.split(',') {
        if !is_integer_literal(member.trim()) {
            return false;
        }
        found = true;
    }
    found
}

fn is_integer_literal(expression: &str) -> bool {
    let unsigned = expression.strip_prefix(['+', '-']).unwrap_or(expression);
    !unsigned.is_empty() && unsigned.bytes().all(|byte| byte.is_ascii_digit())
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
    fn distinguishes_atomic_path_operands_from_atomic_axis_contexts() {
        for expression in ["1/3", "(10)/child::*", "(1, 2, 3)[1]/child"] {
            let failure = classify(expression, location(expression)).expect("known atomic path");
            assert_eq!(failure.standard_code, "XPTY0019");
        }
        for expression in ["123[..]", "1[element()]"] {
            let failure = classify(expression, location(expression)).expect("known atomic context");
            assert_eq!(failure.standard_code, "XPTY0020");
        }
        assert!(classify("source/child", location("source/child")).is_none());
    }
}
