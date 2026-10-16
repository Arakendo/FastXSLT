//! Invocation-owned initialization state; completed values have one store.

#[cfg(test)]
#[path = "global_resolution_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "global_constructor_tests.rs"]
mod constructor_tests;

#[path = "global_constructor_execution.rs"]
mod constructor;

use std::cell::{Cell, RefCell};

use super::{
    BTreeMap, Document, ExecutionFailure, FailureCategory, GlobalBindingDefault,
    GlobalDocumentContext, GlobalInitializerReads, InitializedGlobal, InvocationControl,
    InvocationParameter, InvocationParameterValue, RuntimeGlobals, StylesheetProgram, WorkDomain,
    control_failure, failure, materialize_global_default, materialize_global_literal_document,
};

// Stack-safety bound, not a deployment quota. Already ordered initializers use
// depth one; this bounds newly discovered recursive dependency chains.
const MAX_GLOBAL_INITIALIZATION_DEPTH: usize = 128;

pub(in crate::runtime) struct GlobalInitialization<'a> {
    program: &'a StylesheetProgram,
    source: Option<&'a Document>,
    parameters: &'a BTreeMap<String, InvocationParameter>,
    request_id: &'a str,
    documents: &'a GlobalDocumentContext<'a>,
    completed: RefCell<RuntimeGlobals>,
    // Initialization aborts on its first failure. Retain one primary failure,
    // not a cloned diagnostic for every dependent binding in the stack.
    failure: RefCell<Option<ExecutionFailure>>,
    active: [Cell<usize>; MAX_GLOBAL_INITIALIZATION_DEPTH],
    depth: Cell<usize>,
    poisoned: Cell<bool>,
}

impl<'a> GlobalInitialization<'a> {
    pub(super) fn new(
        program: &'a StylesheetProgram,
        source: Option<&'a Document>,
        parameters: &'a BTreeMap<String, InvocationParameter>,
        request_id: &'a str,
        documents: &'a GlobalDocumentContext<'a>,
    ) -> Self {
        Self {
            program,
            source,
            parameters,
            request_id,
            documents,
            completed: RefCell::new(RuntimeGlobals::default()),
            failure: RefCell::new(None),
            active: std::array::from_fn(|_| Cell::new(usize::MAX)),
            depth: Cell::new(0),
            poisoned: Cell::new(false),
        }
    }

    pub(super) fn initialize_all(
        self,
        control: &mut InvocationControl,
    ) -> Result<RuntimeGlobals, ExecutionFailure> {
        // Retain the compiled eager schedule; recursive reads use this owner.
        for index in 0..self.program.global_bindings.len() {
            self.resolve_index(index, control)?;
        }
        Ok(self.completed.into_inner())
    }

    fn is_complete(&self, name: &str) -> bool {
        let values = self.completed.borrow();
        values.atomics.contains_key(name)
            || values.nodes.contains_key(name)
            || values.temporary_trees.contains_key(name)
            || values.empty_sequences.contains(name)
            || values.literal_documents.contains_key(name)
    }

    fn resolve_index(
        &self,
        index: usize,
        control: &mut InvocationControl,
    ) -> Result<(), ExecutionFailure> {
        assert!(
            !self.poisoned.get(),
            "global initialization cannot resume after unwinding"
        );
        let binding = &self.program.global_bindings[index];
        if self.is_complete(&binding.name) {
            return Ok(());
        }
        if let Some(error) = self.failure.borrow().as_ref() {
            return Err(error.clone());
        }
        let depth = self.depth.get();
        if self.active[..depth]
            .iter()
            .any(|active| active.get() == index)
        {
            let mut error = failure(
                "XTDE0640",
                FailureCategory::Invalid,
                Some(self.request_id),
                format!("circular global dependency re-entered ${}", binding.name),
            );
            if let GlobalBindingDefault::SharedConstructor { location, .. } = &binding.default {
                error.location = Some(location.clone());
            }
            return Err(error);
        }
        if depth == MAX_GLOBAL_INITIALIZATION_DEPTH {
            return Err(failure(
                "FXRT0003",
                FailureCategory::Limit,
                Some(self.request_id),
                format!(
                    "global initialization depth exceeds private limit {MAX_GLOBAL_INITIALIZATION_DEPTH}"
                ),
            ));
        }
        // Additional recursive work is charged before entering it. The existing
        // eager schedule and completed reads retain their previous charges.
        if depth > 0 {
            control
                .charge(WorkDomain::XPathOperation, 1)
                .map_err(|error| control_failure(error, self.request_id))?;
        }
        self.active[depth].set(index);
        self.depth.set(depth + 1);
        let mut active = ActiveInitialization {
            owner: self,
            depth,
            settled: false,
        };
        let result = self.evaluate(index, control);
        match result {
            Ok(value) => {
                value.publish(&mut self.completed.borrow_mut(), &binding.name);
                active.settled = true;
                Ok(())
            }
            Err(error) => {
                self.failure
                    .borrow_mut()
                    .get_or_insert_with(|| error.clone());
                active.settled = true;
                Err(error)
            }
        }
    }

