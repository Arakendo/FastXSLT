//! Local-only compatibility measurement over the archival OASIS XSLT 1.0 suite.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use crate::runtime::workbench_experiment::{
    ExperimentalEngine, WorkbenchFailure, WorkbenchLimits, WorkbenchResource,
    WorkbenchStylesheetResources,
};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

const SUITE_ROOT_ENVIRONMENT: &str = "FASTXSLT_OASIS_XSLT10_ROOT";
const TRACE_CASE_ENVIRONMENT: &str = "FASTXSLT_OASIS_XSLT10_TRACE_CASE";
const TRACE_FRONTIER_ENVIRONMENT: &str = "FASTXSLT_OASIS_XSLT10_TRACE_FRONTIER";

#[derive(Debug)]
struct LegacyCase {
    identity: String,
    id: String,
    category: String,
    operation: String,
    directory: PathBuf,
    reference_directory: PathBuf,
    principal_source: String,
    principal_stylesheet: String,
    supplemental_stylesheets: Vec<String>,
    supplemental_data: Vec<String>,
    output_file: Option<String>,
    output_compare: Option<String>,
}

#[derive(Default)]
struct Measurement {
    counters: BTreeMap<String, usize>,
    initialization_frontiers: BTreeMap<String, usize>,
    execution_frontiers: BTreeMap<String, usize>,
    standard_initialization_frontiers: BTreeMap<String, usize>,
    standard_execution_frontiers: BTreeMap<String, usize>,
    category_totals: BTreeMap<String, usize>,
    category_executed: BTreeMap<String, usize>,
    panic_cases: Vec<String>,
    frontier_examples: BTreeMap<String, String>,
    comparison_frontiers: BTreeMap<String, usize>,
    comparison_examples: BTreeMap<String, String>,
    comparison_unsupported_cases: Vec<(String, String, String)>,
    exact_normalized_non_xml_pass_cases: Vec<String>,
    xml_wrapped_text_pass_cases: Vec<String>,
    infrastructure_cases: Vec<String>,
    expected_error_unexpected_success_cases: Vec<String>,
    unusable_error_expectation_excluded_cases: Vec<String>,
    host_parser_policy_excluded_cases: Vec<String>,
    host_collation_policy_excluded_cases: Vec<String>,
    serialization_layout_policy_excluded_cases: Vec<String>,
    xslt10_discretionary_policy_excluded_cases: Vec<String>,
    legacy_processor_profile_excluded_cases: Vec<String>,
    unusable_reference_excluded_cases: Vec<String>,
    mismatch_cases: Vec<String>,
    doubt_annotated_mismatch_cases: Vec<String>,
}

impl Measurement {
    fn increment(&mut self, key: impl Into<String>) {
        *self.counters.entry(key.into()).or_default() += 1;
    }

    fn increment_by(&mut self, key: impl Into<String>, amount: usize) {
        *self.counters.entry(key.into()).or_default() += amount;
    }

    fn initialization_failure(
        &mut self,
        identity: &str,
        failure: &WorkbenchFailure,
        expected_error: bool,
    ) {
        self.increment("initialization-failure");
        let frontier = failure_frontier(failure);
        trace_frontier_failure(identity, "initialization", &frontier, failure);
        *self
            .initialization_frontiers
            .entry(frontier.clone())
            .or_default() += 1;
        if !expected_error {
            *self
                .standard_initialization_frontiers
                .entry(frontier.clone())
                .or_default() += 1;
        }
        self.frontier_examples
            .entry(frontier)
            .or_insert_with(|| format!("{identity}: {}", bounded_detail(&failure.detail)));
    }

    fn execution_failure(
        &mut self,
        identity: &str,
        failure: &WorkbenchFailure,
        expected_error: bool,
    ) {
        self.increment("execution-failure");
        let frontier = failure_frontier(failure);
        trace_frontier_failure(identity, "execution", &frontier, failure);
        *self
            .execution_frontiers
            .entry(frontier.clone())
            .or_default() += 1;
        if !expected_error {
            *self
                .standard_execution_frontiers
                .entry(frontier.clone())
                .or_default() += 1;
        }
        self.frontier_examples
            .entry(frontier)
            .or_insert_with(|| format!("{identity}: {}", bounded_detail(&failure.detail)));
    }
}

