use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xpath::constant_integer_experiment;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocationPath {
    pub(crate) steps: Vec<PathStep>,
    origin: PathOrigin,
    final_predicate: Option<ExistencePredicate>,
    step_position_predicates: Vec<Option<PositionPredicate>>,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathOrigin {
    ContextItem,
    DocumentNode,
    Relative,
    Descendant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathStep {
    ChildNamed(String),
    ChildAnyElement,
    ChildAnyNode,
    AttributeNamed(String),
    AttributeAny,
}

impl PathStep {
    fn from_validated(value: &str) -> Self {
        if let Some(name_test) = value
            .strip_prefix("attribute::")
            .or_else(|| value.strip_prefix('@'))
        {
            return match name_test {
                "*" | "node()" => Self::AttributeAny,
                _ => Self::AttributeNamed(name_test.to_owned()),
            };
        }
        let name_test = value.strip_prefix("child::").unwrap_or(value);
        match name_test {
            "*" => Self::ChildAnyElement,
            "node()" => Self::ChildAnyNode,
            _ => Self::ChildNamed(name_test.to_owned()),
        }
    }

    fn uses_attribute_axis(&self) -> bool {
        matches!(self, Self::AttributeNamed(_) | Self::AttributeAny)
    }
}

impl PartialEq<&str> for PathStep {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::ChildNamed(value) | Self::AttributeNamed(value) => value == *other,
            Self::ChildAnyElement => *other == "*",
            Self::ChildAnyNode => *other == "node()",
            Self::AttributeAny => matches!(*other, "*" | "node()"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PredicateAxis {
    Child,
    Attribute,
    Ancestor,
    AncestorOrSelf,
    DescendantOrSelf,
    Parent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExistencePredicate {
    axis: PredicateAxis,
    name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositionPredicate {
    Select(usize),
    Last,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathFailure {
    Invalid {
        detail: String,
        location: SourceLocation,
    },
    Unsupported {
        detail: String,
        location: SourceLocation,
    },
}

pub(crate) fn parse_location_path(
    expression: &str,
    location: SourceLocation,
) -> Result<LocationPath, PathFailure> {
    if expression.is_empty() {
        return Err(PathFailure::Invalid {
            detail: "the path expression is empty".to_owned(),
            location,
        });
    }
    if expression == "." {
        return Ok(LocationPath {
            steps: Vec::new(),
            origin: PathOrigin::ContextItem,
            final_predicate: None,
            step_position_predicates: Vec::new(),
            location,
        });
    }
    if expression == "/" {
        return Ok(LocationPath {
            steps: Vec::new(),
            origin: PathOrigin::DocumentNode,
            final_predicate: None,
            step_position_predicates: Vec::new(),
            location,
        });
    }
    let (expression, final_predicate) = parse_final_axis_predicate(expression);
    let (expression, origin) = expression
        .strip_prefix("//")
        .map_or((expression, PathOrigin::Relative), |expression| {
            (expression, PathOrigin::Descendant)
        });
    let parsed_steps = if final_predicate.is_none() {
        parse_position_steps(expression)
    } else {
        Some((
            expression.split('/').map(str::to_owned).collect(),
            vec![None; expression.split('/').count()],
        ))
    };
    if expression.starts_with('/')
        || expression.ends_with('/')
        || expression.contains("//")
        || parsed_steps.is_none()
    {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice supports only relative unprefixed child-name paths: {expression}"
            ),
            location,
        });
    }

    if !expression.is_ascii() {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice does not yet classify or evaluate non-ASCII child names: {expression}"
            ),
            location,
        });
    }

    let (steps, step_position_predicates) = parsed_steps.expect("checked above");
    if steps.iter().any(|step| step == "." || step == "..") {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice supports the context item only as the complete expression: {expression}"
            ),
            location,
        });
    }
    if steps.iter().any(|step| {
        let name_test = step
            .strip_prefix("child::")
            .or_else(|| step.strip_prefix("attribute::"))
            .or_else(|| step.strip_prefix('@'))
            .unwrap_or(step);
        !matches!(name_test, "*" | "node()") && !is_ascii_ncname(name_test)
    }) {
        if steps.iter().any(|step| {
            step.chars().any(|character| {
                !character.is_ascii_alphanumeric() && !matches!(character, '_' | '-' | '.')
            })
        }) {
            return Err(PathFailure::Unsupported {
                detail: format!(
                    "the expression uses syntax outside the private location-path grammar: {expression}"
                ),
                location,
            });
        }
        return Err(PathFailure::Invalid {
            detail: format!("the private slice found an invalid child name in: {expression}"),
            location,
        });
    }
    let steps = lower_validated_steps(&steps, origin == PathOrigin::Descendant, &location)?;
    Ok(LocationPath {
        steps,
        origin,
        final_predicate,
        step_position_predicates,
        location,
    })
}

