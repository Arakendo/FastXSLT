//! Shared scalar semantics for sequential match-pattern focus predicates.

use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::{MatchNumericRelation, MatchSequencePredicate};

pub(super) fn required_attribute(predicate: &MatchSequencePredicate) -> Option<&ExpandedName> {
    match predicate {
        MatchSequencePredicate::AttributeModulo { attribute, .. }
        | MatchSequencePredicate::AttributeNumber { attribute, .. } => Some(attribute),
        _ => None,
    }
}

pub(super) fn evaluate(
    predicate: &MatchSequencePredicate,
    position: usize,
    size: usize,
    attribute_value: Option<&str>,
) -> bool {
    match predicate {
        MatchSequencePredicate::PositionModulo {
            divisor,
            relation,
            value,
        } => *divisor != 0 && compare_usize(position % divisor, *value, *relation),
        MatchSequencePredicate::Position { relation, value } => {
            compare_usize(position, *value, *relation)
        }
        MatchSequencePredicate::AttributeModulo {
            divisor,
            relation,
            value,
            ..
        } => {
            *divisor != 0
                && attribute_value
                    .and_then(crate::xpath::constant_boolean_experiment::parse_xpath_number_literal)
                    .is_some_and(|number| {
                        compare(number % f64::from(*divisor), f64::from(*value), *relation)
                    })
        }
        MatchSequencePredicate::AttributeNumber {
            relation, value, ..
        } => attribute_value
            .and_then(crate::xpath::constant_boolean_experiment::parse_xpath_number_literal)
            .is_some_and(|number| compare(number, f64::from(*value), *relation)),
        MatchSequencePredicate::Last => position == size,
    }
}

pub(super) fn context_number_greater_than_variable(
    context_value: &str,
    variable: &AtomicValue,
) -> bool {
    let parse = crate::xpath::constant_boolean_experiment::parse_xpath_number_literal;
    parse(context_value)
        .zip(parse(variable.lexical()))
        .is_some_and(|(left, right)| left > right)
}

fn compare_usize(left: usize, right: usize, relation: MatchNumericRelation) -> bool {
    match relation {
        MatchNumericRelation::Equal => left == right,
        MatchNumericRelation::Greater => left > right,
        MatchNumericRelation::Less => left < right,
    }
}

#[allow(
    clippy::float_cmp,
    reason = "XPath numeric equality is exact IEEE comparison, including NaN behavior"
)]
fn compare(left: f64, right: f64, relation: MatchNumericRelation) -> bool {
    match relation {
        MatchNumericRelation::Equal => left == right,
        MatchNumericRelation::Greater => left > right,
        MatchNumericRelation::Less => left < right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequential_predicates_use_the_current_filtered_focus() {
        let odd = MatchSequencePredicate::PositionModulo {
            divisor: 2,
            relation: MatchNumericRelation::Equal,
            value: 1,
        };
        let after_three = MatchSequencePredicate::Position {
            relation: MatchNumericRelation::Greater,
            value: 3,
        };
        assert!(evaluate(&odd, 5, 12, None));
        assert!(evaluate(&after_three, 4, 6, None));
        assert!(evaluate(&MatchSequencePredicate::Last, 2, 2, None));
        assert!(!evaluate(&MatchSequencePredicate::Last, 1, 2, None));
    }
}
