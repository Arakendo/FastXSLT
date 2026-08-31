//! Narrow ordered `for` expression used by the native XSLT30 `for-001` slice.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};

use super::path_experiment::{
    LocationPath, PathFailure, evaluate_location_path_controlled, parse_location_path,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ForDistinctValuesExpression {
    variable: String,
    binding_path: LocationPath,
    first_equal_path: LocationPath,
    related_parent_path: LocationPath,
    related_test_child: String,
    related_result_child: String,
    location: SourceLocation,
}

#[cfg(feature = "workbench")]
impl ForDistinctValuesExpression {
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        self.variable.capacity()
            + self.binding_path.known_owned_capacity_bytes()
            + self.first_equal_path.known_owned_capacity_bytes()
            + self.related_parent_path.known_owned_capacity_bytes()
            + self.related_test_child.capacity()
            + self.related_result_child.capacity()
            + self.location.resource.capacity()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ForExpressionFailure {
    Invalid {
        detail: String,
        location: SourceLocation,
    },
    Unsupported {
        detail: String,
        location: SourceLocation,
    },
}

pub(crate) fn parse(
    expression: &str,
    location: SourceLocation,
) -> Result<ForDistinctValuesExpression, ForExpressionFailure> {
    let normalized = expression.split_whitespace().collect::<Vec<_>>().join(" ");
    let after_for = normalized
        .strip_prefix("for $")
        .ok_or_else(|| unsupported(&normalized, &location))?;
    let (variable, after_variable) = after_for
        .split_once(" in distinct-values(")
        .ok_or_else(|| unsupported(&normalized, &location))?;
    if !is_ascii_ncname(variable) {
        return Err(ForExpressionFailure::Invalid {
            detail: format!("invalid for variable name: ${variable}"),
            location,
        });
    }
    let (binding, return_expression) = after_variable
        .split_once(") return ")
        .ok_or_else(|| unsupported(&normalized, &location))?;
    let returned = return_expression
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| unsupported(&normalized, &location))?;
    let (first, related) =
        split_top_level_comma(returned).ok_or_else(|| unsupported(&normalized, &location))?;

    let first = first
        .trim()
        .strip_suffix("[1]")
        .and_then(|value| value.strip_prefix('('))
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| unsupported(&normalized, &location))?;
    let equality = format!("[. = ${variable}]");
    let first_path = first
        .strip_suffix(&equality)
        .ok_or_else(|| unsupported(&normalized, &location))?;

    let related = related.trim();
    let predicate_start = related
        .find('[')
        .ok_or_else(|| unsupported(&normalized, &location))?;
    let related_parent = &related[..predicate_start];
    let after_parent = &related[predicate_start + 1..];
    let (predicate, related_result) = after_parent
        .split_once("]/")
        .ok_or_else(|| unsupported(&normalized, &location))?;
    let expected_variable = format!("${variable}");
    let (related_test_child, predicate_variable) = predicate
        .split_once(" = ")
        .ok_or_else(|| unsupported(&normalized, &location))?;
    if predicate_variable != expected_variable
        || !is_ascii_ncname(related_test_child)
        || !is_ascii_ncname(related_result)
    {
        return Err(unsupported(&normalized, &location));
    }

    Ok(ForDistinctValuesExpression {
        variable: variable.to_owned(),
        binding_path: parse_absolute_path(binding, location.clone())?,
        first_equal_path: parse_absolute_path(first_path, location.clone())?,
        related_parent_path: parse_absolute_path(related_parent, location.clone())?,
        related_test_child: related_test_child.to_owned(),
        related_result_child: related_result.to_owned(),
        location,
    })
}

