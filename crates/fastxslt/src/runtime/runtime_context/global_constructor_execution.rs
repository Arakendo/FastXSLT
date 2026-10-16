//! Bounded legacy constructors use the ordinary executor with fresh entry state.
use super::{
    ExecutionFailure, FailureCategory, GlobalBindingDefault, GlobalInitialization,
    InitializedGlobal, InvocationControl, RefCell, RuntimeGlobals, WorkDomain, control_failure,
};
use crate::runtime::golden_runtime_experiment::runtime_context::{
    RuntimeVariables, SequenceInputs, materialize_result_nodes,
};
use crate::runtime::golden_runtime_experiment::{SequenceContext, execute_sequence, failure_at};
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xslt::golden_semantics_experiment::{
    ApplySelection, BooleanExpression, Instruction, LiteralAttributeValue, ValueExpression,
};

pub(super) fn execute(
    owner: &GlobalInitialization<'_>,
    body: &[Instruction],
    location: &SourceLocation,
    control: &mut InvocationControl,
) -> Result<InitializedGlobal, ExecutionFailure> {
    let source = owner.source.ok_or_else(|| {
        failure_at(
            "FXRT1004",
            FailureCategory::Unsupported,
            Some(owner.request_id),
            location.clone(),
            "legacy global constructors require the principal effective root",
        )
    })?;
    // No completed map borrow crosses execution. Reads go back to the owner.
    let empty = RuntimeGlobals::default();
    let inputs = SequenceInputs {
        initializing_globals: Some(owner),
        program: owner.program,
        source: Some(source),
        request_id: owner.request_id,
        globals: &empty,
        multiple_match_policy: owner.documents.multiple_match_policy,
        document_rooted_matches: RefCell::default(),
        key_indexes: RefCell::default(),
        complete_atomic_frame_clones: false,
        resource_snapshot: owner.documents.resource_snapshot,
        denied_resources: owner.documents.denied_resources,
        dynamic_documents: owner.documents.dynamic_documents,
    };
    let nodes = execute_sequence(
        &inputs,
        body,
        SequenceContext::new(Some(source.document_node()), None),
        &RuntimeVariables::default(),
        control,
    )?;
    let tree = materialize_result_nodes(&nodes, owner.request_id, control)?;
    control
        .charge(WorkDomain::XPathOperation, 0)
        .map_err(|error| control_failure(error, owner.request_id))?;
    Ok(InitializedGlobal::TemporaryTree(tree))
}

