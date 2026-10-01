//! Local-only compatibility measurement over the archival OASIS XSLT 1.0 suite.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

use encoding_rs::{BIG5, Encoding, ISO_2022_JP, SHIFT_JIS, WINDOWS_1252};

use crate::runtime::oasis_html_comparator::normalize_for_xml_comparison;
use crate::runtime::workbench_experiment::{
    ExperimentalEngine, WorkbenchExternalSubsetLimits, WorkbenchFailure, WorkbenchLimits,
    WorkbenchResource, WorkbenchStylesheetResources,
};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
use crate::xml::internal_subset::InternalSubsetLimits;
use crate::xml::quick_xml_experiment::{
    ParseLimits, parse_document, parse_document_with_internal_subset,
};

const SUITE_ROOT_ENVIRONMENT: &str = "FASTXSLT_OASIS_XSLT10_ROOT";
const TRACE_CASE_ENVIRONMENT: &str = "FASTXSLT_OASIS_XSLT10_TRACE_CASE";
const TRACE_FRONTIER_ENVIRONMENT: &str = "FASTXSLT_OASIS_XSLT10_TRACE_FRONTIER";
const MAX_REVIEWED_EXTERNAL_SUBSET_BYTES: usize = 64 * 1_024;

struct ReviewedExternalSubset {
    reference: &'static str,
    identity: String,
    bytes: Vec<u8>,
}

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
    bounded_doctype_pass_cases: Vec<String>,
    bounded_doctype_mismatch_cases: Vec<String>,
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
    direct_dtd_frontier_properties: BTreeMap<String, usize>,
    direct_dtd_frontier_comparators: BTreeMap<String, usize>,
    direct_dtd_reference_outcomes: BTreeMap<String, usize>,
    direct_dtd_reference_cases: Vec<(String, &'static str, &'static str, String)>,
}

#[derive(Debug, Clone)]
struct DtdProperties(Vec<&'static str>);

impl DtdProperties {
    fn inspect(bytes: &[u8]) -> Self {
        let text = String::from_utf8_lossy(bytes);
        let doctype = text
            .find("<!DOCTYPE")
            .map(|start| {
                let remainder = &text[start..];
                let length = remainder.find("]>").map_or_else(
                    || remainder.find('>').map_or(remainder.len(), |end| end + 1),
                    |end| end + 2,
                );
                &remainder[..length]
            })
            .unwrap_or_default();
        let candidates = [
            ("internal-subset", doctype.contains('[')),
            (
                "external-identifier",
                doctype.contains(" SYSTEM ") || doctype.contains(" PUBLIC "),
            ),
            ("entity-declaration", doctype.contains("<!ENTITY ")),
            (
                "internal-general-entity-candidate",
                dtd_declaration_matches(doctype, "<!ENTITY", |declaration| {
                    !is_parameter_entity(declaration)
                        && !declaration.contains(" SYSTEM ")
                        && !declaration.contains(" PUBLIC ")
                }),
            ),
            (
                "external-general-entity-candidate",
                dtd_declaration_matches(doctype, "<!ENTITY", |declaration| {
                    !is_parameter_entity(declaration)
                        && (declaration.contains(" SYSTEM ") || declaration.contains(" PUBLIC "))
                }),
            ),
            (
                "unparsed-entity-candidate",
                dtd_declaration_matches(doctype, "<!ENTITY", |declaration| {
                    declaration.contains(" NDATA ")
                }),
            ),
            (
                "parameter-entity-declaration",
                doctype.contains("<!ENTITY %") || doctype.contains("<!ENTITY  %"),
            ),
            ("attribute-list-declaration", doctype.contains("<!ATTLIST ")),
            (
                "default-attribute-candidate",
                dtd_declaration_matches(doctype, "<!ATTLIST", |declaration| {
                    declaration.contains('"') || declaration.contains('\'')
                }),
            ),
            (
                "id-typing-candidate",
                doctype.contains(" ID ") || doctype.contains("\tID "),
            ),
            ("notation-declaration", doctype.contains("<!NOTATION ")),
        ];
        Self(
            candidates
                .into_iter()
                .filter_map(|(name, present)| present.then_some(name))
                .collect(),
        )
    }

    fn named(self) -> impl Iterator<Item = &'static str> {
        self.0.into_iter()
    }

    fn has_dtd(&self) -> bool {
        !self.0.is_empty()
    }
}

fn is_parameter_entity(declaration: &str) -> bool {
    declaration.strip_prefix("<!ENTITY").is_some_and(|body| {
        body.trim_start_matches([' ', '\t', '\r', '\n'])
            .starts_with('%')
    })
}

