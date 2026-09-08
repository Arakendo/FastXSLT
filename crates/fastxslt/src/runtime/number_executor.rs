//! Private execution of the admitted `xsl:number` surface.

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xslt::golden_semantics_experiment::{
    Instruction, NumberFormat, NumberGrouping, NumberLevel, NumberPattern, NumberPositionPredicate,
    NumberTokenStyle, NumberValue,
};

use super::{
    ExecutionFailure, ResultNode, SequenceContext, SequenceInputs, append_text, control_failure,
    execution_context_string_value, required_source_context,
};

pub(super) fn execute(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    execution: SequenceContext<'_>,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let Instruction::Number {
        value,
        level,
        count,
        from,
        format,
        ..
    } = instruction
    else {
        unreachable!("number execution receives only number instructions")
    };
    let formatted = if let Some(value) = value {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        Some(apply_format(
            &evaluate_value(inputs, value, execution, control)?,
            format,
        ))
    } else {
        match level {
            NumberLevel::Single if count.is_some() || from.is_some() => execute_patterned_single(
                inputs,
                execution.node,
                count.as_ref(),
                from.as_ref(),
                control,
            )?
            .map(|value| apply_format(&value, format)),
            NumberLevel::Single => Some(apply_format(
                &execute_default_single(inputs, execution.node, control)?.to_string(),
                format,
            )),
            NumberLevel::Multiple => execute_multiple(
                inputs,
                execution.node,
                count.as_ref(),
                from.as_ref(),
                format,
                control,
            )?,
            NumberLevel::Any => execute_any(
                inputs,
                execution.node,
                count.as_ref(),
                from.as_ref(),
                control,
            )?
            .map(|value| apply_format(&value, format)),
        }
    };
    if let Some(value) = formatted {
        append_text(result, &value, inputs.request_id, control)?;
    }
    Ok(())
}

fn apply_format(value: &str, format: &NumberFormat) -> String {
    let token = format
        .tokens
        .first()
        .expect("number formats always retain at least one token");
    let mut result = String::with_capacity(
        format
            .prefix
            .len()
            .saturating_add(token.minimum_width.max(value.len()))
            .saturating_add(format.suffix.len()),
    );
    result.push_str(&format.prefix);
    append_number_token(
        &mut result,
        value,
        token.style,
        token.minimum_width,
        format.grouping,
    );
    result.push_str(&format.suffix);
    result
}

fn format_sequence_tokens(values: &[String], format: &NumberFormat) -> String {
    let mut result = String::with_capacity(
        format
            .prefix
            .len()
            .saturating_add(values.iter().map(String::len).sum::<usize>())
            .saturating_add(format.suffix.len()),
    );
    result.push_str(&format.prefix);
    for (index, value) in values.iter().enumerate() {
        if index != 0 {
            if format.tokens.len() == 1 {
                result.push('.');
            } else {
                let separator = &format.separators[index
                    .saturating_sub(1)
                    .min(format.separators.len().saturating_sub(1))];
                result.push_str(separator);
            }
        }
        let token = &format.tokens[index.min(format.tokens.len() - 1)];
        append_number_token(
            &mut result,
            value,
            token.style,
            token.minimum_width,
            format.grouping,
        );
    }
    result.push_str(&format.suffix);
    result
}

fn append_number_token(
    result: &mut String,
    value: &str,
    token_style: NumberTokenStyle,
    minimum_width: usize,
    grouping: Option<NumberGrouping>,
) {
    match token_style {
        NumberTokenStyle::Decimal => append_decimal_token(result, value, minimum_width, grouping),
        NumberTokenStyle::AlphabeticUpper => append_alphabetic_token(result, value, b'A'),
        NumberTokenStyle::AlphabeticLower => append_alphabetic_token(result, value, b'a'),
        NumberTokenStyle::RomanUpper => append_roman_token(result, value, false),
        NumberTokenStyle::RomanLower => append_roman_token(result, value, true),
    }
}

fn append_decimal_token(
    result: &mut String,
    value: &str,
    minimum_width: usize,
    grouping: Option<NumberGrouping>,
) {
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        result.push_str(value);
        return;
    }
    let zeroes = minimum_width.saturating_sub(value.len());
    let total_digits = zeroes.saturating_add(value.len());
    for index in 0..total_digits {
        if index != 0
            && grouping.is_some_and(|grouping| (total_digits - index) % grouping.size == 0)
        {
            result.push(grouping.expect("checked grouping presence").separator);
        }
        if index < zeroes {
            result.push('0');
        } else {
            result.push(char::from(value.as_bytes()[index - zeroes]));
        }
    }
}

fn append_alphabetic_token(result: &mut String, value: &str, first: u8) {
    let Ok(mut number) = value.parse::<usize>() else {
        result.push_str(value);
        return;
    };
    if number == 0 {
        result.push_str(value);
        return;
    }
    let mut reversed = Vec::new();
    while number != 0 {
        number -= 1;
        reversed.push(char::from(
            first + u8::try_from(number % 26).expect("remainder is below 26"),
        ));
        number /= 26;
    }
    result.extend(reversed.into_iter().rev());
}

