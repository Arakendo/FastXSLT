//! Focus-preserving `sum(for ...)` seam used by native XSLT30 `for-003`.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};

use super::path_experiment::{
    LocationPath, PathFailure, evaluate_location_path_controlled, parse_location_path,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FocusSumForExpression {
    binding_path: LocationPath,
    left_attribute: String,
    right_attribute: String,
}

#[cfg(feature = "workbench")]
impl FocusSumForExpression {
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        self.binding_path.known_owned_capacity_bytes()
            + self.left_attribute.capacity()
            + self.right_attribute.capacity()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FocusSumForFailure {
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FocusSumEvaluationFailure {
    Control(ControlFailure),
    Unsupported,
}

pub(crate) fn parse(
    expression: &str,
    location: &SourceLocation,
) -> Result<FocusSumForExpression, FocusSumForFailure> {
    let normalized = expression.split_whitespace().collect::<Vec<_>>().join(" ");
    let inner = normalized
        .strip_prefix("sum(for $")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| unsupported(&normalized, location))?;
    let (variable, after_variable) = inner
        .split_once(" in ")
        .ok_or_else(|| unsupported(&normalized, location))?;
    if !is_ascii_ncname(variable) {
        return Err(unsupported(&normalized, location));
    }
    let (binding_path, returned) = after_variable
        .split_once(" return ")
        .ok_or_else(|| unsupported(&normalized, location))?;
    let (left, right) = returned
        .split_once(" * ")
        .ok_or_else(|| unsupported(&normalized, location))?;
    let left_attribute = left
        .strip_prefix('@')
        .filter(|name| is_ascii_ncname(name))
        .ok_or_else(|| unsupported(&normalized, location))?;
    let right_attribute = right
        .strip_prefix('@')
        .filter(|name| is_ascii_ncname(name))
        .ok_or_else(|| unsupported(&normalized, location))?;
    let binding_path = parse_location_path(binding_path, location.clone()).map_err(|failure| {
        let detail = match failure {
            PathFailure::Invalid { detail, .. } | PathFailure::Unsupported { detail, .. } => detail,
        };
        FocusSumForFailure {
            detail,
            location: location.clone(),
        }
    })?;

    Ok(FocusSumForExpression {
        binding_path,
        left_attribute: left_attribute.to_owned(),
        right_attribute: right_attribute.to_owned(),
    })
}

pub(crate) fn evaluate(
    expression: &FocusSumForExpression,
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<i64, FocusSumEvaluationFailure> {
    let tuples =
        evaluate_location_path_controlled(document, context, &expression.binding_path, control)
            .map_err(FocusSumEvaluationFailure::Control)?;
    let mut produced_value = false;
    for _bound_item in tuples {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(FocusSumEvaluationFailure::Control)?;
        let left = find_attribute(document, context, &expression.left_attribute, control)?;
        let right = find_attribute(document, context, &expression.right_attribute, control)?;
        if left.is_some() && right.is_some() {
            produced_value = true;
        }
    }
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(FocusSumEvaluationFailure::Control)?;
    if produced_value {
        return Err(FocusSumEvaluationFailure::Unsupported);
    }
    Ok(0)
}

fn find_attribute(
    document: &Document,
    context: NodeId,
    local: &str,
    control: &mut InvocationControl,
) -> Result<Option<NodeId>, FocusSumEvaluationFailure> {
    for attribute in document.attributes(context).iter().copied() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(FocusSumEvaluationFailure::Control)?;
        if document
            .name(attribute)
            .is_some_and(|name| name.namespace.is_none() && name.local == local)
        {
            return Ok(Some(attribute));
        }
    }
    Ok(None)
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    characters.next().is_some_and(|first| {
        (first.is_ascii_alphabetic() || first == '_')
            && characters.all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
            })
    })
}

fn unsupported(expression: &str, location: &SourceLocation) -> FocusSumForFailure {
    FocusSumForFailure {
        detail: format!(
            "the private slice supports one focus-preserving sum(for ...) expression shape: {expression}"
        ),
        location: location.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{FocusSumEvaluationFailure, evaluate, parse};
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};
    use crate::xdm::owned_tree_experiment::{Document, SourceLocation};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn expression() -> super::FocusSumForExpression {
        parse(
            "sum(for $item in order-item return @price * @qty)",
            &SourceLocation {
                resource: "memory:focus-sum".to_owned(),
                span: 0..52,
            },
        )
        .expect("admitted focus-preserving expression should parse")
    }

    #[test]
    fn binding_does_not_replace_the_outer_focus() {
        let parsed = parse_document(
            "memory:order",
            b"<order><order-item price='4' qty='2'/><order-item price='3' qty='1'/></order>",
            ParseLimits {
                max_events: 16,
                max_depth: 4,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let order = document.children(document.document_node())[0];
        let mut control = InvocationControl::unbounded();

        let sum = evaluate(&expression(), &document, order, &mut control)
            .expect("empty multiplication sequence should sum to zero");

        assert_eq!(sum, 0);
        assert_eq!(control.consumed(WorkDomain::XPathOperation), 3);
    }

    #[test]
    fn non_empty_numeric_multiplication_is_not_approximated() {
        let parsed = parse_document(
            "memory:order",
            b"<order price='4' qty='2'><order-item/></order>",
            ParseLimits {
                max_events: 12,
                max_depth: 4,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let order = document.children(document.document_node())[0];

        assert_eq!(
            evaluate(
                &expression(),
                &document,
                order,
                &mut InvocationControl::unbounded()
            ),
            Err(FocusSumEvaluationFailure::Unsupported)
        );
    }
}