#[test]
#[ignore = "local-only OASIS XSLT 1.0 compatibility measurement; use scripts/measure-oasis-xslt10.ps1"]
#[allow(clippy::too_many_lines)]
fn measures_local_oasis_xslt10_compatibility() {
    let root = PathBuf::from(
        std::env::var_os(SUITE_ROOT_ENVIRONMENT)
            .expect("measurement script must supply the extracted OASIS TESTS directory"),
    );
    let catalog = load_document(&root.join("catalog.xml"), 200_000, 32)
        .expect("reviewed OASIS catalog should parse");
    let cases = catalog_cases(&catalog, &root);
    assert_eq!(cases.len(), 3_173, "reviewed catalog denominator changed");
    let doubtful = doubtful_case_ids(&root.join("doubts.xml"));

    let mut measurement = Measurement::default();
    measurement.increment_by("catalog-cases", cases.len());
    measurement.increment_by("doubt-annotated-identities", doubtful.len());

    for case in cases {
        if case.operation == "execution-error" {
            measurement.increment("expected-error-catalog-case");
        }
        *measurement
            .category_totals
            .entry(case.category.clone())
            .or_default() += 1;
        if doubtful.contains(&case.id) {
            measurement.increment("cases-with-doubt-metadata");
        }
        let Some(source) = read_case_file(&case.directory, &case.principal_source) else {
            measurement.increment("missing-principal-source");
            if case.operation == "execution-error" {
                measurement.increment("expected-error-infrastructure-excluded");
            }
            measurement
                .infrastructure_cases
                .push(format!("missing-principal-source/{}", case.identity));
            continue;
        };
        let Some(stylesheet) = read_case_file(&case.directory, &case.principal_stylesheet) else {
            measurement.increment("missing-principal-stylesheet");
            if case.operation == "execution-error" {
                measurement.increment("expected-error-infrastructure-excluded");
            }
            measurement
                .infrastructure_cases
                .push(format!("missing-principal-stylesheet/{}", case.identity));
            continue;
        };
        let mut resources =
            Vec::with_capacity(case.supplemental_stylesheets.len() + case.supplemental_data.len());
        let mut missing_supplement = false;
        for dependency in &case.supplemental_stylesheets {
            let Some(bytes) = read_case_file(&case.directory, dependency) else {
                missing_supplement = true;
                break;
            };
            resources.push(WorkbenchResource {
                identity: logical_identity(&case, dependency),
                bytes,
            });
        }
        if missing_supplement {
            measurement.increment("missing-supplemental-stylesheet");
            if case.operation == "execution-error" {
                measurement.increment("expected-error-infrastructure-excluded");
            }
            measurement
                .infrastructure_cases
                .push(format!("missing-supplemental-stylesheet/{}", case.identity));
            continue;
        }
        resources = admit_transitive_case_stylesheets(&case, &stylesheet, resources);
        let mut missing_supplemental_data = false;
        for data in &case.supplemental_data {
            let Some(bytes) = read_case_file(&case.directory, data) else {
                missing_supplemental_data = true;
                break;
            };
            resources.push(WorkbenchResource {
                identity: logical_identity(&case, data),
                bytes,
            });
        }
        if missing_supplemental_data {
            measurement.increment("missing-supplemental-data");
            if case.operation == "execution-error" {
                measurement.increment("expected-error-infrastructure-excluded");
            }
            measurement
                .infrastructure_cases
                .push(format!("missing-supplemental-data/{}", case.identity));
            continue;
        }

        let engine = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            ExperimentalEngine::new_with_stylesheet_resources(
                format!(
                    "{}source/{}",
                    logical_case_base(&case),
                    case.principal_source
                ),
                source,
                logical_identity(&case, &case.principal_stylesheet),
                stylesheet,
                WorkbenchStylesheetResources {
                    dependencies: resources,
                    denied_identities: Vec::new(),
                },
                measurement_limits(),
            )
        }));
        let engine = match engine {
            Ok(Ok(engine)) => {
                measurement.increment("initialized");
                engine
            }
            Ok(Err(failure)) => {
                trace_case_failure(&case.identity, "initialization", &failure);
                measurement.initialization_failure(
                    &case.identity,
                    &failure,
                    case.operation == "execution-error",
                );
                if case.operation == "execution-error" {
                    measurement.increment("expected-error-observed-during-initialization");
                    measurement.increment("expected-error-credit");
                }
                continue;
            }
            Err(_) => {
                measurement.increment("initialization-panic");
                measurement
                    .panic_cases
                    .push(format!("initialization/{}", case.identity));
                continue;
            }
        };

        let execution = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            engine.transform_bytes(&case.identity)
        }));
        let actual = match execution {
            Ok(Ok(actual)) => {
                measurement.increment("executed-successfully");
                *measurement
                    .category_executed
                    .entry(case.category.clone())
                    .or_default() += 1;
                match decode_serialized_xml(&actual, engine.selected_output_encoding()) {
                    Ok(actual) => actual,
                    Err(frontier) => {
                        measurement.increment("actual-output-decode-failure");
                        *measurement
                            .comparison_frontiers
                            .entry(frontier.clone())
                            .or_default() += 1;
                        measurement
                            .comparison_examples
                            .entry(frontier)
                            .or_insert(case.identity.clone());
                        continue;
                    }
                }
            }
            Ok(Err(failure)) => {
                trace_case_failure(&case.identity, "execution", &failure);
                measurement.execution_failure(
                    &case.identity,
                    &failure,
                    case.operation == "execution-error",
                );
                if case.operation == "execution-error" {
                    measurement.increment("expected-error-observed-during-execution");
                    measurement.increment("expected-error-credit");
                }
                continue;
            }
            Err(_) => {
                measurement.increment("execution-panic");
                measurement
                    .panic_cases
                    .push(format!("execution/{}", case.identity));
                continue;
            }
        };

        if case.operation == "execution-error" {
            if has_unusable_archival_error_expectation(&case.id) {
                measurement.increment("unusable-error-expectation-excluded");
                measurement
                    .unusable_error_expectation_excluded_cases
                    .push(case.identity.clone());
            } else {
                measurement.increment("expected-error-unexpected-success");
                measurement
                    .expected_error_unexpected_success_cases
                    .push(case.identity.clone());
            }
            continue;
        }
        if case.output_compare.as_deref() != Some("XML") {
            measurement.increment("comparison-not-xml");
            continue;
        }
        if requires_historical_host_parser_whitespace_policy(&case.id) {
            measurement.increment("host-parser-policy-excluded");
            measurement
                .host_parser_policy_excluded_cases
                .push(case.identity.clone());
            continue;
        }
        if requires_host_collation_policy(&case.id) {
            measurement.increment("host-collation-policy-excluded");
            measurement
                .host_collation_policy_excluded_cases
                .push(case.identity.clone());
            continue;
        }
        if requires_serialization_layout_policy(&case.id) {
            measurement.increment("serialization-layout-policy-excluded");
            measurement
                .serialization_layout_policy_excluded_cases
                .push(case.identity.clone());
            continue;
        }
        if requires_xslt10_discretionary_policy(&case.id) {
            measurement.increment("xslt10-discretionary-policy-excluded");
            measurement
                .xslt10_discretionary_policy_excluded_cases
                .push(case.identity.clone());
            continue;
        }
        if requires_legacy_processor_profile(&case.id) {
            measurement.increment("legacy-processor-profile-excluded");
            measurement
                .legacy_processor_profile_excluded_cases
                .push(case.identity.clone());
            continue;
        }
        if has_unusable_archival_reference_result(&case.id) {
            measurement.increment("unusable-reference-result-excluded");
            measurement
                .unusable_reference_excluded_cases
                .push(case.identity.clone());
            continue;
        }
        let Some(output_file) = &case.output_file else {
            measurement.increment("missing-output-metadata");
            continue;
        };
        let Some(expected) = read_case_file(&case.directory, output_file)
            .or_else(|| read_case_file(&case.reference_directory, output_file))
        else {
            measurement.increment("missing-expected-output");
            continue;
        };
        let preserve_whitespace_only_text = case.id.to_ascii_lowercase().contains("whitespace");
        if engine.selected_output_method() == Some("text")
            && let Some(equivalent) = xml_wrapped_text_reference_equivalent(
                &actual,
                &expected,
                engine.selected_output_encoding(),
            )
        {
            trace_case_comparison(&case.identity, &actual, &expected);
            if equivalent {
                measurement.increment("xml-comparison-pass");
                measurement.increment("xml-wrapped-text-comparison-pass");
                measurement
                    .xml_wrapped_text_pass_cases
                    .push(case.identity.clone());
            } else {
                measurement.increment("xml-comparison-mismatch");
                if doubtful.contains(&case.id) {
                    measurement.increment("xml-comparison-mismatch-with-doubt-metadata");
                    measurement
                        .doubt_annotated_mismatch_cases
                        .push(case.identity.clone());
                }
                measurement.mismatch_cases.push(case.identity.clone());
            }
            continue;
        }
        match xml_equivalent(
            &actual,
            &expected,
            preserve_whitespace_only_text,
            engine.selected_output_encoding(),
        ) {
            Ok(true) => {
                trace_case_comparison(&case.identity, &actual, &expected);
                measurement.increment("xml-comparison-pass");
                if exact_normalized_non_xml_payloads(
                    &actual,
                    &expected,
                    engine.selected_output_encoding(),
                ) {
                    measurement.increment("exact-normalized-non-xml-comparison-pass");
                    measurement
                        .exact_normalized_non_xml_pass_cases
                        .push(case.identity.clone());
                }
            }
            Ok(false) => {
                trace_case_comparison(&case.identity, &actual, &expected);
                measurement.increment("xml-comparison-mismatch");
                if doubtful.contains(&case.id) {
                    measurement.increment("xml-comparison-mismatch-with-doubt-metadata");
                    measurement
                        .doubt_annotated_mismatch_cases
                        .push(case.identity.clone());
                }
                measurement.mismatch_cases.push(case.identity.clone());
            }
            Err(frontier) => {
                trace_case_comparison(&case.identity, &actual, &expected);
                measurement.increment("xml-comparator-unsupported");
                if doubtful.contains(&case.id) {
                    measurement.increment("xml-comparator-unsupported-with-doubt-metadata");
                }
                *measurement
                    .comparison_frontiers
                    .entry(frontier.clone())
                    .or_default() += 1;
                measurement
                    .comparison_examples
                    .entry(frontier.clone())
                    .or_insert(case.identity.clone());
                measurement.comparison_unsupported_cases.push((
                    case.identity.clone(),
                    frontier,
                    engine
                        .selected_output_method()
                        .unwrap_or("inferred")
                        .to_owned(),
                ));
            }
        }
    }

    let expected_error_denominator = measurement
        .counters
        .get("expected-error-catalog-case")
        .copied()
        .unwrap_or_default();
    let expected_error_credit = measurement
        .counters
        .get("expected-error-credit")
        .copied()
        .unwrap_or_default();
    let expected_error_infrastructure = measurement
        .counters
        .get("expected-error-infrastructure-excluded")
        .copied()
        .unwrap_or_default();
    assert_eq!(
        expected_error_denominator,
        expected_error_credit
            + measurement.expected_error_unexpected_success_cases.len()
            + measurement.unusable_error_expectation_excluded_cases.len()
            + expected_error_infrastructure,
        "every catalog expected-error case must retain one visible disposition"
    );

    println!("OASIS_XSLT10_MEASUREMENT_BEGIN");
    for (key, value) in &measurement.counters {
        println!("counter\t{key}\t{value}");
    }
    print_ranked(
        "initialization-frontier",
        &measurement.initialization_frontiers,
    );
    print_ranked("execution-frontier", &measurement.execution_frontiers);
    print_ranked(
        "standard-initialization-frontier",
        &measurement.standard_initialization_frontiers,
    );
    print_ranked(
        "standard-execution-frontier",
        &measurement.standard_execution_frontiers,
    );
    print_ranked("comparison-frontier", &measurement.comparison_frontiers);
    for (frontier, example) in &measurement.frontier_examples {
        println!("frontier-example\t{frontier}\t{example}");
    }
    for (frontier, example) in &measurement.comparison_examples {
        println!("comparison-example\t{frontier}\t{example}");
    }
    for identity in &measurement.infrastructure_cases {
        println!("infrastructure-case\t{identity}");
    }
    for identity in &measurement.expected_error_unexpected_success_cases {
        println!("expected-error-unexpected-success-case\t{identity}");
    }
    for identity in &measurement.unusable_error_expectation_excluded_cases {
        println!("unusable-error-expectation-excluded-case\t{identity}");
    }
    for identity in &measurement.host_parser_policy_excluded_cases {
        println!("host-parser-policy-excluded-case\t{identity}");
    }
    for identity in &measurement.host_collation_policy_excluded_cases {
        println!("host-collation-policy-excluded-case\t{identity}");
    }
    for identity in &measurement.serialization_layout_policy_excluded_cases {
        println!("serialization-layout-policy-excluded-case\t{identity}");
    }
    for identity in &measurement.xslt10_discretionary_policy_excluded_cases {
        println!("xslt10-discretionary-policy-excluded-case\t{identity}");
    }
    for identity in &measurement.legacy_processor_profile_excluded_cases {
        println!("legacy-processor-profile-excluded-case\t{identity}");
    }
    for identity in &measurement.unusable_reference_excluded_cases {
        println!("unusable-reference-result-excluded-case\t{identity}");
    }
    for (identity, frontier, output_method) in &measurement.comparison_unsupported_cases {
        println!("comparison-unsupported-case\t{frontier}\tmethod={output_method}\t{identity}");
    }
    for identity in &measurement.exact_normalized_non_xml_pass_cases {
        println!("exact-normalized-non-xml-comparison-pass-case\t{identity}");
    }
    for identity in &measurement.xml_wrapped_text_pass_cases {
        println!("xml-wrapped-text-comparison-pass-case\t{identity}");
    }
    for identity in &measurement.mismatch_cases {
        println!("mismatch-case\t{identity}");
    }
    for identity in &measurement.doubt_annotated_mismatch_cases {
        println!("doubt-annotated-mismatch-case\t{identity}");
    }
    for (category, total) in &measurement.category_totals {
        let executed = measurement
            .category_executed
            .get(category)
            .copied()
            .unwrap_or(0);
        println!("category\t{category}\t{executed}\t{total}");
    }
    for identity in measurement.panic_cases.iter().take(100) {
        println!("panic-case\t{identity}");
    }
    println!("OASIS_XSLT10_MEASUREMENT_END");
}