pub(crate) fn evaluate(
    expression: &ForDistinctValuesExpression,
    document: &Document,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    let binding_nodes = evaluate_location_path_controlled(
        document,
        document.document_node(),
        &expression.binding_path,
        control,
    )?;
    let mut distinct_values = Vec::new();
    for node in binding_nodes {
        let value = document.string_value_controlled(node, control)?;
        if !distinct_values.contains(&value) {
            distinct_values.push(value);
        }
    }

    let mut result = Vec::new();
    for value in distinct_values {
        let first_candidates = evaluate_location_path_controlled(
            document,
            document.document_node(),
            &expression.first_equal_path,
            control,
        )?;
        for candidate in first_candidates {
            if document.string_value_controlled(candidate, control)? == value {
                result.push(candidate);
                break;
            }
        }

        let parents = evaluate_location_path_controlled(
            document,
            document.document_node(),
            &expression.related_parent_path,
            control,
        )?;
        for parent in parents {
            let mut matches_value = false;
            for child in document.children(parent).iter().copied() {
                control.charge(WorkDomain::XPathNodeVisit, 1)?;
                if is_unnamespaced_element(document, child, &expression.related_test_child)
                    && document.string_value_controlled(child, control)? == value
                {
                    matches_value = true;
                    break;
                }
            }
            if !matches_value {
                continue;
            }
            for child in document.children(parent).iter().copied() {
                control.charge(WorkDomain::XPathNodeVisit, 1)?;
                if is_unnamespaced_element(document, child, &expression.related_result_child) {
                    result.push(child);
                }
            }
        }
    }
    Ok(result)
}

fn parse_absolute_path(
    expression: &str,
    location: SourceLocation,
) -> Result<LocationPath, ForExpressionFailure> {
    let relative = expression
        .strip_prefix('/')
        .ok_or_else(|| unsupported(expression, &location))?;
    parse_location_path(relative, location).map_err(|failure| match failure {
        PathFailure::Invalid { detail, location } => {
            ForExpressionFailure::Invalid { detail, location }
        }
        PathFailure::Unsupported { detail, location } => {
            ForExpressionFailure::Unsupported { detail, location }
        }
    })
}

fn split_top_level_comma(expression: &str) -> Option<(&str, &str)> {
    let mut parenthesis_depth = 0_usize;
    let mut bracket_depth = 0_usize;
    for (offset, character) in expression.char_indices() {
        match character {
            '(' => parenthesis_depth += 1,
            ')' => parenthesis_depth = parenthesis_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            ',' if parenthesis_depth == 0 && bracket_depth == 0 => {
                return Some((&expression[..offset], &expression[offset + 1..]));
            }
            _ => {}
        }
    }
    None
}

fn is_unnamespaced_element(document: &Document, node: NodeId, local: &str) -> bool {
    document.kind(node) == NodeKind::Element
        && document
            .name(node)
            .is_some_and(|name| name.namespace.is_none() && name.local == local)
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

fn unsupported(expression: &str, location: &SourceLocation) -> ForExpressionFailure {
    ForExpressionFailure::Unsupported {
        detail: format!(
            "the private slice supports one ordered for/distinct-values expression shape: {expression}"
        ),
        location: location.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::{evaluate, parse};
    use crate::execution_control_experiment::{InvocationControl, WorkDomain};
    use crate::xdm::owned_tree_experiment::{Document, SourceLocation};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    #[test]
    fn preserves_distinct_value_and_return_sequence_order_without_fixed_names() {
        let expression = parse(
            "for $creator in distinct-values(/library/item/creator) return ((/library/item/creator[. = $creator])[1], /library/item[creator = $creator]/title)",
            SourceLocation {
                resource: "memory:expression".to_owned(),
                span: 0..155,
            },
        )
        .expect("admitted for-expression shape should parse");
        let parsed = parse_document(
            "memory:source",
            b"<library><item><title>A</title><creator>X</creator></item><item><title>B</title><creator>X</creator><creator>Y</creator></item></library>",
            ParseLimits {
                max_events: 32,
                max_depth: 8,
            },
        )
        .expect("source should parse");
        let document = Document::from_parsed(parsed).expect("source XDM should build");

        let mut control = InvocationControl::unbounded();
        let selected =
            evaluate(&expression, &document, &mut control).expect("for expression should evaluate");
        let projection: Vec<_> = selected
            .iter()
            .map(|node| {
                (
                    document
                        .name(*node)
                        .expect("selected nodes are elements")
                        .local
                        .clone(),
                    document.string_value(*node),
                )
            })
            .collect();

        assert_eq!(
            projection,
            [
                ("creator".to_owned(), "X".to_owned()),
                ("title".to_owned(), "A".to_owned()),
                ("title".to_owned(), "B".to_owned()),
                ("creator".to_owned(), "Y".to_owned()),
                ("title".to_owned(), "B".to_owned()),
            ]
        );
        assert!(control.consumed(WorkDomain::XPathNodeVisit) > 0);
        assert!(control.consumed(WorkDomain::XdmStringValueNode) > 0);
    }
}
