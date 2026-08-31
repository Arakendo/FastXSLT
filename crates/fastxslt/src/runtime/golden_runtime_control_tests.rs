//! Phase-specific cancellation tests for the private golden runtime.

use std::{
    collections::{BTreeMap, HashSet},
    fmt::Write as _,
    mem::size_of,
    sync::Arc,
    time::Instant,
};

use super::{
    ExecutionPolicy, FailureCategory, InvocationEntry, ResultNode, TransformRequest,
    TransformSetBuilder, append_text, compile_resource, execute_transform_set,
};
use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document_controlled};

const SOURCE_ID: &str = "urn:fastxslt:golden:hello:input";
const STYLESHEET_ID: &str = "urn:fastxslt:golden:hello:stylesheet";
const FANOUT_SOURCE_ID: &str = "urn:fastxslt:template-fanout:source";
const FANOUT_STYLESHEET_ID: &str = "urn:fastxslt:template-fanout:stylesheet";
const ROOTED_MATCH_SOURCE_ID: &str = "urn:fastxslt:rooted-match:source";
const ROOTED_MATCH_STYLESHEET_ID: &str = "urn:fastxslt:rooted-match:stylesheet";
const GLOBAL_CLONE_STYLESHEET_ID: &str = "urn:fastxslt:global-clone:stylesheet";

fn snapshot() -> crate::resources::ResourceSnapshot {
    let source = include_bytes!("../../../../corpus/golden/hello/input.xml").to_vec();
    let stylesheet = include_bytes!("../../../../corpus/golden/hello/stylesheet.xsl").to_vec();
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE_ID, source)
        .expect("admit golden source");
    resources
        .admit(STYLESHEET_ID, stylesheet)
        .expect("admit golden stylesheet");
    resources.seal()
}