fn requires_historical_host_parser_whitespace_policy(case_id: &str) -> bool {
    matches!(
        case_id,
        "Whitespaces__91221"
            | "Whitespaces__91222"
            | "Whitespaces__91223"
            | "Whitespaces__91224"
            | "Whitespaces__91225"
            | "Whitespaces__91226"
            | "Whitespaces__91227"
            | "Whitespaces__91228"
            | "Output__77927"
            | "Output__77928"
            | "Output__77939"
            | "Output__78175"
            | "Output__78177"
            | "Output__78182"
            | "Output__78183"
            | "Output__84010"
            | "Output__84015"
            | "Output__84480"
            | "ConflictResolution__77781"
            | "ConflictResolution__77782"
            | "ConflictResolution__77783"
            | "BVTs_bvt056"
            | "Variables__84439"
    )
}

fn requires_host_collation_policy(case_id: &str) -> bool {
    matches!(
        case_id,
        "sort_sort08" | "sort_sort27" | "Sorting__78286" | "Sorting__78291"
    )
}

fn requires_serialization_layout_policy(case_id: &str) -> bool {
    matches!(
        case_id,
        "Attributes__78386"
            | "AttributeSets_AttributeSets_WithPI"
            | "Messages__91758"
            | "Output_EntityRefInAttribHtml"
            | "Output_HtmlOutputWithLessThanInAttribute"
            | "whitespace_whitespace17"
    )
}

fn requires_xslt10_discretionary_policy(case_id: &str) -> bool {
    matches!(case_id, "numbering_numbering79" | "Number__84687")
}

fn has_unusable_archival_reference_result(case_id: &str) -> bool {
    matches!(
        case_id,
        "Output__78221"
            | "Output__77936"
            | "Output__78180"
            | "Output__84455"
            | "Output__84456"
            | "Output__84457"
            | "Output__84458"
            | "Output__84459"
            | "Output__84461"
            | "Output__84462"
            | "Output_EmptyElement1"
            | "Output_MethodEqualsHtmlWithoutIndentSet"
            | "Output_UseLiteralResultElementHead"
            | "Attributes__78365"
            | "Attributes__78372"
            | "AVTs__77574"
            | "AVTs__77591"
            | "BVTs_bvt029"
            | "BVTs_bvt057"
            | "BVTs_bvt091"
            | "BVTs_bvt083"
            | "BVTs_bvt085"
            | "ConflictResolution__77879"
            | "Elements__78362"
            | "Include_RelUriTest5"
            | "Keys__91726"
            | "Keys__91727"
            | "Number__84683"
            | "Text__78272"
            | "Text__78275"
            | "Whitespaces__91443"
            | "Whitespaces__91444"
            | "Whitespaces__91422"
            | "Whitespaces__91423"
            | "Whitespaces__91425"
            | "Whitespaces__91428"
            | "Whitespaces__91453"
            | "Whitespaces__91455"
            | "Whitespaces__91456"
            | "XSLTFunctions__defaultPattern"
            | "XSLTFunctions__EuropeanPattern"
            | "XSLTFunctions__Non_DigitPattern"
            | "XSLTFunctions__Pattern-separator"
            | "XSLTFunctions__percentPattern"
            | "ver_ver05"
            | "ver_ver06"
    )
}

fn requires_legacy_processor_profile(case_id: &str) -> bool {
    case_id == "Namespace__78214"
}

fn has_unusable_archival_error_expectation(case_id: &str) -> bool {
    matches!(
        case_id,
        "Errors_err031"
            | "Miscellaneous__84001"
            | "Namespace__77665"
            | "Namespace__77675"
            | "Output__78176"
    )
}

fn trace_case_comparison(identity: &str, actual: &str, expected: &[u8]) {
    let Some(requested) = std::env::var_os(TRACE_CASE_ENVIRONMENT) else {
        return;
    };
    if !identity.contains(requested.to_string_lossy().as_ref()) {
        return;
    }
    let expected =
        decode_expected_xml(expected, None).unwrap_or_else(|failure| format!("<{failure}>"));
    println!(
        "comparison-trace\t{identity}\tactual={}\texpected={}",
        escaped_detail(actual),
        escaped_detail(&expected)
    );
}

