use std::collections::HashSet;

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xpath::constant_integer_experiment;
use crate::xpath::constant_numeric_experiment;
use crate::xpath::language_experiment;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocationPath {
    pub(crate) steps: Vec<PathStep>,
    origin: PathOrigin,
    final_predicate: Option<AxisPredicate>,
    final_context_predicate: Option<FinalContextPredicate>,
    first_step_predicates_use_document_order: bool,
    step_axis_predicates: Vec<Option<AxisPredicate>>,
    step_position_predicates: Vec<Vec<StepPredicate>>,
    pub(crate) location: SourceLocation,
}

impl LocationPath {
    pub(crate) fn starts_at_document_node(&self) -> bool {
        self.origin == PathOrigin::DocumentNode
    }

    pub(crate) fn has_non_simple_position_predicate(&self) -> bool {
        self.step_position_predicates.iter().any(|predicates| {
            predicates.len() > 1
                || predicates.iter().any(|predicate| {
                    matches!(
                        predicate,
                        StepPredicate::Position(
                            PositionPredicate::Compare { .. } | PositionPredicate::LastMinus(_)
                        ) | StepPredicate::ContextName(_)
                    )
                })
        })
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
                .map_or(0, AxisPredicate::known_owned_capacity_bytes)
            + self.step_position_predicates.capacity() * std::mem::size_of::<Vec<StepPredicate>>()
            + self
                .step_position_predicates
                .iter()
                .map(|predicates| {
                    predicates.capacity() * std::mem::size_of::<StepPredicate>()
                        + predicates
                            .iter()
                            .map(|predicate| match predicate {
                                StepPredicate::ContextName(name) => name.len(),
                                StepPredicate::Position(_) => 0,
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self.step_axis_predicates.capacity() * std::mem::size_of::<Option<AxisPredicate>>()
            + self
                .step_axis_predicates
                .iter()
                .flatten()
                .map(AxisPredicate::known_owned_capacity_bytes)
                .sum::<usize>()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FinalContextPredicate {
    TextHasNonWhitespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathStep {
    ChildNamed(String),
    ChildLocalName(String),
    ChildExpandedName(ExpandedName),
    ChildAnyElement,
    ChildAnyNode,
    ChildText,
    ChildComment,
    ChildProcessingInstruction,
    ChildProcessingInstructionNamed(String),
    AttributeNamed(String),
    AttributeExpandedName(ExpandedName),
    AttributeAny,
    AncestorNamed(String),
    AncestorAnyElement,
    AncestorAnyNode,
    AncestorOrSelfNamed(String),
    AncestorOrSelfAnyElement,
    AncestorOrSelfAnyNode,
    ParentNamed(String),
    ParentAnyElement,
    ParentAnyNode,
    SelfNamed(String),
    SelfAnyElement,
    SelfAnyNode,
    SelfText,
    SelfComment,
    SelfProcessingInstruction,
    DescendantNamed(String),
    DescendantAnyElement,
    DescendantAnyNode,
    DescendantOrSelfNamed(String),
    DescendantOrSelfAnyElement,
    DescendantOrSelfAnyNode,
    FollowingNamed(String),
    FollowingAnyElement,
    FollowingAnyNode,
    FollowingText,
    FollowingComment,
    FollowingProcessingInstruction,
    FollowingSiblingNamed(String),
    FollowingSiblingAnyElement,
    FollowingSiblingAnyNode,
    PrecedingNamed(String),
    PrecedingAnyElement,
    PrecedingAnyNode,
    PrecedingText,
    PrecedingComment,
    PrecedingProcessingInstruction,
    PrecedingSiblingNamed(String),
    PrecedingSiblingAnyElement,
    PrecedingSiblingAnyNode,
}

impl PathStep {
    #[cfg(feature = "workbench")]
    fn known_owned_capacity_bytes(&self) -> usize {
        match self {
            Self::ChildNamed(value)
            | Self::ChildLocalName(value)
            | Self::ChildProcessingInstructionNamed(value)
            | Self::AttributeNamed(value)
            | Self::AncestorNamed(value)
            | Self::AncestorOrSelfNamed(value)
            | Self::ParentNamed(value)
            | Self::SelfNamed(value)
            | Self::DescendantNamed(value)
            | Self::DescendantOrSelfNamed(value)
            | Self::FollowingNamed(value)
            | Self::FollowingSiblingNamed(value)
            | Self::PrecedingNamed(value)
            | Self::PrecedingSiblingNamed(value) => value.capacity(),
            Self::ChildExpandedName(name) | Self::AttributeExpandedName(name) => {
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
        if value == "." {
            return Some(Self::SelfAnyNode);
        }
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
        if let Some(name_test) = value.strip_prefix("ancestor::") {
            return Self::from_ancestor_name_test(name_test);
        }
        if let Some(name_test) = value.strip_prefix("ancestor-or-self::") {
            return Self::from_ancestor_or_self_name_test(name_test);
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
            return Some(Self::from_self_name_test(name_test));
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
        if let Some(name_test) = value.strip_prefix("following-sibling::") {
            return match name_test {
                "*" => Some(Self::FollowingSiblingAnyElement),
                "node()" => Some(Self::FollowingSiblingAnyNode),
                "text()" => None,
                _ => Some(Self::FollowingSiblingNamed(name_test.to_owned())),
            };
        }
        if let Some(name_test) = value.strip_prefix("following::") {
            return match name_test {
                "*" => Some(Self::FollowingAnyElement),
                "node()" => Some(Self::FollowingAnyNode),
                "text()" => Some(Self::FollowingText),
                "comment()" => Some(Self::FollowingComment),
                "processing-instruction()" => Some(Self::FollowingProcessingInstruction),
                _ => Some(Self::FollowingNamed(name_test.to_owned())),
            };
        }
        if let Some(name_test) = value.strip_prefix("preceding::") {
            return Some(Self::from_preceding_name_test(name_test));
        }
        if let Some(name_test) = value.strip_prefix("preceding-sibling::") {
            return match name_test {
                "*" => Some(Self::PrecedingSiblingAnyElement),
                "node()" => Some(Self::PrecedingSiblingAnyNode),
                "text()" => None,
                _ => Some(Self::PrecedingSiblingNamed(name_test.to_owned())),
            };
        }
        let name_test = value.strip_prefix("child::").unwrap_or(value);
        if let Some(target) = processing_instruction_target(name_test) {
            return Some(Self::ChildProcessingInstructionNamed(target.to_owned()));
        }
        Some(match name_test {
            "*" | "element()" => Self::ChildAnyElement,
            "node()" => Self::ChildAnyNode,
            "text()" => Self::ChildText,
            "comment()" => Self::ChildComment,
            "processing-instruction()" => Self::ChildProcessingInstruction,
            _ if name_test.strip_prefix("*:").is_some_and(is_ascii_ncname) => {
                Self::ChildLocalName(name_test[2..].to_owned())
            }
            _ => Self::ChildNamed(name_test.to_owned()),
        })
    }

    fn from_ancestor_name_test(name_test: &str) -> Option<Self> {
        match name_test {
            "*" => Some(Self::AncestorAnyElement),
            "node()" => Some(Self::AncestorAnyNode),
            "text()" => None,
            _ => Some(Self::AncestorNamed(name_test.to_owned())),
        }
    }

    fn from_ancestor_or_self_name_test(name_test: &str) -> Option<Self> {
        match name_test {
            "*" => Some(Self::AncestorOrSelfAnyElement),
            "node()" => Some(Self::AncestorOrSelfAnyNode),
            "text()" => None,
            _ => Some(Self::AncestorOrSelfNamed(name_test.to_owned())),
        }
    }

    fn from_self_name_test(name_test: &str) -> Self {
        match name_test {
            "*" => Self::SelfAnyElement,
            "node()" => Self::SelfAnyNode,
            "text()" => Self::SelfText,
            "comment()" => Self::SelfComment,
            "processing-instruction()" => Self::SelfProcessingInstruction,
            _ => Self::SelfNamed(name_test.to_owned()),
        }
    }

    fn from_preceding_name_test(name_test: &str) -> Self {
        match name_test {
            "*" => Self::PrecedingAnyElement,
            "node()" => Self::PrecedingAnyNode,
            "text()" => Self::PrecedingText,
            "comment()" => Self::PrecedingComment,
            "processing-instruction()" => Self::PrecedingProcessingInstruction,
            _ => Self::PrecedingNamed(name_test.to_owned()),
        }
    }

    fn uses_attribute_axis(&self) -> bool {
        matches!(
            self,
            Self::AttributeNamed(_) | Self::AttributeExpandedName(_) | Self::AttributeAny
        )
    }

    fn uses_parent_axis(&self) -> bool {
        matches!(
            self,
            Self::ParentNamed(_) | Self::ParentAnyElement | Self::ParentAnyNode
        )
    }

    fn uses_ancestor_axis(&self) -> bool {
        matches!(
            self,
            Self::AncestorNamed(_) | Self::AncestorAnyElement | Self::AncestorAnyNode
        )
    }

    fn uses_ancestor_or_self_axis(&self) -> bool {
        matches!(
            self,
            Self::AncestorOrSelfNamed(_)
                | Self::AncestorOrSelfAnyElement
                | Self::AncestorOrSelfAnyNode
        )
    }

    fn uses_self_axis(&self) -> bool {
        matches!(
            self,
            Self::SelfNamed(_)
                | Self::SelfAnyElement
                | Self::SelfAnyNode
                | Self::SelfText
                | Self::SelfComment
                | Self::SelfProcessingInstruction
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

    fn uses_following_sibling_axis(&self) -> bool {
        matches!(
            self,
            Self::FollowingSiblingNamed(_)
                | Self::FollowingSiblingAnyElement
                | Self::FollowingSiblingAnyNode
        )
    }

    fn uses_following_axis(&self) -> bool {
        matches!(
            self,
            Self::FollowingNamed(_)
                | Self::FollowingAnyElement
                | Self::FollowingAnyNode
                | Self::FollowingText
                | Self::FollowingComment
                | Self::FollowingProcessingInstruction
        )
    }

    fn uses_preceding_axis(&self) -> bool {
        matches!(
            self,
            Self::PrecedingNamed(_)
                | Self::PrecedingAnyElement
                | Self::PrecedingAnyNode
                | Self::PrecedingText
                | Self::PrecedingComment
                | Self::PrecedingProcessingInstruction
        )
    }

    fn uses_preceding_sibling_axis(&self) -> bool {
        matches!(
            self,
            Self::PrecedingSiblingNamed(_)
                | Self::PrecedingSiblingAnyElement
                | Self::PrecedingSiblingAnyNode
        )
    }
}

impl PartialEq<&str> for PathStep {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Self::ChildNamed(value)
            | Self::ChildLocalName(value)
            | Self::AttributeNamed(value)
            | Self::AncestorNamed(value)
            | Self::AncestorOrSelfNamed(value)
            | Self::ParentNamed(value)
            | Self::SelfNamed(value)
            | Self::DescendantNamed(value)
            | Self::DescendantOrSelfNamed(value)
            | Self::FollowingNamed(value)
            | Self::FollowingSiblingNamed(value)
            | Self::PrecedingNamed(value)
            | Self::PrecedingSiblingNamed(value) => value == *other,
            Self::ChildExpandedName(value) | Self::AttributeExpandedName(value) => {
                value.namespace.is_none() && value.local == *other
            }
            Self::ChildAnyElement
            | Self::AncestorAnyElement
            | Self::AncestorOrSelfAnyElement
            | Self::ParentAnyElement
            | Self::SelfAnyElement
            | Self::DescendantAnyElement
            | Self::DescendantOrSelfAnyElement
            | Self::FollowingAnyElement
            | Self::FollowingSiblingAnyElement
            | Self::PrecedingAnyElement
            | Self::PrecedingSiblingAnyElement => *other == "*",
            Self::ChildAnyNode
            | Self::AncestorAnyNode
            | Self::AncestorOrSelfAnyNode
            | Self::ParentAnyNode
            | Self::SelfAnyNode
            | Self::DescendantAnyNode
            | Self::DescendantOrSelfAnyNode
            | Self::FollowingAnyNode
            | Self::FollowingSiblingAnyNode
            | Self::PrecedingAnyNode
            | Self::PrecedingSiblingAnyNode => *other == "node()",
            Self::ChildText | Self::SelfText | Self::FollowingText | Self::PrecedingText => {
                *other == "text()"
            }
            Self::ChildComment
            | Self::SelfComment
            | Self::FollowingComment
            | Self::PrecedingComment => *other == "comment()",
            Self::ChildProcessingInstruction
            | Self::SelfProcessingInstruction
            | Self::FollowingProcessingInstruction
            | Self::PrecedingProcessingInstruction => *other == "processing-instruction()",
            Self::ChildProcessingInstructionNamed(target) => {
                processing_instruction_target(other).is_some_and(|other| other == target)
            }
            Self::AttributeAny => matches!(*other, "*" | "node()"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PredicateAxis {
    Child,
    Attribute,
    MissingAttribute,
    Ancestor,
    AncestorOrSelf,
    DescendantOrSelf,
    Parent,
    ContextLanguage,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AxisPredicate {
    axis: PredicateAxis,
    name: String,
    value: Option<String>,
    position: Option<PositionPredicate>,
    conjunct: Option<Box<AxisPredicate>>,
}

impl AxisPredicate {
    fn known_owned_capacity_bytes(&self) -> usize {
        self.name.capacity()
            + self.value.as_ref().map_or(0, String::capacity)
            + self
                .conjunct
                .as_ref()
                .map_or(0, |predicate| predicate.known_owned_capacity_bytes())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositionPredicate {
    Select(usize),
    Compare {
        operator: PositionComparison,
        value: i64,
    },
    Last,
    LastMinus(usize),
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StepPredicate {
    Position(PositionPredicate),
    ContextName(Box<str>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositionComparison {
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

struct ParsedPathSteps {
    steps: Vec<String>,
    position_predicates: Vec<Vec<StepPredicate>>,
    axis_predicates: Vec<Option<AxisPredicate>>,
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
    let normalized_filter = unwrap_parenthesized_reverse_axis_filter(expression);
    let first_step_predicates_use_document_order = normalized_filter.is_some();
    let expression = normalized_filter.as_deref().unwrap_or(expression);
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
        Some(ParsedPathSteps {
            steps: expression.split('/').map(str::to_owned).collect(),
            position_predicates: vec![Vec::new(); expression.split('/').count()],
            axis_predicates: vec![None; expression.split('/').count()],
        })
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

    let ParsedPathSteps {
        mut steps,
        position_predicates: step_position_predicates,
        axis_predicates: step_axis_predicates,
    } = parsed_steps.expect("checked above");
    for step in &mut steps {
        normalize_axis_separator_whitespace(step);
    }
    validate_unambiguous_step_syntax(&steps, expression, &location)?;
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
        first_step_predicates_use_document_order,
        step_axis_predicates,
        step_position_predicates,
        location,
    })
}

fn normalize_axis_separator_whitespace(step: &mut String) {
    let Some((axis, node_test)) = step.split_once("::") else {
        return;
    };
    let axis = axis.trim_end_matches(is_xpath_whitespace);
    let node_test = node_test.trim_start_matches(is_xpath_whitespace);
    if axis.len() + 2 + node_test.len() != step.len() {
        *step = format!("{axis}::{node_test}");
    }
}

fn unwrap_parenthesized_reverse_axis_filter(expression: &str) -> Option<String> {
    let expression = expression.strip_prefix('(')?;
    let close = expression.find(')')?;
    let inner = &expression[..close];
    let suffix = &expression[close + 1..];
    let supported_reverse_axis = inner.starts_with("ancestor::")
        || inner.starts_with("ancestor-or-self::")
        || inner.starts_with("preceding::")
        || inner.starts_with("preceding-sibling::");
    if !supported_reverse_axis
        || inner.contains(['/', '[', ']', '(', ')'])
        || !suffix.starts_with('[')
    {
        return None;
    }
    Some(format!("{inner}{suffix}"))
}

fn is_xpath_whitespace(character: char) -> bool {
    matches!(character, '\u{9}' | '\u{A}' | '\u{D}' | ' ')
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
        let (attribute, name_test) = step
            .strip_prefix('@')
            .map_or((false, step), |name| (true, name));
        let Some((prefix, local)) = name_test.split_once(':') else {
            if !is_ascii_ncname(name_test) {
                return Err(invalid_syntax(
                    format!("the qualified path contains an invalid name test: {expression}"),
                    &location,
                ));
            }
            steps.push(if attribute {
                PathStep::AttributeNamed(name_test.to_owned())
            } else {
                PathStep::ChildNamed(name_test.to_owned())
            });
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
        let name = ExpandedName {
            namespace: Some(namespace),
            local: local.to_owned(),
        };
        steps.push(if attribute {
            PathStep::AttributeExpandedName(name)
        } else {
            PathStep::ChildExpandedName(name)
        });
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
        first_step_predicates_use_document_order: false,
        step_axis_predicates: vec![None; step_count],
        step_position_predicates: vec![Vec::new(); step_count],
        location,
    })
}

fn has_unadmitted_name_test(step: &str) -> bool {
    let name_test = step
        .strip_prefix("child::")
        .or_else(|| step.strip_prefix("attribute::"))
        .or_else(|| step.strip_prefix("ancestor::"))
        .or_else(|| step.strip_prefix("ancestor-or-self::"))
        .or_else(|| step.strip_prefix("parent::"))
        .or_else(|| step.strip_prefix("self::"))
        .or_else(|| step.strip_prefix("descendant::"))
        .or_else(|| step.strip_prefix("descendant-or-self::"))
        .or_else(|| step.strip_prefix("following::"))
        .or_else(|| step.strip_prefix("following-sibling::"))
        .or_else(|| step.strip_prefix("preceding::"))
        .or_else(|| step.strip_prefix("preceding-sibling::"))
        .or_else(|| step.strip_prefix('@'))
        .unwrap_or(if matches!(step, "." | "..") {
            "node()"
        } else {
            step
        });
    let admitted_axis_kind = (!step.contains("::")
        || step.starts_with("child::")
        || step.starts_with("following::")
        || step.starts_with("preceding::")
        || step.starts_with("self::"))
        && matches!(
            name_test,
            "element()" | "comment()" | "processing-instruction()"
        );
    let admitted_named_processing_instruction = (!step.contains("::")
        || step.starts_with("child::"))
        && processing_instruction_target(name_test).is_some();
    let admitted_local_wildcard = (!step.contains("::") || step.starts_with("child::"))
        && name_test.strip_prefix("*:").is_some_and(is_ascii_ncname);
    !matches!(name_test, "*" | "node()" | "text()")
        && !admitted_axis_kind
        && !admitted_named_processing_instruction
        && !admitted_local_wildcard
        && !is_ascii_ncname(name_test)
}

fn processing_instruction_target(name_test: &str) -> Option<&str> {
    let argument = name_test
        .strip_prefix("processing-instruction(")?
        .strip_suffix(')')?;
    for delimiter in ['\'', '"'] {
        let target = argument
            .strip_prefix(delimiter)
            .and_then(|value| value.strip_suffix(delimiter));
        if let Some(target) = target.filter(|value| *value == "*" || is_ascii_ncname(value)) {
            return Some(target);
        }
    }
    None
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
        first_step_predicates_use_document_order: false,
        step_axis_predicates: Vec::new(),
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

fn parse_final_axis_predicate(expression: &str) -> (&str, Option<AxisPredicate>) {
    let Some((path, predicate)) = expression.split_once('[') else {
        return (expression, None);
    };
    let Some(predicate) = predicate.strip_suffix(']') else {
        return (expression, None);
    };
    let Some(predicate) = parse_axis_predicate(predicate) else {
        return (expression, None);
    };
    if path.is_empty() || path.contains('[') {
        return (expression, None);
    }
    (path, Some(predicate))
}

fn parse_axis_predicate(predicate: &str) -> Option<AxisPredicate> {
    if let Some((left, right)) = split_top_level_predicate_and(predicate) {
        let mut left = parse_axis_predicate(left.trim())?;
        if let Some(position) = parse_position_predicate(right.trim()) {
            left.position = Some(position);
        } else {
            left.conjunct = Some(Box::new(parse_axis_predicate(right.trim())?));
        }
        return Some(left);
    }
    if let Some(language) = language_experiment::parse_literal(predicate) {
        return Some(AxisPredicate {
            axis: PredicateAxis::ContextLanguage,
            name: language,
            value: None,
            position: None,
            conjunct: None,
        });
    }
    let (predicate, value) = parse_attribute_value_predicate(predicate)
        .map_or((predicate, None), |(name, value)| (name, Some(value)));
    let (axis, name) = if value.is_some() {
        (PredicateAxis::Attribute, predicate)
    } else if let Some(name) = predicate
        .strip_prefix("not(@")
        .and_then(|value| value.strip_suffix(')'))
    {
        (PredicateAxis::MissingAttribute, name)
    } else if let Some(name) = predicate.strip_prefix("child::") {
        (PredicateAxis::Child, name)
    } else if let Some(name) = predicate
        .strip_prefix("attribute::")
        .or_else(|| predicate.strip_prefix('@'))
    {
        (PredicateAxis::Attribute, name)
    } else if let Some(name) = predicate.strip_prefix("ancestor::") {
        (PredicateAxis::Ancestor, name)
    } else if let Some(name) = predicate.strip_prefix("ancestor-or-self::") {
        (PredicateAxis::AncestorOrSelf, name)
    } else if let Some(name) = predicate.strip_prefix("descendant-or-self::") {
        (PredicateAxis::DescendantOrSelf, name)
    } else if let Some(name) = predicate.strip_prefix("parent::") {
        (PredicateAxis::Parent, name)
    } else if is_ascii_ncname(predicate) {
        (PredicateAxis::Child, predicate)
    } else {
        return None;
    };
    if !is_ascii_ncname(name) {
        return None;
    }
    Some(AxisPredicate {
        axis,
        name: name.to_owned(),
        value,
        position: None,
        conjunct: None,
    })
}

fn split_top_level_predicate_and(predicate: &str) -> Option<(&str, &str)> {
    let bytes = predicate.as_bytes();
    let mut quote = None;
    let mut depth = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' | b'"' if quote == Some(bytes[index]) => quote = None,
            b'\'' | b'"' if quote.is_none() => quote = Some(bytes[index]),
            b'(' if quote.is_none() => depth += 1,
            b')' if quote.is_none() => depth = depth.checked_sub(1)?,
            _ if quote.is_none() && depth == 0 && bytes[index..].starts_with(b" and ") => {
                return Some((&predicate[..index], &predicate[index + 5..]));
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn parse_attribute_value_predicate(predicate: &str) -> Option<(&str, String)> {
    let (name, value) = predicate.split_once('=')?;
    let name = name.trim();
    let name = name
        .strip_prefix("attribute::")
        .or_else(|| name.strip_prefix('@'))?;
    if !is_ascii_ncname(name) {
        return None;
    }
    let value = value.trim();
    for delimiter in ['\'', '"'] {
        if let Some(value) = value
            .strip_prefix(delimiter)
            .and_then(|value| value.strip_suffix(delimiter))
            .filter(|value| !value.contains(delimiter))
        {
            return Some((name, value.to_owned()));
        }
    }
    None
}

fn parse_position_steps(expression: &str) -> Option<ParsedPathSteps> {
    let raw_steps = split_path_steps(expression)?;
    let mut steps = Vec::with_capacity(raw_steps.len());
    let mut position_predicates = Vec::with_capacity(raw_steps.len());
    let mut axis_predicates = Vec::with_capacity(raw_steps.len());
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
            position_predicates.push(Vec::new());
            axis_predicates.push(None);
            continue;
        }
        let (name, predicate_texts) = split_step_predicates(raw_step)?;
        let mut axis_predicate = None;
        let mut position_predicate = Vec::new();
        for (predicate_index, predicate) in predicate_texts.into_iter().enumerate() {
            if let Some(position) = parse_position_predicate(predicate) {
                position_predicate.push(StepPredicate::Position(position));
            } else if predicate_index == 0 {
                axis_predicate = Some(parse_axis_predicate(predicate)?);
            } else if !position_predicate.is_empty()
                && !position_predicate
                    .iter()
                    .any(|item| matches!(item, StepPredicate::ContextName(_)))
            {
                position_predicate.push(StepPredicate::ContextName(
                    parse_context_name_predicate(predicate)?.into_boxed_str(),
                ));
            } else {
                return None;
            }
        }
        steps.push(name.to_owned());
        position_predicates.push(position_predicate);
        axis_predicates.push(axis_predicate);
    }
    Some(ParsedPathSteps {
        steps,
        position_predicates,
        axis_predicates,
    })
}

fn parse_context_name_predicate(predicate: &str) -> Option<String> {
    let value = predicate.strip_prefix("name()")?.trim_start();
    let value = value.strip_prefix('=')?.trim();
    for delimiter in ['\'', '"'] {
        if let Some(value) = value
            .strip_prefix(delimiter)
            .and_then(|value| value.strip_suffix(delimiter))
            .filter(|value| !value.contains(delimiter))
        {
            return Some(value.to_owned());
        }
    }
    None
}

fn split_step_predicates(step: &str) -> Option<(&str, Vec<&str>)> {
    let Some(first_open) = step.find('[') else {
        return Some((step, Vec::new()));
    };
    let name = &step[..first_open];
    if name.is_empty() {
        return None;
    }
    let mut predicates = Vec::new();
    let mut remaining = &step[first_open..];
    while let Some(body) = remaining.strip_prefix('[') {
        let close = body.find(']')?;
        let predicate = &body[..close];
        if predicate.is_empty() || predicate.contains(['[', ']']) {
            return None;
        }
        predicates.push(predicate);
        remaining = &body[close + 1..];
    }
    remaining.is_empty().then_some((name, predicates))
}

fn parse_position_predicate(predicate: &str) -> Option<PositionPredicate> {
    let predicate = predicate.trim();
    if matches!(
        predicate,
        "last()" | "last()=position()" | "position()=last()"
    ) {
        return Some(PositionPredicate::Last);
    }
    if let Some(operand) = predicate.strip_prefix("last()-") {
        let value = constant_integer_experiment::evaluate(operand.trim()).ok()?;
        return Some(
            usize::try_from(value)
                .ok()
                .map_or(PositionPredicate::Never, PositionPredicate::LastMinus),
        );
    }
    if let Some(remainder) = predicate.strip_prefix("position()") {
        let remainder = remainder.trim_start();
        for (token, operator) in [
            ("!=", PositionComparison::NotEqual),
            (">=", PositionComparison::GreaterThanOrEqual),
            ("<=", PositionComparison::LessThanOrEqual),
            (">", PositionComparison::GreaterThan),
            ("<", PositionComparison::LessThan),
        ] {
            if let Some(operand) = remainder.strip_prefix(token) {
                let value = constant_integer_experiment::evaluate(operand.trim()).ok()?;
                return Some(PositionPredicate::Compare { operator, value });
            }
        }
        if let Some(operand) = remainder.strip_prefix('=') {
            let value = constant_integer_experiment::evaluate(operand.trim()).ok()?;
            return Some(
                usize::try_from(value)
                    .ok()
                    .filter(|position| *position > 0)
                    .map_or(PositionPredicate::Never, PositionPredicate::Select),
            );
        }
        return None;
    }
    let folded_number = constant_numeric_experiment::fold_number_conversion(predicate);
    let numeric_expression = folded_number.as_deref().unwrap_or(predicate);
    let value = constant_integer_experiment::evaluate(numeric_expression).ok()?;
    Some(
        usize::try_from(value)
            .ok()
            .filter(|position| *position > 0)
            .map_or(PositionPredicate::Never, PositionPredicate::Select),
    )
}

fn position_predicate_matches(
    predicate: Option<&PositionPredicate>,
    position: usize,
    size: usize,
) -> bool {
    match predicate {
        Some(PositionPredicate::Select(selected)) => *selected == position,
        Some(PositionPredicate::Compare { operator, value }) => {
            compare_position(position, *operator, *value)
        }
        Some(PositionPredicate::Last) => position == size,
        Some(PositionPredicate::LastMinus(offset)) => size.checked_sub(*offset) == Some(position),
        Some(PositionPredicate::Never) => false,
        None => true,
    }
}

fn compare_position(position: usize, operator: PositionComparison, value: i64) -> bool {
    let Ok(value) = usize::try_from(value) else {
        return matches!(
            operator,
            PositionComparison::NotEqual
                | PositionComparison::GreaterThan
                | PositionComparison::GreaterThanOrEqual
        );
    };
    match operator {
        PositionComparison::NotEqual => position != value,
        PositionComparison::LessThan => position < value,
        PositionComparison::LessThanOrEqual => position <= value,
        PositionComparison::GreaterThan => position > value,
        PositionComparison::GreaterThanOrEqual => position >= value,
    }
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
    let mut current = initial_path_nodes(document, context, path.origin, control)?;
    for (step_index, step) in path.steps.iter().enumerate() {
        let mut next = Vec::new();
        for node in current {
            let candidates = step_candidates(document, node, step, control)?;
            let mut named_candidates = Vec::new();
            for child in candidates {
                if !step.uses_descendant_axis()
                    && !step.uses_descendant_or_self_axis()
                    && !step.uses_following_axis()
                    && !step.uses_preceding_axis()
                {
                    control.charge(WorkDomain::XPathNodeVisit, 1)?;
                }
                if step_matches_candidate(document, child, step) {
                    named_candidates.push(child);
                }
            }
            if step_index == 0 && path.first_step_predicates_use_document_order {
                named_candidates.sort_unstable_by_key(|node| document.document_order(*node));
                named_candidates.dedup();
            }
            let mut predicate_candidates = Vec::with_capacity(named_candidates.len());
            let named_count = named_candidates.len();
            for (offset, child) in named_candidates.into_iter().enumerate() {
                let matches = evaluate_optional_axis_predicate(
                    document,
                    child,
                    path.step_axis_predicates[step_index].as_ref(),
                    offset + 1,
                    named_count,
                    control,
                )?;
                if matches {
                    predicate_candidates.push(child);
                }
            }
            let predicate_candidates = apply_position_predicates(
                document,
                predicate_candidates,
                &path.step_position_predicates[step_index],
            );
            let matching_count = predicate_candidates.len();
            for (offset, child) in predicate_candidates.into_iter().enumerate() {
                let final_predicate = (step_index + 1 == path.steps.len())
                    .then_some(path.final_predicate.as_ref())
                    .flatten();
                let axis_predicate_matches = evaluate_optional_axis_predicate(
                    document,
                    child,
                    final_predicate,
                    offset + 1,
                    matching_count,
                    control,
                )?;
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
                if axis_predicate_matches && context_predicate_matches {
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

pub(crate) fn evaluate_location_path_union_controlled(
    document: &Document,
    context: NodeId,
    alternatives: &[LocationPath],
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    let mut selected = Vec::new();
    for alternative in alternatives {
        selected.extend(evaluate_location_path_controlled(
            document,
            context,
            alternative,
            control,
        )?);
    }
    selected.sort_unstable_by_key(|node| document.document_order(*node));
    selected.dedup();
    Ok(selected)
}

fn apply_position_predicates(
    document: &Document,
    mut candidates: Vec<NodeId>,
    predicates: &[StepPredicate],
) -> Vec<NodeId> {
    for predicate in predicates {
        if let StepPredicate::ContextName(required) = predicate {
            let (required_prefix, required_local) = required
                .split_once(':')
                .map_or((None, required.as_ref()), |(prefix, local)| {
                    (Some(prefix), local)
                });
            candidates.retain(|node| {
                document.name(*node).is_some_and(|name| {
                    name.local == required_local && document.prefix(*node) == required_prefix
                })
            });
            continue;
        }
        let StepPredicate::Position(predicate) = predicate else {
            unreachable!("context-name predicate returned above")
        };
        let size = candidates.len();
        candidates = candidates
            .into_iter()
            .enumerate()
            .filter_map(|(offset, node)| {
                position_predicate_matches(Some(predicate), offset + 1, size).then_some(node)
            })
            .collect();
    }
    candidates
}

fn initial_path_nodes(
    document: &Document,
    context: NodeId,
    origin: PathOrigin,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    match origin {
        PathOrigin::EmptySequence => Ok(Vec::new()),
        PathOrigin::DocumentNode => Ok(vec![document.document_node()]),
        PathOrigin::Descendant => {
            descendant_or_self_nodes(document, document.document_node(), control)
        }
        PathOrigin::ContextDescendant => descendant_or_self_nodes(document, context, control),
        PathOrigin::ContextItem | PathOrigin::Relative => Ok(vec![context]),
    }
}

fn evaluate_optional_axis_predicate(
    document: &Document,
    node: NodeId,
    predicate: Option<&AxisPredicate>,
    context_position: usize,
    context_size: usize,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    predicate.map_or(Ok(true), |predicate| {
        evaluate_axis_predicate(
            document,
            node,
            predicate,
            context_position,
            context_size,
            control,
        )
    })
}

fn evaluate_axis_predicate(
    document: &Document,
    node: NodeId,
    predicate: &AxisPredicate,
    context_position: usize,
    context_size: usize,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let matches = match predicate.axis {
        PredicateAxis::Child => has_named_child(document, node, &predicate.name, control),
        PredicateAxis::Attribute => has_named_attribute(
            document,
            node,
            &predicate.name,
            predicate.value.as_deref(),
            control,
        ),
        PredicateAxis::MissingAttribute => {
            has_named_attribute(document, node, &predicate.name, None, control).map(|found| !found)
        }
        PredicateAxis::Ancestor => {
            has_named_ancestor(document, node, &predicate.name, false, control)
        }
        PredicateAxis::AncestorOrSelf => {
            has_named_ancestor(document, node, &predicate.name, true, control)
        }
        PredicateAxis::DescendantOrSelf => {
            has_named_descendant_or_self(document, node, &predicate.name, control)
        }
        PredicateAxis::Parent => has_named_parent(document, node, &predicate.name, control),
        PredicateAxis::ContextLanguage => {
            language_experiment::evaluate(document, node, &predicate.name, control)
        }
    }?;
    if !matches {
        return Ok(false);
    }
    let position_matches =
        position_predicate_matches(predicate.position.as_ref(), context_position, context_size);
    if !position_matches {
        return Ok(false);
    }
    predicate.conjunct.as_ref().map_or(Ok(true), |conjunct| {
        evaluate_axis_predicate(
            document,
            node,
            conjunct,
            context_position,
            context_size,
            control,
        )
    })
}

fn step_candidates(
    document: &Document,
    node: NodeId,
    step: &PathStep,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    if step.uses_attribute_axis() {
        Ok(document.attributes(node).to_vec())
    } else if step.uses_ancestor_axis() {
        Ok(ancestor_nodes(document, node, false))
    } else if step.uses_ancestor_or_self_axis() {
        Ok(ancestor_nodes(document, node, true))
    } else if step.uses_parent_axis() {
        Ok(document.parent(node).into_iter().collect())
    } else if step.uses_self_axis() {
        Ok(vec![node])
    } else if step.uses_descendant_axis() {
        descendant_nodes(document, node, control)
    } else if step.uses_descendant_or_self_axis() {
        descendant_or_self_nodes(document, node, control)
    } else if step.uses_following_axis() {
        following_nodes(document, node, control)
    } else if step.uses_following_sibling_axis() {
        Ok(following_siblings(document, node))
    } else if step.uses_preceding_axis() {
        preceding_nodes(document, node, control)
    } else if step.uses_preceding_sibling_axis() {
        Ok(preceding_siblings(document, node))
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
        PathStep::ChildLocalName(required) => {
            document.kind(child) == NodeKind::Element
                && document
                    .name(child)
                    .is_some_and(|name| name.local == required.as_str())
        }
        PathStep::ChildExpandedName(required) => {
            document.kind(child) == NodeKind::Element && document.name(child) == Some(required)
        }
        PathStep::ChildAnyElement
        | PathStep::AncestorAnyElement
        | PathStep::AncestorOrSelfAnyElement
        | PathStep::ParentAnyElement
        | PathStep::SelfAnyElement
        | PathStep::DescendantAnyElement
        | PathStep::DescendantOrSelfAnyElement
        | PathStep::FollowingAnyElement
        | PathStep::FollowingSiblingAnyElement
        | PathStep::PrecedingAnyElement
        | PathStep::PrecedingSiblingAnyElement => document.kind(child) == NodeKind::Element,
        PathStep::ChildAnyNode
        | PathStep::AncestorAnyNode
        | PathStep::AncestorOrSelfAnyNode
        | PathStep::ParentAnyNode
        | PathStep::SelfAnyNode
        | PathStep::DescendantAnyNode
        | PathStep::DescendantOrSelfAnyNode
        | PathStep::FollowingAnyNode
        | PathStep::FollowingSiblingAnyNode
        | PathStep::PrecedingAnyNode
        | PathStep::PrecedingSiblingAnyNode => true,
        PathStep::ChildText
        | PathStep::SelfText
        | PathStep::FollowingText
        | PathStep::PrecedingText => document.kind(child) == NodeKind::Text,
        PathStep::ChildComment
        | PathStep::SelfComment
        | PathStep::FollowingComment
        | PathStep::PrecedingComment => document.kind(child) == NodeKind::Comment,
        PathStep::ChildProcessingInstruction
        | PathStep::SelfProcessingInstruction
        | PathStep::FollowingProcessingInstruction
        | PathStep::PrecedingProcessingInstruction => {
            document.kind(child) == NodeKind::ProcessingInstruction
        }
        PathStep::ChildProcessingInstructionNamed(required) => {
            document.kind(child) == NodeKind::ProcessingInstruction
                && document
                    .name(child)
                    .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())
        }
        PathStep::AttributeNamed(required) => {
            document.kind(child) == NodeKind::Attribute
                && document
                    .name(child)
                    .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())
        }
        PathStep::AttributeExpandedName(required) => {
            document.kind(child) == NodeKind::Attribute && document.name(child) == Some(required)
        }
        PathStep::AttributeAny => document.kind(child) == NodeKind::Attribute,
        PathStep::AncestorNamed(required) | PathStep::AncestorOrSelfNamed(required) => {
            document.kind(child) == NodeKind::Element
                && document
                    .name(child)
                    .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())
        }
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
        PathStep::DescendantNamed(required)
        | PathStep::DescendantOrSelfNamed(required)
        | PathStep::FollowingNamed(required)
        | PathStep::FollowingSiblingNamed(required)
        | PathStep::PrecedingNamed(required)
        | PathStep::PrecedingSiblingNamed(required) => {
            document.kind(child) == NodeKind::Element
                && document
                    .name(child)
                    .is_some_and(|name| name.namespace.is_none() && name.local == required.as_str())
        }
    }
}

fn following_siblings(document: &Document, context: NodeId) -> Vec<NodeId> {
    let Some(parent) = document.parent(context) else {
        return Vec::new();
    };
    document
        .children(parent)
        .iter()
        .copied()
        .skip_while(|candidate| *candidate != context)
        .skip(1)
        .collect()
}

fn ancestor_nodes(document: &Document, context: NodeId, include_self: bool) -> Vec<NodeId> {
    let mut ancestors = Vec::new();
    let mut current = if include_self {
        Some(context)
    } else {
        document.parent(context)
    };
    while let Some(node) = current {
        ancestors.push(node);
        current = document.parent(node);
    }
    ancestors
}

fn following_nodes(
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    let descendants: HashSet<_> = descendant_nodes(document, context, control)?
        .into_iter()
        .collect();
    let context_order = document.document_order(context);
    let mut following = descendant_nodes(document, document.document_node(), control)?;
    following.retain(|candidate| {
        document.document_order(*candidate) > context_order && !descendants.contains(candidate)
    });
    Ok(following)
}

fn preceding_siblings(document: &Document, context: NodeId) -> Vec<NodeId> {
    let Some(parent) = document.parent(context) else {
        return Vec::new();
    };
    let siblings = document.children(parent);
    let Some(position) = siblings.iter().position(|candidate| *candidate == context) else {
        return Vec::new();
    };
    siblings[..position].iter().rev().copied().collect()
}

fn preceding_nodes(
    document: &Document,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    let mut ancestors = HashSet::new();
    let mut current = document.parent(context);
    while let Some(ancestor) = current {
        ancestors.insert(ancestor);
        current = document.parent(ancestor);
    }
    let context_order = document.document_order(context);
    let mut preceding = descendant_nodes(document, document.document_node(), control)?;
    preceding.retain(|candidate| {
        document.document_order(*candidate) < context_order && !ancestors.contains(candidate)
    });
    preceding.reverse();
    Ok(preceding)
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
    required_value: Option<&str>,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    for attribute in document.attributes(node).iter().copied() {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        if document.kind(attribute) == NodeKind::Attribute
            && document
                .name(attribute)
                .is_some_and(|name| name.namespace.is_none() && name.local == required)
            && required_value.is_none_or(|required| document.value(attribute) == Some(required))
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
