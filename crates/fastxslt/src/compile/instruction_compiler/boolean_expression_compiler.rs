//! Private compilation of instruction-local `XPath` boolean expressions.

use crate::compile::golden_stylesheet_experiment::CompileFailure;
use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xpath::constant_numeric_experiment::{self, ConstantNumericFailure};
use crate::xpath::path_experiment::parse_location_path;
use crate::xslt::golden_semantics_experiment::{
    BooleanExpression, DocumentRootReference, EqualityTest, FocusComparison, StringComparison,
    Xslt10AncestorFilter, Xslt10ContextTranslateStartsWith,
};

use super::{
    conditional_expression_compiler, invalid, is_ascii_ncname, map_path_failure,
    parse_context_focus_equality, parse_context_focus_operand, parse_generated_document_root,
    parse_generated_temporary_root, unsupported, xpath_string_literal,
};

pub(super) fn compile(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
    comparison: StringComparison,
    xslt10_compatibility: bool,
) -> Result<BooleanExpression, CompileFailure> {
    if expression.trim() == "()" {
        return Ok(BooleanExpression::Constant(false));
    }
    let parsed = strip_enclosing_parentheses(expression.trim());
    if let Some(conditional) =
        conditional_expression_compiler::parse_integer_conditional(parsed, location)?
    {
        return Ok(BooleanExpression::ConditionalInteger(Box::new(conditional)));
    }
    if let Some(variable) = parsed
        .strip_prefix("boolean(")
        .and_then(|value| value.strip_suffix(')'))
        .and_then(|value| value.trim().strip_prefix('$'))
        && is_ascii_ncname(variable)
    {
        return Ok(BooleanExpression::VariableEffectiveBooleanValue(
            variable.to_owned(),
        ));
    }
    if let Some(language) = crate::xpath::language_experiment::parse_literal(parsed) {
        return Ok(BooleanExpression::ContextLanguageMatches(language));
    }
    if let Some(composition) = compile_composition(
        document,
        element,
        parsed,
        location,
        comparison,
        xslt10_compatibility,
    ) {
        return composition;
    }
    if let Some(identity) = compile_identity_test(parsed, location)? {
        return Ok(identity);
    }
    if is_context_position_not_equal_size(parsed) {
        return Ok(BooleanExpression::ContextPositionNotEqualSize(
            location.clone(),
        ));
    }
    if let Some((left, right)) = parse_context_focus_equality(parsed) {
        return Ok(BooleanExpression::ContextFocusEquals {
            left,
            right,
            location: location.clone(),
        });
    }
    if let Some((left, operator, right)) = parse_context_focus_comparison(parsed) {
        return Ok(BooleanExpression::ContextFocusCompares {
            left,
            operator,
            right,
            location: location.clone(),
        });
    }
    if let Some((divisor, remainder)) = parse_context_position_modulo_equality(parsed) {
        return Ok(BooleanExpression::ContextPositionModuloEquals {
            divisor,
            remainder,
            location: location.clone(),
        });
    }
    if let Some(count) = compile_count_path_equality(parsed, location)? {
        return Ok(count);
    }
    if xslt10_compatibility && parsed.trim_start().starts_with("key(") {
        return super::value_expression_compiler::compile_xslt10_literal_key_lookup(
            document, element, parsed, location,
        )
        .map(Box::new)
        .map(BooleanExpression::Xslt10KeyLookupEffectiveBooleanValue);
    }
    if let Some(expression) = compile_xslt10_special(parsed, location, xslt10_compatibility) {
        return Ok(expression);
    }
    compile_scalar(
        parsed,
        expression,
        location,
        comparison,
        xslt10_compatibility,
    )
}

fn compile_composition(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
    comparison: StringComparison,
    xslt10_compatibility: bool,
) -> Option<Result<BooleanExpression, CompileFailure>> {
    let (left, right, is_and) = split_top_level_or(expression)
        .map(|(left, right)| (left, Some(right), false))
        .or_else(|| split_top_level_and(expression).map(|(left, right)| (left, Some(right), true)))
        .or_else(|| {
            expression
                .strip_prefix("not(")
                .and_then(|inner| inner.strip_suffix(')'))
                .map(|inner| (inner, None, false))
        })?;
    let left = compile(
        document,
        element,
        left,
        location,
        comparison,
        xslt10_compatibility,
    );
    Some(match right {
        None => left.map(Box::new).map(BooleanExpression::Not),
        Some(right) => left.and_then(|left| {
            compile(
                document,
                element,
                right,
                location,
                comparison,
                xslt10_compatibility,
            )
            .map(|right| {
                let left = Box::new(left);
                let right = Box::new(right);
                if is_and {
                    BooleanExpression::And { left, right }
                } else {
                    BooleanExpression::Or { left, right }
                }
            })
        }),
    })
}

