use std::collections::BTreeMap;

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ExpandedName, ParseLimits};
use crate::xpath::castable_experiment::{CastEvaluationFailure, CastExpression, evaluate_cast};
use crate::xpath::for_distinct_values_experiment::{
    ForDistinctValuesExpression, evaluate as evaluate_for_distinct_values,
};
use crate::xpath::path_experiment::evaluate_location_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    ApplySelection, BooleanExpression, Instruction, MatchPattern, NodeTest, SequenceItemExpression,
    StylesheetProgram, TemplateArgument,
};

#[path = "resource_compiler.rs"]
mod resource_compiler;
#[path = "runtime_context.rs"]
mod runtime_context;
#[path = "runtime_failure.rs"]
mod runtime_failure;
mod serialization;
#[path = "stylesheet_dependency_loader.rs"]
mod stylesheet_dependency_loader;
#[path = "template_selector.rs"]
mod template_selector;
#[cfg(test)]
#[path = "transform_set_experiment.rs"]
mod transform_set_experiment;
#[path = "value_evaluator.rs"]
mod value_evaluator;

#[cfg(test)]
pub(super) use resource_compiler::compile_resource;
pub(super) use resource_compiler::compile_resource_with_denied;
use runtime_context::{
    RuntimeGlobals, RuntimeVariables, SequenceInputs, TemporaryTree, bind_template_parameters,
    materialize_global_defaults, required_source_context,
};
pub(super) use runtime_failure::ExecutionFailure;
use runtime_failure::{FailureCategory, control_failure, failure, failure_at};
pub(super) use serialization::serialize_xml;
use template_selector::{accepts_mode as template_accepts_mode, select_template};
#[cfg(test)]
use transform_set_experiment::{
    ExecutionPolicy, InvocationEntry, TransformRequest, TransformSetBuilder, execute_transform_set,
};
use value_evaluator::execute_value_of;

