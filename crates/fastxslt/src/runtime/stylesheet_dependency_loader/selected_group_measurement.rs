//! Conserved private-profile replay, not production profile promotion.
use super::*;
use crate::compile::golden_stylesheet_experiment::stylesheet_preprocessing::StrictPreprocessingFailure;
use crate::resources::ResourceSnapshot;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_measures_complete_impincl_group() {
    let root = PathBuf::from(std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"))
        .join("Xalan_Conformance_Tests");
    let snapshot = import_resources(&root, true);
    let mut counts = BTreeMap::new();
    println!("profile\tselected-legacy\tstrict-xslt10\tprivate");
    for index in 1..=29 {
        let case = format!("impincl{index:02}");
        let (status, detail) = match replay(&root, &snapshot, &case) {
            Ok(true) => ("exact", String::new()),
            Ok(false) => ("mismatch", String::new()),
            Err((status, detail)) => (status, detail),
        };
        *counts.entry(status).or_insert(0usize) += 1;
        println!("case\t{case}\t{status}\t{detail}");
        assert_eq!(
            status, "exact",
            "{case}: classification changed; review before updating the ledger: {detail}"
        );
    }
    assert_eq!(counts.values().sum::<usize>(), 29);
    for (status, count) in &counts {
        println!("counter\t{status}\t{count}");
    }
}

fn import_resources(root: &Path, historical_aliases: bool) -> ResourceSnapshot {
    import_group_resources(root, &["impincl", "impincl-test"], historical_aliases)
}

fn import_group_resources(
    root: &Path,
    groups: &[&str],
    historical_aliases: bool,
) -> ResourceSnapshot {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(256, 1_048_576, 16_777_216));
    let mut originals = BTreeMap::new();
    let mut pending: Vec<_> = groups.iter().map(|group| root.join(group)).collect();
    while let Some(folder) = pending.pop() {
        for entry in std::fs::read_dir(folder).unwrap() {
            let entry = entry.unwrap();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext == "xsl" || ext == "xml")
            {
                let path = entry.path();
                assert!(
                    entry.metadata().unwrap().len() <= 1_048_576,
                    "test import exceeds per-resource ceiling"
                );
                let logical = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/");
                let bytes = std::fs::read(path).unwrap();
                builder
                    .admit(format!("https://oasis.invalid/{logical}"), bytes.clone())
                    .unwrap();
                originals.insert(logical, bytes);
            }
        }
    }
    if historical_aliases {
        admit_historical_aliases(root, &originals, &mut builder);
    }
    builder.seal()
}

fn admit_historical_aliases(
    root: &Path,
    originals: &BTreeMap<String, Vec<u8>>,
    builder: &mut ResourceSetBuilder,
) {
    use crate::runtime::oasis_xslt10_measurement_tests::{
        resolve_case_dependency_file, stylesheet_dependency_references,
    };
    let mut aliases = BTreeMap::new();
    for (physical, bytes) in originals {
        for reference in stylesheet_dependency_references(physical, bytes) {
            // Explicit acquired-corpus import policy, not engine URI acquisition.
            // Only already loaded resources can be aliased; no new file reads.
            if !reference.starts_with("file:") {
                continue;
            }
            let Some(target) = resolve_case_dependency_file(root, physical, &reference, true)
            else {
                continue;
            };
            let Some(bytes) = originals.get(&target) else {
                continue;
            };
            if let Some(prior) = aliases.insert(reference.clone(), bytes) {
                assert_eq!(prior, bytes, "ambiguous historical alias: {reference}");
            }
        }
    }
    for (identity, bytes) in aliases {
        builder.admit(identity, bytes.clone()).unwrap();
    }
}