fn request(
    identity: &str,
    result_identity: &str,
    cancellation_fault: Option<(WorkDomain, usize)>,
) -> TransformRequest {
    TransformRequest {
        identity: identity.to_owned(),
        result_identity: result_identity.to_owned(),
        entry: InvocationEntry::PrincipalSource {
            resource: SOURCE_ID.to_owned(),
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault,
    }
}

fn policy() -> ExecutionPolicy {
    ExecutionPolicy {
        denied_sources: HashSet::new(),
        serialized_byte_limit: 4_096,
        work_limits: WorkLimits::unbounded(),
    }
}

#[test]
fn cancellation_after_partial_work_retains_phase_and_request_identity() {
    let cases = [
        (WorkDomain::XmlEvent, 2),
        (WorkDomain::XdmNode, 2),
        (WorkDomain::XsltInstruction, 1),
        (WorkDomain::XPathNodeVisit, 1),
        (WorkDomain::XdmStringValueNode, 1),
        (WorkDomain::ResultNode, 1),
        (WorkDomain::ResultTextByte, 0),
        (WorkDomain::SerializedByte, 2),
    ];

    for (domain, accepted_charges_before_signal) in cases {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile golden program");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy());
        let request_id = format!("cancel-at-{}", domain.name());
        builder
            .add(request(
                &request_id,
                "cancelled-result",
                Some((domain, accepted_charges_before_signal)),
            ))
            .expect("admit fault-injected request");

        let failure = execute_transform_set(builder.seal())
            .expect_err("phase-specific cancellation should stop execution");

        assert_eq!(failure.code, "FXCT0001");
        assert_eq!(failure.category, FailureCategory::Cancelled);
        assert_eq!(failure.request_id.as_deref(), Some(request_id.as_str()));
        assert_eq!(failure.work_domain, Some(domain));
    }
}

#[test]
fn a_completed_sibling_is_not_exposed_when_the_set_returns_cancellation() {
    let snapshot = snapshot();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile golden program");
    let mut builder = TransformSetBuilder::new(snapshot, program, 2, policy());
    builder
        .add(request(
            "cancelled-after-sibling",
            "cancelled-result",
            Some((WorkDomain::SerializedByte, 2)),
        ))
        .expect("admit request that will cancel");
    builder
        .add(request("completed-first", "completed-result", None))
        .expect("admit healthy sibling");

    // The private reference executor runs in reverse admission order, so the
    // healthy sibling completes before cancellation is injected. The operation
    // still returns only the structured failure and no partial ResultSet.
    let failure = execute_transform_set(builder.seal())
        .expect_err("the private set operation is all-result or failure");

    assert_eq!(failure.code, "FXCT0001");
    assert_eq!(failure.category, FailureCategory::Cancelled);
    assert_eq!(
        failure.request_id.as_deref(),
        Some("cancelled-after-sibling")
    );
    assert_eq!(failure.work_domain, Some(WorkDomain::SerializedByte));
}

#[test]
fn result_nodes_and_utf8_text_bytes_are_bounded_before_serialization() {
    let mut limits = WorkLimits::unbounded();
    limits.result_nodes = 1;
    limits.result_text_bytes = 4;
    limits.serialized_bytes = 0;
    let mut control = InvocationControl::new(CancellationToken::new(), limits);
    let mut nodes = Vec::new();

    append_text(&mut nodes, "🚀", "result-growth", &mut control)
        .expect("one node and four UTF-8 bytes should fit exactly");
    assert_eq!(nodes, [ResultNode::Text("🚀".to_owned())]);

    let failure = append_text(&mut nodes, "!", "result-growth", &mut control)
        .expect_err("semantic result text should be bounded before serialization");
    assert_eq!(failure.code, "FXCT0002");
    assert_eq!(failure.category, FailureCategory::Limit);
    assert_eq!(failure.request_id.as_deref(), Some("result-growth"));
    assert_eq!(failure.work_domain, Some(WorkDomain::ResultTextByte));
}

#[test]
fn golden_path_has_an_exact_attributable_charge_profile() {
    let snapshot = snapshot();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile golden program");
    let mut control = InvocationControl::unbounded();
    let parsed = parse_document_controlled(
        SOURCE_ID,
        snapshot.get(SOURCE_ID).expect("golden source"),
        ParseLimits {
            max_events: 1_024,
            max_depth: 64,
        },
        &mut control,
    )
    .expect("parse controlled golden source");
    let source = Document::from_parsed_controlled(parsed, &mut control)
        .expect("construct controlled golden XDM");
    let semantic = super::execute_program(&program, &source, "charge-profile", &mut control)
        .expect("execute controlled golden program");
    let serialized = super::serialize_xml(
        &semantic,
        &program.output,
        "charge-profile",
        4_096,
        &mut control,
    )
    .expect("serialize controlled golden result");
    let profile: Vec<_> = [
        WorkDomain::XmlEvent,
        WorkDomain::XdmNode,
        WorkDomain::XsltInstruction,
        WorkDomain::XPathNodeVisit,
        WorkDomain::XPathOperation,
        WorkDomain::XdmStringValueNode,
        WorkDomain::ResultNode,
        WorkDomain::ResultTextByte,
        WorkDomain::SerializedByte,
    ]
    .into_iter()
    .map(|domain| (domain.name(), control.consumed(domain)))
    .collect();

    assert_eq!(serialized, "<message>Hello, FastXSLT!</message>");
    assert_eq!(
        profile,
        [
            ("xml-event", 10),
            ("xdm-node", 6),
            ("xslt-instruction", 4),
            ("xpath-node-visit", 4),
            ("xpath-operation", 0),
            ("xdm-string-value-node", 2),
            ("result-node", 2),
            ("result-text-byte", 16),
            ("serialized-byte", 35),
        ]
    );
}

fn template_fanout_workload(
    template_count: usize,
    source_nodes: usize,
) -> (
    crate::xslt::golden_semantics_experiment::StylesheetProgram,
    Document,
) {
    let mut stylesheet = String::from(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/"><xsl:apply-templates select="root/item"/></xsl:template>"#,
    );
    for index in 0..template_count {
        write!(stylesheet, r#"<xsl:template match="miss{index}"/>"#)
            .expect("write generated template");
    }
    stylesheet.push_str(r#"<xsl:template match="item"><out/></xsl:template></xsl:stylesheet>"#);
    let source = format!("<root>{}</root>", "<item/>".repeat(source_nodes));
    let total_bytes = source.len() + stylesheet.len();
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, total_bytes, total_bytes));
    resources
        .admit(FANOUT_SOURCE_ID, source.into_bytes())
        .expect("admit fanout source");
    resources
        .admit(FANOUT_STYLESHEET_ID, stylesheet.into_bytes())
        .expect("admit fanout stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, FANOUT_STYLESHEET_ID).expect("compile fanout stylesheet");
    let mut preparation = InvocationControl::unbounded();
    let parsed = parse_document_controlled(
        FANOUT_SOURCE_ID,
        snapshot.get(FANOUT_SOURCE_ID).expect("fanout source bytes"),
        ParseLimits {
            max_events: source_nodes * 2 + 8,
            max_depth: 8,
        },
        &mut preparation,
    )
    .expect("parse fanout source");
    let document =
        Document::from_parsed_controlled(parsed, &mut preparation).expect("construct fanout XDM");
    (program, document)
}

fn document_rooted_match_workload(
    source_nodes: usize,
) -> (
    crate::xslt::golden_semantics_experiment::StylesheetProgram,
    Document,
) {
    let stylesheet = br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/"><xsl:apply-templates select="root/item"/></xsl:template><xsl:template match="/root/item"><out/></xsl:template></xsl:stylesheet>"#.to_vec();
    let source = format!("<root>{}</root>", "<item/>".repeat(source_nodes)).into_bytes();
    let total_bytes = source.len() + stylesheet.len();
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, total_bytes, total_bytes));
    resources
        .admit(ROOTED_MATCH_SOURCE_ID, source)
        .expect("admit rooted-match source");
    resources
        .admit(ROOTED_MATCH_STYLESHEET_ID, stylesheet)
        .expect("admit rooted-match stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, ROOTED_MATCH_STYLESHEET_ID)
        .expect("compile rooted-match stylesheet");
    let mut preparation = InvocationControl::unbounded();
    let parsed = parse_document_controlled(
        ROOTED_MATCH_SOURCE_ID,
        snapshot
            .get(ROOTED_MATCH_SOURCE_ID)
            .expect("rooted-match source bytes"),
        ParseLimits {
            max_events: source_nodes * 2 + 8,
            max_depth: 8,
        },
        &mut preparation,
    )
    .expect("parse rooted-match source");
    let document = Document::from_parsed_controlled(parsed, &mut preparation)
        .expect("construct rooted-match XDM");
    (program, document)
}

fn global_clone_workload(
    global_count: usize,
    call_depth: usize,
) -> crate::xslt::golden_semantics_experiment::StylesheetProgram {
    let mut stylesheet = String::from(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">"#,
    );
    for index in 0..global_count {
        write!(
            stylesheet,
            r#"<xsl:variable name="g{index}" select="{index}"/>"#
        )
        .expect("write generated global");
    }
    for index in 0..call_depth {
        write!(
            stylesheet,
            r#"<xsl:template name="t{index}"><xsl:call-template name="t{}"/></xsl:template>"#,
            index + 1
        )
        .expect("write generated named-template call");
    }
    write!(
        stylesheet,
        r#"<xsl:template name="t{call_depth}"><out/></xsl:template></xsl:stylesheet>"#
    )
    .expect("finish generated stylesheet");

    let bytes = stylesheet.into_bytes();
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, bytes.len(), bytes.len()));
    resources
        .admit(GLOBAL_CLONE_STYLESHEET_ID, bytes)
        .expect("admit global-clone stylesheet");
    compile_resource(&resources.seal(), GLOBAL_CLONE_STYLESHEET_ID)
        .expect("compile global-clone stylesheet")
}

