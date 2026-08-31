//! Private AR-0009 experiment for explicit prepared-input ownership and reuse.

use std::{collections::BTreeMap, sync::Arc};

use crate::execution_control_experiment::{ControlFailure, InvocationControl};
use crate::resources::ResourceSnapshot;
use crate::xdm::owned_tree_experiment::{BuildFailure, Document, SourceLocation};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document_controlled};

#[cfg(test)]
const PREPARATION_XML_LIMITS: ParseLimits = ParseLimits {
    max_events: 1_024,
    max_depth: 64,
};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum PreparationFailure {
    MissingResource {
        identity: String,
    },
    DuplicateResource {
        identity: String,
    },
    InvalidXml {
        identity: String,
        location: SourceLocation,
        detail: String,
    },
    InvalidXdm {
        identity: String,
        detail: String,
    },
    Control(ControlFailure),
}

#[derive(Debug)]
pub(super) struct PreparedInputBuilder {
    snapshot: ResourceSnapshot,
    parse_limits: ParseLimits,
    documents: BTreeMap<String, Arc<Document>>,
    parsed_phase_capacity_bytes: BTreeMap<String, usize>,
}

impl PreparedInputBuilder {
    #[cfg(test)]
    pub(super) fn new(snapshot: ResourceSnapshot) -> Self {
        Self {
            snapshot,
            parse_limits: PREPARATION_XML_LIMITS,
            documents: BTreeMap::new(),
            parsed_phase_capacity_bytes: BTreeMap::new(),
        }
    }

    pub(super) fn with_parse_limits(snapshot: ResourceSnapshot, parse_limits: ParseLimits) -> Self {
        Self {
            snapshot,
            parse_limits,
            documents: BTreeMap::new(),
            parsed_phase_capacity_bytes: BTreeMap::new(),
        }
    }

    pub(super) fn prepare(
        &mut self,
        identity: &str,
        control: &mut InvocationControl,
    ) -> Result<(), PreparationFailure> {
        if self.documents.contains_key(identity) {
            return Err(PreparationFailure::DuplicateResource {
                identity: identity.to_owned(),
            });
        }
        let bytes =
            self.snapshot
                .get(identity)
                .ok_or_else(|| PreparationFailure::MissingResource {
                    identity: identity.to_owned(),
                })?;
        let parsed = parse_document_controlled(identity, bytes, self.parse_limits, control)
            .map_err(|failure| match failure.control_failure() {
                Some(failure) => PreparationFailure::Control(*failure),
                None => PreparationFailure::InvalidXml {
                    identity: identity.to_owned(),
                    location: SourceLocation {
                        resource: identity.to_owned(),
                        span: failure
                            .source_span()
                            .expect("non-control XML failures must own a source span"),
                    },
                    detail: format!("{failure:?}"),
                },
            })?;
        let parsed_phase_capacity_bytes = parsed.owned_capacity_bytes();
        let document =
            Document::from_parsed_controlled(parsed, control).map_err(|failure| match failure {
                BuildFailure::Control(failure) => PreparationFailure::Control(failure),
                _ => PreparationFailure::InvalidXdm {
                    identity: identity.to_owned(),
                    detail: format!("{failure:?}"),
                },
            })?;
        self.documents
            .insert(identity.to_owned(), Arc::new(document));
        self.parsed_phase_capacity_bytes
            .insert(identity.to_owned(), parsed_phase_capacity_bytes);
        Ok(())
    }

