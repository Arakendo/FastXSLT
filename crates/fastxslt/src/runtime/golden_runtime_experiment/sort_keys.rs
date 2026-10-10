//! Charged sort-key evaluation, controls, collation, and stable comparison.

use super::{
    Document, ExecutionFailure, FailureCategory, InvocationControl, LocationPath, NodeId, NodeKind,
    RuntimeVariables, SequenceFocus, SequenceInputs, SortCaseOrder, SortDataType, SortKey,
    SortLanguage, SortOrder, SortSelect, StringValueVisitFailure, WorkDomain, control_failure,
    evaluate_location_path_controlled, evaluate_source_path_union, failure_at, key_lookup,
    value_evaluator,
};
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

#[derive(Debug)]
pub(super) enum EvaluatedSortKey {
    Text(String),
    Number(Option<f64>),
}

#[derive(Debug, Clone, Copy)]
pub(super) struct EvaluatedSortControl {
    pub(super) data_type: EvaluatedSortDataType,
    order: EvaluatedSortOrder,
    case_order: EvaluatedSortCaseOrder,
    language: EvaluatedSortLanguage,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum EvaluatedSortDataType {
    Text,
    Number,
}

#[derive(Debug, Clone, Copy)]
enum EvaluatedSortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Clone, Copy)]
enum EvaluatedSortCaseOrder {
    Default,
    UpperFirst,
    LowerFirst,
}

#[derive(Debug, Clone, Copy)]
enum EvaluatedSortLanguage {
    Codepoint,
    English,
    Spanish,
    French,
    Swedish,
    Turkish,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct SortFocus<'a> {
    pub(super) source: &'a Document,
    pub(super) node: NodeId,
    pub(super) position: usize,
    pub(super) size: usize,
}

pub(super) fn sort_selected_nodes(
    inputs: &SequenceInputs<'_>,
    selected: Vec<NodeId>,
    sorts: &[SortKey],
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    if sorts.is_empty() {
        return Ok(selected);
    }
    let sort_controls = sorts
        .iter()
        .map(|sort| evaluate_sort_control(inputs, sort, variables, control))
        .collect::<Result<Vec<_>, ExecutionFailure>>()?;
    if selected.len() < 2 {
        return Ok(selected);
    }
    let source = inputs
        .source
        .expect("source-node sorting requires a source document");
    let mut keyed = Vec::with_capacity(selected.len());
    let focus_size = selected.len();
    for (offset, node) in selected.into_iter().enumerate() {
        let mut values = Vec::with_capacity(sorts.len());
        for (sort, sort_control) in sorts.iter().zip(&sort_controls) {
            let value = evaluate_sort_key_value(
                inputs,
                SortFocus {
                    source,
                    node,
                    position: offset + 1,
                    size: focus_size,
                },
                sort,
                variables,
                control,
            )?;
            values.push(typed_sort_key(
                sort_control.data_type,
                sort.xslt10_numeric_conversion,
                value,
            ));
        }
        keyed.push((node, values));
    }
    sort_keyed_items(inputs.request_id, keyed, &sort_controls, control)
}

pub(super) fn sort_keyed_items<T>(
    request_id: &str,
    mut keyed: Vec<(T, Vec<EvaluatedSortKey>)>,
    sort_controls: &[EvaluatedSortControl],
    control: &mut InvocationControl,
) -> Result<Vec<T>, ExecutionFailure> {
    let comparison_charge = keyed
        .len()
        .saturating_mul(keyed.len().ilog2() as usize + 1)
        .saturating_mul(sort_controls.len());
    control
        .charge(WorkDomain::XPathOperation, comparison_charge)
        .map_err(|failure| control_failure(failure, request_id))?;
    keyed.sort_by(|left, right| {
        for (index, sort_control) in sort_controls.iter().enumerate() {
            let ordering = compare_sort_keys(
                &left.1[index],
                &right.1[index],
                sort_control.case_order,
                sort_control.language,
            );
            let ordering = match sort_control.order {
                EvaluatedSortOrder::Ascending => ordering,
                EvaluatedSortOrder::Descending => ordering.reverse(),
            };
            if !ordering.is_eq() {
                return ordering;
            }
        }
        std::cmp::Ordering::Equal
    });
    Ok(keyed.into_iter().map(|(item, _)| item).collect())
}

