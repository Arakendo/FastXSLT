//! Private compilation of the admitted `xsl:number` surface.

use crate::xdm::owned_tree_experiment::{Document, NodeId, SourceLocation};
use crate::xml::quick_xml_experiment::ExpandedName;
use crate::xslt::golden_semantics_experiment::{
    Instruction, NumberFormat, NumberFormatToken, NumberGrouping, NumberLevel, NumberPattern,
    NumberPositionPredicate, NumberTokenStyle, NumberValue,
};

use super::{
    CompileFailure, effective_xpath_default_namespace, ensure_no_meaningful_children,
    ensure_only_attributes, invalid, is_ascii_ncname, namespace_for_prefix, optional_attribute,
    unsupported, xpath_string_literal,
};

pub(super) fn compile(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &[
            "level",
            "value",
            "count",
            "from",
            "format",
            "grouping-separator",
            "grouping-size",
        ],
        "xsl:number",
    )?;
    ensure_no_meaningful_children(document, element, "xsl:number")?;
    let level = match optional_attribute(document, element, None, "level").map(str::trim) {
        None | Some("single") => NumberLevel::Single,
        Some("multiple") => NumberLevel::Multiple,
        Some("any") => NumberLevel::Any,
        Some(level) => {
            return Err(unsupported(
                "FXST1048",
                format!("unsupported xsl:number level: {level}"),
                document.location(element),
            ));
        }
    };
    let value = optional_attribute(document, element, None, "value")
        .map(|value| compile_value(value, document.location(element)))
        .transpose()?;
    let count = optional_attribute(document, element, None, "count")
        .map(|pattern| compile_pattern(document, element, pattern, "count"))
        .transpose()?;
    let from = optional_attribute(document, element, None, "from")
        .map(|pattern| compile_pattern(document, element, pattern, "from"))
        .transpose()?;
    let mut format = if let Some(format) = optional_attribute(document, element, None, "format") {
        compile_format(format, document.location(element))?
    } else {
        default_format()
    };
    format.grouping = compile_grouping(document, element)?;
    Ok(Instruction::Number {
        value,
        level,
        count,
        from,
        format,
        location: document.location(element).clone(),
    })
}