fn dtd_declaration_matches(doctype: &str, marker: &str, predicate: impl Fn(&str) -> bool) -> bool {
    let mut remainder = doctype;
    while let Some(start) = remainder.find(marker) {
        let declaration = &remainder[start..];
        let mut quote = None;
        let mut end = None;
        for (offset, character) in declaration.char_indices() {
            match (quote, character) {
                (Some(active), current) if current == active => quote = None,
                (None, '\'' | '"') => quote = Some(character),
                (None, '>') => {
                    end = Some(offset);
                    break;
                }
                _ => {}
            }
        }
        let Some(end) = end else {
            return false;
        };
        if predicate(&declaration[..=end]) {
            return true;
        }
        remainder = &declaration[end + 1..];
    }
    false
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

    fn direct_dtd_failure(
        &mut self,
        case: &LegacyCase,
        failure: &WorkbenchFailure,
        source_identity: &str,
        input_observations: &BTreeMap<String, (DtdProperties, &'static str)>,
    ) -> bool {
        if case.operation != "standard" {
            return false;
        }
        let observation = failure
            .location
            .as_ref()
            .and_then(|location| input_observations.get_key_value(&location.resource))
            .filter(|(_, (properties, _))| properties.has_dtd())
            .or_else(|| {
                input_observations.iter().find(|(identity, observation)| {
                    observation.0.has_dtd() && failure.detail.contains(*identity)
                })
            });
        let Some((identity, (properties, reference_outcome))) = observation else {
            return false;
        };
        let role = if identity == source_identity {
            "source"
        } else {
            "stylesheet"
        };
        self.record_direct_dtd_case(case, role, properties, reference_outcome);
        true
    }

    fn direct_dtd_success(
        &mut self,
        case: &LegacyCase,
        source_identity: &str,
        input_observations: &BTreeMap<String, (DtdProperties, &'static str)>,
        used_reviewed_source_external_subset: bool,
        used_reviewed_stylesheet_external_subset: bool,
    ) {
        if case.operation != "standard" {
            return;
        }
        let observation = input_observations
            .get(source_identity)
            .filter(|(properties, _)| properties.has_dtd())
            .map(|observation| ("source", observation))
            .or_else(|| {
                input_observations
                    .iter()
                    .find(|(_, (properties, _))| properties.has_dtd())
                    .map(|(_, observation)| ("stylesheet", observation))
            });
        if let Some((role, (properties, reference_outcome))) = observation {
            let reference_outcome = if (used_reviewed_source_external_subset && role == "source")
                || (used_reviewed_stylesheet_external_subset && role == "stylesheet")
            {
                &"parsed-single-external-subset"
            } else {
                reference_outcome
            };
            self.record_direct_dtd_case(case, role, properties, reference_outcome);
        }
    }

    fn record_direct_dtd_case(
        &mut self,
        case: &LegacyCase,
        role: &'static str,
        properties: &DtdProperties,
        reference_outcome: &&'static str,
    ) {
        self.increment("direct-dtd-frontier-case");
        *self
            .direct_dtd_frontier_properties
            .entry(format!("role:{role}"))
            .or_default() += 1;
        let property_names = properties.0.join(",");
        for name in properties.clone().named() {
            *self
                .direct_dtd_frontier_properties
                .entry(name.to_owned())
                .or_default() += 1;
        }
        *self
            .direct_dtd_frontier_comparators
            .entry(
                case.output_compare
                    .clone()
                    .unwrap_or_else(|| "missing".to_owned()),
            )
            .or_default() += 1;
        *self
            .direct_dtd_reference_outcomes
            .entry((*reference_outcome).to_owned())
            .or_default() += 1;
        self.direct_dtd_reference_cases.push((
            case.identity.clone(),
            role,
            *reference_outcome,
            property_names,
        ));
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
        resources = admit_transitive_case_stylesheets(
            &case,
            &stylesheet,
            resources,
            case.operation == "standard" && !has_non_equivalent_historical_uri_alias(&case.id),
        );
        resources = admit_principal_literal_document_resources(&case, &stylesheet, resources);
        resources.extend(reviewed_source_document_resources(&case));
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

        let source_identity = format!(
            "{}source/{}",
            logical_case_base(&case),
            case.principal_source
        );
        let stylesheet_identity = logical_identity(&case, &case.principal_stylesheet);
        let reviewed_external_subset =
            reviewed_external_source_subset(&case, &source_identity, &source);
        let reviewed_stylesheet_external_subset = reviewed_external_stylesheet_subset(
            &case,
            &stylesheet_identity,
            &stylesheet,
            &resources,
        );
        let used_reviewed_external_subset = reviewed_external_subset.is_some();
        let used_reviewed_stylesheet_external_subset =
            reviewed_stylesheet_external_subset.is_some();
        if let Some(external_subset) = &reviewed_external_subset {
            resources.push(WorkbenchResource {
                identity: external_subset.identity.clone(),
                bytes: external_subset.bytes.clone(),
            });
            measurement.increment("reviewed-single-external-subset-case");
        }
        if let Some(external_subset) = &reviewed_stylesheet_external_subset {
            if !resources
                .iter()
                .any(|resource| resource.identity == external_subset.identity)
            {
                resources.push(WorkbenchResource {
                    identity: external_subset.identity.clone(),
                    bytes: external_subset.bytes.clone(),
                });
            }
            measurement.increment("reviewed-external-stylesheet-subset-case");
        }
        let mut dtd_input_observations = BTreeMap::new();
        record_dtd_input_observation(&mut dtd_input_observations, &source_identity, &source);
        record_dtd_input_observation(
            &mut dtd_input_observations,
            &stylesheet_identity,
            &stylesheet,
        );
        for resource in &resources {
            record_dtd_input_observation(
                &mut dtd_input_observations,
                &resource.identity,
                &resource.bytes,
            );
        }

        let engine = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let stylesheet_resources = WorkbenchStylesheetResources {
                dependencies: resources,
                denied_identities: Vec::new(),
            };
            match (
                reviewed_external_subset,
                reviewed_stylesheet_external_subset,
            ) {
                (source_external, stylesheet_external)
                    if source_external.is_some() || stylesheet_external.is_some() =>
                {
                    ExperimentalEngine::new_with_bounded_external_subsets(
                        source_identity.clone(),
                        source,
                        stylesheet_identity,
                        stylesheet,
                        stylesheet_resources,
                        measurement_limits(),
                        WorkbenchExternalSubsetLimits {
                            declarations: dtd_reference_limits(),
                            source: source_external.map(|external| {
                                (external.reference, MAX_REVIEWED_EXTERNAL_SUBSET_BYTES)
                            }),
                            stylesheet: stylesheet_external.map(|external| {
                                (external.reference, MAX_REVIEWED_EXTERNAL_SUBSET_BYTES)
                            }),
                        },
                    )
                }
                _ => ExperimentalEngine::new_with_bounded_internal_subsets(
                    source_identity.clone(),
                    source,
                    stylesheet_identity,
                    stylesheet,
                    stylesheet_resources,
                    measurement_limits(),
                    dtd_reference_limits(),
                ),
            }
        }));
        let engine = match engine {
            Ok(Ok(engine)) => {
                measurement.increment("initialized");
                measurement.direct_dtd_success(
                    &case,
                    &source_identity,
                    &dtd_input_observations,
                    used_reviewed_external_subset,
                    used_reviewed_stylesheet_external_subset,
                );
                engine
            }
            Ok(Err(failure)) => {
                trace_case_failure(&case.identity, "initialization", &failure);
                let recorded_dtd_failure = measurement.direct_dtd_failure(
                    &case,
                    &failure,
                    &source_identity,
                    &dtd_input_observations,
                );
                if !recorded_dtd_failure
                    && (used_reviewed_external_subset || used_reviewed_stylesheet_external_subset)
                {
                    measurement.direct_dtd_success(
                        &case,
                        &source_identity,
                        &dtd_input_observations,
                        used_reviewed_external_subset,
                        used_reviewed_stylesheet_external_subset,
                    );
                }
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
        let reference_encoding =
            archival_reference_encoding(&case.id, engine.selected_output_encoding());
        if engine.selected_output_method() == Some("text")
            && let Some(equivalent) =
                xml_wrapped_text_reference_equivalent(&actual, &expected, reference_encoding)
        {
            trace_case_comparison(&case.identity, &actual, &expected, reference_encoding);
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
        let bounded_doctype_pair =
            has_matching_bounded_doctype(&actual, &expected, reference_encoding);
        match xml_equivalent(
            &actual,
            &expected,
            preserve_whitespace_only_text,
            reference_encoding,
            engine.selected_output_method(),
        ) {
            Ok(true) => {
                trace_case_comparison(&case.identity, &actual, &expected, reference_encoding);
                measurement.increment("xml-comparison-pass");
                if bounded_doctype_pair {
                    measurement.increment("bounded-doctype-comparison-pass");
                    measurement
                        .bounded_doctype_pass_cases
                        .push(case.identity.clone());
                }
                if exact_normalized_non_xml_payloads(&actual, &expected, reference_encoding) {
                    measurement.increment("exact-normalized-non-xml-comparison-pass");
                    measurement
                        .exact_normalized_non_xml_pass_cases
                        .push(case.identity.clone());
                }
            }
            Ok(false) => {
                trace_case_comparison(&case.identity, &actual, &expected, reference_encoding);
                measurement.increment("xml-comparison-mismatch");
                if bounded_doctype_pair {
                    measurement.increment("bounded-doctype-comparison-mismatch");
                    measurement
                        .bounded_doctype_mismatch_cases
                        .push(case.identity.clone());
                }
                if doubtful.contains(&case.id) {
                    measurement.increment("xml-comparison-mismatch-with-doubt-metadata");
                    measurement
                        .doubt_annotated_mismatch_cases
                        .push(case.identity.clone());
                }
                measurement.mismatch_cases.push(case.identity.clone());
            }
            Err(frontier) => {
                trace_case_comparison(&case.identity, &actual, &expected, reference_encoding);
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
    print_ranked(
        "direct-dtd-frontier-property",
        &measurement.direct_dtd_frontier_properties,
    );
    print_ranked(
        "direct-dtd-frontier-comparator",
        &measurement.direct_dtd_frontier_comparators,
    );
    print_ranked(
        "direct-dtd-reference-outcome",
        &measurement.direct_dtd_reference_outcomes,
    );
    for (identity, role, outcome, properties) in &measurement.direct_dtd_reference_cases {
        println!(
            "direct-dtd-reference-case\t{outcome}\t{role}\tproperties={properties}\t{identity}"
        );
    }
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
    for identity in &measurement.bounded_doctype_pass_cases {
        println!("bounded-doctype-comparison-pass-case\t{identity}");
    }
    for identity in &measurement.bounded_doctype_mismatch_cases {
        println!("bounded-doctype-comparison-mismatch-case\t{identity}");
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

fn reviewed_source_document_resources(case: &LegacyCase) -> Vec<WorkbenchResource> {
    let resources: &[(&str, &str)] = match case.identity.as_str() {
        "Lotus/mdocs_mdocs02#1" => &[
            ("mdocs02a.xml", "source/mdocs02a.xml"),
            ("mdocs02b.xml", "source/mdocs02b.xml"),
        ],
        "Lotus/mdocs_mdocs03#1" => &[("mdocs03a.xml", "source/mdocs03a.xml")],
        "Lotus/mdocs_mdocs15#1" => &[("mdocs15a.xml", "source/mdocs15a.xml")],
        "Lotus/mdocs_mdocs04#1" | "Lotus/mdocs_mdocs08#1" => &[
            ("mdocs04a.xml", "source/mdocs04a.xml"),
            ("mdocs04b.xml", "source/mdocs04b.xml"),
        ],
        "Lotus/mdocs_mdocs05#1" => &[("compu.xml", "source/compu.xml")],
        "Lotus/mdocs_mdocs06#1"
        | "Lotus/mdocs_mdocs07#1"
        | "Microsoft/XSLTFunctions_DocumentInUnionWithDuplicateNodes#1" => &[
            ("mdocs04a.xml", "source/mdocs04a.xml"),
            ("mdocs04b.xml", "source/mdocs04b.xml"),
            ("mdocs06a.xml", "source/mdocs06a.xml"),
            ("mdocs06b.xml", "source/mdocs06b.xml"),
        ],
        "Lotus/mdocs_mdocs18#1" => &[
            ("mdwords-a.xml", "source/mdwords-a.xml"),
            ("mdwords-b.xml", "source/mdwords-b.xml"),
        ],
        // The catalog makes select68.xml the principal input under the
        // harness's source/ identity, while document('select68.xml') resolves
        // beside the stylesheet. Admit the same reviewed bytes under that
        // distinct logical identity; resolution itself remains engine-owned.
        "Lotus/select_select68#1" => &[("select68.xml", "select68.xml")],
        "Lotus/reluri_reluri10#1" => &[(
            "level1/level2/level3/xreluri09a.xml",
            "level1/level2/level3/xreluri09a.xml",
        )],
        _ => &[],
    };
    resources
        .iter()
        .filter_map(|(physical, logical)| {
            read_case_file(&case.directory, physical).map(|bytes| WorkbenchResource {
                identity: format!("{}{logical}", logical_case_base(case)),
                bytes,
            })
        })
        .collect()
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
            | "Output__77940"
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
        "sort_sort08" | "sort_sort27" | "Sorting__77977" | "Sorting__78286" | "Sorting__78291"
    )
}

fn requires_serialization_layout_policy(case_id: &str) -> bool {
    matches!(
        case_id,
        "Attributes__78386"
            | "AttributeSets_AttributeSets_WithPI"
            | "BVTs_bvt063"
            | "BVTs_bvt064"
            | "Messages__91758"
            | "Output__84260"
            | "Output__84264"
            | "Output__84271"
            | "Output__84273"
            | "Output__84277"
            | "Output__84280"
            | "Output__84282"
            | "Output__84285"
            | "Output__84309"
            | "Output_EntityRefInAttribHtml"
            | "Output_HtmlOutputWithLessThanInAttribute"
            | "output_output36"
            | "whitespace_whitespace17"
    )
}

fn requires_xslt10_discretionary_policy(case_id: &str) -> bool {
    matches!(
        case_id,
        "numbering_numbering79"
            | "Number__84687"
            | "Number__84700"
            | "Number__91028"
            | "Number__91029"
    )
}

fn has_non_equivalent_historical_uri_alias(case_id: &str) -> bool {
    matches!(case_id, "Include__77745")
}

fn has_unusable_archival_reference_result(case_id: &str) -> bool {
    matches!(
        case_id,
        "Output__78221"
            | "Output__77936"
            | "Output__78180"
            | "Output__84165"
            | "Output__84374"
            | "Output__84429"
            | "Output__84452"
            | "Output__84453"
            | "Output__84454"
            | "Output__84455"
            | "Output__84456"
            | "Output__84457"
            | "Output__84458"
            | "Output__84459"
            | "Output__84460"
            | "Output__84461"
            | "Output__84462"
            | "Output_EmptyElement1"
            | "Output_MethodEqualsHtmlWithoutIndentSet"
            | "Output_UseLiteralResultElementHead"
            | "output_output40"
            | "output_output48"
            | "output_output59"
            | "Output_DoctypePublicAndSystemAttribute"
            | "Output_DoctypePublicAttribute"
            | "Output_DoctypeSystemAttribute"
            | "Attributes__78365"
            | "Attributes__78372"
            | "AVTs__77574"
            | "AVTs__77591"
            | "BVTs_bvt029"
            | "BVTs_bvt055"
            | "BVTs_bvt057"
            | "BVTs_bvt067"
            | "BVTs_bvt098"
            | "BVTs_bvt091"
            | "BVTs_bvt083"
            | "BVTs_bvt085"
            | "ConflictResolution__77879"
            | "Elements__78362"
            | "Include__77515"
            | "Include__77736"
            | "Include_RelUriTest5"
            | "Keys__91726"
            | "Keys__91727"
            | "Number__84683"
            | "Number__84692"
            | "Number__84722"
            | "Number__91022"
            | "Number__91027"
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
            | "XSLTFunctions__minimalValue"
            | "XSLTFunctions__minimumValue"
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

fn archival_reference_encoding<'a>(
    case_id: &str,
    selected_encoding: Option<&'a str>,
) -> Option<&'a str> {
    if matches!(
        case_id,
        "Output__84374" | "Output__84428" | "Output__84429" | "Sorting__77977"
    ) {
        Some("windows-1252")
    } else {
        selected_encoding
    }
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

fn trace_case_comparison(
    identity: &str,
    actual: &str,
    expected: &[u8],
    selected_encoding: Option<&str>,
) {
    let Some(requested) = std::env::var_os(TRACE_CASE_ENVIRONMENT) else {
        return;
    };
    if !identity.contains(requested.to_string_lossy().as_ref()) {
        return;
    }
    let expected = decode_expected_xml(expected, selected_encoding)
        .unwrap_or_else(|failure| format!("<{failure}>"));
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

fn record_dtd_input_observation(
    observations: &mut BTreeMap<String, (DtdProperties, &'static str)>,
    identity: &str,
    bytes: &[u8],
) {
    let properties = DtdProperties::inspect(bytes);
    let outcome = dtd_reference_outcome(identity, bytes, &properties);
    if properties.has_dtd() || !observations.contains_key(identity) {
        observations.insert(identity.to_owned(), (properties, outcome));
    }
}

fn dtd_reference_outcome(identity: &str, bytes: &[u8], properties: &DtdProperties) -> &'static str {
    if !properties.has_dtd() {
        return "not-applicable";
    }
    match parse_document_with_internal_subset(
        identity,
        bytes,
        ParseLimits {
            max_events: 1_000_000,
            max_depth: 256,
        },
        dtd_reference_limits(),
    ) {
        Ok(_) => "parsed",
        Err(failure) => failure.dtd_reference_category(),
    }
}

const fn dtd_reference_limits() -> InternalSubsetLimits {
    InternalSubsetLimits {
        declarations: 4_096,
        nesting_depth: 32,
        references: 100_000,
        replacement_bytes: 16 * 1_048_576,
    }
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

fn reviewed_external_source_subset(
    case: &LegacyCase,
    source_identity: &str,
    source: &[u8],
) -> Option<ReviewedExternalSubset> {
    let (reference, declaration) = if case.principal_source.eq_ignore_ascii_case("plants.xml") {
        ("plants.dtd", br#"SYSTEM "plants.dtd""#.as_slice())
    } else if case.id == "idkey_idkey04" {
        ("t04.dtd", br#"SYSTEM "t04.dtd""#.as_slice())
    } else if case.id == "numbering_numbering91" {
        ("iddata.dtd", br#"SYSTEM "iddata.dtd""#.as_slice())
    } else {
        return None;
    };
    if !source
        .windows(declaration.len())
        .any(|window| window == declaration)
    {
        return None;
    }
    let bytes = read_case_file(&case.directory, reference)?;
    if bytes.len() > MAX_REVIEWED_EXTERNAL_SUBSET_BYTES {
        return None;
    }
    let (identity, fragment) =
        crate::resources::resolve_reference(source_identity, reference).ok()?;
    if fragment.is_some() {
        return None;
    }
    Some(ReviewedExternalSubset {
        reference,
        identity,
        bytes,
    })
}

fn reviewed_external_stylesheet_subset(
    case: &LegacyCase,
    stylesheet_identity: &str,
    stylesheet: &[u8],
    resources: &[WorkbenchResource],
) -> Option<ReviewedExternalSubset> {
    const REVIEWED_REFERENCES: [&str; 3] = ["stylesheet.dtd", "stylesheet1.dtd", "htmllat1.dtd"];
    let (reference, declaring_identity) =
        REVIEWED_REFERENCES.into_iter().find_map(|reference| {
            if declares_system_reference(stylesheet, reference) {
                return Some((reference, stylesheet_identity));
            }
            resources
                .iter()
                .find(|resource| declares_system_reference(&resource.bytes, reference))
                .map(|resource| (reference, resource.identity.as_str()))
        })?;
    let bytes = read_case_file(&case.directory, reference)?;
    if bytes.len() > MAX_REVIEWED_EXTERNAL_SUBSET_BYTES {
        return None;
    }
    let (identity, fragment) =
        crate::resources::resolve_reference(declaring_identity, reference).ok()?;
    if fragment.is_some() {
        return None;
    }
    Some(ReviewedExternalSubset {
        reference,
        identity,
        bytes,
    })
}

fn declares_system_reference(bytes: &[u8], reference: &str) -> bool {
    let single_quoted = format!("SYSTEM '{reference}'");
    let double_quoted = format!("SYSTEM \"{reference}\"");
    bytes
        .windows(single_quoted.len())
        .any(|window| window == single_quoted.as_bytes())
        || bytes
            .windows(double_quoted.len())
            .any(|window| window == double_quoted.as_bytes())
}

fn read_case_file(directory: &Path, relative: &str) -> Option<Vec<u8>> {
    std::fs::read(directory.join(relative.replace('/', std::path::MAIN_SEPARATOR_STR))).ok()
}

fn admit_transitive_case_stylesheets(
    case: &LegacyCase,
    principal_bytes: &[u8],
    explicit: Vec<WorkbenchResource>,
    allow_historical_uri_alias: bool,
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
            let Some(resolved_name) = resolve_case_dependency_file(
                &case.directory,
                &physical_name,
                &reference,
                allow_historical_uri_alias,
            ) else {
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

fn admit_principal_literal_document_resources(
    case: &LegacyCase,
    principal_bytes: &[u8],
    mut resources: Vec<WorkbenchResource>,
) -> Vec<WorkbenchResource> {
    const MAX_DISCOVERED_DOCUMENTS: usize = 16;
    const MAX_DISCOVERED_BYTES: usize = 8 * 1_048_576;

    if case.operation != "standard" {
        return resources;
    }
    let principal_identity = logical_identity(case, &case.principal_stylesheet);
    let authority_root = case.directory.parent().unwrap_or(&case.directory);
    let mut admitted = resources
        .iter()
        .map(|resource| resource.identity.clone())
        .collect::<BTreeSet<_>>();
    let mut discovered_documents = 0usize;
    let mut discovered_bytes = 0usize;
    for reference in stylesheet_literal_document_references(&principal_identity, principal_bytes) {
        if !is_parent_relative_document_reference(&reference) {
            continue;
        }
        if discovered_documents >= MAX_DISCOVERED_DOCUMENTS {
            break;
        }
        let Ok((resolved_identity, fragment)) =
            crate::resources::resolve_reference(&principal_identity, &reference)
        else {
            continue;
        };
        if fragment.is_some() || admitted.contains(&resolved_identity) {
            continue;
        }
        let Some(resolved_path) = resolve_case_relative_resource_file(
            &case.directory,
            authority_root,
            &case.principal_stylesheet,
            &reference,
        ) else {
            continue;
        };
        let Ok(bytes) = std::fs::read(resolved_path) else {
            continue;
        };
        let Some(next_bytes) = discovered_bytes.checked_add(bytes.len()) else {
            continue;
        };
        if next_bytes > MAX_DISCOVERED_BYTES {
            continue;
        }
        discovered_documents += 1;
        discovered_bytes = next_bytes;
        admitted.insert(resolved_identity.clone());
        resources.push(WorkbenchResource {
            identity: resolved_identity,
            bytes,
        });
    }
    resources
}

fn is_parent_relative_document_reference(reference: &str) -> bool {
    reference.starts_with("../")
}

fn stylesheet_literal_document_references(identity: &str, bytes: &[u8]) -> Vec<String> {
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
    let mut references = BTreeSet::new();
    let mut pending = vec![document.document_node()];
    while let Some(node) = pending.pop() {
        for attribute in document.attributes(node) {
            if let Some(value) = document.value(*attribute) {
                references.extend(literal_document_references(value));
            }
        }
        pending.extend(document.children(node));
    }
    references.into_iter().collect()
}

fn literal_document_references(expression: &str) -> Vec<String> {
    let mut remaining = expression;
    let mut references = Vec::new();
    while let Some((_, after_call)) = remaining.split_once("document(") {
        let Some(quote) = after_call.as_bytes().first().copied() else {
            break;
        };
        if !matches!(quote, b'\'' | b'"') {
            remaining = after_call;
            continue;
        }
        let Some(closing_quote) = after_call.as_bytes()[1..]
            .iter()
            .position(|byte| *byte == quote)
            .map(|offset| offset + 1)
        else {
            break;
        };
        if after_call[closing_quote + 1..].starts_with(')') {
            let reference = &after_call[1..closing_quote];
            if !reference.is_empty() {
                references.push(reference.to_owned());
            }
        }
        remaining = &after_call[closing_quote + 1..];
    }
    references
}

fn resolve_case_relative_resource_file(
    directory: &Path,
    authority_root: &Path,
    current: &str,
    reference: &str,
) -> Option<PathBuf> {
    if reference.contains(['#', '?']) || reference.split('/').next()?.contains(':') {
        return None;
    }
    let current = current.replace('/', std::path::MAIN_SEPARATOR_STR);
    let reference = reference.replace('/', std::path::MAIN_SEPARATOR_STR);
    let relative = Path::new(&current)
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(reference);
    let authority_root = authority_root.canonicalize().ok()?;
    let resolved = directory.join(relative).canonicalize().ok()?;
    resolved.strip_prefix(authority_root).ok()?;
    Some(resolved)
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

fn resolve_case_dependency_file(
    directory: &Path,
    current: &str,
    reference: &str,
    allow_historical_uri_alias: bool,
) -> Option<String> {
    let relative =
        case_dependency_physical_reference(current, reference, allow_historical_uri_alias)?;
    let root = directory.canonicalize().ok()?;
    let resolved = directory.join(&relative).canonicalize().ok()?;
    let relative = resolved.strip_prefix(root).ok()?;
    Some(relative.to_string_lossy().replace('\\', "/"))
}

fn case_dependency_physical_reference(
    current: &str,
    reference: &str,
    allow_historical_uri_alias: bool,
) -> Option<PathBuf> {
    if reference.contains(['#', '?']) {
        return None;
    }
    let physical_reference =
        if allow_historical_uri_alias && let Some(reference) = reference.strip_prefix("file:") {
            if reference.starts_with("//") {
                Path::new(reference).file_name()?.to_str()?.to_owned()
            } else {
                reference.to_owned()
            }
        } else if allow_historical_uri_alias
            && let Some((scheme, authority_and_path)) = reference.split_once("://")
        {
            if scheme.is_empty()
                || !scheme
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.'))
            {
                return None;
            }
            Path::new(
                authority_and_path
                    .replace('/', std::path::MAIN_SEPARATOR_STR)
                    .as_str(),
            )
            .file_name()?
            .to_str()?
            .to_owned()
        } else if reference.split('/').next()?.contains(':') {
            return None;
        } else {
            return Some(
                Path::new(current.replace('/', std::path::MAIN_SEPARATOR_STR).as_str())
                    .parent()
                    .unwrap_or_else(|| Path::new(""))
                    .join(reference.replace('/', std::path::MAIN_SEPARATOR_STR)),
            );
        };
    Some(
        Path::new(current.replace('/', std::path::MAIN_SEPARATOR_STR).as_str())
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(physical_reference.replace('/', std::path::MAIN_SEPARATOR_STR)),
    )
}

fn failure_frontier(failure: &WorkbenchFailure) -> String {
    let detail = failure.detail.as_str();
    if failure.code == "FXXM0001" {
        let stylesheet_input_shape = if detail.contains("DtdForbidden") {
            Some("stylesheet-input:dtd-forbidden")
        } else if detail.contains("cannot decode input using UTF-8") {
            Some("stylesheet-input:non-utf8")
        } else {
            None
        };
        if let Some(stylesheet_input_shape) = stylesheet_input_shape {
            return format!(
                "{}/{}/{stylesheet_input_shape}",
                failure.category, failure.code
            );
        }
    }
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
    selected_method: Option<&str>,
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
    let (actual_doctype, actual_content) = split_leading_doctype(actual_content)
        .map_err(|()| "actual-doctype-not-bounded".to_owned())?;
    let (expected_doctype, expected_content) = split_leading_doctype(expected_content)
        .map_err(|()| "expected-doctype-not-bounded".to_owned())?;
    let comparison_method = if selected_method.is_none() && has_lexical_html_root(actual_content) {
        Some("html")
    } else {
        selected_method
    };
    if !bounded_doctypes_equivalent(actual_doctype, expected_doctype, comparison_method) {
        return Ok(false);
    }
    if comparison_method == Some("html") {
        if let (Ok(actual), Ok(expected)) = (
            parse_comparison_document("actual", actual_content.trim()),
            parse_comparison_document("expected", expected_content.trim()),
        ) {
            if xml_nodes_equal(
                &actual,
                actual.document_node(),
                &expected,
                expected.document_node(),
                preserve_whitespace_only_text,
            ) {
                return Ok(true);
            }
        }
    }
    let actual_content = normalized_html_content(actual_content, comparison_method)
        .ok_or_else(|| "actual-html-normalization-failed".to_owned())?;
    let expected_content = normalized_html_content(expected_content, comparison_method)
        .ok_or_else(|| "expected-html-normalization-failed".to_owned())?;
    if let (Ok(actual), Ok(expected)) = (
        parse_comparison_document("actual", actual_content.trim()),
        parse_comparison_document("expected", expected_content.trim()),
    ) {
        return Ok(xml_nodes_equal(
            &actual,
            actual.document_node(),
            &expected,
            expected.document_node(),
            preserve_whitespace_only_text,
        ));
    }
    let actual = parse_comparison_fragment("actual", actual_content.trim());
    let expected = parse_comparison_fragment("expected", expected_content.trim());
    if actual.is_err() && expected.is_err() {
        let actual = normalize_inter_tag_whitespace(&actual_content);
        let expected = normalize_inter_tag_whitespace(&expected_content);
        return Ok(normalize_lexical_empty_elements(&actual)
            == normalize_lexical_empty_elements(&expected));
    }
    let actual = actual.map_err(|()| "actual-not-parseable-document-or-fragment".to_owned())?;
    let expected =
        expected.map_err(|()| "expected-not-parseable-document-or-fragment".to_owned())?;
    Ok(xml_nodes_equal(
        &actual,
        actual.document_node(),
        &expected,
        expected.document_node(),
        preserve_whitespace_only_text,
    ))
}

fn normalize_lexical_empty_elements(content: &str) -> Cow<'_, str> {
    let bytes = content.as_bytes();
    let mut normalized = String::with_capacity(content.len());
    let mut cursor = 0;
    let mut changed = false;
    while let Some(relative_open) = content[cursor..].find('<') {
        let open = cursor + relative_open;
        normalized.push_str(&content[cursor..open]);
        let Some(first) = bytes.get(open + 1).copied() else {
            normalized.push('<');
            cursor = open + 1;
            continue;
        };
        if matches!(first, b'/' | b'!' | b'?') {
            normalized.push('<');
            cursor = open + 1;
            continue;
        }
        let mut name_end = open + 1;
        while bytes.get(name_end).is_some_and(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'_' | b'-' | b'.')
        }) {
            name_end += 1;
        }
        if name_end == open + 1 {
            normalized.push('<');
            cursor = open + 1;
            continue;
        }
        let mut quote = None;
        let mut tag_end = name_end;
        while let Some(byte) = bytes.get(tag_end).copied() {
            if let Some(active) = quote {
                if byte == active {
                    quote = None;
                }
            } else if matches!(byte, b'\'' | b'"') {
                quote = Some(byte);
            } else if byte == b'>' {
                break;
            }
            tag_end += 1;
        }
        if bytes.get(tag_end) != Some(&b'>') || bytes.get(tag_end.wrapping_sub(1)) == Some(&b'/') {
            normalized.push('<');
            cursor = open + 1;
            continue;
        }
        let name = &content[open + 1..name_end];
        let closing = format!("</{name}>");
        if content[tag_end + 1..].starts_with(&closing) {
            normalized.push_str(&content[open..tag_end]);
            normalized.push_str("/>");
            cursor = tag_end + 1 + closing.len();
            changed = true;
        } else {
            normalized.push('<');
            cursor = open + 1;
        }
    }
    normalized.push_str(&content[cursor..]);
    if changed {
        Cow::Owned(normalized)
    } else {
        Cow::Borrowed(content)
    }
}

fn normalize_inter_tag_whitespace(content: &str) -> Cow<'_, str> {
    let mut normalized = String::with_capacity(content.len());
    let mut characters = content.chars().peekable();
    let mut changed = false;
    while let Some(character) = characters.next() {
        if character.is_ascii_whitespace() && normalized.ends_with('>') {
            let mut whitespace = String::from(character);
            while characters.peek().is_some_and(char::is_ascii_whitespace) {
                whitespace.push(characters.next().expect("peeked whitespace exists"));
            }
            if characters.peek() == Some(&'<') {
                changed = true;
                continue;
            }
            normalized.push_str(&whitespace);
            continue;
        }
        normalized.push(character);
    }
    if changed {
        Cow::Owned(normalized)
    } else {
        Cow::Borrowed(content)
    }
}

fn has_lexical_html_root(content: &str) -> bool {
    let content = content.trim_start();
    let Some(after_open) = content.get(1..) else {
        return false;
    };
    let Some(name) = after_open.get(..4) else {
        return false;
    };
    name.eq_ignore_ascii_case("html")
        && after_open
            .get(4..)
            .and_then(|tail| tail.chars().next())
            .is_some_and(|character| {
                character.is_ascii_whitespace() || matches!(character, '/' | '>')
            })
}

fn normalized_html_content<'a>(
    content: &'a str,
    selected_method: Option<&str>,
) -> Option<Cow<'a, str>> {
    if selected_method == Some("html") {
        normalize_for_xml_comparison(content).map(Cow::Owned)
    } else {
        Some(Cow::Borrowed(content))
    }
}

fn bounded_doctypes_equivalent(
    actual: Option<&str>,
    expected: Option<&str>,
    selected_method: Option<&str>,
) -> bool {
    if actual == expected {
        return true;
    }
    if selected_method != Some("html") {
        return false;
    }
    let Some((actual_name, actual_suffix)) = actual.and_then(split_doctype_name) else {
        return false;
    };
    let Some((expected_name, expected_suffix)) = expected.and_then(split_doctype_name) else {
        return false;
    };
    actual_name.eq_ignore_ascii_case(expected_name) && actual_suffix == expected_suffix
}

fn split_doctype_name(value: &str) -> Option<(&str, &str)> {
    let remainder = value.strip_prefix("<!DOCTYPE")?;
    let remainder = remainder.strip_prefix(char::is_whitespace)?;
    let name_end = remainder
        .find(|character: char| character.is_whitespace() || matches!(character, '[' | '>'))
        .unwrap_or(remainder.len());
    (name_end > 0).then(|| (&remainder[..name_end], &remainder[name_end..]))
}

fn split_leading_doctype(value: &str) -> Result<(Option<&str>, &str), ()> {
    let value = value.trim_start();
    if !value.starts_with("<!DOCTYPE") {
        return Ok((None, value));
    }
    let mut quote = None;
    let mut internal_subset_depth = 0usize;
    for (offset, character) in value.char_indices() {
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '[' => internal_subset_depth = internal_subset_depth.checked_add(1).ok_or(())?,
            ']' => internal_subset_depth = internal_subset_depth.checked_sub(1).ok_or(())?,
            '>' if internal_subset_depth == 0 => {
                let end = offset + character.len_utf8();
                return Ok((Some(&value[..end]), value[end..].trim_start()));
            }
            _ => {}
        }
    }
    Err(())
}

fn has_matching_bounded_doctype(
    actual: &str,
    expected: &[u8],
    selected_encoding: Option<&str>,
) -> bool {
    let Ok(expected) = decode_expected_xml(expected, selected_encoding) else {
        return false;
    };
    let actual = normalize_xml_source_line_endings(actual);
    let expected = normalize_xml_source_line_endings(&expected);
    let actual = strip_xml_declaration(&actual).trim();
    let expected = strip_xml_declaration(&expected).trim();
    matches!(
        (split_leading_doctype(actual), split_leading_doctype(expected)),
        (Ok((Some(actual), _)), Ok((Some(expected), _))) if actual == expected
    )
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
fn classifies_direct_dtd_declaration_pressure_without_scanning_document_content() {
    let properties = DtdProperties::inspect(
        br#"<!DOCTYPE root SYSTEM "sealed.dtd" [
            <!ENTITY local "value">
            <!ENTITY remote SYSTEM "remote.ent">
            <!ENTITY image SYSTEM "image.bin" NDATA png>
            <!ATTLIST root identity ID #REQUIRED label CDATA "default">
            <!NOTATION png SYSTEM "image/png">
        ]><root>ordinary ID text and <!ENTITY fake "content"></root>"#,
    );
    let properties = properties.named().collect::<BTreeSet<_>>();

    for expected in [
        "internal-subset",
        "external-identifier",
        "entity-declaration",
        "internal-general-entity-candidate",
        "external-general-entity-candidate",
        "unparsed-entity-candidate",
        "attribute-list-declaration",
        "default-attribute-candidate",
        "id-typing-candidate",
        "notation-declaration",
    ] {
        assert!(properties.contains(expected), "missing {expected}");
    }
    assert!(!properties.contains("parameter-entity-declaration"));
}

#[test]
fn oasis_xml_comparator_ignores_serialization_only_empty_element_and_prolog_spacing() {
    let actual = r#"<?xml version="1.0" encoding="UTF-8"?><out test="hello"></out>"#;
    let expected = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<out test=\"hello\"/>\r\n";

    assert_eq!(
        xml_equivalent(actual, expected, false, None, None),
        Ok(true)
    );
}

#[test]
fn oasis_xml_comparator_accepts_exact_normalized_non_xml_payloads_only() {
    let malformed = "<out>&bogus;</out>";
    assert_eq!(
        xml_equivalent(malformed, malformed.as_bytes(), false, None, None),
        Ok(true)
    );
    assert!(exact_normalized_non_xml_payloads(
        malformed,
        malformed.as_bytes(),
        None
    ));
    assert_ne!(
        xml_equivalent(malformed, b"<out>&different;</out>", false, None, None),
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

    assert_eq!(
        xml_equivalent(declaration_only, b"", false, None, None),
        Ok(true)
    );
    assert_eq!(
        xml_equivalent(declaration_only, b"<out/>", false, None, None),
        Ok(false)
    );
}

#[test]
fn oasis_xml_comparator_compares_generated_prefixes_by_expanded_name() {
    let actual = r#"<root xmlns:ns0="urn:example" ns0:value="kept"></root>"#;
    let expected = br#"<root xmlns:auto-ns1="urn:example" auto-ns1:value="kept"/>"#;

    assert_eq!(
        xml_equivalent(actual, expected, false, None, None),
        Ok(true)
    );
}

#[test]
fn oasis_xml_comparator_normalizes_literal_xml_line_endings_before_parsing() {
    assert_eq!(
        xml_equivalent("<out>a\nb</out>", b"<out>a\r\nb</out>", false, None, None),
        Ok(true)
    );
    assert_eq!(
        xml_equivalent("<out>&#13;</out>", b"<out>\r</out>", false, None, None),
        Ok(false)
    );
    let actual = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>far-north north near-north far-west west near-west center\nnear-south south near-south-west near-east east far-east </out>";
    let expected = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\r\n<out>far-north north near-north far-west west near-west center\r\nnear-south south near-south-west near-east east far-east </out>";
    assert_eq!(
        xml_equivalent(actual, expected, false, None, None),
        Ok(true)
    );
}

#[test]
fn oasis_xml_comparator_compares_a_bounded_doctype_before_parsing_the_document() {
    let actual = "<?xml version=\"1.0\"?><!DOCTYPE out PUBLIC \"a>b\" \"out.dtd\"><out></out>";
    let expected =
        b"<?xml version=\"1.0\"?>\r\n<!DOCTYPE out PUBLIC \"a>b\" \"out.dtd\">\r\n<out/>";

    assert_eq!(
        xml_equivalent(actual, expected, false, None, None),
        Ok(true)
    );
    assert_eq!(
        xml_equivalent(
            actual,
            b"<!DOCTYPE out SYSTEM \"different.dtd\"><out/>",
            false,
            None,
            None,
        ),
        Ok(false)
    );
    assert_eq!(
        split_leading_doctype("<!DOCTYPE out [<!ELEMENT out ANY>]><out/>")
            .map(|(doctype, body)| (doctype.map(str::to_owned), body.to_owned())),
        Ok((
            Some("<!DOCTYPE out [<!ELEMENT out ANY>]>".to_owned()),
            "<out/>".to_owned(),
        ))
    );
    assert!(split_leading_doctype("<!DOCTYPE out [<!ELEMENT out ANY><out/>").is_err());
}

#[test]
fn oasis_html_comparator_treats_only_the_doctype_name_as_ascii_case_insensitive() {
    let actual = "<!DOCTYPE html PUBLIC \"-//W3C//DTD HTML 4.0\"><html/>";
    let expected = b"<!DOCTYPE HTML PUBLIC \"-//W3C//DTD HTML 4.0\"><html/>";

    assert_eq!(
        xml_equivalent(actual, expected, false, None, Some("html")),
        Ok(true)
    );
    assert_eq!(
        xml_equivalent(actual, expected, false, None, Some("xml")),
        Ok(false)
    );
    assert_eq!(
        xml_equivalent(
            "<!DOCTYPE root PUBLIC \"-//W3C//DTD HTML 4.0\"><root/>",
            expected,
            false,
            None,
            Some("html"),
        ),
        Ok(false)
    );
}

#[test]
fn oasis_html_comparator_follows_unnamespaced_html_root_method_inference() {
    assert_eq!(
        xml_equivalent(
            r#"<HTML><META http-equiv="Content-Type" content="text/html; charset=UTF-8"></HTML>"#,
            br#"<html><meta http-equiv="Content-Type" content="text/html; charset=utf-8"></html>"#,
            false,
            None,
            None,
        ),
        Ok(true)
    );
    assert_eq!(
        xml_equivalent("<htmlish></htmlish>", b"<HTMLISH/>", false, None, None),
        Ok(false)
    );
}

#[test]
fn oasis_html_comparator_preserves_xml_readable_foreign_element_names() {
    assert_eq!(
        xml_equivalent(
            r#"<HTML xmlns="urn:foreign"><HR></HR><BR></BR></HTML>"#,
            br#"<HTML xmlns="urn:foreign"><HR/><BR/></HTML>"#,
            false,
            None,
            Some("html"),
        ),
        Ok(true)
    );
}

#[test]
fn oasis_xml_comparator_switches_indentation_whitespace_for_whitespace_groups() {
    let actual = "<outer>\n  <inner></inner>\n</outer>";
    let expected = b"<outer>\r\n<inner/>\r\n</outer>";

    assert_eq!(
        xml_equivalent(actual, expected, false, None, None),
        Ok(true)
    );
    assert_eq!(
        xml_equivalent(actual, expected, true, None, None),
        Ok(false)
    );
    assert_eq!(
        xml_equivalent("<out> </out>", b"<out>meaningful</out>", false, None, None,),
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
fn oasis_actual_decoder_admits_selected_bounded_legacy_bytes() {
    for (label, value, encoding) in [
        ("SHIFT_JIS", "日本語", SHIFT_JIS),
        ("BIG5", "中文", BIG5),
        ("ISO-2022-JP", "日本語", ISO_2022_JP),
    ] {
        let (bytes, _, had_errors) = encoding.encode(value);
        assert!(!had_errors);
        assert_eq!(
            decode_serialized_xml(&bytes, Some(label)),
            Ok(value.to_owned())
        );
        assert_eq!(
            decode_expected_xml(&bytes, Some(label)),
            Ok(value.to_owned())
        );
    }
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
    if let Some(encoding) = selected_encoding.and_then(bounded_legacy_encoding) {
        return decode_bounded_legacy(expected, encoding, "expected");
    }
    std::str::from_utf8(expected)
        .map(|text| text.strip_prefix('\u{feff}').unwrap_or(text).to_owned())
        .map_err(|_| "expected-not-utf8-or-utf16".to_owned())
}

fn decode_serialized_xml(actual: &[u8], selected_encoding: Option<&str>) -> Result<String, String> {
    if let Some(decoded) = decode_bom_utf16(actual) {
        return decoded.map_err(|endian| format!("actual-output-invalid-utf16{endian}"));
    }
    if let Some(encoding) = selected_encoding.and_then(bounded_legacy_encoding) {
        return decode_bounded_legacy(actual, encoding, "actual-output");
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

fn bounded_legacy_encoding(label: &str) -> Option<&'static Encoding> {
    if label.eq_ignore_ascii_case("SHIFT_JIS") || label.eq_ignore_ascii_case("SHIFT-JIS") {
        Some(SHIFT_JIS)
    } else if label.eq_ignore_ascii_case("BIG5") {
        Some(BIG5)
    } else if label.eq_ignore_ascii_case("ISO-2022-JP") {
        Some(ISO_2022_JP)
    } else if label.eq_ignore_ascii_case("WINDOWS-1252") {
        Some(WINDOWS_1252)
    } else {
        None
    }
}

fn decode_bounded_legacy(
    bytes: &[u8],
    encoding: &'static Encoding,
    label: &str,
) -> Result<String, String> {
    encoding
        .decode_without_bom_handling_and_without_replacement(bytes)
        .map(std::borrow::Cow::into_owned)
        .ok_or_else(|| format!("{label}-invalid-{}", encoding.name().to_ascii_lowercase()))
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
fn splits_oasis_stylesheet_input_policy_frontiers_from_other_xml_failures() {
    let failure = |detail: &str| WorkbenchFailure {
        code: "FXXM0001".to_owned(),
        category: "invalid".to_owned(),
        request_id: None,
        location: None,
        detail: detail.to_owned(),
    };

    assert_eq!(
        failure_frontier(&failure(
            "stylesheet XML is invalid: LocatedFailure { failure: DtdForbidden { span: 1..2 } }"
        )),
        "invalid/FXXM0001/stylesheet-input:dtd-forbidden"
    );
    assert_eq!(
        failure_frontier(&failure(
            "stylesheet XML is invalid: Malformed { detail: cannot decode input using UTF-8 }"
        )),
        "invalid/FXXM0001/stylesheet-input:non-utf8"
    );
    assert_eq!(
        failure_frontier(&failure("stylesheet XML is invalid: MultipleRoots")),
        "invalid/FXXM0001/FXXM0001"
    );
}

#[test]
fn maps_historical_oasis_dependency_uris_to_case_local_physical_files() {
    assert_eq!(
        case_dependency_physical_reference("principal.xsl", "file:fragments/included.xsl", true),
        Some(PathBuf::from("fragments/included.xsl"))
    );
    assert_eq!(
        case_dependency_physical_reference(
            "principal.xsl",
            "http://webxtest/testcases/included.xsl",
            true
        ),
        Some(PathBuf::from("included.xsl"))
    );
    assert_eq!(
        case_dependency_physical_reference(
            "styles/principal.xsl",
            "file://webxtest/testcases/included.xsl",
            true
        ),
        Some(PathBuf::from("styles/included.xsl"))
    );
    assert_eq!(
        case_dependency_physical_reference(
            "principal.xsl",
            "https://example.invalid/a.xsl#x",
            true
        ),
        None
    );
    assert_eq!(
        case_dependency_physical_reference(
            "principal.xsl",
            "http://webxtest/testcases/included.xsl",
            false
        ),
        None
    );
}

#[test]
fn discovers_only_single_argument_literal_document_references() {
    assert_eq!(
        literal_document_references(
            "document('a.xml')//body | document(\"b.xml\")/root | document($dynamic)"
        ),
        vec!["a.xml".to_owned(), "b.xml".to_owned()]
    );
    assert!(literal_document_references("document('a.xml', /)").is_empty());
    assert!(is_parent_relative_document_reference("../shared/data.xml"));
    assert!(!is_parent_relative_document_reference("local.xml"));
    assert!(!is_parent_relative_document_reference(
        "https://example.invalid/data.xml"
    ));
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
        "Output__77940",
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
        "Output__84165",
        "Output__84374",
        "Output__84429",
        "Output__84452",
        "Output__84453",
        "Output__84454",
        "Output__84455",
        "Output__84456",
        "Output__84457",
        "Output__84458",
        "Output__84459",
        "Output__84460",
        "Output__84461",
        "Output__84462",
        "Output_EmptyElement1",
        "Output_MethodEqualsHtmlWithoutIndentSet",
        "Output_UseLiteralResultElementHead",
        "output_output40",
        "output_output48",
        "output_output59",
        "Output_DoctypePublicAndSystemAttribute",
        "Output_DoctypePublicAttribute",
        "Output_DoctypeSystemAttribute",
        "Attributes__78365",
        "Attributes__78372",
        "AVTs__77574",
        "AVTs__77591",
        "BVTs_bvt029",
        "BVTs_bvt055",
        "BVTs_bvt057",
        "BVTs_bvt067",
        "BVTs_bvt098",
        "BVTs_bvt091",
        "BVTs_bvt083",
        "BVTs_bvt085",
        "ConflictResolution__77879",
        "Elements__78362",
        "Include__77515",
        "Include__77736",
        "Include_RelUriTest5",
        "Keys__91726",
        "Keys__91727",
        "Number__84683",
        "Number__84692",
        "Number__84722",
        "Number__91022",
        "Number__91027",
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
        "XSLTFunctions__minimalValue",
        "XSLTFunctions__minimumValue",
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
fn oasis_archival_reference_encoding_override_is_exact_and_bounded() {
    for case_id in [
        "Output__84374",
        "Output__84428",
        "Output__84429",
        "Sorting__77977",
    ] {
        assert_eq!(
            archival_reference_encoding(case_id, Some("UTF-8")),
            Some("windows-1252")
        );
    }
    assert_eq!(
        archival_reference_encoding("Output__84373", Some("UTF-8")),
        Some("UTF-8")
    );
}

#[test]
fn oasis_host_collation_policy_exclusion_is_exact_and_bounded() {
    for case_id in [
        "sort_sort08",
        "sort_sort27",
        "Sorting__77977",
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
        "BVTs_bvt063",
        "BVTs_bvt064",
        "Messages__91758",
        "Output__84260",
        "Output__84264",
        "Output__84271",
        "Output__84273",
        "Output__84277",
        "Output__84280",
        "Output__84282",
        "Output__84285",
        "Output__84309",
        "Output_EntityRefInAttribHtml",
        "Output_HtmlOutputWithLessThanInAttribute",
        "output_output36",
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
    for case_id in [
        "numbering_numbering79",
        "Number__84687",
        "Number__84700",
        "Number__91028",
        "Number__91029",
    ] {
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

#[test]
fn malformed_doe_reference_fallback_normalizes_only_layout_and_empty_elements() {
    let actual = "<xml><one></one><raw><<<P>>></raw></xml>";
    let expected = "<xml>\r\n<one/>\r\n<raw><<<P>>></raw>\r\n</xml>";
    let actual = normalize_inter_tag_whitespace(actual);
    let expected = normalize_inter_tag_whitespace(expected);

    assert_eq!(
        normalize_lexical_empty_elements(&actual),
        normalize_lexical_empty_elements(&expected)
    );
    assert_ne!(
        normalize_lexical_empty_elements("<raw><<A>></raw>"),
        normalize_lexical_empty_elements("<raw><<B>></raw>")
    );
}