#[test]
fn named_template_calls_clone_every_global_atomic_entry() {
    let global_count = 16;
    let call_depth = 8;
    let program = global_clone_workload(global_count, call_depth);
    let mut control = InvocationControl::unbounded();

    let result = super::execute_initial_template(
        &program,
        "t0",
        super::MultipleMatchPolicy::UseLast,
        "global-clone",
        &mut control,
    )
    .expect("execute global-clone workload");

    assert_eq!(result.children.len(), 1);
    assert_eq!(
        control.global_atomic_frame_clone_observation(),
        (call_depth, global_count * call_depth)
    );
}

#[cfg(feature = "allocation-observation")]
#[test]
#[ignore = "release-mode global-frame clone allocation measurement probe"]
fn measure_named_template_global_frame_cloning() {
    const CALL_DEPTH: usize = 8;
    for global_count in [0, 16, 64, 256] {
        let baseline_program = global_clone_workload(global_count, 0);
        let program = global_clone_workload(global_count, CALL_DEPTH);
        let mut baseline_samples = Vec::with_capacity(5);
        let mut samples = Vec::with_capacity(5);
        let mut observed = None;
        for _ in 0..5 {
            let mut baseline_control = InvocationControl::unbounded();
            let baseline_started = Instant::now();
            let baseline_result = super::execute_initial_template(
                &baseline_program,
                "t0",
                super::MultipleMatchPolicy::UseLast,
                "global-clone-baseline",
                &mut baseline_control,
            )
            .expect("execute global-clone baseline");
            assert_eq!(baseline_result.children.len(), 1);
            baseline_samples.push(baseline_started.elapsed().as_secs_f64() * 1_000_000.0);

            let mut control = InvocationControl::unbounded();
            let started = Instant::now();
            let result = super::execute_initial_template(
                &program,
                "t0",
                super::MultipleMatchPolicy::UseLast,
                "global-clone-measurement",
                &mut control,
            )
            .expect("execute measured global-clone workload");
            assert_eq!(result.children.len(), 1);
            samples.push(started.elapsed().as_secs_f64() * 1_000_000.0);
            observed = Some(control.global_atomic_frame_clone_observation());
        }
        baseline_samples.sort_by(f64::total_cmp);
        samples.sort_by(f64::total_cmp);

        let mut baseline_allocation_control = InvocationControl::unbounded();
        let baseline_allocations = allocation_counter::measure(|| {
            let result = super::execute_initial_template(
                &baseline_program,
                "t0",
                super::MultipleMatchPolicy::UseLast,
                "global-clone-allocation-baseline",
                &mut baseline_allocation_control,
            )
            .expect("execute allocation-observed global-clone baseline");
            assert_eq!(result.children.len(), 1);
        });
        let mut allocation_control = InvocationControl::unbounded();
        let allocations = allocation_counter::measure(|| {
            let result = super::execute_initial_template(
                &program,
                "t0",
                super::MultipleMatchPolicy::UseLast,
                "global-clone-allocation",
                &mut allocation_control,
            )
            .expect("execute allocation-observed global-clone workload");
            assert_eq!(result.children.len(), 1);
        });
        println!(
            "globals={global_count} call_depth={CALL_DEPTH} clone_observation={:?} baseline_median_us={:.3} chain_median_us={:.3} baseline_allocations={:?} chain_allocations={:?}",
            observed.expect("one clone observation"),
            baseline_samples[baseline_samples.len() / 2],
            samples[samples.len() / 2],
            baseline_allocations,
            allocations
        );
    }
}

