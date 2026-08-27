//! Structured runtime failures and execution-control translation.

use crate::execution_control_experiment::{ControlFailure, WorkDomain};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FailureCategory {
    Invalid,
    Unsupported,
    MissingResource,
    #[cfg(test)]
    Denied,
    Limit,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::runtime) struct ExecutionFailure {
    pub(super) code: &'static str,
    pub(super) category: FailureCategory,
    pub(super) request_id: Option<String>,
    pub(super) work_domain: Option<WorkDomain>,
    pub(super) detail: String,
}

#[cfg(feature = "workbench")]
impl ExecutionFailure {
    pub(in crate::runtime) fn workbench_parts(
        &self,
    ) -> (&'static str, &'static str, Option<&str>, &str) {
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

pub(super) fn failure(
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

pub(super) fn control_failure(failure: ControlFailure, request_id: &str) -> ExecutionFailure {
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
