//! Selected-policy integration over the existing sealed graph loader.
use super::*;
use crate::compile::golden_stylesheet_experiment::compile_selected_legacy_import_module_for_test;
use crate::compile::golden_stylesheet_experiment::compile_selected_legacy_include_module_for_test;
use crate::compile::golden_stylesheet_experiment::compile_selected_legacy_mixed_module_for_test;
use crate::compile::golden_stylesheet_experiment::compile_selected_legacy_resolved_chain_for_test;
use crate::compile::golden_stylesheet_experiment::compile_selected_legacy_resolved_pair_for_test;
use crate::execution_control_experiment::InvocationControl;
use crate::resources::{ResolutionLimits, ResourceLimits, ResourceSetBuilder};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;

#[path = "selected_group_measurement.rs"]
mod group_measurement;

#[path = "selected_document_tests.rs"]
mod document_tests;

const PRINCIPAL: &str = "https://example.invalid/styles/main.xsl";
const CHILD: &str = "https://example.invalid/shared/child.xsl";

fn load(
    snapshot: &crate::resources::ResourceSnapshot,
    identity: &str,
    denied: Vec<String>,
    attempts: usize,
) -> Result<LoadedStylesheetModule, DependencyFailure> {
    load_stylesheet_dependency_graph(
        &mut SnapshotResolver::new(snapshot, denied, ResolutionLimits::new(attempts)),
        identity,
        DependencyLimits::new(2, 3, 65_536),
        ParseLimits {
            max_events: 10_000,
            max_depth: 64,
        },
        None,
        None,
    )
}

fn compile(
    graph: &LoadedStylesheetModule,
) -> crate::xslt::golden_semantics_experiment::StylesheetProgram {
    let [child] = graph.dependencies.as_slice() else {
        panic!("one resolved dependency")
    };
    assert!(child.dependency_kind.is_some());
    assert!(child.dependencies.is_empty());
    assert_eq!(
        graph.document.parent(graph.root),
        Some(graph.document.document_node())
    );
    assert_eq!(
        child.document.parent(child.root),
        Some(child.document.document_node())
    );
    compile_selected_legacy_resolved_pair_for_test(
        &graph.document,
        &child.document,
        &mut InvocationControl::unbounded(),
    )
    .unwrap()
}