    pub(super) fn seal(self) -> PreparedInputSet {
        PreparedInputSet {
            #[cfg(test)]
            snapshot: self.snapshot,
            documents: Arc::new(self.documents),
            #[cfg(test)]
            parsed_phase_capacity_bytes: Arc::new(self.parsed_phase_capacity_bytes),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct PreparedInputSet {
    #[cfg(test)]
    snapshot: ResourceSnapshot,
    documents: Arc<BTreeMap<String, Arc<Document>>>,
    #[cfg(test)]
    parsed_phase_capacity_bytes: Arc<BTreeMap<String, usize>>,
}

#[cfg(feature = "workbench")]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct PreparedRetentionObservation {
    pub(super) document_count: usize,
    pub(super) xdm_node_count: usize,
    pub(super) prepared_map_known_capacity_bytes: usize,
    pub(super) xdm_owned_capacity_bytes: usize,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct PreparedInputObservation {
    raw_bytes: usize,
    parsed_phase_owned_capacity_bytes: usize,
    xdm_nodes: usize,
    xdm_owned_capacity_bytes: usize,
}

impl PreparedInputSet {
    #[cfg(test)]
    fn belongs_to(&self, snapshot: &ResourceSnapshot) -> bool {
        self.snapshot.same_generation(snapshot)
    }

    pub(super) fn get(&self, identity: &str) -> Option<Arc<Document>> {
        self.documents.get(identity).cloned()
    }

    #[cfg(test)]
    pub(super) fn test_only_snapshot_known_capacity_bytes(&self) -> usize {
        self.snapshot.test_only_known_capacity_bytes()
    }

    #[cfg(feature = "workbench")]
    pub(super) fn retention_observation(&self) -> PreparedRetentionObservation {
        let prepared_map_known_capacity_bytes =
            std::mem::size_of::<BTreeMap<String, Arc<Document>>>()
                + self.documents.len()
                    * (std::mem::size_of::<String>() + std::mem::size_of::<Arc<Document>>())
                + self.documents.keys().map(String::capacity).sum::<usize>();
        PreparedRetentionObservation {
            document_count: self.documents.len(),
            xdm_node_count: self
                .documents
                .values()
                .map(|document| document.node_count())
                .sum(),
            prepared_map_known_capacity_bytes,
            xdm_owned_capacity_bytes: self
                .documents
                .values()
                .map(|document| document.owned_capacity_bytes())
                .sum(),
        }
    }

    #[cfg(test)]
    fn observe(&self, identity: &str) -> Option<PreparedInputObservation> {
        let raw_bytes = self.snapshot.get(identity)?.len();
        let document = self.documents.get(identity)?;
        let parsed_phase_owned_capacity_bytes = *self.parsed_phase_capacity_bytes.get(identity)?;
        Some(PreparedInputObservation {
            raw_bytes,
            parsed_phase_owned_capacity_bytes,
            xdm_nodes: document.node_count(),
            xdm_owned_capacity_bytes: document.owned_capacity_bytes(),
        })
    }

    #[cfg(test)]
    fn observe_totals(&self) -> PreparedInputObservation {
        self.documents
            .keys()
            .filter_map(|identity| self.observe(identity))
            .fold(PreparedInputObservation::default(), |mut total, item| {
                total.raw_bytes += item.raw_bytes;
                total.parsed_phase_owned_capacity_bytes += item.parsed_phase_owned_capacity_bytes;
                total.xdm_nodes += item.xdm_nodes;
                total.xdm_owned_capacity_bytes += item.xdm_owned_capacity_bytes;
                total
            })
    }
}

#[cfg(test)]
#[path = "prepared_input_representative_lifecycle_tests.rs"]
mod representative_lifecycle_tests;

#[cfg(test)]
mod tests {
    use std::{
        hint::black_box,
        sync::{Arc, Barrier},
        time::Instant,
    };

    use crate::execution_control_experiment::{
        CancellationToken, ControlFailure, InvocationControl, WorkDomain, WorkLimits,
    };
    use crate::resources::{ResourceLimits, ResourceSetBuilder, ResourceSnapshot};
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    use super::{PreparationFailure, PreparedInputBuilder};
    use crate::runtime::golden_runtime_experiment::{
        compile_resource, execute_program, serialize_xml,
    };

    const SOURCE_A: &str = "urn:fastxslt:prepared:source-a";
    const SOURCE_B: &str = "urn:fastxslt:prepared:source-b";
    const STYLE_A: &str = "urn:fastxslt:prepared:style-a";
    const STYLE_B: &str = "urn:fastxslt:prepared:style-b";

    fn snapshot() -> ResourceSnapshot {
        let source = include_bytes!("../../../../corpus/golden/hello/input.xml");
        let stylesheet = include_bytes!("../../../../corpus/golden/hello/stylesheet.xsl");
        let mut builder = ResourceSetBuilder::new(ResourceLimits::new(8, 4_096, 16_384));
        for (identity, bytes) in [
            (SOURCE_A, source.as_slice()),
            (SOURCE_B, source.as_slice()),
            (STYLE_A, stylesheet.as_slice()),
            (STYLE_B, stylesheet.as_slice()),
        ] {
            builder
                .admit(identity, bytes.to_vec())
                .expect("admit prepared-input fixture");
        }
        builder.seal()
    }

    fn prepare(snapshot: &ResourceSnapshot, identities: &[&str]) -> super::PreparedInputSet {
        let mut builder = PreparedInputBuilder::new(snapshot.clone());
        for identity in identities {
            builder
                .prepare(identity, &mut InvocationControl::unbounded())
                .expect("prepare selected source");
        }
        builder.seal()
    }

    #[test]
    fn one_explicit_prepared_document_is_reused_by_multiple_stylesheets() {
        let snapshot = snapshot();
        let prepared = prepare(&snapshot, &[SOURCE_A]);
        let first_document = prepared.get(SOURCE_A).expect("prepared source");
        let second_document = prepared.get(SOURCE_A).expect("same prepared source");
        assert!(Arc::ptr_eq(&first_document, &second_document));

        let programs = [
            compile_resource(&snapshot, STYLE_A).expect("compile first stylesheet"),
            compile_resource(&snapshot, STYLE_B).expect("compile second stylesheet"),
        ];
        let mut results = Vec::new();
        for (index, program) in programs.iter().enumerate() {
            let request_id = format!("prepared-{index}");
            let mut control = InvocationControl::unbounded();
            let semantic = execute_program(program, &first_document, &request_id, &mut control)
                .expect("execute over prepared source");
            let serialized =
                serialize_xml(&semantic, &program.output, &request_id, 4_096, &mut control)
                    .expect("serialize prepared result");
            results.push((semantic, serialized));
        }

        assert_eq!(results[0], results[1]);
        assert_eq!(results[0].1, "<message>Hello, FastXSLT!</message>");

        let parsed = parse_document(
            SOURCE_A,
            snapshot.get(SOURCE_A).expect("admitted source bytes"),
            ParseLimits {
                max_events: 1_024,
                max_depth: 64,
            },
        )
        .expect("parse direct reference source");
        let direct = Document::from_parsed(parsed).expect("build direct reference source");
        let mut direct_control = InvocationControl::unbounded();
        let direct_semantic = execute_program(
            &programs[0],
            &direct,
            "direct-reference",
            &mut direct_control,
        )
        .expect("execute direct reference");
        let direct_serialized = serialize_xml(
            &direct_semantic,
            &programs[0].output,
            "direct-reference",
            4_096,
            &mut direct_control,
        )
        .expect("serialize direct reference");

        assert_eq!(results[0], (direct_semantic, direct_serialized));
    }

    #[test]
    fn equal_bytes_under_distinct_identities_produce_distinct_prepared_documents() {
        let snapshot = snapshot();
        let prepared = prepare(&snapshot, &[SOURCE_A, SOURCE_B]);
        let first = prepared.get(SOURCE_A).expect("first source");
        let second = prepared.get(SOURCE_B).expect("second source");

        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(first.node_count(), 6);
        assert_eq!(second.node_count(), 6);
        assert!(first.owned_capacity_bytes() > 87);
        assert!(second.owned_capacity_bytes() > 87);
        assert_eq!(first.location(first.document_node()).resource, SOURCE_A);
        assert_eq!(second.location(second.document_node()).resource, SOURCE_B);

        let program = compile_resource(&snapshot, STYLE_A).expect("compile shared stylesheet");
        let mut serialized = Vec::new();
        for (request_id, document) in [(SOURCE_A, first), (SOURCE_B, second)] {
            let mut control = InvocationControl::unbounded();
            let semantic = execute_program(&program, &document, request_id, &mut control)
                .expect("execute over distinct prepared source");
            serialized.push(
                serialize_xml(&semantic, &program.output, request_id, 4_096, &mut control)
                    .expect("serialize distinct prepared result"),
            );
        }
        assert_eq!(serialized[0], serialized[1]);
    }

    #[test]
    fn raw_and_xdm_retention_are_observed_as_separate_classes() {
        const SCALED_SOURCE: &str = "urn:fastxslt:prepared:scaled-source";
        const ITEM_COUNT: usize = 100;
        let baseline_snapshot = snapshot();
        let baseline = prepare(&baseline_snapshot, &[SOURCE_A]);
        let baseline_observation = baseline.observe(SOURCE_A).expect("observe baseline source");
        assert_eq!(baseline_observation.raw_bytes, 87);
        assert!(baseline_observation.parsed_phase_owned_capacity_bytes > 87);
        assert_eq!(baseline_observation.xdm_nodes, 6);
        assert!(
            baseline_observation.xdm_owned_capacity_bytes > baseline_observation.raw_bytes,
            "owned XDM capacity should remain visibly separate from retained source bytes"
        );
        assert_eq!(baseline.observe_totals(), baseline_observation);

        let mut xml = String::from("<catalog>");
        for index in 0..ITEM_COUNT {
            xml.push_str("<item>value-");
            xml.push_str(&index.to_string());
            xml.push_str("</item>");
        }
        xml.push_str("</catalog>");
        let raw_bytes = xml.len();
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 64_000, 64_000));
        resources
            .admit(SCALED_SOURCE, xml.into_bytes())
            .expect("admit scaled source");
        let scaled_snapshot = resources.seal();
        let scaled = prepare(&scaled_snapshot, &[SCALED_SOURCE]);
        let scaled_observation = scaled
            .observe(SCALED_SOURCE)
            .expect("observe scaled prepared source");

        assert_eq!(scaled_observation.raw_bytes, raw_bytes);
        assert!(scaled_observation.parsed_phase_owned_capacity_bytes > raw_bytes);
        assert_eq!(scaled_observation.xdm_nodes, 2 + ITEM_COUNT * 2);
        assert!(scaled_observation.xdm_owned_capacity_bytes > raw_bytes);
        assert_eq!(scaled.observe_totals(), scaled_observation);
        println!(
            "baseline={baseline_observation:?} scaled_items={ITEM_COUNT} scaled={scaled_observation:?}"
        );
    }

    #[test]
    fn prepared_document_and_compiled_program_are_shared_across_concurrent_reads() {
        let snapshot = snapshot();
        let prepared = prepare(&snapshot, &[SOURCE_A]);
        let expected_document = prepared.get(SOURCE_A).expect("prepared source");
        let program =
            Arc::new(compile_resource(&snapshot, STYLE_A).expect("compile shared stylesheet"));

        let workers: Vec<_> = (0..8)
            .map(|worker| {
                let prepared = prepared.clone();
                let program = Arc::clone(&program);
                std::thread::spawn(move || {
                    let document = prepared.get(SOURCE_A).expect("shared prepared source");
                    let mut control = InvocationControl::unbounded();
                    let request_id = format!("concurrent-{worker}");
                    let semantic = execute_program(&program, &document, &request_id, &mut control)
                        .expect("execute concurrent read");
                    let serialized =
                        serialize_xml(&semantic, &program.output, &request_id, 4_096, &mut control)
                            .expect("serialize concurrent read");
                    (document, serialized)
                })
            })
            .collect();

        for worker in workers {
            let (document, serialized) = worker.join().expect("worker should not panic");
            assert!(Arc::ptr_eq(&document, &expected_document));
            assert_eq!(serialized, "<message>Hello, FastXSLT!</message>");
        }
    }

    #[test]
    fn independent_concurrent_builders_duplicate_preparation_by_design() {
        let snapshot = snapshot();
        let start = Arc::new(Barrier::new(2));
        let workers: Vec<_> = (0..2)
            .map(|_| {
                let snapshot = snapshot.clone();
                let start = Arc::clone(&start);
                std::thread::spawn(move || {
                    let mut builder = PreparedInputBuilder::new(snapshot);
                    start.wait();
                    builder
                        .prepare(SOURCE_A, &mut InvocationControl::unbounded())
                        .expect("independent preparation succeeds");
                    builder
                        .seal()
                        .get(SOURCE_A)
                        .expect("independently prepared source")
                })
            })
            .collect();
        let mut documents = workers
            .into_iter()
            .map(|worker| worker.join().expect("preparation worker should not panic"));
        let first = documents.next().expect("first prepared document");
        let second = documents.next().expect("second prepared document");

        assert!(!Arc::ptr_eq(&first, &second));
        assert_eq!(first.node_count(), second.node_count());
        assert_eq!(
            first.location(first.document_node()),
            second.location(second.document_node())
        );
        assert_eq!(
            first.string_value(first.document_node()),
            second.string_value(second.document_node())
        );
    }

    #[test]
    fn prepared_inputs_remain_bound_to_the_snapshot_generation_that_created_them() {
        let original = snapshot();
        let replacement = snapshot();
        let prepared = prepare(&original, &[SOURCE_A]);

        assert!(prepared.belongs_to(&original));
        assert!(!prepared.belongs_to(&replacement));
        drop(original);
        assert_eq!(
            prepared
                .get(SOURCE_A)
                .expect("retained old prepared source")
                .string_value(
                    prepared
                        .get(SOURCE_A)
                        .expect("retained source")
                        .document_node()
                ),
            "\n  FastXSLT\n"
        );
    }

    #[test]
    fn preparation_has_explicit_cancellation_and_work_limits() {
        let snapshot = snapshot();
        let token = CancellationToken::new();
        token.cancel();
        let mut cancelled = PreparedInputBuilder::new(snapshot.clone());
        assert_eq!(
            cancelled.prepare(
                SOURCE_A,
                &mut InvocationControl::new(token, WorkLimits::unbounded())
            ),
            Err(PreparationFailure::Control(ControlFailure::Cancelled {
                domain: WorkDomain::XmlEvent,
            }))
        );
        cancelled
            .prepare(SOURCE_A, &mut InvocationControl::unbounded())
            .expect("cancelled preparation must not poison a retry");
        assert!(cancelled.seal().get(SOURCE_A).is_some());

        let mut limits = WorkLimits::unbounded();
        limits.xdm_nodes = 1;
        let mut limited = PreparedInputBuilder::new(snapshot);
        let failure = limited
            .prepare(
                SOURCE_A,
                &mut InvocationControl::new(CancellationToken::new(), limits),
            )
            .expect_err("XDM preparation should be bounded");
        assert!(matches!(
            failure,
            PreparationFailure::Control(ControlFailure::BudgetExhausted {
                domain: WorkDomain::XdmNode,
                ..
            })
        ));
        limited
            .prepare(SOURCE_A, &mut InvocationControl::unbounded())
            .expect("budget failure must not retain a partial entry");
        assert!(limited.seal().get(SOURCE_A).is_some());
    }

    #[test]
    fn missing_and_duplicate_preparation_are_explicit() {
        let snapshot = snapshot();
        let mut builder = PreparedInputBuilder::new(snapshot);
        assert_eq!(
            builder.prepare("missing", &mut InvocationControl::unbounded()),
            Err(PreparationFailure::MissingResource {
                identity: "missing".to_owned(),
            })
        );
        builder
            .prepare(SOURCE_A, &mut InvocationControl::unbounded())
            .expect("prepare source once");
        assert_eq!(
            builder.prepare(SOURCE_A, &mut InvocationControl::unbounded()),
            Err(PreparationFailure::DuplicateResource {
                identity: SOURCE_A.to_owned(),
            })
        );
    }

    #[test]
    #[ignore = "manual release-mode parse-per-invocation versus prepared-input probe"]
    fn measures_parse_per_invocation_against_prepared_reuse() {
        const BENCH_SOURCE: &str = "urn:fastxslt:measure:built-in:source";
        const BENCH_STYLE: &str = "urn:fastxslt:measure:built-in:stylesheet";
        const ITERATIONS: usize = 10_000;
        const ITERATIONS_F64: f64 = 10_000.0;
        const SAMPLES: usize = 7;

        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(4, 4_096, 8_192));
        resources
            .admit(
                BENCH_SOURCE,
                include_bytes!("../../../../corpus/golden/built-in-template-rules/input.xml")
                    .to_vec(),
            )
            .expect("admit measurement source");
        resources
            .admit(
                BENCH_STYLE,
                include_bytes!("../../../../corpus/golden/built-in-template-rules/stylesheet.xsl")
                    .to_vec(),
            )
            .expect("admit measurement stylesheet");
        let snapshot = resources.seal();
        let program = compile_resource(&snapshot, BENCH_STYLE).expect("compile measurement style");
        let prepared = prepare(&snapshot, &[BENCH_SOURCE]);

        let run = |document: &Document, request_id: &str| {
            let mut control = InvocationControl::unbounded();
            let semantic = execute_program(&program, document, request_id, &mut control)
                .expect("execute measured transform");
            serialize_xml(&semantic, &program.output, request_id, 4_096, &mut control)
                .expect("serialize measured transform")
        };
        let source_bytes = snapshot
            .get(BENCH_SOURCE)
            .expect("measurement source remains admitted");
        let direct_document = Document::from_parsed(
            parse_document(BENCH_SOURCE, source_bytes, super::PREPARATION_XML_LIMITS)
                .expect("parse direct correctness reference"),
        )
        .expect("build direct correctness reference");
        let prepared_document = prepared
            .get(BENCH_SOURCE)
            .expect("prepared measurement source");
        assert_eq!(
            run(&direct_document, "direct-correctness"),
            run(&prepared_document, "prepared-correctness")
        );

        let mut direct_ns = Vec::with_capacity(SAMPLES);
        let mut prepared_ns = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            let direct_start = Instant::now();
            for iteration in 0..ITERATIONS {
                let parsed = parse_document(
                    BENCH_SOURCE,
                    black_box(source_bytes),
                    super::PREPARATION_XML_LIMITS,
                )
                .expect("parse measured direct source");
                let document = Document::from_parsed(parsed).expect("build measured direct XDM");
                black_box(run(&document, black_box("direct")));
                black_box(iteration);
            }
            direct_ns.push(direct_start.elapsed().as_secs_f64() * 1_000_000_000.0 / ITERATIONS_F64);

            let prepared_start = Instant::now();
            for iteration in 0..ITERATIONS {
                let document = black_box(
                    prepared
                        .get(BENCH_SOURCE)
                        .expect("get measured prepared source"),
                );
                black_box(run(&document, black_box("prepared")));
                black_box(iteration);
            }
            prepared_ns
                .push(prepared_start.elapsed().as_secs_f64() * 1_000_000_000.0 / ITERATIONS_F64);
        }