fn append_roman_token(result: &mut String, value: &str, lowercase: bool) {
    const PARTS: &[(usize, &str)] = &[
        (1_000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];

    let Ok(mut number) = value.parse::<usize>() else {
        result.push_str(value);
        return;
    };
    if number == 0 || number > 3_999 {
        result.push_str(value);
        return;
    }
    let start = result.len();
    for &(magnitude, lexical) in PARTS {
        while number >= magnitude {
            result.push_str(lexical);
            number -= magnitude;
        }
    }
    if lowercase {
        result[start..].make_ascii_lowercase();
    }
}

fn evaluate_value(
    inputs: &SequenceInputs<'_>,
    value: &NumberValue,
    execution: SequenceContext<'_>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let value = match value {
        NumberValue::Literal(value) => value.trim().parse::<f64>().unwrap_or(f64::NAN),
        NumberValue::ContextPosition => execution
            .focus_position
            .to_string()
            .parse::<f64>()
            .expect("a decimal usize representation is an XPath number"),
        NumberValue::ContextItem => execution_context_string_value(inputs, execution, control)?
            .map_or(f64::NAN, |value| {
                value.trim().parse::<f64>().unwrap_or(f64::NAN)
            }),
    };
    if value.is_nan() {
        return Ok("NaN".to_owned());
    }
    if value == f64::INFINITY {
        return Ok("Infinity".to_owned());
    }
    if value == f64::NEG_INFINITY {
        return Ok("-Infinity".to_owned());
    }
    if value < 0.5 {
        return Ok(if value == 0.0 {
            "0".to_owned()
        } else {
            value.to_string()
        });
    }
    Ok((value + 0.5).floor().to_string())
}

fn execute_default_single(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let Some(parent) = source.parent(context) else {
        return Ok(1);
    };
    let siblings = if source.kind(context) == NodeKind::Attribute {
        source.attributes(parent)
    } else {
        source.children(parent)
    };
    let mut number = 1usize;
    for sibling in siblings.iter().copied() {
        charge_node_visit(control, inputs.request_id)?;
        if sibling == context {
            break;
        }
        if nodes_share_default_pattern(source, sibling, context) {
            number = number.saturating_add(1);
        }
    }
    Ok(number)
}

fn execute_patterned_single(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    count: Option<&NumberPattern>,
    from: Option<&NumberPattern>,
    control: &mut InvocationControl,
) -> Result<Option<String>, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let mut candidate = Some(context);
    let numbered = loop {
        let Some(node) = candidate else {
            return Ok(None);
        };
        charge_node_visit(control, inputs.request_id)?;
        let matches = count_matches(source, node, context, count, inputs.request_id, control)?;
        if matches {
            break node;
        }
        if let Some(pattern) = from {
            if pattern_matches(source, node, pattern, inputs.request_id, control)? {
                return Ok(None);
            }
        }
        candidate = source.parent(node);
    };
    if let Some(pattern) = from {
        let mut boundary = Some(numbered);
        loop {
            let Some(node) = boundary else {
                return Ok(None);
            };
            charge_node_visit(control, inputs.request_id)?;
            if pattern_matches(source, node, pattern, inputs.request_id, control)? {
                break;
            }
            boundary = source.parent(node);
        }
    }
    sibling_position(source, numbered, context, count, inputs.request_id, control)
        .map(|number| Some(number.to_string()))
}

fn sibling_position(
    source: &Document,
    numbered: NodeId,
    context: NodeId,
    count: Option<&NumberPattern>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<usize, ExecutionFailure> {
    let Some(parent) = source.parent(numbered) else {
        return Ok(1);
    };
    let siblings = if source.kind(numbered) == NodeKind::Attribute {
        source.attributes(parent)
    } else {
        source.children(parent)
    };
    let mut number = 1usize;
    for sibling in siblings.iter().copied() {
        charge_node_visit(control, request_id)?;
        if sibling == numbered {
            break;
        }
        let matches = count_matches(source, sibling, context, count, request_id, control)?;
        if matches {
            number = number.saturating_add(1);
        }
    }
    Ok(number)
}

fn execute_multiple(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    count: Option<&NumberPattern>,
    from: Option<&NumberPattern>,
    format: &NumberFormat,
    control: &mut InvocationControl,
) -> Result<Option<String>, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let mut lineage = Vec::new();
    let mut candidate = Some(context);
    while let Some(node) = candidate {
        charge_node_visit(control, inputs.request_id)?;
        if let Some(pattern) = from {
            if pattern_matches(source, node, pattern, inputs.request_id, control)? {
                break;
            }
        }
        let matches = count_matches(source, node, context, count, inputs.request_id, control)?;
        if matches {
            lineage.push(node);
        }
        candidate = source.parent(node);
    }
    if lineage.is_empty() {
        return Ok(None);
    }
    lineage.reverse();
    let mut numbers = Vec::with_capacity(lineage.len());
    for node in lineage {
        numbers.push(
            sibling_position(source, node, context, count, inputs.request_id, control)?.to_string(),
        );
    }
    Ok(Some(format_sequence_tokens(&numbers, format)))
}

