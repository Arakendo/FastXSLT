//! Private boolean composition around the admitted `deep-equal` function.

use crate::execution_control_experiment::InvocationControl;
use crate::xdm::owned_tree_experiment::{Document, SourceLocation};

use super::deep_equal_experiment::{
    DeepEqualEvaluationFailure, DeepEqualExpression, DeepEqualFailure,
    evaluate as evaluate_deep_equal, parse as parse_deep_equal,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeepEqualBooleanExpression {
    inner: DeepEqualExpression,
    projection: BooleanProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BooleanProjection {
    Identity,
    Not,
    Equals(bool),
}

pub(crate) fn recognizes(expression: &str) -> bool {
    let expression = expression.trim();
    expression.starts_with("deep-equal(")
        || expression.starts_with("fn:deep-equal(")
        || expression.starts_with("not(deep-equal(")
        || expression.starts_with("not(fn:deep-equal(")
}

pub(crate) fn parse(
    expression: &str,
    location: &SourceLocation,
) -> Result<DeepEqualBooleanExpression, DeepEqualFailure> {
    let expression = expression.trim();
    let (inner, projection) = if let Some(inner) = expression
        .strip_prefix("not(")
        .and_then(|inner| inner.strip_suffix(')'))
    {
        (inner, BooleanProjection::Not)
    } else if let Some(inner) = expression.strip_suffix(" eq true()") {
        (inner, BooleanProjection::Equals(true))
    } else if let Some(inner) = expression.strip_suffix(" eq false()") {
        (inner, BooleanProjection::Equals(false))
    } else {
        (expression, BooleanProjection::Identity)
    };
    parse_deep_equal(inner, location).map(|inner| DeepEqualBooleanExpression { inner, projection })
}

pub(crate) fn evaluate(
    expression: &DeepEqualBooleanExpression,
    document: Option<&Document>,
    control: &mut InvocationControl,
) -> Result<bool, DeepEqualEvaluationFailure> {
    let value = evaluate_deep_equal(&expression.inner, document, control)?;
    Ok(match expression.projection {
        BooleanProjection::Identity => value,
        BooleanProjection::Not => !value,
        BooleanProjection::Equals(expected) => value == expected,
    })
}

#[cfg(test)]
mod tests {
    use super::{evaluate, parse, recognizes};
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};
    use crate::xdm::owned_tree_experiment::SourceLocation;

    fn location() -> SourceLocation {
        SourceLocation {
            resource: "urn:fastxslt:deep-equal:boolean".to_owned(),
            span: 0..1,
        }
    }

    #[test]
    fn composes_boolean_results_without_changing_inner_work_charges() {
        for (expression, expected) in [
            ("deep-equal((1, 2), (1, 2))", true),
            ("not(deep-equal((1, 2), (2, 1)))", true),
            ("deep-equal((), ()) eq true()", true),
            ("deep-equal((1), (2)) eq false()", true),
        ] {
            assert!(recognizes(expression));
            let parsed = parse(expression, &location()).expect("parse boolean composition");
            let mut control = InvocationControl::unbounded();
            assert_eq!(
                evaluate(&parsed, None, &mut control).expect("evaluate boolean composition"),
                expected
            );
            assert!(control.consumed(WorkDomain::XPathOperation) > 0);
        }
    }
}
