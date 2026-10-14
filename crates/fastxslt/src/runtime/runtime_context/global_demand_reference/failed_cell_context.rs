//! Failed-cell attachment tests. Paths are explicit reference metadata; the
//! shared executor's recursive read-site callback integration remains separate.

use super::failure_context_reference::ContextEnd;
use super::*;
use crate::xdm::owned_tree_experiment::SourceLocation;

fn edge(binding: usize) -> Dependency {
    Dependency {
        binding,
        declaration: Arc::new(SourceLocation {
            resource: "urn:decl".into(),
            span: binding..binding + 1,
        }),
        read: Some(Arc::new(SourceLocation {
            resource: "urn:read".into(),
            span: binding + 10..binding + 11,
        })),
    }
}

fn fail(
    binding: usize,
    resolver: &DemandReference<String, &'static str>,
    control: &mut InvocationControl,
) -> Result<String, Arc<Failure<&'static str>>> {
    if binding == 0 {
        resolver
            .read(1, control, &fail)
            .map(|value| (*value).clone())
    } else {
        Err(Arc::new(Failure::Evaluation("origin")))
    }
}

#[test]
fn demand_failed_cell_context_reuses_primary_and_does_not_retain_a_second_value_map() {
    let mut control = InvocationControl::unbounded();
    let resolver = DemandReference::new(&[None, None], 8, &mut control).unwrap();
    let error = resolver.read(0, &mut control, &fail).unwrap_err();
    let path = [edge(0), edge(1)];
    let context = resolver.failed_context(0, &path, 2, &mut control).unwrap();
    assert!(Arc::ptr_eq(&error, &context.primary));
    assert_eq!(context.dependencies, path);
    assert_eq!(context.end, ContextEnd::Complete);
    let work = control.consumed(WorkDomain::XdmNode);
    let repeated = resolver.failed_context(0, &[], 0, &mut control).unwrap();
    assert!(Arc::ptr_eq(&context, &repeated));
    assert_eq!(control.consumed(WorkDomain::XdmNode), work);
    assert!(Arc::ptr_eq(
        &error,
        &resolver.read(0, &mut control, &fail).unwrap_err()
    ));
    assert!(Arc::ptr_eq(
        &error,
        &resolver.read(1, &mut control, &fail).unwrap_err()
    ));
    let weak = Arc::downgrade(&error);
    drop(error);
    drop(resolver);
    assert!(weak.upgrade().is_some());
    drop(context);
    drop(repeated);
    assert!(weak.upgrade().is_none());
}

#[test]
fn demand_failed_cell_context_ignores_unresolved_successful_and_unknown_cells() {
    let mut control = InvocationControl::unbounded();
    let resolver = DemandReference::<String, &'static str>::new(
        &[None, Some(Arc::new("ready".into()))],
        8,
        &mut control,
    )
    .unwrap();
    let before = control.consumed(WorkDomain::XdmNode);
    for binding in [0, 1, 2] {
        assert!(
            resolver
                .failed_context(binding, &[edge(0)], 1, &mut control)
                .is_none()
        );
    }
    assert_eq!(control.consumed(WorkDomain::XdmNode), before);
}

#[test]
fn demand_failed_cell_context_truncation_is_cached_without_poisoning_primary() {
    let mut setup = InvocationControl::unbounded();
    let resolver = DemandReference::new(&[None, None], 8, &mut setup).unwrap();
    let error = resolver.read(0, &mut setup, &fail).unwrap_err();
    let mut limits = crate::execution_control_experiment::WorkLimits::unbounded();
    limits.xdm_nodes = 1;
    let mut control = InvocationControl::new(
        crate::execution_control_experiment::CancellationToken::new(),
        limits,
    );
    let context = resolver
        .failed_context(0, &[edge(0), edge(1)], 2, &mut control)
        .unwrap();
    assert!(Arc::ptr_eq(&error, &context.primary));
    assert_eq!(context.dependencies.len(), 1);
    assert!(matches!(
        context.end,
        ContextEnd::Control(ControlFailure::BudgetExhausted { .. })
    ));
    let repeated = resolver
        .failed_context(0, &[edge(0), edge(1)], 2, &mut setup)
        .unwrap();
    assert!(Arc::ptr_eq(&context, &repeated));
    assert!(resolver.stack.borrow().is_empty());
}

#[test]
fn demand_failed_cell_context_never_aliases_distinct_equal_text_failures() {
    let mut control = InvocationControl::unbounded();
    let resolver = DemandReference::new(&[None, None, None], 8, &mut control).unwrap();
    let first = resolver.read(1, &mut control, &fail).unwrap_err();
    let second = resolver.read(2, &mut control, &fail).unwrap_err();
    assert!(!Arc::ptr_eq(&first, &second));
    let first_context = resolver
        .failed_context(1, &[edge(1)], 1, &mut control)
        .unwrap();
    let second_context = resolver
        .failed_context(2, &[edge(2)], 1, &mut control)
        .unwrap();
    assert!(Arc::ptr_eq(&first, &first_context.primary));
    assert!(Arc::ptr_eq(&second, &second_context.primary));
    assert!(!Arc::ptr_eq(&first_context, &second_context));
    assert_eq!(first_context.dependencies[0].binding, 1);
    assert_eq!(second_context.dependencies[0].binding, 2);
}