#[test]
fn document_rooted_match_cache_builds_once_for_the_invocation() {
    let source_nodes = 16;
    let (program, source) = document_rooted_match_workload(source_nodes);
    let mut control = InvocationControl::unbounded();

    let result = super::execute_program(&program, &source, "rooted-match", &mut control)
        .expect("execute rooted-match workload");

    assert_eq!(result.children.len(), source_nodes);
    assert_eq!(control.document_rooted_match_evaluations(), 1);
    assert_eq!(
        control.document_rooted_match_cache_observation(),
        (
            1,
            source_nodes - 1,
            source.node_count().div_ceil(u64::BITS as usize) * size_of::<u64>()
        )
    );
    assert_eq!(
        control.consumed(WorkDomain::XPathNodeVisit),
        (source_nodes + 1) * 2
    );
}

#[test]
fn concurrent_rooted_match_caches_remain_invocation_owned() {
    let source_nodes = 32;
    let (program, source) = document_rooted_match_workload(source_nodes);
    let program = Arc::new(program);
    let source = Arc::new(source);

    let observations = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4)
            .map(|worker| {
                let program = Arc::clone(&program);
                let source = Arc::clone(&source);
                scope.spawn(move || {
                    let mut control = InvocationControl::unbounded();
                    let result = super::execute_program(
                        &program,
                        &source,
                        &format!("rooted-match-concurrent-{worker}"),
                        &mut control,
                    )
                    .expect("execute concurrent rooted-match workload");
                    assert_eq!(result.children.len(), source_nodes);
                    control.document_rooted_match_cache_observation()
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| worker.join().expect("rooted-match worker completes"))
            .collect::<Vec<_>>()
    });

    assert!(
        observations
            .iter()
            .all(|&(builds, hits, _)| builds == 1 && hits == source_nodes - 1)
    );
}

