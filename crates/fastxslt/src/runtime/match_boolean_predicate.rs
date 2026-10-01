//! Shared evaluation shape for bounded boolean match predicates.

use crate::xslt::golden_semantics_experiment::{MatchBooleanPredicate, MatchNumericRelation};

pub(super) trait Context {
    type Error;

    fn charge_operation(&mut self) -> Result<(), Self::Error>;
    fn context_number_equals(&mut self, value: i32) -> Result<bool, Self::Error>;
    fn position(&self) -> usize;
    fn attribute_equals(
        &mut self,
        name: &crate::xml::quick_xml_experiment::ExpandedName,
        value: &str,
    ) -> Result<bool, Self::Error>;
    fn attribute_number(
        &mut self,
        name: &crate::xml::quick_xml_experiment::ExpandedName,
        relation: MatchNumericRelation,
        value: i32,
    ) -> Result<bool, Self::Error>;
}

pub(super) fn evaluate<C: Context>(
    predicate: &MatchBooleanPredicate,
    context: &mut C,
) -> Result<bool, C::Error> {
    context.charge_operation()?;
    match predicate {
        MatchBooleanPredicate::ContextNumberEquals(value) => context.context_number_equals(*value),
        MatchBooleanPredicate::Position { relation, value } => {
            Ok(compare_usize(context.position(), *value, *relation))
        }
        MatchBooleanPredicate::AttributeEquals { attribute, value } => {
            context.attribute_equals(attribute, value)
        }
        MatchBooleanPredicate::AttributeNumber {
            attribute,
            relation,
            value,
        } => context.attribute_number(attribute, *relation, *value),
        MatchBooleanPredicate::And(left, right) => {
            if !evaluate(left, context)? {
                return Ok(false);
            }
            evaluate(right, context)
        }
        MatchBooleanPredicate::Or(left, right) => {
            if evaluate(left, context)? {
                return Ok(true);
            }
            evaluate(right, context)
        }
        MatchBooleanPredicate::Not(inner) => Ok(!evaluate(inner, context)?),
    }
}

pub(super) fn compare_number(left: f64, right: i32, relation: MatchNumericRelation) -> bool {
    match relation {
        MatchNumericRelation::Equal => left
            .partial_cmp(&f64::from(right))
            .is_some_and(std::cmp::Ordering::is_eq),
        MatchNumericRelation::Greater => left > f64::from(right),
        MatchNumericRelation::Less => left < f64::from(right),
    }
}

fn compare_usize(left: usize, right: usize, relation: MatchNumericRelation) -> bool {
    match relation {
        MatchNumericRelation::Equal => left == right,
        MatchNumericRelation::Greater => left > right,
        MatchNumericRelation::Less => left < right,
    }
}