fn compile_pattern(
    document: &Document,
    element: NodeId,
    pattern: &str,
    attribute: &str,
) -> Result<NumberPattern, CompileFailure> {
    let pattern = pattern.trim();
    if pattern.contains('|') {
        let alternatives = pattern
            .split('|')
            .map(str::trim)
            .map(|alternative| {
                if alternative.is_empty() {
                    Err(unsupported_pattern(document, element, pattern, attribute))
                } else {
                    compile_pattern_atom(document, element, alternative, attribute)
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(NumberPattern::Alternatives(alternatives));
    }
    if pattern != "/" {
        if let Some((parent, child)) = pattern.split_once('/') {
            if parent.trim().is_empty() || child.trim().is_empty() || child.contains('/') {
                return Err(unsupported_pattern(document, element, pattern, attribute));
            }
            return Ok(NumberPattern::ChildOf {
                parent: Box::new(compile_pattern_atom(
                    document,
                    element,
                    parent.trim(),
                    attribute,
                )?),
                child: Box::new(compile_pattern_atom(
                    document,
                    element,
                    child.trim(),
                    attribute,
                )?),
            });
        }
    }
    compile_pattern_atom(document, element, pattern, attribute)
}

fn compile_pattern_atom(
    document: &Document,
    element: NodeId,
    pattern: &str,
    attribute: &str,
) -> Result<NumberPattern, CompileFailure> {
    if let Some((element_pattern, predicate)) = pattern.split_once('[') {
        let predicate = predicate
            .strip_suffix(']')
            .ok_or_else(|| unsupported_pattern(document, element, pattern, attribute))?;
        let NumberPattern::Element(element_name) =
            compile_pattern_atom(document, element, element_pattern.trim(), attribute)?
        else {
            return Err(unsupported_pattern(document, element, pattern, attribute));
        };
        if let Some(attribute_predicate) = predicate.strip_prefix('@') {
            let (attribute_name, expected_value) = attribute_predicate
                .split_once('=')
                .ok_or_else(|| unsupported_pattern(document, element, pattern, attribute))?;
            let expected_value = xpath_string_literal(expected_value.trim())
                .ok_or_else(|| unsupported_pattern(document, element, pattern, attribute))?;
            let attribute_name = compile_attribute_name(
                document,
                element,
                attribute_name.trim(),
                pattern,
                attribute,
            )?;
            return Ok(NumberPattern::ElementWithAttributeValue {
                element: element_name,
                attribute: attribute_name,
                value: expected_value.to_owned(),
            });
        }
        let position = compile_position_predicate(predicate)
            .ok_or_else(|| unsupported_pattern(document, element, pattern, attribute))?;
        return Ok(NumberPattern::ElementAtSiblingPosition {
            element: element_name,
            predicate: position,
        });
    }
    if pattern == "/" {
        return Ok(NumberPattern::Document);
    }
    if pattern == "node()" {
        return Ok(NumberPattern::AnyNode);
    }
    if pattern == "*" {
        return Ok(NumberPattern::AnyElement);
    }
    if pattern == "@*" {
        return Ok(NumberPattern::AnyAttribute);
    }
    let (prefix, local) = pattern
        .split_once(':')
        .map_or((None, pattern), |(prefix, local)| (Some(prefix), local));
    if !is_ascii_ncname(local) || prefix.is_some_and(|prefix| !is_ascii_ncname(prefix)) {
        return Err(unsupported_pattern(document, element, pattern, attribute));
    }
    let namespace = if let Some(prefix) = prefix {
        Some(
            namespace_for_prefix(document, element, prefix)
                .ok_or_else(|| {
                    invalid(
                        "FXST0038",
                        format!("unbound prefix in xsl:number {attribute} pattern: {prefix}"),
                        document.location(element),
                    )
                })?
                .to_owned(),
        )
    } else {
        effective_xpath_default_namespace(document, element).map(str::to_owned)
    };
    Ok(NumberPattern::Element(ExpandedName {
        namespace,
        local: local.to_owned(),
    }))
}

fn compile_position_predicate(predicate: &str) -> Option<NumberPositionPredicate> {
    let predicate = predicate.trim();
    if let Ok(position) = predicate.parse::<usize>() {
        return (position != 0).then_some(NumberPositionPredicate::Exact(position));
    }
    let mut tokens = predicate.split_whitespace();
    let position = tokens.next()?;
    let modulo = tokens.next()?;
    let divisor = tokens.next()?.parse::<usize>().ok()?;
    let equals = tokens.next()?;
    let remainder = tokens.next()?.parse::<usize>().ok()?;
    if tokens.next().is_some()
        || position != "position()"
        || modulo != "mod"
        || equals != "="
        || divisor == 0
    {
        return None;
    }
    Some(NumberPositionPredicate::Modulo { divisor, remainder })
}

fn compile_attribute_name(
    document: &Document,
    element: NodeId,
    lexical: &str,
    pattern: &str,
    pattern_attribute: &str,
) -> Result<ExpandedName, CompileFailure> {
    let (prefix, local) = lexical
        .split_once(':')
        .map_or((None, lexical), |(prefix, local)| (Some(prefix), local));
    if !is_ascii_ncname(local) || prefix.is_some_and(|prefix| !is_ascii_ncname(prefix)) {
        return Err(unsupported_pattern(
            document,
            element,
            pattern,
            pattern_attribute,
        ));
    }
    let namespace = prefix
        .map(|prefix| {
            namespace_for_prefix(document, element, prefix)
                .ok_or_else(|| {
                    invalid(
                        "FXST0038",
                        format!(
                            "unbound prefix in xsl:number {pattern_attribute} pattern: {prefix}"
                        ),
                        document.location(element),
                    )
                })
                .map(str::to_owned)
        })
        .transpose()?;
    Ok(ExpandedName {
        namespace,
        local: local.to_owned(),
    })
}

fn unsupported_pattern(
    document: &Document,
    element: NodeId,
    pattern: &str,
    attribute: &str,
) -> CompileFailure {
    unsupported(
        "FXST1050",
        format!("unsupported xsl:number {attribute} pattern: {pattern}"),
        document.location(element),
    )
}

fn default_format() -> NumberFormat {
    NumberFormat {
        prefix: String::new(),
        tokens: vec![NumberFormatToken {
            minimum_width: 1,
            style: NumberTokenStyle::Decimal,
        }],
        separators: Vec::new(),
        grouping: None,
        suffix: String::new(),
    }
}

fn compile_format(format: &str, location: &SourceLocation) -> Result<NumberFormat, CompileFailure> {
    let mut token_ranges = Vec::new();
    let mut token_start = None;
    for (index, character) in format.char_indices() {
        if character.is_ascii_alphanumeric() {
            token_start.get_or_insert(index);
        } else if let Some(start) = token_start.take() {
            token_ranges.push((start, index));
        }
    }
    if let Some(start) = token_start {
        token_ranges.push((start, format.len()));
    }
    let Some(&(first_start, _)) = token_ranges.first() else {
        return Err(unsupported_format(format, location));
    };
    let mut tokens = Vec::with_capacity(token_ranges.len());
    let mut separators = Vec::with_capacity(token_ranges.len().saturating_sub(1));
    for (index, &(start, end)) in token_ranges.iter().enumerate() {
        tokens.push(compile_format_token(&format[start..end], format, location)?);
        if let Some(&(next_start, _)) = token_ranges.get(index + 1) {
            separators.push(format[end..next_start].to_owned());
        }
    }
    let suffix_start = token_ranges
        .last()
        .map(|&(_, end)| end)
        .expect("a first token implies a last token");
    Ok(NumberFormat {
        prefix: format[..first_start].to_owned(),
        tokens,
        separators,
        grouping: None,
        suffix: format[suffix_start..].to_owned(),
    })
}

fn compile_grouping(
    document: &Document,
    element: NodeId,
) -> Result<Option<NumberGrouping>, CompileFailure> {
    let separator = optional_attribute(document, element, None, "grouping-separator");
    let size = optional_attribute(document, element, None, "grouping-size");
    let (Some(separator), Some(size)) = (separator, size) else {
        return Ok(None);
    };
    if separator.contains(['{', '}']) || size.contains(['{', '}']) {
        return Err(unsupported(
            "FXST1051",
            "dynamic xsl:number grouping attributes are outside the admitted slice",
            document.location(element),
        ));
    }
    let mut characters = separator.chars();
    let separator = characters.next().filter(|_| characters.next().is_none());
    let size = size.trim().parse::<usize>().ok().filter(|size| *size != 0);
    match (separator, size) {
        (Some(separator), Some(size)) => Ok(Some(NumberGrouping { separator, size })),
        _ => Err(unsupported(
            "FXST1051",
            "xsl:number grouping requires one separator character and a positive integer size",
            document.location(element),
        )),
    }
}

fn compile_format_token(
    token: &str,
    complete_format: &str,
    location: &SourceLocation,
) -> Result<NumberFormatToken, CompileFailure> {
    let (style, minimum_width) = match token {
        "A" => (NumberTokenStyle::AlphabeticUpper, 1),
        "a" => (NumberTokenStyle::AlphabeticLower, 1),
        "I" => (NumberTokenStyle::RomanUpper, 1),
        "i" => (NumberTokenStyle::RomanLower, 1),
        _ if token.ends_with('1')
            && token[..token.len() - 1]
                .chars()
                .all(|character| character == '0') =>
        {
            (NumberTokenStyle::Decimal, token.len())
        }
        _ => return Err(unsupported_format(complete_format, location)),
    };
    Ok(NumberFormatToken {
        minimum_width,
        style,
    })
}

fn unsupported_format(format: &str, location: &SourceLocation) -> CompileFailure {
    unsupported(
        "FXST1049",
        format!("unsupported xsl:number format token: {format}"),
        location,
    )
}

fn compile_value(
    expression: &str,
    location: &SourceLocation,
) -> Result<NumberValue, CompileFailure> {
    let expression = expression.trim();
    if expression == "position()" {
        return Ok(NumberValue::ContextPosition);
    }
    if expression == "." {
        return Ok(NumberValue::ContextItem);
    }
    let lexical = xpath_string_literal(expression).unwrap_or(expression);
    if lexical.parse::<f64>().is_ok() || xpath_string_literal(expression).is_some() {
        return Ok(NumberValue::Literal(lexical.to_owned()));
    }
    Err(unsupported(
        "FXXP1022",
        format!("unsupported xsl:number value expression: {expression}"),
        location,
    ))
}