fn lower_validated_steps(
    steps: &[String],
    starts_with_descendant_search: bool,
    location: &SourceLocation,
) -> Result<Vec<PathStep>, PathFailure> {
    let steps: Vec<_> = steps
        .iter()
        .map(|step| PathStep::from_validated(step))
        .collect();
    if starts_with_descendant_search && steps.first().is_some_and(PathStep::uses_attribute_axis) {
        return Err(PathFailure::Unsupported {
            detail: "the private slice does not yet expand leading // before an attribute step"
                .to_owned(),
            location: location.clone(),
        });
    }
    Ok(steps)
}

fn parse_final_axis_predicate(expression: &str) -> (&str, Option<ExistencePredicate>) {
    let Some((path, predicate)) = expression.split_once('[') else {
        return (expression, None);
    };
    let Some(predicate) = predicate.strip_suffix(']') else {
        return (expression, None);
    };
    let (axis, name) = if let Some(name) = predicate.strip_prefix("child::") {
        (PredicateAxis::Child, name)
    } else if let Some(name) = predicate.strip_prefix("attribute::") {
        (PredicateAxis::Attribute, name)
    } else if let Some(name) = predicate.strip_prefix("ancestor::") {
        (PredicateAxis::Ancestor, name)
    } else if let Some(name) = predicate.strip_prefix("ancestor-or-self::") {
        (PredicateAxis::AncestorOrSelf, name)
    } else if let Some(name) = predicate.strip_prefix("descendant-or-self::") {
        (PredicateAxis::DescendantOrSelf, name)
    } else if let Some(name) = predicate.strip_prefix("parent::") {
        (PredicateAxis::Parent, name)
    } else {
        return (expression, None);
    };
    if path.is_empty() || path.contains('[') || !is_ascii_ncname(name) {
        return (expression, None);
    }
    (
        path,
        Some(ExistencePredicate {
            axis,
            name: name.to_owned(),
        }),
    )
}

fn parse_position_steps(expression: &str) -> Option<(Vec<String>, Vec<Option<PositionPredicate>>)> {
    let raw_steps = split_path_steps(expression)?;
    let mut steps = Vec::with_capacity(raw_steps.len());
    let mut predicates = Vec::with_capacity(raw_steps.len());
    for raw_step in raw_steps {
        let (name, predicate) = if let Some((name, predicate)) = raw_step.split_once('[') {
            let predicate = predicate.strip_suffix(']')?;
            if name.is_empty() || predicate.contains(['[', ']']) {
                return None;
            }
            let predicate = if predicate.trim() == "last()" {
                PositionPredicate::Last
            } else {
                let value = constant_integer_experiment::evaluate(predicate).ok()?;
                usize::try_from(value)
                    .ok()
                    .filter(|position| *position > 0)
                    .map_or(PositionPredicate::Never, PositionPredicate::Select)
            };
            (name, Some(predicate))
        } else {
            (raw_step, None)
        };
        if name.is_empty() {
            return None;
        }
        steps.push(name.to_owned());
        predicates.push(predicate);
    }
    Some((steps, predicates))
}

fn split_path_steps(expression: &str) -> Option<Vec<&str>> {
    let mut steps = Vec::new();
    let mut start = 0;
    let mut bracket_depth: usize = 0;
    for (offset, character) in expression.char_indices() {
        match character {
            '[' => bracket_depth += 1,
            ']' => {
                bracket_depth = bracket_depth.checked_sub(1)?;
            }
            '/' if bracket_depth == 0 => {
                steps.push(&expression[start..offset]);
                start = offset + 1;
            }
            _ => {}
        }
    }
    if bracket_depth != 0 {
        return None;
    }
    steps.push(&expression[start..]);
    Some(steps)
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}

#[cfg(test)]
pub(crate) fn evaluate_location_path(
    document: &Document,
    context: NodeId,
    path: &LocationPath,
) -> Vec<NodeId> {
    let mut control = InvocationControl::unbounded();
    evaluate_location_path_controlled(document, context, path, &mut control)
        .expect("unbounded private control cannot reject XPath work")
}

