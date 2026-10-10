//! AR-0028 resolver and existing literal-document frame ownership controls.
//! Does not admit retained ordinary-node variables or compiled focus switching.

use std::{cell::RefCell, collections::BTreeMap, sync::Arc};

use super::super::runtime_context::{RuntimeGlobals, RuntimeVariables};
use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder, ResourceSnapshot};
use crate::xdm::retained_nodes_reference::retain;

const BASE: &str = "https://example.test/owners/style.xsl";
const FIRST: &str = "https://example.test/owners/a.xml";

fn snapshot() -> ResourceSnapshot {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(3, 128, 384));
    builder
        .admit(FIRST, b"<r><a>first</a></r>".to_vec())
        .unwrap();
    builder
        .admit(
            "https://example.test/owners/b.xml",
            b"<r><a>second</a></r>".to_vec(),
        )
        .unwrap();
    builder
        .admit("https://example.test/owners/broken.xml", b"<r>".to_vec())
        .unwrap();
    builder.seal()
}

fn reference(name: &str) -> DocumentRootReference {
    DocumentRootReference {
        base: BASE.to_owned(),
        reference: name.to_owned(),
        descendant_name: None,
    }
}

fn root(document: &Document) -> NodeId {
    document.children(document.document_node())[0]
}

#[test]
fn sealed_resolution_reuses_owner_but_other_invocation_and_generation_do_not() {
    let snapshot = snapshot();
    let cache = RefCell::new(BTreeMap::new());
    let mut control = InvocationControl::unbounded();
    let first = prepare_document_from_snapshot(
        Some(&snapshot),
        None,
        &cache,
        "owners",
        &reference("a.xml"),
        &mut control,
    )
    .unwrap();
    let again = prepare_document_from_snapshot(
        Some(&snapshot),
        None,
        &cache,
        "owners",
        &reference("./a.xml"),
        &mut control,
    )
    .unwrap();
    let second = prepare_document_from_snapshot(
        Some(&snapshot),
        None,
        &cache,
        "owners",
        &reference("b.xml"),
        &mut control,
    )
    .unwrap();
    let id = root(&first.document);
    assert_eq!(id, root(&second.document));
    let kept = retain(&first.document, &[id], &mut control).unwrap();
    let alias = retain(&again.document, &[id], &mut control).unwrap();
    let foreign = retain(&second.document, &[id], &mut control).unwrap();
    assert!(kept[0].same_node(&alias[0]));
    assert!(!kept[0].same_node(&foreign[0]));
    assert_eq!(first.identity, again.identity);
    assert_eq!(cache.borrow().len(), 2);
    for other_snapshot in [&snapshot, &self::snapshot()] {
        let other_cache = RefCell::new(BTreeMap::new());
        let other = prepare_document_from_snapshot(
            Some(other_snapshot),
            None,
            &other_cache,
            "other",
            &reference("a.xml"),
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let other_nodes =
            retain(&other.document, &[id], &mut InvocationControl::unbounded()).unwrap();
        // Independently allocated invocation scalar IDs can coincide. They are
        // not an identity oracle across invocations.
        assert_eq!(first.identity, other.identity);
        assert!(!kept[0].same_node(&other_nodes[0]));
    }
    let weak = Arc::downgrade(&first.document);
    drop(first);
    drop(again);
    drop(cache);
    assert!(weak.upgrade().is_some());
    drop(kept);
    drop(alias);
    assert!(weak.upgrade().is_none());
}

#[test]
fn existing_document_frames_preserve_owners_under_cow_alias_and_cross_kind_shadowing() {
    let snapshot = snapshot();
    let cache = RefCell::new(BTreeMap::new());
    let mut control = InvocationControl::unbounded();
    let first = prepare_document_from_snapshot(
        Some(&snapshot),
        None,
        &cache,
        "frames",
        &reference("a.xml"),
        &mut control,
    )
    .unwrap();
    let second = prepare_document_from_snapshot(
        Some(&snapshot),
        None,
        &cache,
        "frames",
        &reference("b.xml"),
        &mut control,
    )
    .unwrap();
    let mut globals = RuntimeGlobals::default();
    globals
        .literal_documents
        .insert("global".to_owned(), first.clone());
    let mut parent = RuntimeVariables::default();
    parent.bind_literal_document("local".to_owned(), first.clone());
    for complete in [false, true] {
        let mut child = if complete {
            parent.clone_complete_for_sequence()
        } else {
            parent.clone()
        };
        assert!(child.bind_alias("alias".to_owned(), "local", &globals));
        let original_id = root(&first.document);
        let original = retain(&first.document, &[original_id], &mut control).unwrap();
        child.bind_literal_document("local".to_owned(), second.clone());
        child.bind_source_nodes("global".to_owned(), vec![original_id]);
        assert!(child.literal_document(&globals, "global").is_none());
        assert!(parent.literal_document(&globals, "global").is_some());
        let owner = child.literal_document(&globals, "alias").unwrap();
        let value = retain(&owner.document, &[original_id], &mut control).unwrap();
        assert!(original[0].same_node(&value[0]));
        let replaced = child.literal_document(&globals, "local").unwrap();
        let replaced = retain(&replaced.document, &[original_id], &mut control).unwrap();
        assert!(!original[0].same_node(&replaced[0]));
        assert!(
            parent
                .literal_document(&globals, "local")
                .unwrap()
                .document
                .same_origin(&first.document)
        );
    }
}

#[test]
fn failed_resolution_and_preparation_never_install_partial_owners_or_bypass_denial() {
    let snapshot = snapshot();
    let cache = RefCell::new(BTreeMap::new());
    let mut control = InvocationControl::unbounded();
    let healthy = prepare_document_from_snapshot(
        Some(&snapshot),
        None,
        &cache,
        "failures",
        &reference("a.xml"),
        &mut control,
    )
    .unwrap();
    let denied = HashSet::from([FIRST.to_owned()]);
    let failure = prepare_document_from_snapshot(
        Some(&snapshot),
        Some(&denied),
        &cache,
        "failures",
        &reference("a.xml"),
        &mut control,
    )
    .unwrap_err();
    assert_eq!(
        (failure.code, failure.category),
        ("FXRS0003", FailureCategory::Denied)
    );
    for (name, code, category) in [
        ("missing.xml", "FXRS0001", FailureCategory::MissingResource),
        ("broken.xml", "FXXD0002", FailureCategory::Invalid),
    ] {
        let failure = prepare_document_from_snapshot(
            Some(&snapshot),
            None,
            &cache,
            "failures",
            &reference(name),
            &mut control,
        )
        .unwrap_err();
        assert_eq!((failure.code, failure.category), (code, category));
        assert_eq!(failure.request_id.as_deref(), Some("failures"));
        assert_eq!(cache.borrow().len(), 1);
    }
    let cancel = CancellationToken::new();
    cancel.cancel();
    let mut cancelled = InvocationControl::new(cancel, WorkLimits::unbounded());
    let failure = prepare_document_from_snapshot(
        Some(&snapshot),
        None,
        &cache,
        "failures",
        &reference("b.xml"),
        &mut cancelled,
    )
    .unwrap_err();
    assert_eq!(
        (failure.code, failure.category),
        ("FXCT0001", FailureCategory::Cancelled)
    );
    assert_eq!(cache.borrow().len(), 1);
    let recovered = prepare_document_from_snapshot(
        Some(&snapshot),
        None,
        &cache,
        "failures",
        &reference("a.xml"),
        &mut control,
    )
    .unwrap();
    assert!(healthy.document.same_origin(&recovered.document));
}
