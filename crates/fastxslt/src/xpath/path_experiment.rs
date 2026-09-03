use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xpath::constant_integer_experiment;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocationPath {
    pub(crate) steps: Vec<PathStep>,
    origin: PathOrigin,
    final_predicate: Option<ExistencePredicate>,
    final_context_predicate: Option<FinalContextPredicate>,
    step_position_predicates: Vec<Option<PositionPredicate>>,
    pub(crate) location: SourceLocation,
}

impl LocationPath {
    pub(crate) fn starts_at_document_node(&self) -> bool {
        self.origin == PathOrigin::DocumentNode
    }

    #[cfg(feature = "workbench")]
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        self.steps.capacity() * std::mem::size_of::<PathStep>()
            + self
                .steps
                .iter()
                .map(PathStep::known_owned_capacity_bytes)
                .sum::<usize>()
            + self
                .final_predicate
                .as_ref()
                .map_or(0, |predicate| predicate.name.capacity())
            + self.step_position_predicates.capacity()
                * std::mem::size_of::<Option<PositionPredicate>>()
            + self.location.resource.capacity()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathOrigin {
    ContextItem,
    DocumentNode,
    EmptySequence,
    Relative,
    Descendant,
    ContextDescendant,
}

impl PathOrigin {
    fn is_leading_descendant(self) -> bool {
        matches!(self, Self::Descendant | Self::ContextDescendant)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FinalContextPredicate {
    TextHasNonWhitespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathStep {
    ChildNamed(String),
    ChildExpandedName(ExpandedName),
    ChildAnyElement,
    ChildAnyNode,
    ChildText,
    ChildComment,
    ChildProcessingInstruction,
    AttributeNamed(String),
    AttributeAny,
    ParentNamed(String),
    ParentAnyElement,
    ParentAnyNode,
    SelfNamed(String),
    SelfAnyElement,
    SelfAnyNode,
    DescendantNamed(String),
    DescendantAnyElement,
    DescendantAnyNode,
    DescendantOrSelfNamed(String),
    DescendantOrSelfAnyElement,
    DescendantOrSelfAnyNode,
}

impl PathStep {
    #[cfg(feature = "workbench")]
    fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::ChildNamed(value)
            | Self::AttributeNamed(value)
            | Self::ParentNamed(value)
            | Self::SelfNamed(value)
            | Self::DescendantNamed(value)
            | Self::DescendantOrSelfNamed(value) => value.capacity(),
            Self::ChildExpandedName(name) => {
                name.local.capacity()
                    + name
                        .namespace
                        .as_ref()
                        .map_or(0, std::string::String::capacity)
            }
            _ => 0,
        }
    }

    fn from_validated(value: &str) -> Option<Self> {
        if value == ".." {
            return Some(Self::ParentAnyNode);
        }
        if let Some(name_test) = value
            .strip_prefix("attribute::")
            .or_else(|| value.strip_prefix('@'))
        {
            return match name_test {
                "*" | "node()" => Some(Self::AttributeAny),
                "text()" => None,
                _ => Some(Self::AttributeNamed(name_test.to_owned())),
            };
        }
        if let Some(name_test) = value.strip_prefix("parent::") {
            return match name_test {
                "*" => Some(Self::ParentAnyElement),
                "node()" => Some(Self::ParentAnyNode),
                "text()" => None,
                _ => Some(Self::ParentNamed(name_test.to_owned())),
            };
        }
        if let Some(name_test) = value.strip_prefix("self::") {
            return match name_test {
                "*" => Some(Self::SelfAnyElement),
                "node()" => Some(Self::SelfAnyNode),
                "text()" => None,
                _ => Some(Self::SelfNamed(name_test.to_owned())),
            };
        }
        if let Some(name_test) = value.strip_prefix("descendant::") {
            return match name_test {
                "*" => Some(Self::DescendantAnyElement),
                "node()" => Some(Self::DescendantAnyNode),
                "text()" => None,
                _ => Some(Self::DescendantNamed(name_test.to_owned())),
            };
        }
        if let Some(name_test) = value.strip_prefix("descendant-or-self::") {
            return match name_test {
                "*" => Some(Self::DescendantOrSelfAnyElement),
                "node()" => Some(Self::DescendantOrSelfAnyNode),
                "text()" => None,
                _ => Some(Self::DescendantOrSelfNamed(name_test.to_owned())),
            };
        }
        let name_test = value.strip_prefix("child::").unwrap_or(value);
        Some(match name_test {
            "*" | "element()" => Self::ChildAnyElement,
            "node()" => Self::ChildAnyNode,
            "text()" => Self::ChildText,
            "comment()" => Self::ChildComment,
            "processing-instruction()" => Self::ChildProcessingInstruction,
            _ => Self::ChildNamed(name_test.to_owned()),
        })
    }

    fn uses_attribute_axis(&self) -> bool {
        matches!(self, Self::AttributeNamed(_) | Self::AttributeAny)
    }

    fn uses_parent_axis(&self) -> bool {
        matches!(
            self,
            Self::ParentNamed(_) | Self::ParentAnyElement | Self::ParentAnyNode
        )
    }

    fn uses_self_axis(&self) -> bool {
        matches!(
            self,
            Self::SelfNamed(_) | Self::SelfAnyElement | Self::SelfAnyNode
        )
    }

    fn uses_descendant_axis(&self) -> bool {
        matches!(
            self,
            Self::DescendantNamed(_) | Self::DescendantAnyElement | Self::DescendantAnyNode
        )
    }

    fn uses_descendant_or_self_axis(&self) -> bool {
        matches!(
            self,
            Self::DescendantOrSelfNamed(_)
                | Self::DescendantOrSelfAnyElement
                | Self::DescendantOrSelfAnyNode
        )
    }
}

impl PartialEq<&str> for PathStep {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::ChildNamed(value)
            | Self::AttributeNamed(value)
            | Self::ParentNamed(value)
            | Self::SelfNamed(value)
            | Self::DescendantNamed(value)
            | Self::DescendantOrSelfNamed(value) => value == *other,
            Self::ChildExpandedName(value) => value.namespace.is_none() && value.local == *other,
            Self::ChildAnyElement
            | Self::ParentAnyElement
            | Self::SelfAnyElement
            | Self::DescendantAnyElement
            | Self::DescendantOrSelfAnyElement => *other == "*",
            Self::ChildAnyNode
            | Self::ParentAnyNode
            | Self::SelfAnyNode
            | Self::DescendantAnyNode
            | Self::DescendantOrSelfAnyNode => *other == "node()",
            Self::ChildText => *other == "text()",
            Self::ChildComment => *other == "comment()",
            Self::ChildProcessingInstruction => *other == "processing-instruction()",
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
        standard_code: &'static str,
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
    validate_expression_opening(expression, &location)?;
    if expression == "." {
        return Ok(origin_only_path(PathOrigin::ContextItem, location));
    }
    if expression == "/" {
        return Ok(origin_only_path(PathOrigin::DocumentNode, location));
    }
    if expression == "()" {
        return Ok(origin_only_path(PathOrigin::EmptySequence, location));
    }
    let (expression, final_context_predicate) = parse_final_context_predicate(expression);
    let (expression, final_predicate) = parse_final_axis_predicate(expression);
    let (expression, origin) = parse_path_origin(expression);
    if expression.is_empty() {
        return Err(invalid_syntax(
            "the descendant path separator must be followed by a step",
            &location,
        ));
    }
    let parsed_steps = if final_predicate.is_none() {
        parse_position_steps(expression)
    } else {
        Some((
            expression.split('/').map(str::to_owned).collect(),
            vec![None; expression.split('/').count()],
        ))
    };
    if expression.ends_with('/') {
        return Err(invalid_syntax(
            format!("the location path ends with an empty step: {expression}"),
            &location,
        ));
    }
    if expression.starts_with('/')
        || (final_predicate.is_some() && expression.contains("//"))
        || parsed_steps.is_none()
    {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice does not yet support this location-path form: {expression}"
            ),
            location,
        });
    }

    if !expression.is_ascii() {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice does not yet classify or evaluate non-ASCII name tests: {expression}"
            ),
            location,
        });
    }

    let (steps, step_position_predicates) = parsed_steps.expect("checked above");
    validate_unambiguous_step_syntax(&steps, expression, &location)?;
    if steps.iter().any(|step| step == ".") {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice supports the context item only as the complete expression: {expression}"
            ),
            location,
        });
    }
    if steps.iter().any(|step| has_unadmitted_name_test(step)) {
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
        return Err(invalid_syntax(
            format!("the private slice found an invalid name test in: {expression}"),
            &location,
        ));
    }
    let steps = lower_validated_steps(&steps, &location)?;
    Ok(LocationPath {
        steps,
        origin,
        final_predicate,
        final_context_predicate,
        step_position_predicates,
        location,
    })
}

