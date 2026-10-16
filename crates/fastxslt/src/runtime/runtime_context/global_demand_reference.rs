//! AR-0030 test-only demand state. The callback owns evaluation through the
//! shared executor; this module owns no `XPath` or instruction semantics.

use std::{cell::RefCell, sync::Arc};

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};

#[path = "global_demand_reference/shared_execution.rs"]
mod shared_execution;

#[path = "global_demand_reference/failure_context_reference.rs"]
mod failure_context_reference;
use failure_context_reference::{CapturedFailure, Dependency, FailureContext};

#[path = "global_demand_reference/failed_cell_context.rs"]
mod failed_cell_context;

#[path = "global_demand_reference/temporary_read.rs"]
mod temporary_read;
pub(in crate::runtime::golden_runtime_experiment) use temporary_read::TemporaryReadHook;

#[path = "global_demand_reference/source_read.rs"]
mod source_read;
pub(in crate::runtime::golden_runtime_experiment) use source_read::SourceReadHook;

type StringReader = dyn Fn(
    &str,
    Option<&crate::xdm::owned_tree_experiment::SourceLocation>,
    bool,
    &mut InvocationControl,
) -> Result<Option<String>, super::ExecutionFailure>;

type BooleanReader = dyn Fn(
    &str,
    Option<&crate::xdm::owned_tree_experiment::SourceLocation>,
    &mut InvocationControl,
) -> Result<Option<bool>, super::ExecutionFailure>;

pub(in crate::runtime::golden_runtime_experiment) struct BooleanReadHook(
    std::rc::Rc<BooleanReader>,
);

impl BooleanReadHook {
    fn new(
        reader: impl Fn(
            &str,
            Option<&crate::xdm::owned_tree_experiment::SourceLocation>,
            &mut InvocationControl,
        ) -> Result<Option<bool>, super::ExecutionFailure>
        + 'static,
    ) -> Self {
        Self(std::rc::Rc::new(reader))
    }

    pub(in crate::runtime) fn read(
        &self,
        name: &str,
        location: Option<&crate::xdm::owned_tree_experiment::SourceLocation>,
        control: &mut InvocationControl,
    ) -> Result<Option<bool>, super::ExecutionFailure> {
        (self.0)(name, location, control)
    }
}

impl std::fmt::Debug for BooleanReadHook {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("BooleanReadHook(test-only)")
    }
}

pub(in crate::runtime::golden_runtime_experiment) struct StringReadHook(std::rc::Rc<StringReader>);

impl StringReadHook {
    fn new(
        reader: impl Fn(
            &str,
            Option<&crate::xdm::owned_tree_experiment::SourceLocation>,
            bool,
            &mut InvocationControl,
        ) -> Result<Option<String>, super::ExecutionFailure>
        + 'static,
    ) -> Self {
        Self(std::rc::Rc::new(reader))
    }

    pub(in crate::runtime) fn read(
        &self,
        name: &str,
        location: Option<&crate::xdm::owned_tree_experiment::SourceLocation>,
        comparison: bool,
        control: &mut InvocationControl,
    ) -> Result<Option<String>, super::ExecutionFailure> {
        (self.0)(name, location, comparison, control)
    }
}

impl std::fmt::Debug for StringReadHook {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("StringReadHook(test-only)")
    }
}

#[derive(Debug)]
enum Failure<E> {
    MissingBinding(usize),
    Circularity(Vec<usize>),
    DepthLimit(usize),
    Control(ControlFailure),
    Evaluation(E),
}

enum State<T, E> {
    Unevaluated,
    Evaluating,
    Complete(Arc<T>),
    Failed(CapturedFailure<Failure<E>>),
}

struct DemandReference<T, E> {
    states: RefCell<Vec<State<T, E>>>,
    stack: RefCell<Vec<usize>>,
    max_depth: usize,
}

impl<T, E> DemandReference<T, E> {
    fn failed_context(
        &self,
        binding: usize,
        path: &[Dependency],
        max_entries: usize,
        control: &mut InvocationControl,
    ) -> Option<Arc<FailureContext<Failure<E>>>> {
        let states = self.states.borrow();
        let State::Failed(failure) = states.get(binding)? else {
            return None;
        };
        // Capture only bounded metadata. No executor callback or value read is
        // invoked while borrowing the cell; recursive evaluation stays outside.
        Some(failure.capture_once(path, max_entries, control))
    }

