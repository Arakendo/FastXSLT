//! Test-only demand reads through the existing compiled sequence executor.
//!
//! Older probes use named template bodies as constructor stand-ins; the
//! declaration-body probe supplies plans lowered at actual global declarations.
//! Placeholder global declarations permit ordinary static variable resolution;
//! constructor placeholders are never installed. Source-path defaults use the
//! existing controlled path evaluator and retain their effective owner.

use std::{
    cell::RefCell,
    collections::BTreeMap,
    rc::{Rc, Weak},
    sync::Arc,
};

use super::super::super::{
    ExecutionFailure, FailureCategory, MultipleMatchPolicy, SequenceContext, control_failure,
    execute_sequence, failure,
};
use super::super::{
    RuntimeGlobals, RuntimeVariables, SequenceInputs, TemporaryTree, materialize_result_nodes,
    temporary_tree_string_value,
};
use super::{DemandReference, Failure, SourceReadHook, StringReadHook, TemporaryReadHook};
use crate::execution_control_experiment::InvocationControl;
use crate::xdm::owned_tree_experiment::Document;
use crate::xslt::golden_semantics_experiment::{GlobalBindingKind, Instruction, StylesheetProgram};

#[path = "owned_values.rs"]
mod owned_values;
use owned_values::OwnedValue;

#[path = "evaluation_schedule.rs"]
mod evaluation_schedule;

#[path = "typed_copy.rs"]
mod typed_copy;

#[path = "failure_observations.rs"]
mod failure_observations;

#[path = "read_provenance.rs"]
mod read_provenance;

#[path = "effective_boolean.rs"]
mod effective_boolean;

struct Driver {
    program: Arc<StylesheetProgram>,
    constructor_bodies: BTreeMap<String, Vec<Instruction>>,
    source: Arc<Document>,
    resolver: DemandReference<OwnedValue, ExecutionFailure>,
    self_reference: Weak<Self>,
    evaluations: RefCell<Vec<usize>>,
    complete_frame_clones: bool,
    provenance: read_provenance::ReadProvenance,
}

impl Driver {
    fn new(
        program: Arc<StylesheetProgram>,
        source: Arc<Document>,
        supplied: &BTreeMap<String, String>,
        complete_frame_clones: bool,
        control: &mut InvocationControl,
    ) -> Result<Rc<Self>, ExecutionFailure> {
        Self::with_constructor_bodies(
            program,
            source,
            supplied,
            complete_frame_clones,
            BTreeMap::new(),
            control,
        )
    }