fn trace_case_failure(identity: &str, phase: &str, failure: &WorkbenchFailure) {
    let Some(requested) = std::env::var_os(TRACE_CASE_ENVIRONMENT) else {
        return;
    };
    if !identity.contains(requested.to_string_lossy().as_ref()) {
        return;
    }
    println!(
        "failure-trace\t{identity}\tphase={phase}\tcategory={:?}\tcode={}\tdetail={}",
        failure.category,
        failure.code,
        escaped_detail(&failure.detail)
    );
}

fn trace_frontier_failure(identity: &str, phase: &str, frontier: &str, failure: &WorkbenchFailure) {
    let Some(requested) = std::env::var_os(TRACE_FRONTIER_ENVIRONMENT) else {
        return;
    };
    if !frontier.contains(requested.to_string_lossy().as_ref()) {
        return;
    }
    println!(
        "frontier-trace\t{identity}\tphase={phase}\tfrontier={frontier}\tdetail={}",
        escaped_detail(&failure.detail)
    );
}

fn escaped_detail(detail: &str) -> String {
    const MAX_CHARS: usize = 16_384;
    let mut escaped = detail.escape_debug().take(MAX_CHARS).collect::<String>();
    if detail.escape_debug().count() > MAX_CHARS {
        escaped.push('…');
    }
    escaped
}

fn measurement_limits() -> WorkbenchLimits {
    WorkbenchLimits {
        max_resource_bytes: 16 * 1_048_576,
        max_result_bytes: 16 * 1_048_576,
        max_xml_events: 1_000_000,
        max_xml_depth: 256,
        max_xdm_nodes: 1_000_000,
        max_xpath_operations: 10_000_000,
        max_xslt_instructions: 10_000_000,
        max_xslt_template_candidates: 10_000_000,
        max_result_nodes: 1_000_000,
        max_stylesheet_dependency_depth: 8,
        max_stylesheet_modules: 64,
        max_stylesheet_dependency_bytes: 8 * 1_048_576,
        max_stylesheet_resolution_attempts: 64,
    }
}

fn catalog_cases(document: &Document, root: &Path) -> Vec<LegacyCase> {
    let suite = document_element(document);
    let mut cases = Vec::new();
    for catalog in element_children(document, suite)
        .into_iter()
        .filter(|node| local_name(document, *node) == "test-catalog")
    {
        let submitter = attribute(document, catalog, "submitter").unwrap_or("unknown");
        let major_path = child_named(document, catalog, "major-path")
            .map(|node| document.string_value(node))
            .unwrap_or_default();
        let mut ordinal_by_id = BTreeMap::<String, usize>::new();
        for case in element_children(document, catalog)
            .into_iter()
            .filter(|node| local_name(document, *node) == "test-case")
        {
            let id = attribute(document, case, "id")
                .expect("catalog test case should have identity")
                .to_owned();
            let ordinal = ordinal_by_id.entry(id.clone()).or_default();
            *ordinal += 1;
            let identity = format!("{submitter}/{id}#{}", *ordinal);
            let category = attribute(document, case, "category")
                .unwrap_or("uncategorized")
                .to_owned();
            let file_path = child_named(document, case, "file-path")
                .map(|node| document.string_value(node))
                .unwrap_or_default();
            let scenario = child_named(document, case, "scenario").expect("case scenario");
            let operation = attribute(document, scenario, "operation")
                .unwrap_or("unknown")
                .to_owned();
            let mut principal_source = None;
            let mut principal_stylesheet = None;
            let mut supplemental_stylesheets = Vec::new();
            let mut supplemental_data = Vec::new();
            let mut output_file = None;
            let mut output_compare = None;
            for child in element_children(document, scenario) {
                match local_name(document, child) {
                    "input-file" => match attribute(document, child, "role") {
                        Some("principal-data") => {
                            principal_source = Some(document.string_value(child));
                        }
                        Some("principal-stylesheet") => {
                            principal_stylesheet = Some(document.string_value(child));
                        }
                        Some("supplemental-stylesheet") => {
                            supplemental_stylesheets.push(document.string_value(child));
                        }
                        Some("supplemental-data") => {
                            supplemental_data.push(document.string_value(child));
                        }
                        _ => {}
                    },
                    "output-file" if attribute(document, child, "role") == Some("principal") => {
                        output_file = Some(document.string_value(child));
                        output_compare = attribute(document, child, "compare").map(str::to_owned);
                    }
                    _ => {}
                }
            }
            cases.push(LegacyCase {
                identity,
                id,
                category,
                operation,
                directory: root.join(&major_path).join(&file_path),
                reference_directory: root.join(&major_path).join("REF_OUT").join(&file_path),
                principal_source: principal_source.expect("principal source"),
                principal_stylesheet: principal_stylesheet.expect("principal stylesheet"),
                supplemental_stylesheets,
                supplemental_data,
                output_file,
                output_compare,
            });
        }
    }
    cases
}

fn doubtful_case_ids(path: &Path) -> BTreeSet<String> {
    let document = load_document(path, 100_000, 16).expect("reviewed doubts file should parse");
    let mut doubtful = BTreeSet::new();
    let suite = document_element(&document);
    for catalog in element_children(&document, suite) {
        for case in element_children(&document, catalog)
            .into_iter()
            .filter(|node| local_name(&document, *node) == "test-case")
        {
            if !element_children(&document, case).is_empty()
                && let Some(id) = attribute(&document, case, "id")
            {
                doubtful.insert(id.to_owned());
            }
        }
    }
    doubtful
}

fn logical_case_base(case: &LegacyCase) -> String {
    let identity = case.identity.replace(['/', '#'], "_");
    format!("https://oasis.invalid/{identity}/")
}

fn logical_identity(case: &LegacyCase, file: &str) -> String {
    // Catalog fields are archive-relative file names, not URI references. Keep
    // URI delimiter characters as path data before resolving dot segments.
    let path_reference = file.replace('#', "%23").replace('?', "%3F");
    crate::resources::resolve_reference(&logical_case_base(case), &path_reference).map_or_else(
        |_| format!("{}{path_reference}", logical_case_base(case)),
        |(identity, fragment)| {
            debug_assert!(fragment.is_none());
            identity
        },
    )
}

fn read_case_file(directory: &Path, relative: &str) -> Option<Vec<u8>> {
    std::fs::read(directory.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR))).ok()
}

fn admit_transitive_case_stylesheets(
    case: &LegacyCase,
    principal_bytes: &[u8],
    explicit: Vec<WorkbenchResource>,
) -> Vec<WorkbenchResource> {
    const MAX_DISCOVERED_MODULES: usize = 64;
    const MAX_DISCOVERED_BYTES: usize = 8 * 1_048_576;

    let principal_identity = logical_identity(case, &case.principal_stylesheet);
    let mut admitted = explicit
        .into_iter()
        .map(|resource| (resource.identity, resource.bytes))
        .collect::<BTreeMap<_, _>>();
    let mut pending = VecDeque::from([(
        case.principal_stylesheet.clone(),
        principal_identity.clone(),
        principal_bytes.to_vec(),
    )]);
    for dependency in &case.supplemental_stylesheets {
        let identity = logical_identity(case, dependency);
        if let Some(bytes) = admitted.get(&identity) {
            pending.push_back((dependency.clone(), identity, bytes.clone()));
        }
    }
    let mut scanned = BTreeSet::new();
    let mut discovered_bytes = admitted.values().map(Vec::len).sum::<usize>();
    while let Some((physical_name, identity, bytes)) = pending.pop_front() {
        if !scanned.insert(identity.clone()) {
            continue;
        }
        for reference in stylesheet_dependency_references(&identity, &bytes) {
            let Ok((resolved_identity, fragment)) =
                crate::resources::resolve_reference(&identity, &reference)
            else {
                continue;
            };
            if fragment.is_some()
                || admitted.contains_key(&resolved_identity)
                || resolved_identity == principal_identity
                || admitted.len() >= MAX_DISCOVERED_MODULES
            {
                continue;
            }
            let Some(resolved_name) =
                resolve_case_relative_file(&case.directory, &physical_name, &reference)
            else {
                continue;
            };
            let Some(resolved_bytes) = read_case_file(&case.directory, &resolved_name) else {
                continue;
            };
            let Some(next_bytes) = discovered_bytes.checked_add(resolved_bytes.len()) else {
                continue;
            };
            if next_bytes > MAX_DISCOVERED_BYTES {
                continue;
            }
            discovered_bytes = next_bytes;
            admitted.insert(resolved_identity.clone(), resolved_bytes.clone());
            pending.push_back((resolved_name, resolved_identity, resolved_bytes));
        }
    }
    admitted
        .into_iter()
        .map(|(identity, bytes)| WorkbenchResource { identity, bytes })
        .collect()
}

