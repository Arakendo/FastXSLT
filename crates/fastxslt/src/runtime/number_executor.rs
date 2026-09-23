//! Private execution of the admitted `xsl:number` surface.

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xslt::golden_semantics_experiment::{
    Instruction, NumberFormat, NumberFormatPlan, NumberGrouping, NumberLevel, NumberPattern,
    NumberPositionPredicate, NumberTokenStyle, NumberValue, parse_admitted_number_format,
};

use super::{
    ExecutionFailure, FailureCategory, ResultNode, RuntimeVariables, SequenceContext,
    SequenceInputs, append_text, control_failure, execution_context_string_value, failure,
    required_source_context, value_evaluator,
};

pub(super) fn execute(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    if let Some(value) = evaluate(inputs, instruction, execution, variables, control)? {
        append_text(result, &value, inputs.request_id, control)?;
    }
    Ok(())
}

pub(super) fn evaluate(
    inputs: &SequenceInputs<'_>,
    instruction: &Instruction,
    execution: SequenceContext<'_>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Option<String>, ExecutionFailure> {
    let Instruction::Number {
        value,
        level,
        count,
        from,
        format,
        xslt10_compatibility,
        ..
    } = instruction
    else {
        unreachable!("number execution receives only number instructions")
    };
    let owned_format;
    let format = match format {
        NumberFormatPlan::Static(format) => format,
        NumberFormatPlan::Xslt10Variable(variable) => {
            let lexical = value_evaluator::xslt10_variable_string_value(
                inputs, variable, variables, control,
            )?;
            owned_format = parse_admitted_number_format(&lexical).ok_or_else(|| {
                failure(
                    "FXRT1017",
                    FailureCategory::Unsupported,
                    Some(inputs.request_id),
                    format!("unsupported dynamic xsl:number format token: {lexical}"),
                )
            })?;
            &owned_format
        }
    };
    let formatted = if let Some(value) = value {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, inputs.request_id))?;
        let value = evaluate_value(inputs, value, execution, variables, control)?;
        Some(if *xslt10_compatibility && !value.is_formattable {
            value.xslt10_lexical_override.unwrap_or(value.lexical)
        } else {
            apply_format(&value.lexical, format)
        })
    } else {
        let (source, _) = required_source_context(inputs, execution.node)?;
        let count = count
            .as_ref()
            .map(|pattern| evaluate_pattern(inputs, source, pattern, control))
            .transpose()?;
        let from = from
            .as_ref()
            .map(|pattern| evaluate_pattern(inputs, source, pattern, control))
            .transpose()?;
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
    Ok(formatted)
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
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<EvaluatedNumberValue, ExecutionFailure> {
    let (value, original_lexical) = match value {
        NumberValue::Literal(value) => (
            value.trim().parse::<f64>().unwrap_or(f64::NAN),
            Some(value.trim().to_owned()),
        ),
        NumberValue::ContextPosition => execution
            .focus_position
            .to_string()
            .parse::<f64>()
            .map(|value| (value, None))
            .expect("a decimal usize representation is an XPath number"),
        NumberValue::ContextItem => {
            let lexical =
                execution_context_string_value(inputs, execution, control)?.unwrap_or_default();
            (
                lexical.trim().parse::<f64>().unwrap_or(f64::NAN),
                Some(lexical),
            )
        }
        NumberValue::BinaryNumeric(expression) => {
            let value = super::value_evaluator::evaluate_binary_numeric_value(
                inputs,
                execution.node,
                execution.sequence_focus(),
                expression,
                variables,
                control,
            )?
            .parse::<f64>()
            .expect("the checked numeric evaluator returns an XPath number");
            (value, None)
        }
    };
    if value.is_nan() {
        return Ok(EvaluatedNumberValue {
            lexical: "NaN".to_owned(),
            xslt10_lexical_override: original_lexical,
            is_formattable: false,
        });
    }
    if value == f64::INFINITY {
        return Ok(EvaluatedNumberValue {
            lexical: "Infinity".to_owned(),
            xslt10_lexical_override: None,
            is_formattable: false,
        });
    }
    if value == f64::NEG_INFINITY {
        return Ok(EvaluatedNumberValue {
            lexical: "-Infinity".to_owned(),
            xslt10_lexical_override: None,
            is_formattable: false,
        });
    }
    if value < 0.5 {
        return Ok(EvaluatedNumberValue {
            lexical: if value == 0.0 {
                "0".to_owned()
            } else {
                value.to_string()
            },
            xslt10_lexical_override: None,
            is_formattable: true,
        });
    }
    Ok(EvaluatedNumberValue {
        lexical: (value + 0.5).floor().to_string(),
        xslt10_lexical_override: None,
        is_formattable: true,
    })
}

struct EvaluatedNumberValue {
    lexical: String,
    xslt10_lexical_override: Option<String>,
    is_formattable: bool,
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
    count: Option<&EvaluatedNumberPattern<'_>>,
    from: Option<&EvaluatedNumberPattern<'_>>,
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
        let mut found_boundary = false;
        while let Some(node) = boundary {
            charge_node_visit(control, inputs.request_id)?;
            if pattern_matches(source, node, pattern, inputs.request_id, control)? {
                found_boundary = true;
                break;
            }
            boundary = source.parent(node);
        }
        if !found_boundary && numbered != context {
            return Ok(None);
        }
    }
    sibling_position(source, numbered, context, count, inputs.request_id, control)
        .map(|number| Some(number.to_string()))
}

