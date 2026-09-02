//! Private `fn:empty` seam for bounded, source-free path evidence.

use crate::execution_control_experiment::{ControlFailure, InvocationControl};
use crate::xdm::owned_tree_experiment::{Document, SourceLocation};

use super::path_experiment::{PathFailure, evaluate_location_path_controlled, parse_location_path};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EmptyFailure {
    Path(PathFailure),
    Control(ControlFailure),
    Unsupported,
}

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
    use super::evaluate;
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
}