pub(super) fn validate(
    owner: &GlobalInitialization<'_>,
    instruction: &Instruction,
) -> Result<(), ExecutionFailure> {
    let allowed = match instruction {
        Instruction::Text { .. }
        | Instruction::CopyOfCurrent { .. }
        | Instruction::CopyOfChildElements { .. }
        | Instruction::CopyOfLocationPath { .. }
        | Instruction::CommentNode { .. }
        | Instruction::Xslt10CommentNode { .. }
        | Instruction::ProcessingInstructionNode { .. }
        | Instruction::Xslt10Message { .. }
        | Instruction::Xslt10DeferredFailure { .. }
        | Instruction::StaticAtomicVariable { .. } => true,
        Instruction::Xslt10ProcessingInstructionNode { target, .. } => {
            matches!(target, LiteralAttributeValue::Text(_))
        }
        Instruction::If { test, .. } => admitted_condition(test),
        Instruction::Choose { branches, .. } => branches
            .iter()
            .all(|branch| admitted_condition(&branch.test)),
        Instruction::LiteralElement {
            attributes,
            attribute_set_names,
            computed_attributes,
            ..
        } => {
            attributes
                .iter()
                .all(|attribute| admitted_attribute(&attribute.value))
                && attribute_set_names.is_empty()
                && computed_attributes.is_empty()
        }
        Instruction::ValueOf { select, .. } => matches!(
            select,
            ValueExpression::LiteralString(_)
                | ValueExpression::ContextNodeName
                | ValueExpression::ContextNodeLocalName
                | ValueExpression::ContextNodeNamespaceUri
                | ValueExpression::ContextPosition(_)
                | ValueExpression::ContextSize(_)
                | ValueExpression::LocationPath(_)
                | ValueExpression::Xslt10FirstNodeLocationPath(_)
                | ValueExpression::StringPath(_)
                | ValueExpression::Xslt10FirstNodeStringPath(_)
                | ValueExpression::Xslt10FirstNodeNamePath(_)
                | ValueExpression::Xslt10VariableString(_)
                | ValueExpression::Xslt10NormalizedVariable(_)
                | ValueExpression::Xslt10VariableStringLength(_)
                | ValueExpression::Xslt10VariableStringFunction(_)
                | ValueExpression::Xslt10VariableSubstring(_)
                | ValueExpression::Xslt10VariableContains { .. }
                | ValueExpression::VariableEffectiveBooleanValue(_)
                | ValueExpression::Xslt10VariableBooleanComparison { .. }
                | ValueExpression::Xslt10VariableStringComparison { .. }
                | ValueExpression::Xslt10VariableStringVariablesComparison { .. }
                | ValueExpression::Xslt10VariableNumberComparison { .. }
        ),
        Instruction::ApplyTemplates {
            select,
            sorts,
            arguments,
            ..
        } => {
            matches!(select, None | Some(ApplySelection::LocationPath(_)))
                && sorts.is_empty()
                && arguments.is_empty()
        }
        Instruction::CallTemplate {
            name, arguments, ..
        } => {
            arguments.is_empty()
                && owner
                    .program
                    .named_templates
                    .iter()
                    .find(|target| target.name == *name)
                    .is_some_and(|target| target.template.parameters.is_empty())
        }
        Instruction::ForEachNodes { select, sorts, .. } => {
            matches!(select, ApplySelection::LocationPath(_)) && sorts.is_empty()
        }
        _ => false,
    };
    if allowed {
        return Ok(());
    }
    let index = owner.active[owner.depth.get() - 1].get();
    let GlobalBindingDefault::SharedConstructor { location, .. } =
        &owner.program.global_bindings[index].default
    else {
        unreachable!("constructor guard runs inside a shared constructor")
    };
    Err(failure_at(
        "FXRT1004",
        FailureCategory::Unsupported,
        Some(owner.request_id),
        location.clone(),
        "instruction is outside the admitted shared global-constructor slice",
    ))
}

// Eligibility only: evaluation and short-circuit behavior stay in the shared
// executor. Connected effective-boolean and string-literal readers are admitted;
// numeric-literal relations and legacy pair equality also use the ordinary
// evaluator, as do legacy pair relations. Modern comparisons remain guarded.
fn admitted_attribute(value: &LiteralAttributeValue) -> bool {
    matches!(
        value,
        LiteralAttributeValue::Text(_)
            | LiteralAttributeValue::Variable(_)
            | LiteralAttributeValue::ContextPosition
            | LiteralAttributeValue::ContextSize
            | LiteralAttributeValue::ContextLocalName
            | LiteralAttributeValue::ContextLexicalName
            | LiteralAttributeValue::ContextStringValue
            | LiteralAttributeValue::SourceAttribute(_)
    )
}

fn admitted_condition(expression: &BooleanExpression) -> bool {
    match expression {
        BooleanExpression::Constant(_)
        | BooleanExpression::VariableEffectiveBooleanValue(_)
        | BooleanExpression::Xslt10VariableStringLiteralEquals { .. }
        | BooleanExpression::Xslt10VariableNumericLiteralComparison { .. }
        | BooleanExpression::Xslt10VariableNumericComparison { .. }
        | BooleanExpression::VariableStringEquals {
            xslt10_compatibility: true,
            ..
        }
        | BooleanExpression::NodeExists(_)
        | BooleanExpression::NodeStringEquals { .. }
        | BooleanExpression::ContextStringEquals(_) => true,
        BooleanExpression::Not(value) => admitted_condition(value),
        BooleanExpression::And { left, right } | BooleanExpression::Or { left, right } => {
            admitted_condition(left) && admitted_condition(right)
        }
        _ => false,
    }
}