    fn with_constructor_bodies(
        program: Arc<StylesheetProgram>,
        source: Arc<Document>,
        supplied: &BTreeMap<String, String>,
        complete_frame_clones: bool,
        constructor_bodies: BTreeMap<String, Vec<Instruction>>,
        control: &mut InvocationControl,
    ) -> Result<Rc<Self>, ExecutionFailure> {
        let seeds = program
            .global_bindings
            .iter()
            .map(|binding| {
                if binding.kind != GlobalBindingKind::Parameter {
                    return Ok(None);
                }
                supplied
                    .get(&binding.name)
                    .map(|value| {
                        materialize_result_nodes(
                            &[super::super::super::ResultNode::Text(value.clone())],
                            "demand-shared-reference",
                            control,
                        )
                        .map(|tree| Arc::new(OwnedValue::Temporary(Arc::new(tree))))
                    })
                    .transpose()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let resolver = DemandReference::new(&seeds, 32, control)
            .map_err(|error| control_failure(error, "demand-shared-reference"))?;
        Ok(Rc::new_cyclic(|self_reference| Self {
            program,
            constructor_bodies,
            source,
            resolver,
            self_reference: self_reference.clone(),
            evaluations: RefCell::new(vec![0; seeds.len()]),
            complete_frame_clones,
            provenance: read_provenance::ReadProvenance::default(),
        }))
    }

    fn execute(
        &self,
        body: &[Instruction],
        control: &mut InvocationControl,
    ) -> Result<TemporaryTree, ExecutionFailure> {
        self.execute_with_frame(body, &RuntimeVariables::default(), &self.source, control)
    }

    fn execute_with_frame(
        &self,
        body: &[Instruction],
        variables: &RuntimeVariables,
        source: &Document,
        control: &mut InvocationControl,
    ) -> Result<TemporaryTree, ExecutionFailure> {
        let nodes = self.execute_nodes_with_frame(body, variables, source, control)?;
        materialize_result_nodes(&nodes, "demand-shared-reference", control)
    }

    fn execute_nodes_with_frame(
        &self,
        body: &[Instruction],
        variables: &RuntimeVariables,
        source: &Document,
        control: &mut InvocationControl,
    ) -> Result<Vec<super::super::super::ResultNode>, ExecutionFailure> {
        let weak = self.self_reference.clone();
        let temporary_weak = self.self_reference.clone();
        let source_weak = self.self_reference.clone();
        let boolean_weak = self.self_reference.clone();
        let globals = RuntimeGlobals {
            demand_boolean_reference: Some(super::BooleanReadHook::new(
                move |name, location, control| {
                    boolean_weak
                        .upgrade()
                        .expect("active invocation owner")
                        .read_value_at(name, location, control)
                        .map(|value| {
                            value.map(|value| match value.as_ref() {
                                OwnedValue::Temporary(_) => true,
                                OwnedValue::Source(selection) => !selection.nodes.is_empty(),
                            })
                        })
                },
            )),
            demand_source_reference: Some(SourceReadHook::new(move |name, location, control| {
                let owner = source_weak.upgrade().expect("active invocation owner");
                if !owner.program.global_bindings.iter().any(|binding| binding.name == name && matches!(binding.default, crate::xslt::golden_semantics_experiment::GlobalBindingDefault::LocationPath(_))) {
                    return Ok(None);
                }
                let value = owner
                    .read_value_at(name, location, control)?
                    .expect("known binding");
                let OwnedValue::Source(selection) = value.as_ref() else {
                    return Ok(None);
                };
                control
                    .charge(
                        crate::execution_control_experiment::WorkDomain::XdmNode,
                        selection.nodes.len(),
                    )
                    .map_err(|error| control_failure(error, "demand-shared-reference"))?;
                Ok(Some((
                    Arc::clone(&selection.owner),
                    selection.nodes.clone(),
                )))
            })),
            demand_temporary_reference: Some(TemporaryReadHook::new(
                move |name, location, control| {
                    let owner = temporary_weak.upgrade().expect("active invocation owner");
                    owner
                        .read_value_at(name, Some(location), control)?
                        .map(|value| match value.as_ref() {
                            OwnedValue::Temporary(tree) => Ok(Arc::clone(tree)),
                            OwnedValue::Source(_) => Err(failure(
                                "FXAR0030",
                                FailureCategory::Unsupported,
                                Some("demand-shared-reference"),
                                "typed demand copy reference admits temporary trees only",
                            )),
                        })
                        .transpose()
                },
            )),
            demand_string_reference: Some(StringReadHook::new(
                move |name, location, comparison, control| {
                    weak.upgrade()
                    .expect("active invocation owner")
                    .read_value_at(name, location, control)?
                    .map(|value| {
                        if comparison && matches!(value.as_ref(), OwnedValue::Source(_)) {
                            return Err(failure("FXAR0030", FailureCategory::Unsupported,
                                Some("demand-shared-reference"),
                                "source-sequence demand comparison requires a typed read bridge"));
                        }
                        value.string_value(control)
                    })
                    .transpose()
                },
            )),
            ..RuntimeGlobals::default()
        };
        let dynamic_documents = RefCell::default();
        let inputs = SequenceInputs {
            initializing_globals: None,
            program: &self.program,
            source: Some(source),
            request_id: "demand-shared-reference",
            globals: &globals,
            multiple_match_policy: MultipleMatchPolicy::UseLast,
            document_rooted_matches: RefCell::default(),
            key_indexes: RefCell::default(),
            complete_atomic_frame_clones: self.complete_frame_clones,
            resource_snapshot: None,
            denied_resources: None,
            dynamic_documents: &dynamic_documents,
        };
        execute_sequence(
            &inputs,
            body,
            SequenceContext::new(Some(source.document_node()), None),
            variables,
            control,
        )
    }

    fn read_string(
        &self,
        name: &str,
        control: &mut InvocationControl,
    ) -> Result<Option<String>, ExecutionFailure> {
        self.read_value(name, control)?
            .map(|value| value.string_value(control))
            .transpose()
    }

    fn read_value(
        &self,
        name: &str,
        control: &mut InvocationControl,
    ) -> Result<Option<Arc<OwnedValue>>, ExecutionFailure> {
        self.read_value_at(name, None, control)
    }

    fn read_value_at(
        &self,
        name: &str,
        location: Option<&crate::xdm::owned_tree_experiment::SourceLocation>,
        control: &mut InvocationControl,
    ) -> Result<Option<Arc<OwnedValue>>, ExecutionFailure> {
        let Some(binding) = self
            .program
            .global_bindings
            .iter()
            .position(|value| value.name == name)
        else {
            return Ok(None);
        };
        let _read = if matches!(
            &self.resolver.states.borrow()[binding],
            super::State::Unevaluated | super::State::Evaluating
        ) {
            Some(
                self.provenance
                    .enter(binding, location, &self.program, control)?,
            )
        } else {
            None
        };
        let value = self
            .resolver
            .read(binding, control, &|binding, _, control| {
                self.evaluations.borrow_mut()[binding] += 1;
                if let crate::xslt::golden_semantics_experiment::GlobalBindingDefault::LocationPath(path) =
                    &self.program.global_bindings[binding].default
                {
                    return OwnedValue::select_source(&self.source, path, control)
                        .map_err(|error| Arc::new(Failure::Evaluation(error)));
                }
                let name = &self.program.global_bindings[binding].name;
                let body = if let Some(body) = self.constructor_bodies.get(name) {
                    body.as_slice()
                } else {
                    let constructor_name = format!("{name}_constructor");
                    &self.program.named_templates.iter()
                        .find(|template| template.name == constructor_name)
                        .expect("reference fixture provides a constructor body").template.body
                };
                self.execute(body, control)
                    .map(|tree| OwnedValue::Temporary(Arc::new(tree)))
                    .map_err(|error| Arc::new(Failure::Evaluation(error)))
            })
            .map_err(|error| {
                self.provenance.observe_cycle(&error, control);
                self.provenance.capture(&self.resolver, binding, control);
                Self::translate_failure(&error)
            })?;
        Ok(Some(value))
    }

    fn translate_failure(error: &Failure<ExecutionFailure>) -> ExecutionFailure {
        match error {
            Failure::Evaluation(error) => error.clone(),
            Failure::Control(error) => control_failure(*error, "demand-shared-reference"),
            // Experimental identities only: no standards diagnostic is selected here.
            Failure::Circularity(path) => failure(
                "FXAR0030",
                FailureCategory::Invalid,
                Some("demand-shared-reference"),
                format!("reference circular dependency: {path:?}"),
            ),
            Failure::MissingBinding(binding) => failure(
                "FXAR0030",
                FailureCategory::Invalid,
                Some("demand-shared-reference"),
                format!("reference missing binding: {binding}"),
            ),
            Failure::DepthLimit(depth) => failure(
                "FXAR0030",
                FailureCategory::Limit,
                Some("demand-shared-reference"),
                format!("reference depth limit: {depth}"),
            ),
        }
    }

    fn run(&self, control: &mut InvocationControl) -> Result<String, ExecutionFailure> {
        let tree = self.execute(&self.program.root_template.as_ref().unwrap().body, control)?;
        temporary_tree_string_value(&tree, "demand-shared-reference", control)
    }

    fn evaluations(&self, name: &str) -> usize {
        let binding = self
            .program
            .global_bindings
            .iter()
            .position(|binding| binding.name == name)
            .unwrap();
        self.evaluations.borrow()[binding]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn document(identity: &str, xml: &str) -> Arc<Document> {
        Arc::new(
            Document::from_parsed(
                parse_document(
                    identity,
                    xml.as_bytes(),
                    ParseLimits {
                        max_events: 1024,
                        max_depth: 64,
                    },
                )
                .unwrap(),
            )
            .unwrap(),
        )
    }

    fn program(body: &str) -> Arc<StylesheetProgram> {
        program_at("1.0", body)
    }

    fn program_at(version: &str, body: &str) -> Arc<StylesheetProgram> {
        let xml = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}">
          <xsl:param name="first" select="'placeholder'"/>
          <xsl:variable name="second" select="'placeholder'"/>
          <xsl:variable name="unused" select="'placeholder'"/>
          {body}</xsl:stylesheet>"#
        );
        Arc::new(compile_stylesheet(&document("urn:demand:stylesheet", &xml)).unwrap())
    }

    fn control() -> InvocationControl {
        InvocationControl::new(CancellationToken::new(), WorkLimits::unbounded())
    }

    #[test]
    fn demand_shared_forward_reads_through_dispatch_reuse_and_skip_unused_failure() {
        let program = program(
            r#"
          <xsl:template match="/"><xsl:value-of select="$first"/><xsl:text>|</xsl:text><xsl:value-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:apply-templates select="//b" mode="picked"/></xsl:template>
          <xsl:template match="b" mode="picked"><xsl:value-of select="$second"/></xsl:template>
          <xsl:template match="b"><xsl:text>wrong-mode</xsl:text></xsl:template>
          <xsl:template name="second_constructor"><xsl:value-of select="position()"/><xsl:text>/</xsl:text><xsl:value-of select="last()"/><xsl:text>:</xsl:text><xsl:value-of select="/r/@value"/></xsl:template>
          <xsl:template name="unused_constructor"><xsl:message terminate="yes">must-not-run</xsl:message></xsl:template>"#,
        );
        for complete in [false, true] {
            let mut control = control();
            let driver = Driver::new(
                Arc::clone(&program),
                document("urn:demand:source", "<r value='root'><b/><b/></r>"),
                &BTreeMap::new(),
                complete,
                &mut control,
            )
            .unwrap();
            assert_eq!(
                driver.run(&mut control).unwrap(),
                "1/1:root1/1:root|1/1:root1/1:root"
            );
            assert_eq!(driver.evaluations("first"), 1);
            assert_eq!(driver.evaluations("second"), 1);
            assert_eq!(driver.evaluations("unused"), 0);
            assert!(driver.resolver.stack.borrow().is_empty());
        }
    }

    #[test]
    fn demand_shared_global_entry_resets_caller_locals_and_shadowing_stays_local() {
        let program = program(
            r#"
          <xsl:template match="/"><xsl:variable name="second" select="'caller'"/><xsl:value-of select="$first"/><xsl:text>|</xsl:text><xsl:value-of select="$second"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:value-of select="$second"/><xsl:text>:</xsl:text><xsl:variable name="second"><xsl:text>local-tree</xsl:text></xsl:variable><xsl:value-of select="$second"/></xsl:template>
          <xsl:template name="second_constructor"><xsl:text>global</xsl:text></xsl:template>"#,
        );
        for complete in [false, true] {
            let mut control = control();
            let driver = Driver::new(
                Arc::clone(&program),
                document("urn:demand:source", "<r/>"),
                &BTreeMap::new(),
                complete,
                &mut control,
            )
            .unwrap();
            assert_eq!(
                driver.run(&mut control).unwrap(),
                "global:local-tree|caller"
            );
            assert_eq!(driver.evaluations("second"), 1);
        }
    }

    #[test]
    fn demand_shared_conditional_reads_only_enter_taken_dependency() {
        let program = program(
            r#"
          <xsl:template match="/"><xsl:value-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:choose><xsl:when test="/r/@read = 'yes'"><xsl:value-of select="$second"/></xsl:when><xsl:otherwise><xsl:text>skip</xsl:text></xsl:otherwise></xsl:choose></xsl:template>
          <xsl:template name="second_constructor"><xsl:message terminate="yes">taken-error</xsl:message></xsl:template>"#,
        );
        for read in [false, true] {
            let mut control = control();
            let driver = Driver::new(
                Arc::clone(&program),
                document(
                    "urn:demand:source",
                    if read { "<r read='yes'/>" } else { "<r/>" },
                ),
                &BTreeMap::new(),
                false,
                &mut control,
            )
            .unwrap();
            if read {
                let error = driver.run(&mut control).unwrap_err();
                assert_eq!(error.code, "XTMM9000");
                assert!(error.location.is_some());
                assert_eq!(driver.evaluations("second"), 1);
            } else {
                assert_eq!(driver.run(&mut control).unwrap(), "skip");
                assert_eq!(driver.evaluations("second"), 0);
            }
            assert!(driver.resolver.stack.borrow().is_empty());
        }
    }

    #[test]
    fn demand_shared_dispatched_cycle_fails_without_publishing_and_untaken_cycle_is_valid() {
        let program = program(
            r#"
          <xsl:template match="/"><xsl:value-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:apply-templates select="//b"/></xsl:template>
          <xsl:template match="b"><xsl:value-of select="$second"/></xsl:template>
          <xsl:template name="second_constructor"><xsl:choose><xsl:when test="/r/@cycle = 'yes'"><xsl:value-of select="$first"/></xsl:when><xsl:otherwise><xsl:text>acyclic</xsl:text></xsl:otherwise></xsl:choose></xsl:template>"#,
        );
        for cycle in [false, true] {
            let mut control = control();
            let driver = Driver::new(
                Arc::clone(&program),
                document(
                    "urn:demand:source",
                    if cycle {
                        "<r cycle='yes'><b/></r>"
                    } else {
                        "<r><b/></r>"
                    },
                ),
                &BTreeMap::new(),
                false,
                &mut control,
            )
            .unwrap();
            if cycle {
                let first = driver.run(&mut control).unwrap_err();
                assert_eq!(first.code, "FXAR0030");
                assert!(first.detail.contains("circular dependency"));
                assert_eq!(driver.run(&mut control).unwrap_err(), first);
                assert!(
                    !driver
                        .resolver
                        .states
                        .borrow()
                        .iter()
                        .any(|state| matches!(state, super::super::State::Complete(_)))
                );
            } else {
                assert_eq!(driver.run(&mut control).unwrap(), "acyclic");
            }
            assert_eq!(driver.evaluations("first"), 1);
            assert_eq!(driver.evaluations("second"), 1);
            assert!(driver.resolver.stack.borrow().is_empty());
        }
    }

    #[test]
    fn demand_shared_supplied_parameter_skips_constructor_but_not_variable_definition() {
        let program = program(
            r#"
          <xsl:template match="/"><xsl:value-of select="$first"/><xsl:text>|</xsl:text><xsl:value-of select="$second"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:message terminate="yes">default-error</xsl:message></xsl:template>
          <xsl:template name="second_constructor"><xsl:text>variable</xsl:text></xsl:template>"#,
        );
        let mut control = control();
        let driver = Driver::new(
            program,
            document("urn:demand:source", "<r/>"),
            &BTreeMap::from([
                ("first".into(), "supplied".into()),
                ("second".into(), "not-a-parameter".into()),
            ]),
            false,
            &mut control,
        )
        .unwrap();
        assert_eq!(driver.run(&mut control).unwrap(), "supplied|variable");
        assert_eq!(driver.evaluations("first"), 0);
        assert_eq!(driver.evaluations("second"), 1);
    }

    #[test]
    fn demand_shared_constructor_has_no_current_matched_rule() {
        let program = program(
            r#"
          <xsl:template match="/"><xsl:apply-templates select="//b"/></xsl:template>
          <xsl:template match="b"><xsl:value-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:call-template name="delegate"/></xsl:template>
          <xsl:template name="delegate"><xsl:apply-imports/></xsl:template>"#,
        );
        let mut control = control();
        let driver = Driver::new(
            program,
            document("urn:demand:source", "<r><b/></r>"),
            &BTreeMap::new(),
            false,
            &mut control,
        )
        .unwrap();
        let error = driver.run(&mut control).unwrap_err();
        assert_eq!(error.code, "XTDE0560");
        assert_eq!(error.location.unwrap().resource, "urn:demand:stylesheet");
        assert!(driver.resolver.stack.borrow().is_empty());
    }

    #[test]
    fn demand_shared_result_retention_exact_and_one_less_preserve_cleanup() {
        use crate::execution_control_experiment::WorkDomain;
        let program = program(
            r#"
          <xsl:template match="/"><xsl:value-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor"><box><xsl:text>owned</xsl:text></box></xsl:template>"#,
        );
        let source = document("urn:demand:source", "<r/>");
        let mut baseline = control();
        let driver = Driver::new(
            Arc::clone(&program),
            Arc::clone(&source),
            &BTreeMap::new(),
            false,
            &mut baseline,
        )
        .unwrap();
        assert_eq!(
            driver
                .read_string("first", &mut baseline)
                .unwrap()
                .as_deref(),
            Some("owned")
        );
        let exact = baseline.consumed(WorkDomain::XdmNode);
        for (limit, success) in [(exact, true), (exact - 1, false)] {
            let mut limits = WorkLimits::unbounded();
            limits.xdm_nodes = limit;
            let mut control = InvocationControl::new(CancellationToken::new(), limits);
            let driver = Driver::new(
                Arc::clone(&program),
                Arc::clone(&source),
                &BTreeMap::new(),
                false,
                &mut control,
            )
            .unwrap();
            let result = driver.read_string("first", &mut control);
            assert_eq!(result.is_ok(), success);
            if !success {
                assert_eq!(result.unwrap_err().code, "FXCT0002");
                assert!(
                    !driver
                        .resolver
                        .states
                        .borrow()
                        .iter()
                        .any(|state| matches!(state, super::super::State::Complete(_)))
                );
            }
            assert!(driver.resolver.stack.borrow().is_empty());
        }
    }

    #[test]
    fn demand_shared_cancellation_during_constructor_does_not_publish_partial_tree() {
        use crate::execution_control_experiment::WorkDomain;
        let program = program(
            r#"
          <xsl:template match="/"><xsl:value-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor"><box><xsl:text>one</xsl:text><xsl:text>two</xsl:text></box></xsl:template>"#,
        );
        let mut control = control().cancelling_on_charge(WorkDomain::XsltInstruction, 1);
        let driver = Driver::new(
            program,
            document("urn:demand:source", "<r/>"),
            &BTreeMap::new(),
            false,
            &mut control,
        )
        .unwrap();
        let error = driver.read_string("first", &mut control).unwrap_err();
        assert_eq!(error.code, "FXCT0001");
        assert_eq!(driver.evaluations("first"), 1);
        assert!(
            !driver
                .resolver
                .states
                .borrow()
                .iter()
                .any(|state| matches!(state, super::super::State::Complete(_)))
        );
        assert!(driver.resolver.stack.borrow().is_empty());
    }

    #[test]
    fn demand_shared_invocations_isolate_values_and_release_owners() {
        let program = program(
            r#"
          <xsl:template match="/"><xsl:value-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:value-of select="/r/@value"/></xsl:template>"#,
        );
        let mut first_control = control();
        let mut second_control = control();
        let first = Driver::new(
            Arc::clone(&program),
            document("urn:demand:source", "<r value='one'/>"),
            &BTreeMap::new(),
            false,
            &mut first_control,
        )
        .unwrap();
        let second = Driver::new(
            Arc::clone(&program),
            document("urn:demand:source", "<r value='two'/>"),
            &BTreeMap::new(),
            false,
            &mut second_control,
        )
        .unwrap();
        let weak_driver = Rc::downgrade(&first);
        let weak_source = Arc::downgrade(&first.source);
        let weak_program = Arc::downgrade(&program);
        assert_eq!(first.run(&mut first_control).unwrap(), "one");
        assert_eq!(second.run(&mut second_control).unwrap(), "two");
        let value = first
            .resolver
            .states
            .borrow()
            .iter()
            .find_map(|state| match state {
                super::super::State::Complete(value) => Some(Arc::clone(value)),
                _ => None,
            })
            .unwrap();
        drop(first);
        assert!(weak_driver.upgrade().is_none());
        assert!(weak_source.upgrade().is_none());
        assert_eq!(value.string_value(&mut first_control).unwrap(), "one");
        assert_eq!(second.run(&mut second_control).unwrap(), "two");
        drop(program);
        drop(second);
        assert!(weak_program.upgrade().is_none());
    }

    #[test]
    fn demand_shared_builtin_dispatch_uses_same_executor() {
        let program = program_at(
            "1.0",
            r#"
          <xsl:template match="/"><xsl:value-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:apply-templates select="//b"/></xsl:template>"#,
        );
        let mut control = control();
        let driver = Driver::new(
            program,
            document("urn:demand:source", "<r><b>builtin</b></r>"),
            &BTreeMap::new(),
            false,
            &mut control,
        )
        .unwrap();
        assert_eq!(driver.run(&mut control).unwrap(), "builtin");
        assert_eq!(driver.evaluations("first"), 1);
    }

    #[test]
    fn demand_shared_concurrent_invocations_share_only_immutable_compilation() {
        let program = program(
            r#"
          <xsl:template match="/"><xsl:value-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:message terminate="yes">not-supplied</xsl:message></xsl:template>"#,
        );
        let source = document("urn:demand:source", "<r/>");
        let outputs = std::thread::scope(|scope| {
            let handles = (0..4)
                .map(|index| {
                    let program = Arc::clone(&program);
                    let source = Arc::clone(&source);
                    scope.spawn(move || {
                        let mut control = control();
                        let value = format!("invocation-{index}");
                        let driver = Driver::new(
                            program,
                            source,
                            &BTreeMap::from([("first".into(), value.clone())]),
                            index % 2 == 0,
                            &mut control,
                        )
                        .unwrap();
                        for _ in 0..8 {
                            assert_eq!(driver.run(&mut control).unwrap(), value);
                        }
                        assert_eq!(driver.evaluations("first"), 0);
                        value
                    })
                })
                .collect::<Vec<_>>();
            handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .collect::<Vec<_>>()
        });
        assert_eq!(
            outputs,
            [
                "invocation-0",
                "invocation-1",
                "invocation-2",
                "invocation-3"
            ]
        );
        assert_eq!(Arc::strong_count(&program), 1);
        assert_eq!(Arc::strong_count(&source), 1);
    }

    #[test]
    fn demand_shared_modern_string_compile_boundary_remains_explicit() {
        let stylesheet = document(
            "urn:demand:modern-boundary",
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:variable name="first" select="'literal'"/>
            <xsl:template match="/"><xsl:value-of select="string($first)"/></xsl:template>
          </xsl:stylesheet>"#,
        );
        let error = compile_stylesheet(&stylesheet).unwrap_err();
        assert_eq!(error.code, "FXXP1001");
        assert_eq!(error.location.resource, "urn:demand:modern-boundary");
    }

    #[test]
    fn demand_shared_qualified_names_separate_equal_local_names_and_preserve_aliases() {
        let stylesheet = document(
            "urn:demand:qualified",
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:a="urn:a" xmlns:alias="urn:a" xmlns:b="urn:b" version="1.0">
            <xsl:variable name="a:value" select="'placeholder'"/>
            <xsl:variable name="b:value" select="'placeholder'"/>
            <xsl:template match="/"><xsl:value-of select="$a:value"/><xsl:text>|</xsl:text><xsl:value-of select="$b:value"/><xsl:text>|</xsl:text><xsl:value-of select="$alias:value"/></xsl:template>
            <xsl:template name="a:value_constructor"><xsl:text>A</xsl:text></xsl:template>
            <xsl:template name="b:value_constructor"><xsl:text>B</xsl:text></xsl:template>
          </xsl:stylesheet>"#,
        );
        let program = Arc::new(compile_stylesheet(&stylesheet).unwrap());
        let mut control = control();
        let driver = Driver::new(
            program,
            document("urn:demand:source", "<r/>"),
            &BTreeMap::new(),
            false,
            &mut control,
        )
        .unwrap();
        assert_eq!(driver.run(&mut control).unwrap(), "A|B|A");
        assert_eq!(driver.evaluations("Q{urn:a}value"), 1);
        assert_eq!(driver.evaluations("Q{urn:b}value"), 1);
    }

    #[test]
    fn demand_shared_linked_modules_use_resolved_import_precedence() {
        use crate::compile::golden_stylesheet_experiment::compile_stylesheet_with_import_and_include;
        let principal = document(
            "urn:demand:principal",
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
            <xsl:import href="imported"/><xsl:include href="included"/>
            <xsl:variable name="first" select="'principal-placeholder'"/>
            <xsl:template match="/"><xsl:value-of select="$second"/></xsl:template>
            <xsl:template name="first_constructor"><xsl:text>principal</xsl:text></xsl:template>
          </xsl:stylesheet>"#,
        );
        let imported = document(
            "urn:demand:imported",
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
            <xsl:variable name="first" select="'imported-placeholder'"/>
            <xsl:template name="first_constructor"><xsl:message terminate="yes">wrong-precedence</xsl:message></xsl:template>
          </xsl:stylesheet>"#,
        );
        let included = document(
            "urn:demand:included",
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
            <xsl:variable name="second" select="'included-placeholder'"/>
            <xsl:template name="second_constructor"><xsl:value-of select="$first"/></xsl:template>
          </xsl:stylesheet>"#,
        );
        let program = Arc::new(
            compile_stylesheet_with_import_and_include(
                &principal,
                (&imported, imported.children(imported.document_node())[0]),
                (&included, included.children(included.document_node())[0]),
            )
            .unwrap(),
        );
        let mut control = control();
        let driver = Driver::new(
            program,
            document("urn:demand:source", "<r/>"),
            &BTreeMap::new(),
            false,
            &mut control,
        )
        .unwrap();
        assert_eq!(driver.run(&mut control).unwrap(), "principal");
        assert_eq!(driver.evaluations("first"), 1);
        assert_eq!(driver.evaluations("second"), 1);
    }
}