fn compile_xslt10_special(
    expression: &str,
    location: &SourceLocation,
    xslt10_compatibility: bool,
) -> Option<BooleanExpression> {
    if let Some((numeric_variable, nodes_variable)) =
        parse_xslt10_variable_less_than_node_count(expression, xslt10_compatibility)
    {
        return Some(BooleanExpression::Xslt10VariableLessThanNodeCount {
            numeric_variable: numeric_variable.to_owned(),
            nodes_variable: nodes_variable.to_owned(),
        });
    }
    if xslt10_compatibility
        && expression.split_whitespace().collect::<String>()
            == "descendant::*[name()=name(current())]|following::*[name()=name(current())]"
    {
        return Some(BooleanExpression::Xslt10DescendantOrFollowingSameNameAsCurrent);
    }
    if let Some(position_variable) =
        parse_xslt10_prior_descendant_same_name(expression, xslt10_compatibility)
    {
        return Some(BooleanExpression::Xslt10PriorDescendantSameNameAsCurrent(
            position_variable,
        ));
    }
    if let Some((position_variable, name_variable)) =
        parse_xslt10_prior_descendant_variable_name(expression, xslt10_compatibility)
    {
        return Some(BooleanExpression::Xslt10PriorDescendantSameNameAsVariable {
            position_variable,
            name_variable,
        });
    }
    if let Some((parent_name_variable, position_variable)) =
        parse_xslt10_prior_child_of_variable_named_elements(expression, xslt10_compatibility)
    {
        return Some(
            BooleanExpression::Xslt10PriorChildOfVariableNamedElementsSameNameAsCurrent {
                parent_name_variable,
                position_variable,
            },
        );
    }
    if xslt10_compatibility
        && let Some(value) =
            constant_numeric_experiment::fold_xslt10_nested_string_number_equality(expression)
    {
        return Some(BooleanExpression::Constant(value));
    }
    if let Some(expression) =
        compile_xslt10_child_attribute_variable_path(expression, xslt10_compatibility)
    {
        return Some(expression);
    }
    if let Some(variable) =
        parse_xslt10_context_node_set_variable_equality(expression, xslt10_compatibility)
    {
        return Some(BooleanExpression::Xslt10ContextNodeSetEqualsVariable {
            variable: variable.to_owned(),
            location: location.clone(),
        });
    }
    if let Some(filter) = compile_xslt10_ancestor_filter(expression, location, xslt10_compatibility)
    {
        return Some(BooleanExpression::Xslt10AncestorFilter(Box::new(filter)));
    }
    if let Some(divisor) =
        parse_xslt10_context_position_modulo_variable(expression, xslt10_compatibility)
    {
        return Some(BooleanExpression::Xslt10ContextPositionModuloVariable {
            divisor,
            location: location.clone(),
        });
    }
    if let Some(comparison) =
        compile_xslt10_variable_string_length_comparison(expression, xslt10_compatibility)
    {
        return Some(comparison);
    }
    if let Some(variable) =
        parse_xslt10_variable_string_length_boolean(expression, xslt10_compatibility)
    {
        return Some(BooleanExpression::Xslt10VariableStringLength(variable));
    }
    compile_xslt10_variable_numeric_comparison(expression, xslt10_compatibility)
        .or_else(|| compile_xslt10_context_translate_starts_with(expression, xslt10_compatibility))
}