fn stylesheet_dependency_references(identity: &str, bytes: &[u8]) -> Vec<String> {
    let Ok(parsed) = parse_document(
        identity,
        bytes,
        ParseLimits {
            max_events: 100_000,
            max_depth: 64,
        },
    ) else {
        return Vec::new();
    };
    let Ok(document) = Document::from_parsed(parsed) else {
        return Vec::new();
    };
    let Some(root) = element_children(&document, document.document_node())
        .first()
        .copied()
    else {
        return Vec::new();
    };
    element_children(&document, root)
        .into_iter()
        .filter(|node| {
            document.name(*node).is_some_and(|name| {
                name.namespace.as_deref() == Some("http://www.w3.org/1999/XSL/Transform")
                    && matches!(name.local.as_str(), "include" | "import")
            })
        })
        .filter_map(|node| attribute(&document, node, "href").map(str::to_owned))
        .collect()
}

fn resolve_case_relative_file(directory: &Path, current: &str, reference: &str) -> Option<String> {
    if reference.contains(['#', '?']) || reference.split('/').next()?.contains(':') {
        return None;
    }
    let relative = Path::new(current.replace('/', std::path::MAIN_SEPARATOR_STR).as_str())
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(reference.replace('/', std::path::MAIN_SEPARATOR_STR));
    let root = directory.canonicalize().ok()?;
    let resolved = directory.join(&relative).canonicalize().ok()?;
    let relative = resolved.strip_prefix(root).ok()?;
    Some(relative.to_string_lossy().replace('\\', "/"))
}

fn failure_frontier(failure: &WorkbenchFailure) -> String {
    let detail = failure.detail.as_str();
    if failure.code == "FXXM0002" {
        let source_input_shape = if detail.contains("DtdForbidden") {
            Some("source-input:dtd-forbidden")
        } else if detail.contains("cannot decode input using UTF-8") {
            Some("source-input:non-utf8")
        } else {
            None
        };
        if let Some(source_input_shape) = source_input_shape {
            return format!("{}/{}/{source_input_shape}", failure.category, failure.code);
        }
    }
    for prefix in [
        "unsupported XSLT instruction: ",
        "unsupported top-level XSLT declaration: ",
        "unsupported stylesheet top-level element: ",
    ] {
        if let Some(subject) = detail_subject(detail, prefix) {
            return format!("{}/{}/{prefix}{subject}", failure.category, failure.code);
        }
    }
    if failure.code == "FXST1009"
        && let Some(subject) = detail_subject(detail, "unsupported attribute on ")
    {
        return format!(
            "{}/{}/unsupported attribute on {subject}",
            failure.category, failure.code
        );
    }
    if failure.code == "FXXP1001"
        && let Some(expression) = detail_subject(
            detail,
            "the expression uses syntax outside the private location-path grammar: ",
        )
    {
        return format!(
            "{}/{}/location-path-shape:{}",
            failure.category,
            failure.code,
            xpath_shape(expression)
        );
    }
    let normalized = [
        "unsupported attribute on xsl:output",
        "unsupported xsl:copy-of selection",
        "unsupported xsl:value-of selection",
        "unsupported xsl:for-each selection",
        "unsupported xsl:if test",
        "unsupported xsl:when test",
        "unsupported template match pattern",
    ]
    .into_iter()
    .find(|prefix| detail.contains(prefix))
    .unwrap_or(&failure.code);
    format!("{}/{}/{}", failure.category, failure.code, normalized)
}

fn detail_subject<'a>(detail: &'a str, prefix: &str) -> Option<&'a str> {
    let remainder = detail.split_once(prefix)?.1;
    Some(remainder.split(" at ").next().unwrap_or(remainder).trim())
}

fn xpath_shape(expression: &str) -> String {
    let mut features = Vec::new();
    if expression.contains("::") {
        features.push("axis");
    }
    if expression.contains("//") {
        features.push("descendant");
    }
    if expression.contains('[') {
        features.push("predicate");
    }
    if expression.contains('|') {
        features.push("union");
    }
    if expression.contains('@') {
        features.push("attribute");
    }
    if expression.contains('$') {
        features.push("variable");
    }
    if expression.contains('(') {
        features.push("function");
    }
    if expression.contains(':') && !expression.contains("::") {
        features.push("qualified-name");
    }
    if expression.starts_with('/') {
        features.push("absolute");
    }
    if expression.contains('*') {
        features.push("wildcard");
    }
    if features.is_empty() {
        features.push("other");
    }
    features.join("+")
}

fn bounded_detail(detail: &str) -> String {
    const MAX_CHARS: usize = 240;
    let mut bounded = detail.chars().take(MAX_CHARS).collect::<String>();
    if detail.chars().count() > MAX_CHARS {
        bounded.push('…');
    }
    bounded.replace(['\r', '\n', '\t'], " ")
}

fn print_ranked(label: &str, values: &BTreeMap<String, usize>) {
    let mut values = values.iter().collect::<Vec<_>>();
    values.sort_by(|left, right| right.1.cmp(left.1).then_with(|| left.0.cmp(right.0)));
    for (key, value) in values.into_iter().take(30) {
        println!("{label}\t{key}\t{value}");
    }
}

fn xml_equivalent(
    actual: &str,
    expected: &[u8],
    preserve_whitespace_only_text: bool,
    selected_encoding: Option<&str>,
) -> Result<bool, String> {
    let expected = decode_expected_xml(expected, selected_encoding)?;
    let actual = normalize_xml_source_line_endings(actual);
    let expected = normalize_xml_source_line_endings(&expected);
    let actual_content = strip_xml_declaration(&actual).trim();
    let expected_content = strip_xml_declaration(&expected).trim();
    if actual_content.is_empty() || expected_content.is_empty() {
        return Ok(actual_content == expected_content);
    }
    if actual_content == expected_content {
        return Ok(true);
    }
    if let (Ok(actual), Ok(expected)) = (
        parse_comparison_document("actual", actual.trim()),
        parse_comparison_document("expected", expected.trim()),
    ) {
        return Ok(xml_nodes_equal(
            &actual,
            actual.document_node(),
            &expected,
            expected.document_node(),
            preserve_whitespace_only_text,
        ));
    }
    let actual = parse_comparison_fragment("actual", actual.trim())
        .map_err(|()| "actual-not-parseable-document-or-fragment".to_owned())?;
    let expected = parse_comparison_fragment("expected", expected.trim())
        .map_err(|()| "expected-not-parseable-document-or-fragment".to_owned())?;
    Ok(xml_nodes_equal(
        &actual,
        actual.document_node(),
        &expected,
        expected.document_node(),
        preserve_whitespace_only_text,
    ))
}

