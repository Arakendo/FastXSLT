use std::collections::BTreeMap;
#[cfg(test)]
use std::collections::HashSet;

use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
#[cfg(test)]
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::resources::ResourceSnapshot;
use crate::xdm::atomic_value_experiment::AtomicValue;
#[cfg(test)]
use crate::xdm::owned_tree_experiment::BuildFailure;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, StringValueVisitFailure};
#[cfg(test)]
use crate::xml::quick_xml_experiment::parse_document_controlled;
use crate::xml::quick_xml_experiment::{ExpandedName, ParseLimits, parse_document};
use crate::xpath::castable_experiment::{
    CastEvaluationFailure, CastExpression, CastableExpression, evaluate as evaluate_castable,
    evaluate_cast, evaluate_value as evaluate_castable_value,
    variable_name as castable_variable_name,
};
use crate::xpath::decimal_sum_for_experiment::{
    DecimalSumEvaluationFailure, evaluate as evaluate_decimal_sum_for,
};
use crate::xpath::focus_sum_for_experiment::{
    FocusSumEvaluationFailure, evaluate as evaluate_focus_sum_for,
};
use crate::xpath::for_distinct_values_experiment::{
    ForDistinctValuesExpression, evaluate as evaluate_for_distinct_values,
};
use crate::xpath::integer_for_experiment::evaluate as evaluate_integer_for;
use crate::xpath::path_experiment::evaluate_child_path_controlled;
use crate::xslt::golden_semantics_experiment::{
    ApplySelection, BooleanExpression, Instruction, MatchPattern, NodeTest, StylesheetProgram,
    TemplateArgument, ValueExpression,
};

mod serialization;

pub(super) use serialization::serialize_xml;

const XML_LIMITS: ParseLimits = ParseLimits {
    max_events: 1_024,
    max_depth: 64,
};
const MAX_NAMED_TEMPLATE_CALL_DEPTH: usize = 256;

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

#[derive(Debug)]
#[cfg(test)]
enum InvocationEntry {
    PrincipalSource { resource: String },
    InitialTemplate { name: String },
}

#[derive(Debug)]
#[cfg(test)]
struct TransformRequest {
    identity: String,
    result_identity: String,
    entry: InvocationEntry,
    cancellation: CancellationToken,
    cancellation_fault: Option<(WorkDomain, usize)>,
}

#[derive(Debug)]
#[cfg(test)]
struct TransformSetBuilder {
    snapshot: ResourceSnapshot,
    stylesheet: StylesheetProgram,
    requests: Vec<TransformRequest>,
    request_ids: HashSet<String>,
    result_ids: HashSet<String>,
    request_limit: usize,
    policy: ExecutionPolicy,
}

#[derive(Debug)]
#[cfg(test)]
struct TransformSet {
    snapshot: ResourceSnapshot,
    stylesheet: StylesheetProgram,
    requests: Vec<TransformRequest>,
    policy: ExecutionPolicy,
}

#[derive(Debug, Clone)]
#[cfg(test)]
struct ExecutionPolicy {
    denied_sources: HashSet<String>,
    serialized_byte_limit: usize,
    work_limits: WorkLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg(test)]
struct ResultEntry {
    result_id: String,
    semantic: SemanticResult,
    serialized: String,
}

