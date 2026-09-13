//! Bounded sequential focus predicates for XSLT 1.0 match patterns.

use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::{MatchNumericRelation, MatchSequencePredicate};

use super::is_ascii_ncname;

pub(super) fn parse(pattern: &str) -> Option<(&str, Vec<MatchSequencePredicate>)> {
    let (element, predicates) = pattern.split_once('[')?;
    if !is_ascii_ncname(element) {
        return None;
    }
    let predicates = predicates
        .strip_suffix(']')?
        .split("][")
        .map(parse_predicate)
        .collect::<Option<Vec<_>>>()?;
    (predicates.len() >= 2).then_some((element, predicates))
}

fn parse_predicate(lexical: &str) -> Option<MatchSequencePredicate> {
    let lexical = lexical.trim();
    if lexical == "last()" {
        return Some(MatchSequencePredicate::Last);
    }
    if let Ok(value) = lexical.parse::<usize>() {
        return Some(MatchSequencePredicate::Position {
            relation: MatchNumericRelation::Equal,
            value,
        });
    }
    if let Some(remainder) = lexical.strip_prefix("position()") {
        let (relation, value) = parse_relation(remainder)?;
        return Some(MatchSequencePredicate::Position {
            relation,
            value: value.try_into().ok()?,
        });
    }
    if let Some(expression) = lexical.strip_prefix('(').and_then(|v| v.rsplit_once(')')) {
        let (operand, remainder) = expression;
        let (relation, value) = parse_relation(remainder)?;
        if let Some(divisor) = operand.strip_prefix("position() mod ") {
            return Some(MatchSequencePredicate::PositionModulo {
                divisor: divisor.trim().parse().ok()?,
                relation,
                value: value.try_into().ok()?,
            });
        }
        if let Some(operand) = operand.strip_prefix('@') {
            let (attribute, divisor) = operand.split_once(" mod ")?;
            if !is_ascii_ncname(attribute) {
                return None;
            }
            return Some(MatchSequencePredicate::AttributeModulo {
                attribute: unqualified(attribute),
                divisor: divisor.trim().parse().ok()?,
                relation,
                value,
            });
        }
    }
    if let Some(operand) = lexical.strip_prefix('@') {
        let boundary = operand.find(['=', '>', '<'])?;
        let (attribute, remainder) = operand.split_at(boundary);
        let attribute = attribute.trim();
        if !is_ascii_ncname(attribute) {
            return None;
        }
        let (relation, value) = parse_relation(remainder)?;
        return Some(MatchSequencePredicate::AttributeNumber {
            attribute: unqualified(attribute),
            relation,
            value,
        });
    }
    None
}

fn parse_relation(lexical: &str) -> Option<(MatchNumericRelation, i32)> {
    let lexical = lexical.trim();
    let (relation, value) = if let Some(value) = lexical.strip_prefix('=') {
        (MatchNumericRelation::Equal, value)
    } else if let Some(value) = lexical.strip_prefix('>') {
        (MatchNumericRelation::Greater, value)
    } else if let Some(value) = lexical.strip_prefix('<') {
        (MatchNumericRelation::Less, value)
    } else {
        return None;
    };
    Some((relation, value.trim().parse().ok()?))
}

fn unqualified(local: &str) -> ExpandedName {
    ExpandedName {
        namespace: None,
        local: local.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_bounded_xslt10_sequential_predicate_family() {
        for pattern in [
            "x[(position() mod 2)=1][position() > 3]",
            "x[(position() mod 2) > 0][position() > 3][2]",
            "x[(position() mod 2)=1][position() > 3][last()]",
            "x[(position() mod 2)=1][@num > 5][last()]",
            "x[(@num mod 3)=2][position() > 2][last()]",
            "x[(position() mod 2)=1][2][@num < 10]",
        ] {
            assert!(parse(pattern).is_some(), "must parse {pattern}");
        }
    }

    #[test]
    fn rejects_single_and_general_expression_predicates() {
        assert!(parse("x[2]").is_none());
        assert!(parse("x[position() + 1][2]").is_none());
        assert!(parse("x[(position() mod $n)=1][2]").is_none());
    }
}