fn snapshot(with_child: bool) -> crate::resources::ResourceSnapshot {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(3, 16_384, 65_536));
    builder.admit(PRINCIPAL, br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:include href="../shared/child.xsl"/><xsl:output method="text"/><xsl:template match="/"><xsl:call-template name="child"/></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    if with_child {
        builder.admit(CHILD, br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template name="child"><xsl:sequence><xsl:fallback>selected</xsl:fallback></xsl:sequence></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    }
    builder.seal()
}

#[test]
fn selected_policy_resolved_include_retains_identity_and_lifetime() {
    let snapshot = snapshot(true);
    let graph = load(&snapshot, PRINCIPAL, vec![], 2).unwrap();
    assert_eq!(graph.dependencies[0].identity, CHILD);
    let program = compile(&graph);
    drop(graph);
    drop(snapshot);
    let source = Document::from_parsed(
        parse_document(
            "urn:source",
            b"<r/>",
            ParseLimits {
                max_events: 100,
                max_depth: 10,
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        execute_compiled_stylesheet_for_test(&program, &source, "resolved").unwrap(),
        "selected"
    );
}

#[test]
fn selected_policy_resolved_include_preserves_missing_denied_and_attempt_limits() {
    assert!(
        matches!(load(&snapshot(false), PRINCIPAL, vec![], 2), Err(DependencyFailure::Resolution { error: ResolutionFailure::Missing { identity }, location: Some(location) }) if identity == CHILD && location.resource == PRINCIPAL)
    );
    assert!(
        matches!(load(&snapshot(true), PRINCIPAL, vec![CHILD.to_owned()], 2), Err(DependencyFailure::Resolution { error: ResolutionFailure::Denied { identity }, .. }) if identity == CHILD)
    );
    assert!(matches!(
        load(&snapshot(true), PRINCIPAL, vec![], 1),
        Err(DependencyFailure::Resolution {
            error: ResolutionFailure::AttemptLimit { maximum: 1 },
            ..
        })
    ));
}

#[test]
fn selected_policy_resolved_compilation_charges_both_modules_and_keeps_them_immutable() {
    use crate::compile::golden_stylesheet_experiment::stylesheet_preprocessing::StrictPreprocessingFailure;
    use crate::execution_control_experiment::{
        CancellationToken, ControlFailure, WorkDomain, WorkLimits,
    };
    let snapshot = snapshot(true);
    let graph = load(&snapshot, PRINCIPAL, vec![], 2).unwrap();
    let child = &graph.dependencies[0];
    let principal_children = graph.document.children(graph.root).to_vec();
    let child_children = child.document.children(child.root).to_vec();
    let mut measured = InvocationControl::unbounded();
    compile_selected_legacy_resolved_pair_for_test(&graph.document, &child.document, &mut measured)
        .unwrap();
    let required = measured.consumed(WorkDomain::XdmNode);
    assert!(required > 1);
    let mut limits = WorkLimits::unbounded();
    limits.xdm_nodes = required;
    compile_selected_legacy_resolved_pair_for_test(
        &graph.document,
        &child.document,
        &mut InvocationControl::new(CancellationToken::new(), limits),
    )
    .unwrap();
    limits.xdm_nodes -= 1;
    assert!(matches!(
        compile_selected_legacy_resolved_pair_for_test(
            &graph.document,
            &child.document,
            &mut InvocationControl::new(CancellationToken::new(), limits)
        ),
        Err(StrictPreprocessingFailure::Control(
            ControlFailure::BudgetExhausted {
                domain: WorkDomain::XdmNode,
                ..
            }
        ))
    ));
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, required - 1);
    assert!(matches!(
        compile_selected_legacy_resolved_pair_for_test(
            &graph.document,
            &child.document,
            &mut cancelled
        ),
        Err(StrictPreprocessingFailure::Control(
            ControlFailure::Cancelled {
                domain: WorkDomain::XdmNode
            }
        ))
    ));
    assert_eq!(graph.document.children(graph.root), principal_children);
    assert_eq!(child.document.children(child.root), child_children);
    compile(&graph);
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_include_replays_impincl04() {
    replay("impincl04", "impincl-test/impincl04.xsl");
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_import_replays_impincl10() {
    replay("impincl10", "impincl/l.xsl");
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_import_preserves_mode_in_impincl15() {
    replay("impincl15", "impincl/s.xsl");
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_import_preserves_parameter_boundaries() {
    replay("impincl28", "impincl/fragments/impwparam.xsl");
    replay("impincl29", "impincl/fragments/impwparam.xsl");
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_chain_preserves_distinct_basenames_in_impincl19() {
    replay_chain(
        "impincl19",
        &[
            "impincl19.xsl",
            "fragments/frag-subdir/main_import.xsl",
            "fragments/main_import.xsl",
            "main_import.xsl",
        ],
    );
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_chain_preserves_builtin_import_dispatch_in_impincl25() {
    replay_chain(
        "impincl25",
        &[
            "impincl25.xsl",
            "fragments/imp25b.xsl",
            "fragments/imp21c.xsl",
        ],
    );
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_branching_imports_replay_impincl05() {
    replay_branches("impincl05", StylesheetDependencyKind::Import);
    replay_branches("impincl07", StylesheetDependencyKind::Import);
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_positioned_includes_replay_impincl06() {
    replay_branches("impincl06", StylesheetDependencyKind::Include);
    replay_branch_resources(
        "impincl12",
        StylesheetDependencyKind::Include,
        &["c.xsl", "e.xsl"],
    );
}

fn replay_branches(case: &str, kind: StylesheetDependencyKind) {
    replay_branch_resources(case, kind, &["b.xsl", "c.xsl", "d.xsl", "e.xsl"]);
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_import_subtrees_replay_impincl20_and_21() {
    for (case, paths) in [
        (
            "impincl20",
            [
                "fragments/imp20b.xsl",
                "fragments/imp20c.xsl",
                "fragments/imp20d.xsl",
            ],
        ),
        (
            "impincl21",
            [
                "fragments/imp21b.xsl",
                "fragments/imp21c.xsl",
                "fragments/imp21d.xsl",
            ],
        ),
    ] {
        replay_branch_resources(case, StylesheetDependencyKind::Import, &paths);
    }
    replay_branch_resources(
        "impincl22",
        StylesheetDependencyKind::Import,
        &["f.xsl", "g.xsl", "h.xsl"],
    );
    replay_branch_resources(
        "impincl14",
        StylesheetDependencyKind::Import,
        &["r.xsl", "c.xsl", "e.xsl"],
    );
}

fn load_branch_resources(
    case: &str,
    paths: &[&str],
) -> (std::path::PathBuf, LoadedStylesheetModule) {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let base = root.join("Xalan_Conformance_Tests/impincl");
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(5, 16_384, 65_536));
    for path in
        std::iter::once(format!("{case}.xsl")).chain(paths.iter().map(|path| (*path).to_owned()))
    {
        builder
            .admit(
                format!("https://oasis.invalid/impincl/{path}"),
                std::fs::read(base.join(path)).unwrap(),
            )
            .unwrap();
    }
    let snapshot = builder.seal();
    let graph = load_stylesheet_dependency_graph(
        &mut SnapshotResolver::new(&snapshot, vec![], ResolutionLimits::new(5)),
        &format!("https://oasis.invalid/impincl/{case}.xsl"),
        DependencyLimits::new(4, 5, 65_536),
        ParseLimits {
            max_events: 10_000,
            max_depth: 64,
        },
        None,
        None,
    )
    .unwrap();
    (root, graph)
}

fn replay_branch_resources(case: &str, kind: StylesheetDependencyKind, paths: &[&str]) {
    let (root, graph) = load_branch_resources(case, paths);
    let mut control = InvocationControl::unbounded();
    let program = match kind {
        StylesheetDependencyKind::Import => compile_import_graph(&graph, &mut control, true),
        StylesheetDependencyKind::Include => {
            let includes = graph
                .dependencies
                .iter()
                .map(|child| {
                    assert_eq!(
                        child.dependency_kind,
                        Some(StylesheetDependencyKind::Include)
                    );
                    compile_import_graph(child, &mut control, false)
                })
                .collect();
            compile_selected_legacy_include_module_for_test(&graph.document, includes, &mut control)
                .unwrap()
        }
    };
    drop(graph);
    assert_corpus_output(&program, &root, case);
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_mixed_import_include_replays_impincl23() {
    replay_mixed(
        "impincl23",
        &["h.xsl", "fragments/i23incl.xsl", "fragments/i23sub.xsl"],
    );
}

fn replay_mixed(case: &str, paths: &[&str]) {
    let (root, graph) = load_branch_resources(case, paths);
    let [import, include] = graph.dependencies.as_slice() else {
        panic!("two dependency edges")
    };
    assert_eq!(
        import.dependency_kind,
        Some(StylesheetDependencyKind::Import)
    );
    assert_eq!(
        include.dependency_kind,
        Some(StylesheetDependencyKind::Include)
    );
    let mut control = InvocationControl::unbounded();
    let imported = compile_import_graph(import, &mut control, false);
    let included = compile_import_graph(include, &mut control, false);
    let program = compile_selected_legacy_mixed_module_for_test(
        &graph.document,
        imported,
        included,
        &mut control,
    )
    .unwrap();
    drop(graph);
    assert_corpus_output(&program, &root, case);
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_impincl13_replays_computed_element_name() {
    replay_mixed(
        "impincl13",
        &["fragments/i13imp.xsl", "fragments/i13incl.xsl"],
    );
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_deeper_imports_replay_impincl26() {
    replay_branch_resources(
        "impincl26",
        StylesheetDependencyKind::Import,
        &[
            "fragments/imp26a.xsl",
            "fragments/imp26b.xsl",
            "fragments/imp26c.xsl",
            "fragments/imp26d.xsl",
        ],
    );
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_resolved_impincl03_admits_constant_leaf_parameter() {
    // Unchanged leaf dependency of impincl03; not a whole-case execution credit.
    let (_, graph) = load_branch_resources("p", &[]);
    let program = compile_selected_legacy_import_module_for_test(
        &graph.document,
        vec![],
        &mut InvocationControl::unbounded(),
        false,
    )
    .unwrap();
    assert_eq!(program.global_bindings.len(), 1);
    assert_eq!(program.global_bindings[0].name, "pParam");
    assert!(matches!(&program.global_bindings[0].default,
        crate::xslt::golden_semantics_experiment::GlobalBindingDefault::Text(value)
        if value == "!From p!"));
}

fn compile_import_graph(
    graph: &LoadedStylesheetModule,
    control: &mut InvocationControl,
    complete_graph: bool,
) -> crate::xslt::golden_semantics_experiment::StylesheetProgram {
    assert_eq!(
        graph.document.parent(graph.root),
        Some(graph.document.document_node())
    );
    let imports = graph
        .dependencies
        .iter()
        .map(|child| {
            assert_eq!(
                child.dependency_kind,
                Some(StylesheetDependencyKind::Import)
            );
            compile_import_graph(child, control, false)
        })
        .collect();
    compile_selected_legacy_import_module_for_test(
        &graph.document,
        imports,
        control,
        complete_graph,
    )
    .unwrap()
}

fn replay_chain(case: &str, paths: &[&str]) {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let base = root.join("Xalan_Conformance_Tests/impincl");
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(4, 16_384, 65_536));
    for path in paths {
        builder
            .admit(
                format!("https://oasis.invalid/impincl/{path}"),
                std::fs::read(base.join(path)).unwrap(),
            )
            .unwrap();
    }
    let snapshot = builder.seal();
    let graph = load_stylesheet_dependency_graph(
        &mut SnapshotResolver::new(&snapshot, vec![], ResolutionLimits::new(4)),
        &format!("https://oasis.invalid/impincl/{case}.xsl"),
        DependencyLimits::new(3, 4, 65_536),
        ParseLimits {
            max_events: 10_000,
            max_depth: 64,
        },
        None,
        None,
    )
    .unwrap();
    let mut current = &graph;
    let mut documents = Vec::new();
    for (index, path) in paths.iter().enumerate() {
        assert_eq!(
            current.identity,
            format!("https://oasis.invalid/impincl/{path}")
        );
        assert_eq!(
            current.document.parent(current.root),
            Some(current.document.document_node())
        );
        documents.push(&current.document);
        if index + 1 < paths.len() {
            let [child] = current.dependencies.as_slice() else {
                panic!("one chain edge")
            };
            assert_eq!(
                child.dependency_kind,
                Some(StylesheetDependencyKind::Import)
            );
            current = child;
        } else {
            assert!(current.dependencies.is_empty());
        }
    }
    let program = compile_selected_legacy_resolved_chain_for_test(
        &documents,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    drop(documents);
    drop(graph);
    drop(snapshot);
    assert_corpus_output(&program, &root, case);
}

fn assert_corpus_output(
    program: &crate::xslt::golden_semantics_experiment::StylesheetProgram,
    root: &std::path::Path,
    case: &str,
) {
    let base = root.join("Xalan_Conformance_Tests/impincl");
    let source = Document::from_parsed(
        parse_document(
            "urn:source",
            &std::fs::read(base.join(format!("{case}.xml"))).unwrap(),
            ParseLimits {
                max_events: 1000,
                max_depth: 64,
            },
        )
        .unwrap(),
    )
    .unwrap();
    let actual = execute_compiled_stylesheet_for_test(program, &source, case).unwrap();
    let expected = std::fs::read(root.join(format!(
        "Xalan_Conformance_Tests/REF_OUT/impincl/{case}.out"
    )))
    .unwrap();
    assert_eq!(
        crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
            &actual,
            &expected,
            false,
            None,
            Some("xml")
        ),
        Ok(true),
        "{actual}"
    );
}

fn replay(case: &str, child_path: &str) {
    let root = std::path::PathBuf::from(
        std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"),
    );
    let base = root.join("Xalan_Conformance_Tests");
    let principal = format!("https://oasis.invalid/impincl/{case}.xsl");
    let dependency = format!("https://oasis.invalid/{child_path}");
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(2, 16_384, 32_768));
    builder
        .admit(
            principal.clone(),
            std::fs::read(base.join(format!("impincl/{case}.xsl"))).unwrap(),
        )
        .unwrap();
    builder
        .admit(
            dependency.clone(),
            std::fs::read(base.join(child_path)).unwrap(),
        )
        .unwrap();
    let snapshot = builder.seal();
    let graph = load(&snapshot, &principal, vec![], 2).unwrap();
    assert_eq!(graph.dependencies[0].identity, dependency);
    let program = compile(&graph);
    let source = Document::from_parsed(
        parse_document(
            "urn:source",
            &std::fs::read(base.join(format!("impincl/{case}.xml"))).unwrap(),
            ParseLimits {
                max_events: 1000,
                max_depth: 64,
            },
        )
        .unwrap(),
    )
    .unwrap();
    let actual = execute_compiled_stylesheet_for_test(&program, &source, case).unwrap();
    let expected = std::fs::read(base.join(format!("REF_OUT/impincl/{case}.out"))).unwrap();
    assert_eq!(
        crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
            &actual,
            &expected,
            false,
            None,
            Some("xml")
        ),
        Ok(true),
        "{actual}"
    );
}