/// Parses the deliberately narrow qualified-child path needed by static `XPath`
/// expressions while preserving the existing location-path execution backend.
pub(crate) fn parse_qualified_child_path(
    expression: &str,
    location: SourceLocation,
    mut resolve_prefix: impl FnMut(&str) -> Option<String>,
) -> Result<LocationPath, PathFailure> {
    validate_expression_opening(expression, &location)?;
    if !expression.is_ascii()
        || expression.starts_with('/')
        || expression.ends_with('/')
        || expression.contains("//")
    {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice does not support this qualified location-path form: {expression}"
            ),
            location,
        });
    }

    let mut found_qualified_step = false;
    let mut steps = Vec::new();
    for step in expression.split('/') {
        let Some((prefix, local)) = step.split_once(':') else {
            if !is_ascii_ncname(step) {
                return Err(invalid_syntax(
                    format!("the qualified path contains an invalid name test: {expression}"),
                    &location,
                ));
            }
            steps.push(PathStep::ChildNamed(step.to_owned()));
            continue;
        };
        if !is_ascii_ncname(prefix) || !is_ascii_ncname(local) || local.contains(':') {
            return Err(invalid_syntax(
                format!("the qualified path contains an invalid QName: {step}"),
                &location,
            ));
        }
        let namespace = resolve_prefix(prefix).ok_or_else(|| PathFailure::Invalid {
            standard_code: "XPST0081",
            detail: format!("the XPath name test uses an unbound prefix: {prefix}"),
            location: location.clone(),
        })?;
        found_qualified_step = true;
        steps.push(PathStep::ChildExpandedName(ExpandedName {
            namespace: Some(namespace),
            local: local.to_owned(),
        }));
    }
    if !found_qualified_step {
        return Err(PathFailure::Unsupported {
            detail: format!("the location path has no qualified child step: {expression}"),
            location,
        });
    }
    let step_count = steps.len();
    Ok(LocationPath {
        steps,
        origin: PathOrigin::Relative,
        final_predicate: None,
        final_context_predicate: None,
        step_position_predicates: vec![None; step_count],
        location,
    })
}