#[test]
#[ignore = "release-mode document-rooted match-path measurement probe"]
fn measure_document_rooted_match_path_reevaluation() {
    for source_nodes in [8, 32, 128, 256] {
        let (program, source) = document_rooted_match_workload(source_nodes);
        let mut reference_samples = Vec::with_capacity(5);
        let mut cached_samples = Vec::with_capacity(5);
        let mut reference_observed = None;
        let mut cached_observed = None;
        for _ in 0..5 {
            let mut reference_control =
                InvocationControl::unbounded().without_document_rooted_match_cache();
            let reference_started = Instant::now();
            let reference = super::execute_program(
                &program,
                &source,
                "rooted-match-reference",
                &mut reference_control,
            )
            .expect("execute rooted-match reference");
            reference_samples.push(reference_started.elapsed().as_secs_f64() * 1_000_000.0);
            reference_observed = Some((
                reference_control.document_rooted_match_evaluations(),
                reference_control.consumed(WorkDomain::XPathNodeVisit),
            ));

            let mut cached_control = InvocationControl::unbounded();
            let cached_started = Instant::now();
            let cached = super::execute_program(
                &program,
                &source,
                "rooted-match-cached",
                &mut cached_control,
            )
            .expect("execute cached rooted-match workload");
            cached_samples.push(cached_started.elapsed().as_secs_f64() * 1_000_000.0);
            cached_observed = Some((
                cached_control.document_rooted_match_evaluations(),
                cached_control.consumed(WorkDomain::XPathNodeVisit),
                cached_control.document_rooted_match_cache_observation(),
            ));
            assert_eq!(reference, cached);
        }
        reference_samples.sort_by(f64::total_cmp);
        cached_samples.sort_by(f64::total_cmp);
        let (reference_evaluations, reference_visits) =
            reference_observed.expect("one reference observation");
        let (cached_evaluations, cached_visits, cache) =
            cached_observed.expect("one cache observation");

        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = cached_visits - 1;
        let mut limited = InvocationControl::new(CancellationToken::new(), limits);
        let failure =
            super::execute_program(&program, &source, "rooted-match-budget", &mut limited)
                .expect_err("one fewer node visit must exhaust cached construction");
        assert_eq!(failure.category, FailureCategory::Limit);
        assert_eq!(failure.work_domain, Some(WorkDomain::XPathNodeVisit));

        #[cfg(feature = "allocation-observation")]
        let allocation_summary = {
            let mut reference_control =
                InvocationControl::unbounded().without_document_rooted_match_cache();
            let reference_allocations = allocation_counter::measure(|| {
                super::execute_program(
                    &program,
                    &source,
                    "rooted-match-reference-allocation",
                    &mut reference_control,
                )
                .expect("execute allocation-observed rooted-match reference");
            });
            let mut cached_control = InvocationControl::unbounded();
            let cached_allocations = allocation_counter::measure(|| {
                super::execute_program(
                    &program,
                    &source,
                    "rooted-match-cache-allocation",
                    &mut cached_control,
                )
                .expect("execute allocation-observed rooted-match cache");
            });
            format!(
                " reference_allocations={reference_allocations:?} cached_allocations={cached_allocations:?}"
            )
        };
        #[cfg(not(feature = "allocation-observation"))]
        let allocation_summary = String::new();

        println!(
            "source_nodes={source_nodes} reference_evaluations={reference_evaluations} reference_visits={reference_visits} cached_evaluations={cached_evaluations} cached_visits={cached_visits} cache={cache:?} budget_exhaustion_limit={} reference_median_us={:.3} cached_median_us={:.3}{allocation_summary}",
            cached_visits - 1,
            reference_samples[reference_samples.len() / 2],
            cached_samples[cached_samples.len() / 2]
        );
    }
}