#[derive(Debug, PartialEq, Eq)]
#[cfg(test)]
struct ResultSet {
    by_request: BTreeMap<String, ResultEntry>,
    completion_order: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailureCategory {
    Invalid,
    Unsupported,
    MissingResource,
    #[cfg(test)]
    Denied,
    Limit,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExecutionFailure {
    code: &'static str,
    category: FailureCategory,
    request_id: Option<String>,
    work_domain: Option<WorkDomain>,
    detail: String,
}

#[cfg(feature = "workbench")]
impl ExecutionFailure {
    pub(super) fn workbench_parts(&self) -> (&'static str, &'static str, Option<&str>, &str) {
        let category = match self.category {
            FailureCategory::Invalid => "invalid",
            FailureCategory::Unsupported => "unsupported",
            FailureCategory::MissingResource => "missing-resource",
            #[cfg(test)]
            FailureCategory::Denied => "denied",
            FailureCategory::Limit => "limit",
            FailureCategory::Cancelled => "cancelled",
        };
        (
            self.code,
            category,
            self.request_id.as_deref(),
            &self.detail,
        )
    }
}

#[cfg(test)]
impl TransformSetBuilder {
    fn new(
        snapshot: ResourceSnapshot,
        stylesheet: StylesheetProgram,
        request_limit: usize,
        policy: ExecutionPolicy,
    ) -> Self {
        Self {
            snapshot,
            stylesheet,
            requests: Vec::new(),
            request_ids: HashSet::new(),
            result_ids: HashSet::new(),
            request_limit,
            policy,
        }
    }

    fn add(&mut self, request: TransformRequest) -> Result<(), ExecutionFailure> {
        if self.requests.len() >= self.request_limit {
            return Err(failure(
                "FXBT0001",
                FailureCategory::Limit,
                Some(&request.identity),
                format!("transform-set request limit is {}", self.request_limit),
            ));
        }
        if !self.request_ids.insert(request.identity.clone()) {
            return Err(failure(
                "FXBT0002",
                FailureCategory::Invalid,
                Some(&request.identity),
                "duplicate request identity",
            ));
        }
        if !self.result_ids.insert(request.result_identity.clone()) {
            self.request_ids.remove(&request.identity);
            return Err(failure(
                "FXBT0003",
                FailureCategory::Invalid,
                Some(&request.identity),
                "duplicate result identity",
            ));
        }
        match &request.entry {
            InvocationEntry::PrincipalSource { resource } => {
                if self.policy.denied_sources.contains(resource) {
                    self.request_ids.remove(&request.identity);
                    self.result_ids.remove(&request.result_identity);
                    return Err(failure(
                        "FXRS0003",
                        FailureCategory::Denied,
                        Some(&request.identity),
                        format!("source authority is denied: {resource}"),
                    ));
                }
                if self.snapshot.get(resource).is_none() {
                    self.request_ids.remove(&request.identity);
                    self.result_ids.remove(&request.result_identity);
                    return Err(failure(
                        "FXRS0001",
                        FailureCategory::MissingResource,
                        Some(&request.identity),
                        format!("source is not admitted: {resource}"),
                    ));
                }
            }
            InvocationEntry::InitialTemplate { name } => {
                if !self
                    .stylesheet
                    .named_templates
                    .iter()
                    .any(|template| template.name == *name)
                {
                    self.request_ids.remove(&request.identity);
                    self.result_ids.remove(&request.result_identity);
                    return Err(failure(
                        "FXRT0004",
                        FailureCategory::Invalid,
                        Some(&request.identity),
                        format!("unknown initial template: {name}"),
                    ));
                }
            }
        }
        self.requests.push(request);
        Ok(())
    }

    fn seal(self) -> TransformSet {
        TransformSet {
            snapshot: self.snapshot,
            stylesheet: self.stylesheet,
            requests: self.requests,
            policy: self.policy,
        }
    }
}

pub(super) fn compile_resource(
    snapshot: &ResourceSnapshot,
    stylesheet_id: &str,
) -> Result<StylesheetProgram, ExecutionFailure> {
    let bytes = snapshot.get(stylesheet_id).ok_or_else(|| {
        failure(
            "FXRS0002",
            FailureCategory::MissingResource,
            None,
            format!("stylesheet is not admitted: {stylesheet_id}"),
        )
    })?;
    let parsed = parse_document(stylesheet_id, bytes, XML_LIMITS).map_err(|error| {
        failure(
            "FXXM0001",
            FailureCategory::Invalid,
            None,
            format!("stylesheet XML is invalid: {error:?}"),
        )
    })?;
    let document = Document::from_parsed(parsed).map_err(|error| {
        failure(
            "FXXD0001",
            FailureCategory::Invalid,
            None,
            format!("stylesheet XDM construction failed: {error:?}"),
        )
    })?;
    compile_stylesheet(&document).map_err(|error| {
        failure(
            error.code,
            match error.category {
                crate::compile::golden_stylesheet_experiment::CompileCategory::Invalid => {
                    FailureCategory::Invalid
                }
                crate::compile::golden_stylesheet_experiment::CompileCategory::Unsupported => {
                    FailureCategory::Unsupported
                }
            },
            None,
            format!(
                "{} at {}:{}..{}",
                error.detail,
                error.location.resource,
                error.location.span.start,
                error.location.span.end
            ),
        )
    })
}

#[cfg(test)]
fn execute_transform_set(set: TransformSet) -> Result<ResultSet, ExecutionFailure> {
    let mut by_request = BTreeMap::new();
    let mut completion_order = Vec::new();

    for request in set.requests.into_iter().rev() {
        let mut control =
            InvocationControl::new(request.cancellation.clone(), set.policy.work_limits);
        if let Some((domain, accepted_charges_before_signal)) = request.cancellation_fault {
            control = control.cancelling_on_charge(domain, accepted_charges_before_signal);
        }
        let semantic = match &request.entry {
            InvocationEntry::PrincipalSource { resource } => {
                let bytes = set
                    .snapshot
                    .get(resource)
                    .expect("sealed transform sets contain admitted sources");
                let parsed = parse_document_controlled(resource, bytes, XML_LIMITS, &mut control)
                    .map_err(|error| {
                    error.control_failure().map_or_else(
                        || {
                            failure(
                                "FXXM0002",
                                FailureCategory::Invalid,
                                Some(&request.identity),
                                format!("source XML is invalid: {error:?}"),
                            )
                        },
                        |failure| control_failure(*failure, &request.identity),
                    )
                })?;
                let source =
                    Document::from_parsed_controlled(parsed, &mut control).map_err(|error| {
                        match error {
                            BuildFailure::Control(failure) => {
                                control_failure(failure, &request.identity)
                            }
                            _ => failure(
                                "FXXD0002",
                                FailureCategory::Invalid,
                                Some(&request.identity),
                                format!("source XDM construction failed: {error:?}"),
                            ),
                        }
                    })?;
                execute_program(&set.stylesheet, &source, &request.identity, &mut control)?
            }
            InvocationEntry::InitialTemplate { name } => {
                execute_initial_template(&set.stylesheet, name, &request.identity, &mut control)?
            }
        };
        let serialized = serialize_xml(
            &semantic,
            &set.stylesheet.output,
            &request.identity,
            set.policy.serialized_byte_limit,
            &mut control,
        )?;
        completion_order.push(request.identity.clone());
        by_request.insert(
            request.identity,
            ResultEntry {
                result_id: request.result_identity,
                semantic,
                serialized,
            },
        );
    }
    Ok(ResultSet {
        by_request,
        completion_order,
    })
}

pub(super) fn execute_program(
    program: &StylesheetProgram,
    source: &Document,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<SemanticResult, ExecutionFailure> {
    let variables = BTreeMap::new();
    let inputs = SequenceInputs {
        program,
        source: Some(source),
        request_id,
    };
    let children = if let Some(root_template) = &program.root_template {
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
            control,
        )?
    };
    Ok(SemanticResult { children })
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
    let inputs = SequenceInputs {
        program,
        source: None,
        request_id,
    };
    let children = execute_sequence(
        &inputs,
        &template.template.body,
        None,
        &BTreeMap::new(),
        0,
        control,
    )?;
    Ok(SemanticResult { children })
}

struct SequenceInputs<'a> {
    program: &'a StylesheetProgram,
    source: Option<&'a Document>,
    request_id: &'a str,
}

fn required_source_context<'a>(
    inputs: &SequenceInputs<'a>,
    context: Option<NodeId>,
) -> Result<(&'a Document, NodeId), ExecutionFailure> {
    let source = inputs.source.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "the instruction requires a principal source and context item",
        )
    })?;
    let context = context.ok_or_else(|| {
        failure(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(inputs.request_id),
            "the instruction requires a principal source and context item",
        )
    })?;
    Ok((source, context))
}