    fn evaluate(
        &self,
        index: usize,
        control: &mut InvocationControl,
    ) -> Result<InitializedGlobal, ExecutionFailure> {
        let binding = &self.program.global_bindings[index];
        if binding.kind == crate::xslt::golden_semantics_experiment::GlobalBindingKind::Parameter {
            if let Some(parameter) = self
                .parameters
                .get(&binding.name)
                .filter(|value| !value.tunnel)
            {
                return match &parameter.value {
                    InvocationParameterValue::Atomic(value) => {
                        Ok(InitializedGlobal::Atomic(value.clone()))
                    }
                    InvocationParameterValue::AtomicSequence(_)
                    | InvocationParameterValue::SourceNodes(_)
                    | InvocationParameterValue::TemporaryTree(_) => Err(failure(
                        "XTTE0590",
                        FailureCategory::Invalid,
                        Some(self.request_id),
                        format!(
                            "global parameter requires an atomic host value: ${}",
                            binding.name
                        ),
                    )),
                };
            }
            if binding.required {
                return Err(failure(
                    "XTDE0050",
                    FailureCategory::Invalid,
                    Some(self.request_id),
                    format!(
                        "required global parameter was not supplied: ${}",
                        binding.name
                    ),
                ));
            }
        }
        if let GlobalBindingDefault::SharedConstructor { body, location } = &binding.default {
            constructor::execute(self, body, location, control)
        } else if let GlobalBindingDefault::Xslt10LiteralDocument(reference) = &binding.default {
            materialize_global_literal_document(reference, self.request_id, self.documents, control)
        } else {
            materialize_global_default(self, binding, self.source, self.request_id, control)
        }
    }

    pub(in crate::runtime) fn validate_constructor_instruction(
        &self,
        instruction: &crate::xslt::golden_semantics_experiment::Instruction,
    ) -> Result<(), ExecutionFailure> {
        constructor::validate(self, instruction)
    }

    pub(in crate::runtime) fn read_legacy_string(
        &self,
        name: &str,
        control: &mut InvocationControl,
    ) -> Result<String, ExecutionFailure> {
        self.with_completed(name, control, |globals, control| {
            if globals.literal_documents.contains_key(name) {
                return Err(failure(
                    "FXRT1004",
                    FailureCategory::Unsupported,
                    Some(self.request_id),
                    "secondary-document string reads during global construction are not yet admitted",
                ));
            }
            super::global_string_value(globals, name, self.source, self.request_id, control)
        })
    }

    pub(in crate::runtime) fn read_string_comparison(
        &self,
        name: &str,
        expected: &str,
        equal: bool,
        control: &mut InvocationControl,
    ) -> Result<bool, ExecutionFailure> {
        self.with_completed(name, control, |globals, control| {
            if let Some(value) = globals.atomics.get(name)
                && !matches!(value.atomic_type(),
                    crate::xdm::atomic_value_experiment::BuiltinAtomicType::String
                    | crate::xdm::atomic_value_experiment::BuiltinAtomicType::UntypedAtomic)
            {
                return Err(failure("FXRT1004", FailureCategory::Unsupported,
                    Some(self.request_id),
                    "typed scalar comparisons during global construction are not yet admitted"));
            }
            if globals.literal_documents.contains_key(name) {
                return Err(failure("FXRT1004", FailureCategory::Unsupported,
                    Some(self.request_id),
                    "secondary-document comparisons during global construction are not yet admitted"));
            }
            if let Some(nodes) = globals.nodes.get(name) {
                let source = self.source.ok_or_else(|| failure("FXRT1004",
                    FailureCategory::Unsupported, Some(self.request_id),
                    "source-node comparison requires a principal source"))?;
                return crate::runtime::golden_runtime_experiment::value_evaluator::source_string_comparison(
                    source, nodes, expected, equal, self.request_id, control);
            }
            let actual = super::global_string_value(
                globals, name, self.source, self.request_id, control)?;
            Ok((actual == expected) == equal)
        })
    }

