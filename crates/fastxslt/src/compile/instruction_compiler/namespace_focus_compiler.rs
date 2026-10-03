//! Eligibility for the first qualified namespace-focus sequence slice.

use super::{
    CompileFailure, Document, Instruction, NodeId, SortKey, ValueExpression,
    compile_sequence_excluding, map_path_failure, unsupported,
};
use crate::xslt::golden_semantics_experiment::ApplySelection;
use crate::xslt::golden_semantics_experiment::NamespaceScalarKind;
use crate::xslt::golden_semantics_experiment::SortSelect;

pub(super) fn compile_scalar(
    expression: &str,
    location: &super::SourceLocation,
    first_node: bool,
) -> Result<ValueExpression, CompileFailure> {
    let expression = expression.trim();
    if let Some((left, right, equal)) = generated_identity_operands(expression) {
        let parse = |operand: &str| {
            crate::xpath::path_experiment::qualified_nodes::parse(operand, location.clone())
                .map_err(map_path_failure)
        };
        return Ok(ValueExpression::QualifiedGeneratedIdentityComparison {
            left: parse(left)?,
            right: parse(right)?,
            equal,
            first_node,
        });
    }
    for (function, kind) in [
        ("name", NamespaceScalarKind::Name),
        ("local-name", NamespaceScalarKind::LocalName),
        ("namespace-uri", NamespaceScalarKind::NamespaceUri),
        ("count", NamespaceScalarKind::Count),
    ] {
        if let Some(argument) = expression
            .strip_prefix(function)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
        {
            let path =
                crate::xpath::path_experiment::qualified_nodes::parse(argument, location.clone())
                    .map_err(map_path_failure)?;
            if matches!(
                path,
                crate::xpath::path_experiment::qualified_nodes::QualifiedLocationPath::Namespace { .. }
            ) {
                return Ok(ValueExpression::NamespacePathScalar {
                    path,
                    kind,
                    first_node,
                });
            }
        }
    }
    Err(unsupported(
        "FXXP1001",
        "this qualified namespace scalar expression is not admitted",
        location,
    ))
}

// Only a top-level comparison of two complete generate-id calls is admitted.
// Quotes and nested path predicates must not be mistaken for its operator.
fn generated_identity_operands(expression: &str) -> Option<(&str, &str, bool)> {
    fn argument(call: &str) -> Option<&str> {
        call.trim()
            .strip_prefix("generate-id(")?
            .strip_suffix(')')
            .map(str::trim)
    }
    let mut quote = None;
    let mut depth = 0_usize;
    for (index, character) in expression.char_indices() {
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.checked_sub(1)?,
            '=' | '!' if depth == 0 => {
                let (width, equal) = match character {
                    '=' => (1, true),
                    '!' if expression[index..].starts_with("!=") => (2, false),
                    _ => return None,
                };
                // Keep empty calls out of this source-path slice.
                let left = argument(&expression[..index])?;
                let right = argument(&expression[index + width..])?;
                return (!left.is_empty() && !right.is_empty()).then_some((left, right, equal));
            }
            _ => {}
        }
    }
    None
}

pub(super) fn compile(
    document: &Document,
    element: NodeId,
    select: &str,
    sorts: Vec<SortKey>,
    sort_nodes: &[NodeId],
) -> Result<Instruction, CompileFailure> {
    let location = document.location(element).clone();
    if sorts.iter().any(|sort| {
        !matches!(
            &sort.select,
            SortSelect::ContextNodeName
                | SortSelect::ContextNodeLocalName
                | SortSelect::ContextPosition
                | SortSelect::ContextSize
                | SortSelect::Literal(_)
        ) && !matches!(&sort.select, SortSelect::LocationPath(path) if path.is_bare_context_item())
    }) {
        return Err(unsupported(
            "FXST1044",
            "this qualified namespace sort-key consumer is not yet admitted",
            &location,
        ));
    }
    let path = crate::xpath::path_experiment::qualified_nodes::parse(select, location.clone())
        .map_err(map_path_failure)?;
    if !sorts.is_empty()
        && matches!(
            path,
            crate::xpath::path_experiment::qualified_nodes::QualifiedLocationPath::Union(_)
        )
    {
        return Err(unsupported(
            "FXST1044",
            "sorting mixed qualified node selections is not yet admitted",
            &location,
        ));
    }
    let body = compile_sequence_excluding(document, element, sort_nodes)?;
    if !body.iter().all(admitted_body) {
        return Err(unsupported(
            "FXST1007",
            "this namespace-focus body consumer is not yet admitted",
            &location,
        ));
    }
    Ok(Instruction::ForEachNodes {
        select: ApplySelection::QualifiedPath(path),
        sorts,
        body,
        location,
    })
}