fn evaluate_concat_sort_key(
    inputs: &SequenceInputs<'_>,
    focus: SortFocus<'_>,
    sort: &SortKey,
    expression: &crate::xslt::golden_semantics_experiment::Xslt10ConcatExpression,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    // A node-valued variable still carries principal-local IDs. Do not
    // reinterpret it against a secondary document merely to sort it.
    for dynamic in inputs.dynamic_documents.borrow().values() {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|error| control_failure(error, inputs.request_id))?;
        if focus.source.same_origin(&dynamic.document) {
            return Err(failure_at(
                "FXRT0007",
                FailureCategory::Unsupported,
                Some(inputs.request_id),
                sort.location.clone(),
                "concat sort keys on secondary documents remain unsupported",
            ));
        }
    }
    value_evaluator::evaluate_xslt10_concat_lexical(
        inputs,
        Some(focus.node),
        expression,
        variables,
        control,
    )
    .map_err(|mut failure| {
        if failure.location.is_none() {
            failure.location = Some(sort.location.clone());
        }
        failure
    })
}

pub(super) fn evaluate_sort_key_value(
    inputs: &SequenceInputs<'_>,
    focus: SortFocus<'_>,
    sort: &SortKey,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let source = focus.source;
    let node = focus.node;
    match &sort.select {
        SortSelect::Xslt10Concat(expression) => {
            evaluate_concat_sort_key(inputs, focus, sort, expression, variables, control)
        }
        SortSelect::LocationPath(path) => {
            let nodes = evaluate_location_path_controlled(source, node, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            selected_sort_string(source, nodes.first().copied(), inputs.request_id, control)
        }
        SortSelect::Xslt10BinaryNumeric(expression) => {
            evaluate_xslt10_binary_numeric_sort(inputs, focus, expression, variables, control)
        }
        SortSelect::Xslt10VariablePositionPath {
            path,
            variable,
            explicit_position_comparison,
        } => evaluate_xslt10_variable_position_sort(
            inputs,
            focus,
            path,
            variable,
            *explicit_position_comparison,
            variables,
            control,
        ),
        SortSelect::Xslt10ChildNameEqualsVariable { variable } => {
            evaluate_xslt10_child_name_variable_sort(
                inputs, source, node, variable, variables, control,
            )
        }
        SortSelect::Xslt10PathStringLiteralComparison {
            path,
            literal,
            equal,
        } => evaluate_xslt10_path_string_literal_sort(
            inputs, source, node, path, literal, *equal, control,
        ),
        SortSelect::Xslt10KeyLookup(lookup) => {
            let selected = key_lookup::select(inputs, lookup, Some(node), variables, control)?;
            selected_sort_string(
                source,
                selected.first().copied(),
                inputs.request_id,
                control,
            )
        }
        SortSelect::Xslt10PathSubstring(expression) => {
            evaluate_xslt10_substring_sort_key(inputs, source, node, expression, control)
        }
        SortSelect::Xslt10PathSubstringAfter { path, delimiter } => {
            evaluate_xslt10_substring_after_sort_key(inputs, source, node, path, delimiter, control)
        }
        SortSelect::PathUnion(alternatives) => {
            let selected = evaluate_source_path_union(inputs, source, node, alternatives, control)?;
            selected_sort_string(
                source,
                selected.first().copied(),
                inputs.request_id,
                control,
            )
        }
        SortSelect::Literal(value) => Ok(value.clone()),
        SortSelect::Variable(name) => sort_variable(inputs, name, variables, control),
        SortSelect::ContextPosition => Ok(focus.position.to_string()),
        SortSelect::ContextSize => Ok(focus.size.to_string()),
        SortSelect::ContextNodeName => {
            evaluate_sort_context_node_name(source, node, inputs.request_id, control)
        }
        SortSelect::ContextNodeLocalName => {
            evaluate_sort_context_node_local_name(source, node, inputs.request_id, control)
        }
        SortSelect::ContextStringLength => Ok(evaluate_sort_context_string_length(
            source,
            node,
            inputs.request_id,
            &sort.location,
            control,
        )?
        .to_string()),
        SortSelect::CountPath(path) => {
            let count = evaluate_location_path_controlled(source, node, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?
                .len();
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            Ok(count.to_string())
        }
        SortSelect::NumberPath(path) => evaluate_sort_number_path(
            source,
            node,
            path,
            sort.xslt10_numeric_conversion,
            inputs.request_id,
            control,
        ),
    }
}

fn selected_sort_string(
    source: &Document,
    selected: Option<NodeId>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    selected.map_or_else(
        || Ok(String::new()),
        |node| {
            source
                .string_value_controlled(node, control)
                .map_err(|failure| control_failure(failure, request_id))
        },
    )
}

fn evaluate_xslt10_binary_numeric_sort(
    inputs: &SequenceInputs<'_>,
    focus: SortFocus<'_>,
    expression: &crate::xpath::binary_numeric_experiment::BinaryNumericExpression,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    value_evaluator::evaluate_binary_numeric_value(
        inputs,
        Some(focus.node),
        Some(SequenceFocus {
            position: focus.position,
            size: focus.size,
        }),
        expression,
        variables,
        control,
    )
}

fn evaluate_xslt10_variable_position_sort(
    inputs: &SequenceInputs<'_>,
    focus: SortFocus<'_>,
    path: &crate::xpath::path_experiment::LocationPath,
    variable: &str,
    explicit_position_comparison: bool,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let nodes = evaluate_location_path_controlled(focus.source, focus.node, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    if !explicit_position_comparison && variables.temporary_tree(inputs.globals, variable).is_some()
    {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        return selected_sort_string(
            focus.source,
            nodes.first().copied(),
            inputs.request_id,
            control,
        );
    }
    let position = sort_variable(inputs, variable, variables, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let selected = crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&position)
        .filter(|position| position.is_finite() && *position >= 1.0 && position.fract() == 0.0)
        .and_then(|position| position.to_string().parse::<usize>().ok())
        .and_then(|position| nodes.get(position.saturating_sub(1)).copied());
    selected_sort_string(focus.source, selected, inputs.request_id, control)
}

fn evaluate_xslt10_path_string_literal_sort(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    node: NodeId,
    path: &crate::xpath::path_experiment::LocationPath,
    literal: &str,
    equal: bool,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let selected = evaluate_location_path_controlled(source, node, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    for selected in selected {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let value = source
            .string_value_controlled(selected, control)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if (value == literal) == equal {
            return Ok("true".to_owned());
        }
    }
    Ok("false".to_owned())
}

fn evaluate_sort_context_node_name(
    source: &Document,
    node: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    Ok(source.name(node).map_or_else(String::new, |name| {
        source.prefix(node).map_or_else(
            || name.local.clone(),
            |prefix| format!("{prefix}:{}", name.local),
        )
    }))
}

fn evaluate_sort_context_node_local_name(
    source: &Document,
    node: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    Ok(source
        .name(node)
        .map_or_else(String::new, |name| name.local.clone()))
}

fn evaluate_xslt10_substring_sort_key(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    node: NodeId,
    expression: &crate::xslt::golden_semantics_experiment::Xslt10PathSubstring,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    value_evaluator::evaluate_xslt10_path_substring_lexical(
        source,
        node,
        expression,
        inputs.request_id,
        control,
    )
}

fn evaluate_xslt10_substring_after_sort_key(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    node: NodeId,
    path: &LocationPath,
    delimiter: &str,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let nodes = evaluate_location_path_controlled(source, node, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let value = selected_sort_string(source, nodes.first().copied(), inputs.request_id, control)?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(value
        .split_once(delimiter)
        .map_or_else(String::new, |(_, suffix)| suffix.to_owned()))
}

fn evaluate_xslt10_child_name_variable_sort(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    node: NodeId,
    variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let expected = sort_variable(inputs, variable, variables, control)?;
    for child in source.children(node) {
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        if source.kind(*child) != NodeKind::Element {
            continue;
        }
        let matches = source.name(*child).is_some_and(|name| {
            source.prefix(*child).map_or_else(
                || name.local == expected,
                |prefix| format!("{prefix}:{}", name.local) == expected,
            )
        });
        if matches {
            return selected_sort_string(source, Some(*child), inputs.request_id, control);
        }
    }
    Ok(String::new())
}

pub(super) fn evaluate_sort_control(
    inputs: &SequenceInputs<'_>,
    sort: &SortKey,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<EvaluatedSortControl, ExecutionFailure> {
    let data_type = match &sort.data_type {
        SortDataType::Text => EvaluatedSortDataType::Text,
        SortDataType::Number => EvaluatedSortDataType::Number,
        SortDataType::Variable(variable) => {
            match sort_variable(inputs, variable, variables, control)?.as_str() {
                "text" => EvaluatedSortDataType::Text,
                "number" => EvaluatedSortDataType::Number,
                value => {
                    return Err(failure_at(
                        "FXST1044",
                        FailureCategory::Unsupported,
                        Some(inputs.request_id),
                        sort.location.clone(),
                        format!("unsupported dynamic xsl:sort data-type: {value}"),
                    ));
                }
            }
        }
    };
    let order = match &sort.order {
        SortOrder::Ascending => EvaluatedSortOrder::Ascending,
        SortOrder::Descending => EvaluatedSortOrder::Descending,
        SortOrder::Variable(variable) => {
            match sort_variable(inputs, variable, variables, control)?.as_str() {
                "ascending" => EvaluatedSortOrder::Ascending,
                "descending" => EvaluatedSortOrder::Descending,
                value => {
                    return Err(failure_at(
                        "XTDE0030",
                        FailureCategory::Invalid,
                        Some(inputs.request_id),
                        sort.location.clone(),
                        format!("invalid dynamic xsl:sort order: {value}"),
                    ));
                }
            }
        }
    };
    let case_order = match &sort.case_order {
        SortCaseOrder::Default => EvaluatedSortCaseOrder::Default,
        SortCaseOrder::UpperFirst => EvaluatedSortCaseOrder::UpperFirst,
        SortCaseOrder::LowerFirst => EvaluatedSortCaseOrder::LowerFirst,
        SortCaseOrder::Variable(variable) => {
            match sort_variable(inputs, variable, variables, control)?.as_str() {
                "upper-first" => EvaluatedSortCaseOrder::UpperFirst,
                "lower-first" => EvaluatedSortCaseOrder::LowerFirst,
                value => {
                    return Err(failure_at(
                        "XTDE0030",
                        FailureCategory::Invalid,
                        Some(inputs.request_id),
                        sort.location.clone(),
                        format!("invalid dynamic xsl:sort case-order: {value}"),
                    ));
                }
            }
        }
    };
    let language = match &sort.language {
        SortLanguage::Codepoint => EvaluatedSortLanguage::Codepoint,
        SortLanguage::English => EvaluatedSortLanguage::English,
        SortLanguage::Spanish => EvaluatedSortLanguage::Spanish,
        SortLanguage::French => EvaluatedSortLanguage::French,
        SortLanguage::Swedish => EvaluatedSortLanguage::Swedish,
        SortLanguage::Turkish => EvaluatedSortLanguage::Turkish,
        SortLanguage::Xslt10Concat(expression) => {
            let lexical = value_evaluator::evaluate_xslt10_concat(
                inputs, None, expression, variables, control,
            )?;
            evaluated_sort_language(&lexical, inputs, sort)?
        }
    };
    Ok(EvaluatedSortControl {
        data_type,
        order,
        case_order,
        language,
    })
}

fn evaluated_sort_language(
    lexical: &str,
    inputs: &SequenceInputs<'_>,
    sort: &SortKey,
) -> Result<EvaluatedSortLanguage, ExecutionFailure> {
    if !is_dynamic_sort_language_tag(lexical) {
        return Err(failure_at(
            "XTDE0030",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            sort.location.clone(),
            format!("invalid dynamic xsl:sort language hint: {lexical}"),
        ));
    }
    match lexical
        .split('-')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "en" => Ok(EvaluatedSortLanguage::English),
        "es" => Ok(EvaluatedSortLanguage::Spanish),
        "fr" => Ok(EvaluatedSortLanguage::French),
        "sv" => Ok(EvaluatedSortLanguage::Swedish),
        "tr" => Ok(EvaluatedSortLanguage::Turkish),
        _ => Err(failure_at(
            "FXST1063",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            sort.location.clone(),
            format!("unsupported dynamic xsl:sort language hint: {lexical}"),
        )),
    }
}

fn is_dynamic_sort_language_tag(value: &str) -> bool {
    let mut parts = value.split('-');
    let Some(primary) = parts.next() else {
        return false;
    };
    !primary.is_empty()
        && primary.len() <= 8
        && primary.bytes().all(|byte| byte.is_ascii_alphabetic())
        && parts.all(|part| {
            !part.is_empty()
                && part.len() <= 8
                && part.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
}

pub(super) fn typed_sort_key(
    data_type: EvaluatedSortDataType,
    xslt10_numeric_conversion: bool,
    value: String,
) -> EvaluatedSortKey {
    match data_type {
        EvaluatedSortDataType::Text => EvaluatedSortKey::Text(value),
        EvaluatedSortDataType::Number => EvaluatedSortKey::Number(if xslt10_numeric_conversion {
            crate::xpath::constant_boolean_experiment::parse_xpath_number_literal(&value)
        } else {
            value.trim().parse().ok()
        }),
    }
}

fn evaluate_sort_number_path(
    source: &Document,
    context: NodeId,
    path: &crate::xpath::path_experiment::LocationPath,
    xslt10_compatibility: bool,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    if !xslt10_compatibility && selected.len() > 1 {
        return Err(failure_at(
            "XPTY0004",
            FailureCategory::Invalid,
            Some(request_id),
            path.location.clone(),
            "fn:number requires a zero-or-one item argument",
        ));
    }
    let Some(node) = selected.first().copied() else {
        return Ok("NaN".to_owned());
    };
    let lexical = source
        .string_value_controlled(node, control)
        .map_err(|failure| control_failure(failure, request_id))?;
    control
        .charge(WorkDomain::XPathOperation, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    crate::xpath::constant_numeric_experiment::evaluate_number_lexical(&lexical).map_err(
        |numeric_failure| match numeric_failure {
            crate::xpath::constant_numeric_experiment::ConstantNumericFailure::Invalid => {
                failure_at(
                    "FORG0001",
                    FailureCategory::Invalid,
                    Some(request_id),
                    path.location.clone(),
                    format!("value is not a valid finite numeric lexical: {lexical}"),
                )
            }
            crate::xpath::constant_numeric_experiment::ConstantNumericFailure::Unsupported => {
                failure_at(
                    "FXXP1020",
                    FailureCategory::Unsupported,
                    Some(request_id),
                    path.location.clone(),
                    format!(
                        "numeric lexical is outside the admitted finite-decimal slice: {lexical}"
                    ),
                )
            }
        },
    )
}

fn evaluate_sort_context_string_length(
    source: &Document,
    node: NodeId,
    request_id: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    let mut length = 0_usize;
    source
        .visit_string_value_controlled(node, control, &mut |part, control| {
            for _ in part.chars() {
                control
                    .charge(WorkDomain::XPathOperation, 1)
                    .map_err(|failure| control_failure(failure, request_id))?;
                length = length.checked_add(1).ok_or_else(|| {
                    failure_at(
                        "FOAR0002",
                        FailureCategory::Invalid,
                        Some(request_id),
                        location.clone(),
                        "sort-key string length exceeds the supported integer range",
                    )
                })?;
            }
            Ok(())
        })
        .map_err(|failure| match failure {
            StringValueVisitFailure::Control(failure) => control_failure(failure, request_id),
            StringValueVisitFailure::Sink(failure) => failure,
        })?;
    Ok(length)
}

fn sort_variable(
    inputs: &SequenceInputs<'_>,
    variable: &str,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    value_evaluator::xslt10_variable_string_value(inputs, variable, variables, control)
}

fn compare_sort_keys(
    left: &EvaluatedSortKey,
    right: &EvaluatedSortKey,
    case_order: EvaluatedSortCaseOrder,
    language: EvaluatedSortLanguage,
) -> std::cmp::Ordering {
    match (left, right) {
        (EvaluatedSortKey::Text(left), EvaluatedSortKey::Text(right)) => {
            compare_text_sort_keys(left, right, case_order, language)
        }
        (EvaluatedSortKey::Number(left), EvaluatedSortKey::Number(right)) => match (left, right) {
            (Some(left), Some(right)) => left
                .partial_cmp(right)
                .expect("evaluated numeric sort keys exclude NaN"),
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
        },
        _ => unreachable!("a compiled sort key has one stable data type"),
    }
}

fn compare_text_sort_keys(
    left: &str,
    right: &str,
    case_order: EvaluatedSortCaseOrder,
    language: EvaluatedSortLanguage,
) -> std::cmp::Ordering {
    let primary =
        text_collation_primary(left, language).cmp(&text_collation_primary(right, language));
    if !primary.is_eq() {
        return primary;
    }
    let secondary =
        text_collation_secondary(left, language).cmp(&text_collation_secondary(right, language));
    if !secondary.is_eq() || matches!(case_order, EvaluatedSortCaseOrder::Default) {
        return secondary;
    }
    let left_case = text_case_signature(left, case_order);
    let right_case = text_case_signature(right, case_order);
    left_case.cmp(&right_case)
}

fn text_collation_primary(value: &str, language: EvaluatedSortLanguage) -> String {
    if matches!(language, EvaluatedSortLanguage::Codepoint) {
        return value.to_ascii_lowercase();
    }
    let mut primary = String::new();
    let normalized_whitespace = normalize_internal_collation_whitespace(value);
    for character in normalized_whitespace.chars() {
        match (language, character) {
            (EvaluatedSortLanguage::Swedish, 'å' | 'Å') => primary.push_str("{0"),
            (EvaluatedSortLanguage::Swedish, 'ä' | 'Ä') => primary.push_str("{1"),
            (EvaluatedSortLanguage::Swedish, 'ö' | 'Ö') => primary.push_str("{2"),
            (EvaluatedSortLanguage::Swedish, 'ü' | 'Ü') => primary.push_str("yz"),
            (EvaluatedSortLanguage::Turkish, 'ı' | 'I') | (_, 'ı') => primary.push_str("i0"),
            (EvaluatedSortLanguage::Turkish, 'i' | 'İ') | (_, 'İ') => primary.push_str("i1"),
            (_, 'æ' | 'Æ') => primary.push_str("ae"),
            (_, 'œ' | 'Œ') => primary.push_str("oe"),
            _ => {
                for decomposed in character.to_lowercase().flat_map(char::to_lowercase).nfd() {
                    if !is_combining_mark(decomposed) {
                        primary.push(decomposed);
                    }
                }
            }
        }
    }
    if matches!(language, EvaluatedSortLanguage::Spanish) {
        primary.replace("ch", "c{")
    } else {
        primary
    }
}

fn normalize_internal_collation_whitespace(value: &str) -> String {
    let characters = value.chars().collect::<Vec<_>>();
    let mut normalized = String::with_capacity(value.len());
    let mut index = 0;
    while index < characters.len() {
        if !characters[index].is_whitespace() {
            normalized.push(characters[index]);
            index += 1;
            continue;
        }
        let start = index;
        while index < characters.len() && characters[index].is_whitespace() {
            index += 1;
        }
        if start > 0 && index < characters.len() {
            normalized.push(' ');
        } else {
            normalized.extend(characters[start..index].iter());
        }
    }
    normalized
}

fn text_collation_secondary(value: &str, language: EvaluatedSortLanguage) -> Vec<u8> {
    if matches!(
        language,
        EvaluatedSortLanguage::Codepoint
            | EvaluatedSortLanguage::Swedish
            | EvaluatedSortLanguage::Turkish
    ) {
        return Vec::new();
    }
    let mut accents = Vec::with_capacity(value.chars().count());
    for character in value.chars() {
        if matches!(character, 'æ' | 'Æ' | 'œ' | 'Œ') {
            accents.extend([0, 0]);
        } else {
            accents.push(u8::from(character.nfd().skip(1).any(is_combining_mark)));
        }
    }
    if matches!(language, EvaluatedSortLanguage::French) {
        accents.reverse();
    }
    accents
}

fn text_case_signature(value: &str, case_order: EvaluatedSortCaseOrder) -> Vec<u8> {
    let mut signature = Vec::with_capacity(value.chars().count());
    for character in value.chars().filter(|character| character.is_alphabetic()) {
        let weight = match case_order {
            EvaluatedSortCaseOrder::UpperFirst => u8::from(!character.is_uppercase()),
            EvaluatedSortCaseOrder::LowerFirst => u8::from(character.is_uppercase()),
            EvaluatedSortCaseOrder::Default => 0,
        };
        signature.push(weight);
        if matches!(character, 'æ' | 'Æ' | 'œ' | 'Œ') {
            signature.push(weight);
        }
    }
    signature
}
