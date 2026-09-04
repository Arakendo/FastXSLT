//! Private `fn:empty` seam for bounded, source-free path evidence.

use crate::execution_control_experiment::{ControlFailure, InvocationControl};
#[cfg(test)]
use crate::xdm::owned_tree_experiment::{Document, SourceLocation};

use super::deep_equal_atomic::{parse_sequence, split_top_level_once};
#[cfg(test)]
use super::path_experiment::{PathFailure, evaluate_location_path_controlled, parse_location_path};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SequenceCardinalityExpression {
    source: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SequenceCardinalityParseFailure {
    InvalidArity,
    Unsupported,
}

impl SequenceCardinalityExpression {
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        self.source.capacity()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EmptyFailure {
    #[cfg(test)]
    Path(PathFailure),
    Control(ControlFailure),
    InvalidArity,
    Unsupported,
}

pub(crate) fn recognizes_source_free(expression: &str) -> bool {
    expression.contains("empty(")
        || expression.contains("fn:empty(")
        || expression.contains("exists(")
        || expression.contains("fn:exists(")
}

pub(crate) fn parse_source_free(
    expression: &str,
) -> Result<SequenceCardinalityExpression, SequenceCardinalityParseFailure> {
    if !recognizes_source_free(expression) {
        return Err(SequenceCardinalityParseFailure::Unsupported);
    }
    evaluate_source_free(expression, &mut InvocationControl::unbounded()).map_err(|failure| {
        match failure {
            EmptyFailure::InvalidArity => SequenceCardinalityParseFailure::InvalidArity,
            #[cfg(test)]
            EmptyFailure::Path(_) => SequenceCardinalityParseFailure::Unsupported,
            EmptyFailure::Control(_) | EmptyFailure::Unsupported => {
                SequenceCardinalityParseFailure::Unsupported
            }
        }
    })?;
    Ok(SequenceCardinalityExpression {
        source: expression.to_owned(),
    })
}

pub(crate) fn evaluate_compiled_source_free(
    expression: &SequenceCardinalityExpression,
    control: &mut InvocationControl,
) -> Result<bool, EmptyFailure> {
    evaluate_source_free(&expression.source, control)
}

pub(crate) fn evaluate_source_free(
    expression: &str,
    control: &mut InvocationControl,
) -> Result<bool, EmptyFailure> {
    let expression = expression.trim();
    if let Some(inner) = function_argument(expression, &["not", "fn:not"]) {
        control
            .charge(
                crate::execution_control_experiment::WorkDomain::XPathOperation,
                1,
            )
            .map_err(EmptyFailure::Control)?;
        return evaluate_source_free(inner, control).map(|value| !value);
    }
    let (inner, invert_cardinality) =
        if let Some(inner) = function_argument(expression, &["empty", "fn:empty"]) {
            (inner, false)
        } else if let Some(inner) = function_argument(expression, &["exists", "fn:exists"]) {
            (inner, true)
        } else {
            return Err(EmptyFailure::Unsupported);
        };
    if inner.trim().is_empty() || split_top_level_once(inner).is_some() {
        return Err(EmptyFailure::InvalidArity);
    }
    let sequence = parse_sequence(inner.trim()).ok_or(EmptyFailure::Unsupported)?;
    control
        .charge(
            crate::execution_control_experiment::WorkDomain::XPathOperation,
            1,
        )
        .map_err(EmptyFailure::Control)?;
    Ok(sequence.is_empty() ^ invert_cardinality)
}

fn function_argument<'a>(expression: &'a str, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| {
        expression
            .strip_prefix(name)
            .and_then(|remainder| remainder.strip_prefix('('))
            .and_then(|remainder| remainder.strip_suffix(')'))
    })
}

#[cfg(test)]
pub(crate) fn evaluate(
    expression: &str,
    document: &Document,
    location: SourceLocation,
    control: &mut InvocationControl,
) -> Result<bool, EmptyFailure> {
    let inner = expression
        .strip_prefix("empty(")
        .and_then(|expression| expression.strip_suffix(')'))
        .ok_or(EmptyFailure::Unsupported)?;
    let path = parse_location_path(inner.trim(), location).map_err(EmptyFailure::Path)?;
    evaluate_location_path_controlled(document, document.document_node(), &path, control)
        .map(|selected| selected.is_empty())
        .map_err(EmptyFailure::Control)
}

#[cfg(test)]
mod tests {
    use super::{EmptyFailure, evaluate, evaluate_source_free};
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};
    use crate::xdm::owned_tree_experiment::{Document, SourceLocation};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    #[test]
    fn an_empty_sequence_path_does_not_visit_the_supplied_document() {
        let parsed = parse_document(
            "memory:unused.xml",
            b"<unused attribute='value'><child/></unused>",
            ParseLimits {
                max_events: 16,
                max_depth: 8,
            },
        )
        .expect("unused document should parse");
        let document = Document::from_parsed(parsed).expect("unused XDM should build");
        let mut control = InvocationControl::unbounded();

        for expression in ["empty(()/@attribute)", "empty(()/child)"] {
            assert!(
                evaluate(
                    expression,
                    &document,
                    SourceLocation {
                        resource: "memory:expression".to_owned(),
                        span: 0..expression.len(),
                    },
                    &mut control,
                )
                .expect("bounded empty-sequence path should execute")
            );
        }
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 0);
    }

    #[test]
    fn source_free_empty_distinguishes_sequences_from_function_arity() {
        let mut control = InvocationControl::unbounded();
        assert!(evaluate_source_free("empty(())", &mut control).unwrap());
        assert!(evaluate_source_free("not(empty((1, (), \"string\")))", &mut control).unwrap());
        assert!(evaluate_source_free("exists(reverse((1, 2, 3)))", &mut control).unwrap());
        assert_eq!(
            evaluate_source_free("empty()", &mut control),
            Err(EmptyFailure::InvalidArity)
        );
        assert_eq!(
            evaluate_source_free("empty(1, 2)", &mut control),
            Err(EmptyFailure::InvalidArity)
        );
        assert_eq!(control.consumed(WorkDomain::XPathOperation), 4);
    }
}
