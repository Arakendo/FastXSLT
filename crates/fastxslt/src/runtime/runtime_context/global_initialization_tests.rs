//! Owned initializer handoff: evaluation cannot publish into completed maps.

use super::*;
use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
use crate::xslt::golden_semantics_experiment::{GlobalBindingKind, Xslt10TemporaryTextPart};
use std::cell::RefCell;

pub(super) fn document(xml: &str) -> Document {
    Document::from_parsed(
        parse_document(
            "urn:initializer-handoff",
            xml.as_bytes(),
            ParseLimits {
                max_events: 1024,
                max_depth: 64,
            },
        )
        .unwrap(),
    )
    .unwrap()
}

fn binding(default: GlobalBindingDefault) -> GlobalBinding {
    GlobalBinding {
        declaration_reference: SourceLocation {
            resource: "urn:initializer-handoff".into(),
            span: 0..1,
        },
        kind: GlobalBindingKind::Variable,
        name: "target".into(),
        required: false,
        default,
        import_precedence: 0,
    }
}

pub(super) fn program(declarations: &str) -> StylesheetProgram {
    compile_stylesheet(&document(&format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">{declarations}<xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
    ))).unwrap()
}

#[test]
fn initialized_value_moves_into_exactly_one_completed_kind() {
    let source = Arc::new(document("<r/>"));
    let nodes = vec![source.document_node()];
    let mut control = InvocationControl::unbounded();
    let tree = materialize_parentless_temporary_node(
        TemporaryNodeKind::Text("payload".into()),
        "handoff",
        &mut control,
    )
    .unwrap();
    let identity = tree.identity;
    let mut globals = RuntimeGlobals::default();
    InitializedGlobal::Empty.publish(&mut globals, "empty");
    InitializedGlobal::Atomic(AtomicValue::string("atomic")).publish(&mut globals, "atomic");
    InitializedGlobal::SourceNodes(nodes.clone()).publish(&mut globals, "nodes");
    InitializedGlobal::TemporaryTree(tree).publish(&mut globals, "tree");
    InitializedGlobal::LiteralDocument(super::super::super::dynamic_document::DynamicDocument {
        identity: 7,
        document: Arc::clone(&source),
    })
    .publish(&mut globals, "document");
    assert_eq!(globals.empty_sequences.len(), 1);
    assert_eq!(globals.atomics.len(), 1);
    assert_eq!(globals.nodes.len(), 1);
    assert_eq!(globals.temporary_trees.len(), 1);
    assert_eq!(globals.literal_documents.len(), 1);
    assert_eq!(globals.nodes["nodes"], nodes);
    assert_eq!(globals.temporary_trees["tree"].identity, identity);
    assert!(Arc::ptr_eq(
        &globals.literal_documents["document"].document,
        &source
    ));
}

