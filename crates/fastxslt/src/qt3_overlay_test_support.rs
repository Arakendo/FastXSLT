use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::PathBuf,
    sync::OnceLock,
};

use serde::Deserialize;

use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const EXPECTED_SUITE: &str = "QT3";
const EXPECTED_REVISION: &str = "83993587711dbd5c18ed846385ec37d079d6e492";
const PRIVATE_LEDGER_SOURCE: &str =
    include_str!("../../../corpus/overlays/qt3/private-ledger-v0.toml");
const AXIS_DENOMINATOR_SOURCE: &str =
    include_str!("../../../corpus/overlays/qt3/axis-step-denominator-v0.toml");
const DEEP_EQUAL_DENOMINATOR_SOURCE: &str =
    include_str!("../../../corpus/overlays/qt3/deep-equal-denominator-v0.toml");
const TRUE_DENOMINATOR_SOURCE: &str =
    include_str!("../../../corpus/overlays/qt3/true-denominator-v0.toml");
const FALSE_DENOMINATOR_SOURCE: &str =
    include_str!("../../../corpus/overlays/qt3/false-denominator-v0.toml");

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum SelectionDisposition {
    Selected,
    HarnessUnsupported,
    ProfileExcluded,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum ExecutionDisposition {
    Passed,
    NotRun,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CaseRecord {
    set_file: String,
    case_name: String,
    selection: SelectionDisposition,
    execution: ExecutionDisposition,
    rationale: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateLedger {
    suite: String,
    revision: String,
    case: Vec<CaseRecord>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DefaultDisposition {
    selection: SelectionDisposition,
    execution: ExecutionDisposition,
    rationale: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DependencyRule {
    dependency_type: String,
    values: BTreeSet<String>,
    selection: SelectionDisposition,
    execution: ExecutionDisposition,
    rationale: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DenominatorOverlay {
    suite: String,
    revision: String,
    set_file: String,
    case_count: usize,
    selected_source: String,
    #[serde(default)]
    dependency_rule: Vec<DependencyRule>,
    default_disposition: DefaultDisposition,
}

fn validate_header(suite: &str, revision: &str) -> Result<(), String> {
    if suite != EXPECTED_SUITE {
        return Err(format!("unexpected suite identity {suite:?}"));
    }
    if revision != EXPECTED_REVISION {
        return Err(format!("unexpected suite revision {revision:?}"));
    }
    Ok(())
}

fn validate_text(value: &str, field: &str, identity: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{identity} has an empty {field}"))
    } else {
        Ok(())
    }
}

fn parse_private_ledger(source: &str) -> Result<PrivateLedger, String> {
    let ledger: PrivateLedger = toml::from_str(source).map_err(|error| error.to_string())?;
    validate_header(&ledger.suite, &ledger.revision)?;
    let mut identities = BTreeSet::new();
    for case in &ledger.case {
        let identity = format!("{}::{}", case.set_file, case.case_name);
        validate_text(&case.set_file, "set_file", &identity)?;
        validate_text(&case.case_name, "case_name", &identity)?;
        validate_text(&case.rationale, "rationale", &identity)?;
        if case.selection != SelectionDisposition::Selected
            || case.execution != ExecutionDisposition::Passed
        {
            return Err(format!("{identity} is not selected/passed"));
        }
        if !identities.insert((case.set_file.as_str(), case.case_name.as_str())) {
            return Err(format!("duplicate case identity {identity}"));
        }
    }
    Ok(ledger)
}

fn parse_denominator(source: &str) -> Result<DenominatorOverlay, String> {
    let overlay: DenominatorOverlay = toml::from_str(source).map_err(|error| error.to_string())?;
    validate_header(&overlay.suite, &overlay.revision)?;
    validate_text(&overlay.set_file, "set_file", "QT3 denominator")?;
    validate_text(
        &overlay.default_disposition.rationale,
        "default rationale",
        &overlay.set_file,
    )?;
    if overlay.selected_source != "private-ledger-v0.toml" {
        return Err(format!(
            "{} references unexpected selected source {:?}",
            overlay.set_file, overlay.selected_source
        ));
    }
    if overlay.default_disposition.selection != SelectionDisposition::HarnessUnsupported
        || overlay.default_disposition.execution != ExecutionDisposition::NotRun
    {
        return Err(format!(
            "{} default must remain harness-unsupported/not-run",
            overlay.set_file
        ));
    }
    for rule in &overlay.dependency_rule {
        validate_text(&rule.dependency_type, "dependency type", &overlay.set_file)?;
        validate_text(&rule.rationale, "rule rationale", &overlay.set_file)?;
        if rule.values.is_empty() || rule.values.iter().any(|value| value.trim().is_empty()) {
            return Err(format!(
                "{} has an empty dependency rule value set",
                overlay.set_file
            ));
        }
        if rule.selection != SelectionDisposition::ProfileExcluded
            || rule.execution != ExecutionDisposition::NotRun
        {
            return Err(format!(
                "{} dependency rules must remain profile-excluded/not-run",
                overlay.set_file
            ));
        }
    }
    Ok(overlay)
}

fn private_ledger() -> &'static PrivateLedger {
    static LEDGER: OnceLock<PrivateLedger> = OnceLock::new();
    LEDGER.get_or_init(|| {
        parse_private_ledger(PRIVATE_LEDGER_SOURCE)
            .unwrap_or_else(|error| panic!("invalid private QT3 ledger: {error}"))
    })
}

pub(crate) fn assert_private_case_passed(set_file: &str, case_name: &str) {
    let case = private_ledger()
        .case
        .iter()
        .find(|case| case.set_file == set_file && case.case_name == case_name)
        .unwrap_or_else(|| panic!("missing private QT3 case {set_file}::{case_name}"));
    assert_eq!(
        case.selection,
        SelectionDisposition::Selected,
        "{case_name}"
    );
    assert_eq!(case.execution, ExecutionDisposition::Passed, "{case_name}");
}

pub(crate) fn assert_selected_count(set_file: &str, expected: usize) {
    assert_eq!(
        private_ledger()
            .case
            .iter()
            .filter(|case| case.set_file == set_file)
            .count(),
        expected,
        "{set_file} selected count"
    );
}

fn catalog_cases(set_file: &str) -> BTreeMap<String, BTreeSet<(String, String)>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/qt3tests")
        .join(set_file);
    let bytes = fs::read(path).expect("read pinned QT3 denominator");
    let identity = format!("urn:w3c:qt3:{set_file}");
    let parsed = parse_document(
        &identity,
        &bytes,
        ParseLimits {
            max_events: 100_000,
            max_depth: 64,
        },
    )
    .expect("parse pinned QT3 denominator");
    let document = Document::from_parsed(parsed).expect("build pinned QT3 denominator");
    descendants_named(&document, document.document_node(), "test-case")
        .into_iter()
        .map(|case| {
            let name = attribute(&document, case, "name")
                .expect("QT3 case name")
                .to_owned();
            let dependencies = children_named(&document, case, "dependency")
                .into_iter()
                .map(|dependency| {
                    (
                        attribute(&document, dependency, "type")
                            .expect("QT3 dependency type")
                            .to_owned(),
                        attribute(&document, dependency, "value")
                            .expect("QT3 dependency value")
                            .to_owned(),
                    )
                })
                .collect();
            (name, dependencies)
        })
        .collect()
}

fn children_named(document: &Document, parent: NodeId, local: &str) -> Vec<NodeId> {
    document
        .children(parent)
        .iter()
        .copied()
        .filter(|child| {
            document.kind(*child) == NodeKind::Element
                && document
                    .name(*child)
                    .is_some_and(|name| name.local == local)
        })
        .collect()
}

fn descendants_named(document: &Document, parent: NodeId, local: &str) -> Vec<NodeId> {
    let mut found = Vec::new();
    for child in document.children(parent).iter().copied() {
        if document.kind(child) != NodeKind::Element {
            continue;
        }
        if document.name(child).is_some_and(|name| name.local == local) {
            found.push(child);
        }
        found.extend(descendants_named(document, child, local));
    }
    found
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        document
            .name(*attribute)
            .filter(|name| name.namespace.is_none() && name.local == local)
            .and_then(|_| document.value(*attribute))
    })
}

fn assert_complete_denominator(
    source: &str,
    expected_set: &str,
    expected_count: usize,
    expected_passed: usize,
    expected_profile_excluded: usize,
) {
    let overlay = parse_denominator(source).expect("valid QT3 denominator overlay");
    assert_eq!(overlay.set_file, expected_set);
    assert_eq!(overlay.case_count, expected_count);
    let catalog_cases = catalog_cases(expected_set);
    assert_eq!(catalog_cases.len(), expected_count);
    let selected = private_ledger()
        .case
        .iter()
        .filter(|case| case.set_file == expected_set)
        .collect::<Vec<_>>();
    assert_eq!(selected.len(), expected_passed);
    for case in &selected {
        assert!(
            catalog_cases.contains_key(&case.case_name),
            "{}::{} is absent upstream",
            expected_set,
            case.case_name
        );
    }
    let selected_names = selected
        .iter()
        .map(|case| case.case_name.as_str())
        .collect::<BTreeSet<_>>();
    let profile_excluded = catalog_cases
        .iter()
        .filter(|(name, dependencies)| {
            !selected_names.contains(name.as_str())
                && overlay.dependency_rule.iter().any(|rule| {
                    dependencies.iter().any(|(dependency_type, value)| {
                        dependency_type == &rule.dependency_type && rule.values.contains(value)
                    })
                })
        })
        .count();
    assert_eq!(profile_excluded, expected_profile_excluded);
    assert_eq!(
        expected_passed
            + expected_profile_excluded
            + (expected_count - expected_passed - expected_profile_excluded),
        expected_count
    );
}

#[test]
fn qt3_denominator_overlays_conserve_their_parent_sets() {
    assert_eq!(private_ledger().case.len(), 456);
    assert!(private_ledger().case.iter().all(|case| matches!(
        case.set_file.as_str(),
        "prod/AxisStep.xml" | "fn/deep-equal.xml" | "fn/true.xml" | "fn/false.xml"
    )));
    assert_complete_denominator(AXIS_DENOMINATOR_SOURCE, "prod/AxisStep.xml", 349, 224, 112);
    assert_complete_denominator(
        DEEP_EQUAL_DENOMINATOR_SOURCE,
        "fn/deep-equal.xml",
        263,
        184,
        67,
    );
    assert_complete_denominator(TRUE_DENOMINATOR_SOURCE, "fn/true.xml", 25, 24, 0);
    assert_complete_denominator(FALSE_DENOMINATOR_SOURCE, "fn/false.xml", 25, 24, 0);
}

#[test]
fn duplicate_private_identity_is_rejected() {
    let duplicate = format!(
        "{PRIVATE_LEDGER_SOURCE}\n[[case]]\nset_file = \"prod/AxisStep.xml\"\ncase_name = \"Axes001-1\"\nselection = \"selected\"\nexecution = \"passed\"\nrationale = \"duplicate\"\n"
    );
    assert!(
        parse_private_ledger(&duplicate)
            .expect_err("duplicate QT3 identity must fail")
            .contains("duplicate case identity")
    );
}