const XML_LIMITS: ParseLimits = ParseLimits {
    max_events: 1_024,
    max_depth: 64,
};
const MAX_NAMED_TEMPLATE_CALL_DEPTH: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResultNode {
    Element {
        name: ExpandedName,
        namespaces: Vec<crate::xml::quick_xml_experiment::NamespaceBinding>,
        children: Vec<ResultNode>,
    },
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SemanticResult {
    children: Vec<ResultNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InvocationParameter {
    value: AtomicValue,
    tunnel: bool,
}

pub(super) fn execute_program(
    program: &StylesheetProgram,
    source: &Document,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    execute_program_with_parameters(program, source, &BTreeMap::new(), request_id, control)
}

fn execute_program_with_parameters(
    program: &StylesheetProgram,
    source: &Document,
    parameters: &BTreeMap<String, InvocationParameter>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    let globals =
        materialize_global_defaults(program, Some(source), parameters, request_id, control)?;
    let inputs = SequenceInputs {
        program,
        source: Some(source),
        request_id,
        globals: &globals,
    };
    let children = if let Some(root_template) = program
        .root_template
        .as_ref()
        .filter(|_| program.root_template_modes.is_empty())
    {
        let variables = bind_template_parameters(root_template, &BTreeMap::new(), &globals.atomics);
        execute_sequence(
            &inputs,
            &root_template.body,
            Some(source.document_node()),
            &variables,
            0,
            control,
        )?
    } else {
        apply_template(
            program,
            source,
            source.document_node(),
            None,
            request_id,
            &globals,
            control,
        )?
    };
    Ok(SemanticResult { children })
}

#[cfg(test)]
fn execute_initial_mode(
    program: &StylesheetProgram,
    source: &Document,
    name: &str,
    parameters: &BTreeMap<String, InvocationParameter>,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    if !program_has_mode(program, name) {
        return Err(failure(
            "XTDE0045",
            FailureCategory::Invalid,
            Some(request_id),
            format!("unknown initial mode: {name}"),
        ));
    }
    let globals =
        materialize_global_defaults(program, Some(source), parameters, request_id, control)?;
    let children = if program.root_template_modes.iter().any(|mode| mode == name) {
        let template = program
            .root_template
            .as_ref()
            .expect("a compiled root initial mode has a root template");
        let inputs = SequenceInputs {
            program,
            source: Some(source),
            request_id,
            globals: &globals,
        };
        let variables = bind_template_parameters(template, parameters, &globals.atomics);
        execute_sequence(
            &inputs,
            &template.body,
            Some(source.document_node()),
            &variables,
            0,
            control,
        )?
    } else {
        apply_template(
            program,
            source,
            source.document_node(),
            Some(name),
            request_id,
            &globals,
            control,
        )?
    };
    Ok(SemanticResult { children })
}

#[cfg(test)]
fn program_has_mode(program: &StylesheetProgram, name: &str) -> bool {
    program.root_template_modes.iter().any(|mode| mode == name)
        || program
            .matched_templates
            .iter()
            .any(|template| template.modes.iter().any(|mode| mode == name))
}

#[cfg(test)]
fn execute_initial_template(
    program: &StylesheetProgram,
    name: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    let template = program
        .named_templates
        .iter()
        .find(|template| template.name == name)
        .expect("initial-template entries are validated during request admission");
    if !template.parameters.is_empty() {
        return Err(failure(
            "FXRT1003",
            FailureCategory::Unsupported,
            Some(request_id),
            "initial-template parameters are outside the private invocation-entry slice",
        ));
    }
    let globals =
        materialize_global_defaults(program, None, &BTreeMap::new(), request_id, control)?;
    let inputs = SequenceInputs {
        program,
        source: None,
        request_id,
        globals: &globals,
    };
    let children = execute_sequence(
        &inputs,
        &template.template.body,
        None,
        &RuntimeVariables::from_atomics(&globals.atomics),
        0,
        control,
    )?;
    Ok(SemanticResult { children })
}

fn execute_sequence(
    inputs: &SequenceInputs<'_>,
    instructions: &[Instruction],
    context: Option<crate::xdm::owned_tree_experiment::NodeId>,
    variables: &RuntimeVariables,
    call_depth: usize,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (mut result, mut scope) = (Vec::new(), variables.clone());
    for instruction in instructions {
        charge_xslt_instruction(control, inputs.request_id)?;
        match instruction {
            Instruction::LiteralElement {
                name,
                namespaces,
                body,
                ..
            } => {
                control
                    .charge(WorkDomain::ResultNode, 1)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                result.push(ResultNode::Element {
                    name: name.clone(),
                    namespaces: namespaces.clone(),
                    children: execute_sequence(inputs, body, context, &scope, call_depth, control)?,
                });
            }
            Instruction::Text { value, .. } => {
                append_text(&mut result, value, inputs.request_id, control)?;
            }
            Instruction::ValueOf {
                select, separator, ..
            } => {
                execute_value_of(
                    inputs,
                    select,
                    separator,
                    context,
                    &scope,
                    &mut result,
                    control,
                )?;
            }
            Instruction::SequenceNodes { select, .. } => {
                result.extend(execute_sequence_nodes(inputs, select, context, control)?);
            }
            Instruction::SequenceItems { select, .. } => {
                result.extend(execute_sequence_items(
                    inputs, select, context, &scope, control,
                )?);
            }
            Instruction::Variable { name, select, .. } => {
                let value = execute_variable_binding(inputs, name, select, context, control)?;
                scope.atomics.insert(name.clone(), value);
            }
            Instruction::IntegerRangeVariable {
                name, start, end, ..
            } => {
                let values = materialize_integer_range(*start, *end, inputs.request_id, control)?;
                scope.atomic_sequences.insert(name.clone(), values);
            }
            Instruction::ApplyTemplates { select, mode, .. } => {
                result.extend(execute_apply_templates(
                    inputs,
                    select.as_ref(),
                    mode.as_deref(),
                    context,
                    control,
                )?);
            }
            Instruction::If { test, body, .. } => {
                result.extend(execute_if(
                    inputs, test, body, context, &scope, call_depth, control,
                )?);
            }
            Instruction::Choose {
                branches,
                otherwise,
                ..
            } => {
                result.extend(execute_choose(
                    inputs, branches, otherwise, context, &scope, call_depth, control,
                )?);
            }
            Instruction::CallTemplate {
                name, arguments, ..
            } => {
                result.extend(execute_named_call(
                    inputs, name, arguments, context, call_depth, control,
                )?);
            }
            Instruction::Copy { .. } => {
                return Err(failure(
                    "FXRT1007",
                    FailureCategory::Unsupported,
                    Some(inputs.request_id),
                    "xsl:copy requires a temporary-tree context in this private slice",
                ));
            }
        }
    }
    Ok(result)
}

fn execute_apply_templates(
    inputs: &SequenceInputs<'_>,
    select: Option<&ApplySelection>,
    mode: Option<&str>,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if let Some(ApplySelection::GlobalTemporaryChildren(name)) = select {
        let tree = inputs.globals.temporary_trees.get(name).ok_or_else(|| {
            failure(
                "FXRT0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("unbound temporary tree: ${name}"),
            )
        })?;
        let mut result = Vec::new();
        for node in &tree.roots {
            result.extend(apply_temporary_template(
                inputs, tree, *node, mode, control,
            )?);
        }
        return Ok(result);
    }
    let (source, context) = required_source_context(inputs, context)?;
    let selected = select_apply_nodes(inputs, select, context, control)?;
    let mut result = Vec::new();
    for node in selected {
        result.extend(apply_template(
            inputs.program,
            source,
            node,
            mode,
            inputs.request_id,
            inputs.globals,
            control,
        )?);
    }
    Ok(result)
}

fn charge_xslt_instruction(
    control: &mut InvocationControl,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    control
        .charge(WorkDomain::XsltInstruction, 1)
        .map_err(|failure| control_failure(failure, request_id))
}

fn execute_sequence_nodes(
    inputs: &SequenceInputs<'_>,
    select: &ForDistinctValuesExpression,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (source, _) = required_source_context(inputs, context)?;
    let selected = evaluate_for_distinct_values(select, source, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let mut result = Vec::new();
    for node in selected {
        result.extend(copy_source_node(source, inputs.request_id, node, control)?);
    }
    Ok(result)
}

fn execute_sequence_items(
    inputs: &SequenceInputs<'_>,
    select: &[SequenceItemExpression],
    context: Option<NodeId>,
    variables: &RuntimeVariables,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let mut result = Vec::new();
    let mut previous_was_atomic = false;
    for item in select {
        match item {
            SequenceItemExpression::ChildElements => {
                let (source, context) = required_source_context(inputs, context)?;
                for child in source.children(context).iter().copied() {
                    control
                        .charge(WorkDomain::XPathNodeVisit, 1)
                        .map_err(|failure| control_failure(failure, inputs.request_id))?;
                    if source.kind(child) == NodeKind::Element {
                        result.extend(copy_source_node(source, inputs.request_id, child, control)?);
                    }
                }
                previous_was_atomic = false;
            }
            SequenceItemExpression::Variable(name) => {
                let values = variable_atomic_values(variables, name, inputs.request_id)?;
                for value in values {
                    if previous_was_atomic {
                        append_text(&mut result, " ", inputs.request_id, control)?;
                    }
                    append_text(&mut result, value.lexical(), inputs.request_id, control)?;
                    previous_was_atomic = true;
                }
            }
        }
    }
    Ok(result)
}

fn execute_if(
    inputs: &SequenceInputs<'_>,
    test: &BooleanExpression,
    body: &[Instruction],
    context: Option<NodeId>,
    variables: &RuntimeVariables,
    call_depth: usize,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if evaluate_boolean(test, variables, inputs.request_id)? {
        execute_sequence(inputs, body, context, variables, call_depth, control)
    } else {
        Ok(Vec::new())
    }
}

fn execute_choose(
    inputs: &SequenceInputs<'_>,
    branches: &[crate::xslt::golden_semantics_experiment::ChooseBranch],
    otherwise: &[Instruction],
    context: Option<NodeId>,
    variables: &RuntimeVariables,
    call_depth: usize,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    for branch in branches {
        if evaluate_boolean(&branch.test, variables, inputs.request_id)? {
            return execute_sequence(
                inputs,
                &branch.body,
                context,
                variables,
                call_depth,
                control,
            );
        }
    }
    execute_sequence(inputs, otherwise, context, variables, call_depth, control)
}

fn evaluate_boolean(
    expression: &BooleanExpression,
    variables: &RuntimeVariables,
    request_id: &str,
) -> Result<bool, ExecutionFailure> {
    match expression {
        BooleanExpression::Constant(value) => Ok(*value),
        BooleanExpression::VariableEqualsInteger(test) => {
            let value = variables.atomics.get(&test.variable).ok_or_else(|| {
                failure(
                    "FXRT0002",
                    FailureCategory::Invalid,
                    Some(request_id),
                    format!("unbound variable: ${}", test.variable),
                )
            })?;
            Ok(value.lexical().trim().parse::<i64>() == Ok(test.integer))
        }
    }
}

fn execute_variable_binding(
    inputs: &SequenceInputs<'_>,
    name: &str,
    select: &CastExpression,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<AtomicValue, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    evaluate_cast(select, source, context, control).map_err(|evaluation_failure| {
        match evaluation_failure {
            CastEvaluationFailure::Control(control) => control_failure(control, inputs.request_id),
            CastEvaluationFailure::InvalidValue => failure(
                "FXRT0006",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("the value selected for ${name} cannot be cast to its target type"),
            ),
        }
    })
}

fn materialize_integer_range(
    start: i64,
    end: i64,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<AtomicValue>, ExecutionFailure> {
    let mut values = Vec::new();
    if start > end {
        return Ok(values);
    }
    let mut value = start;
    loop {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        values.push(AtomicValue::from_validated_lexical(
            BuiltinAtomicType::Integer,
            value.to_string(),
        ));
        if value == end {
            break;
        }
        value = value.checked_add(1).ok_or_else(|| {
            failure(
                "FXRT0007",
                FailureCategory::Invalid,
                Some(request_id),
                "integer range overflowed during materialization",
            )
        })?;
    }
    Ok(values)
}

fn variable_atomic_values<'a>(
    variables: &'a RuntimeVariables,
    name: &str,
    request_id: &str,
) -> Result<Vec<&'a AtomicValue>, ExecutionFailure> {
    if let Some(value) = variables.atomics.get(name) {
        return Ok(vec![value]);
    }
    if let Some(values) = variables.atomic_sequences.get(name) {
        return Ok(values.iter().collect());
    }
    Err(failure(
        "FXRT0002",
        FailureCategory::Invalid,
        Some(request_id),
        format!("unbound variable: ${name}"),
    ))
}

fn copy_source_node(
    source: &Document,
    request_id: &str,
    node: NodeId,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    match source.kind(node) {
        NodeKind::Element => {
            if !source.attributes(node).is_empty() {
                return Err(failure(
                    "FXRT1002",
                    FailureCategory::Unsupported,
                    Some(request_id),
                    "copying selected source attributes is outside the private xsl:sequence slice",
                ));
            }
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            let mut children = Vec::new();
            for child in source.children(node).iter().copied() {
                children.extend(copy_source_node(source, request_id, child, control)?);
            }
            Ok(vec![ResultNode::Element {
                name: source
                    .name(node)
                    .expect("source element nodes have names")
                    .clone(),
                namespaces: Vec::new(),
                children,
            }])
        }
        NodeKind::Text => {
            let mut copied = Vec::new();
            append_text(
                &mut copied,
                source.value(node).unwrap_or_default(),
                request_id,
                control,
            )?;
            Ok(copied)
        }
        NodeKind::Document
        | NodeKind::Attribute
        | NodeKind::Comment
        | NodeKind::ProcessingInstruction => Err(failure(
            "FXRT1002",
            FailureCategory::Unsupported,
            Some(request_id),
            "the selected source node kind is outside the private xsl:sequence copy slice",
        )),
    }
}

fn select_apply_nodes(
    inputs: &SequenceInputs<'_>,
    select: Option<&ApplySelection>,
    context: NodeId,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ExecutionFailure> {
    let source = inputs.source.expect("apply selection requires a source");
    let Some(select) = select else {
        return Ok(source.children(context).to_vec());
    };
    match select {
        ApplySelection::LocationPath(path) => {
            evaluate_location_path_controlled(source, context, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))
        }
        ApplySelection::ChildElement(name) => {
            let mut selected = Vec::new();
            for child in source.children(context).iter().copied() {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                if source.kind(child) == NodeKind::Element && source.name(child) == Some(name) {
                    selected.push(child);
                }
            }
            Ok(selected)
        }
        ApplySelection::ChildNodes(node_test) => {
            let mut selected = Vec::new();
            for child in source.children(context).iter().copied() {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                let matches = match node_test {
                    NodeTest::Comment => source.kind(child) == NodeKind::Comment,
                    NodeTest::ProcessingInstruction => {
                        source.kind(child) == NodeKind::ProcessingInstruction
                    }
                    NodeTest::AnyNode => true,
                };
                if matches {
                    selected.push(child);
                }
            }
            Ok(selected)
        }
        ApplySelection::Attribute(name) => {
            let mut selected = Vec::new();
            for attribute in source.attributes(context).iter().copied() {
                control
                    .charge(WorkDomain::XPathNodeVisit, 1)
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                if source.name(attribute) == Some(name) {
                    selected.push(attribute);
                }
            }
            Ok(selected)
        }
        ApplySelection::GlobalTemporaryChildren(_) => unreachable!(
            "global temporary-tree selection is dispatched before principal-source selection"
        ),
    }
}

fn execute_named_call(
    inputs: &SequenceInputs<'_>,
    name: &str,
    arguments: &[TemplateArgument],
    context: Option<NodeId>,
    call_depth: usize,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    if call_depth >= MAX_NAMED_TEMPLATE_CALL_DEPTH {
        return Err(failure(
            "FXRT0003",
            FailureCategory::Limit,
            Some(inputs.request_id),
            format!(
                "named-template call depth exceeds private limit {MAX_NAMED_TEMPLATE_CALL_DEPTH}"
            ),
        ));
    }
    let target = inputs
        .program
        .named_templates
        .iter()
        .find(|template| template.name == name)
        .expect("named-template references were validated during compilation");
    let mut frame = RuntimeVariables::default();
    frame.atomics.extend(
        target
            .parameters
            .iter()
            .map(|parameter| (parameter.clone(), AtomicValue::string(""))),
    );
    for argument in arguments {
        frame.atomics.insert(
            argument.name.clone(),
            AtomicValue::string(argument.value.clone()),
        );
    }
    execute_sequence(
        inputs,
        &target.template.body,
        context,
        &frame,
        call_depth + 1,
        control,
    )
}

fn apply_template(
    program: &StylesheetProgram,
    source: &Document,
    node: NodeId,
    mode: Option<&str>,
    request_id: &str,
    globals: &RuntimeGlobals,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    control
        .charge(WorkDomain::XsltInstruction, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    if let Some(template) = select_template(program, source, node, mode, request_id, control)? {
        let inputs = SequenceInputs {
            program,
            source: Some(source),
            request_id,
            globals,
        };
        let variables =
            bind_template_parameters(&template.template, &BTreeMap::new(), &globals.atomics);
        return execute_sequence(
            &inputs,
            &template.template.body,
            Some(node),
            &variables,
            0,
            control,
        );
    }

    match source.kind(node) {
        NodeKind::Document | NodeKind::Element => {
            let mut result = Vec::new();
            for child in source.children(node) {
                result.extend(apply_template(
                    program, source, *child, mode, request_id, globals, control,
                )?);
            }
            Ok(result)
        }
        NodeKind::Text | NodeKind::Attribute => {
            let mut result = Vec::new();
            append_text(
                &mut result,
                source.value(node).unwrap_or_default(),
                request_id,
                control,
            )?;
            Ok(result)
        }
        NodeKind::Comment | NodeKind::ProcessingInstruction => Ok(Vec::new()),
    }
}

fn apply_temporary_template(
    inputs: &SequenceInputs<'_>,
    tree: &TemporaryTree,
    node: usize,
    mode: Option<&str>,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    charge_xslt_instruction(control, inputs.request_id)?;
    let temporary = &tree.nodes[node];
    let template = inputs
        .program
        .matched_templates
        .iter()
        .rev()
        .find(|template| {
            template_accepts_mode(&template.modes, mode)
                && match &template.pattern {
                    MatchPattern::Element(name) => &temporary.name == name,
                    MatchPattern::AnyElement | MatchPattern::AnyNode => true,
                    _ => false,
                }
        });
    if let Some(template) = template {
        if matches!(
            template.template.body.as_slice(),
            [Instruction::Copy { .. }]
        ) {
            return Ok(vec![copy_temporary_node(
                tree,
                node,
                inputs.request_id,
                control,
            )?]);
        }
        return Err(failure(
            "FXRT1007",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "temporary-tree template bodies other than xsl:copy are outside the private slice",
        ));
    }
    let mut result = Vec::new();
    for child in &temporary.children {
        result.extend(apply_temporary_template(
            inputs, tree, *child, mode, control,
        )?);
    }
    Ok(result)
}

fn copy_temporary_node(
    tree: &TemporaryTree,
    node: usize,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<ResultNode, ExecutionFailure> {
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let node = &tree.nodes[node];
    let children = node
        .children
        .iter()
        .map(|child| copy_temporary_node(tree, *child, request_id, control))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ResultNode::Element {
        name: node.name.clone(),
        namespaces: node.namespaces.clone(),
        children,
    })
}

fn append_text(
    nodes: &mut Vec<ResultNode>,
    value: &str,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    if value.is_empty() {
        return Ok(());
    }
    if !matches!(nodes.last(), Some(ResultNode::Text(_))) {
        control
            .charge(WorkDomain::ResultNode, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
    }
    control
        .charge(WorkDomain::ResultTextByte, value.len())
        .map_err(|failure| control_failure(failure, request_id))?;
    if let Some(ResultNode::Text(existing)) = nodes.last_mut() {
        existing.push_str(value);
    } else {
        nodes.push(ResultNode::Text(value.to_owned()));
    }
    Ok(())
}

#[cfg(test)]
#[path = "golden_runtime_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "golden_runtime_control_tests.rs"]
mod control_phase_tests;

#[cfg(test)]
#[path = "golden_runtime_workflow_tests.rs"]
mod workflow_tests;

#[cfg(test)]
#[path = "golden_runtime_xslt30_tests.rs"]
mod xslt30_tests;

#[cfg(test)]
#[path = "xslt30_apply_templates_inventory_tests.rs"]
mod xslt30_apply_templates_inventory_tests;

#[cfg(test)]
#[path = "xslt30_for_inventory_tests.rs"]
mod xslt30_for_inventory_tests;

#[cfg(test)]
#[path = "xslt30_castable_inventory_tests.rs"]
mod xslt30_castable_inventory_tests;
#[cfg(test)]
#[path = "xslt30_data_manipulation_inventory_tests.rs"]
mod xslt30_data_manipulation_inventory_tests;
#[cfg(test)]
#[path = "xslt30_initial_mode_inventory_tests.rs"]
mod xslt30_initial_mode_inventory_tests;

#[cfg(test)]
#[path = "xslt30_output_inventory_tests.rs"]
mod xslt30_output_inventory_tests;

#[cfg(test)]
#[path = "xslt30_include_inventory_tests.rs"]
mod xslt30_include_inventory_tests;

#[cfg(test)]
#[path = "xslt30_deep_equal_inventory_tests.rs"]
mod xslt30_deep_equal_inventory_tests;