#[test]
fn initializer_reads_completed_values_without_mutating_or_cloning_the_maps() {
    let mut globals = RuntimeGlobals::default();
    InitializedGlobal::Atomic(AtomicValue::string("seed")).publish(&mut globals, "seed");
    let retained = Arc::clone(&globals.atomics);
    let value = materialize_global_default(
        &globals,
        &binding(GlobalBindingDefault::Variable("seed".into())),
        None,
        "handoff",
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert!(Arc::ptr_eq(&retained, &globals.atomics));
    assert_eq!(globals.atomics.len(), 1);
    value.publish(&mut globals, "target");
    assert_eq!(globals.atomics["target"].lexical(), "seed");
    assert_eq!(retained.len(), 1);
}

#[test]
fn failed_initializer_keeps_completed_values_and_prior_messages_without_partial_publication() {
    let mut globals = RuntimeGlobals::default();
    InitializedGlobal::Atomic(AtomicValue::string("seed")).publish(&mut globals, "seed");
    let retained = Arc::clone(&globals.atomics);
    let initializer = binding(GlobalBindingDefault::Xslt10TemporaryTextParts(vec![
        Xslt10TemporaryTextPart::Message("before failure".into()),
        Xslt10TemporaryTextPart::Text("partial".into()),
        Xslt10TemporaryTextPart::Variable("missing".into()),
    ]));
    let mut control = InvocationControl::unbounded();
    let failure = materialize_global_default(&globals, &initializer, None, "handoff", &mut control)
        .unwrap_err();
    assert_eq!(failure.code, "FXRT0002");
    assert_eq!(control.messages(), &["before failure"]);
    assert!(Arc::ptr_eq(&retained, &globals.atomics));
    assert!(globals.temporary_trees.is_empty());
    assert_eq!(globals.atomics.len(), 1);
}

#[test]
fn initializer_handoff_preserves_exact_retention_limit_and_cancellation() {
    let globals = RuntimeGlobals::default();
    let initializer = binding(GlobalBindingDefault::TemporaryText("payload".into()));
    let mut measured = InvocationControl::unbounded();
    materialize_global_default(&globals, &initializer, None, "handoff", &mut measured).unwrap();
    let cost = measured.consumed(WorkDomain::XdmNode);
    assert!(cost > 0);
    for (limit, succeeds) in [(cost, true), (cost - 1, false)] {
        let mut control = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xdm_nodes: limit,
                ..WorkLimits::unbounded()
            },
        );
        let result =
            materialize_global_default(&globals, &initializer, None, "handoff", &mut control);
        assert_eq!(result.is_ok(), succeeds);
        assert!(globals.temporary_trees.is_empty());
    }
    let mut cancelled = InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 0);
    let failure =
        materialize_global_default(&globals, &initializer, None, "handoff", &mut cancelled)
            .unwrap_err();
    assert_eq!(failure.code, "FXCT0001");
    assert!(globals.temporary_trees.is_empty());
}