fn exact_normalized_non_xml_payloads(
    actual: &str,
    expected: &[u8],
    selected_encoding: Option<&str>,
) -> bool {
    let Ok(expected) = decode_expected_xml(expected, selected_encoding) else {
        return false;
    };
    let actual = normalize_xml_source_line_endings(actual);
    let expected = normalize_xml_source_line_endings(&expected);
    let actual_content = strip_xml_declaration(&actual).trim();
    let expected_content = strip_xml_declaration(&expected).trim();
    if actual_content.is_empty() || actual_content != expected_content {
        return false;
    }
    parse_comparison_document("actual", actual_content).is_err()
        && parse_comparison_fragment("actual", actual_content).is_err()
}

fn xml_wrapped_text_reference_equivalent(
    actual: &str,
    expected: &[u8],
    selected_encoding: Option<&str>,
) -> Option<bool> {
    let expected = decode_expected_xml(expected, selected_encoding).ok()?;
    let actual = normalize_xml_source_line_endings(actual);
    let expected = normalize_xml_source_line_endings(&expected);
    if !expected.trim_start().starts_with("<?xml") {
        return None;
    }
    let expected = parse_comparison_fragment("expected-text", expected.trim()).ok()?;
    let expected_text = expected.string_value(expected.document_node());
    Some(strip_xml_declaration(&actual).trim() == strip_xml_declaration(&expected_text).trim())
}

#[test]
fn oasis_xml_comparator_ignores_serialization_only_empty_element_and_prolog_spacing() {
    let actual = r#"<?xml version="1.0" encoding="UTF-8"?><out test="hello"></out>"#;
    let expected = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<out test=\"hello\"/>\r\n";

    assert_eq!(xml_equivalent(actual, expected, false, None), Ok(true));
}

#[test]
fn oasis_xml_comparator_accepts_exact_normalized_non_xml_payloads_only() {
    let malformed = "<out>&bogus;</out>";
    assert_eq!(
        xml_equivalent(malformed, malformed.as_bytes(), false, None),
        Ok(true)
    );
    assert!(exact_normalized_non_xml_payloads(
        malformed,
        malformed.as_bytes(),
        None
    ));
    assert_ne!(
        xml_equivalent(malformed, b"<out>&different;</out>", false, None),
        Ok(true)
    );
    assert!(!exact_normalized_non_xml_payloads(
        "<out/>", b"<out/>", None
    ));
}

#[test]
fn oasis_text_comparator_decodes_an_xml_wrapped_archival_reference() {
    let actual = "<!ELEMENT AAA ANY>\n<A href=\"urn:test\">value</A>";
    let expected = b"<?xml version=\"1.0\"?>&lt;!ELEMENT AAA ANY&gt;\r\n&lt;A href=\"urn:test\"&gt;value&lt;/A&gt;";

    assert_eq!(
        xml_wrapped_text_reference_equivalent(actual, expected, None),
        Some(true)
    );
    assert_eq!(
        xml_wrapped_text_reference_equivalent("different", expected, None),
        Some(false)
    );
    assert_eq!(xml_wrapped_text_reference_equivalent("&", b"&", None), None);
    assert_eq!(
        xml_wrapped_text_reference_equivalent("<out/>", b"<out/>", None),
        None
    );
}

#[test]
fn oasis_xml_comparator_ignores_a_declaration_for_an_empty_result_tree_only() {
    let declaration_only = r#"<?xml version="1.0" encoding="UTF-8"?>"#;

    assert_eq!(xml_equivalent(declaration_only, b"", false, None), Ok(true));
    assert_eq!(
        xml_equivalent(declaration_only, b"<out/>", false, None),
        Ok(false)
    );
}

#[test]
fn oasis_xml_comparator_compares_generated_prefixes_by_expanded_name() {
    let actual = r#"<root xmlns:ns0="urn:example" ns0:value="kept"></root>"#;
    let expected = br#"<root xmlns:auto-ns1="urn:example" auto-ns1:value="kept"/>"#;

    assert_eq!(xml_equivalent(actual, expected, false, None), Ok(true));
}

#[test]
fn oasis_xml_comparator_normalizes_literal_xml_line_endings_before_parsing() {
    assert_eq!(
        xml_equivalent("<out>a\nb</out>", b"<out>a\r\nb</out>", false, None),
        Ok(true)
    );
    assert_eq!(
        xml_equivalent("<out>&#13;</out>", b"<out>\r</out>", false, None),
        Ok(false)
    );
    let actual = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>far-north north near-north far-west west near-west center\nnear-south south near-south-west near-east east far-east </out>";
    let expected = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<out>far-north north near-north far-west west near-west center\r\nnear-south south near-south-west near-east east far-east </out>";
    assert_eq!(xml_equivalent(actual, expected, false, None), Ok(true));
}

#[test]
fn oasis_xml_comparator_switches_indentation_whitespace_for_whitespace_groups() {
    let actual = "<outer>\n  <inner></inner>\n</outer>";
    let expected = b"<outer>\r\n<inner/>\r\n</outer>";

    assert_eq!(xml_equivalent(actual, expected, false, None), Ok(true));
    assert_eq!(xml_equivalent(actual, expected, true, None), Ok(false));
    assert_eq!(
        xml_equivalent("<out> </out>", b"<out>meaningful</out>", false, None,),
        Ok(false)
    );
}

#[test]
fn oasis_actual_decoder_admits_selected_iso_8859_1_bytes() {
    let actual = b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><out>\xE9</out>";
    assert_eq!(
        decode_serialized_xml(actual, None),
        Ok("<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><out>é</out>".to_owned())
    );
    assert_eq!(
        decode_serialized_xml(b"\xACtwo\xAC", Some("ISO-8859-1")),
        Ok("¬two¬".to_owned())
    );
    assert_eq!(
        decode_expected_xml(b"\xFFtwo\xFF", Some("ISO-8859-1")),
        Ok("ÿtwoÿ".to_owned())
    );
}

#[test]
fn oasis_actual_decoder_admits_bom_marked_utf16_bytes() {
    let expected = "<?xml version=\"1.0\" encoding=\"UTF-16\"?><out>é</out>";
    let mut big_endian = vec![0xfe, 0xff];
    big_endian.extend(expected.encode_utf16().flat_map(u16::to_be_bytes));
    let mut little_endian = vec![0xff, 0xfe];
    little_endian.extend(expected.encode_utf16().flat_map(u16::to_le_bytes));

    assert_eq!(
        decode_serialized_xml(&big_endian, None),
        Ok(expected.to_owned())
    );
    assert_eq!(
        decode_serialized_xml(&little_endian, None),
        Ok(expected.to_owned())
    );
    assert_eq!(
        decode_serialized_xml(&[0xfe, 0xff, 0], None),
        Err("actual-output-invalid-utf16be".to_owned())
    );
}

fn normalize_xml_source_line_endings(value: &str) -> String {
    let normalized = value.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed = normalized.trim_start();
    let Some(after_declaration) = trimmed.strip_prefix("<?xml").and_then(|value| {
        let end = value.find("?>")? + 2;
        Some((&value[..end], &value[end..]))
    }) else {
        return normalized;
    };
    format!(
        "<?xml{}{}",
        after_declaration.0,
        after_declaration.1.trim_start()
    )
}

