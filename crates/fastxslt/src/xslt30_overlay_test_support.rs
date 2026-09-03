use std::{collections::BTreeSet, sync::OnceLock};

use serde::Deserialize;

const EXPECTED_SUITE: &str = "W3C XSLT 3.0 test suite";
const EXPECTED_REVISION: &str = "6f8fd9e966ae74a251a2604abef9d904c7bc5c9b";
const PRIVATE_OVERLAY_SOURCE: &str =
    include_str!("../../../corpus/overlays/xslt30/private-slice-v0.toml");
const OUTPUT_OVERLAY_SOURCE: &str =
    include_str!("../../../corpus/overlays/xslt30/output-denominator-v0.toml");
const STRIP_SPACE_OVERLAY_SOURCE: &str =
    include_str!("../../../corpus/overlays/xslt30/strip-space-denominator-v0.toml");
const BUILT_IN_TEMPLATES_OVERLAY_SOURCE: &str =
    include_str!("../../../corpus/overlays/xslt30/built-in-templates-denominator-v0.toml");
const ROOT_OVERLAY_SOURCE: &str =
    include_str!("../../../corpus/overlays/xslt30/root-denominator-v0.toml");
const APPLY_IMPORTS_OVERLAY_SOURCE: &str =
    include_str!("../../../corpus/overlays/xslt30/apply-imports-denominator-v0.toml");
const CHOOSE_OVERLAY_SOURCE: &str =
    include_str!("../../../corpus/overlays/xslt30/choose-denominator-v0.toml");
