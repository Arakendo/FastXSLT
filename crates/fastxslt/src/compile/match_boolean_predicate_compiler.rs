//! Bounded boolean wildcard predicates for XSLT 1.0 match patterns.

use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::{MatchBooleanPredicate, MatchNumericRelation};

use super::is_ascii_ncname;

pub(super) fn parse(pattern: &str) -> Option<MatchBooleanPredicate> {
    let predicate = pattern.strip_prefix("*[")?.strip_suffix(']')?;
    parse_expression(predicate)
}

fn parse_expression(lexical: &str) -> Option<MatchBooleanPredicate> {
    let lexical = strip_outer_parentheses(lexical.trim());
    if let Some((left, right)) = split_top_level(lexical, "or") {
        return Some(MatchBooleanPredicate::Or(
            Box::new(parse_expression(left)?),
            Box::new(parse_expression(right)?),
        ));
    }
    if let Some((left, right)) = split_top_level(lexical, "and") {
        return Some(MatchBooleanPredicate::And(
            Box::new(parse_expression(left)?),
            Box::new(parse_expression(right)?),
        ));
    }
    if let Some(inner) = lexical
        .strip_prefix("not(")
        .and_then(|value| value.strip_suffix(')'))
    {
        return Some(MatchBooleanPredicate::Not(Box::new(parse_expression(
            inner,
        )?)));
    }
    parse_context_number(lexical)
        .or_else(|| parse_position(lexical))
        .or_else(|| parse_attribute_equals(lexical))
}

fn parse_context_number(lexical: &str) -> Option<MatchBooleanPredicate> {
    let (left, right) = split_comparison(lexical, '=')?;
    let value = if left.trim() == "." {
        right.trim().parse().ok()?
    } else if right.trim() == "." {
        left.trim().parse().ok()?
    } else {
        return None;
    };
    Some(MatchBooleanPredicate::ContextNumberEquals(value))
}

fn parse_position(lexical: &str) -> Option<MatchBooleanPredicate> {
    let (left, relation, right) = split_numeric_relation(lexical)?;
    let (relation, value) = if left.trim() == "position()" {
        (relation, right.trim().parse().ok()?)
    } else if right.trim() == "position()" {
        (reverse(relation), left.trim().parse().ok()?)
    } else {
        return None;
    };
    Some(MatchBooleanPredicate::Position { relation, value })
}

fn parse_attribute_equals(lexical: &str) -> Option<MatchBooleanPredicate> {
    let (left, right) = split_comparison(lexical, '=')?;
    let (attribute, value) = if let Some(attribute) = left.trim().strip_prefix('@') {
        (attribute, quoted(right.trim())?)
    } else if let Some(attribute) = right.trim().strip_prefix('@') {
        (attribute, quoted(left.trim())?)
    } else {
        return None;
    };
    if !is_ascii_ncname(attribute) {
        return None;
    }
    Some(MatchBooleanPredicate::AttributeEquals {
        attribute: ExpandedName {
            namespace: None,
            local: attribute.to_owned(),
        },
        value: value.to_owned(),
    })
}

fn split_top_level<'a>(lexical: &'a str, operator: &str) -> Option<(&'a str, &'a str)> {
    let bytes = lexical.as_bytes();
    let operator = operator.as_bytes();
    let mut depth = 0usize;
    let mut quote = None;
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            current if quote == Some(current) => quote = None,
            b'\'' | b'"' if quote.is_none() => quote = Some(bytes[index]),
            b'(' if quote.is_none() => depth += 1,
            b')' if quote.is_none() => depth = depth.checked_sub(1)?,
            _ if quote.is_none()
                && depth == 0
                && bytes[index..].starts_with(operator)
                && token_boundary_before(bytes, index)
                && token_boundary_after(bytes, index + operator.len()) =>
            {
                return Some((&lexical[..index], &lexical[index + operator.len()..]));
            }
            _ => {}
        }
        index += 1;
    }
    (depth == 0 && quote.is_none()).then_some(())?;
    None
}

fn token_boundary_before(bytes: &[u8], index: usize) -> bool {
    index == 0 || bytes[index - 1].is_ascii_whitespace() || b"()".contains(&bytes[index - 1])
}

fn token_boundary_after(bytes: &[u8], index: usize) -> bool {
    index == bytes.len() || bytes[index].is_ascii_whitespace() || b"()".contains(&bytes[index])
}

fn strip_outer_parentheses(mut lexical: &str) -> &str {
    loop {
        let Some(inner) = lexical
            .strip_prefix('(')
            .and_then(|value| value.strip_suffix(')'))
        else {
            return lexical;
        };
        if encloses_entire_expression(lexical) {
            lexical = inner.trim();
        } else {
            return lexical;
        }
    }
}

fn encloses_entire_expression(lexical: &str) -> bool {
    let mut depth = 0usize;
    let mut quote = None;
    for (index, byte) in lexical.bytes().enumerate() {
        match byte {
            current if quote == Some(current) => quote = None,
            b'\'' | b'"' if quote.is_none() => quote = Some(byte),
            b'(' if quote.is_none() => depth += 1,
            b')' if quote.is_none() => {
                let Some(next) = depth.checked_sub(1) else {
                    return false;
                };
                depth = next;
                if depth == 0 && index + 1 != lexical.len() {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0 && quote.is_none()
}

fn split_comparison(lexical: &str, operator: char) -> Option<(&str, &str)> {
    let index = lexical.find(operator)?;
    (!lexical[index + operator.len_utf8()..].contains(operator))
        .then_some((&lexical[..index], &lexical[index + operator.len_utf8()..]))
}

fn split_numeric_relation(lexical: &str) -> Option<(&str, MatchNumericRelation, &str)> {
    for (operator, relation) in [
        ('=', MatchNumericRelation::Equal),
        ('>', MatchNumericRelation::Greater),
        ('<', MatchNumericRelation::Less),
    ] {
        if let Some((left, right)) = split_comparison(lexical, operator) {
            return Some((left, relation, right));
        }
    }
    None
}

fn reverse(relation: MatchNumericRelation) -> MatchNumericRelation {
    match relation {
        MatchNumericRelation::Equal => MatchNumericRelation::Equal,
        MatchNumericRelation::Greater => MatchNumericRelation::Less,
        MatchNumericRelation::Less => MatchNumericRelation::Greater,
    }
}

fn quoted(lexical: &str) -> Option<&str> {
    lexical
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .or_else(|| {
            lexical
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_boolean_wildcard_predicate() {
        for expression in [
            ".=117",
            "not(.=117)",
            "position() > 225",
            "(position() > 225) and (position() < 375)",
            "@century='yes'",
            "(@century='yes') or (@foo='nope')",
        ] {
            assert!(
                parse_expression(expression).is_some(),
                "must parse {expression}"
            );
        }
        let pattern = "*[(not(.=117) and ((position() > 225) and (position() < 375))) and ((@century='yes') or (@foo='nope'))]";
        assert!(parse(pattern).is_some());
    }

    #[test]
    fn rejects_unbounded_boolean_shapes() {
        assert!(parse("item[@x='y']").is_none());
        assert!(parse("*[contains(., 'x') and @x='y']").is_none());
        assert!(parse("*[@p:x='y' or @x='y']").is_none());
    }
}
