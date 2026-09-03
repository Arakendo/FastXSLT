//! Private XSLT 3.0 integer-focus template dispatch.

use std::collections::BTreeMap;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xslt::golden_semantics_experiment::{MatchPattern, MatchedTemplate};

use super::{
    ExecutionFailure, FailureCategory, InvocationParameter, ResultNode, SequenceContext,
    SequenceFocus, SequenceInputs, append_text, bind_template_parameters, execute_sequence,
};

pub(super) fn apply_integer_range(
    inputs: &SequenceInputs<'_>,
    start: i64,
    end: i64,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if start > end {
        return Ok(Vec::new());
    }
    let focus_size = end
        .checked_sub(start)
        .and_then(|difference| difference.checked_add(1))
        .and_then(|span| usize::try_from(span).ok())
        .ok_or_else(|| {
            super::failure(
                "FXRT0007",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                "integer range cannot be represented by this host",
            )
        })?;
    let mut result = Vec::new();
    for (offset, value) in (start..=end).enumerate() {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| super::control_failure(failure, inputs.request_id))?;
        result.extend(apply_integer_template(
            inputs,
            value,
            mode,
            parameters,
            SequenceFocus {
                position: offset + 1,
                size: focus_size,
            },
            control,
        )?);
    }
    Ok(result)
}

pub(super) fn apply_imports(
    inputs: &SequenceInputs<'_>,
    value: i64,
    mode: Option<&str>,
    current_index: usize,
    parameters: &BTreeMap<String, InvocationParameter>,
    focus: SequenceFocus,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let current_precedence = inputs.program.matched_templates[current_index].import_precedence;
    let selected = select_integer_template(inputs, value, mode, control, |index, template| {
        // The admitted atomic case has two sibling leaf imports. A rule in the
        // principal level may reach either imported branch; a rule in either
        // leaf has no imported descendants and therefore falls through to the
        // built-in atomic rule rather than crossing into its sibling.
        (current_precedence == 0
            && index != current_index
            && template.import_precedence < current_precedence)
            .then_some((template.import_precedence, template.priority, index))
    })?;
    execute_selected_or_builtin(inputs, value, mode, selected, parameters, focus, control)
}

fn apply_integer_template(
    inputs: &SequenceInputs<'_>,
    value: i64,
    mode: Option<&str>,
    parameters: &BTreeMap<String, InvocationParameter>,
    focus: SequenceFocus,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let selected = select_integer_template(inputs, value, mode, control, |index, template| {
        Some((template.import_precedence, template.priority, index))
    })?;
    execute_selected_or_builtin(inputs, value, mode, selected, parameters, focus, control)
}

fn select_integer_template<'a>(
    inputs: &'a SequenceInputs<'_>,
    value: i64,
    mode: Option<&str>,
    control: &mut InvocationControl,
    eligible_rank: impl Fn(
        usize,
        &MatchedTemplate,
    ) -> Option<(
        i32,
        crate::xslt::golden_semantics_experiment::TemplatePriority,
        usize,
    )>,
) -> Result<Option<(usize, &'a MatchedTemplate)>, ExecutionFailure> {
    let mut selected = None;
    let mut selected_rank = None;
    for (index, template) in inputs.program.matched_templates.iter().enumerate() {
        control
            .charge_template_candidate()
            .map_err(|failure| super::control_failure(failure, inputs.request_id))?;
        if !super::template_selector::accepts_mode(&template.modes, mode)
            || !matches!(
                template.pattern,
                MatchPattern::AtomicIntegerGreaterOrEqual(threshold) if value >= threshold
            )
        {
            continue;
        }
        let Some(rank) = eligible_rank(index, template) else {
            continue;
        };
        if selected_rank.is_none_or(|current| rank >= current) {
            selected = Some((index, template));
            selected_rank = Some(rank);
        }
    }
    Ok(selected)
}

fn execute_selected_or_builtin(
    inputs: &SequenceInputs<'_>,
    value: i64,
    mode: Option<&str>,
    selected: Option<(usize, &MatchedTemplate)>,
    parameters: &BTreeMap<String, InvocationParameter>,
    focus: SequenceFocus,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let Some((index, template)) = selected else {
        let mut result = Vec::new();
        append_text(&mut result, &value.to_string(), inputs.request_id, control)?;
        return Ok(result);
    };
    let variables = bind_template_parameters(
        &template.template,
        parameters,
        &inputs.globals.atomics,
        inputs.complete_atomic_frame_clones,
        inputs.request_id,
    )?;
    execute_sequence(
        inputs,
        &template.template.body,
        SequenceContext::for_atomic_template(value, mode, index, focus),
        &variables,
        control,
    )
}