        direct_ns.sort_by(f64::total_cmp);
        prepared_ns.sort_by(f64::total_cmp);
        let direct_median = direct_ns[SAMPLES / 2];
        let prepared_median = prepared_ns[SAMPLES / 2];
        println!(
            "iterations={ITERATIONS} samples={SAMPLES} direct_median_ns={direct_median:.1} prepared_median_ns={prepared_median:.1} ratio={:.2}",
            direct_median / prepared_median
        );
    }

    #[test]
    #[ignore = "manual release-mode XML-parse versus XDM-construction probe"]
    fn measures_preparation_phase_time_separately() {
        const BENCH_SOURCE: &str = "urn:fastxslt:measure:preparation-phases";
        const ITERATIONS: usize = 10_000;
        const ITERATIONS_F64: f64 = 10_000.0;
        const SAMPLES: usize = 7;
        let source_bytes =
            include_bytes!("../../../../corpus/golden/built-in-template-rules/input.xml");
        let mut parse_ns = Vec::with_capacity(SAMPLES);
        let mut xdm_ns = Vec::with_capacity(SAMPLES);

        for _ in 0..SAMPLES {
            let parse_start = Instant::now();
            for iteration in 0..ITERATIONS {
                black_box(
                    parse_document(
                        BENCH_SOURCE,
                        black_box(source_bytes),
                        super::PREPARATION_XML_LIMITS,
                    )
                    .expect("parse measured preparation source"),
                );
                black_box(iteration);
            }
            parse_ns.push(parse_start.elapsed().as_secs_f64() * 1_000_000_000.0 / ITERATIONS_F64);

            let parsed_documents: Vec<_> = (0..ITERATIONS)
                .map(|_| {
                    parse_document(BENCH_SOURCE, source_bytes, super::PREPARATION_XML_LIMITS)
                        .expect("prepare XDM timing input")
                })
                .collect();
            let xdm_start = Instant::now();
            for parsed in parsed_documents {
                black_box(Document::from_parsed(parsed).expect("build measured XDM document"));
            }
            xdm_ns.push(xdm_start.elapsed().as_secs_f64() * 1_000_000_000.0 / ITERATIONS_F64);
        }

        parse_ns.sort_by(f64::total_cmp);
        xdm_ns.sort_by(f64::total_cmp);
        println!(
            "iterations={ITERATIONS} samples={SAMPLES} parse_median_ns={:.1} xdm_median_ns={:.1}",
            parse_ns[SAMPLES / 2],
            xdm_ns[SAMPLES / 2]
        );
    }

    #[test]
    #[ignore = "manual release-mode multi-source and multi-stylesheet reuse probe"]
    #[allow(
        clippy::too_many_lines,
        reason = "keeping setup, correctness conservation, four timed paths, and reporting together makes this manual probe auditable"
    )]
    fn measures_multi_source_and_multi_stylesheet_reuse_shapes() {
        const SOURCE_COUNT: usize = 8;
        const STYLE_COUNT: usize = 8;
        const ITERATIONS: usize = 1_000;
        const OPERATIONS_F64: f64 = 8_000.0;
        const SAMPLES: usize = 7;
        let source_bytes =
            include_bytes!("../../../../corpus/golden/built-in-template-rules/input.xml");
        let stylesheet_bytes =
            include_bytes!("../../../../corpus/golden/built-in-template-rules/stylesheet.xsl");
        let source_ids: Vec<_> = (0..SOURCE_COUNT)
            .map(|index| format!("urn:fastxslt:measure:shape:source-{index}"))
            .collect();
        let style_ids: Vec<_> = (0..STYLE_COUNT)
            .map(|index| format!("urn:fastxslt:measure:shape:style-{index}"))
            .collect();
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(16, 4_096, 32_768));
        for identity in &source_ids {
            resources
                .admit(identity.clone(), source_bytes.to_vec())
                .expect("admit shape source");
        }
        for identity in &style_ids {
            resources
                .admit(identity.clone(), stylesheet_bytes.to_vec())
                .expect("admit shape stylesheet");
        }
        let snapshot = resources.seal();
        let prepared_identities: Vec<_> = source_ids.iter().map(String::as_str).collect();
        let prepared = prepare(&snapshot, &prepared_identities);
        let programs: Vec<_> = style_ids
            .iter()
            .map(|identity| compile_resource(&snapshot, identity).expect("compile shape style"))
            .collect();
        let run = |program: &crate::xslt::golden_semantics_experiment::StylesheetProgram,
                   document: &Document| {
            let mut control = InvocationControl::unbounded();
            let semantic = execute_program(program, document, "shape", &mut control)
                .expect("execute shape transform");
            serialize_xml(&semantic, &program.output, "shape", 4_096, &mut control)
                .expect("serialize shape transform")
        };

        let direct_reference = Document::from_parsed(
            parse_document(
                &source_ids[0],
                snapshot.get(&source_ids[0]).expect("reference source"),
                super::PREPARATION_XML_LIMITS,
            )
            .expect("parse shape reference"),
        )
        .expect("build shape reference");
        assert_eq!(
            run(&programs[0], &direct_reference),
            run(
                &programs[0],
                &prepared.get(&source_ids[0]).expect("prepared reference")
            )
        );

        let mut multi_source_direct = Vec::with_capacity(SAMPLES);
        let mut multi_source_prepared = Vec::with_capacity(SAMPLES);
        let mut multi_style_direct = Vec::with_capacity(SAMPLES);
        let mut multi_style_prepared = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            let start = Instant::now();
            for _ in 0..ITERATIONS {
                for identity in &source_ids {
                    let parsed = parse_document(
                        identity,
                        black_box(snapshot.get(identity).expect("shape source bytes")),
                        super::PREPARATION_XML_LIMITS,
                    )
                    .expect("parse direct multi-source input");
                    let document =
                        Document::from_parsed(parsed).expect("build direct multi-source XDM");
                    black_box(run(&programs[0], &document));
                }
            }
            multi_source_direct
                .push(start.elapsed().as_secs_f64() * 1_000_000_000.0 / OPERATIONS_F64);

            let start = Instant::now();
            for _ in 0..ITERATIONS {
                for identity in &source_ids {
                    let document = prepared
                        .get(identity)
                        .expect("get prepared multi-source input");
                    black_box(run(&programs[0], &document));
                }
            }
            multi_source_prepared
                .push(start.elapsed().as_secs_f64() * 1_000_000_000.0 / OPERATIONS_F64);

            let start = Instant::now();
            for _ in 0..ITERATIONS {
                for program in &programs {
                    let parsed = parse_document(
                        &source_ids[0],
                        black_box(snapshot.get(&source_ids[0]).expect("shared source bytes")),
                        super::PREPARATION_XML_LIMITS,
                    )
                    .expect("parse direct multi-style input");
                    let document =
                        Document::from_parsed(parsed).expect("build direct multi-style XDM");
                    black_box(run(program, &document));
                }
            }
            multi_style_direct
                .push(start.elapsed().as_secs_f64() * 1_000_000_000.0 / OPERATIONS_F64);

            let start = Instant::now();
            for _ in 0..ITERATIONS {
                for program in &programs {
                    let document = prepared
                        .get(&source_ids[0])
                        .expect("get prepared multi-style input");
                    black_box(run(program, &document));
                }
            }
            multi_style_prepared
                .push(start.elapsed().as_secs_f64() * 1_000_000_000.0 / OPERATIONS_F64);
        }

        for observations in [
            &mut multi_source_direct,
            &mut multi_source_prepared,
            &mut multi_style_direct,
            &mut multi_style_prepared,
        ] {
            observations.sort_by(f64::total_cmp);
        }
        let middle = SAMPLES / 2;
        println!(
            "iterations={ITERATIONS} operations_per_shape={} samples={SAMPLES} multi_source_direct_ns={:.1} multi_source_prepared_ns={:.1} multi_source_ratio={:.2} multi_style_direct_ns={:.1} multi_style_prepared_ns={:.1} multi_style_ratio={:.2}",
            ITERATIONS * SOURCE_COUNT,
            multi_source_direct[middle],
            multi_source_prepared[middle],
            multi_source_direct[middle] / multi_source_prepared[middle],
            multi_style_direct[middle],
            multi_style_prepared[middle],
            multi_style_direct[middle] / multi_style_prepared[middle]
        );
    }

    #[cfg(feature = "allocation-observation")]
    #[test]
    #[ignore = "manual allocator-requested retained and peak preparation probe"]
    fn measures_preparation_allocations() {
        const SCALED_SOURCE: &str = "urn:fastxslt:prepared:scaled-source";
        const ITEM_COUNT: usize = 100;

        let baseline_snapshot = snapshot();
        let mut baseline_builder = PreparedInputBuilder::new(baseline_snapshot);
        let mut baseline_control = InvocationControl::unbounded();
        let baseline_allocations = allocation_counter::measure(|| {
            baseline_builder
                .prepare(SOURCE_A, &mut baseline_control)
                .expect("prepare allocation baseline");
        });
        let baseline = baseline_builder.seal();

        let mut xml = String::from("<catalog>");
        for index in 0..ITEM_COUNT {
            xml.push_str("<item>value-");
            xml.push_str(&index.to_string());
            xml.push_str("</item>");
        }
        xml.push_str("</catalog>");
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 64_000, 64_000));
        resources
            .admit(SCALED_SOURCE, xml.into_bytes())
            .expect("admit allocation source");
        let mut scaled_builder = PreparedInputBuilder::new(resources.seal());
        let mut scaled_control = InvocationControl::unbounded();
        let scaled_allocations = allocation_counter::measure(|| {
            scaled_builder
                .prepare(SCALED_SOURCE, &mut scaled_control)
                .expect("prepare scaled allocation source");
        });
        let scaled = scaled_builder.seal();

        assert!(baseline_allocations.bytes_current > 0);
        let baseline_retained = u64::try_from(baseline_allocations.bytes_current)
            .expect("positive retained bytes fit the unsigned observation");
        assert!(baseline_allocations.bytes_max >= baseline_retained);
        assert!(scaled_allocations.bytes_current > baseline_allocations.bytes_current);
        assert!(scaled_allocations.bytes_max > baseline_allocations.bytes_max);
        println!(
            "baseline_representation={:?} baseline_allocations={baseline_allocations:?} scaled_items={ITEM_COUNT} scaled_representation={:?} scaled_allocations={scaled_allocations:?}",
            baseline.observe(SOURCE_A).expect("observe baseline"),
            scaled.observe(SCALED_SOURCE).expect("observe scaled")
        );
    }
}