type Failure = (&'static str, String);

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_measures_complete_whitespace_group() {
    let root = PathBuf::from(std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"))
        .join("Xalan_Conformance_Tests");
    let snapshot = import_group_resources(&root, &["whitespace"], false);
    let indices: Vec<_> = (1..=13).chain([15]).chain(17..=23).chain([35]).collect();
    assert_eq!(
        catalog_group_ids(root.parent().unwrap(), "whitespace_"),
        indices
            .iter()
            .map(|index| format!("whitespace_whitespace{index:02}"))
            .collect(),
        "reviewed whitespace catalog membership changed"
    );
    let mut counts = BTreeMap::new();
    let mut changed = Vec::new();
    println!("profile\tselected-legacy\tstrict-xslt10\tprivate\twhitespace-sensitive");
    for index in indices {
        let case = format!("whitespace{index:02}");
        let (status, detail) = match replay_group(&root, &snapshot, "whitespace", &case, true) {
            Ok(true) => ("exact", String::new()),
            Ok(false) => ("mismatch", String::new()),
            Err((status, detail)) => (status, detail),
        };
        *counts.entry(status).or_insert(0usize) += 1;
        println!("case\t{case}\t{status}\t{detail}");
        let expected = match index {
            17 | 21 => "mismatch",
            23 | 35 => "selected-unsupported",
            _ => "exact",
        };
        if status != expected {
            changed.push(format!(
                "{case}: expected {expected}, got {status}: {detail}"
            ));
        }
    }
    assert_eq!(counts.values().sum::<usize>(), 22);
    for (status, count) in &counts {
        println!("counter\t{status}\t{count}");
    }
    assert!(
        changed.is_empty(),
        "review classification changes: {changed:?}"
    );
}

fn catalog_group_ids(root: &Path, prefix: &str) -> std::collections::BTreeSet<String> {
    let catalog = Document::from_parsed(
        parse_document(
            "urn:oasis-catalog",
            &std::fs::read(root.join("catalog.xml")).unwrap(),
            ParseLimits {
                max_events: 200_000,
                max_depth: 32,
            },
        )
        .unwrap(),
    )
    .unwrap();
    let mut pending = vec![catalog.document_node()];
    let mut ids = std::collections::BTreeSet::new();
    while let Some(node) = pending.pop() {
        if catalog
            .name(node)
            .is_some_and(|name| name.namespace.is_none() && name.local == "test-case")
        {
            for attribute in catalog.attributes(node) {
                if catalog
                    .name(*attribute)
                    .is_some_and(|name| name.namespace.is_none() && name.local == "id")
                    && let Some(id) = catalog
                        .value(*attribute)
                        .filter(|id| id.starts_with(prefix))
                {
                    assert!(ids.insert(id.to_owned()), "duplicate catalog case: {id}");
                }
            }
        }
        pending.extend_from_slice(catalog.children(node));
    }
    ids
}

#[test]
#[ignore = "requires separately acquired immutable OASIS corpus"]
fn selected_policy_historical_aliases_require_explicit_host_admission() {
    let root = PathBuf::from(std::env::var("FASTXSLT_OASIS_XSLT10_ROOT").expect("set corpus root"))
        .join("Xalan_Conformance_Tests");
    let snapshot = import_resources(&root, false);
    let (status, detail) = replay(&root, &snapshot, "impincl27").unwrap_err();
    assert_eq!(status, "unadmitted-resource");
    assert!(detail.contains("file:fragments/imp27b.xsl"));
    let snapshot = import_resources(&root, true);
    assert!(replay(&root, &snapshot, "impincl27").unwrap());
}

fn replay(root: &Path, snapshot: &ResourceSnapshot, case: &str) -> Result<bool, Failure> {
    replay_group(root, snapshot, "impincl", case, false)
}

fn replay_group(
    root: &Path,
    snapshot: &ResourceSnapshot,
    group: &str,
    case: &str,
    preserve_whitespace: bool,
) -> Result<bool, Failure> {
    let graph = load_stylesheet_dependency_graph(
        &mut SnapshotResolver::new(snapshot, vec![], ResolutionLimits::new(32)),
        &format!("https://oasis.invalid/{group}/{case}.xsl"),
        DependencyLimits::new(8, 32, 1_048_576),
        ParseLimits {
            max_events: 100_000,
            max_depth: 128,
        },
        None,
        None,
    )
    .map_err(|error| {
        let status = if matches!(
            &error,
            DependencyFailure::Resolution {
                error: ResolutionFailure::Missing { .. },
                ..
            }
        ) {
            "unadmitted-resource"
        } else {
            "dependency-failure"
        };
        (status, format!("{error:?}"))
    })?;
    let program = compile_graph(&graph, &mut InvocationControl::unbounded(), true)?;
    let source = Document::from_parsed(
        parse_document(
            &format!("https://oasis.invalid/{group}/{case}.xml"),
            &std::fs::read(root.join(format!("{group}/{case}.xml"))).unwrap(),
            ParseLimits {
                max_events: 100_000,
                max_depth: 128,
            },
        )
        .map_err(|error| ("source-parse-failure", format!("{error:?}")))?,
    )
    .map_err(|error| ("source-xdm-failure", format!("{error:?}")))?;
    let mut control = InvocationControl::unbounded();
    let result = crate::runtime::golden_runtime_experiment::execute_program_with_resources(
        &program,
        &source,
        snapshot,
        None,
        case,
        &mut control,
    )
    .map_err(|error| ("execution-failure", format!("{error:?}")))?;
    let actual = crate::runtime::golden_runtime_experiment::serialize_xml(
        &result,
        &program.output,
        case,
        65_536,
        &mut control,
    )
    .map_err(|error| ("serialization-failure", format!("{error:?}")))?;
    let expected = std::fs::read(root.join(format!("REF_OUT/{group}/{case}.out"))).unwrap();
    crate::runtime::oasis_xslt10_measurement_tests::xml_equivalent(
        &actual,
        &expected,
        preserve_whitespace,
        None,
        Some("xml"),
    )
    .map_err(|error| ("comparator-limitation", error))
}

fn compile_graph(
    graph: &LoadedStylesheetModule,
    control: &mut InvocationControl,
    complete: bool,
) -> Result<StylesheetProgram, Failure> {
    if graph.document.parent(graph.root) != Some(graph.document.document_node()) {
        return Err((
            "harness-limitation",
            "embedded stylesheet root not integrated".into(),
        ));
    }
    if complete
        && let [child] = graph.dependencies.as_slice()
        && child.dependencies.is_empty()
    {
        return compile_selected_legacy_resolved_pair(&graph.document, &child.document, control)
            .map_err(classify_compile_failure);
    }
    let mut children = graph
        .dependencies
        .iter()
        .map(|child| compile_graph(child, control, false))
        .collect::<Result<Vec<_>, _>>()?;
    let kinds: Vec<_> = graph
        .dependencies
        .iter()
        .map(|child| child.dependency_kind)
        .collect();
    let result = if kinds
        .iter()
        .all(|kind| *kind == Some(StylesheetDependencyKind::Import))
    {
        compile_selected_legacy_import_module(&graph.document, children, control, complete)
    } else if kinds
        .iter()
        .all(|kind| *kind == Some(StylesheetDependencyKind::Include))
    {
        compile_selected_legacy_include_module(&graph.document, children, control)
    } else if kinds
        == [
            Some(StylesheetDependencyKind::Import),
            Some(StylesheetDependencyKind::Include),
        ]
    {
        let included = children.pop().unwrap();
        let imported = children.pop().unwrap();
        compile_selected_legacy_mixed_module(&graph.document, imported, included, control)
    } else {
        return Err((
            "harness-limitation",
            "dependency composition shape not integrated".into(),
        ));
    };
    result.map_err(classify_compile_failure)
}

fn classify_compile_failure(error: StrictPreprocessingFailure) -> Failure {
    match error {
        StrictPreprocessingFailure::Compile(error) => {
            let status = match error.category {
                crate::compile::golden_stylesheet_experiment::CompileCategory::Unsupported => {
                    "selected-unsupported"
                }
                crate::compile::golden_stylesheet_experiment::CompileCategory::Invalid => {
                    "compile-invalid"
                }
            };
            (status, format!("{error:?}"))
        }
        StrictPreprocessingFailure::Control(error) => ("control-failure", format!("{error:?}")),
    }
}