fn has_unadmitted_name_test(step: &str) -> bool {
    let name_test = step
        .strip_prefix("child::")
        .or_else(|| step.strip_prefix("attribute::"))
        .or_else(|| step.strip_prefix("parent::"))
        .or_else(|| step.strip_prefix("self::"))
        .or_else(|| step.strip_prefix("descendant::"))
        .or_else(|| step.strip_prefix("descendant-or-self::"))
        .or_else(|| step.strip_prefix('@'))
        .unwrap_or(if step == ".." { "node()" } else { step });
    let admitted_child_kind = (!step.contains("::") || step.starts_with("child::"))
        && matches!(
            name_test,
            "element()" | "comment()" | "processing-instruction()"
        );
    !matches!(name_test, "*" | "node()" | "text()")
        && !admitted_child_kind
        && !is_ascii_ncname(name_test)
}

fn validate_expression_opening(
    expression: &str,
    location: &SourceLocation,
) -> Result<(), PathFailure> {
    if expression.is_empty() {
        return Err(invalid_syntax("the path expression is empty", location));
    }
    if is_declaration_shaped_xpath_syntax(expression) {
        return Err(invalid_syntax(
            "function declarations are not permitted in an XPath expression",
            location,
        ));
    }
    Ok(())
}

fn is_declaration_shaped_xpath_syntax(expression: &str) -> bool {
    let mut tokens = expression.split_ascii_whitespace();
    matches!(tokens.next(), Some("declare" | "eclare")) && tokens.next() == Some("function")
}

fn validate_unambiguous_step_syntax(
    steps: &[String],
    expression: &str,
    location: &SourceLocation,
) -> Result<(), PathFailure> {
    let has_malformed_namespace_wildcard = steps.iter().any(|step| {
        step.contains(':')
            && !step.contains("::")
            && (step.contains("(:") || step.chars().any(char::is_whitespace))
    });
    if has_malformed_namespace_wildcard {
        return Err(invalid_syntax(
            format!("the location path contains a malformed namespace wildcard: {expression}"),
            location,
        ));
    }
    let has_unknown_axis = steps.iter().any(|step| {
        step.split_once("::").is_some_and(|(axis, _)| {
            !matches!(
                axis,
                "ancestor"
                    | "ancestor-or-self"
                    | "attribute"
                    | "child"
                    | "descendant"
                    | "descendant-or-self"
                    | "following"
                    | "following-sibling"
                    | "namespace"
                    | "parent"
                    | "preceding"
                    | "preceding-sibling"
                    | "self"
            )
        })
    });
    if has_unknown_axis {
        return Err(invalid_syntax(
            format!("the location path uses an unknown axis name: {expression}"),
            location,
        ));
    }
    let has_invalid_axis_node_test = steps.iter().any(|step| {
        step.split_once("::").is_some_and(|(_, node_test)| {
            node_test
                .strip_suffix("()")
                .is_some_and(|name| !is_standard_kind_test_name(name))
        })
    });
    if has_invalid_axis_node_test {
        return Err(invalid_syntax(
            format!("the location path contains an invalid axis node test: {expression}"),
            location,
        ));
    }
    if steps.iter().any(|step| step.ends_with(':')) {
        return Err(invalid_syntax(
            format!("the location path contains an incomplete QName: {expression}"),
            location,
        ));
    }
    Ok(())
}

