//! Invocation-time validation for admitted `xsl:number` control AVTs.

use crate::execution_control_experiment::InvocationControl;
use crate::xslt::golden_semantics_experiment::{
    NumberAttributeValuePlan, NumberFormat, NumberTokenStyle,
};

use super::super::{
    ExecutionFailure, FailureCategory, RuntimeVariables, SequenceContext, SequenceInputs, failure,
    value_evaluator,
};

fn evaluate(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    plan: &NumberAttributeValuePlan,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    match plan {
        NumberAttributeValuePlan::Xslt10Variable(variable) => {
            value_evaluator::xslt10_variable_string_value(inputs, variable, variables, control)
        }
        NumberAttributeValuePlan::Xslt10Concat(expression) => {
            value_evaluator::evaluate_xslt10_concat(
                inputs,
                execution.node,
                expression,
                variables,
                control,
            )
        }
    }
}

pub(super) fn validate_effective_letter_value(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    plan: Option<&NumberAttributeValuePlan>,
    format: &NumberFormat,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let Some(plan) = plan else {
        return Ok(());
    };
    let value = evaluate(inputs, execution, plan, variables, control)?;
    if !matches!(value.as_str(), "alphabetic" | "traditional") {
        return Err(failure(
            "XTDE0030",
            FailureCategory::Invalid,
            Some(inputs.request_id),
            format!("invalid effective xsl:number letter-value: {value}"),
        ));
    }
    let alphabetic_is_admitted = format.tokens.iter().all(|token| {
        matches!(
            token.style,
            NumberTokenStyle::Decimal
                | NumberTokenStyle::AlphabeticUpper
                | NumberTokenStyle::AlphabeticLower
                | NumberTokenStyle::RomanUpper
                | NumberTokenStyle::RomanLower
                | NumberTokenStyle::GreekAlphabeticLower
        )
    });
    if value == "traditional" || alphabetic_is_admitted {
        return Ok(());
    }
    Err(failure(
        "FXRT1017",
        FailureCategory::Unsupported,
        Some(inputs.request_id),
        format!(
            "effective xsl:number letter-value is outside the admitted format semantics: {value}"
        ),
    ))
}

pub(super) fn validate_effective_language(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    plan: Option<&NumberAttributeValuePlan>,
    format: &NumberFormat,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let Some(plan) = plan else {
        return Ok(());
    };
    let language = evaluate(inputs, execution, plan, variables, control)?
        .trim()
        .to_ascii_lowercase();
    let language_is_irrelevant = format
        .tokens
        .iter()
        .all(|token| token.style == NumberTokenStyle::Decimal);
    let admitted_latin_language = matches!(language.as_str(), "da" | "en" | "fi" | "no" | "sv")
        && format.tokens.iter().all(|token| {
            matches!(
                token.style,
                NumberTokenStyle::Decimal
                    | NumberTokenStyle::AlphabeticUpper
                    | NumberTokenStyle::AlphabeticLower
            )
        });
    if language_is_irrelevant || admitted_latin_language {
        return Ok(());
    }
    Err(failure(
        "FXRT1017",
        FailureCategory::Unsupported,
        Some(inputs.request_id),
        format!(
            "effective xsl:number language is outside the admitted numbering semantics: {language}"
        ),
    ))
}