fn execute_sequence(
    inputs: &SequenceInputs<'_>,
    instructions: &[Instruction],
    context: Option<crate::xdm::owned_tree_experiment::NodeId>,
    variables: &BTreeMap<String, AtomicValue>,
    call_depth: usize,
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let (mut result, mut scoped_variables) = (Vec::new(), variables.clone());
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
                    children: execute_sequence(
                        inputs,
                        body,
                        context,
                        &scoped_variables,
                        call_depth,
                        control,
                    )?,
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
                    &scoped_variables,
                    &mut result,
                    control,
                )?;
            }
            Instruction::SequenceNodes { select, .. } => {
                result.extend(execute_sequence_nodes(inputs, select, context, control)?);
            }
            Instruction::Variable { name, select, .. } => {
                let value = execute_variable_binding(inputs, name, select, context, control)?;
                scoped_variables.insert(name.clone(), value);
            }
            Instruction::ApplyTemplates { select, mode, .. } => {
                let (source, context) = required_source_context(inputs, context)?;
                let selected = select_apply_nodes(inputs, select.as_ref(), context, control)?;
                for selected_node in selected {
                    result.extend(apply_template(
                        inputs.program,
                        source,
                        selected_node,
                        mode.as_deref(),
                        inputs.request_id,
                        control,
                    )?);
                }
            }
            Instruction::If { test, body, .. } => {
                result.extend(execute_if(
                    inputs,
                    test,
                    body,
                    context,
                    &scoped_variables,
                    call_depth,
                    control,
                )?);
            }
            Instruction::Choose {
                branches,
                otherwise,
                ..
            } => {
                result.extend(execute_choose(
                    inputs,
                    branches,
                    otherwise,
                    context,
                    &scoped_variables,
                    call_depth,
                    control,
                )?);
            }
            Instruction::CallTemplate {
                name, arguments, ..
            } => {
                result.extend(execute_named_call(
                    inputs, name, arguments, context, call_depth, control,
                )?);
            }
        }
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