    fn new(
        seeds: &[Option<Arc<T>>],
        max_depth: usize,
        control: &mut InvocationControl,
    ) -> Result<Self, ControlFailure> {
        // Private reference units, not an estimate of allocator or RSS bytes.
        control.charge(WorkDomain::XdmNode, seeds.len())?;
        Ok(Self {
            states: RefCell::new(
                seeds
                    .iter()
                    .map(|seed| {
                        seed.as_ref().map_or(State::Unevaluated, |value| {
                            State::Complete(Arc::clone(value))
                        })
                    })
                    .collect(),
            ),
            stack: RefCell::new(Vec::new()),
            max_depth,
        })
    }

    fn read<F>(
        &self,
        binding: usize,
        control: &mut InvocationControl,
        evaluate: &F,
    ) -> Result<Arc<T>, Arc<Failure<E>>>
    where
        F: Fn(usize, &Self, &mut InvocationControl) -> Result<T, Arc<Failure<E>>>,
    {
        control
            .charge(WorkDomain::XPathOperation, 1)
            .map_err(|error| Arc::new(Failure::Control(error)))?;
        {
            let states = self.states.borrow();
            match states.get(binding) {
                Some(State::Complete(value)) => return Ok(Arc::clone(value)),
                Some(State::Failed(error)) => return Err(Arc::clone(&error.primary)),
                Some(State::Evaluating) => {
                    control
                        .charge(WorkDomain::XdmNode, self.stack.borrow().len() + 1)
                        .map_err(|error| Arc::new(Failure::Control(error)))?;
                    let mut cycle = self.stack.borrow().clone();
                    cycle.push(binding);
                    return Err(Arc::new(Failure::Circularity(cycle)));
                }
                Some(State::Unevaluated) => {}
                None => return Err(Arc::new(Failure::MissingBinding(binding))),
            }
        }
        if self.stack.borrow().len() >= self.max_depth {
            return Err(Arc::new(Failure::DepthLimit(self.max_depth)));
        }
        control
            .charge(WorkDomain::XdmNode, 1)
            .map_err(|error| Arc::new(Failure::Control(error)))?;
        self.stack.borrow_mut().push(binding);
        self.states.borrow_mut()[binding] = State::Evaluating;
        let _active = ActiveEvaluation {
            resolver: self,
            binding,
        };

        // No map entry or RefCell borrow survives this recursive callback.
        let result = evaluate(binding, self, control).and_then(|value| {
            // Observe cancellation before publishing a fully owned value.
            control
                .charge(WorkDomain::XPathOperation, 0)
                .map_err(|error| Arc::new(Failure::Control(error)))?;
            Ok(Arc::new(value))
        });
        self.states.borrow_mut()[binding] = match &result {
            Ok(value) => State::Complete(Arc::clone(value)),
            Err(error) => State::Failed(CapturedFailure {
                primary: Arc::clone(error),
                context: std::cell::OnceCell::new(),
            }),
        };
        result
    }
}

struct ActiveEvaluation<'a, T, E> {
    resolver: &'a DemandReference<T, E>,
    binding: usize,
}