fn admitted_body(instruction: &Instruction) -> bool {
    use crate::xslt::golden_semantics_experiment::DynamicNamespaceValue;
    match instruction {
        Instruction::LiteralElement {
            attributes,
            attribute_set_names,
            computed_attributes,
            body,
            ..
        } => {
            attributes
                .iter()
                .all(|attribute| admitted_attribute_value(&attribute.value))
                && attribute_set_names.is_empty()
                && computed_attributes.iter().all(admitted_attribute)
                && body.iter().all(admitted_body)
        }
        Instruction::ContextNameElement {
            namespace_override,
            attribute_set_names,
            computed_attributes,
            body,
            ..
        } => {
            namespace_override
                .as_ref()
                .is_none_or(|value| matches!(value, DynamicNamespaceValue::Static(_)))
                && attribute_set_names.is_empty()
                && computed_attributes.iter().all(admitted_attribute)
                && body.iter().all(admitted_body)
        }
        Instruction::Text { .. } => true,
        Instruction::If { test, body, .. } => {
            admitted_boolean(test) && body.iter().all(admitted_body)
        }
        Instruction::Choose {
            branches,
            otherwise,
            ..
        } => {
            branches.iter().all(|branch| {
                admitted_boolean(&branch.test) && branch.body.iter().all(admitted_body)
            }) && otherwise.iter().all(admitted_body)
        }
        Instruction::Copy {
            attributes, body, ..
        } => attributes.is_empty() && body.is_empty(),
        Instruction::Attribute { attribute, .. } => admitted_attribute(attribute),
        Instruction::ValueOf { select, .. } => match select {
            ValueExpression::LocationPath(path)
            | ValueExpression::Xslt10FirstNodeLocationPath(path)
            | ValueExpression::StringPath(path)
            | ValueExpression::Xslt10FirstNodeStringPath(path) => {
                path.is_bare_context_item() || path.is_bare_parent()
            }
            ValueExpression::NodeNamePath(path)
            | ValueExpression::Xslt10FirstNodeNamePath(path)
            | ValueExpression::NodeLocalNamePath(path)
            | ValueExpression::Xslt10FirstNodeLocalNamePath(path)
            | ValueExpression::NodeNamespaceUriPath(path)
            | ValueExpression::Xslt10FirstNodeNamespaceUriPath(path) => path.is_bare_parent(),
            ValueExpression::LiteralString(_)
            | ValueExpression::ContextNodeName
            | ValueExpression::ContextNodeLocalName
            | ValueExpression::ContextNodeNamespaceUri
            | ValueExpression::ContextPosition(_)
            | ValueExpression::ContextSize(_) => true,
            _ => false,
        },
        _ => false,
    }
}

fn admitted_boolean(
    expression: &crate::xslt::golden_semantics_experiment::BooleanExpression,
) -> bool {
    use crate::xslt::golden_semantics_experiment::BooleanExpression;
    match expression {
        BooleanExpression::Constant(_)
        | BooleanExpression::ContextStringContains { .. }
        | BooleanExpression::ContextFocusEquals { .. }
        | BooleanExpression::ContextFocusCompares { .. }
        | BooleanExpression::ContextPositionNotEqualSize(_)
        | BooleanExpression::ContextPositionModuloEquals { .. } => true,
        BooleanExpression::And { left, right } | BooleanExpression::Or { left, right } => {
            admitted_boolean(left) && admitted_boolean(right)
        }
        BooleanExpression::Not(expression) => admitted_boolean(expression),
        _ => false,
    }
}

fn admitted_attribute(
    attribute: &crate::xslt::golden_semantics_experiment::ComputedAttribute,
) -> bool {
    use crate::xslt::golden_semantics_experiment::{DynamicAttributeName, DynamicNamespaceValue};
    let name = attribute
        .dynamic_name
        .as_ref()
        .is_none_or(|name| match name {
            DynamicAttributeName::ContextName {
                namespace_override, ..
            }
            | DynamicAttributeName::Literal {
                namespace_override, ..
            } => namespace_override
                .as_ref()
                .is_none_or(|value| matches!(value, DynamicNamespaceValue::Static(_))),
            _ => false,
        });
    name && admitted_attribute_value(&attribute.value)
}

fn admitted_attribute_value(
    value: &crate::xslt::golden_semantics_experiment::LiteralAttributeValue,
) -> bool {
    use crate::xslt::golden_semantics_experiment::LiteralAttributeValue;
    match value {
        LiteralAttributeValue::Text(_)
        | LiteralAttributeValue::ContextStringValue
        | LiteralAttributeValue::ContextPosition
        | LiteralAttributeValue::ContextSize
        | LiteralAttributeValue::ContextLocalName
        | LiteralAttributeValue::ContextLexicalName => true,
        LiteralAttributeValue::Xslt10SequenceConstructor(body) => body.iter().all(|instruction| {
            matches!(
                instruction,
                Instruction::Text { .. } | Instruction::ValueOf { .. }
            ) && admitted_body(instruction)
        }),
        _ => false,
    }
}