fn pattern_matches(
    source: &Document,
    node: NodeId,
    pattern: &NumberPattern,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    Ok(match pattern {
        NumberPattern::Document => source.kind(node) == NodeKind::Document,
        NumberPattern::AnyNode => matches!(
            source.kind(node),
            NodeKind::Element
                | NodeKind::Text
                | NodeKind::Comment
                | NodeKind::ProcessingInstruction
        ),
        NumberPattern::AnyElement => source.kind(node) == NodeKind::Element,
        NumberPattern::AnyAttribute => source.kind(node) == NodeKind::Attribute,
        NumberPattern::Element(name) => source.name(node) == Some(name),
        NumberPattern::ElementWithAttributeValue {
            element,
            attribute,
            value,
        } => {
            if source.name(node) == Some(element) {
                let mut matched = false;
                for &candidate in source.attributes(node) {
                    charge_node_visit(control, request_id)?;
                    if source.name(candidate) == Some(attribute)
                        && source.value(candidate) == Some(value)
                    {
                        matched = true;
                        break;
                    }
                }
                matched
            } else {
                false
            }
        }
        NumberPattern::ElementAtSiblingPosition { element, predicate } => {
            source.name(node) == Some(element)
                && sibling_element_position(source, node, element, request_id, control)?
                    .is_some_and(|position| position_predicate_matches(position, *predicate))
        }
        NumberPattern::ChildOf { parent, child } => {
            if !pattern_matches(source, node, child, request_id, control)? {
                false
            } else if let Some(parent_node) = source.parent(node) {
                charge_node_visit(control, request_id)?;
                pattern_matches(source, parent_node, parent, request_id, control)?
            } else {
                false
            }
        }
        NumberPattern::Alternatives(alternatives) => {
            let mut matched = false;
            for alternative in alternatives {
                if pattern_matches(source, node, alternative, request_id, control)? {
                    matched = true;
                    break;
                }
            }
            matched
        }
    })
}

fn sibling_element_position(
    source: &Document,
    node: NodeId,
    name: &crate::xml::quick_xml_experiment::ExpandedName,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Option<usize>, ExecutionFailure> {
    let Some(parent) = source.parent(node) else {
        return Ok(None);
    };
    let mut position = 0usize;
    for &sibling in source.children(parent) {
        charge_node_visit(control, request_id)?;
        if source.name(sibling) == Some(name) {
            position = position.saturating_add(1);
        }
        if sibling == node {
            return Ok(Some(position));
        }
    }
    Ok(None)
}

fn position_predicate_matches(position: usize, predicate: NumberPositionPredicate) -> bool {
    match predicate {
        NumberPositionPredicate::Exact(expected) => position == expected,
        NumberPositionPredicate::Modulo { divisor, remainder } => position % divisor == remainder,
    }
}

fn count_matches(
    source: &Document,
    node: NodeId,
    context: NodeId,
    count: Option<&NumberPattern>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    count.map_or_else(
        || Ok(nodes_share_default_pattern(source, node, context)),
        |pattern| pattern_matches(source, node, pattern, request_id, control),
    )
}

fn execute_any(
    inputs: &SequenceInputs<'_>,
    context: Option<NodeId>,
    count: Option<&NumberPattern>,
    from: Option<&NumberPattern>,
    control: &mut InvocationControl,
) -> Result<Option<String>, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let mut traversal = AnyTraversal {
        source,
        context,
        count,
        from,
        request_id: inputs.request_id,
        total: 0,
    };
    traversal.visit(source.document_node(), control)?;
    Ok(Some(if traversal.total == 0 {
        String::new()
    } else {
        traversal.total.to_string()
    }))
}

struct AnyTraversal<'a> {
    source: &'a Document,
    context: NodeId,
    count: Option<&'a NumberPattern>,
    from: Option<&'a NumberPattern>,
    request_id: &'a str,
    total: usize,
}

impl AnyTraversal<'_> {
    fn visit(
        &mut self,
        node: NodeId,
        control: &mut InvocationControl,
    ) -> Result<bool, ExecutionFailure> {
        charge_node_visit(control, self.request_id)?;
        let reset = if let Some(pattern) = self.from {
            pattern_matches(self.source, node, pattern, self.request_id, control)?
        } else {
            false
        };
        if reset {
            self.total = 0;
        } else {
            let matches = count_matches(
                self.source,
                node,
                self.context,
                self.count,
                self.request_id,
                control,
            )?;
            if matches {
                self.total = self.total.saturating_add(1);
            }
        }
        if node == self.context {
            return Ok(true);
        }
        for &attribute in self.source.attributes(node) {
            if self.visit(attribute, control)? {
                return Ok(true);
            }
        }
        for &child in self.source.children(node) {
            if self.visit(child, control)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn nodes_share_default_pattern(source: &Document, left: NodeId, right: NodeId) -> bool {
    source.kind(left) == source.kind(right) && source.name(left) == source.name(right)
}

fn charge_node_visit(
    control: &mut InvocationControl,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, request_id))
}
