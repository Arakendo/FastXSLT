//! Private document-aware effective-boolean-value seam for executable QT3 evidence.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, SourceLocation};

use super::deep_equal_atomic::{
    EffectiveBooleanValueFailure, parse_effective_boolean_value, split_top_level_once,
};
use super::path_experiment::{PathFailure, evaluate_location_path_controlled, parse_location_path};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum EffectiveBooleanFailure {
    Path(PathFailure),
    Control(ControlFailure),
    InvalidTypeOrCardinality,
    Unsupported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EffectiveItem {
    Node,
    Atomic(bool),
    InvalidAtomic,
}

pub(crate) fn evaluate(
    expression: &str,
    document: &Document,
    location: &SourceLocation,
    control: &mut InvocationControl,
) -> Result<bool, EffectiveBooleanFailure> {
    let expression = expression.trim();
    let (argument, negate) =
        if let Some(argument) = function_argument(expression, &["not", "fn:not"]) {
            (argument, true)
        } else if let Some(argument) = function_argument(expression, &["boolean", "fn:boolean"]) {
            (argument, false)
        } else {
            return Err(EffectiveBooleanFailure::Unsupported);
        };

    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(EffectiveBooleanFailure::Control)?;
    let mut items = Vec::new();
    evaluate_sequence(
        strip_outer_parentheses(argument.trim()),
        document,
        location,
        control,
        &mut items,
    )?;
    let value = match items.as_slice() {
        [] => false,
        [EffectiveItem::Atomic(value)] => *value,
        [EffectiveItem::Node, ..] => true,
        [EffectiveItem::InvalidAtomic | EffectiveItem::Atomic(_), ..] => {
            return Err(EffectiveBooleanFailure::InvalidTypeOrCardinality);
        }
    };
    Ok(value ^ negate)
}

fn evaluate_sequence(
    expression: &str,
    document: &Document,
    location: &SourceLocation,
    control: &mut InvocationControl,
    items: &mut Vec<EffectiveItem>,
) -> Result<(), EffectiveBooleanFailure> {
    if expression == "()" {
        return Ok(());
    }
    if let Some((left, right)) = split_top_level_once(expression) {
        evaluate_sequence(left.trim(), document, location, control, items)?;
        return evaluate_sequence(right.trim(), document, location, control, items);
    }
    if let Some(value) = parse_effective_boolean_value(expression) {
        items.push(match value {
            Ok(value) => EffectiveItem::Atomic(value),
            Err(EffectiveBooleanValueFailure::InvalidTypeOrCardinality) => {
                EffectiveItem::InvalidAtomic
            }
        });
        return Ok(());
    }
    let path =
        parse_location_path(expression, location.clone()).map_err(EffectiveBooleanFailure::Path)?;
    let nodes =
        evaluate_location_path_controlled(document, document.document_node(), &path, control)
            .map_err(EffectiveBooleanFailure::Control)?;
    items.extend(nodes.into_iter().map(|_| EffectiveItem::Node));
    Ok(())
}

fn function_argument<'a>(expression: &'a str, names: &[&str]) -> Option<&'a str> {
    names.iter().find_map(|name| {
        expression
            .strip_prefix(name)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
    })
}

fn strip_outer_parentheses(expression: &str) -> &str {
    expression
        .strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
        .unwrap_or(expression)
}

#[cfg(test)]
mod tests {
    use super::{EffectiveBooleanFailure, evaluate};
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};
    use crate::xdm::owned_tree_experiment::{Document, SourceLocation};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    #[test]
    fn evaluates_node_and_mixed_sequences_in_item_order() {
        let parsed = parse_document(
            "memory:source.xml",
            b"<root><present/></root>",
            ParseLimits {
                max_events: 16,
                max_depth: 8,
            },
        )
        .expect("parse source");
        let document = Document::from_parsed(parsed).expect("build XDM");

        for (source, expected) in [
            ("not(//missing)", true),
            ("not(//present)", false),
            ("boolean((/, 93.7))", true),
            ("boolean((true(), //missing))", true),
        ] {
            let mut control = InvocationControl::unbounded();
            let actual = evaluate(
                source,
                &document,
                &SourceLocation {
                    resource: "memory:expression".to_owned(),
                    span: 0..source.len(),
                },
                &mut control,
            )
            .expect("evaluate admitted EBV expression");
            assert_eq!(actual, expected, "{source}");
            assert!(control.consumed(WorkDomain::XPathOperation) > 0);
        }

        let source = "boolean((93.7, /))";
        assert_eq!(
            evaluate(
                source,
                &document,
                &SourceLocation {
                    resource: "memory:expression".to_owned(),
                    span: 0..source.len(),
                },
                &mut InvocationControl::unbounded(),
            ),
            Err(EffectiveBooleanFailure::InvalidTypeOrCardinality)
        );
    }
}