pub(crate) fn evaluate_location_path_controlled(
    document: &Document,
    context: NodeId,
    path: &LocationPath,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    match path.origin {
        PathOrigin::ContextItem => {
            control.charge(WorkDomain::XPathNodeVisit, 1)?;
            return Ok(vec![context]);
        }
        PathOrigin::DocumentNode => {
            control.charge(WorkDomain::XPathNodeVisit, 1)?;
            return Ok(vec![document.document_node()]);
        }
        PathOrigin::Relative | PathOrigin::Descendant => {}
    }
    let mut current = vec![context];
    for (step_index, step) in path.steps.iter().enumerate() {
        let mut next = Vec::new();
        for node in current {
            let candidates = if step_index == 0 && path.origin == PathOrigin::Descendant {
                descendant_nodes(document, node, control)?
            } else if step.uses_attribute_axis() {
                document.attributes(node).to_vec()
            } else {
                document.children(node).to_vec()
            };
            let mut named_candidates = Vec::new();
            for child in candidates {
                if step_index != 0 || path.origin != PathOrigin::Descendant {
                    control.charge(WorkDomain::XPathNodeVisit, 1)?;
                }
                if child_matches_name_test(document, child, step) {
                    named_candidates.push(child);
                }
            }
            let matching_count = named_candidates.len();
            for (offset, child) in named_candidates.into_iter().enumerate() {
                let position_matches = match path.step_position_predicates[step_index] {
                    Some(PositionPredicate::Select(position)) => position == offset + 1,
                    Some(PositionPredicate::Last) => offset + 1 == matching_count,
                    Some(PositionPredicate::Never) => false,
                    None => true,
                };
                let existence_matches = if step_index + 1 == path.steps.len()
                    && let Some(predicate) = &path.final_predicate
                {
                    match predicate.axis {
                        PredicateAxis::Child => {
                            has_named_child(document, child, &predicate.name, control)?
                        }
                        PredicateAxis::Attribute => {
                            has_named_attribute(document, child, &predicate.name, control)?
                        }
                        PredicateAxis::Ancestor => {
                            has_named_ancestor(document, child, &predicate.name, false, control)?
                        }
                        PredicateAxis::AncestorOrSelf => {
                            has_named_ancestor(document, child, &predicate.name, true, control)?
                        }
                        PredicateAxis::DescendantOrSelf => {
                            has_named_descendant_or_self(document, child, &predicate.name, control)?
                        }
                        PredicateAxis::Parent => {
                            has_named_parent(document, child, &predicate.name, control)?
                        }
                    }
                } else {
                    true
                };
                if position_matches && existence_matches {
                    next.push(child);
                }
            }
        }
        current = next;
    }
    Ok(current)
}

fn child_matches_name_test(document: &Document, child: NodeId, name_test: &PathStep) -> bool {
    match name_test {
        PathStep::ChildNamed(required) => {
            document.kind(child) == NodeKind::Element
                && document
                    .name(child)
                    .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())
        }
        PathStep::ChildAnyElement => document.kind(child) == NodeKind::Element,
        PathStep::ChildAnyNode => true,
        PathStep::AttributeNamed(required) => {
            document.kind(child) == NodeKind::Attribute
                && document
                    .name(child)
                    .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())
        }
        PathStep::AttributeAny => document.kind(child) == NodeKind::Attribute,
    }
}

fn descendant_nodes(
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    let mut descendants = Vec::new();
    let mut pending: Vec<_> = document.children(context).iter().rev().copied().collect();
    while let Some(node) = pending.pop() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        pending.extend(document.children(node).iter().rev().copied());
        descendants.push(node);
    }
    Ok(descendants)
}

fn has_named_child(
    document: &Document,
    node: NodeId,
    required: &str,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for child in document.children(node).iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if node_has_unnamespaced_name(document, child, required) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn has_named_attribute(
    document: &Document,
    node: NodeId,
    required: &str,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for attribute in document.attributes(node).iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if document.kind(attribute) == NodeKind::Attribute
            && document
                .name(attribute)
                .is_some_and(|name| name.namespace.is_none() && name.local == required)
        {
            return Ok(true);
        }
    }
    Ok(false)
}

fn has_named_descendant_or_self(
    document: &Document,
    node: NodeId,
    required: &str,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let mut pending = vec![node];
    while let Some(candidate) = pending.pop() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if node_has_unnamespaced_name(document, candidate, required) {
            return Ok(true);
        }
        pending.extend(document.children(candidate).iter().rev().copied());
    }
    Ok(false)
}

fn has_named_parent(
    document: &Document,
    node: NodeId,
    required: &str,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let Some(parent) = document.parent(node) else {
        return Ok(false);
    };
    control.charge(WorkDomain::XPathNodeVisit, 1)?;
    Ok(node_has_unnamespaced_name(document, parent, required))
}

fn has_named_ancestor(
    document: &Document,
    node: NodeId,
    required: &str,
    include_self: bool,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let mut ancestor = include_self
        .then_some(node)
        .or_else(|| document.parent(node));
    while let Some(node) = ancestor {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if node_has_unnamespaced_name(document, node, required) {
            return Ok(true);
        }
        ancestor = document.parent(node);
    }
    Ok(false)
}

fn node_has_unnamespaced_name(document: &Document, node: NodeId, required: &str) -> bool {
    document.kind(node) == NodeKind::Element
        && document
            .name(node)
            .is_some_and(|name| name.namespace.is_none() && name.local == required)
}

#[cfg(test)]
#[path = "path_experiment_tests.rs"]
mod tests;
