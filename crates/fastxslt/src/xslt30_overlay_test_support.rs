use std::{collections::BTreeSet, sync::OnceLock};

use serde::Deserialize;

const EXPECTED_SUITE: &str = "W3C XSLT 3.0 test suite";
const EXPECTED_REVISION: &str = "6f8fd9e966ae74a251a2604abef9d904c7bc5c9b";
const PRIVATE_OVERLAY_SOURCE: &str =
    include_str!("../../../corpus/overlays/xslt30/private-slice-v0.toml");
const OUTPUT_OVERLAY_SOURCE: &str =
    include_str!("../../../corpus/overlays/xslt30/output-denominator-v0.toml");

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum SelectionDisposition {
    Selected,
    ExcludedByProfile,
    HarnessUnsupported,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum ExecutionDisposition {
    Passed,
    NativePass,
    NotRun,
    EngineUnsupported,
    HarnessUnsupported,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
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
struct PrivateOverlay {
    suite: String,
    revision: String,
    case: Vec<CaseRecord>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct DefaultDisposition {
    selection: SelectionDisposition,
    execution: ExecutionDisposition,
    rationale: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct CaseOverride {
    case_name: String,
    selection: SelectionDisposition,
    execution: ExecutionDisposition,
    rationale: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct AssertionFamily {
    name: String,
    count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OutputOverlay {
    suite: String,
    revision: String,
    set_file: String,
    case_count: usize,
    default_disposition: DefaultDisposition,
    case_override: Vec<CaseOverride>,
    assertion_family: Vec<AssertionFamily>,
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

fn validate_disposition(
    selection: SelectionDisposition,
    execution: ExecutionDisposition,
) -> Result<(), String> {
    match (selection, execution) {
        (
            SelectionDisposition::ExcludedByProfile | SelectionDisposition::HarnessUnsupported,
            ExecutionDisposition::NotRun,
        )
        | (
            SelectionDisposition::Selected,
            ExecutionDisposition::Passed
            | ExecutionDisposition::NativePass
            | ExecutionDisposition::EngineUnsupported
            | ExecutionDisposition::HarnessUnsupported,
        ) => Ok(()),
        pair => Err(format!("incoherent selection/execution pair {pair:?}")),
    }
}

fn validate_text(value: &str, field: &str, identity: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{identity} has an empty {field}"))
    } else {
        Ok(())
    }
}

fn parse_private_overlay(source: &str) -> Result<PrivateOverlay, String> {
    let overlay: PrivateOverlay = toml::from_str(source).map_err(|error| error.to_string())?;
    validate_header(&overlay.suite, &overlay.revision)?;

    let mut identities = BTreeSet::new();
    for case in &overlay.case {
        let identity = format!("{}::{}", case.set_file, case.case_name);
        validate_text(&case.set_file, "set_file", &identity)?;
        validate_text(&case.case_name, "case_name", &identity)?;
        validate_text(&case.rationale, "rationale", &identity)?;
        validate_disposition(case.selection, case.execution)?;
        if !identities.insert((case.set_file.as_str(), case.case_name.as_str())) {
            return Err(format!("duplicate case identity {identity}"));
        }
    }
    Ok(overlay)
}

fn parse_output_overlay(source: &str) -> Result<OutputOverlay, String> {
    let overlay: OutputOverlay = toml::from_str(source).map_err(|error| error.to_string())?;
    validate_header(&overlay.suite, &overlay.revision)?;
    validate_text(&overlay.set_file, "set_file", "output denominator")?;
    validate_text(
        &overlay.default_disposition.rationale,
        "default rationale",
        "output denominator",
    )?;
    validate_disposition(
        overlay.default_disposition.selection,
        overlay.default_disposition.execution,
    )?;

    let mut names = BTreeSet::new();
    for case in &overlay.case_override {
        validate_text(&case.case_name, "case_name", "output override")?;
        validate_text(&case.rationale, "rationale", &case.case_name)?;
        validate_disposition(case.selection, case.execution)?;
        if !names.insert(case.case_name.as_str()) {
            return Err(format!("duplicate output override {}", case.case_name));
        }
    }
    if overlay.case_override.len() > overlay.case_count {
        return Err(format!(
            "{} overrides exceed the {}-case denominator",
            overlay.case_override.len(),
            overlay.case_count
        ));
    }
    let mut family_names = BTreeSet::new();
    let mut classified_cases = 0_usize;
    for family in &overlay.assertion_family {
        validate_text(&family.name, "assertion family name", "output denominator")?;
        if family.count == 0 {
            return Err(format!("assertion family {} has no cases", family.name));
        }
        if !family_names.insert(family.name.as_str()) {
            return Err(format!("duplicate assertion family {}", family.name));
        }
        classified_cases = classified_cases
            .checked_add(family.count)
            .ok_or_else(|| "assertion-family count overflow".to_owned())?;
    }
    if classified_cases != overlay.case_count {
        return Err(format!(
            "assertion families classify {classified_cases} cases, expected {}",
            overlay.case_count
        ));
    }
    Ok(overlay)
}

fn private_overlay() -> &'static PrivateOverlay {
    static OVERLAY: OnceLock<PrivateOverlay> = OnceLock::new();
    OVERLAY.get_or_init(|| {
        parse_private_overlay(PRIVATE_OVERLAY_SOURCE)
            .unwrap_or_else(|error| panic!("invalid private XSLT30 overlay: {error}"))
    })
}

fn output_overlay() -> &'static OutputOverlay {
    static OVERLAY: OnceLock<OutputOverlay> = OnceLock::new();
    OVERLAY.get_or_init(|| {
        parse_output_overlay(OUTPUT_OVERLAY_SOURCE)
            .unwrap_or_else(|error| panic!("invalid output XSLT30 overlay: {error}"))
    })
}

fn require_private_case_passed(
    overlay: &PrivateOverlay,
    set_file: &str,
    case_name: &str,
) -> Result<(), String> {
    let case = overlay
        .case
        .iter()
        .find(|case| case.set_file == set_file && case.case_name == case_name)
        .ok_or_else(|| format!("missing private overlay case {set_file}::{case_name}"))?;
    if case.selection != SelectionDisposition::Selected
        || case.execution != ExecutionDisposition::Passed
    {
        return Err(format!(
            "{set_file}::{case_name} is {:?}/{:?}, expected selected/passed",
            case.selection, case.execution
        ));
    }
    Ok(())
}

pub(crate) fn assert_private_case_passed(set_file: &str, case_name: &str) {
    require_private_case_passed(private_overlay(), set_file, case_name)
        .unwrap_or_else(|error| panic!("{error}"));
}

pub(crate) fn assert_output_case_passed(case_name: &str) {
    let case = output_overlay()
        .case_override
        .iter()
        .find(|case| case.case_name == case_name)
        .unwrap_or_else(|| panic!("missing output overlay override {case_name}"));
    assert_eq!(
        case.selection,
        SelectionDisposition::Selected,
        "{case_name}"
    );
    assert_eq!(case.execution, ExecutionDisposition::Passed, "{case_name}");
}

#[test]
fn xslt30_overlays_are_typed_unique_and_complete() {
    assert!(!private_overlay().case.is_empty());
    let output = output_overlay();
    assert_eq!(output.set_file, "tests/decl/output/_output-test-set.xml");
    assert_eq!(output.case_count, 232);
}

#[test]
fn a_tested_case_cannot_borrow_another_records_pass() {
    let mut overlay = parse_private_overlay(PRIVATE_OVERLAY_SOURCE).expect("valid overlay");
    let case = overlay
        .case
        .iter_mut()
        .find(|case| case.case_name == "template-001")
        .expect("tested case");
    case.execution = ExecutionDisposition::EngineUnsupported;

    assert_ne!(case.execution, ExecutionDisposition::Passed);
    assert!(overlay.case.iter().any(|case| {
        case.case_name != "template-001" && case.execution == ExecutionDisposition::Passed
    }));
    assert_eq!(
        require_private_case_passed(
            &overlay,
            "tests/decl/template/_template-test-set.xml",
            "template-001"
        ),
        Err("tests/decl/template/_template-test-set.xml::template-001 is Selected/EngineUnsupported, expected selected/passed".to_owned())
    );
}

#[test]
fn duplicate_case_identity_is_rejected() {
    let duplicate = r#"
suite = "W3C XSLT 3.0 test suite"
revision = "6f8fd9e966ae74a251a2604abef9d904c7bc5c9b"

[[case]]
set_file = "set.xml"
case_name = "same"
selection = "selected"
execution = "passed"
rationale = "first"

[[case]]
set_file = "set.xml"
case_name = "same"
selection = "selected"
execution = "passed"
rationale = "second"
"#;

    assert_eq!(
        parse_private_overlay(duplicate).expect_err("duplicate identity must fail"),
        "duplicate case identity set.xml::same"
    );
}
