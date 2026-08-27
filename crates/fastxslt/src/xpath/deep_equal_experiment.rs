use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};

use super::deep_equal_atomic::{
    AtomicSequence, ExactDecimal, parse_decimal, parse_integer, parse_sequence,
    split_top_level_once,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeepEqualExpression {
    operands: DeepEqualOperands,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeepEqualOperands {
    Nodes {
        left: NodeSelection,
        right: NodeSelection,
    },
    Integers {
        left: i128,
        right: i128,
    },
    Decimals {
        left: ExactDecimal,
        right: ExactDecimal,
    },
    AtomicSequences {
        left: AtomicSequence,
        right: AtomicSequence,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NodeSelection {
    DescendantAttribute {
        element: String,
        position: usize,
        attribute: String,
    },
    DescendantComment {
        position: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DeepEqualFailure {
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
    pub(crate) kind: DeepEqualFailureKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeepEqualFailureKind {
    InvalidArity { standard_code: &'static str },
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeepEqualEvaluationFailure {
    Control(ControlFailure),
    MissingNodeContext,
}

pub(crate) fn parse(
    expression: &str,
    location: &SourceLocation,
) -> Result<DeepEqualExpression, DeepEqualFailure> {
    let body = expression
        .strip_prefix("deep-equal(")
        .or_else(|| expression.strip_prefix("fn:deep-equal("))
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| unsupported(expression, location))?;
    let arguments =
        split_top_level_once(body).ok_or_else(|| invalid_arity(expression, location))?;
    if let Some((_, remaining)) = split_top_level_once(arguments.1) {
        if split_top_level_once(remaining).is_some() {
            return Err(invalid_arity(expression, location));
        }
        return Err(unsupported(expression, location));
    }
    let left = arguments.0.trim();
    let right = arguments.1.trim();
    let operands = if let (Some(left), Some(right)) = (parse_integer(left), parse_integer(right)) {
        DeepEqualOperands::Integers { left, right }
    } else if let (Some(left), Some(right)) = (parse_decimal(left), parse_decimal(right)) {
        DeepEqualOperands::Decimals { left, right }
    } else if let (Some(left), Some(right)) = (parse_sequence(left), parse_sequence(right)) {
        DeepEqualOperands::AtomicSequences { left, right }
    } else {
        DeepEqualOperands::Nodes {
            left: parse_selection(left, location)?,
            right: parse_selection(right, location)?,
        }
    };
    Ok(DeepEqualExpression {
        operands,
        location: location.clone(),
    })
}

fn parse_selection(
    expression: &str,
    location: &SourceLocation,
) -> Result<NodeSelection, DeepEqualFailure> {
    if let Some(position) = expression
        .strip_prefix("//comment()[")
        .and_then(|value| value.strip_suffix(']'))
        .and_then(parse_position)
    {
        return Ok(NodeSelection::DescendantComment { position });
    }
    let (element, attribute) = expression
        .strip_prefix("//")
        .and_then(|value| value.split_once("/@"))
        .ok_or_else(|| unsupported(expression, location))?;
    let (element, position) = parse_positioned_name(element)
        .filter(|(name, _)| is_ascii_ncname(name))
        .ok_or_else(|| unsupported(expression, location))?;
    if !is_ascii_ncname(attribute) {
        return Err(unsupported(expression, location));
    }
    Ok(NodeSelection::DescendantAttribute {
        element: element.to_owned(),
        position,
        attribute: attribute.to_owned(),
    })
}

fn parse_positioned_name(value: &str) -> Option<(&str, usize)> {
    let (name, position) = value.split_once('[')?;
    let position = position.strip_suffix(']').and_then(parse_position)?;
    Some((name, position))
}

fn parse_position(value: &str) -> Option<usize> {
    value.parse::<usize>().ok().filter(|position| *position > 0)
}

fn unsupported(expression: &str, location: &SourceLocation) -> DeepEqualFailure {
    DeepEqualFailure {
        detail: format!("the private deep-equal slice does not support expression: {expression}"),
        location: location.clone(),
        kind: DeepEqualFailureKind::Unsupported,
    }
}

fn invalid_arity(expression: &str, location: &SourceLocation) -> DeepEqualFailure {
    DeepEqualFailure {
        detail: format!("deep-equal requires two or three arguments: {expression}"),
        location: location.clone(),
        kind: DeepEqualFailureKind::InvalidArity {
            standard_code: "XPST0017",
        },
    }
}

pub(crate) fn evaluate(
    expression: &DeepEqualExpression,
    document: Option<&Document>,
    control: &mut InvocationControl,
) -> Result<bool, DeepEqualEvaluationFailure> {
    match &expression.operands {
        DeepEqualOperands::Integers { left, right } => {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(DeepEqualEvaluationFailure::Control)?;
            Ok(left == right)
        }
        DeepEqualOperands::Decimals { left, right } => {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(DeepEqualEvaluationFailure::Control)?;
            Ok(left == right)
        }
        DeepEqualOperands::AtomicSequences { left, right } => {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(DeepEqualEvaluationFailure::Control)?;
            if left.len() != right.len() {
                return Ok(false);
            }
            for index in 0..left.len() {
                control
                    .charge(WorkDomain::XPathOperation, 1)
                    .map_err(DeepEqualEvaluationFailure::Control)?;
                if !left.item_equals(right, index) {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        DeepEqualOperands::Nodes { left, right } => {
            let document = document.ok_or(DeepEqualEvaluationFailure::MissingNodeContext)?;
            evaluate_nodes(left, right, document, control)
        }
    }
}

fn evaluate_nodes(
    left: &NodeSelection,
    right: &NodeSelection,
    document: &Document,
    control: &mut InvocationControl,
) -> Result<bool, DeepEqualEvaluationFailure> {
    let left =
        select_nodes(left, document, control).map_err(DeepEqualEvaluationFailure::Control)?;
    let right =
        select_nodes(right, document, control).map_err(DeepEqualEvaluationFailure::Control)?;
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.into_iter().zip(right) {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(DeepEqualEvaluationFailure::Control)?;
        if !nodes_deep_equal(document, left, right) {
            return Ok(false);
        }
    }
    Ok(true)
}

fn select_nodes(
    selection: &NodeSelection,
    document: &Document,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    let mut selected = Vec::new();
    select_below(
        selection,
        document,
        document.document_node(),
        &mut selected,
        control,
    )?;
    Ok(selected)
}

fn select_below(
    selection: &NodeSelection,
    document: &Document,
    parent: NodeId,
    selected: &mut Vec<NodeId>,
    control: &mut InvocationControl,
) -> Result<(), ControlFailure> {
    let children = document.children(parent);
    let mut matching_children = Vec::new();
    for child in children.iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if matches_child(selection, document, child) {
            matching_children.push(child);
        }
    }
    let position = match selection {
        NodeSelection::DescendantAttribute { position, .. }
        | NodeSelection::DescendantComment { position } => *position,
    };
    if let Some(node) = matching_children.get(position - 1).copied() {
        match selection {
            NodeSelection::DescendantAttribute { attribute, .. } => {
                for candidate in document.attributes(node).iter().copied() {
                    control.charge(WorkDomain::XPathNodeVisit, 1)?;
                    if document
                        .name(candidate)
                        .is_some_and(|name| name.namespace.is_none() && name.local == *attribute)
                    {
                        selected.push(candidate);
                    }
                }
            }
            NodeSelection::DescendantComment { .. } => selected.push(node),
        }
    }
    for child in children.iter().copied() {
        if document.kind(child) == NodeKind::Element {
            select_below(selection, document, child, selected, control)?;
        }
    }
    Ok(())
}

fn matches_child(selection: &NodeSelection, document: &Document, node: NodeId) -> bool {
    match selection {
        NodeSelection::DescendantAttribute { element, .. } => {
            document.kind(node) == NodeKind::Element
                && document
                    .name(node)
                    .is_some_and(|name| name.namespace.is_none() && name.local == *element)
        }
        NodeSelection::DescendantComment { .. } => document.kind(node) == NodeKind::Comment,
    }
}

fn nodes_deep_equal(document: &Document, left: NodeId, right: NodeId) -> bool {
    document.kind(left) == document.kind(right)
        && document.name(left) == document.name(right)
        && document.value(left) == document.value(right)
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}

#[cfg(test)]
mod tests {
    use super::{evaluate, parse};
    use crate::execution_control_experiment::InvocationControl;
    use crate::xdm::owned_tree_experiment::{Document, SourceLocation};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn document() -> Document {
        let parsed = parse_document(
            "urn:fastxslt:deep-equal:unit",
            br#"<doc><!--same--><a a="x"/><a a="x"/><b a="x"/><c c="x"/><!--same--><!--other--></doc>"#,
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .expect("parse focused document");
        Document::from_parsed(parsed).expect("build focused document")
    }

    fn location() -> SourceLocation {
        SourceLocation {
            resource: "urn:fastxslt:deep-equal:expression".to_owned(),
            span: 0..1,
        }
    }

    #[test]
    fn compares_values_and_expanded_names_without_using_node_identity() {
        let document = document();
        let mut control = InvocationControl::unbounded();
        let equal = parse("deep-equal(//a[1]/@a, //a[2]/@a)", &location())
            .expect("parse attribute equality");
        assert!(
            evaluate(&equal, Some(&document), &mut control).expect("evaluate equal attributes")
        );

        let equal_value = parse("deep-equal(//a[1]/@a, //c[1]/@c)", &location())
            .expect("parse attribute comparison");
        assert!(
            !evaluate(&equal_value, Some(&document), &mut control)
                .expect("compare equal values under different names")
        );

        let comments = parse("deep-equal(//comment()[1], //comment()[3])", &location())
            .expect("parse comment comparison");
        assert!(!evaluate(&comments, Some(&document), &mut control).expect("evaluate comments"));
    }

    #[test]
    fn compares_qt3_xs_int_values_numerically() {
        let mut control = InvocationControl::unbounded();
        let equal = parse(
            "fn:deep-equal((xs:int(\"-2147483648\")),(xs:int(\"-2147483648\")))",
            &location(),
        )
        .expect("parse typed integer equality");
        assert!(evaluate(&equal, None, &mut control).expect("evaluate typed integers"));
    }

    #[test]
    fn normalizes_exact_decimal_scale_without_binary_floating_point() {
        let mut control = InvocationControl::unbounded();
        let equal = parse(
            "fn:deep-equal((xs:decimal(\"1.0\")),(xs:decimal(\"1.00\")))",
            &location(),
        )
        .expect("parse exact decimal equality");
        assert!(evaluate(&equal, None, &mut control).expect("evaluate exact decimals"));
    }

    #[test]
    fn enforces_the_xs_long_value_range() {
        let in_range = parse(
            "fn:deep-equal((xs:long(\"9223372036854775807\")),(xs:long(\"9223372036854775807\")))",
            &location(),
        )
        .expect("parse maximum xs:long equality");
        assert!(
            evaluate(&in_range, None, &mut InvocationControl::unbounded())
                .expect("evaluate xs:long equality")
        );
        assert!(
            parse(
                "fn:deep-equal((xs:long(\"9223372036854775808\")),(xs:long(\"9223372036854775808\")))",
                &location(),
            )
            .is_err()
        );
    }

    #[test]
    fn enforces_the_xs_unsigned_short_value_range() {
        let in_range = parse(
            "fn:deep-equal((xs:unsignedShort(\"65535\")),(xs:unsignedShort(\"65535\")))",
            &location(),
        )
        .expect("parse maximum xs:unsignedShort equality");
        assert!(
            evaluate(&in_range, None, &mut InvocationControl::unbounded())
                .expect("evaluate xs:unsignedShort equality")
        );
        for invalid in ["-1", "65536"] {
            assert!(
                parse(
                    &format!(
                        "fn:deep-equal((xs:unsignedShort(\"{invalid}\")),(xs:unsignedShort(\"{invalid}\")))"
                    ),
                    &location(),
                )
                .is_err()
            );
        }
    }

    #[test]
    fn enforces_the_xs_negative_integer_value_space() {
        let in_range = parse(
            "fn:deep-equal((xs:negativeInteger(\"-1\")),(xs:negativeInteger(\"-1\")))",
            &location(),
        )
        .expect("parse upper xs:negativeInteger value");
        assert!(
            evaluate(&in_range, None, &mut InvocationControl::unbounded())
                .expect("evaluate xs:negativeInteger equality")
        );
        for invalid in ["0", "1"] {
            assert!(
                parse(
                    &format!(
                        "fn:deep-equal((xs:negativeInteger(\"{invalid}\")),(xs:negativeInteger(\"{invalid}\")))"
                    ),
                    &location(),
                )
                .is_err()
            );
        }
    }

    #[test]
    fn enforces_the_xs_positive_integer_value_space() {
        let in_range = parse(
            "fn:deep-equal((xs:positiveInteger(\"1\")),(xs:positiveInteger(\"1\")))",
            &location(),
        )
        .expect("parse lower xs:positiveInteger value");
        assert!(
            evaluate(&in_range, None, &mut InvocationControl::unbounded())
                .expect("evaluate xs:positiveInteger equality")
        );
        for invalid in ["0", "-1"] {
            assert!(
                parse(
                    &format!(
                        "fn:deep-equal((xs:positiveInteger(\"{invalid}\")),(xs:positiveInteger(\"{invalid}\")))"
                    ),
                    &location(),
                )
                .is_err()
            );
        }
    }

    #[test]
    fn enforces_the_xs_unsigned_long_value_range() {
        let upper = parse(
            "fn:deep-equal((xs:unsignedLong(\"18446744073709551615\")),(xs:unsignedLong(\"18446744073709551615\")))",
            &location(),
        )
        .expect("parse upper xs:unsignedLong value");
        assert!(
            evaluate(&upper, None, &mut InvocationControl::unbounded())
                .expect("evaluate xs:unsignedLong equality")
        );
        for invalid in ["-1", "18446744073709551616"] {
            assert!(
                parse(
                    &format!(
                        "fn:deep-equal((xs:unsignedLong(\"{invalid}\")),(xs:unsignedLong(\"{invalid}\")))"
                    ),
                    &location(),
                )
                .is_err()
            );
        }
    }

    #[test]
    fn enforces_the_xs_non_positive_integer_value_space() {
        for valid in ["-1", "0"] {
            let parsed = parse(
                &format!(
                    "fn:deep-equal((xs:nonPositiveInteger(\"{valid}\")),(xs:nonPositiveInteger(\"{valid}\")))"
                ),
                &location(),
            )
            .expect("parse valid xs:nonPositiveInteger value");
            assert!(
                evaluate(&parsed, None, &mut InvocationControl::unbounded())
                    .expect("evaluate xs:nonPositiveInteger equality")
            );
        }
        assert!(
            parse(
                "fn:deep-equal((xs:nonPositiveInteger(\"1\")),(xs:nonPositiveInteger(\"1\")))",
                &location(),
            )
            .is_err()
        );
    }

    #[test]
    fn enforces_the_xs_non_negative_integer_value_space() {
        for valid in ["0", "1"] {
            let parsed = parse(
                &format!(
                    "fn:deep-equal((xs:nonNegativeInteger(\"{valid}\")),(xs:nonNegativeInteger(\"{valid}\")))"
                ),
                &location(),
            )
            .expect("parse valid xs:nonNegativeInteger value");
            assert!(
                evaluate(&parsed, None, &mut InvocationControl::unbounded())
                    .expect("evaluate xs:nonNegativeInteger equality")
            );
        }
        assert!(
            parse(
                "fn:deep-equal((xs:nonNegativeInteger(\"-1\")),(xs:nonNegativeInteger(\"-1\")))",
                &location(),
            )
            .is_err()
        );
    }

    #[test]
    fn enforces_the_xs_short_value_range() {
        for valid in ["-32768", "32767"] {
            let parsed = parse(
                &format!("fn:deep-equal((xs:short(\"{valid}\")),(xs:short(\"{valid}\")))"),
                &location(),
            )
            .expect("parse valid xs:short value");
            assert!(
                evaluate(&parsed, None, &mut InvocationControl::unbounded())
                    .expect("evaluate xs:short equality")
            );
        }
        for invalid in ["-32769", "32768"] {
            assert!(
                parse(
                    &format!("fn:deep-equal((xs:short(\"{invalid}\")),(xs:short(\"{invalid}\")))"),
                    &location(),
                )
                .is_err()
            );
        }
    }

    #[test]
    fn compares_atomic_sequences_in_order_and_flattens_empty_parentheses() {
        let unequal = parse("fn:deep-equal((1, 2), (2, 1))", &location())
            .expect("parse ordered integer sequences");
        let mut control = InvocationControl::unbounded();
        assert!(!evaluate(&unequal, None, &mut control).expect("compare ordered sequences"));
        assert_eq!(
            control.consumed(crate::execution_control_experiment::WorkDomain::XPathOperation),
            2
        );

        let empty =
            parse("fn:deep-equal((()), ())", &location()).expect("parse nested empty sequences");
        assert!(
            evaluate(&empty, None, &mut InvocationControl::unbounded())
                .expect("compare empty sequences")
        );
        let string = parse("fn:deep-equal(xs:string(\"A\"), (\"A\"))", &location())
            .expect("parse equivalent string forms");
        assert!(
            evaluate(&string, None, &mut InvocationControl::unbounded()).expect("compare strings")
        );
    }

    #[test]
    fn applies_only_the_admitted_atomic_type_comparisons() {
        for expression in [
            "fn:deep-equal(xs:anyURI(\"urn:example\"), xs:string(\"urn:example\"))",
            "fn:deep-equal(xs:integer(1), xs:decimal(1.0))",
        ] {
            let parsed = parse(expression, &location()).expect("parse admitted atomic comparison");
            assert!(
                evaluate(&parsed, None, &mut InvocationControl::unbounded())
                    .expect("evaluate admitted atomic comparison")
            );
        }
        let fractional = parse(
            "fn:deep-equal(xs:integer(1), xs:decimal(1.01))",
            &location(),
        )
        .expect("parse unequal exact numeric comparison");
        assert!(
            !evaluate(&fractional, None, &mut InvocationControl::unbounded())
                .expect("evaluate exact numeric comparison")
        );
    }

    #[test]
    fn applies_float_promotion_and_deep_equal_nan_rules() {
        for expression in [
            "fn:deep-equal(xs:decimal(1.01), xs:float(1.01))",
            "fn:deep-equal(xs:decimal(1.01), xs:double(1.01))",
            "fn:deep-equal(xs:float(\"INF\"), xs:double(\"INF\"))",
            "fn:deep-equal(xs:float(\"NaN\"), xs:double(\"NaN\"))",
        ] {
            let parsed = parse(expression, &location()).expect("parse floating comparison");
            assert!(
                evaluate(&parsed, None, &mut InvocationControl::unbounded())
                    .expect("evaluate floating comparison")
            );
        }
        let distinct = parse(
            "fn:deep-equal(xs:float(1.01), xs:double(1.01))",
            &location(),
        )
        .expect("parse promoted float/double comparison");
        assert!(
            !evaluate(&distinct, None, &mut InvocationControl::unbounded())
                .expect("evaluate promoted float/double comparison")
        );
    }

    #[test]
    fn normalizes_only_valid_boolean_lexicals_and_functions() {
        for expression in [
            "fn:deep-equal(xs:boolean(\"1\"), true())",
            "fn:deep-equal(xs:boolean(\"0\"), false())",
        ] {
            let parsed = parse(expression, &location()).expect("parse boolean comparison");
            assert!(
                evaluate(&parsed, None, &mut InvocationControl::unbounded())
                    .expect("evaluate boolean comparison")
            );
        }
        assert!(parse("fn:deep-equal(xs:boolean(\"yes\"), true())", &location(),).is_err());
    }

    #[test]
    fn retains_admitted_calendar_types_and_validates_their_fields() {
        let equal_dates = parse(
            "fn:deep-equal(xs:date(\"1993-03-31\"), xs:date(\"1993-03-31\"))",
            &location(),
        )
        .expect("parse equal typed dates");
        assert!(
            evaluate(&equal_dates, None, &mut InvocationControl::unbounded())
                .expect("compare typed dates")
        );
        let typed_and_string = parse(
            "fn:deep-equal(xs:time(\"12:30:00\"), \"12:30:00\")",
            &location(),
        )
        .expect("parse typed time and string");
        assert!(
            !evaluate(&typed_and_string, None, &mut InvocationControl::unbounded())
                .expect("compare typed time and string")
        );
        for expression in [
            "fn:deep-equal(xs:date(\"1993-02-29\"), xs:date(\"1993-02-29\"))",
            "fn:deep-equal(xs:time(\"24:01:00\"), xs:time(\"24:01:00\"))",
            "fn:deep-equal(xs:dateTime(\"1972-13-01T00:00:00\"), xs:dateTime(\"1972-13-01T00:00:00\"))",
        ] {
            assert!(parse(expression, &location()).is_err());
        }
    }
}
