//! Private AR-0009 experiment for explicit prepared-input ownership and reuse.

use std::{collections::BTreeMap, sync::Arc};

use crate::execution_control_experiment::{ControlFailure, InvocationControl};
use crate::resources::ResourceSnapshot;
use crate::xdm::owned_tree_experiment::{BuildFailure, Document};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document_controlled};

const PREPARATION_XML_LIMITS: ParseLimits = ParseLimits {
    max_events: 1_024,
    max_depth: 64,
};

#[derive(Debug, PartialEq, Eq)]
enum PreparationFailure {
    MissingResource { identity: String },
    DuplicateResource { identity: String },
    InvalidXml { identity: String, detail: String },
    InvalidXdm { identity: String, detail: String },
    Control(ControlFailure),
}

#[derive(Debug)]
struct PreparedInputBuilder {
    snapshot: ResourceSnapshot,
    documents: BTreeMap<String, Arc<Document>>,
}

impl PreparedInputBuilder {
    fn new(snapshot: ResourceSnapshot) -> Self {
        Self {
            snapshot,
            documents: BTreeMap::new(),
        }
    }

    fn prepare(
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
        let parsed = parse_document_controlled(identity, bytes, PREPARATION_XML_LIMITS, control)
            .map_err(|failure| {
                failure.control_failure().map_or_else(
                    || PreparationFailure::InvalidXml {
                        identity: identity.to_owned(),
                        detail: format!("{failure:?}"),
                    },
                    |failure| PreparationFailure::Control(*failure),
                )
            })?;
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
        Ok(())
    }

    fn seal(self) -> PreparedInputSet {
        PreparedInputSet {
            snapshot: self.snapshot,
            documents: Arc::new(self.documents),
        }
    }
}

#[derive(Clone, Debug)]
struct PreparedInputSet {
    snapshot: ResourceSnapshot,
    documents: Arc<BTreeMap<String, Arc<Document>>>,
}

impl PreparedInputSet {
    fn belongs_to(&self, snapshot: &ResourceSnapshot) -> bool {
        self.snapshot.same_generation(snapshot)
    }

    fn get(&self, identity: &str) -> Option<Arc<Document>> {
        self.documents.get(identity).cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::{hint::black_box, sync::Arc, time::Instant};

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
}
