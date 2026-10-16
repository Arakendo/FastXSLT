use super::super::tests::{document, named, program};
use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xslt::golden_semantics_experiment::Xslt10TemporaryTextPart;

pub(super) fn with_owner<T>(
    program: &StylesheetProgram,
    source: Option<&Document>,
    parameters: &BTreeMap<String, InvocationParameter>,
    test: impl FnOnce(GlobalInitialization<'_>) -> T,
) -> T {
    let dynamic = RefCell::new(BTreeMap::new());
    let documents = GlobalDocumentContext {
        multiple_match_policy:
            crate::runtime::golden_runtime_experiment::MultipleMatchPolicy::UseLast,
        resource_snapshot: None,
        denied_resources: None,
        dynamic_documents: &dynamic,
    };
    test(GlobalInitialization::new(
        program, source, parameters, "owner", &documents,
    ))
}

#[test]
fn production_owner_recurses_at_read_and_eagerly_reuses_completed_values() {
    let mut program = program("");
    program.global_bindings = vec![
        named(
            "outer",
            GlobalBindingDefault::Xslt10TemporaryTextParts(vec![
                Xslt10TemporaryTextPart::Message("before".into()),
                Xslt10TemporaryTextPart::Variable("inner".into()),
                Xslt10TemporaryTextPart::Message("after".into()),
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
        named(
            "unused",
            GlobalBindingDefault::Xslt10TemporaryTextParts(vec![Xslt10TemporaryTextPart::Message(
                "unused still evaluated".into(),
            )]),
        ),
    ];
    with_owner(&program, None, &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        owner.resolve_index(0, &mut control).unwrap();
        assert_eq!(owner.depth.get(), 0);
        assert!(owner.active.iter().all(|slot| slot.get() == usize::MAX));
        let globals = owner.initialize_all(&mut control).unwrap();
        assert_eq!(
            control.messages(),
            &["before", "inner", "after", "unused still evaluated"]
        );
        assert_eq!(
            super::super::global_string_value(&globals, "outer", None, "owner", &mut control)
                .unwrap(),
            "payloadpayload"
        );
        assert_eq!(control.consumed(WorkDomain::XPathOperation), 1);
    });
}

#[test]
fn production_owner_supplied_parameter_suppresses_recursive_default() {
    let mut program =
        program(r#"<xsl:param name="p" select="/r"/><xsl:variable name="q" select="$p"/>"#);
    program.global_bindings.reverse();
    let parameters = BTreeMap::from([(
        "p".into(),
        InvocationParameter {
            value: InvocationParameterValue::Atomic(AtomicValue::string("supplied")),
            tunnel: false,
        },
    )]);
    with_owner(&program, None, &parameters, |owner| {
        let values = owner
            .initialize_all(&mut InvocationControl::unbounded())
            .unwrap();
        assert_eq!(values.atomics["q"].lexical(), "supplied");
        assert!(values.nodes.is_empty());
    });
}

#[test]
fn production_owner_circularity_is_runtime_reentry_and_failure_is_stable() {
    let mut program = program("");
    program.global_bindings = vec![
        named("a", GlobalBindingDefault::Variable("b".into())),
        named("b", GlobalBindingDefault::Variable("a".into())),
    ];
    with_owner(&program, None, &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        let error = owner.resolve_index(0, &mut control).unwrap_err();
        assert_eq!(error.code, "XTDE0640");
        assert_eq!(error.category, FailureCategory::Invalid);
        assert_eq!(owner.resolve_index(1, &mut control).unwrap_err(), error);
        assert_eq!(owner.resolve_index(0, &mut control).unwrap_err(), error);
        assert_eq!(owner.depth.get(), 0);
        assert!(owner.completed.borrow().atomics.is_empty());
        assert_eq!(owner.failure.borrow().as_ref(), Some(&error));
        assert_eq!(control.consumed(WorkDomain::XPathOperation), 1);
    });
}

#[test]
fn production_owner_preserves_originating_failure_without_message_replay() {
    let mut program = program("");
    program.global_bindings = vec![
        named(
            "outer",
            GlobalBindingDefault::Xslt10TemporaryTextParts(vec![
                Xslt10TemporaryTextPart::Message("before failure".into()),
                Xslt10TemporaryTextPart::Variable("inner".into()),
            ]),
        ),
        named("inner", GlobalBindingDefault::Variable("missing".into())),
    ];
    with_owner(&program, None, &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        let first = owner.resolve_index(0, &mut control).unwrap_err();
        assert_eq!(first.code, "FXRT0002");
        assert_eq!(owner.resolve_index(0, &mut control).unwrap_err(), first);
        assert_eq!(owner.resolve_index(1, &mut control).unwrap_err(), first);
        assert_eq!(control.messages(), &["before failure"]);
        assert!(owner.completed.borrow().temporary_trees.is_empty());
        assert_eq!(owner.depth.get(), 0);
    });
}

fn chain(count: usize) -> StylesheetProgram {
    let mut program = program("");
    program.global_bindings = (0..count)
        .map(|index| {
            named(
                &format!("v{index}"),
                if index + 1 == count {
                    GlobalBindingDefault::Text("end".into())
                } else {
                    GlobalBindingDefault::Variable(format!("v{}", index + 1))
                },
            )
        })
        .collect();
    program
}

#[test]
fn production_owner_bounds_recursive_depth_and_charges_recursive_work() {
    for count in [128, 129] {
        with_owner(&chain(count), None, &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::unbounded();
            let result = owner.resolve_index(0, &mut control);
            if count == 128 {
                result.unwrap();
            } else {
                assert_eq!(result.unwrap_err().category, FailureCategory::Limit);
            }
            assert_eq!(control.consumed(WorkDomain::XPathOperation), 127);
            assert_eq!(owner.depth.get(), 0);
        });
    }
    for (limit, succeeds) in [(127, true), (126, false)] {
        with_owner(&chain(128), None, &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xpath_operations: limit,
                    ..WorkLimits::unbounded()
                },
            );
            assert_eq!(owner.resolve_index(0, &mut control).is_ok(), succeeds);
            assert_eq!(owner.depth.get(), 0);
        });
    }
}