fn sibling_position(
    source: &Document,
    numbered: NodeId,
    context: NodeId,
    count: Option<&EvaluatedNumberPattern<'_>>,
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
    count: Option<&EvaluatedNumberPattern<'_>>,
    from: Option<&EvaluatedNumberPattern<'_>>,
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

enum EvaluatedNumberPattern<'a> {
    Direct(&'a NumberPattern),
    KeyNodes(Vec<NodeId>),
    ChildOf {
        parent: Box<EvaluatedNumberPattern<'a>>,
        child: Box<EvaluatedNumberPattern<'a>>,
    },
    Alternatives(Vec<EvaluatedNumberPattern<'a>>),
}

fn evaluate_pattern<'a>(
    inputs: &SequenceInputs<'_>,
    source: &Document,
    pattern: &'a NumberPattern,
    control: &mut InvocationControl,
) -> Result<EvaluatedNumberPattern<'a>, ExecutionFailure> {
    Ok(match pattern {
        NumberPattern::Xslt10KeyLookup(lookup) => {
            EvaluatedNumberPattern::KeyNodes(super::key_lookup::select_static_pattern(
                inputs.program,
                source,
                lookup,
                inputs.request_id,
                &inputs.document_rooted_matches,
                control,
            )?)
        }
        NumberPattern::ChildOf { parent, child } => EvaluatedNumberPattern::ChildOf {
            parent: Box::new(evaluate_pattern(inputs, source, parent, control)?),
            child: Box::new(evaluate_pattern(inputs, source, child, control)?),
        },
        NumberPattern::Alternatives(alternatives) => EvaluatedNumberPattern::Alternatives(
            alternatives
                .iter()
                .map(|alternative| evaluate_pattern(inputs, source, alternative, control))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        pattern => EvaluatedNumberPattern::Direct(pattern),
    })
}

fn pattern_matches(
    source: &Document,
    node: NodeId,
    pattern: &EvaluatedNumberPattern<'_>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    Ok(match pattern {
        EvaluatedNumberPattern::Direct(NumberPattern::Document) => {
            source.kind(node) == NodeKind::Document
        }
        EvaluatedNumberPattern::Direct(NumberPattern::AnyNode) => matches!(
            source.kind(node),
            NodeKind::Element
                | NodeKind::Text
                | NodeKind::Comment
                | NodeKind::ProcessingInstruction
        ),
        EvaluatedNumberPattern::Direct(NumberPattern::AnyElement) => {
            source.kind(node) == NodeKind::Element
        }
        EvaluatedNumberPattern::Direct(NumberPattern::AnyAttribute) => {
            source.kind(node) == NodeKind::Attribute
        }
        EvaluatedNumberPattern::Direct(NumberPattern::Element(name)) => {
            source.name(node) == Some(name)
        }
        EvaluatedNumberPattern::Direct(NumberPattern::ElementWithAttributeValue {
            element,
            attribute,
            value,
        }) => {
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
        EvaluatedNumberPattern::Direct(NumberPattern::ElementAtSiblingPosition {
            element,
            predicate,
        }) => {
            source.name(node) == Some(element)
                && sibling_element_position(source, node, element, request_id, control)?
                    .is_some_and(|position| position_predicate_matches(position, *predicate))
        }
        EvaluatedNumberPattern::KeyNodes(nodes) => nodes.contains(&node),
        EvaluatedNumberPattern::ChildOf { parent, child } => {
            if !pattern_matches(source, node, child, request_id, control)? {
                false
            } else if let Some(parent_node) = source.parent(node) {
                charge_node_visit(control, request_id)?;
                pattern_matches(source, parent_node, parent, request_id, control)?
            } else {
                false
            }
        }
        EvaluatedNumberPattern::Alternatives(alternatives) => {
            let mut matched = false;
            for alternative in alternatives {
                if pattern_matches(source, node, alternative, request_id, control)? {
                    matched = true;
                    break;
                }
            }
            matched
        }
        EvaluatedNumberPattern::Direct(
            NumberPattern::Xslt10KeyLookup(_)
            | NumberPattern::ChildOf { .. }
            | NumberPattern::Alternatives(_),
        ) => unreachable!("composed number patterns are evaluated before execution"),
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
    count: Option<&EvaluatedNumberPattern<'_>>,
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
    count: Option<&EvaluatedNumberPattern<'_>>,
    from: Option<&EvaluatedNumberPattern<'_>>,
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
    count: Option<&'a EvaluatedNumberPattern<'a>>,
    from: Option<&'a EvaluatedNumberPattern<'a>>,
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
