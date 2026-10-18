//! Bounded typed `XPath` 1.0 concatenation compilation; no runtime or host state.

use super::{
    CompileFailure, Document, ExpandedName, NodeId, PathStep, SourceLocation,
    Xslt10ConcatExpression, Xslt10ConcatPart, compile_xslt10_sum_path,
    effective_xpath_default_namespace, generated_node_argument, is_ascii_ncname, map_path_failure,
    parse_location_path, parse_xslt10_variable_position_selection, xpath_string_literal,
};

pub(in crate::compile) fn compile_xslt10_concat(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<Xslt10ConcatExpression>, CompileFailure> {
    const MAX_ARGUMENTS: usize = 64;
    let Some(arguments) = expression
        .trim()
        .strip_prefix("concat(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Ok(None);
    };
    let Some(arguments) =
        crate::xpath::static_string_experiment::split_arguments(arguments, MAX_ARGUMENTS)
    else {
        return Ok(None);
    };
    if arguments.len() < 2 {
        return Ok(None);
    }
    let mut parts = Vec::with_capacity(arguments.len());
    for argument in arguments {
        if let Some(value) = xpath_string_literal(argument) {
            parts.push(Xslt10ConcatPart::Literal(value.to_owned()));
            continue;
        }
        if argument.parse::<i64>().is_ok() {
            parts.push(Xslt10ConcatPart::Literal(argument.to_owned()));
            continue;
        }
        if let Some(variable) = argument
            .strip_prefix('$')
            .filter(|name| is_ascii_ncname(name))
        {
            parts.push(Xslt10ConcatPart::Variable(variable.to_owned()));
            continue;
        }
        if let Some(variable) = argument
            .strip_prefix("string(")
            .and_then(|value| value.strip_suffix(')'))
            .map(str::trim)
            .and_then(|value| value.strip_prefix('$'))
            .filter(|name| is_ascii_ncname(name))
        {
            parts.push(Xslt10ConcatPart::Variable(variable.to_owned()));
            continue;
        }
        if let Some((variable, position_variable)) =
            parse_xslt10_variable_position_selection(argument)
        {
            parts.push(Xslt10ConcatPart::VariablePosition {
                variable: variable.to_owned(),
                position_variable: position_variable.to_owned(),
            });
            continue;
        }
        if let Some(path) = compile_xslt10_sum_path(document, element, argument, location)? {
            parts.push(Xslt10ConcatPart::SumPath(path));
            continue;
        }
        if let Some(expression) =
            super::super::composed_string_compiler::compile(document, element, argument, location)?
        {
            parts.push(Xslt10ConcatPart::ComposedString(Box::new(expression)));
            continue;
        }
        if let Some(expression) =
            super::compile_xslt10_path_substring(document, element, argument, location)?
        {
            parts.push(Xslt10ConcatPart::PathSubstring(Box::new(expression)));
            continue;
        }
        if let Some(argument) = generated_node_argument(argument) {
            if crate::xpath::path_experiment::qualified_nodes::recognizes_namespace_axis(argument) {
                let path = crate::xpath::path_experiment::qualified_nodes::parse(
                    argument,
                    location.clone(),
                )
                .map_err(map_path_failure)?;
                parts.push(Xslt10ConcatPart::QualifiedGeneratedIdentity(path));
                continue;
            }
            let path = parse_location_path(argument, location.clone()).map_err(map_path_failure)?;
            parts.push(Xslt10ConcatPart::GeneratedNodeIdentity(path));
            continue;
        }
        parts.push(compile_path_operand(document, element, argument, location)?);
    }
    Ok(Some(Xslt10ConcatExpression { parts }))
}

fn compile_path_operand(
    document: &Document,
    element: NodeId,
    argument: &str,
    location: &SourceLocation,
) -> Result<Xslt10ConcatPart, CompileFailure> {
    if (super::super::is_xslt_element(document, element, "value-of")
        || super::super::is_xslt_element(document, element, "sort"))
        && let Some(lookup) = super::super::bound_variable_path_compiler::compile_current_lookup(
            document, element, argument, location,
        )?
    {
        return Ok(Xslt10ConcatPart::CurrentVariableLookup(Box::new(lookup)));
    }
    let mut path = parse_location_path(argument, location.clone()).map_err(map_path_failure)?;
    if let Some(namespace) = effective_xpath_default_namespace(document, element) {
        for step in &mut path.steps {
            if let PathStep::ChildNamed(local) = step {
                *step = PathStep::ChildExpandedName(ExpandedName {
                    namespace: Some(namespace.to_owned()),
                    local: local.clone(),
                });
            }
        }
    }
    Ok(Xslt10ConcatPart::Path(path))
}