fn is_standard_kind_test_name(name: &str) -> bool {
    matches!(
        name,
        "attribute"
            | "comment"
            | "document-node"
            | "element"
            | "namespace-node"
            | "node"
            | "processing-instruction"
            | "schema-attribute"
            | "schema-element"
            | "text"
    )
}

fn invalid_syntax(detail: impl Into<String>, location: &SourceLocation) -> PathFailure {
    PathFailure::Invalid {
        standard_code: "XPST0003",
        detail: detail.into(),
        location: location.clone(),
    }
}

fn origin_only_path(origin: PathOrigin, location: SourceLocation) -> LocationPath {
    LocationPath {
        steps: Vec::new(),
        origin,
        final_predicate: None,
        final_context_predicate: None,
        step_position_predicates: Vec::new(),
        location,
    }
}

fn parse_final_context_predicate(expression: &str) -> (&str, Option<FinalContextPredicate>) {
    let Some(path) = expression.strip_suffix("[normalize-space()]") else {
        return (expression, None);
    };
    let final_step = path.rsplit('/').next().unwrap_or(path);
    if path.is_empty()
        || path.contains(['[', ']'])
        || !matches!(final_step, "text()" | "child::text()")
    {
        return (expression, None);
    }
    (path, Some(FinalContextPredicate::TextHasNonWhitespace))
}

fn parse_path_origin(expression: &str) -> (&str, PathOrigin) {
    if let Some(expression) = expression.strip_prefix("()/") {
        (expression, PathOrigin::EmptySequence)
    } else if let Some(expression) = expression.strip_prefix(".//") {
        (expression, PathOrigin::ContextDescendant)
    } else if let Some(expression) = expression.strip_prefix("./") {
        (expression, PathOrigin::Relative)
    } else if let Some(expression) = expression.strip_prefix("//") {
        (expression, PathOrigin::Descendant)
    } else if let Some(expression) = expression.strip_prefix('/') {
        (expression, PathOrigin::DocumentNode)
    } else {
        (expression, PathOrigin::Relative)
    }
}