impl<T, E> Drop for ActiveEvaluation<'_, T, E> {
    fn drop(&mut self) {
        let popped = self.resolver.stack.borrow_mut().pop();
        debug_assert_eq!(popped, Some(self.binding));
        let mut states = self.resolver.states.borrow_mut();
        if matches!(states[self.binding], State::Evaluating) {
            // Panic unwind is test evidence, not a production recovery promise.
            states[self.binding] = State::Unevaluated;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    use std::cell::Cell;

    fn control() -> InvocationControl {
        InvocationControl::new(CancellationToken::new(), WorkLimits::unbounded())
    }

    fn forward(
        binding: usize,
        resolver: &DemandReference<String, &'static str>,
        control: &mut InvocationControl,
    ) -> Result<String, Arc<Failure<&'static str>>> {
        if binding == 0 {
            Ok(format!("{}!", resolver.read(1, control, &forward)?))
        } else {
            Ok("later".into())
        }
    }

    #[test]
    fn demand_reference_forward_reads_reuse_values_without_recursive_borrows() {
        let mut control = control();
        let resolver = DemandReference::new(&[None, None], 2, &mut control).unwrap();
        let first = resolver.read(0, &mut control, &forward).unwrap();
        let repeat = resolver.read(0, &mut control, &forward).unwrap();
        assert_eq!(first.as_str(), "later!");
        assert!(Arc::ptr_eq(&first, &repeat));
        assert!(resolver.stack.borrow().is_empty());
        assert_eq!(control.consumed(WorkDomain::XdmNode), 4);
    }

    fn cycle(
        binding: usize,
        resolver: &DemandReference<String, &'static str>,
        control: &mut InvocationControl,
    ) -> Result<String, Arc<Failure<&'static str>>> {
        resolver
            .read(1 - binding, control, &cycle)
            .map(|value| (*value).clone())
    }

    #[test]
    fn demand_reference_cycles_cache_failure_and_release_all_active_markers() {
        let mut control = control();
        let resolver = DemandReference::new(&[None, None], 2, &mut control).unwrap();
        let error = resolver.read(0, &mut control, &cycle).unwrap_err();
        assert!(matches!(error.as_ref(), Failure::Circularity(path) if path == &[0, 1, 0]));
        assert!(Arc::ptr_eq(
            &error,
            &resolver.read(0, &mut control, &cycle).unwrap_err()
        ));
        assert!(resolver.stack.borrow().is_empty());
        assert!(
            resolver
                .states
                .borrow()
                .iter()
                .all(|state| matches!(state, State::Failed(_)))
        );
        let direct = DemandReference::new(&[None], 1, &mut control).unwrap();
        let error = direct
            .read(0, &mut control, &|binding, resolver, control| {
                resolver
                    .read(binding, control, &forward)
                    .map(|value| (*value).clone())
            })
            .unwrap_err();
        assert!(matches!(error.as_ref(), Failure::Circularity(path) if path == &[0, 0]));
    }

    #[test]
    fn demand_reference_seeds_override_defaults_and_unused_failure_is_not_evaluated() {
        let mut control = control();
        let seed = Arc::new(String::from("host"));
        let resolver =
            DemandReference::new(&[Some(Arc::clone(&seed)), None], 2, &mut control).unwrap();
        let calls = Cell::new(0);
        let evaluate = |_, _: &DemandReference<String, &str>, _: &mut InvocationControl| {
            calls.set(calls.get() + 1);
            Err(Arc::new(Failure::Evaluation("failure")))
        };
        assert!(Arc::ptr_eq(
            &seed,
            &resolver.read(0, &mut control, &evaluate).unwrap()
        ));
        assert_eq!(calls.get(), 0);
        let error = resolver.read(1, &mut control, &evaluate).unwrap_err();
        assert!(matches!(error.as_ref(), Failure::Evaluation("failure")));
        assert!(Arc::ptr_eq(
            &error,
            &resolver.read(1, &mut control, &evaluate).unwrap_err()
        ));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn demand_reference_budget_depth_and_missing_binding_fail_without_partial_values() {
        for (xdm_limit, success) in [(4, true), (3, false)] {
            let mut limits = WorkLimits::unbounded();
            limits.xdm_nodes = xdm_limit;
            let mut control = InvocationControl::new(CancellationToken::new(), limits);
            let resolver = DemandReference::new(&[None, None], 2, &mut control).unwrap();
            let result = resolver.read(0, &mut control, &forward);
            assert_eq!(result.is_ok(), success);
            if !success {
                assert!(matches!(
                    result.unwrap_err().as_ref(),
                    Failure::Control(ControlFailure::BudgetExhausted { .. })
                ));
                assert!(
                    !resolver
                        .states
                        .borrow()
                        .iter()
                        .any(|state| matches!(state, State::Complete(_)))
                );
            }
            assert!(resolver.stack.borrow().is_empty());
        }
        let mut control = control();
        let resolver = DemandReference::new(&[None, None], 1, &mut control).unwrap();
        assert!(matches!(
            resolver
                .read(0, &mut control, &forward)
                .unwrap_err()
                .as_ref(),
            Failure::DepthLimit(1)
        ));
        assert!(matches!(
            resolver
                .read(2, &mut control, &forward)
                .unwrap_err()
                .as_ref(),
            Failure::MissingBinding(2)
        ));
        assert!(resolver.stack.borrow().is_empty());
    }

    #[test]
    fn demand_reference_cancellation_before_publication_retains_no_value() {
        let token = CancellationToken::new();
        let mut control = InvocationControl::new(token.clone(), WorkLimits::unbounded());
        let resolver = DemandReference::new(&[None], 1, &mut control).unwrap();
        let error = resolver
            .read(0, &mut control, &|_, _, _| {
                token.cancel();
                Ok::<_, Arc<Failure<&str>>>(String::from("not published"))
            })
            .unwrap_err();
        assert!(matches!(
            error.as_ref(),
            Failure::Control(ControlFailure::Cancelled { .. })
        ));
        assert!(matches!(resolver.states.borrow()[0], State::Failed(_)));
        assert!(resolver.stack.borrow().is_empty());
    }

    #[test]
    fn demand_reference_panic_unwinds_marker_without_publishing() {
        let mut control = control();
        let resolver: DemandReference<String, &str> =
            DemandReference::new(&[None], 1, &mut control).unwrap();
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = resolver.read(0, &mut control, &|_, _, _| panic!("reference fault"));
        }));
        assert!(panic.is_err());
        assert!(matches!(resolver.states.borrow()[0], State::Unevaluated));
        assert!(resolver.stack.borrow().is_empty());
    }

    #[test]
    fn demand_reference_calls_shared_executor_with_root_focus_and_owned_tree() {
        use super::super::super::{
            ExecutionFailure, MultipleMatchPolicy, SequenceContext, execute_sequence,
        };
        use super::super::{
            RuntimeGlobals, RuntimeVariables, SequenceInputs, TemporaryTree,
            materialize_result_nodes, temporary_tree_string_value,
        };
        use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
        use crate::xdm::owned_tree_experiment::Document;
        use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

        let document = |identity, xml: &str| {
            Document::from_parsed(
                parse_document(
                    identity,
                    xml.as_bytes(),
                    ParseLimits {
                        max_events: 256,
                        max_depth: 32,
                    },
                )
                .unwrap(),
            )
            .unwrap()
        };
        let source = document("urn:demand:source", "<r><b>one</b><b>two</b></r>");
        let stylesheet = document(
            "urn:demand:stylesheet",
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
              <xsl:template match="/">
                <xsl:value-of select="position()"/><xsl:text>/</xsl:text>
                <xsl:value-of select="last()"/><xsl:text>:</xsl:text>
                <xsl:apply-templates select="//b"/>
              </xsl:template>
              <xsl:template match="b"><xsl:value-of select="."/></xsl:template>
            </xsl:stylesheet>"#,
        );
        let program = compile_stylesheet(&stylesheet).unwrap();
        let globals = RuntimeGlobals::default();
        let dynamic_documents = RefCell::default();
        let inputs = SequenceInputs {
            initializing_globals: None,
            program: &program,
            source: Some(&source),
            request_id: "demand-reference",
            globals: &globals,
            multiple_match_policy: MultipleMatchPolicy::UseLast,
            document_rooted_matches: RefCell::default(),
            key_indexes: RefCell::default(),
            complete_atomic_frame_clones: false,
            resource_snapshot: None,
            denied_resources: None,
            dynamic_documents: &dynamic_documents,
        };
        let mut control = control();
        let resolver = DemandReference::new(&[None], 1, &mut control).unwrap();
        let calls = Cell::new(0);
        let evaluate = |_,
                        _: &DemandReference<TemporaryTree, ExecutionFailure>,
                        control: &mut InvocationControl| {
            calls.set(calls.get() + 1);
            let nodes = execute_sequence(
                &inputs,
                &program.root_template.as_ref().unwrap().body,
                SequenceContext::new(Some(source.document_node()), None),
                &RuntimeVariables::default(),
                control,
            )
            .map_err(|error| Arc::new(Failure::Evaluation(error)))?;
            materialize_result_nodes(&nodes, inputs.request_id, control)
                .map_err(|error: ExecutionFailure| Arc::new(Failure::Evaluation(error)))
        };
        let value = resolver.read(0, &mut control, &evaluate).unwrap();
        let repeat = resolver.read(0, &mut control, &evaluate).unwrap();
        assert!(Arc::ptr_eq(&value, &repeat));
        assert_eq!(calls.get(), 1);
        assert_eq!(
            temporary_tree_string_value(&value, inputs.request_id, &mut control).unwrap(),
            "1/1:onetwo"
        );
        assert!(resolver.stack.borrow().is_empty());
    }
}