#[test]
fn production_owner_cancellation_leaves_no_active_or_partial_value() {
    with_owner(&chain(2), None, &BTreeMap::new(), |owner| {
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 0);
        let first = owner.resolve_index(0, &mut control).unwrap_err();
        assert_eq!(first.code, "FXCT0001");
        assert_eq!(owner.resolve_index(0, &mut control).unwrap_err(), first);
        assert_eq!(owner.depth.get(), 0);
        assert!(owner.completed.borrow().atomics.is_empty());
    });
}

#[test]
fn production_owner_unwind_clears_active_marker_and_forbids_restart() {
    with_owner(&chain(1), None, &BTreeMap::new(), |owner| {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            owner.active[0].set(0);
            owner.depth.set(1);
            let _active = ActiveInitialization {
                owner: &owner,
                depth: 0,
                settled: false,
            };
            panic!("injected initializer unwind");
        }));
        assert!(result.is_err());
        assert_eq!(owner.depth.get(), 0);
        assert_eq!(owner.active[0].get(), usize::MAX);
        assert!(owner.poisoned.get());
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                owner
                    .resolve_index(0, &mut InvocationControl::unbounded())
                    .unwrap();
            }))
            .is_err()
        );
        assert!(owner.completed.borrow().atomics.is_empty());
    });
}

#[test]
fn production_owner_ordered_source_values_preserve_existing_charges() {
    let program = program(
        r#"<xsl:variable name="nodes" select="/r/b"/><xsl:variable name="alias" select="$nodes"/>"#,
    );
    let source = document("<r><b>payload</b></r>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut actual_control = InvocationControl::unbounded();
        let actual = owner.initialize_all(&mut actual_control).unwrap();
        let mut expected_control = InvocationControl::unbounded();
        let mut expected = RuntimeGlobals::default();
        for binding in &program.global_bindings {
            materialize_global_default(
                &expected,
                binding,
                Some(&source),
                "owner",
                &mut expected_control,
            )
            .unwrap()
            .publish(&mut expected, &binding.name);
        }
        assert_eq!(actual.nodes, expected.nodes);
        for domain in [
            WorkDomain::XPathOperation,
            WorkDomain::XPathNodeVisit,
            WorkDomain::XdmNode,
        ] {
            assert_eq!(
                actual_control.consumed(domain),
                expected_control.consumed(domain)
            );
        }
    });
}