#[test]
fn eager_handoff_preserves_parameter_override_and_unused_default_failure() {
    let program = program(r#"<xsl:param name="p" select="/r"/>"#);
    let dynamic_documents = RefCell::new(BTreeMap::new());
    let documents = GlobalDocumentContext {
        multiple_match_policy:
            crate::runtime::golden_runtime_experiment::MultipleMatchPolicy::UseLast,
        resource_snapshot: None,
        denied_resources: None,
        dynamic_documents: &dynamic_documents,
    };
    let parameters = BTreeMap::from([(
        "p".into(),
        InvocationParameter {
            value: InvocationParameterValue::Atomic(AtomicValue::string("supplied")),
            tunnel: false,
        },
    )]);
    let globals = materialize_global_defaults(
        &program,
        None,
        &parameters,
        "handoff",
        &documents,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(globals.atomics["p"].lexical(), "supplied");
    assert!(globals.nodes.is_empty());
    let failure = materialize_global_defaults(
        &program,
        None,
        &BTreeMap::new(),
        "handoff",
        &documents,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(failure.code, "FXRT1004");
    let source = document("<r/>");
    let globals = materialize_global_defaults(
        &program,
        Some(&source),
        &BTreeMap::new(),
        "handoff",
        &documents,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    assert_eq!(globals.nodes["p"].len(), 1);
    assert!(globals.principal_owner.is_some());
}

// A bounded test driver, not the production resolver. It exercises recursive
// entry into the actual initializer evaluator through the new read boundary.
struct RecursiveReads<'a> {
    bindings: BTreeMap<String, GlobalBinding>,
    source: Option<&'a Document>,
    completed: RefCell<RuntimeGlobals>,
    finished: RefCell<HashSet<String>>,
    active: RefCell<HashSet<String>>,
    failures: RefCell<BTreeMap<String, ExecutionFailure>>,
    evaluated: RefCell<Vec<String>>,
}

impl<'a> RecursiveReads<'a> {
    fn new(bindings: Vec<GlobalBinding>, source: Option<&'a Document>) -> Self {
        assert!(bindings.len() <= 16);
        Self {
            bindings: bindings
                .into_iter()
                .map(|binding| (binding.name.clone(), binding))
                .collect(),
            source,
            completed: RefCell::new(RuntimeGlobals::default()),
            finished: RefCell::new(HashSet::new()),
            active: RefCell::new(HashSet::new()),
            failures: RefCell::new(BTreeMap::new()),
            evaluated: RefCell::new(Vec::new()),
        }
    }

    fn resolve(&self, name: &str, control: &mut InvocationControl) -> Result<(), ExecutionFailure> {
        // A read must reach resolution before borrowing any completed map.
        assert!(self.completed.try_borrow_mut().is_ok());
        if self.finished.borrow().contains(name) {
            return Ok(());
        }
        if let Some(error) = self.failures.borrow().get(name) {
            return Err(error.clone());
        }
        let Some(binding) = self.bindings.get(name) else {
            return Err(failure(
                "FXRT0002",
                FailureCategory::Invalid,
                Some("handoff"),
                "missing test binding",
            ));
        };
        assert!(
            self.active.borrow_mut().insert(name.into()),
            "test driver expects an acyclic graph"
        );
        self.evaluated.borrow_mut().push(name.into());
        let result = materialize_global_default(self, binding, self.source, "handoff", control);
        assert!(self.active.borrow_mut().remove(name));
        match result {
            Ok(value) => {
                value.publish(&mut self.completed.borrow_mut(), name);
                self.finished.borrow_mut().insert(name.into());
                Ok(())
            }
            Err(error) => {
                self.failures
                    .borrow_mut()
                    .insert(name.into(), error.clone());
                Err(error)
            }
        }
    }
}

impl GlobalInitializerReads for RecursiveReads<'_> {
    fn with_completed<T>(
        &self,
        name: &str,
        control: &mut InvocationControl,
        read: impl FnOnce(&RuntimeGlobals, &mut InvocationControl) -> Result<T, ExecutionFailure>,
    ) -> Result<T, ExecutionFailure> {
        self.resolve(name, control)?;
        read(&self.completed.borrow(), control)
    }
}

pub(super) fn named(name: &str, default: GlobalBindingDefault) -> GlobalBinding {
    GlobalBinding {
        name: name.into(),
        ..binding(default)
    }
}

#[test]
fn dependency_resolution_occurs_at_actual_read_after_preceding_messages() {
    let reader = RecursiveReads::new(
        vec![
            named(
                "outer",
                GlobalBindingDefault::Xslt10TemporaryTextParts(vec![
                    Xslt10TemporaryTextPart::Message("outer before".into()),
                    Xslt10TemporaryTextPart::Variable("inner".into()),
                    Xslt10TemporaryTextPart::Message("outer after".into()),
                    Xslt10TemporaryTextPart::Variable("inner".into()),
                ]),
            ),
            named(
                "inner",
                GlobalBindingDefault::Xslt10TemporaryTextParts(vec![
                    Xslt10TemporaryTextPart::Message("inner".into()),
                    Xslt10TemporaryTextPart::Text("payload".into()),
                ]),
            ),
        ],
        None,
    );
    let mut control = InvocationControl::unbounded();
    reader.resolve("outer", &mut control).unwrap();
    reader.resolve("inner", &mut control).unwrap();
    assert_eq!(
        control.messages(),
        &["outer before", "inner", "outer after"]
    );
    assert_eq!(*reader.evaluated.borrow(), ["outer", "inner"]);
    assert_eq!(
        global_string_value(&reader, "outer", None, "handoff", &mut control).unwrap(),
        "payloadpayload"
    );
    assert!(reader.active.borrow().is_empty());
}

#[test]
fn recursive_initializer_reads_match_completed_reference_values_and_work() {
    let program = program(
        r#"
        <xsl:variable name="nodes" select="/r/b"/>
        <xsl:variable name="alias" select="$nodes"/>
        <xsl:variable name="descendants" select="$alias/c"/>
        <xsl:variable name="text" select="concat('prefix:', $descendants)"/>
        <xsl:variable name="copy"><xsl:value-of select="$text"/></xsl:variable>
    "#,
    );
    let source = document("<r><b><c>payload</c></b></r>");
    let reader = RecursiveReads::new(program.global_bindings.clone(), Some(&source));
    let mut recursive_control = InvocationControl::unbounded();
    reader.resolve("copy", &mut recursive_control).unwrap();
    let mut expected = RuntimeGlobals::default();
    let mut reference_control = InvocationControl::unbounded();
    for binding in &program.global_bindings {
        materialize_global_default(
            &expected,
            binding,
            Some(&source),
            "handoff",
            &mut reference_control,
        )
        .unwrap()
        .publish(&mut expected, &binding.name);
        reader
            .resolve(&binding.name, &mut recursive_control)
            .unwrap();
    }
    let actual = reader.completed.borrow();
    assert_eq!(actual.atomics, expected.atomics);
    assert_eq!(actual.nodes, expected.nodes);
    assert_eq!(actual.temporary_trees, expected.temporary_trees);
    for domain in [
        WorkDomain::XdmNode,
        WorkDomain::XdmStringValueNode,
        WorkDomain::XPathNodeVisit,
        WorkDomain::XPathOperation,
        WorkDomain::XsltInstruction,
        WorkDomain::ResultNode,
        WorkDomain::ResultTextByte,
    ] {
        assert_eq!(
            recursive_control.consumed(domain),
            reference_control.consumed(domain),
            "{domain:?}"
        );
    }
    assert_eq!(
        reader.evaluated.borrow().len(),
        program.global_bindings.len()
    );
}

#[test]
fn recursive_read_failure_does_not_replay_messages_or_publish_dependents() {
    let reader = RecursiveReads::new(
        vec![
            named(
                "outer",
                GlobalBindingDefault::Xslt10TemporaryTextParts(vec![
                    Xslt10TemporaryTextPart::Message("before".into()),
                    Xslt10TemporaryTextPart::Variable("inner".into()),
                    Xslt10TemporaryTextPart::Message("unreachable".into()),
                ]),
            ),
            named(
                "inner",
                GlobalBindingDefault::Xslt10TemporaryTextParts(vec![
                    Xslt10TemporaryTextPart::Variable("missing".into()),
                ]),
            ),
        ],
        None,
    );
    let mut control = InvocationControl::unbounded();
    let first = reader.resolve("outer", &mut control).unwrap_err();
    assert_eq!(reader.resolve("outer", &mut control).unwrap_err(), first);
    assert_eq!(reader.resolve("inner", &mut control).unwrap_err(), first);
    assert_eq!(control.messages(), &["before"]);
    assert!(reader.completed.borrow().temporary_trees.is_empty());
    assert!(reader.active.borrow().is_empty());
    assert_eq!(*reader.evaluated.borrow(), ["outer", "inner"]);
}

#[test]
fn source_precondition_failure_does_not_resolve_a_dependency_early() {
    let path = crate::xpath::path_experiment::parse_location_path(
        "child",
        binding(GlobalBindingDefault::EmptySequence).declaration_reference,
    )
    .unwrap();
    let reader = RecursiveReads::new(
        vec![
            named(
                "outer",
                GlobalBindingDefault::SourceVariablePath {
                    variable: "inner".into(),
                    path,
                },
            ),
            named(
                "inner",
                GlobalBindingDefault::Xslt10TemporaryTextParts(vec![
                    Xslt10TemporaryTextPart::Message("must not run".into()),
                ]),
            ),
        ],
        None,
    );
    let mut control = InvocationControl::unbounded();
    assert_eq!(
        reader.resolve("outer", &mut control).unwrap_err().code,
        "FXRT1004"
    );
    assert!(control.messages().is_empty());
    assert_eq!(*reader.evaluated.borrow(), ["outer"]);
}

#[test]
fn nested_initializer_cancellation_preserves_failure_and_clears_active_reads() {
    let reader = RecursiveReads::new(
        vec![
            named(
                "outer",
                GlobalBindingDefault::Xslt10ConditionalText {
                    variable: "inner".into(),
                    expected: "payload".into(),
                    when_true: "yes".into(),
                    when_false: "no".into(),
                },
            ),
            named(
                "inner",
                GlobalBindingDefault::TemporaryText("payload".into()),
            ),
        ],
        None,
    );
    let mut control = InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 0);
    let error = reader.resolve("outer", &mut control).unwrap_err();
    assert_eq!(error.code, "FXCT0001");
    assert_eq!(reader.resolve("inner", &mut control).unwrap_err(), error);
    assert!(reader.active.borrow().is_empty());
    assert!(reader.completed.borrow().temporary_trees.is_empty());
    assert_eq!(*reader.evaluated.borrow(), ["outer", "inner"]);
}