fn parse_xslt10_prior_descendant_same_name(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<String> {
    let expression = xslt10_compatibility.then(|| expression.trim())?;
    let predicate = expression
        .strip_prefix('/')
        .unwrap_or(expression)
        .strip_prefix("descendant::*[")?
        .strip_suffix(']')?;
    let (position, name) = predicate.split_once("and")?;
    let (position, position_variable) = position.split_once('<')?;
    let position_variable = position_variable.trim().strip_prefix('$')?;
    let (candidate_name, current_name) = name.split_once('=')?;
    (position.trim() == "position()"
        && is_ascii_ncname(position_variable)
        && candidate_name.trim() == "name()"
        && current_name.trim() == "name(current())")
        .then(|| position_variable.to_owned())
}

fn parse_xslt10_prior_descendant_variable_name(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<(String, String)> {
    let expression = xslt10_compatibility.then(|| expression.trim())?;
    let predicate = expression
        .strip_prefix('/')
        .unwrap_or(expression)
        .strip_prefix("descendant::*[")?
        .strip_suffix(']')?;
    let (position, name) = predicate.split_once("and")?;
    let (position, position_variable) = position.split_once('<')?;
    let position_variable = position_variable.trim().strip_prefix('$')?;
    let (candidate_name, name_variable) = name.split_once('=')?;
    let name_variable = name_variable.trim().strip_prefix('$')?;
    (position.trim() == "position()"
        && is_ascii_ncname(position_variable)
        && candidate_name.trim() == "name()"
        && is_ascii_ncname(name_variable))
    .then(|| (position_variable.to_owned(), name_variable.to_owned()))
}

fn parse_xslt10_prior_child_of_variable_named_elements(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<(String, String)> {
    let expression = xslt10_compatibility.then(|| expression.trim())?;
    let (parents, predicate) = expression.rsplit_once("/*[")?;
    let parent_name_variable = parents
        .strip_prefix("//*[name()")?
        .trim_start()
        .strip_prefix('=')?
        .trim_start()
        .strip_prefix('$')?
        .strip_suffix(']')?
        .trim();
    let predicate = predicate.strip_suffix(']')?;
    let (position, name) = predicate.split_once("and")?;
    let (position, position_variable) = position.split_once('<')?;
    let position_variable = position_variable.trim().strip_prefix('$')?;
    let (candidate_name, current_name) = name.split_once('=')?;
    (is_ascii_ncname(parent_name_variable)
        && position.trim() == "position()"
        && is_ascii_ncname(position_variable)
        && candidate_name.trim() == "name()"
        && current_name.trim() == "name(current())")
        .then(|| {
            (
                parent_name_variable.to_owned(),
                position_variable.to_owned(),
            )
        })
}

fn parse_xslt10_variable_less_than_node_count(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<(&str, &str)> {
    let (left, right) = xslt10_compatibility.then(|| expression.split_once('<'))??;
    let numeric_variable = left.trim().strip_prefix('$')?;
    let nodes_variable = right
        .trim()
        .strip_prefix("count(")?
        .strip_suffix(')')?
        .trim()
        .strip_prefix('$')?;
    (is_ascii_ncname(numeric_variable) && is_ascii_ncname(nodes_variable))
        .then_some((numeric_variable, nodes_variable))
}

fn compile_xslt10_ancestor_filter(
    expression: &str,
    location: &SourceLocation,
    xslt10_compatibility: bool,
) -> Option<Xslt10AncestorFilter> {
    let predicates = xslt10_compatibility
        .then(|| expression.trim().strip_prefix("ancestor::*["))??
        .strip_suffix(']')?;
    let (first, second) = predicates.split_once("][")?;
    if second.contains("][") {
        return None;
    }
    let (position, attribute, value, require_absent_text_child) =
        if let Ok(position) = first.trim().parse::<usize>() {
            let attribute = second.trim().strip_prefix('@')?;
            (Some(position), attribute, None, false)
        } else {
            let (attribute, value) = first.trim().strip_prefix('@')?.split_once('=')?;
            if second.trim() != "not(text())" {
                return None;
            }
            (
                None,
                attribute.trim(),
                Some(xpath_string_literal(value.trim())?.to_owned()),
                true,
            )
        };
    if position == Some(0) || !is_ascii_ncname(attribute) {
        return None;
    }
    Some(Xslt10AncestorFilter {
        position,
        attribute: attribute.to_owned(),
        value,
        require_absent_text_child,
        location: location.clone(),
    })
}

fn parse_xslt10_context_node_set_variable_equality(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<&str> {
    let (left, right) = xslt10_compatibility.then(|| expression.split_once('='))??;
    let variable = right.trim().strip_prefix('$')?;
    (left.trim() == "." && is_ascii_ncname(variable)).then_some(variable)
}

fn compile_xslt10_child_attribute_variable_path(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<BooleanExpression> {
    let expression = xslt10_compatibility.then(|| {
        expression
            .trim()
            .strip_prefix("./")
            .unwrap_or(expression.trim())
    })?;
    let (child, predicate) = expression.split_once("[@")?;
    let predicate = predicate.strip_suffix(']')?;
    let (attribute, value) = predicate.split_once('=')?;
    let variable = value
        .trim()
        .strip_prefix("string(")?
        .strip_suffix(')')?
        .trim()
        .strip_prefix('$')?;
    if !is_ascii_ncname(child) || !is_ascii_ncname(attribute) || !is_ascii_ncname(variable) {
        return None;
    }
    Some(BooleanExpression::Xslt10ChildAttributeVariableEquals {
        child: ExpandedName {
            namespace: None,
            local: child.to_owned(),
        },
        attribute: ExpandedName {
            namespace: None,
            local: attribute.to_owned(),
        },
        variable: variable.to_owned(),
    })
}

fn parse_xslt10_variable_string_length_boolean(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<String> {
    let variable = xslt10_compatibility
        .then(|| {
            expression
                .trim()
                .strip_prefix("string-length($")?
                .strip_suffix(')')
        })??
        .trim();
    is_ascii_ncname(variable).then(|| variable.to_owned())
}

fn parse_xslt10_context_position_modulo_variable(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<String> {
    let variable = xslt10_compatibility
        .then(|| expression.trim().strip_prefix("position() mod $"))??
        .trim();
    is_ascii_ncname(variable).then(|| variable.to_owned())
}

fn compile_xslt10_context_translate_starts_with(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<BooleanExpression> {
    let arguments = xslt10_compatibility.then(|| {
        expression
            .trim()
            .strip_prefix("starts-with(")?
            .strip_suffix(')')
    })??;
    let arguments = crate::xpath::static_string_experiment::split_arguments(arguments, 2)?;
    let [translated, prefix] = arguments.as_slice() else {
        return None;
    };
    let translated = translated
        .trim()
        .strip_prefix("translate(")?
        .strip_suffix(')')?;
    let translated = crate::xpath::static_string_experiment::split_arguments(translated, 3)?;
    let [context, search, replacement] = translated.as_slice() else {
        return None;
    };
    if context.trim() != "." {
        return None;
    }
    Some(BooleanExpression::Xslt10ContextTranslateStartsWith(
        Xslt10ContextTranslateStartsWith {
            search: xpath_string_literal(search)?.to_owned(),
            replacement: xpath_string_literal(replacement)?.to_owned(),
            prefix: xpath_string_literal(prefix)?.to_owned(),
        },
    ))
}

fn compile_scalar(
    parsed: &str,
    expression: &str,
    location: &SourceLocation,
    comparison: StringComparison,
    xslt10_compatibility: bool,
) -> Result<BooleanExpression, CompileFailure> {
    if let Some(comparison) =
        compile_xslt10_context_comparison(parsed, location, xslt10_compatibility)
    {
        return Ok(comparison);
    }
    if let Some(comparison) = compile_xslt10_non_finite_comparison(parsed, xslt10_compatibility) {
        return Ok(comparison);
    }
    parse_scalar(
        parsed,
        expression,
        location,
        comparison,
        xslt10_compatibility,
    )
}

fn compile_xslt10_non_finite_comparison(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<BooleanExpression> {
    xslt10_compatibility
        .then(|| {
            constant_numeric_experiment::fold_xslt10_finite_comparison(expression).or_else(|| {
                constant_numeric_experiment::fold_xslt10_non_finite_comparison(expression)
            })
        })?
        .map(BooleanExpression::Constant)
}

fn compile_xslt10_context_comparison(
    expression: &str,
    location: &SourceLocation,
    xslt10_compatibility: bool,
) -> Option<BooleanExpression> {
    if !xslt10_compatibility {
        return None;
    }
    if is_xslt10_context_number_nan_test(expression) {
        return Some(BooleanExpression::Xslt10ContextNumberIsNaN);
    }
    let (left, right, equal) = parse_xslt10_source_path_comparison(expression)?;
    let left = parse_location_path(left, location.clone()).ok()?;
    let right = parse_location_path(right, location.clone()).ok()?;
    Some(BooleanExpression::Xslt10SourcePathStringComparison {
        left,
        right: Box::new(right),
        equal,
    })
}

fn compile_count_path_equality(
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<BooleanExpression>, CompileFailure> {
    let Some((path, expected)) = parse_count_path_equality(expression) else {
        return Ok(None);
    };
    parse_location_path(path, location.clone())
        .map(|path| Some(BooleanExpression::CountPathEquals { path, expected }))
        .map_err(map_path_failure)
}

fn compile_xslt10_variable_numeric_comparison(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<BooleanExpression> {
    let (left, operator, right) =
        xslt10_compatibility.then(|| parse_xslt10_variable_numeric_comparison(expression))??;
    Some(BooleanExpression::Xslt10VariableNumericComparison {
        left: left.to_owned(),
        operator,
        right: right.to_owned(),
    })
}

fn compile_xslt10_variable_string_length_comparison(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<BooleanExpression> {
    let (string_variable, operator, numeric_variable) = xslt10_compatibility
        .then(|| parse_xslt10_variable_string_length_comparison(expression))??;
    Some(BooleanExpression::Xslt10VariableStringLengthComparison {
        string_variable: string_variable.to_owned(),
        operator,
        numeric_variable: numeric_variable.to_owned(),
    })
}

fn parse_xslt10_variable_string_length_comparison(
    expression: &str,
) -> Option<(&str, FocusComparison, &str)> {
    for (token, operator) in [
        ("!=", FocusComparison::NotEqual),
        (">=", FocusComparison::GreaterThanOrEqual),
        ("<=", FocusComparison::LessThanOrEqual),
        (">", FocusComparison::GreaterThan),
        ("<", FocusComparison::LessThan),
    ] {
        let Some((left, right)) = expression.split_once(token) else {
            continue;
        };
        if left.contains(['=', '!', '<', '>']) || right.contains(['=', '!', '<', '>']) {
            return None;
        }
        if let Some(string_variable) = parse_variable_string_length_operand(left) {
            let numeric_variable = parse_variable_operand(right)?;
            return Some((string_variable, operator, numeric_variable));
        }
        if let Some(string_variable) = parse_variable_string_length_operand(right) {
            let numeric_variable = parse_variable_operand(left)?;
            return Some((
                string_variable,
                reverse_comparison(operator),
                numeric_variable,
            ));
        }
        return None;
    }
    None
}

fn parse_variable_string_length_operand(expression: &str) -> Option<&str> {
    let variable = expression
        .trim()
        .strip_prefix("string-length(")?
        .strip_suffix(')')?
        .trim();
    parse_variable_operand(variable)
}

fn parse_variable_operand(expression: &str) -> Option<&str> {
    let variable = expression.trim().strip_prefix('$')?;
    is_ascii_ncname(variable).then_some(variable)
}

const fn reverse_comparison(operator: FocusComparison) -> FocusComparison {
    match operator {
        FocusComparison::NotEqual => FocusComparison::NotEqual,
        FocusComparison::LessThan => FocusComparison::GreaterThan,
        FocusComparison::LessThanOrEqual => FocusComparison::GreaterThanOrEqual,
        FocusComparison::GreaterThan => FocusComparison::LessThan,
        FocusComparison::GreaterThanOrEqual => FocusComparison::LessThanOrEqual,
    }
}

fn parse_xslt10_variable_numeric_comparison(
    expression: &str,
) -> Option<(&str, FocusComparison, &str)> {
    for (token, operator) in [
        (">=", FocusComparison::GreaterThanOrEqual),
        ("<=", FocusComparison::LessThanOrEqual),
        (">", FocusComparison::GreaterThan),
        ("<", FocusComparison::LessThan),
    ] {
        let Some((left, right)) = expression.split_once(token) else {
            continue;
        };
        if left.contains(['=', '!', '<', '>']) || right.contains(['=', '!', '<', '>']) {
            return None;
        }
        return Some((
            parse_variable_operand(left)?,
            operator,
            parse_variable_operand(right)?,
        ));
    }
    None
}

pub(super) fn parse_xslt10_source_path_comparison(expression: &str) -> Option<(&str, &str, bool)> {
    let bytes = expression.as_bytes();
    let mut quote = None;
    let mut parentheses = 0_usize;
    let mut brackets = 0_usize;
    let mut index = 0_usize;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(expected) = quote {
            if byte == expected {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => parentheses += 1,
            b')' => parentheses = parentheses.saturating_sub(1),
            b'[' => brackets += 1,
            b']' => brackets = brackets.saturating_sub(1),
            b'=' if parentheses == 0 && brackets == 0 => {
                return path_comparison_parts(expression, index, 1, true);
            }
            b'!' if parentheses == 0 && brackets == 0 && bytes.get(index + 1) == Some(&b'=') => {
                return path_comparison_parts(expression, index, 2, false);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn path_comparison_parts(
    expression: &str,
    index: usize,
    operator_len: usize,
    equal: bool,
) -> Option<(&str, &str, bool)> {
    let left = expression[..index].trim();
    let right = expression[index + operator_len..].trim();
    (!left.is_empty() && !right.is_empty() && !left.starts_with('$') && !right.starts_with('$'))
        .then_some((left, right, equal))
}

fn compile_identity_test(
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<BooleanExpression>, CompileFailure> {
    if let Some((left, right)) = parse_document_root_identity_test(expression) {
        return Ok(Some(BooleanExpression::DocumentRootIdentityEqual {
            left: DocumentRootReference {
                base: location.resource.clone(),
                reference: left.0.to_owned(),
                descendant_local: left.1.map(str::to_owned),
            },
            right: DocumentRootReference {
                base: location.resource.clone(),
                reference: right.0.to_owned(),
                descendant_local: right.1.map(str::to_owned),
            },
        }));
    }
    if let Some((variable, descendant_local)) = parse_temporary_root_identity_test(expression) {
        return Ok(Some(BooleanExpression::TemporaryRootIdentityEqual {
            variable: variable.to_owned(),
            descendant_local: descendant_local.to_owned(),
        }));
    }
    if let Some((path, variable)) = parse_generated_root_identity_test(expression) {
        return Ok(Some(BooleanExpression::RootIdentityEqualsVariable {
            path: parse_location_path(path, location.clone()).map_err(map_path_failure)?,
            variable: variable.to_owned(),
        }));
    }
    let Some((left, right)) = generated_node_identity_test(expression) else {
        return Ok(None);
    };
    Ok(Some(BooleanExpression::NodeIdentityEqual {
        left: parse_location_path(left, location.clone()).map_err(map_path_failure)?,
        right: parse_location_path(right, location.clone()).map_err(map_path_failure)?,
    }))
}

fn split_top_level_or(expression: &str) -> Option<(&str, &str)> {
    split_top_level_boolean_operator(expression, b"or")
}

fn split_top_level_and(expression: &str) -> Option<(&str, &str)> {
    split_top_level_boolean_operator(expression, b"and")
}

fn split_top_level_boolean_operator<'a>(
    expression: &'a str,
    operator: &[u8],
) -> Option<(&'a str, &'a str)> {
    let bytes = expression.as_bytes();
    let mut quote = None;
    let mut parentheses = 0_usize;
    let mut brackets = 0_usize;
    let mut index = 0_usize;
    while index + operator.len() <= bytes.len() {
        let byte = bytes[index];
        if let Some(expected) = quote {
            if byte == expected {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => parentheses += 1,
            b')' => parentheses = parentheses.saturating_sub(1),
            b'[' => brackets += 1,
            b']' => brackets = brackets.saturating_sub(1),
            _ if bytes[index..].starts_with(operator) && parentheses == 0 && brackets == 0 => {
                let left_boundary = index == 0 || !is_xpath_name_byte(bytes[index - 1]);
                let right_boundary = index + operator.len() == bytes.len()
                    || !is_xpath_name_byte(
                        bytes
                            .get(index + operator.len())
                            .copied()
                            .unwrap_or_default(),
                    );
                if left_boundary && right_boundary {
                    let left = expression[..index].trim();
                    let right = expression[index + operator.len()..].trim();
                    if !left.is_empty() && !right.is_empty() {
                        return Some((left, right));
                    }
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn parse_context_focus_comparison(
    expression: &str,
) -> Option<(
    crate::xslt::golden_semantics_experiment::FocusEqualityOperand,
    FocusComparison,
    crate::xslt::golden_semantics_experiment::FocusEqualityOperand,
)> {
    for (token, operator) in [
        ("!=", FocusComparison::NotEqual),
        (">=", FocusComparison::GreaterThanOrEqual),
        ("<=", FocusComparison::LessThanOrEqual),
        (">", FocusComparison::GreaterThan),
        ("<", FocusComparison::LessThan),
    ] {
        let Some((left, right)) = expression.split_once(token) else {
            continue;
        };
        if left.contains(['=', '!', '<', '>']) || right.contains(['=', '!', '<', '>']) {
            return None;
        }
        let left = parse_context_focus_operand(left.trim())?;
        let right = parse_context_focus_operand(right.trim())?;
        if matches!(
            left,
            crate::xslt::golden_semantics_experiment::FocusEqualityOperand::Static(_)
        ) && matches!(
            right,
            crate::xslt::golden_semantics_experiment::FocusEqualityOperand::Static(_)
        ) {
            return None;
        }
        return Some((left, operator, right));
    }
    None
}

fn is_xpath_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':')
}

fn parse_context_position_modulo_equality(expression: &str) -> Option<(usize, usize)> {
    let (left, right) = expression.split_once('=')?;
    if left.contains('=') || right.contains('=') {
        return None;
    }
    parse_context_position_modulo_operand(left.trim(), right.trim())
        .or_else(|| parse_context_position_modulo_operand(right.trim(), left.trim()))
}

fn parse_context_position_modulo_operand(modulo: &str, remainder: &str) -> Option<(usize, usize)> {
    let operands = modulo.strip_prefix("position()")?.trim();
    let divisor = operands.strip_prefix("mod")?.trim().parse::<usize>().ok()?;
    let remainder = remainder.parse::<usize>().ok()?;
    (divisor > 0 && remainder < divisor).then_some((divisor, remainder))
}

fn parse_scalar(
    parsed: &str,
    expression: &str,
    location: &SourceLocation,
    comparison: StringComparison,
    xslt10_compatibility: bool,
) -> Result<BooleanExpression, CompileFailure> {
    if parsed == "()" {
        return Ok(BooleanExpression::Constant(false));
    }
    if let Some(value) = parse_constant_string_boolean(parsed) {
        return Ok(BooleanExpression::Constant(value));
    }
    if let Some(length) = parse_context_string_length_equality(parsed) {
        return Ok(BooleanExpression::ContextStringLengthEquals(length));
    }
    if let Some(comparison) = compile_context_string_comparison(parsed) {
        return Ok(comparison);
    }
    if let Some(expression) = compile_path_boolean_scalar(parsed, location, comparison)? {
        return Ok(expression);
    }
    if let Some(comparison) = compile_xslt10_variable_literal(parsed, xslt10_compatibility) {
        return Ok(comparison);
    }
    if let Some((left, right)) = parse_variable_string_equality(parsed) {
        return Ok(BooleanExpression::VariableStringEquals {
            left: left.to_owned(),
            right: right.to_owned(),
            comparison,
        });
    }
    if parsed.starts_with('/') || parsed.starts_with('@') {
        return parse_location_path(parsed, location.clone())
            .map(BooleanExpression::NodeExists)
            .map_err(map_path_failure);
    }
    if is_ascii_ncname(parsed) {
        return parse_location_path(parsed, location.clone())
            .map(BooleanExpression::NodeExists)
            .map_err(map_path_failure);
    }
    if let Some((variable, integer)) = parsed.split_once('=') {
        let variable = variable.trim().strip_prefix('$').unwrap_or_default();
        if is_ascii_ncname(variable) {
            if integer.trim() == "()" {
                return Ok(BooleanExpression::VariableEqualsEmptySequence(
                    variable.to_owned(),
                ));
            }
            let integer = integer
                .trim()
                .parse::<i64>()
                .map_err(|_| unsupported_boolean_expression(expression, location))?;
            return Ok(BooleanExpression::VariableEqualsInteger(EqualityTest {
                variable: variable.to_owned(),
                integer,
                xslt10_compatibility,
            }));
        }
    }
    if let Some(variable) = parsed.strip_prefix('$')
        && is_ascii_ncname(variable)
    {
        return Ok(BooleanExpression::VariableEffectiveBooleanValue(
            variable.to_owned(),
        ));
    }
    if !parsed.contains('=') && !parsed.contains('>') {
        let ordering =
            constant_numeric_experiment::compare(parsed, "0").map_err(|failure| match failure {
                ConstantNumericFailure::Invalid => invalid(
                    "FXXP0004",
                    format!("invalid conditional expression: {expression}"),
                    location,
                ),
                ConstantNumericFailure::Unsupported => {
                    unsupported_boolean_expression(expression, location)
                }
            })?;
        return Ok(BooleanExpression::Constant(!ordering.is_eq()));
    }
    let (left, right, greater_than) = if let Some((left, right)) = parsed.split_once('>') {
        (left, right, true)
    } else if let Some((left, right)) = parsed.split_once('=') {
        (left, right, false)
    } else {
        return Err(unsupported_boolean_expression(expression, location));
    };
    let ordering =
        constant_numeric_experiment::compare(left.trim(), right.trim()).map_err(|failure| {
            match failure {
                ConstantNumericFailure::Invalid => invalid(
                    "FXXP0004",
                    format!("invalid conditional expression: {expression}"),
                    location,
                ),
                ConstantNumericFailure::Unsupported => {
                    unsupported_boolean_expression(expression, location)
                }
            }
        })?;
    Ok(BooleanExpression::Constant(if greater_than {
        ordering.is_gt()
    } else {
        ordering.is_eq()
    }))
}

fn compile_context_string_comparison(expression: &str) -> Option<BooleanExpression> {
    let (literal, equal) = parse_context_string_comparison(expression)?;
    let equality = BooleanExpression::ContextStringEquals(literal.to_owned());
    Some(if equal {
        equality
    } else {
        BooleanExpression::Not(Box::new(equality))
    })
}

fn compile_path_boolean_scalar(
    expression: &str,
    location: &SourceLocation,
    comparison: StringComparison,
) -> Result<Option<BooleanExpression>, CompileFailure> {
    if let Some(expression) = parse_path_boolean_expression(expression, location, comparison)? {
        return Ok(Some(expression));
    }
    Ok((expression.contains('[') || expression.contains("::"))
        .then(|| parse_location_path(expression, location.clone()).ok())
        .flatten()
        .map(BooleanExpression::NodeExists))
}

fn compile_xslt10_variable_literal(
    expression: &str,
    xslt10_compatibility: bool,
) -> Option<BooleanExpression> {
    let (variable, literal, equal) =
        xslt10_compatibility.then(|| parse_variable_literal_equality(expression))??;
    Some(BooleanExpression::Xslt10VariableStringLiteralEquals {
        variable: variable.to_owned(),
        literal: literal.to_owned(),
        equal,
    })
}

fn parse_variable_literal_equality(expression: &str) -> Option<(&str, &str, bool)> {
    let (left, right, equal) = if let Some((left, right)) = expression.split_once("!=") {
        (left, right, false)
    } else {
        let (left, right) = expression.split_once('=')?;
        (left, right, true)
    };
    if left.contains('=') || right.contains(['=', '!']) {
        return None;
    }
    parse_variable_literal_operands(left, right)
        .or_else(|| parse_variable_literal_operands(right, left))
        .map(|(variable, literal)| (variable, literal, equal))
}

fn parse_variable_literal_operands<'a>(
    variable: &'a str,
    literal: &'a str,
) -> Option<(&'a str, &'a str)> {
    let variable = variable.trim().strip_prefix('$')?;
    is_ascii_ncname(variable)
        .then(|| xpath_string_literal(literal.trim()).map(|literal| (variable, literal)))
        .flatten()
}

fn is_context_position_not_equal_size(expression: &str) -> bool {
    let Some((left, right)) = expression.split_once("!=") else {
        return false;
    };
    matches!(
        (left.trim(), right.trim()),
        ("position()", "last()") | ("last()", "position()")
    )
}

fn parse_context_string_comparison(expression: &str) -> Option<(&str, bool)> {
    for (operator, equal) in [("!=", false), ("=", true)] {
        let Some((left, right)) = expression.split_once(operator) else {
            continue;
        };
        if left.contains(['=', '!']) || right.contains(['=', '!']) {
            return None;
        }
        let literal = if left.trim() == "." {
            xpath_string_literal(right.trim())
        } else if right.trim() == "." {
            xpath_string_literal(left.trim())
        } else {
            None
        };
        if let Some(literal) = literal {
            return Some((literal, equal));
        }
    }
    None
}

fn parse_context_string_length_equality(expression: &str) -> Option<usize> {
    let (left, right) = expression.split_once('=')?;
    (left.trim() == "string-length(.)")
        .then(|| right.trim().parse().ok())
        .flatten()
}

fn is_xslt10_context_number_nan_test(expression: &str) -> bool {
    if let Some(arguments) = expression
        .strip_prefix("contains(")
        .and_then(|value| value.strip_suffix(')'))
        && let Some((number, literal)) = arguments.split_once(',')
    {
        return number.trim() == "number(.)" && xpath_string_literal(literal.trim()) == Some("NaN");
    }
    let Some((left, right)) = expression.split_once('=') else {
        return false;
    };
    !left.contains('=')
        && !right.contains('=')
        && ((left.trim() == "string(number(.))"
            && xpath_string_literal(right.trim()) == Some("NaN"))
            || (right.trim() == "string(number(.))"
                && xpath_string_literal(left.trim()) == Some("NaN")))
}

fn parse_count_path_equality(expression: &str) -> Option<(&str, usize)> {
    let (left, right) = expression.split_once('=')?;
    if left.contains(['=', '!']) || right.contains('=') {
        return None;
    }
    parse_count_path_operand(left.trim(), right.trim())
        .or_else(|| parse_count_path_operand(right.trim(), left.trim()))
}

fn parse_count_path_operand<'a>(count: &'a str, integer: &str) -> Option<(&'a str, usize)> {
    let path = count.strip_prefix("count(")?.strip_suffix(')')?.trim();
    (!path.is_empty()).then_some((path, integer.parse().ok()?))
}

fn parse_path_boolean_expression(
    expression: &str,
    location: &SourceLocation,
    comparison: StringComparison,
) -> Result<Option<BooleanExpression>, CompileFailure> {
    if let Some((lexical, equal)) = parse_context_name_comparison(expression) {
        let equality = BooleanExpression::ContextNodeNameEquals {
            lexical: lexical.to_owned(),
            comparison,
        };
        return Ok(Some(if equal {
            equality
        } else {
            BooleanExpression::Not(Box::new(equality))
        }));
    }
    if let Some((path, value)) = parse_path_context_string_predicate(expression) {
        return parse_location_path(path, location.clone())
            .map(|path| {
                Some(BooleanExpression::NodeStringEquals {
                    path,
                    value: value.to_owned(),
                })
            })
            .map_err(map_path_failure);
    }
    if let Some((path, local)) = parse_unqualified_name_equality(expression) {
        return parse_location_path(path, location.clone())
            .map(|path| {
                Some(BooleanExpression::UnqualifiedNodeNameEquals {
                    path,
                    local: local.to_owned(),
                    comparison,
                })
            })
            .map_err(map_path_failure);
    }
    if let Some((path, value)) = parse_path_string_equality(expression) {
        return parse_location_path(path, location.clone())
            .map(|path| {
                Some(BooleanExpression::NodeStringEquals {
                    path,
                    value: value.to_owned(),
                })
            })
            .map_err(map_path_failure);
    }
    if let Some((path, value)) = parse_path_integer_less_than(expression) {
        return parse_location_path(path, location.clone())
            .map(|path| Some(BooleanExpression::NodeIntegerLessThan { path, value }))
            .map_err(map_path_failure);
    }
    Ok(None)
}

fn parse_context_name_comparison(expression: &str) -> Option<(&str, bool)> {
    for (operator, equal) in [("!=", false), ("=", true)] {
        let Some((left, right)) = expression.split_once(operator) else {
            continue;
        };
        if left.contains(['=', '!']) || right.contains(['=', '!']) {
            return None;
        }
        if let Some(lexical) = parse_context_name_operand(left.trim(), right.trim())
            .or_else(|| parse_context_name_operand(right.trim(), left.trim()))
        {
            return Some((lexical, equal));
        }
    }
    None
}

fn parse_context_name_operand<'a>(name: &str, literal: &'a str) -> Option<&'a str> {
    matches!(name, "name()" | "name(.)")
        .then(|| xpath_string_literal(literal))
        .flatten()
}

fn parse_constant_string_boolean(expression: &str) -> Option<bool> {
    if matches!(expression, "true()" | "false()") {
        return Some(expression == "true()");
    }
    if let Some(value) = xpath_string_literal(expression) {
        return Some(!value.is_empty());
    }
    let (left, right) = expression.split_once('=')?;
    Some(xpath_string_literal(left.trim())? == xpath_string_literal(right.trim())?)
}

fn parse_variable_string_equality(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    Some((
        variable_string_operand(left)?,
        variable_string_operand(right)?,
    ))
}

fn variable_string_operand(expression: &str) -> Option<&str> {
    let expression = expression.trim();
    let expression = expression
        .strip_prefix("string(")
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(expression)
        .trim();
    let variable = expression.strip_prefix('$')?;
    is_ascii_ncname(variable).then_some(variable)
}

fn parse_path_context_string_predicate(expression: &str) -> Option<(&str, &str)> {
    let (path, predicate) = expression.split_once('[')?;
    if path.trim().is_empty() || path.contains(']') {
        return None;
    }
    let predicate = predicate.strip_suffix(']')?;
    if predicate.contains(['[', ']']) {
        return None;
    }
    let (left, right) = predicate.split_once('=')?;
    (left.trim() == ".")
        .then(|| xpath_string_literal(right.trim()))
        .flatten()
        .map(|value| (path.trim(), value))
}

fn parse_path_integer_less_than(expression: &str) -> Option<(&str, i64)> {
    let (path, value) = expression.split_once('<')?;
    let path = path.trim();
    if path.is_empty() || path.contains(['=', '>', '<']) {
        return None;
    }
    Some((path, value.trim().parse().ok()?))
}

fn parse_unqualified_name_equality(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    let path = left.trim().strip_prefix("name(")?.strip_suffix(')')?.trim();
    let local = xpath_string_literal(right.trim())?;
    (!path.is_empty() && is_ascii_ncname(local)).then_some((path, local))
}

fn parse_path_string_equality(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    let left = left.trim();
    let right = right.trim();
    if let Some(value) = xpath_string_literal(right)
        && left != "."
        && !left.starts_with('$')
    {
        return Some((left, value));
    }
    let value = xpath_string_literal(left)?;
    (right != "." && !right.starts_with('$')).then_some((right, value))
}

fn parse_temporary_root_identity_test(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    let (left_variable, left_descendant) = parse_generated_temporary_root(left.trim())?;
    let (right_variable, right_descendant) = parse_generated_temporary_root(right.trim())?;
    if left_variable != right_variable || left_descendant.is_some() {
        return None;
    }
    Some((left_variable, right_descendant?))
}

type ParsedDocumentRootReference<'a> = (&'a str, Option<&'a str>);

fn parse_document_root_identity_test(
    expression: &str,
) -> Option<(
    ParsedDocumentRootReference<'_>,
    ParsedDocumentRootReference<'_>,
)> {
    let (left, right) = expression.split_once('=')?;
    Some((
        parse_generated_document_root(left.trim())?,
        parse_generated_document_root(right.trim())?,
    ))
}

fn parse_generated_root_identity_test(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    let variable = right.trim().strip_prefix('$')?;
    if !is_ascii_ncname(variable) {
        return None;
    }
    let path = left
        .trim()
        .strip_prefix("generate-id(root(")?
        .strip_suffix("))")?
        .trim();
    Some((path, variable))
}

pub(super) fn generated_node_identity_test(expression: &str) -> Option<(&str, &str)> {
    let (left, right) = expression.split_once('=')?;
    Some((
        parse_generated_node_argument(left.trim())?,
        parse_generated_node_argument(right.trim())?,
    ))
}

fn parse_generated_node_argument(expression: &str) -> Option<&str> {
    let argument = expression
        .strip_prefix("generate-id(")?
        .strip_suffix(')')?
        .trim();
    Some(if argument.is_empty() { "." } else { argument })
}

fn strip_enclosing_parentheses(mut expression: &str) -> &str {
    loop {
        let bytes = expression.as_bytes();
        if bytes.first() != Some(&b'(') || bytes.last() != Some(&b')') {
            return expression;
        }
        let mut depth = 0_usize;
        let mut encloses_all = true;
        for (index, byte) in bytes.iter().copied().enumerate() {
            match byte {
                b'(' => depth += 1,
                b')' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 && index + 1 != bytes.len() {
                        encloses_all = false;
                        break;
                    }
                }
                _ => {}
            }
        }
        if !encloses_all || depth != 0 {
            return expression;
        }
        expression = expression[1..expression.len() - 1].trim();
    }
}

fn unsupported_boolean_expression(expression: &str, location: &SourceLocation) -> CompileFailure {
    unsupported(
        "FXXP1002",
        format!("unsupported conditional expression: {expression}"),
        location,
    )
}
