//! Private typed compilation of `xsl:value-of` expressions.

use super::{
    BooleanParseFailure, CaseConversionParseFailure, CompileCategory, CompileFailure,
    DeepEqualFailureKind, DefaultCollationParseFailure, Document, DurationComponentParseFailure,
    EffectiveBooleanFailure, EncodeForUriParseFailure, EscapeHtmlUriParseFailure, ExpandedName,
    IriToUriParseFailure, LocationPath, NodeId, PathFailure, PathStep, ScalarExpression,
    SequenceCardinalityParseFailure, SourceLocation, StringLengthParseFailure, ValueExpression,
    XSLT_NAMESPACE, boolean_expression_compiler, classify_atomic_path_operand,
    classify_missing_context, conditional_expression_compiler, effective_xpath_default_namespace,
    invalid, is_ascii_ncname, map_path_failure, namespace_for_prefix, optional_attribute,
    parse_case_conversion, parse_castable, parse_context_focus_equality, parse_decimal_sum_for,
    parse_deep_equal, parse_default_collation, parse_document_boolean, parse_duration_component,
    parse_encode_for_uri, parse_escape_html_uri, parse_focus_sum_for, parse_format_number,
    parse_generated_document_root, parse_generated_temporary_root, parse_integer_for,
    parse_iri_to_uri, parse_literal_comparison, parse_location_path, parse_qualified_child_path,
    parse_sequence_cardinality, parse_source_free_scalar, parse_string_length,
    recognizes_case_conversion, recognizes_deep_equal, recognizes_default_collation,
    recognizes_document_boolean, recognizes_duration_component, recognizes_encode_for_uri,
    recognizes_escape_html_uri, recognizes_iri_to_uri, recognizes_sequence_cardinality,
    recognizes_source_free_scalar, recognizes_string_length, unsupported, xpath_string_literal,
};
use crate::xslt::golden_semantics_experiment::{
    Xslt10ConcatExpression, Xslt10ConcatPart, Xslt10PathStringFunction,
    Xslt10PathStringFunctionKind, Xslt10PathSubstring, Xslt10PathTranslate,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ValueCompatibilityMode {
    Modern,
    Xslt10,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ValueStaticContext {
    compatibility: ValueCompatibilityMode,
}

impl ValueStaticContext {
    fn for_element(document: &Document, element: NodeId) -> Self {
        let mut current = Some(element);
        while let Some(node) = current {
            if document.name(node).is_some_and(|name| {
                name.namespace.as_deref() == Some(XSLT_NAMESPACE)
                    && matches!(name.local.as_str(), "stylesheet" | "transform")
            }) {
                return Self {
                    compatibility: if optional_attribute(document, node, None, "version")
                        == Some("1.0")
                    {
                        ValueCompatibilityMode::Xslt10
                    } else {
                        ValueCompatibilityMode::Modern
                    },
                };
            }
            current = document.parent(node);
        }
        Self {
            compatibility: ValueCompatibilityMode::Modern,
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the ordered typed expression-family dispatch is one cohesive responsibility"
)]
pub(super) fn compile_value_expression(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    let static_context = ValueStaticContext::for_element(document, element);
    if let Some(literal) = xpath_string_literal(expression.trim()) {
        return Ok(ValueExpression::LiteralString(literal.to_owned()));
    }
    if let Some(literal) = crate::xpath::static_string_experiment::fold_concat_literals(expression)
    {
        return Ok(ValueExpression::LiteralString(literal));
    }
    if let Some(value) =
        crate::xpath::static_string_experiment::fold_binary_literal_function(expression)
    {
        return Ok(match value {
            crate::xpath::static_string_experiment::StaticStringFunctionValue::String(value) => {
                ValueExpression::LiteralString(value)
            }
            crate::xpath::static_string_experiment::StaticStringFunctionValue::Boolean(value) => {
                ValueExpression::SourceFreeScalar(Box::new(ScalarExpression::Boolean(
                    crate::xpath::constant_boolean_experiment::BooleanExpression::Constant(value),
                )))
            }
        });
    }
    if let Some(literal) =
        crate::xpath::static_string_experiment::fold_translate_literals(expression)
    {
        return Ok(ValueExpression::LiteralString(literal));
    }
    if let Some(literal) =
        crate::xpath::static_string_experiment::fold_substring_literals(expression)
    {
        return Ok(ValueExpression::LiteralString(literal));
    }
    if let Some(literal) = crate::xpath::static_string_experiment::fold_string_function(expression)
    {
        return Ok(ValueExpression::LiteralString(literal));
    }
    if let Some(literal) =
        crate::xpath::constant_numeric_experiment::fold_integral_function(expression)
    {
        return Ok(ValueExpression::LiteralString(literal));
    }
    if let Some(literal) =
        crate::xpath::constant_numeric_experiment::fold_exact_integral_arithmetic(expression)
    {
        return Ok(ValueExpression::LiteralString(literal));
    }
    if let Some(literal) =
        crate::xpath::constant_numeric_experiment::fold_number_conversion(expression)
    {
        return Ok(ValueExpression::LiteralString(literal));
    }
    if let Some(value) =
        crate::xpath::constant_boolean_experiment::fold_exact_short_circuit(expression)
    {
        return Ok(ValueExpression::SourceFreeScalar(Box::new(
            ScalarExpression::Boolean(
                crate::xpath::constant_boolean_experiment::BooleanExpression::Constant(value),
            ),
        )));
    }
    if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some(value) =
            crate::xpath::constant_boolean_experiment::fold_xpath10_mixed_equality(expression)
    {
        return Ok(ValueExpression::SourceFreeScalar(Box::new(
            ScalarExpression::Boolean(
                crate::xpath::constant_boolean_experiment::BooleanExpression::Constant(value),
            ),
        )));
    }
    if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some(value) =
            crate::xpath::constant_boolean_experiment::fold_xpath10_ordered_literal_comparison(
                expression,
            )
    {
        return Ok(ValueExpression::SourceFreeScalar(Box::new(
            ScalarExpression::Boolean(
                crate::xpath::constant_boolean_experiment::BooleanExpression::Constant(value),
            ),
        )));
    }
    if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some(value) =
            crate::xpath::constant_numeric_experiment::fold_xslt10_non_finite_division(expression)
    {
        return Ok(match value {
            crate::xpath::constant_numeric_experiment::Xslt10NonFiniteValue::Boolean(value) => {
                ValueExpression::SourceFreeScalar(Box::new(ScalarExpression::Boolean(
                    crate::xpath::constant_boolean_experiment::BooleanExpression::Constant(value),
                )))
            }
            crate::xpath::constant_numeric_experiment::Xslt10NonFiniteValue::Lexical(value) => {
                ValueExpression::LiteralString(value.to_owned())
            }
        });
    }
    if let Some(value) = compile_binary_numeric_path(expression, location, static_context) {
        return Ok(value);
    }
    if let Some(value) =
        crate::xpath::constant_numeric_experiment::fold_boolean_number_equality(expression)
    {
        return Ok(ValueExpression::SourceFreeScalar(Box::new(
            ScalarExpression::Boolean(
                crate::xpath::constant_boolean_experiment::BooleanExpression::Constant(value),
            ),
        )));
    }
    if let Some(value) =
        crate::xpath::constant_numeric_experiment::fold_integral_equality(expression)
    {
        return Ok(ValueExpression::SourceFreeScalar(Box::new(
            ScalarExpression::Boolean(
                crate::xpath::constant_boolean_experiment::BooleanExpression::Constant(value),
            ),
        )));
    }
    if let Some(value) = compile_integral_function_path(document, element, expression, location)? {
        return Ok(value);
    }
    if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some(variable) = parse_xslt10_variable_conversion(expression, "string")
    {
        return Ok(ValueExpression::Xslt10VariableString(variable.to_owned()));
    }
    if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some(variable) = parse_xslt10_variable_conversion(expression, "number")
    {
        return Ok(ValueExpression::Xslt10VariableNumber(variable.to_owned()));
    }
    if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some((path, variable)) =
            compile_xslt10_variable_position_path(document, element, expression, location)?
    {
        return Ok(ValueExpression::Xslt10VariablePositionPath { path, variable });
    }
    if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some((variable, path)) =
            compile_xslt10_variable_path(document, element, expression, location)?
    {
        return Ok(ValueExpression::Xslt10VariablePath { variable, path });
    }
    if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some(function) =
            compile_xslt10_path_string_function(document, element, expression, location)?
    {
        return Ok(ValueExpression::Xslt10PathStringFunction(Box::new(
            function,
        )));
    }
    if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some(path) = compile_xslt10_sum_path(document, element, expression, location)?
    {
        return Ok(ValueExpression::Xslt10SumPath(path));
    }
    if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some(substring) =
            compile_xslt10_path_substring(document, element, expression, location)?
    {
        return Ok(ValueExpression::Xslt10PathSubstring(Box::new(substring)));
    }
    if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some(translate) =
            compile_xslt10_path_translate(document, element, expression, location)?
    {
        return Ok(ValueExpression::Xslt10PathTranslate(Box::new(translate)));
    }
    if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some(concat) = compile_xslt10_concat(document, element, expression, location)?
    {
        return Ok(ValueExpression::Xslt10Concat(Box::new(concat)));
    }
    if let Some(path) = compile_number_path(document, element, expression, location)? {
        return Ok(ValueExpression::NumberPath(path));
    }
    if let Some(path) = compile_string_path(document, element, expression, location) {
        return Ok(ValueExpression::StringPath(path));
    }
    if let Some(path) = compile_name_path(document, element, expression, location) {
        return Ok(ValueExpression::NodeNamePath(path));
    }
    if let Some(failure) = classify_atomic_path_operand(expression, location.clone()) {
        return Err(CompileFailure {
            code: failure.standard_code,
            category: CompileCategory::Invalid,
            detail: failure.detail,
            location: failure.location,
        });
    }
    if let Some(count) = compile_count_value(document, element, expression, location)? {
        return Ok(count);
    }
    if let Some(conditional) =
        conditional_expression_compiler::compile_value(document, element, expression, location)?
    {
        return Ok(conditional);
    }
    if matches!(
        expression.trim(),
        "normalize-space()" | "normalize-space(.)"
    ) {
        return Ok(ValueExpression::ContextNodeNormalizedString);
    }
    if let Some(normalized) = compile_normalize_space_path(document, element, expression, location)
    {
        return Ok(normalized);
    }
    if is_zero_argument_function(expression, "string-length") {
        return Ok(ValueExpression::ContextNodeStringLength(location.clone()));
    }
    if expression.trim() == "position()" {
        return Ok(ValueExpression::ContextPosition(location.clone()));
    }
    if expression.trim() == "last()" {
        return Ok(ValueExpression::ContextSize(location.clone()));
    }
    if let Some((left, right)) = parse_context_focus_equality(expression) {
        return Ok(ValueExpression::ContextFocusEquals {
            left,
            right,
            location: location.clone(),
        });
    }
    if let Some(language) = crate::xpath::language_experiment::parse_literal(expression) {
        return Ok(ValueExpression::ContextLanguageMatches(language));
    }
    if let Some(comparison) = parse_literal_comparison(expression) {
        return Ok(ValueExpression::SourceFreeScalar(Box::new(
            ScalarExpression::Boolean(comparison),
        )));
    }
    Ok(if recognizes_duration_component(expression) {
        compile_duration_component_value(expression, location)?
    } else if recognizes_string_length(expression) {
        compile_string_length_value(expression, location)?
    } else if recognizes_case_conversion(expression) {
        compile_case_conversion_value(expression, location)?
    } else if recognizes_iri_to_uri(expression) {
        compile_iri_to_uri_value(expression, location)?
    } else if recognizes_encode_for_uri(expression) {
        compile_encode_for_uri_value(expression, location)?
    } else if recognizes_escape_html_uri(expression) {
        compile_escape_html_uri_value(expression, location)?
    } else if recognizes_default_collation(expression) {
        compile_default_collation_value(expression, location)?
    } else if recognizes_deep_equal(expression) {
        compile_deep_equal_value(expression, location)?
    } else if let Some(path) = compile_empty_location_path(expression, location)? {
        path
    } else if recognizes_sequence_cardinality(expression) {
        compile_sequence_cardinality_value(expression, location)?
    } else if recognizes_document_boolean(expression) {
        compile_document_boolean_value(expression, location)?
    } else if let Some(variable) = parse_variable_boolean_call(expression) {
        ValueExpression::VariableEffectiveBooleanValue(variable.to_owned())
    } else if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some((variable, value, equal)) = parse_xslt10_variable_boolean_comparison(expression)
    {
        ValueExpression::Xslt10VariableBooleanComparison {
            variable: variable.to_owned(),
            value,
            equal,
        }
    } else if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some((left, right, equal)) =
            parse_xslt10_variable_string_variables_comparison(expression)
    {
        ValueExpression::Xslt10VariableStringVariablesComparison {
            left: left.to_owned(),
            right: right.to_owned(),
            equal,
        }
    } else if static_context.compatibility == ValueCompatibilityMode::Xslt10
        && let Some(comparison) = parse_xslt10_variable_atomic_comparison(expression)
    {
        comparison
    } else if expression.contains('[')
        && let Ok(path) = parse_location_path(expression, location.clone())
    {
        match static_context.compatibility {
            ValueCompatibilityMode::Modern => ValueExpression::LocationPath(path),
            ValueCompatibilityMode::Xslt10 => ValueExpression::Xslt10FirstNodeLocationPath(path),
        }
    } else if recognizes_source_free_scalar(expression) {
        compile_source_free_scalar_value(expression, location)?
    } else if expression.contains(" castable as ") {
        ValueExpression::Castable(Box::new(parse_castable(expression, location).map_err(
            |failure| CompileFailure {
                code: "FXXP1007",
                category: CompileCategory::Unsupported,
                detail: failure.detail,
                location: failure.location,
            },
        )?))
    } else if expression.trim_start().starts_with("format-number(")
        && expression.contains("sum(for $")
    {
        ValueExpression::DecimalSumFor(Box::new(
            parse_decimal_sum_for(expression, location).map_err(|failure| CompileFailure {
                code: "FXXP1006",
                category: CompileCategory::Unsupported,
                detail: failure.detail,
                location: failure.location,
            })?,
        ))
    } else if expression.trim_start().starts_with("format-number(") {
        ValueExpression::FormatNumber(Box::new(
            parse_format_number(expression, location).map_err(|failure| CompileFailure {
                code: "FXXP1009",
                category: CompileCategory::Unsupported,
                detail: failure.detail,
                location: failure.location,
            })?,
        ))
    } else if expression.trim_start().starts_with("sum(for $") {
        ValueExpression::FocusSumFor(Box::new(
            parse_focus_sum_for(expression, location).map_err(|failure| CompileFailure {
                code: "FXXP1005",
                category: CompileCategory::Unsupported,
                detail: failure.detail,
                location: failure.location,
            })?,
        ))
    } else if expression.trim_start().starts_with("for $") {
        ValueExpression::IntegerFor(Box::new(
            parse_integer_for(expression, location.clone()).map_err(|failure| CompileFailure {
                code: "FXXP1004",
                category: CompileCategory::Unsupported,
                detail: failure.detail,
                location: failure.location,
            })?,
        ))
    } else if let Some((left, right)) =
        boolean_expression_compiler::generated_node_identity_test(expression)
    {
        ValueExpression::NodeIdentityEqual {
            left: parse_location_path(left, location.clone()).map_err(map_path_failure)?,
            right: parse_location_path(right, location.clone()).map_err(map_path_failure)?,
        }
    } else if let Some(root) = compile_root_value(document, element, expression, location)? {
        root
    } else if matches!(expression.trim(), "name()" | "name(.)") {
        ValueExpression::ContextNodeName
    } else if expression.trim() == "name(..)" {
        ValueExpression::NodeNamePath(
            parse_location_path("..", location.clone()).map_err(map_path_failure)?,
        )
    } else if matches!(expression.trim(), "local-name()" | "local-name(.)") {
        ValueExpression::ContextNodeLocalName
    } else if let Some(path) =
        compile_expanded_name_path(document, element, expression, "local-name", location)
    {
        ValueExpression::NodeLocalNamePath(path?)
    } else if matches!(expression.trim(), "namespace-uri()" | "namespace-uri(.)") {
        ValueExpression::ContextNodeNamespaceUri
    } else if let Some(path) =
        compile_expanded_name_path(document, element, expression, "namespace-uri", location)
    {
        ValueExpression::NodeNamespaceUriPath(path?)
    } else if matches!(expression.trim(), "string()" | "string(.)") {
        compile_location_path_or_missing_context(document, element, ".", location, static_context)?
    } else if expression.trim() == "upper-case(.)" {
        ValueExpression::UpperCaseContextString
    } else if let Some((literal, variable)) = parse_literal_variable_concat(expression) {
        ValueExpression::LiteralVariableConcat { literal, variable }
    } else if let Some(variable) = expression.strip_prefix('$') {
        if !is_ascii_ncname(variable) {
            return Err(invalid(
                "FXXP0002",
                format!("invalid variable reference: {expression}"),
                location,
            ));
        }
        if static_context.compatibility == ValueCompatibilityMode::Xslt10 {
            ValueExpression::Xslt10VariableString(variable.to_owned())
        } else {
            ValueExpression::Variable(variable.to_owned())
        }
    } else {
        compile_location_path_or_missing_context(
            document,
            element,
            expression,
            location,
            static_context,
        )?
    })
}

fn parse_variable_boolean_call(expression: &str) -> Option<&str> {
    let variable = expression
        .trim()
        .strip_prefix("boolean(")?
        .strip_suffix(')')?
        .trim()
        .strip_prefix('$')?;
    is_ascii_ncname(variable).then_some(variable)
}

fn parse_xslt10_variable_conversion<'a>(expression: &'a str, function: &str) -> Option<&'a str> {
    let variable = expression
        .trim()
        .strip_prefix(function)?
        .trim_start_matches([' ', '\t', '\r', '\n'])
        .strip_prefix('(')?
        .strip_suffix(')')?
        .trim()
        .strip_prefix('$')?;
    is_ascii_ncname(variable).then_some(variable)
}

fn compile_xslt10_variable_position_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<(LocationPath, String)>, CompileFailure> {
    let expression = expression.trim();
    let Some(body) = expression.strip_suffix(']') else {
        return Ok(None);
    };
    let Some(open) = body.rfind('[') else {
        return Ok(None);
    };
    let path = body[..open].trim();
    // Applying the predicate after evaluating an arbitrary multi-step path is
    // not equivalent to XPath's per-step predicate focus.  Admit only the
    // single child-step shape until the general path plan can retain that
    // focus boundary.
    if path.contains('/') {
        return Ok(None);
    }
    let predicate = body[open + 1..].trim();
    let variable = parse_variable_position_predicate(predicate);
    let Some(variable) = variable else {
        return Ok(None);
    };
    let mut path = parse_location_path(path, location.clone()).map_err(map_path_failure)?;
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
    Ok(Some((path, variable.to_owned())))
}

fn parse_variable_position_predicate(predicate: &str) -> Option<&str> {
    let predicate = predicate.trim();
    if let Some(variable) = predicate
        .strip_prefix('$')
        .filter(|name| is_ascii_ncname(name))
    {
        return Some(variable);
    }
    let (left, right) = predicate.split_once('=')?;
    let variable = if left.trim() == "position()" {
        right.trim().strip_prefix('$')?
    } else if right.trim() == "position()" {
        left.trim().strip_prefix('$')?
    } else {
        return None;
    };
    is_ascii_ncname(variable).then_some(variable)
}

fn compile_xslt10_variable_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<(String, LocationPath)>, CompileFailure> {
    let expression = expression.trim();
    let Some((variable, relative)) = expression
        .strip_prefix('$')
        .and_then(|value| value.split_once('/'))
    else {
        return Ok(None);
    };
    if !is_ascii_ncname(variable) || relative.is_empty() {
        return Ok(None);
    }
    let mut path = parse_location_path(relative, location.clone()).map_err(map_path_failure)?;
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
    Ok(Some((variable.to_owned(), path)))
}

fn compile_xslt10_path_string_function(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<Xslt10PathStringFunction>, CompileFailure> {
    let expression = expression.trim();
    let Some((kind, arguments)) = [
        (Xslt10PathStringFunctionKind::Contains, "contains("),
        (Xslt10PathStringFunctionKind::StartsWith, "starts-with("),
        (
            Xslt10PathStringFunctionKind::SubstringBefore,
            "substring-before(",
        ),
        (
            Xslt10PathStringFunctionKind::SubstringAfter,
            "substring-after(",
        ),
    ]
    .into_iter()
    .find_map(|(kind, prefix)| {
        expression
            .strip_prefix(prefix)
            .and_then(|value| value.strip_suffix(')'))
            .map(|arguments| (kind, arguments))
    }) else {
        return Ok(None);
    };
    let Some((path, operand)) = arguments.split_once(',') else {
        return Ok(None);
    };
    let Some(operand) = xpath_string_literal(operand.trim()) else {
        return Ok(None);
    };
    let mut path = parse_location_path(path.trim(), location.clone()).map_err(map_path_failure)?;
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
    Ok(Some(Xslt10PathStringFunction {
        kind,
        path,
        operand: operand.to_owned(),
    }))
}

pub(super) fn compile_xslt10_sum_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<LocationPath>, CompileFailure> {
    let Some(argument) = expression
        .trim()
        .strip_prefix("sum(")
        .and_then(|value| value.strip_suffix(')'))
        .map(str::trim)
        .filter(|argument| !argument.is_empty() && !argument.starts_with('$'))
    else {
        return Ok(None);
    };
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
    Ok(Some(path))
}

fn compile_xslt10_path_substring(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<Xslt10PathSubstring>, CompileFailure> {
    let Some(arguments) = expression
        .trim()
        .strip_prefix("substring(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Ok(None);
    };
    let Some(arguments) = crate::xpath::static_string_experiment::split_arguments(arguments, 3)
    else {
        return Ok(None);
    };
    let [path, start, remainder @ ..] = arguments.as_slice() else {
        return Ok(None);
    };
    if remainder.len() > 1 {
        return Ok(None);
    }
    let Some(start) = crate::xpath::static_string_experiment::parse_finite_number(start) else {
        return Ok(None);
    };
    let length = match remainder.first() {
        Some(length) => {
            let Some(length) = crate::xpath::static_string_experiment::parse_finite_number(length)
            else {
                return Ok(None);
            };
            Some(length.to_bits())
        }
        None => None,
    };
    let mut path = parse_location_path(path, location.clone()).map_err(map_path_failure)?;
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
    Ok(Some(Xslt10PathSubstring {
        path,
        start_bits: start.to_bits(),
        length_bits: length,
    }))
}

fn compile_xslt10_path_translate(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<Xslt10PathTranslate>, CompileFailure> {
    let Some(arguments) = expression
        .trim()
        .strip_prefix("translate(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return Ok(None);
    };
    let Some(arguments) = crate::xpath::static_string_experiment::split_arguments(arguments, 3)
    else {
        return Ok(None);
    };
    let [path, search, replacement] = arguments.as_slice() else {
        return Ok(None);
    };
    let (Some(search), Some(replacement)) = (
        xpath_string_literal(search),
        xpath_string_literal(replacement),
    ) else {
        return Ok(None);
    };
    let mut path = parse_location_path(path, location.clone()).map_err(map_path_failure)?;
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
    Ok(Some(Xslt10PathTranslate {
        path,
        search: search.to_owned(),
        replacement: replacement.to_owned(),
    }))
}

pub(super) fn compile_xslt10_concat(
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
        if let Some(path) = compile_xslt10_sum_path(document, element, argument, location)? {
            parts.push(Xslt10ConcatPart::SumPath(path));
            continue;
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
        parts.push(Xslt10ConcatPart::Path(path));
    }
    Ok(Some(Xslt10ConcatExpression { parts }))
}

fn parse_xslt10_variable_boolean_comparison(expression: &str) -> Option<(&str, bool, bool)> {
    let mut expression = expression.trim();
    let mut invert = false;
    if let Some(inner) = expression
        .strip_prefix("not(")
        .and_then(|value| value.strip_suffix(')'))
    {
        expression = inner.trim();
        invert = true;
    }
    let (left, right, mut equal) = if let Some((left, right)) = expression.split_once("!=") {
        (left.trim(), right.trim(), false)
    } else {
        let (left, right) = expression.split_once('=')?;
        (left.trim(), right.trim(), true)
    };
    equal ^= invert;
    let parse_boolean = |candidate: &str| match candidate {
        "true()" => Some(true),
        "false()" => Some(false),
        _ => None,
    };
    parse_boolean_comparison_variable(left)
        .zip(parse_boolean(right))
        .or_else(|| parse_boolean_comparison_variable(right).zip(parse_boolean(left)))
        .map(|(variable, value)| (variable, value, equal))
}

fn parse_boolean_comparison_variable(candidate: &str) -> Option<&str> {
    candidate
        .strip_prefix('$')
        .filter(|variable| is_ascii_ncname(variable))
}

fn parse_xslt10_variable_string_variables_comparison(
    expression: &str,
) -> Option<(&str, &str, bool)> {
    let (left, right, equal) = if let Some((left, right)) = expression.split_once("!=") {
        (left.trim(), right.trim(), false)
    } else {
        let (left, right) = expression.split_once('=')?;
        (left.trim(), right.trim(), true)
    };
    if left.contains(['=', '!']) || right.contains(['=', '!']) {
        return None;
    }
    Some((
        parse_boolean_comparison_variable(left)?,
        parse_boolean_comparison_variable(right)?,
        equal,
    ))
}

fn parse_xslt10_variable_atomic_comparison(expression: &str) -> Option<ValueExpression> {
    let mut expression = expression.trim();
    let mut negate = false;
    if let Some(inner) = expression
        .strip_prefix("not(")
        .and_then(|value| value.strip_suffix(')'))
    {
        expression = inner.trim();
        negate = true;
    }
    let (left, right, equal) = if let Some((left, right)) = expression.split_once("!=") {
        (left.trim(), right.trim(), false)
    } else {
        let (left, right) = expression.split_once('=')?;
        (left.trim(), right.trim(), true)
    };
    let (variable, atomic) = parse_boolean_comparison_variable(left)
        .map(|variable| (variable, right))
        .or_else(|| parse_boolean_comparison_variable(right).map(|variable| (variable, left)))?;
    if let Some(value) = xpath_string_literal(atomic) {
        return Some(ValueExpression::Xslt10VariableStringComparison {
            variable: variable.to_owned(),
            value: value.to_owned(),
            equal,
            negate,
        });
    }
    atomic
        .parse::<i32>()
        .ok()
        .map(|value| ValueExpression::Xslt10VariableNumberComparison {
            variable: variable.to_owned(),
            value,
            equal,
            negate,
        })
}

fn compile_binary_numeric_path(
    expression: &str,
    location: &SourceLocation,
    static_context: ValueStaticContext,
) -> Option<ValueExpression> {
    let root = compile_binary_numeric_node(
        expression,
        location,
        static_context.compatibility == ValueCompatibilityMode::Xslt10,
    )?;
    if !matches!(
        root,
        crate::xpath::binary_numeric_experiment::BinaryNumericNode::Literal(_)
            | crate::xpath::binary_numeric_experiment::BinaryNumericNode::Operation { .. }
    ) {
        return None;
    }
    let selection = match static_context.compatibility {
        ValueCompatibilityMode::Xslt10 => {
            crate::xpath::binary_numeric_experiment::NumericOperandSelection::FirstInDocumentOrder
        }
        ValueCompatibilityMode::Modern => {
            crate::xpath::binary_numeric_experiment::NumericOperandSelection::ZeroOrOne
        }
    };
    Some(ValueExpression::BinaryNumeric(Box::new(
        crate::xpath::binary_numeric_experiment::BinaryNumericExpression {
            root,
            selection,
            location: location.clone(),
        },
    )))
}

fn compile_binary_numeric_node(
    expression: &str,
    location: &SourceLocation,
    allow_xslt10_variables: bool,
) -> Option<crate::xpath::binary_numeric_experiment::BinaryNumericNode> {
    use crate::xpath::binary_numeric_experiment::BinaryNumericNode;
    if let Some((left, operator, right)) =
        crate::xpath::binary_numeric_experiment::split_paths(expression)
    {
        return Some(BinaryNumericNode::Operation {
            left: Box::new(compile_binary_numeric_node(
                left,
                location,
                allow_xslt10_variables,
            )?),
            operator,
            right: Box::new(compile_binary_numeric_node(
                right,
                location,
                allow_xslt10_variables,
            )?),
        });
    }
    let (operand, negate) = crate::xpath::binary_numeric_experiment::signed_path(expression)?;
    if negate {
        return Some(BinaryNumericNode::Negate(Box::new(
            compile_binary_numeric_node(operand, location, allow_xslt10_variables)?,
        )));
    }
    if let Some(value) =
        crate::xpath::binary_numeric_experiment::ExactRational::parse_decimal(operand)
    {
        return Some(BinaryNumericNode::Literal(value));
    }
    if allow_xslt10_variables
        && let Some(variable) = operand.strip_prefix('$')
        && is_ascii_ncname(variable)
    {
        return Some(BinaryNumericNode::Variable(variable.to_owned()));
    }
    Some(BinaryNumericNode::Path {
        path: parse_location_path(operand, location.clone()).ok()?,
        negate: false,
    })
}

fn compile_location_path_or_missing_context(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
    static_context: ValueStaticContext,
) -> Result<ValueExpression, CompileFailure> {
    let path = parse_location_path(expression, location.clone()).or_else(|failure| {
        let is_simple_qualified_path = expression.contains(':')
            && !expression.contains('*')
            && !expression.contains("::")
            && !expression.contains(['(', ')', '[', ']']);
        if matches!(failure, PathFailure::Unsupported { .. }) && is_simple_qualified_path {
            parse_qualified_child_path(expression, location.clone(), |prefix| {
                namespace_for_prefix(document, element, prefix).map(str::to_owned)
            })
        } else {
            Err(failure)
        }
    });
    match path {
        Ok(path) => Ok(match static_context.compatibility {
            ValueCompatibilityMode::Modern => ValueExpression::LocationPath(path),
            ValueCompatibilityMode::Xslt10 => ValueExpression::Xslt10FirstNodeLocationPath(path),
        }),
        Err(failure @ PathFailure::Unsupported { .. }) => {
            if let Some(requirement) = classify_missing_context(expression, location.clone()) {
                Ok(ValueExpression::ContextRequiredOnly(requirement.location))
            } else {
                Err(map_path_failure(failure))
            }
        }
        Err(failure) => Err(map_path_failure(failure)),
    }
}

fn compile_empty_location_path(
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ValueExpression>, CompileFailure> {
    let expression = expression.trim();
    let Some(argument) = ["empty(", "fn:empty("].iter().find_map(|prefix| {
        expression
            .strip_prefix(prefix)
            .and_then(|value| value.strip_suffix(')'))
    }) else {
        return Ok(None);
    };
    if !argument.trim_start().starts_with("()/") {
        return Ok(None);
    }
    let path = parse_location_path(argument.trim(), location.clone()).map_err(map_path_failure)?;
    Ok(Some(ValueExpression::EmptyLocationPath(path)))
}

fn compile_document_boolean_value(
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    let expression =
        parse_document_boolean(expression, location).map_err(|failure| match failure {
            EffectiveBooleanFailure::Path(failure) => map_path_failure(failure),
            EffectiveBooleanFailure::InvalidTypeOrCardinality => invalid(
                "FORG0006",
                "effective boolean value is undefined for the supplied sequence",
                location,
            ),
            EffectiveBooleanFailure::Control(_) => {
                unreachable!("compilation performs no work charges")
            }
            EffectiveBooleanFailure::Unsupported => unsupported(
                "FXXP1020",
                "document-aware boolean expression is outside the admitted production slice",
                location,
            ),
        })?;
    Ok(ValueExpression::DocumentBoolean(Box::new(expression)))
}

fn compile_source_free_scalar_value(
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    let expression = parse_source_free_scalar(expression).map_err(|failure| {
        let (code, category, detail) = match failure {
            BooleanParseFailure::InvalidArity => (
                "XPST0017",
                CompileCategory::Invalid,
                "boolean function has invalid arity",
            ),
            BooleanParseFailure::InvalidEffectiveBooleanValue => (
                "FORG0006",
                CompileCategory::Invalid,
                "effective boolean value is undefined for the supplied sequence",
            ),
            BooleanParseFailure::Unsupported => (
                "FXXP1019",
                CompileCategory::Unsupported,
                "source-free boolean expression is outside the admitted production slice",
            ),
        };
        CompileFailure {
            code,
            category,
            detail: detail.to_owned(),
            location: location.clone(),
        }
    })?;
    Ok(ValueExpression::SourceFreeScalar(Box::new(expression)))
}

fn compile_sequence_cardinality_value(
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    let expression = parse_sequence_cardinality(expression).map_err(|failure| {
        let (code, category, detail) = match failure {
            SequenceCardinalityParseFailure::InvalidArity => (
                "XPST0017",
                CompileCategory::Invalid,
                "sequence-cardinality function has invalid arity",
            ),
            SequenceCardinalityParseFailure::Unsupported => (
                "FXXP1018",
                CompileCategory::Unsupported,
                "sequence-cardinality expression is outside the admitted production slice",
            ),
        };
        CompileFailure {
            code,
            category,
            detail: detail.to_owned(),
            location: location.clone(),
        }
    })?;
    Ok(ValueExpression::SequenceCardinality(Box::new(expression)))
}

fn compile_duration_component_value(
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    let expression = parse_duration_component(expression).map_err(|failure| {
        let (code, category, detail) = match failure {
            DurationComponentParseFailure::InvalidArity => (
                "XPST0017",
                CompileCategory::Invalid,
                "duration-component function has invalid arity",
            ),
            DurationComponentParseFailure::Unsupported => (
                "FXXP1017",
                CompileCategory::Unsupported,
                "duration-component expression shape is outside the admitted production slice",
            ),
        };
        CompileFailure {
            code,
            category,
            detail: detail.to_owned(),
            location: location.clone(),
        }
    })?;
    Ok(ValueExpression::DurationComponent(Box::new(expression)))
}

fn compile_string_length_value(
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    let expression = parse_string_length(expression, location.clone()).map_err(|failure| {
        let (code, category, detail) = match failure {
            StringLengthParseFailure::InvalidArity => (
                "XPST0017",
                CompileCategory::Invalid,
                "fn:string-length has invalid arity",
            ),
            StringLengthParseFailure::InvalidArgumentType => (
                "XPTY0004",
                CompileCategory::Invalid,
                "fn:string-length has an invalid argument type",
            ),
            StringLengthParseFailure::MissingContext => (
                "XPDY0002",
                CompileCategory::Invalid,
                "fn:string-length requires an absent context item",
            ),
            StringLengthParseFailure::Unsupported => (
                "FXXP1016",
                CompileCategory::Unsupported,
                "string-length expression shape is outside the admitted production slice",
            ),
        };
        CompileFailure {
            code,
            category,
            detail: detail.to_owned(),
            location: location.clone(),
        }
    })?;
    Ok(ValueExpression::StringLength(Box::new(expression)))
}

fn compile_case_conversion_value(
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    let expression = parse_case_conversion(expression).map_err(|failure| {
        let (code, category, detail) = match failure {
            CaseConversionParseFailure::InvalidArity => (
                "XPST0017",
                CompileCategory::Invalid,
                "case-conversion function has invalid arity",
            ),
            CaseConversionParseFailure::InvalidArgumentType => (
                "XPTY0004",
                CompileCategory::Invalid,
                "case-conversion expression has an invalid argument type",
            ),
            CaseConversionParseFailure::MissingContext => (
                "XPDY0002",
                CompileCategory::Invalid,
                "case-conversion expression requires an absent context item",
            ),
            CaseConversionParseFailure::Unsupported => (
                "FXXP1015",
                CompileCategory::Unsupported,
                "case-conversion expression shape is outside the admitted production slice",
            ),
        };
        CompileFailure {
            code,
            category,
            detail: detail.to_owned(),
            location: location.clone(),
        }
    })?;
    Ok(ValueExpression::CaseConversion(Box::new(expression)))
}

fn compile_iri_to_uri_value(
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    let expression = parse_iri_to_uri(expression).map_err(|failure| {
        let (code, category, detail) = match failure {
            IriToUriParseFailure::InvalidArity => (
                "XPST0017",
                CompileCategory::Invalid,
                "fn:iri-to-uri requires one argument",
            ),
            IriToUriParseFailure::InvalidArgumentType => (
                "XPTY0004",
                CompileCategory::Invalid,
                "fn:iri-to-uri requires an optional string",
            ),
            IriToUriParseFailure::Unsupported => (
                "FXXP1014",
                CompileCategory::Unsupported,
                "iri-to-uri expression shape is outside the admitted production slice",
            ),
        };
        CompileFailure {
            code,
            category,
            detail: detail.to_owned(),
            location: location.clone(),
        }
    })?;
    Ok(ValueExpression::IriToUri(Box::new(expression)))
}

fn compile_encode_for_uri_value(
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    let expression = parse_encode_for_uri(expression).map_err(|failure| {
        let (code, category, detail) = match failure {
            EncodeForUriParseFailure::InvalidArity => (
                "XPST0017",
                CompileCategory::Invalid,
                "fn:encode-for-uri requires one argument",
            ),
            EncodeForUriParseFailure::InvalidArgumentType => (
                "XPTY0004",
                CompileCategory::Invalid,
                "fn:encode-for-uri requires an optional string",
            ),
            EncodeForUriParseFailure::Unsupported => (
                "FXXP1013",
                CompileCategory::Unsupported,
                "encode-for-uri expression shape is outside the admitted production slice",
            ),
        };
        CompileFailure {
            code,
            category,
            detail: detail.to_owned(),
            location: location.clone(),
        }
    })?;
    Ok(ValueExpression::EncodeForUri(Box::new(expression)))
}

fn compile_escape_html_uri_value(
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    let expression = parse_escape_html_uri(expression).map_err(|failure| {
        let (code, category, detail) = match failure {
            EscapeHtmlUriParseFailure::InvalidArity => (
                "XPST0017",
                CompileCategory::Invalid,
                "fn:escape-html-uri requires one argument",
            ),
            EscapeHtmlUriParseFailure::InvalidArgumentType => (
                "XPTY0004",
                CompileCategory::Invalid,
                "fn:escape-html-uri requires an optional string",
            ),
            EscapeHtmlUriParseFailure::Unsupported => (
                "FXXP1012",
                CompileCategory::Unsupported,
                "escape-html-uri expression shape is outside the admitted production slice",
            ),
        };
        CompileFailure {
            code,
            category,
            detail: detail.to_owned(),
            location: location.clone(),
        }
    })?;
    Ok(ValueExpression::EscapeHtmlUri(Box::new(expression)))
}

fn compile_default_collation_value(
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    let expression = parse_default_collation(expression).map_err(|failure| {
        let (code, category, detail) = match failure {
            DefaultCollationParseFailure::InvalidArity => (
                "XPST0017",
                CompileCategory::Invalid,
                "fn:default-collation requires zero arguments",
            ),
            DefaultCollationParseFailure::Unsupported => (
                "FXXP1011",
                CompileCategory::Unsupported,
                "default-collation expression shape is outside the admitted production slice",
            ),
        };
        CompileFailure {
            code,
            category,
            detail: detail.to_owned(),
            location: location.clone(),
        }
    })?;
    Ok(ValueExpression::DefaultCollation(Box::new(expression)))
}

fn compile_deep_equal_value(
    expression: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    let expression = parse_deep_equal(expression, location).map_err(|failure| {
        let (code, category) = match failure.kind {
            DeepEqualFailureKind::InvalidArity { standard_code }
            | DeepEqualFailureKind::InvalidCollation { standard_code }
            | DeepEqualFailureKind::InvalidCollationType { standard_code } => {
                (standard_code, CompileCategory::Invalid)
            }
            DeepEqualFailureKind::Unsupported => ("FXXP1010", CompileCategory::Unsupported),
        };
        CompileFailure {
            code,
            category,
            detail: failure.detail,
            location: failure.location,
        }
    })?;
    Ok(ValueExpression::DeepEqual(Box::new(expression)))
}

fn compile_count_value(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ValueExpression>, CompileFailure> {
    let expression = expression.trim();
    let Some(argument) = ["count(", "fn:count("].iter().find_map(|prefix| {
        expression
            .strip_prefix(prefix)
            .and_then(|value| value.strip_suffix(')'))
    }) else {
        return Ok(None);
    };
    if argument.trim_start().starts_with("fn:") && argument.contains('(') {
        return Ok(None);
    }
    let mut path =
        parse_location_path(argument.trim(), location.clone()).map_err(map_path_failure)?;
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
    Ok(Some(ValueExpression::CountLocationPath(path)))
}

fn compile_normalize_space_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Option<ValueExpression> {
    let expression = expression.trim();
    let argument = ["normalize-space(", "fn:normalize-space("]
        .iter()
        .find_map(|prefix| {
            expression
                .strip_prefix(prefix)
                .and_then(|value| value.strip_suffix(')'))
        })?;
    if argument.trim().is_empty() || !has_balanced_parentheses(argument) {
        return None;
    }
    let Ok(mut path) = parse_location_path(argument.trim(), location.clone()) else {
        return None;
    };
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
    Some(ValueExpression::NormalizedStringPath(path))
}

fn compile_integral_function_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ValueExpression>, CompileFailure> {
    let Some((function, argument)) =
        crate::xpath::constant_numeric_experiment::integral_function_call(expression)
    else {
        return Ok(None);
    };
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
    Ok(Some(ValueExpression::IntegralFunctionPath {
        function,
        path,
    }))
}

fn compile_number_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<LocationPath>, CompileFailure> {
    let Some(argument) =
        crate::xpath::constant_numeric_experiment::number_function_call(expression)
    else {
        return Ok(None);
    };
    let argument = if argument.is_empty() { "." } else { argument };
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
    Ok(Some(path))
}

fn compile_string_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Option<LocationPath> {
    let expression = expression.trim();
    let argument = ["string(", "fn:string("]
        .iter()
        .find_map(|prefix| {
            expression
                .strip_prefix(prefix)
                .and_then(|value| value.strip_suffix(')'))
        })?
        .trim();
    if argument.is_empty() || !has_balanced_parentheses(argument) {
        return None;
    }
    let mut path = parse_location_path(argument, location.clone()).ok()?;
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
    Some(path)
}

fn compile_name_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Option<LocationPath> {
    let expression = expression.trim();
    let argument = ["name(", "fn:name("]
        .iter()
        .find_map(|prefix| {
            expression
                .strip_prefix(prefix)
                .and_then(|value| value.strip_suffix(')'))
        })?
        .trim();
    if argument.is_empty() || !has_balanced_parentheses(argument) {
        return None;
    }
    let mut path = parse_location_path(argument, location.clone())
        .or_else(|failure| match failure {
            PathFailure::Unsupported { .. } if argument.contains(':') => {
                parse_qualified_child_path(argument, location.clone(), |prefix| {
                    namespace_for_prefix(document, element, prefix).map(str::to_owned)
                })
            }
            failure => Err(failure),
        })
        .ok()?;
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
    Some(path)
}

fn compile_expanded_name_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    function: &str,
    location: &SourceLocation,
) -> Option<Result<LocationPath, CompileFailure>> {
    let expression = expression.trim();
    let argument = [format!("{function}("), format!("fn:{function}(")]
        .iter()
        .find_map(|prefix| {
            expression
                .strip_prefix(prefix)
                .and_then(|value| value.strip_suffix(')'))
        })?
        .trim();
    if argument.is_empty() || !has_balanced_parentheses(argument) {
        return None;
    }
    let path = match parse_location_path(argument, location.clone()) {
        Ok(path) => Ok(path),
        Err(PathFailure::Unsupported { .. }) if argument.contains(':') => {
            parse_qualified_child_path(argument, location.clone(), |prefix| {
                namespace_for_prefix(document, element, prefix).map(str::to_owned)
            })
        }
        Err(failure) => Err(failure),
    };
    Some(
        path.map(|mut path| {
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
            path
        })
        .map_err(map_path_failure),
    )
}

fn parse_literal_variable_concat(expression: &str) -> Option<(String, String)> {
    let arguments = expression
        .trim()
        .strip_prefix("concat(")?
        .strip_suffix(')')?;
    let (literal, variable) = arguments.rsplit_once(',')?;
    let literal = literal.trim();
    let literal = literal
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .or_else(|| {
            literal
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
        })?;
    let variable = variable.trim().strip_prefix('$')?;
    is_ascii_ncname(variable).then(|| (literal.to_owned(), variable.to_owned()))
}

fn root_argument(expression: &str) -> Option<&str> {
    expression
        .trim()
        .strip_prefix("root(")?
        .strip_suffix(')')
        .map(str::trim)
}

fn compile_root_value(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<ValueExpression>, CompileFailure> {
    if let Some((reference, descendant_local)) = parse_generated_document_root(expression) {
        return Ok(Some(ValueExpression::GeneratedDocumentRootIdentity(
            crate::xslt::golden_semantics_experiment::DocumentRootReference {
                base: location.resource.clone(),
                reference: reference.to_owned(),
                descendant_local: descendant_local.map(str::to_owned),
            },
        )));
    }
    if let Some((variable, descendant_local)) = parse_generated_temporary_root(expression) {
        return Ok(Some(ValueExpression::GeneratedTemporaryRootIdentity {
            variable: variable.to_owned(),
            descendant_local: descendant_local.map(str::to_owned),
        }));
    }
    if let Some(argument) = generated_root_argument(expression) {
        return parse_location_path(argument, location.clone())
            .map(ValueExpression::GeneratedRootIdentity)
            .map(Some)
            .map_err(map_path_failure);
    }
    if let Some(argument) = generated_node_argument(expression) {
        return parse_location_path(argument, location.clone())
            .map(ValueExpression::GeneratedNodeIdentity)
            .map(Some)
            .map_err(map_path_failure);
    }
    root_argument(expression)
        .map(|argument| compile_root_expression(document, element, argument, location))
        .transpose()
}

fn generated_node_argument(expression: &str) -> Option<&str> {
    let argument = expression
        .trim()
        .strip_prefix("generate-id(")?
        .strip_suffix(')')?
        .trim();
    (!argument.is_empty() && has_balanced_parentheses(argument)).then_some(argument)
}

fn has_balanced_parentheses(expression: &str) -> bool {
    let mut depth = 0_usize;
    let mut quote = None;
    for byte in expression.bytes() {
        if let Some(expected) = quote {
            if byte == expected {
                quote = None;
            }
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => depth += 1,
            b')' if depth == 0 => return false,
            b')' => depth -= 1,
            _ => {}
        }
    }
    depth == 0 && quote.is_none()
}

fn is_zero_argument_function(expression: &str, name: &str) -> bool {
    expression
        .trim()
        .strip_prefix(name)
        .is_some_and(|remainder| remainder.trim_start_matches([' ', '\t', '\r', '\n']) == "()")
}

pub(super) fn generated_root_argument(expression: &str) -> Option<&str> {
    expression
        .trim()
        .strip_prefix("generate-id(root(")?
        .strip_suffix("))")
        .map(str::trim)
}

fn compile_root_expression(
    document: &Document,
    element: NodeId,
    argument: &str,
    location: &SourceLocation,
) -> Result<ValueExpression, CompileFailure> {
    if let Some(variable) = argument.strip_prefix('$') {
        if !is_ascii_ncname(variable) {
            return Err(invalid(
                "XPST0003",
                format!("invalid variable reference in root(): {argument}"),
                location,
            ));
        }
        return Ok(ValueExpression::RootVariable(variable.to_owned()));
    }
    let path = match parse_location_path(argument, location.clone()) {
        Ok(path) => Ok(path),
        Err(PathFailure::Unsupported { .. }) if argument.contains(':') => {
            parse_qualified_child_path(argument, location.clone(), |prefix| {
                namespace_for_prefix(document, element, prefix).map(str::to_owned)
            })
        }
        Err(failure) => Err(failure),
    };
    path.map(ValueExpression::RootPath)
        .map_err(map_path_failure)
}

#[cfg(test)]
mod tests {
    use super::{
        ValueCompatibilityMode, ValueExpression, ValueStaticContext, compile_binary_numeric_node,
        compile_binary_numeric_path,
    };
    use crate::xdm::owned_tree_experiment::SourceLocation;

    #[test]
    fn compiles_mixed_path_literal_arithmetic_tree() {
        let location = SourceLocation {
            resource: "urn:fastxslt:test".to_owned(),
            span: 0..1,
        };
        for expression in [
            "n3+5",
            "(n3+5)*(3)",
            "(n2)+2",
            "((n2)+2)*(n1 - 6)",
            "(n4 - n2)",
            "-(4-6)",
            "((n3+5)*(3)+(((n2)+2)*(n1 - 6)))-(n4 - n2)",
            "((((((n3+5)*(3)+(((n2)+2)*(n1 - 6)))-(n4 - n2))+(-(4-6)))))",
        ] {
            assert!(
                compile_binary_numeric_node(expression, &location, false).is_some(),
                "failed to compile {expression}; split={:?}",
                crate::xpath::binary_numeric_experiment::split_paths(expression)
            );
        }

        assert!(compile_binary_numeric_node("n2+$offset", &location, true).is_some());
        assert!(compile_binary_numeric_node("n2+$offset", &location, false).is_none());
        assert!(matches!(
            compile_binary_numeric_path(
                "9876543210",
                &location,
                ValueStaticContext {
                    compatibility: ValueCompatibilityMode::Xslt10,
                }
            ),
            Some(ValueExpression::BinaryNumeric(_))
        ));
        for expression in [
            "100-n6 -4-n1 -1-11",
            "100-$anum -5-15-$anum",
            "$anum*5-4*n2+n6*n1 -n3*3",
        ] {
            assert!(
                compile_binary_numeric_node(expression, &location, true).is_some(),
                "failed to compile XSLT 1.0 arithmetic: {expression}"
            );
        }
    }
}
