//! Bounded default-decimal formatting shared by admitted XSLT 1.0 and 3.0 expressions.

use std::collections::BTreeMap;

use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::binary_numeric_experiment::ExactRational;
use crate::xpath::path_experiment::{LocationPath, parse_xslt10_location_path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FormatNumberExpression {
    number: Operand,
    picture: Operand,
    requested_format_lexical: Option<String>,
    requested_format_variable: Option<String>,
    requested_format: Option<ExpandedName>,
    static_namespaces: Vec<NamespaceBinding>,
    dynamic_formats: Vec<(ExpandedName, DecimalFormat)>,
    decimal_format: DecimalFormat,
    xslt10_compatibility: bool,
    location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DecimalFormat {
    pub(crate) decimal_separator: char,
    pub(crate) grouping_separator: char,
    pub(crate) infinity: String,
    pub(crate) minus_sign: char,
    pub(crate) nan: String,
    pub(crate) percent: char,
    pub(crate) per_mille: char,
    pub(crate) zero_digit: char,
    pub(crate) digit: char,
    pub(crate) pattern_separator: char,
}

impl Default for DecimalFormat {
    fn default() -> Self {
        Self {
            decimal_separator: '.',
            grouping_separator: ',',
            infinity: "Infinity".to_owned(),
            minus_sign: '-',
            nan: "NaN".to_owned(),
            percent: '%',
            per_mille: '‰',
            zero_digit: '0',
            digit: '#',
            pattern_separator: ';',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Operand {
    Literal(String),
    Variable(String),
    Path(LocationPath),
}

impl FormatNumberExpression {
    pub(crate) fn variable_names(&self) -> impl Iterator<Item = &str> {
        [&self.number, &self.picture]
            .into_iter()
            .filter_map(|operand| match operand {
                Operand::Variable(name) => Some(name.as_str()),
                Operand::Literal(_) | Operand::Path(_) => None,
            })
            .chain(self.requested_format_variable.as_deref())
    }

    pub(crate) fn number_path(&self) -> Option<&LocationPath> {
        match &self.number {
            Operand::Path(path) => Some(path),
            Operand::Literal(_) | Operand::Variable(_) => None,
        }
    }

    pub(crate) fn picture_path(&self) -> Option<&LocationPath> {
        match &self.picture {
            Operand::Path(path) => Some(path),
            Operand::Literal(_) | Operand::Variable(_) => None,
        }
    }

    pub(crate) fn set_default_decimal_format(&mut self, format: &DecimalFormat) {
        self.decimal_format.clone_from(format);
    }

    pub(crate) fn requested_format_lexical(&self) -> Option<&str> {
        self.requested_format_lexical.as_deref()
    }

    pub(crate) fn requested_format_variable(&self) -> Option<&str> {
        self.requested_format_variable.as_deref()
    }

    pub(crate) fn set_requested_format_variable(&mut self, name: String) {
        self.requested_format_variable = Some(name);
    }

    pub(crate) fn set_static_namespaces(&mut self, namespaces: Vec<NamespaceBinding>) {
        self.static_namespaces = namespaces;
    }

    pub(crate) fn set_dynamic_formats(&mut self, formats: Vec<(ExpandedName, DecimalFormat)>) {
        self.dynamic_formats = formats;
    }

    pub(crate) fn set_requested_format(&mut self, name: ExpandedName) {
        self.requested_format = Some(name);
    }

    pub(crate) fn requested_format(&self) -> Option<&ExpandedName> {
        self.requested_format.as_ref()
    }

    pub(crate) fn location(&self) -> &SourceLocation {
        &self.location
    }
}

#[cfg(feature = "workbench")]
impl FormatNumberExpression {
    pub(crate) fn known_owned_capacity_bytes(&self) -> usize {
        operand_capacity(&self.number)
            + operand_capacity(&self.picture)
            + self.decimal_format.infinity.capacity()
            + self.decimal_format.nan.capacity()
            + self
                .requested_format_lexical
                .as_ref()
                .map_or(0, String::capacity)
            + self
                .requested_format_variable
                .as_ref()
                .map_or(0, String::capacity)
            + self.requested_format.as_ref().map_or(0, |name| {
                name.local.capacity() + name.namespace.as_ref().map_or(0, String::capacity)
            })
            + self
                .static_namespaces
                .iter()
                .map(|binding| {
                    binding.prefix.as_ref().map_or(0, String::capacity)
                        + binding.namespace.capacity()
                })
                .sum::<usize>()
            + self
                .dynamic_formats
                .iter()
                .map(|(name, format)| {
                    name.local.capacity()
                        + name.namespace.as_ref().map_or(0, String::capacity)
                        + format.infinity.capacity()
                        + format.nan.capacity()
                })
                .sum::<usize>()
            + self.location.resource.capacity()
    }
}

#[cfg(feature = "workbench")]
fn operand_capacity(value: &Operand) -> usize {
    match value {
        Operand::Literal(value) | Operand::Variable(value) => value.capacity(),
        Operand::Path(path) => path.known_owned_capacity_bytes(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FormatNumberFailure {
    pub(crate) kind: FormatNumberFailureKind,
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormatNumberFailureKind {
    InvalidArity,
    InvalidSyntax,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FormatNumberEvaluationFailure {
    UnboundVariable(String),
    InvalidDecimalFormatName,
    InvalidPicture,
    Unsupported(FormatNumberUnsupported),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FormatNumberUnsupported {
    Number,
    FiniteFormatting,
}

impl FormatNumberUnsupported {
    pub(crate) fn detail(self) -> &'static str {
        match self {
            Self::Number => "numeric conversion",
            Self::FiniteFormatting => "finite decimal formatting",
        }
    }
}

#[cfg(test)]
pub(crate) fn parse(
    expression: &str,
    location: &SourceLocation,
) -> Result<FormatNumberExpression, FormatNumberFailure> {
    parse_with_path_operands(expression, location, false)
}

pub(crate) fn parse_with_path_operands(
    expression: &str,
    location: &SourceLocation,
    admit_xslt10_paths: bool,
) -> Result<FormatNumberExpression, FormatNumberFailure> {
    let arguments = expression
        .trim()
        .strip_prefix("format-number(")
        .and_then(|value| value.strip_suffix(')'))
        .ok_or_else(|| unsupported(expression, location))?;
    if let Some(arity) = top_level_argument_count(arguments)
        && !matches!(arity, 2 | 3)
    {
        return Err(FormatNumberFailure {
            kind: FormatNumberFailureKind::InvalidArity,
            detail: format!("format-number has invalid arity {arity}"),
            location: location.clone(),
        });
    }
    let (number, remainder) =
        split_top_level_comma(arguments).ok_or_else(|| unsupported(expression, location))?;
    let (picture, requested_format_lexical, requested_format_variable) =
        match split_top_level_comma(remainder) {
            Some((picture, format)) => {
                let format = format.trim();
                if let Some(name) = parse_static_string(format).filter(|name| !name.is_empty()) {
                    (picture, Some(name), None)
                } else if let Some(name) = variable(format) {
                    (picture, None, Some(name.to_owned()))
                } else {
                    return Err(unsupported(expression, location));
                }
            }
            None => (remainder, None, None),
        };
    let number = number.trim();
    if number.is_empty() {
        return Err(FormatNumberFailure {
            kind: FormatNumberFailureKind::InvalidSyntax,
            detail: "format-number requires a first argument expression".to_owned(),
            location: location.clone(),
        });
    }
    let number = parse_number(number, location, admit_xslt10_paths)
        .ok_or_else(|| unsupported(expression, location))?;
    let picture = parse_picture(picture.trim(), location, admit_xslt10_paths)
        .ok_or_else(|| unsupported(expression, location))?;
    Ok(FormatNumberExpression {
        number,
        picture,
        requested_format_lexical,
        requested_format_variable,
        requested_format: None,
        static_namespaces: Vec::new(),
        dynamic_formats: Vec::new(),
        decimal_format: DecimalFormat::default(),
        xslt10_compatibility: admit_xslt10_paths,
        location: location.clone(),
    })
}

#[cfg(test)]
pub(crate) fn evaluate(
    expression: &FormatNumberExpression,
    variables: &BTreeMap<String, AtomicValue>,
) -> Result<String, FormatNumberEvaluationFailure> {
    evaluate_with_path_values(expression, variables, None, None)
}

pub(crate) fn evaluate_with_path_values(
    expression: &FormatNumberExpression,
    variables: &BTreeMap<String, AtomicValue>,
    number_path_value: Option<&str>,
    picture_path_value: Option<&str>,
) -> Result<String, FormatNumberEvaluationFailure> {
    if expression.requested_format_lexical.is_some() && expression.requested_format.is_none() {
        return Err(FormatNumberEvaluationFailure::InvalidDecimalFormatName);
    }
    let number = resolve(&expression.number, variables, number_path_value)?;
    let number_is_expression = matches!(expression.number, Operand::Literal(_));
    let picture = resolve(&expression.picture, variables, picture_path_value)?;
    let dynamic_format = expression
        .requested_format_variable
        .as_deref()
        .map(|variable| {
            let lexical = variables
                .get(variable)
                .map(AtomicValue::lexical)
                .ok_or_else(|| {
                    FormatNumberEvaluationFailure::UnboundVariable(variable.to_owned())
                })?;
            let name = resolve_qname(lexical, &expression.static_namespaces)
                .ok_or(FormatNumberEvaluationFailure::InvalidDecimalFormatName)?;
            expression
                .dynamic_formats
                .iter()
                .find(|(candidate, _)| candidate == &name)
                .map(|(_, format)| format)
                .ok_or(FormatNumberEvaluationFailure::InvalidDecimalFormatName)
        })
        .transpose()?;
    format_decimal(
        number,
        number_is_expression,
        picture,
        dynamic_format.unwrap_or(&expression.decimal_format),
        expression.xslt10_compatibility,
    )
}

fn resolve_qname(lexical: &str, namespaces: &[NamespaceBinding]) -> Option<ExpandedName> {
    let lexical = lexical.trim();
    let (prefix, local) = lexical
        .split_once(':')
        .map_or((None, lexical), |(prefix, local)| (Some(prefix), local));
    if !is_ascii_ncname(local)
        || prefix.is_some_and(|prefix| !is_ascii_ncname(prefix))
        || local.contains(':')
    {
        return None;
    }
    let namespace = if let Some(prefix) = prefix {
        Some(
            namespaces
                .iter()
                .find(|binding| binding.prefix.as_deref() == Some(prefix))?
                .namespace
                .clone(),
        )
    } else {
        None
    };
    Some(ExpandedName {
        namespace,
        local: local.to_owned(),
    })
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    characters.next().is_some_and(|first| {
        (first.is_ascii_alphabetic() || first == '_')
            && characters.all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
            })
    })
}

fn resolve<'a>(
    operand: &'a Operand,
    variables: &'a BTreeMap<String, AtomicValue>,
    path_value: Option<&'a str>,
) -> Result<&'a str, FormatNumberEvaluationFailure> {
    match operand {
        Operand::Literal(value) => Ok(value),
        Operand::Variable(name) => variables
            .get(name)
            .map(AtomicValue::lexical)
            .ok_or_else(|| FormatNumberEvaluationFailure::UnboundVariable(name.clone())),
        Operand::Path(_) => path_value.ok_or(FormatNumberEvaluationFailure::Unsupported(
            FormatNumberUnsupported::Number,
        )),
    }
}

fn parse_number(
    expression: &str,
    location: &SourceLocation,
    admit_xslt10_paths: bool,
) -> Option<Operand> {
    if let Some(variable) = variable(expression) {
        return Some(Operand::Variable(variable.to_owned()));
    }
    if let Some(inner) = expression
        .strip_prefix("number(")
        .and_then(|value| value.strip_suffix(')'))
    {
        quoted(inner.trim()).map(|value| Operand::Literal(value.to_owned()))
    } else if admit_xslt10_paths && is_ascii_ncname(expression) {
        parse_xslt10_location_path(expression, location.clone())
            .ok()
            .map(Operand::Path)
    } else {
        Some(Operand::Literal(expression.to_owned()))
    }
}

fn parse_picture(
    expression: &str,
    location: &SourceLocation,
    admit_xslt10_paths: bool,
) -> Option<Operand> {
    if let Some(variable) = variable(expression) {
        return Some(Operand::Variable(variable.to_owned()));
    }
    if let Some(value) = quoted(expression) {
        return Some(Operand::Literal(value.to_owned()));
    }
    if admit_xslt10_paths && is_ascii_ncname(expression) {
        return parse_xslt10_location_path(expression, location.clone())
            .ok()
            .map(Operand::Path);
    }
    let arguments = expression
        .strip_prefix("substring-after(")?
        .strip_suffix(')')?;
    let (value, delimiter) = split_top_level_comma(arguments)?;
    let value = quoted(value.trim())?;
    let delimiter = quoted(delimiter.trim())?;
    let offset = value.find(delimiter)? + delimiter.len();
    Some(Operand::Literal(value[offset..].to_owned()))
}

fn variable(value: &str) -> Option<&str> {
    let name = value.strip_prefix('$')?;
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|first| {
            (first.is_ascii_alphabetic() || first == '_')
                && characters.all(|character| {
                    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
                })
        })
        .then_some(name)
}

fn format_decimal(
    value: &str,
    value_is_expression: bool,
    picture: &str,
    format: &DecimalFormat,
    xslt10_compatibility: bool,
) -> Result<String, FormatNumberEvaluationFailure> {
    // XSLT 1.0 deliberately uses the JDK 1.1 DecimalFormat pattern language
    // from before currency-sign support was added. The Recommendation makes
    // U+00A4 invalid anywhere in the picture, including quoted literals.
    if xslt10_compatibility && picture.contains('\u{00a4}') {
        return Err(FormatNumberEvaluationFailure::InvalidPicture);
    }
    let number = if value_is_expression {
        evaluate_source_free_number(value).ok_or(FormatNumberEvaluationFailure::Unsupported(
            FormatNumberUnsupported::Number,
        ))?
    } else {
        value.trim().parse().unwrap_or(f64::NAN)
    };
    if xslt10_compatibility && picture.is_empty() && !number.is_finite() {
        return Ok(if number.is_nan() {
            format.nan.clone()
        } else {
            let mut result = String::new();
            if number.is_sign_negative() {
                result.push(format.minus_sign);
            }
            result.push_str(&format.infinity);
            result
        });
    }
    let normalized_picture;
    let picture = if xslt10_compatibility
        && format.zero_digit != '0'
        && needs_xslt10_ascii_zero_fallback(picture, format)
    {
        normalized_picture = normalize_xslt10_zero_placeholders(picture, format.zero_digit);
        normalized_picture.as_str()
    } else {
        picture
    };
    let (positive, negative) = split_subpictures(picture, format.pattern_separator)
        .ok_or(FormatNumberEvaluationFailure::InvalidPicture)?;
    let negative_value = number.is_sign_negative();
    let selected = if negative_value {
        negative.unwrap_or(positive)
    } else {
        positive
    };
    let parsed =
        parse_subpicture(selected, format).ok_or(FormatNumberEvaluationFailure::InvalidPicture)?;
    let implicit_minus = negative_value
        && negative.is_none_or(|negative| {
            let positive = parse_subpicture(positive, format);
            let negative = parse_subpicture(negative, format);
            positive.zip(negative).is_some_and(|(positive, negative)| {
                positive.prefix == negative.prefix && positive.suffix == negative.suffix
            })
        });
    let mut output = String::new();
    if implicit_minus {
        output.push(format.minus_sign);
    }
    output.push_str(&parsed.prefix);
    if number.is_nan() {
        output.push_str(&format.nan);
    } else if number.is_infinite() {
        output.push_str(&format.infinity);
    } else {
        let exact = (value_is_expression)
            .then(|| evaluate_source_free_exact(value))
            .flatten()
            .and_then(|value| value.format_decimal().ok())
            .or_else(|| simple_decimal_lexical(value))
            .or_else(|| {
                ExactRational::parse_decimal(&number.to_string())?
                    .format_decimal()
                    .ok()
            })
            .ok_or(FormatNumberEvaluationFailure::Unsupported(
                FormatNumberUnsupported::Number,
            ))?;
        output.push_str(&format_finite_exact(&exact, &parsed, format).ok_or(
            FormatNumberEvaluationFailure::Unsupported(FormatNumberUnsupported::FiniteFormatting),
        )?);
    }
    output.push_str(&parsed.suffix);
    Ok(output)
}

fn needs_xslt10_ascii_zero_fallback(picture: &str, format: &DecimalFormat) -> bool {
    picture_tokens(picture).is_some_and(|tokens| {
        let mut has_ascii_zero = false;
        let mut has_declared_placeholder = false;
        for (character, active) in tokens {
            if !active {
                continue;
            }
            has_ascii_zero |= character == '0';
            has_declared_placeholder |= character == format.digit || character == format.zero_digit;
        }
        has_ascii_zero && !has_declared_placeholder
    })
}

fn normalize_xslt10_zero_placeholders(picture: &str, zero_digit: char) -> String {
    let mut normalized = String::with_capacity(picture.len());
    let mut quoted = false;
    let mut characters = picture.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\'' {
            normalized.push(character);
            if characters.peek().is_some_and(|next| *next == '\'') {
                normalized.push(characters.next().expect("peeked apostrophe remains"));
            } else {
                quoted = !quoted;
            }
        } else if character == '0' && !quoted {
            normalized.push(zero_digit);
        } else {
            normalized.push(character);
        }
    }
    normalized
}

#[derive(Clone)]
struct ParsedSubpicture {
    prefix: String,
    suffix: String,
    integer: String,
    fraction: String,
    scale: i128,
}

fn split_subpictures(picture: &str, separator: char) -> Option<(&str, Option<&str>)> {
    let mut quote = false;
    let mut separator_offset = None;
    let mut characters = picture.char_indices().peekable();
    while let Some((offset, character)) = characters.next() {
        if character == '\'' {
            if characters.peek().is_some_and(|(_, next)| *next == '\'') {
                characters.next();
            } else {
                quote = !quote;
            }
        } else if !quote && character == separator && separator_offset.replace(offset).is_some() {
            return None;
        }
    }
    if quote || picture.is_empty() {
        return None;
    }
    separator_offset.map_or(Some((picture, None)), |offset| {
        let negative = &picture[offset + separator.len_utf8()..];
        (!negative.is_empty()).then_some((&picture[..offset], Some(negative)))
    })
}

fn parse_subpicture(picture: &str, format: &DecimalFormat) -> Option<ParsedSubpicture> {
    let tokens = picture_tokens(picture)?;
    let first_digit = tokens.iter().position(|(character, active)| {
        *active && (*character == format.digit || *character == format.zero_digit)
    })?;
    let first = first_digit
        .checked_sub(1)
        .filter(|index| {
            let (character, active) = tokens[*index];
            active && character == format.decimal_separator
        })
        .unwrap_or(first_digit);
    let last = tokens.iter().rposition(|(character, active)| {
        *active && (*character == format.digit || *character == format.zero_digit)
    })?;
    let prefix = tokens[..first]
        .iter()
        .map(|(character, _)| character)
        .collect::<String>();
    let suffix = tokens[last + 1..]
        .iter()
        .map(|(character, _)| character)
        .collect::<String>();
    if tokens[..first]
        .iter()
        .chain(tokens[last + 1..].iter())
        .any(|(character, active)| *active && matches_active_numeric(*character, format))
    {
        return None;
    }
    let numeric_tokens = &tokens[first..=last];
    if numeric_tokens
        .iter()
        .any(|(character, active)| !*active || !matches_active_numeric(*character, format))
        || numeric_tokens
            .iter()
            .filter(|(character, active)| *active && *character == format.decimal_separator)
            .count()
            > 1
    {
        return None;
    }
    let numeric = numeric_tokens
        .iter()
        .map(|(character, _)| character)
        .collect::<String>();
    let (integer, fraction) = numeric
        .split_once(format.decimal_separator)
        .unwrap_or((&numeric, ""));
    let integer_digits = integer
        .chars()
        .filter(|character| *character != format.grouping_separator);
    if integer.starts_with(format.grouping_separator)
        || integer.ends_with(format.grouping_separator)
        || integer.contains(&format!(
            "{}{}",
            format.grouping_separator, format.grouping_separator
        ))
        || fraction.contains(format.grouping_separator)
        || !placeholders_are_ordered(integer_digits, format.digit, format.zero_digit)
        || !placeholders_are_ordered(fraction.chars(), format.zero_digit, format.digit)
    {
        return None;
    }
    let percent = tokens
        .iter()
        .filter(|(character, active)| *active && *character == format.percent)
        .count();
    let per_mille = tokens
        .iter()
        .filter(|(character, active)| *active && *character == format.per_mille)
        .count();
    if percent + per_mille > 1 {
        return None;
    }
    let scale = if percent == 1 {
        100
    } else if per_mille == 1 {
        1000
    } else {
        1
    };
    Some(ParsedSubpicture {
        prefix,
        suffix,
        integer: integer.to_owned(),
        fraction: fraction.to_owned(),
        scale,
    })
}

fn picture_tokens(picture: &str) -> Option<Vec<(char, bool)>> {
    let mut tokens = Vec::with_capacity(picture.len());
    let mut quote = false;
    let mut characters = picture.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\'' {
            if characters.peek().is_some_and(|next| *next == '\'') {
                characters.next();
                tokens.push(('\'', false));
            } else {
                quote = !quote;
            }
        } else {
            tokens.push((character, !quote));
        }
    }
    (!quote).then_some(tokens)
}

fn matches_active_numeric(character: char, format: &DecimalFormat) -> bool {
    character == format.digit
        || character == format.zero_digit
        || character == format.decimal_separator
        || character == format.grouping_separator
}

fn placeholders_are_ordered(
    placeholders: impl Iterator<Item = char>,
    first: char,
    second: char,
) -> bool {
    let mut saw_second = false;
    for placeholder in placeholders {
        if placeholder == second {
            saw_second = true;
        } else if placeholder == first && saw_second {
            return false;
        }
    }
    true
}

fn evaluate_source_free_number(value: &str) -> Option<f64> {
    let value = value.trim();
    if let Some(inner) = value
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        return evaluate_source_free_number(inner);
    }
    if let Some(quoted) = quoted(value) {
        return Some(quoted.trim().parse().unwrap_or(f64::NAN));
    }
    if let Some(inner) = value
        .strip_prefix("round(")
        .and_then(|inner| inner.strip_suffix(')'))
    {
        let number = evaluate_source_free_number(inner)?;
        return Some(if (-0.5..0.0).contains(&number) {
            -0.0
        } else {
            (number + 0.5).floor()
        });
    }
    for (operator, operation) in [
        (
            " div ",
            (|left: f64, right: f64| left / right) as fn(f64, f64) -> f64,
        ),
        (
            "*",
            (|left: f64, right: f64| left * right) as fn(f64, f64) -> f64,
        ),
    ] {
        if let Some((left, right)) = value.split_once(operator) {
            return Some(operation(
                evaluate_source_free_number(left)?,
                evaluate_source_free_number(right)?,
            ));
        }
    }
    value.parse().ok()
}

fn evaluate_source_free_exact(value: &str) -> Option<ExactRational> {
    let value = value.trim();
    if let Some(inner) = value
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        return evaluate_source_free_exact(inner);
    }
    if let Some(quoted) = quoted(value) {
        return ExactRational::parse_decimal(quoted);
    }
    if let Some((left, right)) = value.split_once(" div ") {
        return evaluate_source_free_exact(left)?
            .checked_divide(evaluate_source_free_exact(right)?)
            .ok();
    }
    if let Some((left, right)) = value.split_once('*') {
        return evaluate_source_free_exact(left)?
            .checked_multiply(evaluate_source_free_exact(right)?)
            .ok();
    }
    ExactRational::parse_decimal(value)
}

fn simple_decimal_lexical(value: &str) -> Option<String> {
    let value = value.trim();
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    let mut parts = unsigned.split('.');
    let whole = parts.next()?;
    let fraction = parts.next();
    if parts.next().is_some()
        || (whole.is_empty() && fraction.is_none_or(str::is_empty))
        || !whole.chars().all(|character| character.is_ascii_digit())
        || fraction.is_some_and(|fraction| {
            fraction.is_empty() || !fraction.chars().all(|character| character.is_ascii_digit())
        })
    {
        return None;
    }
    Some(value.to_owned())
}

fn format_finite_exact(
    value: &str,
    picture: &ParsedSubpicture,
    format: &DecimalFormat,
) -> Option<String> {
    let maximum_fraction = picture.fraction.chars().count();
    let minimum_fraction = picture
        .fraction
        .chars()
        .filter(|digit| *digit == format.zero_digit)
        .count();
    let minimum_integer = picture
        .integer
        .chars()
        .filter(|digit| *digit == format.zero_digit)
        .count();
    let value = value.strip_prefix(['+', '-']).unwrap_or(value);
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    if !whole.chars().all(|character| character.is_ascii_digit())
        || !fraction.chars().all(|character| character.is_ascii_digit())
    {
        return None;
    }
    let scale_digits = match picture.scale {
        1 => 0,
        100 => 2,
        1000 => 3,
        _ => return None,
    };
    let mut digits = format!("{whole}{fraction}");
    if digits.is_empty() {
        digits.push('0');
    }
    let decimal_position = whole.len().checked_add(scale_digits)?;
    if decimal_position > digits.len() {
        digits.push_str(&"0".repeat(decimal_position - digits.len()));
    }
    let retained_length = decimal_position.checked_add(maximum_fraction)?;
    let round_up = digits
        .as_bytes()
        .get(retained_length)
        .is_some_and(|digit| *digit >= b'5');
    if digits.len() < retained_length {
        digits.push_str(&"0".repeat(retained_length - digits.len()));
    } else {
        digits.truncate(retained_length);
    }
    if round_up {
        increment_decimal_digits(&mut digits);
    }
    let split = digits.len().saturating_sub(maximum_fraction);
    let mut fixed = if maximum_fraction == 0 {
        digits
    } else {
        format!("{}.{}", &digits[..split], &digits[split..])
    };
    if maximum_fraction > minimum_fraction {
        while fixed.ends_with('0')
            && fixed.len() - fixed.find('.').unwrap_or(fixed.len()) - 1 > minimum_fraction
        {
            fixed.pop();
        }
        if fixed.ends_with('.') {
            fixed.pop();
        }
    }
    let (whole, fraction) = fixed.split_once('.').unwrap_or((&fixed, ""));
    let significant_whole = whole.trim_start_matches('0');
    let mut whole = if significant_whole.is_empty() && !picture.integer.is_empty() {
        "0".to_owned()
    } else {
        significant_whole.to_owned()
    };
    if whole.len() < minimum_integer {
        whole.insert_str(0, &"0".repeat(minimum_integer - whole.len()));
    }
    if let Some(separator) = picture.integer.rfind(format.grouping_separator) {
        let group_size = picture.integer[separator + format.grouping_separator.len_utf8()..]
            .chars()
            .count();
        if group_size == 0 {
            return None;
        }
        let mut grouped = String::with_capacity(whole.len() + whole.len() / group_size);
        for (index, character) in whole.chars().rev().enumerate() {
            if index > 0 && index % group_size == 0 {
                grouped.push(format.grouping_separator);
            }
            grouped.push(character);
        }
        whole = grouped.chars().rev().collect();
    }
    let formatted = if fraction.is_empty() {
        whole
    } else {
        format!("{whole}{}{fraction}", format.decimal_separator)
    };
    substitute_digit_family(&formatted, format.zero_digit)
}

fn increment_decimal_digits(digits: &mut String) {
    let mut bytes = digits.as_bytes().to_vec();
    for digit in bytes.iter_mut().rev() {
        if *digit < b'9' {
            *digit += 1;
            *digits = String::from_utf8(bytes).expect("decimal digits remain UTF-8");
            return;
        }
        *digit = b'0';
    }
    bytes.insert(0, b'1');
    *digits = String::from_utf8(bytes).expect("decimal digits remain UTF-8");
}

fn substitute_digit_family(value: &str, zero_digit: char) -> Option<String> {
    if zero_digit == '0' {
        return Some(value.to_owned());
    }
    let zero = u32::from(zero_digit);
    value
        .chars()
        .map(|character| {
            character.to_digit(10).map_or(Some(character), |digit| {
                char::from_u32(zero.checked_add(digit)?)
            })
        })
        .collect()
}

fn quoted(value: &str) -> Option<&str> {
    value
        .strip_prefix('\'')
        .and_then(|value| value.strip_suffix('\''))
        .or_else(|| {
            value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
        })
}

fn parse_static_string(value: &str) -> Option<String> {
    if let Some(value) = quoted(value) {
        return Some(value.to_owned());
    }
    let arguments = value.strip_prefix("concat(")?.strip_suffix(')')?;
    let mut result = String::new();
    let mut remainder = arguments;
    loop {
        let (argument, next) = split_top_level_comma(remainder)
            .map_or((remainder, None), |(argument, remainder)| {
                (argument, Some(remainder))
            });
        result.push_str(quoted(argument.trim())?);
        let Some(next) = next else {
            break;
        };
        remainder = next;
    }
    Some(result)
}

fn split_top_level_comma(value: &str) -> Option<(&str, &str)> {
    let mut depth = 0_usize;
    let mut quote = None;
    for (offset, character) in value.char_indices() {
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => return Some((&value[..offset], &value[offset + 1..])),
            _ => {}
        }
    }
    None
}

