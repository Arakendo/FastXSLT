//! Private `fn:count` expression seam for executable `QT3` evidence.

use crate::execution_control_experiment::{ControlFailure, InvocationControl};
use crate::xdm::owned_tree_experiment::{Document, SourceLocation};

use super::path_experiment::{PathFailure, evaluate_location_path_controlled, parse_location_path};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CountFailure {
    Path(PathFailure),
    Control(ControlFailure),
    Unsupported,
}

pub(crate) fn evaluate(
    expression: &str,
    document: &Document,
    location: SourceLocation,
    control: &mut InvocationControl,
) -> Result<usize, CountFailure> {
    let inner = expression
        .strip_prefix("fn:count(")
        .and_then(|expression| expression.strip_suffix(')'))
        .ok_or(CountFailure::Unsupported)?;
    let path = parse_location_path(inner, location).map_err(CountFailure::Path)?;
    evaluate_location_path_controlled(document, document.document_node(), &path, control)
        .map(|selected| selected.len())
        .map_err(CountFailure::Control)
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::{CountFailure, evaluate};
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};
    use crate::xdm::owned_tree_experiment::{Document, SourceLocation};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    #[test]
    fn counts_a_descendant_then_explicit_named_child_axis() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<root><center><south-east/><other/></center><center><south-east/></center></root>",
            ParseLimits {
                max_events: 24,
                max_depth: 8,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");
        let mut control = InvocationControl::unbounded();

        let count = evaluate(
            "fn:count(//center/child::south-east)",
            &document,
            SourceLocation {
                resource: "memory:expression".to_owned(),
                span: Range { start: 0, end: 42 },
            },
            &mut control,
        )
        .expect("count expression should execute");

        assert_eq!(count, 2);
        assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 9);
        assert!(matches!(
            evaluate(
                "count(//center)",
                &document,
                SourceLocation {
                    resource: "memory:expression".to_owned(),
                    span: 0..15,
                },
                &mut InvocationControl::unbounded(),
            ),
            Err(CountFailure::Unsupported)
        ));
    }
}
