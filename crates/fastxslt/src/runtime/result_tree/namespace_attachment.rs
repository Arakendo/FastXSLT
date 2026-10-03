//! Ordered, charged attachment of result-owned copied namespace items.

use std::sync::Arc;

use super::{
    ExecutionFailure, ExpandedName, FailureCategory, InvocationControl, NamespaceBinding,
    WorkDomain, control_failure, failure_at,
};
use crate::xdm::owned_tree_experiment::SourceLocation;

pub(in crate::runtime) fn attach_namespaces(
    name: &ExpandedName,
    namespaces: &mut Arc<[NamespaceBinding]>,
    binding: NamespaceBinding,
    location: &SourceLocation,
    has_child: bool,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<(), ExecutionFailure> {
    let reject = |code, detail| {
        failure_at(
            code,
            FailureCategory::Invalid,
            Some(request_id),
            location.clone(),
            detail,
        )
    };
    control
        .charge(WorkDomain::ResultNode, 1)
        .map_err(|error| control_failure(error, request_id))?;
    if has_child {
        return Err(reject(
            "XTDE0410",
            "namespace construction follows a result child",
        ));
    }
    if binding.prefix.is_none() && name.namespace.is_none() {
        return Err(reject(
            "XTDE0440",
            "a default namespace cannot attach to an unnamespaced element",
        ));
    }
    // The mandatory xml binding is implicit, never serialized as a new declaration.
    if binding.prefix.as_deref() == Some("xml") {
        return Ok(());
    }
    for existing in namespaces.iter() {
        control
            .charge(WorkDomain::ResultNode, 1)
            .map_err(|error| control_failure(error, request_id))?;
        if existing.prefix == binding.prefix {
            if existing.namespace != binding.namespace {
                return Err(reject(
                    "XTDE0430",
                    "copied namespace conflicts with a result binding",
                ));
            }
            return Ok(());
        }
    }
    // Rebuilding the immutable slice copies its existing payload. Charge that
    // physical work before cloning; the pending binding itself is moved.
    control
        .charge(WorkDomain::ResultNode, namespaces.len() + 1)
        .map_err(|error| control_failure(error, request_id))?;
    for existing in namespaces.iter() {
        control
            .charge(
                WorkDomain::ResultTextByte,
                existing.prefix.as_ref().map_or(0, String::len) + existing.namespace.len(),
            )
            .map_err(|error| control_failure(error, request_id))?;
    }
    let mut retained = Vec::with_capacity(namespaces.len() + 1);
    retained.extend_from_slice(namespaces);
    retained.push(binding);
    *namespaces = retained.into();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};

    #[test]
    fn attachment_rebuild_charges_before_allocation_and_preserves_shared_slice_on_failure() {
        let original: Arc<[NamespaceBinding]> = vec![NamespaceBinding {
            prefix: Some("p".to_owned()),
            namespace: "urn:p".to_owned(),
        }]
        .into();
        let name = ExpandedName {
            namespace: None,
            local: "out".to_owned(),
        };
        let location = SourceLocation {
            resource: "urn:copy:style".to_owned(),
            span: 10..20,
        };
        for (nodes, bytes, expected) in [
            (4, 6, None),
            (3, 6, Some(WorkDomain::ResultNode)),
            (4, 5, Some(WorkDomain::ResultTextByte)),
        ] {
            let mut limits = WorkLimits::unbounded();
            limits.result_nodes = nodes;
            limits.result_text_bytes = bytes;
            let mut retained = original.clone();
            let result = attach_namespaces(
                &name,
                &mut retained,
                NamespaceBinding {
                    prefix: Some("q".to_owned()),
                    namespace: "urn:q".to_owned(),
                },
                &location,
                false,
                "copy",
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            if let Some(domain) = expected {
                let error = result.unwrap_err();
                assert_eq!(error.code, "FXCT0002");
                assert_eq!(error.work_domain, Some(domain));
                assert!(Arc::ptr_eq(&retained, &original));
            } else {
                result.unwrap();
                assert_eq!(retained.len(), 2);
                assert!(!Arc::ptr_eq(&retained, &original));
            }
            assert_eq!(original.len(), 1);
        }
        let mut limits = WorkLimits::unbounded();
        limits.result_nodes = 2;
        limits.result_text_bytes = 0;
        let mut retained = original.clone();
        attach_namespaces(
            &name,
            &mut retained,
            original[0].clone(),
            &location,
            false,
            "copy",
            &mut InvocationControl::new(CancellationToken::new(), limits),
        )
        .unwrap();
        assert!(Arc::ptr_eq(&retained, &original));
    }
}