fn top_level_argument_count(value: &str) -> Option<usize> {
    if value.trim().is_empty() {
        return Some(0);
    }
    let mut depth = 0_usize;
    let mut quote = None;
    let mut count = 1_usize;
    for character in value.chars() {
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => count += 1,
            _ => {}
        }
    }
    (depth == 0 && quote.is_none()).then_some(count)
}

fn unsupported(expression: &str, location: &SourceLocation) -> FormatNumberFailure {
    FormatNumberFailure {
        kind: FormatNumberFailureKind::Unsupported,
        detail: format!(
            "the private formatting slice supports exact nonnegative decimals or variables, number() over a string literal, substring-after() over string literals, and picture '#,###.00': {expression}"
        ),
        location: location.clone(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::xdm::atomic_value_experiment::AtomicValue;
    use crate::xdm::owned_tree_experiment::SourceLocation;
    use crate::xml::quick_xml_experiment::ExpandedName;
    use crate::xml::quick_xml_experiment::NamespaceBinding;

    use super::{
        DecimalFormat, FormatNumberEvaluationFailure, FormatNumberFailureKind, evaluate,
        evaluate_with_path_values, parse, parse_with_path_operands,
    };

    fn location() -> SourceLocation {
        SourceLocation {
            resource: "memory:format-number".to_owned(),
            span: 0..1,
        }
    }

    #[test]
    fn distinguishes_invalid_arity_from_unsupported_operand_shapes() {
        for source in [
            "format-number()",
            "format-number(1)",
            "format-number(1, '0', 'named', 'extra')",
        ] {
            assert_eq!(
                parse(source, &location())
                    .expect_err("invalid arity must fail")
                    .kind,
                FormatNumberFailureKind::InvalidArity,
                "{source}"
            );
        }
        assert_eq!(
            parse("format-number(,'#')", &location())
                .expect_err("missing first expression must fail")
                .kind,
            FormatNumberFailureKind::InvalidSyntax
        );
        assert_eq!(
            parse("format-number(1, concat('0', '0'))", &location())
                .expect_err("unsupported operand must fail")
                .kind,
            FormatNumberFailureKind::Unsupported
        );
    }

    #[test]
    fn composes_admitted_constant_number_and_picture_functions() {
        for source in [
            "format-number(1234.78,substring-after('this#,###.00','this'))",
            "format-number(number('1234.78'),'#,###.00')",
            "format-number(number('1234.78'),substring-after('this#,###.00','this'))",
        ] {
            let expression = parse(source, &location()).expect("admitted formatting expression");
            assert_eq!(
                evaluate(&expression, &BTreeMap::new()),
                Ok("1,234.78".to_owned())
            );
        }
    }

    #[test]
    fn admits_double_quoted_pictures_with_commas_and_apostrophe_literals() {
        let expression = parse(
            r#"format-number(4321.1234, "special '#0.,;' number: #,###.###")"#,
            &location(),
        )
        .expect("double-quoted picture should parse as one argument");

        assert_eq!(
            evaluate(&expression, &BTreeMap::new()),
            Ok("special #0.,; number: 4,321.123".to_owned())
        );
    }

    #[test]
    fn xslt10_empty_path_operands_format_nan_without_a_picture() {
        let expression = parse_with_path_operands("format-number(x,x)", &location(), true)
            .expect("XSLT 1.0 child-path operands should compile");

        assert_eq!(
            evaluate_with_path_values(&expression, &BTreeMap::new(), Some(""), Some("")),
            Ok("NaN".to_owned())
        );
    }

    #[test]
    fn xslt10_ascii_zero_picture_selects_the_declared_digit_family() {
        let mut expression =
            parse_with_path_operands("format-number(123, '0000')", &location(), true)
                .expect("XSLT 1.0 alternate digit-family expression should compile");
        expression.set_default_decimal_format(&DecimalFormat {
            zero_digit: 'a',
            ..DecimalFormat::default()
        });

        assert_eq!(
            evaluate_with_path_values(&expression, &BTreeMap::new(), None, None),
            Ok("abcd".to_owned())
        );

        let mut mixed = parse_with_path_operands(
            "format-number(4030201.0506, '#!!!,!!!,aaa.aaaaaa0')",
            &location(),
            true,
        )
        .expect("mixed declared placeholders and ASCII zero should compile");
        mixed.set_default_decimal_format(&DecimalFormat {
            digit: '!',
            zero_digit: 'a',
            ..DecimalFormat::default()
        });
        assert_eq!(
            evaluate_with_path_values(&mixed, &BTreeMap::new(), None, None),
            Ok("#e,ada,cab.afagaa0".to_owned())
        );
    }

    #[test]
    fn resolves_invocation_local_variable_operands() {
        let expression = parse("format-number($value,$picture)", &location())
            .expect("variable operands should parse");
        let mut variables = BTreeMap::new();
        variables.insert("value".to_owned(), AtomicValue::string("1234.78"));
        variables.insert("picture".to_owned(), AtomicValue::string("#,###.00"));
        assert_eq!(evaluate(&expression, &variables), Ok("1,234.78".to_owned()));
        variables.insert("value".to_owned(), AtomicValue::string("-1234.5"));
        variables.insert(
            "picture".to_owned(),
            AtomicValue::string("'#'#,##0.00'#';('#'#,##0.00'#') "),
        );
        assert_eq!(
            evaluate(&expression, &variables),
            Ok("(#1,234.50#) ".to_owned())
        );
        variables.insert("value".to_owned(), AtomicValue::string("not-a-number"));
        variables.insert("picture".to_owned(), AtomicValue::string("#"));
        assert_eq!(evaluate(&expression, &variables), Ok("NaN".to_owned()));
        variables.remove("picture");
        assert_eq!(
            evaluate(&expression, &variables),
            Err(FormatNumberEvaluationFailure::UnboundVariable(
                "picture".to_owned()
            ))
        );
    }

    #[test]
    fn folds_a_literal_concat_decimal_format_name() {
        let expression = parse(
            "format-number(12.34, '---.--', concat('xsl:', 'format'))",
            &location(),
        )
        .expect("literal concat decimal-format name should parse");
        assert_eq!(expression.requested_format_lexical(), Some("xsl:format"));
    }

    #[test]
    fn resolves_a_variable_decimal_format_name_against_static_namespaces() {
        let mut expression = parse("format-number($value, $picture, $format)", &location())
            .expect("variable decimal-format name should parse");
        expression.set_static_namespaces(vec![NamespaceBinding {
            prefix: Some("f".to_owned()),
            namespace: "urn:format".to_owned(),
        }]);
        expression.set_dynamic_formats(vec![(
            ExpandedName {
                namespace: Some("urn:format".to_owned()),
                local: "european".to_owned(),
            },
            DecimalFormat {
                decimal_separator: ',',
                grouping_separator: '.',
                ..DecimalFormat::default()
            },
        )]);
        let variables = BTreeMap::from([
            ("value".to_owned(), AtomicValue::string("1234.5")),
            ("picture".to_owned(), AtomicValue::string("#.##0,0")),
            ("format".to_owned(), AtomicValue::string("f:european")),
        ]);
        assert_eq!(evaluate(&expression, &variables), Ok("1.234,5".to_owned()));

        let invalid_variables = BTreeMap::from([
            ("value".to_owned(), AtomicValue::string("1234.5")),
            ("picture".to_owned(), AtomicValue::string("#.##0,0")),
            ("format".to_owned(), AtomicValue::string("")),
        ]);
        assert_eq!(
            evaluate(&expression, &invalid_variables),
            Err(FormatNumberEvaluationFailure::InvalidDecimalFormatName)
        );
    }

    #[test]
    fn applies_a_compiled_unnamed_decimal_format() {
        let mut expression = parse("format-number(931.4857, '000.000|###')", &location())
            .expect("format-number expression should parse");
        expression.set_default_decimal_format(&DecimalFormat {
            decimal_separator: '|',
            grouping_separator: '.',
            infinity: "huge".to_owned(),
            minus_sign: '_',
            nan: "not-a-number".to_owned(),
            percent: 'c',
            per_mille: 'm',
            zero_digit: '0',
            digit: '#',
            pattern_separator: '\\',
        });
        assert_eq!(
            evaluate(&expression, &BTreeMap::new()),
            Ok("000.931|486".to_owned())
        );

        let mut negative = parse(
            "format-number(-26931.4, '###,###.###;###,###.###')",
            &location(),
        )
        .expect("negative formatting should parse");
        negative.set_default_decimal_format(&DecimalFormat {
            minus_sign: '_',
            ..DecimalFormat::default()
        });
        assert_eq!(
            evaluate(&negative, &BTreeMap::new()),
            Ok("_26,931.4".to_owned())
        );

        let mut non_finite = parse("format-number(1 div 0, '0')", &location())
            .expect("non-finite formatting should parse");
        non_finite.set_default_decimal_format(&DecimalFormat {
            infinity: "huge".to_owned(),
            ..DecimalFormat::default()
        });
        assert_eq!(
            evaluate(&non_finite, &BTreeMap::new()),
            Ok("huge".to_owned())
        );

        let mut named = parse("format-number(1234.5, '#.##0,0', 'european')", &location())
            .expect("static named formatting should parse");
        named.set_requested_format(ExpandedName {
            namespace: None,
            local: "european".to_owned(),
        });
        named.set_default_decimal_format(&DecimalFormat {
            decimal_separator: ',',
            grouping_separator: '.',
            ..DecimalFormat::default()
        });
        assert_eq!(evaluate(&named, &BTreeMap::new()), Ok("1.234,5".to_owned()));

        let mut alternate_digits = parse("format-number(102.3, 'aaa.a')", &location())
            .expect("alternate digit family should parse");
        alternate_digits.set_default_decimal_format(&DecimalFormat {
            zero_digit: 'a',
            ..DecimalFormat::default()
        });
        assert_eq!(
            evaluate(&alternate_digits, &BTreeMap::new()),
            Ok("bac.d".to_owned())
        );
    }

    #[test]
    fn rejects_invalid_pictures() {
        let variables = BTreeMap::new();
        for source in [
            "format-number(1.23, '0.0.0')",
            "format-number(1.23, '#,')",
            "format-number(0, '#.#0')",
            "format-number(0, '0#.#')",
        ] {
            let expression = parse(source, &location()).expect("expression shape should parse");
            assert_eq!(
                evaluate(&expression, &variables),
                Err(FormatNumberEvaluationFailure::InvalidPicture)
            );
        }
    }

    #[test]
    fn xslt10_rejects_the_currency_sign_in_static_and_dynamic_pictures() {
        let expression =
            parse_with_path_operands("format-number(1234.5, '¤#,##0.00')", &location(), true)
                .expect("XSLT 1.0 format-number expression should parse");
        assert_eq!(
            evaluate(&expression, &BTreeMap::new()),
            Err(FormatNumberEvaluationFailure::InvalidPicture)
        );

        let expression =
            parse_with_path_operands("format-number($value, $picture)", &location(), true)
                .expect("dynamic XSLT 1.0 picture should parse");
        let variables = BTreeMap::from([
            ("value".to_owned(), AtomicValue::string("1234.5")),
            ("picture".to_owned(), AtomicValue::string("'¤'0.00")),
        ]);
        assert_eq!(
            evaluate(&expression, &variables),
            Err(FormatNumberEvaluationFailure::InvalidPicture)
        );
    }

    #[test]
    fn formats_static_default_decimal_pictures() {
        for (source, expected) in [
            ("format-number(.12, '.000')", ".120"),
            ("format-number(239236.588, '00000.00')", "239236.59"),
            ("format-number(0.4857, '###.###%')", "48.57%"),
            (
                "format-number(-87505.2812, '+###,###.000###;-###,###.000###')",
                "-87,505.2812",
            ),
            (
                "format-number(2392.14*(-36.58), '+###,###.000###;-###,###.000###')",
                "-87,504.4812",
            ),
            (
                "format-number(185.2812, 'PREFIX##00.000###SUFFIX')",
                "PREFIX185.2812SUFFIX",
            ),
            (
                "format-number(987654321, '###,##0,00.00')",
                "9,87,65,43,21.00",
            ),
            (
                "format-number(9999999999999999999999999999999999999999999999999999999999999999, '0')",
                "9999999999999999999999999999999999999999999999999999999999999999",
            ),
            ("format-number('foo', '#############')", "NaN"),
        ] {
            let expression = parse(source, &location()).expect("parse static format-number");
            assert_eq!(
                evaluate(&expression, &BTreeMap::new()),
                Ok(expected.to_owned())
            );
        }
    }
}
