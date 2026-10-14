//! Bounded optional failure context experiment. This is not a public diagnostic
//! format or compiler provenance integration. Metadata is supplied explicitly.

use super::*;
use crate::xdm::owned_tree_experiment::SourceLocation;

#[path = "compiled_provenance.rs"]
mod compiled_provenance;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Dependency {
    pub(super) binding: usize,
    pub(super) declaration: Arc<SourceLocation>,
    pub(super) read: Option<Arc<SourceLocation>>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ContextEnd {
    Complete,
    EntryLimit,
    Control(ControlFailure),
}

pub(super) struct FailureContext<E> {
    pub(super) primary: Arc<E>,
    pub(super) dependencies: Vec<Dependency>,
    pub(super) end: ContextEnd,
}

// One slot belongs to one already-established failure, not to a global registry
// or variable name. Repeated reads cannot grow or relabel its first observation.
pub(super) struct CapturedFailure<E> {
    pub(super) primary: Arc<E>,
    pub(super) context: std::cell::OnceCell<Arc<FailureContext<E>>>,
}

impl<E> CapturedFailure<E> {
    pub(super) fn capture_once(
        &self,
        path: &[Dependency],
        max_entries: usize,
        control: &mut InvocationControl,
    ) -> Arc<FailureContext<E>> {
        Arc::clone(self.context.get_or_init(|| {
            Arc::new(FailureContext::capture(
                Arc::clone(&self.primary),
                path,
                max_entries,
                control,
            ))
        }))
    }
}

impl<E> FailureContext<E> {
    pub(super) fn capture(
        primary: Arc<E>,
        path: &[Dependency],
        max_entries: usize,
        control: &mut InvocationControl,
    ) -> Self {
        let mut result = Self {
            primary,
            dependencies: Vec::new(),
            end: ContextEnd::Complete,
        };
        for edge in path {
            if result.dependencies.len() == max_entries {
                result.end = ContextEnd::EntryLimit;
                break;
            }
            // Provisional reference entry-work charge, not allocator accounting.
            // A secondary failure must not replace the already established one.
            if let Err(error) = control.charge(WorkDomain::XdmNode, 1) {
                result.end = ContextEnd::Control(error);
                break;
            }
            result.dependencies.push(edge.clone());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    use crate::runtime::golden_runtime_experiment::{FailureCategory, failure_at};

    fn location(resource: &str, start: usize) -> Arc<SourceLocation> {
        Arc::new(SourceLocation {
            resource: resource.into(),
            span: start..start + 4,
        })
    }

    fn path() -> Vec<Dependency> {
        let first = location("urn:principal", 10);
        let second = location("urn:included", 20);
        vec![
            Dependency {
                binding: 0,
                declaration: Arc::clone(&first),
                read: Some(location("urn:principal", 80)),
            },
            Dependency {
                binding: 1,
                declaration: second,
                read: Some(location("urn:principal", 110)),
            },
            Dependency {
                binding: 0,
                declaration: first,
                read: Some(location("urn:included", 150)),
            },
        ]
    }

    fn primary() -> Arc<super::super::super::ExecutionFailure> {
        Arc::new(failure_at(
            "XTMM9000",
            FailureCategory::Invalid,
            Some("request"),
            SourceLocation {
                resource: "urn:included".into(),
                span: 200..240,
            },
            "original failure",
        ))
    }

    #[test]
    fn demand_context_preserves_primary_and_ordered_cross_module_locations() {
        let original = primary();
        let path = path();
        let mut control = InvocationControl::unbounded();
        let context = FailureContext::capture(Arc::clone(&original), &path, 3, &mut control);
        assert!(Arc::ptr_eq(&original, &context.primary));
        assert_eq!(context.dependencies, path);
        assert_eq!(context.end, ContextEnd::Complete);
        assert_eq!(control.consumed(WorkDomain::XdmNode), 3);
        assert_eq!(context.primary.location.as_ref().unwrap().span, 200..240);
        assert!(Arc::ptr_eq(
            &context.dependencies[0].declaration,
            &context.dependencies[2].declaration
        ));
        assert_ne!(context.dependencies[0].read, context.dependencies[2].read);
    }

    #[test]
    fn demand_context_entry_cap_reports_omission_without_overreading_or_relabeling() {
        let path = path();
        for cap in 0..=4 {
            let original = primary();
            let mut control = InvocationControl::unbounded();
            let context = FailureContext::capture(Arc::clone(&original), &path, cap, &mut control);
            assert!(Arc::ptr_eq(&original, &context.primary));
            assert_eq!(context.dependencies, path[..cap.min(path.len())]);
            assert_eq!(control.consumed(WorkDomain::XdmNode), cap.min(path.len()));
            assert_eq!(
                context.end,
                if cap < path.len() {
                    ContextEnd::EntryLimit
                } else {
                    ContextEnd::Complete
                }
            );
        }
    }

    #[test]
    fn demand_context_exhaustion_is_secondary_and_partial_trace_is_explicit() {
        let path = path();
        for limit in [0, 2, 3] {
            let mut limits = WorkLimits::unbounded();
            limits.xdm_nodes = limit;
            let mut control = InvocationControl::new(CancellationToken::new(), limits);
            let original = primary();
            let context = FailureContext::capture(Arc::clone(&original), &path, 3, &mut control);
            assert!(Arc::ptr_eq(&original, &context.primary));
            assert_eq!(context.primary.code, "XTMM9000");
            assert_eq!(context.dependencies.len(), limit);
            if limit == 3 {
                assert_eq!(context.end, ContextEnd::Complete);
            } else {
                assert!(matches!(
                    context.end,
                    ContextEnd::Control(ControlFailure::BudgetExhausted { .. })
                ));
            }
        }
    }

    #[test]
    fn demand_context_cancellation_keeps_original_failure_and_releases_metadata() {
        let path = path();
        let weak = Arc::downgrade(&path[1].declaration);
        let original = primary();
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 1);
        let context = FailureContext::capture(Arc::clone(&original), &path, 3, &mut control);
        assert!(Arc::ptr_eq(&original, &context.primary));
        assert!(matches!(
            context.end,
            ContextEnd::Control(ControlFailure::Cancelled { .. })
        ));
        assert_eq!(context.dependencies.len(), 1);
        drop(path);
        assert!(weak.upgrade().is_none());
        assert_eq!(
            context.dependencies[0].declaration.resource,
            "urn:principal"
        );
    }

    #[test]
    fn demand_context_empty_path_needs_no_secondary_work() {
        let mut limits = WorkLimits::unbounded();
        limits.xdm_nodes = 0;
        let mut control = InvocationControl::new(CancellationToken::new(), limits);
        let context = FailureContext::capture(primary(), &[], 0, &mut control);
        assert!(context.dependencies.is_empty());
        assert_eq!(context.end, ContextEnd::Complete);
        assert_eq!(control.consumed(WorkDomain::XdmNode), 0);
    }

    #[test]
    fn demand_context_cached_attachment_never_grows_or_relabels_truncated_trace() {
        let path = path();
        let failure = CapturedFailure {
            primary: primary(),
            context: std::cell::OnceCell::new(),
        };
        let mut control = InvocationControl::unbounded();
        let first = failure.capture_once(&path, 1, &mut control);
        assert_eq!(first.end, ContextEnd::EntryLimit);
        assert_eq!(control.consumed(WorkDomain::XdmNode), 1);
        let second = failure.capture_once(&path[1..], 3, &mut control);
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(control.consumed(WorkDomain::XdmNode), 1);
        assert_eq!(second.dependencies, path[..1]);
        let mut exhausted = crate::execution_control_experiment::WorkLimits::unbounded();
        exhausted.xdm_nodes = 0;
        let mut control = InvocationControl::new(
            crate::execution_control_experiment::CancellationToken::new(),
            exhausted,
        );
        assert!(Arc::ptr_eq(
            &first,
            &failure.capture_once(&[], 0, &mut control)
        ));
        let weak = Arc::downgrade(&failure.primary);
        drop(failure);
        assert!(weak.upgrade().is_some());
        drop(first);
        drop(second);
        assert!(weak.upgrade().is_none());
    }
}