fn decode_expected_xml(expected: &[u8], selected_encoding: Option<&str>) -> Result<String, String> {
    if let Some(decoded) = decode_bom_utf16(expected) {
        return decoded.map_err(|endian| format!("expected-invalid-utf16{endian}"));
    }
    if selected_encoding.is_some_and(|encoding| encoding.eq_ignore_ascii_case("ISO-8859-1")) {
        return Ok(expected.iter().map(|byte| char::from(*byte)).collect());
    }
    if selected_encoding.is_some_and(|encoding| encoding.eq_ignore_ascii_case("ISO-8859-2")) {
        return Ok(expected
            .iter()
            .map(|byte| super::golden_runtime_experiment::decode_iso_8859_2_byte(*byte))
            .collect());
    }
    std::str::from_utf8(expected)
        .map(|text| text.strip_prefix('\u{feff}').unwrap_or(text).to_owned())
        .map_err(|_| "expected-not-utf8-or-utf16".to_owned())
}

fn decode_serialized_xml(actual: &[u8], selected_encoding: Option<&str>) -> Result<String, String> {
    if let Some(decoded) = decode_bom_utf16(actual) {
        return decoded.map_err(|endian| format!("actual-output-invalid-utf16{endian}"));
    }
    if let Ok(actual) = std::str::from_utf8(actual) {
        return Ok(actual.strip_prefix('\u{feff}').unwrap_or(actual).to_owned());
    }
    let declaration_end = actual
        .windows(2)
        .position(|bytes| bytes == b"?>")
        .map_or(actual.len().min(160), |position| position + 2);
    let declaration = actual
        .get(..declaration_end)
        .and_then(|declaration| std::str::from_utf8(declaration).ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if declaration.contains("encoding=\"iso-8859-1\"")
        || declaration.contains("encoding='iso-8859-1'")
        || selected_encoding.is_some_and(|encoding| encoding.eq_ignore_ascii_case("ISO-8859-1"))
    {
        return Ok(actual.iter().map(|byte| char::from(*byte)).collect());
    }
    if declaration.contains("encoding=\"iso-8859-2\"")
        || declaration.contains("encoding='iso-8859-2'")
        || selected_encoding.is_some_and(|encoding| encoding.eq_ignore_ascii_case("ISO-8859-2"))
    {
        return Ok(actual
            .iter()
            .map(|byte| super::golden_runtime_experiment::decode_iso_8859_2_byte(*byte))
            .collect());
    }
    Err("actual-output-encoding-not-decodable".to_owned())
}

fn decode_bom_utf16(bytes: &[u8]) -> Option<Result<String, &'static str>> {
    let (payload, endian, little_endian) = if let Some(payload) = bytes.strip_prefix(&[0xff, 0xfe])
    {
        (payload, "le", true)
    } else if let Some(payload) = bytes.strip_prefix(&[0xfe, 0xff]) {
        (payload, "be", false)
    } else {
        return None;
    };
    if payload.len() % 2 != 0 {
        return Some(Err(endian));
    }
    let code_units = payload
        .chunks_exact(2)
        .map(|bytes| {
            if little_endian {
                u16::from_le_bytes([bytes[0], bytes[1]])
            } else {
                u16::from_be_bytes([bytes[0], bytes[1]])
            }
        })
        .collect::<Vec<_>>();
    Some(String::from_utf16(&code_units).map_err(|_| endian))
}

fn parse_comparison_document(label: &str, xml: &str) -> Result<Document, ()> {
    let limits = ParseLimits {
        max_events: 1_000_000,
        max_depth: 256,
    };
    let parsed = parse_document(
        &format!("urn:fastxslt:oasis:{label}"),
        xml.as_bytes(),
        limits,
    )
    .map_err(|_| ())?;
    Document::from_parsed(parsed).map_err(|_| ())
}

fn parse_comparison_fragment(label: &str, xml: &str) -> Result<Document, ()> {
    let content = strip_xml_declaration(xml);
    let wrapped = format!("<fastxslt-comparison-root>{content}</fastxslt-comparison-root>");
    parse_comparison_document(label, &wrapped)
}

fn strip_xml_declaration(xml: &str) -> &str {
    let trimmed = xml.trim_start();
    if trimmed.starts_with("<?xml")
        && let Some(end) = trimmed.find("?>")
    {
        return &trimmed[end + 2..];
    }
    xml
}

fn xml_nodes_equal(
    actual: &Document,
    actual_node: NodeId,
    expected: &Document,
    expected_node: NodeId,
    preserve_whitespace_only_text: bool,
) -> bool {
    if actual.kind(actual_node) != expected.kind(expected_node)
        || actual.name(actual_node) != expected.name(expected_node)
        || actual.value(actual_node) != expected.value(expected_node)
    {
        return false;
    }
    let actual_attributes = actual.attributes(actual_node);
    let expected_attributes = expected.attributes(expected_node);
    if actual_attributes.len() != expected_attributes.len() {
        return false;
    }
    for expected_attribute in expected_attributes {
        let Some(expected_name) = expected.name(*expected_attribute) else {
            return false;
        };
        let Some(actual_attribute) = actual_attributes
            .iter()
            .find(|attribute| actual.name(**attribute) == Some(expected_name))
        else {
            return false;
        };
        if actual.value(*actual_attribute) != expected.value(*expected_attribute) {
            return false;
        }
    }
    let actual_children = comparison_children(actual, actual_node, preserve_whitespace_only_text);
    let expected_children =
        comparison_children(expected, expected_node, preserve_whitespace_only_text);
    actual_children.len() == expected_children.len()
        && actual_children.iter().zip(expected_children.iter()).all(
            |(actual_child, expected_child)| {
                xml_nodes_equal(
                    actual,
                    *actual_child,
                    expected,
                    *expected_child,
                    preserve_whitespace_only_text,
                )
            },
        )
}

fn comparison_children(
    document: &Document,
    node: NodeId,
    preserve_whitespace_only_text: bool,
) -> Vec<NodeId> {
    let has_element_child = document
        .children(node)
        .iter()
        .any(|child| document.kind(*child) == NodeKind::Element);
    document
        .children(node)
        .iter()
        .copied()
        .filter(|child| {
            preserve_whitespace_only_text
                || !has_element_child
                || document.kind(*child) != NodeKind::Text
                || document
                    .value(*child)
                    .is_none_or(|value| !value.chars().all(char::is_whitespace))
        })
        .collect()
}

fn load_document(path: &Path, max_events: usize, max_depth: usize) -> Result<Document, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    let parsed = parse_document(
        &format!("urn:fastxslt:local-oasis:legacy:{}", path.display()),
        &bytes,
        ParseLimits {
            max_events,
            max_depth,
        },
    )
    .map_err(|error| format!("{error:?}"))?;
    Document::from_parsed(parsed).map_err(|error| format!("{error:?}"))
}

fn document_element(document: &Document) -> NodeId {
    element_children(document, document.document_node())[0]
}

fn element_children(document: &Document, parent: NodeId) -> Vec<NodeId> {
    document
        .children(parent)
        .iter()
        .copied()
        .filter(|node| document.kind(*node) == NodeKind::Element)
        .collect()
}

fn child_named(document: &Document, parent: NodeId, local: &str) -> Option<NodeId> {
    element_children(document, parent)
        .into_iter()
        .find(|node| local_name(document, *node) == local)
}

fn local_name(document: &Document, node: NodeId) -> &str {
    &document.name(node).expect("element name").local
}

fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
    document.attributes(node).iter().find_map(|attribute| {
        let name = document.name(*attribute)?;
        (name.namespace.is_none() && name.local == local)
            .then(|| document.value(*attribute))
            .flatten()
    })
}

#[test]
fn splits_unsupported_xslt_attributes_by_instruction_and_expanded_name() {
    let failure = WorkbenchFailure {
        code: "FXST1009".to_owned(),
        category: "unsupported".to_owned(),
        request_id: None,
        location: None,
        detail: "unsupported attribute on xsl:copy: {}use-attribute-sets at memory:test.xsl:1..2"
            .to_owned(),
    };

    assert_eq!(
        failure_frontier(&failure),
        "unsupported/FXST1009/unsupported attribute on xsl:copy: {}use-attribute-sets"
    );
}