const CALL_TEMPLATE_OVERLAY_SOURCE: &str =
    include_str!("../../../corpus/overlays/xslt30/call-template-denominator-v0.toml");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DenominatorIdentity {
    Root,
    ApplyImports,
    Choose,
    CallTemplate,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SelectionDisposition {
    Selected,
    ExcludedByProfile,
    HarnessUnsupported,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum ExecutionDisposition {
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DenominatorOverlay {
    suite: String,
    revision: String,
    set_file: String,
    case_count: usize,
    default_disposition: DefaultDisposition,
    case_override: Vec<CaseOverride>,
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

fn parse_denominator_overlay(source: &str, identity: &str) -> Result<DenominatorOverlay, String> {
    let overlay: DenominatorOverlay = toml::from_str(source).map_err(|error| error.to_string())?;
    validate_header(&overlay.suite, &overlay.revision)?;
    validate_text(&overlay.set_file, "set_file", identity)?;
    validate_text(
        &overlay.default_disposition.rationale,
        "default rationale",
        identity,
    )?;
    validate_disposition(
        overlay.default_disposition.selection,
        overlay.default_disposition.execution,
    )?;

    let mut names = BTreeSet::new();
    for case in &overlay.case_override {
        validate_text(&case.case_name, "case_name", identity)?;
        validate_text(&case.rationale, "rationale", &case.case_name)?;
        validate_disposition(case.selection, case.execution)?;
        if !names.insert(case.case_name.as_str()) {
            return Err(format!("duplicate {identity} override {}", case.case_name));
        }
    }
    if overlay.case_override.len() > overlay.case_count {
        return Err(format!(
            "{} overrides exceed the {}-case {identity}",
            overlay.case_override.len(),
            overlay.case_count
        ));
    }
    Ok(overlay)
}

fn parse_known_denominator(
    source: &str,
    identity: DenominatorIdentity,
) -> Result<DenominatorOverlay, String> {
    let (label, expected_set, expected_count) = match identity {
        DenominatorIdentity::Root => ("root denominator", "tests/fn/root/_root-test-set.xml", 10),
        DenominatorIdentity::ApplyImports => (
            "apply-imports denominator",
            "tests/insn/apply-imports/_apply-imports-test-set.xml",
            1,
        ),
        DenominatorIdentity::Choose => (
            "choose denominator",
            "tests/insn/choose/_choose-test-set.xml",
            55,
        ),
        DenominatorIdentity::CallTemplate => (
            "call-template denominator",
            "tests/insn/call-template/_call-template-test-set.xml",
            42,
        ),
    };
    let overlay = parse_denominator_overlay(source, label)?;
    if overlay.set_file != expected_set || overlay.case_count != expected_count {
        return Err(format!(
            "{label} is {}/{}, expected {expected_set}/{expected_count}",
            overlay.set_file, overlay.case_count
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

fn strip_space_overlay() -> &'static DenominatorOverlay {
    static OVERLAY: OnceLock<DenominatorOverlay> = OnceLock::new();
    OVERLAY.get_or_init(|| {
        parse_denominator_overlay(STRIP_SPACE_OVERLAY_SOURCE, "strip-space denominator")
            .unwrap_or_else(|error| panic!("invalid strip-space XSLT30 overlay: {error}"))
    })
}

fn built_in_templates_overlay() -> &'static DenominatorOverlay {
    static OVERLAY: OnceLock<DenominatorOverlay> = OnceLock::new();
    OVERLAY.get_or_init(|| {
        parse_denominator_overlay(
            BUILT_IN_TEMPLATES_OVERLAY_SOURCE,
            "built-in-templates denominator",
        )
        .unwrap_or_else(|error| panic!("invalid built-in-templates XSLT30 overlay: {error}"))
    })
}

fn known_denominator(identity: DenominatorIdentity) -> &'static DenominatorOverlay {
    static ROOT: OnceLock<DenominatorOverlay> = OnceLock::new();
    static APPLY_IMPORTS: OnceLock<DenominatorOverlay> = OnceLock::new();
    static CHOOSE: OnceLock<DenominatorOverlay> = OnceLock::new();
    static CALL_TEMPLATE: OnceLock<DenominatorOverlay> = OnceLock::new();
    let (cell, source) = match identity {
        DenominatorIdentity::Root => (&ROOT, ROOT_OVERLAY_SOURCE),
        DenominatorIdentity::ApplyImports => (&APPLY_IMPORTS, APPLY_IMPORTS_OVERLAY_SOURCE),
        DenominatorIdentity::Choose => (&CHOOSE, CHOOSE_OVERLAY_SOURCE),
        DenominatorIdentity::CallTemplate => (&CALL_TEMPLATE, CALL_TEMPLATE_OVERLAY_SOURCE),
    };
    cell.get_or_init(|| {
        parse_known_denominator(source, identity)
            .unwrap_or_else(|error| panic!("invalid {identity:?}: {error}"))
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

fn require_private_case_execution(
    overlay: &PrivateOverlay,
    set_file: &str,
    case_name: &str,
    expected: ExecutionDisposition,
) -> Result<(), String> {
    let case = overlay
        .case
        .iter()
        .find(|case| case.set_file == set_file && case.case_name == case_name)
        .ok_or_else(|| format!("missing private overlay case {set_file}::{case_name}"))?;
    if case.selection != SelectionDisposition::Selected || case.execution != expected {
        return Err(format!(
            "{set_file}::{case_name} is {:?}/{:?}, expected selected/{expected:?}",
            case.selection, case.execution
        ));
    }
    Ok(())
}

pub(crate) fn assert_private_case_passed(set_file: &str, case_name: &str) {
    require_private_case_passed(private_overlay(), set_file, case_name)
        .unwrap_or_else(|error| panic!("{error}"));
}

pub(crate) fn assert_private_case_disposition(
    set_file: &str,
    case_name: &str,
    selection: SelectionDisposition,
    execution: ExecutionDisposition,
) {
    let case = private_overlay()
        .case
        .iter()
        .find(|case| case.set_file == set_file && case.case_name == case_name)
        .unwrap_or_else(|| panic!("missing private overlay case {set_file}::{case_name}"));
    assert_eq!(case.selection, selection, "{set_file}::{case_name}");
    assert_eq!(case.execution, execution, "{set_file}::{case_name}");
}

pub(crate) fn assert_private_case_native_pass(set_file: &str, case_name: &str) {
    require_private_case_execution(
        private_overlay(),
        set_file,
        case_name,
        ExecutionDisposition::NativePass,
    )
    .unwrap_or_else(|error| panic!("{error}"));
}

pub(crate) fn assert_private_case_engine_unsupported(set_file: &str, case_name: &str) {
    require_private_case_execution(
        private_overlay(),
        set_file,
        case_name,
        ExecutionDisposition::EngineUnsupported,
    )
    .unwrap_or_else(|error| panic!("{error}"));
}

pub(crate) fn assert_private_set_case_names(set_file: &str, expected: &[&str]) {
    let actual = private_overlay()
        .case
        .iter()
        .filter(|case| case.set_file == set_file)
        .map(|case| case.case_name.as_str())
        .collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "private overlay set {set_file}");
}

pub(crate) fn assert_denominator_override_names(identity: DenominatorIdentity, expected: &[&str]) {
    let actual = known_denominator(identity)
        .case_override
        .iter()
        .map(|case| case.case_name.as_str())
        .collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    assert_eq!(actual, expected, "{identity:?} override identities");
}

pub(crate) fn assert_denominator_case_passed(identity: DenominatorIdentity, case_name: &str) {
    assert_denominator_case_disposition(
        identity,
        case_name,
        SelectionDisposition::Selected,
        ExecutionDisposition::Passed,
    );
}

pub(crate) fn assert_denominator_case_disposition(
    identity: DenominatorIdentity,
    case_name: &str,
    selection: SelectionDisposition,
    execution: ExecutionDisposition,
) {
    let case = known_denominator(identity)
        .case_override
        .iter()
        .find(|case| case.case_name == case_name)
        .unwrap_or_else(|| panic!("missing {identity:?} override {case_name}"));
    assert_eq!(case.selection, selection, "{identity:?}::{case_name}");
    assert_eq!(case.execution, execution, "{identity:?}::{case_name}");
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

pub(crate) fn assert_strip_space_case_passed(case_name: &str) {
    let case = strip_space_overlay()
        .case_override
        .iter()
        .find(|case| case.case_name == case_name)
        .unwrap_or_else(|| panic!("missing strip-space overlay override {case_name}"));
    assert_eq!(
        case.selection,
        SelectionDisposition::Selected,
        "{case_name}"
    );
    assert_eq!(case.execution, ExecutionDisposition::Passed, "{case_name}");
}

pub(crate) fn assert_built_in_templates_case_passed(case_name: &str) {
    let case = built_in_templates_overlay()
        .case_override
        .iter()
        .find(|case| case.case_name == case_name)
        .unwrap_or_else(|| panic!("missing built-in-templates overlay override {case_name}"));
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
    let strip_space = strip_space_overlay();
    assert_eq!(
        strip_space.set_file,
        "tests/decl/strip-space/_strip-space-test-set.xml"
    );
    assert_eq!(strip_space.case_count, 30);
    let built_in_templates = built_in_templates_overlay();
    assert_eq!(
        built_in_templates.set_file,
        "tests/misc/built-in-templates/_built-in-templates-test-set.xml"
    );
    assert_eq!(built_in_templates.case_count, 6);
    assert_eq!(known_denominator(DenominatorIdentity::Root).case_count, 10);
    assert_eq!(
        known_denominator(DenominatorIdentity::ApplyImports).case_count,
        1
    );
    assert_eq!(
        known_denominator(DenominatorIdentity::Choose).case_count,
        55
    );
    assert_eq!(
        known_denominator(DenominatorIdentity::CallTemplate).case_count,
        42
    );
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