fn execute_if(
    inputs: &SequenceInputs<'_>,
    test: &BooleanExpression,
    body: &[Instruction],
    context: Option<NodeId>,
    variables: &BTreeMap<String, AtomicValue>,
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
    variables: &BTreeMap<String, AtomicValue>,
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
    variables: &BTreeMap<String, AtomicValue>,
    request_id: &str,
) -> Result<bool, ExecutionFailure> {
    match expression {
        BooleanExpression::Constant(value) => Ok(*value),
        BooleanExpression::VariableEqualsInteger(test) => {
            let value = variables.get(&test.variable).ok_or_else(|| {
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

fn execute_value_of(
    inputs: &SequenceInputs<'_>,
    select: &ValueExpression,
    separator: &str,
    context: Option<NodeId>,
    variables: &BTreeMap<String, AtomicValue>,
    result: &mut Vec<ResultNode>,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    match select {
        ValueExpression::ChildPath(path) => {
            let (source, context) = required_source_context(inputs, context)?;
            let selected = evaluate_child_path_controlled(source, context, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            if selected.len() > 1 {
                return Err(failure(
                    "FXRT1001",
                    FailureCategory::Unsupported,
                    Some(inputs.request_id),
                    "the private value-of slice does not define multi-node conversion",
                ));
            }
            if let Some(node) = selected.first() {
                source
                    .visit_string_value_controlled(*node, control, &mut |part, control| {
                        append_text(result, part, inputs.request_id, control)
                    })
                    .map_err(|failure| match failure {
                        StringValueVisitFailure::Control(failure) => {
                            control_failure(failure, inputs.request_id)
                        }
                        StringValueVisitFailure::Sink(failure) => failure,
                    })?;
            }
        }
        ValueExpression::Variable(name) => {
            let value = variables.get(name).ok_or_else(|| {
                failure(
                    "FXRT0002",
                    FailureCategory::Invalid,
                    Some(inputs.request_id),
                    format!("unbound variable: ${name}"),
                )
            })?;
            append_text(result, value.lexical(), inputs.request_id, control)?;
        }
        ValueExpression::IntegerFor(expression) => {
            let values = evaluate_integer_for(expression, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))?;
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    append_text(result, separator, inputs.request_id, control)?;
                }
                append_text(result, &value.to_string(), inputs.request_id, control)?;
            }
        }
        ValueExpression::FocusSumFor(expression) => {
            let (source, context) = required_source_context(inputs, context)?;
            let value = evaluate_focus_sum_for(expression, source, context, control).map_err(
                |evaluation_failure| match evaluation_failure {
                    FocusSumEvaluationFailure::Control(failure) => {
                        control_failure(failure, inputs.request_id)
                    }
                    FocusSumEvaluationFailure::Unsupported => failure(
                        "FXRT1005",
                        FailureCategory::Unsupported,
                        Some(inputs.request_id),
                        "non-empty numeric multiplication is outside the private focus-preserving sum slice",
                    ),
                },
            )?;
            append_text(result, &value.to_string(), inputs.request_id, control)?;
        }
        ValueExpression::DecimalSumFor(expression) => {
            let value = execute_decimal_sum(inputs, expression, context, control)?;
            append_text(result, &value, inputs.request_id, control)?;
        }
        ValueExpression::ConstantFormatNumber(expression) => {
            append_text(result, expression.formatted(), inputs.request_id, control)?;
        }
        ValueExpression::Castable(expression) => {
            let value =
                execute_castable_expression(inputs, expression, context, variables, control)?;
            append_text(
                result,
                if value { "true" } else { "false" },
                inputs.request_id,
                control,
            )?;
        }
    }
    Ok(())
}

fn execute_decimal_sum(
    inputs: &SequenceInputs<'_>,
    expression: &crate::xpath::decimal_sum_for_experiment::DecimalSumForExpression,
    context: Option<NodeId>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    evaluate_decimal_sum_for(expression, source, context, control).map_err(|evaluation_failure| {
        match evaluation_failure {
            DecimalSumEvaluationFailure::Control(control) => {
                control_failure(control, inputs.request_id)
            }
            DecimalSumEvaluationFailure::InvalidValue => failure(
                "FXRT0005",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                "an exact-decimal operand has an invalid lexical value",
            ),
            DecimalSumEvaluationFailure::Unsupported => failure(
                "FXRT1006",
                FailureCategory::Unsupported,
                Some(inputs.request_id),
                "decimal overflow or rounding is outside the private exact-decimal sum slice",
            ),
        }
    })
}

fn execute_castable_expression(
    inputs: &SequenceInputs<'_>,
    expression: &CastableExpression,
    context: Option<NodeId>,
    variables: &BTreeMap<String, AtomicValue>,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    if let Some(name) = castable_variable_name(expression) {
        let value = variables.get(name).ok_or_else(|| {
            failure(
                "FXRT0002",
                FailureCategory::Invalid,
                Some(inputs.request_id),
                format!("unbound variable: ${name}"),
            )
        })?;
        return evaluate_castable_value(expression, value, control)
            .map_err(|control| control_failure(control, inputs.request_id));
    }
    let (source, context) = required_source_context(inputs, context)?;
    evaluate_castable(expression, source, context, control)
        .map_err(|control| control_failure(control, inputs.request_id))
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
        ApplySelection::ChildPath(path) => {
            evaluate_child_path_controlled(source, context, path, control)
                .map_err(|failure| control_failure(failure, inputs.request_id))
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
    let mut frame: BTreeMap<_, _> = target
        .parameters
        .iter()
        .map(|parameter| (parameter.clone(), AtomicValue::string("")))
        .collect();
    for argument in arguments {
        frame.insert(
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
    control: &mut InvocationControl,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    control
        .charge(WorkDomain::XsltInstruction, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    let mut selected_template = None;
    let mut selected_priority = 0;
    for template in &program.matched_templates {
        if template.mode.as_deref() != mode
            || !match_pattern(&template.pattern, source, node, request_id, control)?
        {
            continue;
        }
        let priority = match template.pattern {
            MatchPattern::Path(_) => 3,
            MatchPattern::Element(_) | MatchPattern::Attribute(_) => 2,
            MatchPattern::Comment | MatchPattern::ProcessingInstruction => 1,
            MatchPattern::AnyNode => 0,
        };
        if selected_template.is_none() || priority >= selected_priority {
            selected_template = Some(template);
            selected_priority = priority;
        }
    }
    if let Some(template) = selected_template {
        let inputs = SequenceInputs {
            program,
            source: Some(source),
            request_id,
        };
        return execute_sequence(
            &inputs,
            &template.template.body,
            Some(node),
            &BTreeMap::new(),
            0,
            control,
        );
    }

    match source.kind(node) {
        NodeKind::Document | NodeKind::Element => {
            let mut result = Vec::new();
            for child in source.children(node) {
                result.extend(apply_template(
                    program, source, *child, mode, request_id, control,
                )?);
            }
            Ok(result)
        }
        NodeKind::Text => {
            let mut result = Vec::new();
            append_text(
                &mut result,
                source.value(node).unwrap_or_default(),
                request_id,
                control,
            )?;
            Ok(result)
        }
        NodeKind::Attribute | NodeKind::Comment | NodeKind::ProcessingInstruction => Ok(Vec::new()),
    }
}

fn match_pattern(
    pattern: &MatchPattern,
    source: &Document,
    node: NodeId,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    match pattern {
        MatchPattern::Element(name) => Ok(source.name(node) == Some(name)),
        MatchPattern::Path(path) => match_path_pattern(source, node, path, request_id, control),
        MatchPattern::Attribute(name) => {
            Ok(source.kind(node) == NodeKind::Attribute && source.name(node) == Some(name))
        }
        MatchPattern::Comment => Ok(source.kind(node) == NodeKind::Comment),
        MatchPattern::ProcessingInstruction => {
            Ok(source.kind(node) == NodeKind::ProcessingInstruction)
        }
        MatchPattern::AnyNode => Ok(matches!(
            source.kind(node),
            NodeKind::Element
                | NodeKind::Text
                | NodeKind::Comment
                | NodeKind::ProcessingInstruction
        )),
    }
}

fn match_path_pattern(
    source: &Document,
    node: NodeId,
    path: &crate::xpath::path_experiment::ChildPath,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<bool, ExecutionFailure> {
    let mut first_step = node;
    for _ in 1..path.steps.len() {
        let Some(parent) = source.parent(first_step) else {
            return Ok(false);
        };
        control
            .charge(WorkDomain::XPathNodeVisit, 1)
            .map_err(|failure| control_failure(failure, request_id))?;
        first_step = parent;
    }
    let Some(context) = source.parent(first_step) else {
        return Ok(false);
    };
    control
        .charge(WorkDomain::XPathNodeVisit, 1)
        .map_err(|failure| control_failure(failure, request_id))?;
    evaluate_child_path_controlled(source, context, path, control)
        .map(|selected| selected.contains(&node))
        .map_err(|failure| control_failure(failure, request_id))
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

fn failure(
    code: &'static str,
    category: FailureCategory,
    request_id: Option<&str>,
    detail: impl Into<String>,
) -> ExecutionFailure {
    ExecutionFailure {
        code,
        category,
        request_id: request_id.map(str::to_owned),
        work_domain: None,
        detail: detail.into(),
    }
}

fn control_failure(failure: ControlFailure, request_id: &str) -> ExecutionFailure {
    let work_domain = failure.domain();
    match failure {
        ControlFailure::Cancelled { .. } => ExecutionFailure {
            code: "FXCT0001",
            category: FailureCategory::Cancelled,
            request_id: Some(request_id.to_owned()),
            work_domain: Some(work_domain),
            detail: format!(
                "host cancellation observed while charging {} work",
                work_domain.name()
            ),
        },
        ControlFailure::BudgetExhausted {
            limit,
            consumed,
            attempted,
            ..
        } => ExecutionFailure {
            code: "FXCT0002",
            category: FailureCategory::Limit,
            request_id: Some(request_id.to_owned()),
            work_domain: Some(work_domain),
            detail: format!(
                "{} work budget exhausted: limit {limit}, consumed {consumed}, next charge {attempted}",
                work_domain.name()
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::execution_control_experiment::{
        CancellationToken, InvocationControl, WorkDomain, WorkLimits,
    };
    use crate::resources::{ResourceLimits, ResourceSetBuilder};

    use super::{
        ExecutionPolicy, FailureCategory, InvocationEntry, ResultNode, SemanticResult,
        TransformRequest, TransformSetBuilder, compile_resource, execute_transform_set,
        serialize_xml,
    };

    const SOURCE_ID: &str = "urn:fastxslt:golden:hello:source";
    const STYLESHEET_ID: &str = "urn:fastxslt:golden:hello:stylesheet";
    type ConfigureWorkLimits = fn(&mut WorkLimits);

    fn snapshot() -> crate::resources::ResourceSnapshot {
        let mut builder = ResourceSetBuilder::new(ResourceLimits::new(8, 4_096, 8_192));
        builder
            .admit(
                SOURCE_ID,
                include_bytes!("../../../../corpus/golden/hello/input.xml").to_vec(),
            )
            .expect("admit source");
        builder
            .admit(
                STYLESHEET_ID,
                include_bytes!("../../../../corpus/golden/hello/stylesheet.xsl").to_vec(),
            )
            .expect("admit stylesheet");
        builder.seal()
    }

    fn request(request_id: &str, result_id: &str, source_id: &str) -> TransformRequest {
        TransformRequest {
            identity: request_id.to_owned(),
            result_identity: result_id.to_owned(),
            entry: InvocationEntry::PrincipalSource {
                resource: source_id.to_owned(),
            },
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        }
    }

    fn policy(serialized_byte_limit: usize) -> ExecutionPolicy {
        ExecutionPolicy {
            denied_sources: HashSet::new(),
            serialized_byte_limit,
            work_limits: WorkLimits::unbounded(),
        }
    }

    fn execute_with_work_limits(
        request_id: &str,
        work_limits: WorkLimits,
    ) -> super::ExecutionFailure {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(
            snapshot,
            program,
            1,
            ExecutionPolicy {
                denied_sources: HashSet::new(),
                serialized_byte_limit: 4_096,
                work_limits,
            },
        );
        builder
            .add(request(request_id, "controlled-result", SOURCE_ID))
            .expect("admit controlled request");
        execute_transform_set(builder.seal()).expect_err("work limit should stop execution")
    }

    #[test]
    fn golden_transform_executes_through_an_unordered_identified_set() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 4, policy(4_096));
        builder
            .add(request("request-a", "result-a.html", SOURCE_ID))
            .expect("add first request");
        builder
            .add(request("request-b", "result-b.html", SOURCE_ID))
            .expect("add second request");

        let results = execute_transform_set(builder.seal()).expect("execute set");

        assert_eq!(results.completion_order, ["request-b", "request-a"]);
        let first = &results.by_request["request-a"];
        assert_eq!(first.result_id, "result-a.html");
        assert_eq!(
            first.semantic.children,
            [ResultNode::Element {
                name: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: "message".to_owned(),
                },
                namespaces: Vec::new(),
                children: vec![ResultNode::Text("Hello, FastXSLT!".to_owned())],
            }]
        );
        assert_eq!(first.serialized, "<message>Hello, FastXSLT!</message>");
        assert_eq!(
            format!("{}\n", first.serialized),
            include_str!("../../../../corpus/golden/hello/expected.xml")
        );
        assert_eq!(results.by_request["request-b"].semantic, first.semantic);
    }

    #[test]
    fn exact_element_templates_dispatch_repeated_nodes_in_document_order() {
        const DISPATCH_SOURCE: &str = "urn:fastxslt:golden:template-dispatch:source";
        const DISPATCH_STYLESHEET: &str = "urn:fastxslt:golden:template-dispatch:stylesheet";
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(4, 4_096, 8_192));
        resources
            .admit(
                DISPATCH_SOURCE,
                include_bytes!("../../../../corpus/golden/template-dispatch/input.xml").to_vec(),
            )
            .expect("admit dispatch source");
        resources
            .admit(
                DISPATCH_STYLESHEET,
                include_bytes!("../../../../corpus/golden/template-dispatch/stylesheet.xsl")
                    .to_vec(),
            )
            .expect("admit dispatch stylesheet");
        let snapshot = resources.seal();
        let program = compile_resource(&snapshot, DISPATCH_STYLESHEET)
            .expect("compile dispatch stylesheet once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
        builder
            .add(request(
                "dispatch-request",
                "dispatch-result",
                DISPATCH_SOURCE,
            ))
            .expect("add dispatch request");

        let results = execute_transform_set(builder.seal()).expect("execute dispatch set");

        assert_eq!(
            results.by_request["dispatch-request"].serialized,
            include_str!("../../../../corpus/golden/template-dispatch/expected.xml").trim()
        );
    }

    #[test]
    fn default_selection_uses_built_in_element_and_text_rules() {
        const BUILT_IN_SOURCE: &str = "urn:fastxslt:golden:built-in-rules:source";
        const BUILT_IN_STYLESHEET: &str = "urn:fastxslt:golden:built-in-rules:stylesheet";
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(4, 4_096, 8_192));
        resources
            .admit(
                BUILT_IN_SOURCE,
                include_bytes!("../../../../corpus/golden/built-in-template-rules/input.xml")
                    .to_vec(),
            )
            .expect("admit built-in-rule source");
        resources
            .admit(
                BUILT_IN_STYLESHEET,
                include_bytes!("../../../../corpus/golden/built-in-template-rules/stylesheet.xsl")
                    .to_vec(),
            )
            .expect("admit built-in-rule stylesheet");
        let snapshot = resources.seal();
        let program = compile_resource(&snapshot, BUILT_IN_STYLESHEET)
            .expect("compile built-in-rule stylesheet once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
        builder
            .add(request(
                "built-in-request",
                "built-in-result",
                BUILT_IN_SOURCE,
            ))
            .expect("add built-in-rule request");

        let results = execute_transform_set(builder.seal()).expect("execute built-in-rule set");

        assert_eq!(
            results.by_request["built-in-request"].serialized,
            include_str!("../../../../corpus/golden/built-in-template-rules/expected.xml").trim()
        );
    }

    #[test]
    fn named_template_recursion_stops_at_the_private_depth_limit() {
        const SOURCE: &str = "urn:fastxslt:recursion:source";
        const STYLESHEET: &str = "urn:fastxslt:recursion:stylesheet";
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
        resources
            .admit(SOURCE, b"<doc/>".to_vec())
            .expect("admit recursion source");
        resources
            .admit(
                STYLESHEET,
                br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="loop"><xsl:call-template name="loop"/></xsl:template><xsl:template match="/"><xsl:call-template name="loop"/></xsl:template></xsl:stylesheet>"#.to_vec(),
            )
            .expect("admit recursive stylesheet");
        let snapshot = resources.seal();
        let program =
            compile_resource(&snapshot, STYLESHEET).expect("compile recursive stylesheet");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
        builder
            .add(request("recursive", "recursive-result", SOURCE))
            .expect("admit recursive request");

        let failure = execute_transform_set(builder.seal())
            .expect_err("recursive call chain must stop at the private depth limit");

        assert_eq!(failure.code, "FXRT0003");
        assert_eq!(failure.category, FailureCategory::Limit);
        assert_eq!(failure.request_id.as_deref(), Some("recursive"));
    }

    #[test]
    fn batch_of_one_matches_the_same_semantic_and_serialization_path() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
        builder
            .add(request("only", "only-result", SOURCE_ID))
            .expect("add request");

        let results = execute_transform_set(builder.seal()).expect("execute one");

        assert_eq!(results.completion_order, ["only"]);
        assert_eq!(
            results.by_request["only"].serialized,
            "<message>Hello, FastXSLT!</message>"
        );
    }

    #[test]
    fn absent_output_declaration_does_not_silently_apply_html_serialization() {
        let result = SemanticResult {
            children: vec![ResultNode::Element {
                name: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: "html".to_owned(),
                },
                namespaces: Vec::new(),
                children: Vec::new(),
            }],
        };
        let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
            method: None,
            omit_xml_declaration: false,
        };

        let mut control = InvocationControl::unbounded();
        let failure = serialize_xml(&result, &settings, "html-result", 4_096, &mut control)
            .expect_err("adaptive HTML output remains unsupported");

        assert_eq!(failure.code, "FXSR1001");
        assert_eq!(failure.category, FailureCategory::Unsupported);
        assert_eq!(failure.request_id.as_deref(), Some("html-result"));
    }

    #[test]
    fn builder_rejects_duplicates_limits_and_unadmitted_sibling_results() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 2, policy(4_096));
        builder
            .add(request("first", "future.xml", SOURCE_ID))
            .expect("add first request");

        let failure = builder
            .add(request("first", "other.xml", SOURCE_ID))
            .expect_err("duplicate request should fail");
        assert_eq!(failure.code, "FXBT0002");
        assert_eq!(failure.category, FailureCategory::Invalid);

        let failure = builder
            .add(request("second", "future.xml", SOURCE_ID))
            .expect_err("duplicate result should fail");
        assert_eq!(failure.code, "FXBT0003");

        let failure = builder
            .add(request("second", "second-result", "future.xml"))
            .expect_err("a sibling result is not an admitted source");
        assert_eq!(failure.code, "FXRS0001");
        assert_eq!(failure.category, FailureCategory::MissingResource);

        builder
            .add(request("second", "second-result", SOURCE_ID))
            .expect("failed additions do not mutate the builder");
        let failure = builder
            .add(request("third", "third-result", SOURCE_ID))
            .expect_err("request limit should fail");
        assert_eq!(failure.code, "FXBT0001");
        assert_eq!(failure.category, FailureCategory::Limit);
    }

    #[test]
    fn initial_template_entry_rejects_an_unknown_compiled_name_without_a_source() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));

        let failure = builder
            .add(TransformRequest {
                identity: "unknown-entry".to_owned(),
                result_identity: "unknown-result".to_owned(),
                entry: InvocationEntry::InitialTemplate {
                    name: "missing".to_owned(),
                },
                cancellation: CancellationToken::new(),
                cancellation_fault: None,
            })
            .expect_err("unknown initial-template entry should fail admission");

        assert_eq!(failure.code, "FXRT0004");
        assert_eq!(failure.category, FailureCategory::Invalid);
        assert_eq!(failure.request_id.as_deref(), Some("unknown-entry"));
    }

    #[test]
    fn explicit_source_denial_is_distinct_from_missing_resource() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut denied_sources = HashSet::new();
        denied_sources.insert(SOURCE_ID.to_owned());
        let mut builder = TransformSetBuilder::new(
            snapshot,
            program,
            1,
            ExecutionPolicy {
                denied_sources,
                serialized_byte_limit: 4_096,
                work_limits: WorkLimits::unbounded(),
            },
        );

        let failure = builder
            .add(request("denied", "denied-result", SOURCE_ID))
            .expect_err("admitted source should still be deniable");

        assert_eq!(failure.code, "FXRS0003");
        assert_eq!(failure.category, FailureCategory::Denied);
        assert_eq!(failure.request_id.as_deref(), Some("denied"));
    }

    #[test]
    fn serialization_stops_before_exceeding_the_host_byte_limit() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(16));
        builder
            .add(request("limited", "limited-result", SOURCE_ID))
            .expect("add limited request");

        let failure = execute_transform_set(builder.seal()).expect_err("output should be limited");

        assert_eq!(failure.code, "FXSR0002");
        assert_eq!(failure.category, FailureCategory::Limit);
        assert_eq!(failure.request_id.as_deref(), Some("limited"));
        assert_eq!(failure.work_domain, None);
    }

    #[test]
    fn host_cancellation_is_observed_as_cooperative_control_not_a_budget_failure() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let token = CancellationToken::new();
        let mut controlled_request = request("cancelled", "cancelled-result", SOURCE_ID);
        controlled_request.cancellation = token.clone();
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
        builder
            .add(controlled_request)
            .expect("admit cancellable request");
        token.cancel();

        let failure =
            execute_transform_set(builder.seal()).expect_err("cancelled work should stop");

        assert_eq!(failure.code, "FXCT0001");
        assert_eq!(failure.category, FailureCategory::Cancelled);
        assert_eq!(failure.request_id.as_deref(), Some("cancelled"));
        assert_eq!(failure.work_domain, Some(WorkDomain::XmlEvent));
    }

    #[test]
    fn each_implemented_layer_charges_its_own_work_domain() {
        let cases: [(WorkDomain, ConfigureWorkLimits); 8] = [
            (WorkDomain::XmlEvent, |limits: &mut WorkLimits| {
                limits.xml_events = 0;
            }),
            (WorkDomain::XdmNode, |limits: &mut WorkLimits| {
                limits.xdm_nodes = 1;
            }),
            (WorkDomain::XPathNodeVisit, |limits: &mut WorkLimits| {
                limits.xpath_node_visits = 0;
            }),
            (WorkDomain::XdmStringValueNode, |limits: &mut WorkLimits| {
                limits.xdm_string_value_nodes = 0;
            }),
            (WorkDomain::XsltInstruction, |limits: &mut WorkLimits| {
                limits.xslt_instructions = 0;
            }),
            (WorkDomain::ResultNode, |limits: &mut WorkLimits| {
                limits.result_nodes = 0;
            }),
            (WorkDomain::ResultTextByte, |limits: &mut WorkLimits| {
                limits.result_text_bytes = 0;
            }),
            (WorkDomain::SerializedByte, |limits: &mut WorkLimits| {
                limits.serialized_bytes = 0;
            }),
        ];

        for (domain, configure) in cases {
            let mut limits = WorkLimits::unbounded();
            configure(&mut limits);
            let failure = execute_with_work_limits(domain.name(), limits);

            assert_eq!(failure.code, "FXCT0002");
            assert_eq!(failure.category, FailureCategory::Limit);
            assert_eq!(failure.request_id.as_deref(), Some(domain.name()));
            assert_eq!(failure.work_domain, Some(domain));
        }
    }
}

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
#[path = "xslt30_for_inventory_tests.rs"]
mod xslt30_for_inventory_tests;

#[cfg(test)]
#[path = "xslt30_castable_inventory_tests.rs"]
mod xslt30_castable_inventory_tests;
#[cfg(test)]
#[path = "xslt30_data_manipulation_inventory_tests.rs"]
mod xslt30_data_manipulation_inventory_tests;