    pub(in crate::runtime) fn read_numeric_predicate(
        &self,
        name: &str,
        control: &mut InvocationControl,
        matches: &impl Fn(&str) -> bool,
    ) -> Result<bool, ExecutionFailure> {
        use crate::runtime::golden_runtime_experiment::value_evaluator;
        self.with_completed(name, control, |globals, control| {
            if let Some(value) = globals.atomics.get(name)
                && !value_evaluator::numeric_initializer_scalar(value)
            {
                return Err(failure("FXRT1004", FailureCategory::Unsupported,
                    Some(self.request_id),
                    "this typed scalar numeric comparison during global construction is not yet admitted"));
            }
            if globals.literal_documents.contains_key(name) {
                return Err(failure("FXRT1004", FailureCategory::Unsupported,
                    Some(self.request_id),
                    "secondary-document comparisons during global construction are not yet admitted"));
            }
            if let Some(nodes) = globals.nodes.get(name) {
                let source = self.source.ok_or_else(|| failure("FXRT1004",
                    FailureCategory::Unsupported, Some(self.request_id),
                    "source-node comparison requires a principal source"))?;
                return value_evaluator::source_value_predicate(
                    source, nodes, self.request_id, control, matches);
            }
            let actual = super::global_string_value(
                globals, name, self.source, self.request_id, control)?;
            Ok(matches(&actual))
        })
    }

    pub(in crate::runtime) fn read_equality_operand(
        &self,
        name: &str,
        control: &mut InvocationControl,
    ) -> Result<
        crate::runtime::golden_runtime_experiment::value_evaluator::EqualityOperand,
        ExecutionFailure,
    > {
        self.with_completed(name, control, |globals, control| {
            crate::runtime::golden_runtime_experiment::value_evaluator::completed_equality_operand(
                globals,
                self.source,
                name,
                self.request_id,
                control,
            )
        })
    }

    pub(in crate::runtime) fn read_numeric_pair_operand(
        &self,
        name: &str,
        control: &mut InvocationControl,
    ) -> Result<
        crate::runtime::golden_runtime_experiment::value_evaluator::NumericPairOperand,
        ExecutionFailure,
    > {
        self.with_completed(name, control, |globals, control| {
            crate::runtime::golden_runtime_experiment::value_evaluator::completed_numeric_pair_operand(
                globals, self.source, name, self.request_id, control)
        })
    }

    pub(in crate::runtime) fn read_effective_boolean(
        &self,
        name: &str,
        control: &mut InvocationControl,
    ) -> Result<bool, ExecutionFailure> {
        self.with_completed(name, control, |globals, _control| {
            if globals.literal_documents.contains_key(name) {
                return Err(failure("FXRT1004", FailureCategory::Unsupported,
                    Some(self.request_id),
                    "secondary-document boolean reads during global construction are not yet admitted"));
            }
            super::super::completed_global_boolean(globals, name, self.request_id)?
                .ok_or_else(|| failure("FXRT0002", FailureCategory::Invalid,
                    Some(self.request_id), format!("unbound variable: ${name}")))
        })
    }
}

impl GlobalInitializerReads for GlobalInitialization<'_> {
    fn with_completed<T>(
        &self,
        name: &str,
        control: &mut InvocationControl,
        read: impl FnOnce(&RuntimeGlobals, &mut InvocationControl) -> Result<T, ExecutionFailure>,
    ) -> Result<T, ExecutionFailure> {
        assert!(
            !self.poisoned.get(),
            "global initialization cannot resume after unwinding"
        );
        // Ordered production reads do not scan declarations or create a second
        // index. Only an unresolved read discovers its linked binding ordinal.
        if !self.is_complete(name)
            && let Some(index) = self
                .program
                .global_bindings
                .iter()
                .position(|binding| binding.name == name)
        {
            self.resolve_index(index, control)?;
        }
        read(&self.completed.borrow(), control)
    }
}

struct ActiveInitialization<'a, 'context> {
    owner: &'a GlobalInitialization<'context>,
    depth: usize,
    settled: bool,
}

impl Drop for ActiveInitialization<'_, '_> {
    fn drop(&mut self) {
        self.owner.active[self.depth].set(usize::MAX);
        self.owner.depth.set(self.depth);
        if !self.settled {
            self.owner.poisoned.set(true);
        }
    }
}