fn lower_validated_steps(
    steps: &[String],
    location: &SourceLocation,
) -> Result<Vec<PathStep>, PathFailure> {
    let steps: Option<Vec<_>> = steps
        .iter()
        .map(|step| PathStep::from_validated(step))
        .collect();
    let Some(steps) = steps else {
        return Err(PathFailure::Unsupported {
            detail: "the private slice does not support that kind test on the requested axis"
                .to_owned(),
            location: location.clone(),
        });
    };
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
    for (index, raw_step) in raw_steps.iter().copied().enumerate() {
        if raw_step.is_empty() {
            let is_isolated_internal_separator = index > 0
                && index + 1 < raw_steps.len()
                && !raw_steps[index - 1].is_empty()
                && !raw_steps[index + 1].is_empty();
            if !is_isolated_internal_separator {
                return None;
            }
            steps.push("descendant-or-self::node()".to_owned());
            predicates.push(None);
            continue;
        }
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
        PathOrigin::DocumentNode if path.steps.is_empty() => {
            control.charge(WorkDomain::XPathNodeVisit, 1)?;
            return Ok(vec![document.document_node()]);
        }
        PathOrigin::DocumentNode
        | PathOrigin::EmptySequence
        | PathOrigin::Relative
        | PathOrigin::Descendant
        | PathOrigin::ContextDescendant => {}
    }
    let mut current = if path.origin == PathOrigin::EmptySequence {
        Vec::new()
    } else if matches!(
        path.origin,
        PathOrigin::DocumentNode | PathOrigin::Descendant
    ) {
        vec![document.document_node()]
    } else {
        vec![context]
    };
    for (step_index, step) in path.steps.iter().enumerate() {
        let mut next = Vec::new();
        for node in current {
            let candidates =
                step_candidates(document, node, step, step_index, path.origin, control)?;
            let mut named_candidates = Vec::new();
            for child in candidates {
                if (step_index != 0 || !path.origin.is_leading_descendant())
                    && !step.uses_descendant_axis()
                    && !step.uses_descendant_or_self_axis()
                {
                    control.charge(WorkDomain::XPathNodeVisit, 1)?;
                }
                if step_matches_candidate(document, child, step) {
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
                let context_predicate_matches = match path.final_context_predicate {
                    Some(FinalContextPredicate::TextHasNonWhitespace)
                        if step_index + 1 == path.steps.len() =>
                    {
                        document.value(child).is_some_and(|value| {
                            value.chars().any(|character| {
                                !matches!(character, '\u{9}' | '\u{A}' | '\u{D}' | ' ')
                            })
                        })
                    }
                    _ => true,
                };
                if position_matches && existence_matches && context_predicate_matches {
                    next.push(child);
                }
            }
        }
        next.sort_unstable_by_key(|node| document.document_order(*node));
        next.dedup();
        current = next;
    }
    Ok(current)
}

fn step_candidates(
    document: &Document,
    node: NodeId,
    step: &PathStep,
    step_index: usize,
    origin: PathOrigin,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    if step_index == 0 && origin.is_leading_descendant() && step.uses_self_axis() {
        descendant_or_self_nodes(document, node, control)
    } else if step_index == 0 && origin.is_leading_descendant() && step.uses_attribute_axis() {
        descendant_attributes(document, node, control)
    } else if step_index == 0 && origin.is_leading_descendant() {
        descendant_nodes(document, node, control)
    } else if step.uses_attribute_axis() {
        Ok(document.attributes(node).to_vec())
    } else if step.uses_parent_axis() {
        Ok(document.parent(node).into_iter().collect())
    } else if step.uses_self_axis() {
        Ok(vec![node])
    } else if step.uses_descendant_axis() {
        descendant_nodes(document, node, control)
    } else if step.uses_descendant_or_self_axis() {
        descendant_or_self_nodes(document, node, control)
    } else {
        Ok(document.children(node).to_vec())
    }
}

fn step_matches_candidate(document: &Document, child: NodeId, name_test: &PathStep) -> bool {
    match name_test {
        PathStep::ChildNamed(required) => {
            document.kind(child) == NodeKind::Element
                && document
                    .name(child)
                    .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())
        }
        PathStep::ChildExpandedName(required) => {
            document.kind(child) == NodeKind::Element && document.name(child) == Some(required)
        }
        PathStep::ChildAnyElement
        | PathStep::ParentAnyElement
        | PathStep::SelfAnyElement
        | PathStep::DescendantAnyElement
        | PathStep::DescendantOrSelfAnyElement => document.kind(child) == NodeKind::Element,
        PathStep::ChildAnyNode
        | PathStep::ParentAnyNode
        | PathStep::SelfAnyNode
        | PathStep::DescendantAnyNode
        | PathStep::DescendantOrSelfAnyNode => true,
        PathStep::ChildText => document.kind(child) == NodeKind::Text,
        PathStep::ChildComment => document.kind(child) == NodeKind::Comment,
        PathStep::ChildProcessingInstruction => {
            document.kind(child) == NodeKind::ProcessingInstruction
        }
        PathStep::AttributeNamed(required) => {
            document.kind(child) == NodeKind::Attribute
                && document
                    .name(child)
                    .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())
        }
        PathStep::AttributeAny => document.kind(child) == NodeKind::Attribute,
        PathStep::ParentNamed(required) => {
            document.kind(child) == NodeKind::Element
                && document
                    .name(child)
                    .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())
        }
        PathStep::SelfNamed(required) => {
            document.kind(child) == NodeKind::Element
                && document
                    .name(child)
                    .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())
        }
        PathStep::DescendantNamed(required) | PathStep::DescendantOrSelfNamed(required) => {
            document.kind(child) == NodeKind::Element
                && document
                    .name(child)
                    .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())
        }
    }
}

fn descendant_or_self_nodes(
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    control.charge(WorkDomain::XPathNodeVisit, 1)?;
    let mut nodes = vec![context];
    nodes.extend(descendant_nodes(document, context, control)?);
    Ok(nodes)
}

fn descendant_attributes(
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    let descendants = descendant_nodes(document, context, control)?;
    let mut attributes = Vec::new();
    for descendant in descendants {
        for attribute in document.attributes(descendant).iter().copied() {
            control.charge(WorkDomain::XPathNodeVisit, 1)?;
            attributes.push(attribute);
        }
    }
    Ok(attributes)
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
mod evaluation_tests;

#[cfg(test)]
#[path = "path_syntax_tests.rs"]
mod syntax_tests;