#[test]
fn splits_oasis_source_input_policy_frontiers_from_other_xml_failures() {
    let failure = |detail: &str| WorkbenchFailure {
        code: "FXXM0002".to_owned(),
        category: "invalid".to_owned(),
        request_id: None,
        location: None,
        detail: detail.to_owned(),
    };

    assert_eq!(
        failure_frontier(&failure(
            "InvalidXml { detail: DtdForbidden { span: 1..2 } }"
        )),
        "invalid/FXXM0002/source-input:dtd-forbidden"
    );
    assert_eq!(
        failure_frontier(&failure(
            "InvalidXml { detail: cannot decode input using UTF-8: invalid byte }"
        )),
        "invalid/FXXM0002/source-input:non-utf8"
    );
    assert_eq!(
        failure_frontier(&failure("InvalidXml { detail: MultipleRoots }")),
        "invalid/FXXM0002/FXXM0002"
    );
}

#[test]
fn oasis_supplemental_stylesheet_identity_normalizes_parent_segments() {
    let case = LegacyCase {
        identity: "Lotus/impincl_impincl04#1".to_owned(),
        id: "impincl_impincl04".to_owned(),
        category: "XSLT-Result-Tree".to_owned(),
        operation: "standard".to_owned(),
        directory: PathBuf::new(),
        reference_directory: PathBuf::new(),
        principal_source: "impincl04.xml".to_owned(),
        principal_stylesheet: "impincl04.xsl".to_owned(),
        supplemental_stylesheets: Vec::new(),
        supplemental_data: Vec::new(),
        output_file: None,
        output_compare: None,
    };

    assert_eq!(
        logical_identity(&case, "../impincl-test/impincl04.xsl"),
        "https://oasis.invalid/impincl-test/impincl04.xsl"
    );
    assert_eq!(
        logical_identity(&case, "output_#_sign.xsl"),
        "https://oasis.invalid/Lotus_impincl_impincl04_1/output_%23_sign.xsl"
    );
}

#[test]
fn oasis_stylesheet_dependency_discovery_reads_only_top_level_xslt_references() {
    let references = stylesheet_dependency_references(
        "https://oasis.invalid/root.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:import href="imports/base.xsl"/>
          <xsl:include href="parts/body.xsl"/>
          <xsl:template match="/"><xsl:include href="not-top-level.xsl"/></xsl:template>
          <other:include xmlns:other="urn:other" href="not-xslt.xsl"/>
        </xsl:stylesheet>"#,
    );

    assert_eq!(references, ["imports/base.xsl", "parts/body.xsl"]);
}

#[test]
fn oasis_host_parser_whitespace_policy_exclusion_is_exact_and_bounded() {
    for suffix in 21..=28 {
        assert!(requires_historical_host_parser_whitespace_policy(&format!(
            "Whitespaces__912{suffix}"
        )));
    }
    assert!(requires_historical_host_parser_whitespace_policy(
        "Variables__84439"
    ));
    for case_id in [
        "Output__77927",
        "Output__77928",
        "Output__77939",
        "Output__78175",
        "Output__78177",
        "Output__78182",
        "Output__78183",
        "Output__84480",
        "Output__84010",
        "Output__84015",
        "ConflictResolution__77781",
        "ConflictResolution__77782",
        "ConflictResolution__77783",
        "BVTs_bvt056",
    ] {
        assert!(requires_historical_host_parser_whitespace_policy(case_id));
    }
    assert!(!requires_historical_host_parser_whitespace_policy(
        "Whitespaces__91421"
    ));
    assert!(!requires_historical_host_parser_whitespace_policy(
        "Output__77938"
    ));
}

#[test]
fn oasis_unusable_reference_result_exclusion_is_exact_and_bounded() {
    for case_id in [
        "Output__78221",
        "Output__77936",
        "Output__78180",
        "Output__84455",
        "Output__84456",
        "Output__84457",
        "Output__84458",
        "Output__84459",
        "Output__84461",
        "Output__84462",
        "Output_EmptyElement1",
        "Output_MethodEqualsHtmlWithoutIndentSet",
        "Output_UseLiteralResultElementHead",
        "Attributes__78365",
        "Attributes__78372",
        "AVTs__77574",
        "AVTs__77591",
        "BVTs_bvt029",
        "BVTs_bvt057",
        "BVTs_bvt091",
        "BVTs_bvt083",
        "BVTs_bvt085",
        "ConflictResolution__77879",
        "Elements__78362",
        "Include_RelUriTest5",
        "Keys__91726",
        "Keys__91727",
        "Number__84683",
        "Text__78272",
        "Text__78275",
        "Whitespaces__91443",
        "Whitespaces__91444",
        "Whitespaces__91422",
        "Whitespaces__91423",
        "Whitespaces__91425",
        "Whitespaces__91428",
        "Whitespaces__91453",
        "Whitespaces__91455",
        "Whitespaces__91456",
        "XSLTFunctions__defaultPattern",
        "XSLTFunctions__EuropeanPattern",
        "XSLTFunctions__Non_DigitPattern",
        "XSLTFunctions__Pattern-separator",
        "XSLTFunctions__percentPattern",
        "ver_ver05",
        "ver_ver06",
    ] {
        assert!(has_unusable_archival_reference_result(case_id));
    }
    assert!(!has_unusable_archival_reference_result("Output__77939"));
    assert!(!has_unusable_archival_reference_result("Text__78242"));
}

#[test]
fn oasis_host_collation_policy_exclusion_is_exact_and_bounded() {
    for case_id in [
        "sort_sort08",
        "sort_sort27",
        "Sorting__78286",
        "Sorting__78291",
    ] {
        assert!(requires_host_collation_policy(case_id));
    }
    assert!(!requires_host_collation_policy("sort_sort07"));
    assert!(!requires_host_collation_policy("BVTs_bvt083"));
}

#[test]
fn oasis_serialization_layout_policy_exclusion_is_exact_and_bounded() {
    for case_id in [
        "Attributes__78386",
        "AttributeSets_AttributeSets_WithPI",
        "Messages__91758",
        "Output_EntityRefInAttribHtml",
        "Output_HtmlOutputWithLessThanInAttribute",
        "whitespace_whitespace17",
    ] {
        assert!(requires_serialization_layout_policy(case_id));
    }
    assert!(!requires_serialization_layout_policy("Attributes__78385"));
    assert!(!requires_serialization_layout_policy(
        "Output_EmptyElement1"
    ));
}

#[test]
fn oasis_xslt10_discretionary_policy_exclusion_is_exact_and_bounded() {
    for case_id in ["numbering_numbering79", "Number__84687"] {
        assert!(requires_xslt10_discretionary_policy(case_id));
    }
    assert!(!requires_xslt10_discretionary_policy("Number__84684"));
    assert!(!requires_xslt10_discretionary_policy(
        "numbering_numbering80"
    ));
}

#[test]
fn oasis_legacy_processor_profile_exclusion_is_exact_and_bounded() {
    assert!(requires_legacy_processor_profile("Namespace__78214"));
    assert!(!requires_legacy_processor_profile("Namespace__78213"));
    assert!(!requires_legacy_processor_profile("Namespace__78215"));
}

#[test]
fn oasis_unusable_error_expectation_exclusion_is_exact_and_bounded() {
    for case_id in [
        "Errors_err031",
        "Miscellaneous__84001",
        "Namespace__77665",
        "Namespace__77675",
        "Output__78176",
    ] {
        assert!(has_unusable_archival_error_expectation(case_id));
    }
    assert!(!has_unusable_archival_error_expectation("Errors_err030"));
    assert!(!has_unusable_archival_error_expectation("Output__78175"));
}