#[test]
fn template_candidate_fanout_is_charged_in_its_own_domain() {
    let (program, source) = template_fanout_workload(32, 16);
    let templates = program.matched_templates.len();
    let mut control = InvocationControl::unbounded();

    let result = super::execute_program(&program, &source, "template-fanout", &mut control)
        .expect("execute fanout workload");

    assert_eq!(result.children.len(), 16);
    assert_eq!(templates, 33);
    assert_eq!(
        control.template_candidate_observation(),
        (templates * 16, 1)
    );
    assert_eq!(
        control.consumed(WorkDomain::XsltTemplateCandidate),
        templates * 16
    );
    assert_eq!(control.consumed(WorkDomain::XsltInstruction), 33);
}

#[test]
fn cancellation_signalled_during_simple_pattern_scan_stops_at_the_candidate_charge() {
    let (program, source) = template_fanout_workload(128, 1);
    let mut control = InvocationControl::unbounded().cancelling_after_template_candidates(1);

    let failure = super::execute_program(
        &program,
        &source,
        "template-fanout-cancellation-gap",
        &mut control,
    )
    .expect_err("the literal-result instruction should observe cancellation after selection");

    assert_eq!(failure.code, "FXCT0001");
    assert_eq!(failure.category, FailureCategory::Cancelled);
    assert_eq!(failure.work_domain, Some(WorkDomain::XsltTemplateCandidate));
    assert_eq!(control.template_candidate_observation(), (1, 1));
    assert_eq!(control.template_candidates_after_cancellation_signal(), 0);
}

#[test]
fn template_candidate_limit_stops_before_the_first_pattern_test() {
    let (program, source) = template_fanout_workload(8, 1);
    let mut limits = WorkLimits::unbounded();
    limits.xslt_template_candidates = 0;
    let mut control = InvocationControl::new(CancellationToken::new(), limits);

    let failure =
        super::execute_program(&program, &source, "template-candidate-limit", &mut control)
            .expect_err("zero template-candidate work must stop selection");

    assert_eq!(failure.code, "FXCT0002");
    assert_eq!(failure.category, FailureCategory::Limit);
    assert_eq!(failure.work_domain, Some(WorkDomain::XsltTemplateCandidate));
    assert_eq!(control.template_candidate_observation(), (1, 1));
}

#[test]
#[ignore = "release-mode measurement probe"]
fn measure_template_candidate_fanout() {
    for template_count in [8, 32, 128] {
        for source_nodes in [8, 64, 256] {
            let (program, source) = template_fanout_workload(template_count, source_nodes);
            let mut uncharged_samples = Vec::with_capacity(5);
            let mut charged_samples = Vec::with_capacity(5);
            let mut observed = None;
            for _ in 0..5 {
                let mut uncharged_control =
                    InvocationControl::unbounded().without_template_candidate_charging();
                let uncharged_started = Instant::now();
                let uncharged_result = super::execute_program(
                    &program,
                    &source,
                    "template-fanout-uncharged",
                    &mut uncharged_control,
                )
                .expect("execute uncharged reference fanout workload");
                assert_eq!(uncharged_result.children.len(), source_nodes);
                uncharged_samples.push(uncharged_started.elapsed().as_secs_f64() * 1_000_000.0);

                let mut control = InvocationControl::unbounded();
                let started = Instant::now();
                let result = super::execute_program(
                    &program,
                    &source,
                    "template-fanout-measurement",
                    &mut control,
                )
                .expect("execute measured fanout workload");
                assert_eq!(result.children.len(), source_nodes);
                charged_samples.push(started.elapsed().as_secs_f64() * 1_000_000.0);
                observed = Some(control.template_candidate_observation());
            }
            uncharged_samples.sort_by(f64::total_cmp);
            charged_samples.sort_by(f64::total_cmp);
            let (candidates, maximum_gap) = observed.expect("one observation");
            println!(
                "templates={} source_nodes={} candidates={} maximum_gap={} uncharged_median_us={:.3} charged_median_us={:.3}",
                program.matched_templates.len(),
                source_nodes,
                candidates,
                maximum_gap,
                uncharged_samples[uncharged_samples.len() / 2],
                charged_samples[charged_samples.len() / 2]
            );
        }
    }
}
