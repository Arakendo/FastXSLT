//! Bounded selected-profile call timing; shared Boolean IR/runtime, no evaluator.
use super::*;

pub(super) fn text_result(condition: BooleanExpression, location: &SourceLocation) -> Instruction {
    let text = |value: &str| Instruction::Text {
        value: value.to_owned(),
        disable_output_escaping: false,
        location: location.clone(),
    };
    Instruction::Choose {
        branches: vec![crate::xslt::golden_semantics_experiment::ChooseBranch {
            condition_reference: location.clone(),
            test: condition,
            body: vec![text("true")],
        }],
        otherwise: vec![text("false")],
        location: location.clone(),
    }
}

pub(super) fn compile(
    expression: &str,
    version: LocalVersion,
    location: &SourceLocation,
    bindings: &[String],
) -> Result<Option<BooleanExpression>, CompileFailure> {
    compile_bounded(expression, version, location, bindings, true)
}

pub(super) fn condition(
    expression: &str,
    version: LocalVersion,
    location: &SourceLocation,
    bindings: &[String],
) -> Result<Option<BooleanExpression>, CompileFailure> {
    compile_bounded(expression, version, location, bindings, false)
}

fn compile_bounded(
    expression: &str,
    version: LocalVersion,
    location: &SourceLocation,
    bindings: &[String],
    require_call: bool,
) -> Result<Option<BooleanExpression>, CompileFailure> {
    // Private admission bounds, not supported XPath or host-policy limits.
    if expression.len() > 4096 {
        return Ok(None);
    }
    let Some((value, contains_call)) = parse(expression, location, version, 0) else {
        return Ok(None);
    };
    if require_call && !contains_call {
        return Ok(None);
    }
    if let Some(name) = unbound_variable(&value, bindings) {
        return Err(invalid(
            "XPST0008",
            format!("unbound variable: ${name}"),
            location,
        ));
    }
    if contains_arity_error(&value) && !matches!(version, LocalVersion::LegacyForward) {
        return Err(invalid(
            "XPST0017",
            "system-property requires exactly one argument",
            location,
        ));
    }
    Ok(Some(value))
}

fn contains_arity_error(value: &BooleanExpression) -> bool {
    match value {
        BooleanExpression::Xslt10DeferredFailure {
            code: "XPST0017", ..
        } => true,
        BooleanExpression::And { left, right } | BooleanExpression::Or { left, right } => {
            contains_arity_error(left) || contains_arity_error(right)
        }
        BooleanExpression::Not(value) => contains_arity_error(value),
        _ => false,
    }
}

// Static lexical resolution visits both operands, independently of runtime
// short circuit. Do this only after the complete bounded grammar is admitted.
fn unbound_variable<'a>(value: &'a BooleanExpression, bindings: &[String]) -> Option<&'a str> {
    match value {
        BooleanExpression::VariableEffectiveBooleanValue(name) => {
            (!bindings.contains(name)).then_some(name.as_str())
        }
        BooleanExpression::And { left, right } | BooleanExpression::Or { left, right } => {
            unbound_variable(left, bindings).or_else(|| unbound_variable(right, bindings))
        }
        BooleanExpression::Not(value) => unbound_variable(value, bindings),
        _ => None,
    }
}

fn parse(
    expression: &str,
    location: &SourceLocation,
    version: LocalVersion,
    depth: usize,
) -> Option<(BooleanExpression, bool)> {
    if depth >= 32 {
        return None;
    }
    let expression = boolean_expression_compiler::strip_enclosing_parentheses(expression.trim());
    for (parts, conjunction) in [
        (
            boolean_expression_compiler::split_top_level_or(expression),
            false,
        ),
        (
            boolean_expression_compiler::split_top_level_and(expression),
            true,
        ),
    ] {
        if let Some((left, right)) = parts {
            let (left, left_call) = parse(left, location, version, depth + 1)?;
            let (right, right_call) = parse(right, location, version, depth + 1)?;
            let (left, right) = (Box::new(left), Box::new(right));
            return Some((
                if conjunction {
                    BooleanExpression::And { left, right }
                } else {
                    BooleanExpression::Or { left, right }
                },
                left_call || right_call,
            ));
        }
    }
    if matches!(
        version,
        LocalVersion::LegacyNormal | LocalVersion::LegacyForward
    ) && let Some((function, code)) =
        super::super::xslt10_static_introspection_compiler::numeric_literal_qname_error(expression)
    {
        return Some((
            BooleanExpression::Xslt10DeferredFailure {
                code,
                detail: format!(
                    "{function} argument converts to a string that is not a valid QName"
                ),
                location: location.clone(),
            },
            true,
        ));
    }
    if direct_literal_property_arity_error(expression) {
        return Some((
            BooleanExpression::Xslt10DeferredFailure {
                code: "XPST0017",
                detail: "system-property requires exactly one argument".to_owned(),
                location: location.clone(),
            },
            true,
        ));
    }
    for (name, negate) in [("not(", true), ("boolean(", false)] {
        if let Some(argument) = expression
            .strip_prefix(name)
            .and_then(|s| s.strip_suffix(')'))
        {
            let (value, call) = parse(argument, location, version, depth + 1)?;
            return Some((
                if negate {
                    BooleanExpression::Not(Box::new(value))
                } else {
                    value
                },
                call,
            ));
        }
    }
    if matches!(expression, "true()" | "false()") {
        return Some((BooleanExpression::Constant(expression == "true()"), false));
    }
    if let Some(name) = expression
        .strip_prefix('$')
        .filter(|name| is_ascii_ncname(name))
    {
        return Some((
            BooleanExpression::VariableEffectiveBooleanValue(name.to_owned()),
            false,
        ));
    }
    // Dynamic source truth, not compile-time folding of a known branch. Keep
    // qualified names, predicates and all other calls guarded.
    if !expression.split('/').all(is_ascii_ncname) {
        return None;
    }
    let path =
        crate::xpath::path_experiment::parse_location_path(expression, location.clone()).ok()?;
    Some((BooleanExpression::NodeExists(path), false))
}
