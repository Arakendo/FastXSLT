//! Private AR-0020 reference for immutable prepared packets and bounded ready state.

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::execution_control_experiment::InvocationControl;
use crate::resources::ResourceSnapshot;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::ParseLimits;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

use super::transform_set_experiment::{
    ExecutionPolicy, ResultEntry, TransformRequest, execute_prepared_request, request_control,
    request_source_identity,
};
use super::{ExecutionFailure, FailureCategory, MultipleMatchPolicy};
use crate::runtime::prepared_input_experiment::{PreparationFailure, prepare_document};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ReadyQueueLimits {
    pub(super) packet_count: usize,
    pub(super) known_prepared_capacity_bytes: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct ReadyQueueObservation {
    pub(super) packet_count: usize,
    pub(super) known_prepared_capacity_bytes: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct PipelinePressureObservation {
    pub(super) ready: ReadyQueueObservation,
    pub(super) executing_packet_count: usize,
    pub(super) executing_prepared_capacity_bytes: usize,
    pub(super) high_water_ready_packet_count: usize,
    pub(super) high_water_ready_prepared_capacity_bytes: usize,
    pub(super) high_water_executing_packet_count: usize,
    pub(super) high_water_executing_prepared_capacity_bytes: usize,
    pub(super) high_water_total_prepared_capacity_bytes: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct PacketPhaseObservation {
    pub(super) preparation_service: Duration,
    pub(super) ready_queue_residence: Duration,
    pub(super) execution_service: Duration,
}

#[derive(Debug)]
pub(super) struct ObservedExecution {
    pub(super) result: Result<ResultEntry, ExecutionFailure>,
    pub(super) phases: PacketPhaseObservation,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ReadyQueueAdmissionFailure {
    PacketCount {
        limit: usize,
        attempted: usize,
    },
    KnownPreparedCapacity {
        limit: usize,
        current: usize,
        attempted: usize,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum PacketPreparationFailure {
    DeniedSource { identity: String },
    Preparation(PreparationFailure),
}

#[derive(Debug)]
pub(super) struct PreparedExecutionPacket {
    snapshot: ResourceSnapshot,
    stylesheet: Arc<StylesheetProgram>,
    policy: ExecutionPolicy,
    multiple_match_policy: MultipleMatchPolicy,
    request: TransformRequest,
    source: Option<Arc<Document>>,
    control: InvocationControl,
    pub(super) known_prepared_capacity_bytes: usize,
    preparation_service_time: Duration,
}

impl PreparedExecutionPacket {
    pub(super) fn prepare_duplicate(
        snapshot: ResourceSnapshot,
        stylesheet: Arc<StylesheetProgram>,
        policy: ExecutionPolicy,
        multiple_match_policy: MultipleMatchPolicy,
        request: TransformRequest,
        parse_limits: ParseLimits,
    ) -> Result<Self, PacketPreparationFailure> {
        let preparation_started = Instant::now();
        let mut control = request_control(&request, &policy);
        let source = request_source_identity(&request.entry)
            .map(|identity| {
                if policy.denied_sources.contains(identity) {
                    return Err(PacketPreparationFailure::DeniedSource {
                        identity: identity.to_owned(),
                    });
                }
                prepare_document(&snapshot, parse_limits, identity, &mut control)
                    .map(|(document, _parsed_capacity)| document)
                    .map_err(PacketPreparationFailure::Preparation)
            })
            .transpose()?;
        let known_prepared_capacity_bytes = source
            .as_ref()
            .map_or(0, |document| document.owned_capacity_bytes());
        Ok(Self {
            snapshot,
            stylesheet,
            policy,
            multiple_match_policy,
            request,
            source,
            control,
            known_prepared_capacity_bytes,
            preparation_service_time: preparation_started.elapsed(),
        })
    }

    pub(super) fn execute(mut self) -> Result<ResultEntry, ExecutionFailure> {
        execute_prepared_request(
            &self.snapshot,
            &self.stylesheet,
            &self.policy,
            self.multiple_match_policy,
            &self.request,
            self.source.as_deref(),
            &mut self.control,
        )
    }
}

#[derive(Debug, Default)]
struct PipelinePressureState {
    ready: ReadyQueueObservation,
    executing_packet_count: usize,
    executing_prepared_capacity_bytes: usize,
    high_water_ready_packet_count: usize,
    high_water_ready_prepared_capacity_bytes: usize,
    high_water_executing_packet_count: usize,
    high_water_executing_prepared_capacity_bytes: usize,
    high_water_total_prepared_capacity_bytes: usize,
}

impl PipelinePressureState {
    fn update_high_water(&mut self) {
        self.high_water_ready_packet_count = self
            .high_water_ready_packet_count
            .max(self.ready.packet_count);
        self.high_water_ready_prepared_capacity_bytes = self
            .high_water_ready_prepared_capacity_bytes
            .max(self.ready.known_prepared_capacity_bytes);
        self.high_water_executing_packet_count = self
            .high_water_executing_packet_count
            .max(self.executing_packet_count);
        self.high_water_executing_prepared_capacity_bytes = self
            .high_water_executing_prepared_capacity_bytes
            .max(self.executing_prepared_capacity_bytes);
        self.high_water_total_prepared_capacity_bytes =
            self.high_water_total_prepared_capacity_bytes.max(
                self.ready
                    .known_prepared_capacity_bytes
                    .saturating_add(self.executing_prepared_capacity_bytes),
            );
    }

    fn observe(&self) -> PipelinePressureObservation {
        PipelinePressureObservation {
            ready: self.ready,
            executing_packet_count: self.executing_packet_count,
            executing_prepared_capacity_bytes: self.executing_prepared_capacity_bytes,
            high_water_ready_packet_count: self.high_water_ready_packet_count,
            high_water_ready_prepared_capacity_bytes: self.high_water_ready_prepared_capacity_bytes,
            high_water_executing_packet_count: self.high_water_executing_packet_count,
            high_water_executing_prepared_capacity_bytes: self
                .high_water_executing_prepared_capacity_bytes,
            high_water_total_prepared_capacity_bytes: self.high_water_total_prepared_capacity_bytes,
        }
    }
}

#[derive(Debug, Default)]
struct PipelinePressureTracker {
    state: Mutex<PipelinePressureState>,
}

impl PipelinePressureTracker {
    fn admit_ready(
        &self,
        known_prepared_capacity_bytes: usize,
        limits: ReadyQueueLimits,
    ) -> Result<(), ReadyQueueAdmissionFailure> {
        let mut state = self.state.lock().expect("pipeline pressure lock");
        let attempted_count = state.ready.packet_count.saturating_add(1);
        if attempted_count > limits.packet_count {
            return Err(ReadyQueueAdmissionFailure::PacketCount {
                limit: limits.packet_count,
                attempted: attempted_count,
            });
        }
        let attempted_bytes = state
            .ready
            .known_prepared_capacity_bytes
            .saturating_add(known_prepared_capacity_bytes);
        if attempted_bytes > limits.known_prepared_capacity_bytes {
            return Err(ReadyQueueAdmissionFailure::KnownPreparedCapacity {
                limit: limits.known_prepared_capacity_bytes,
                current: state.ready.known_prepared_capacity_bytes,
                attempted: attempted_bytes,
            });
        }
        state.ready.packet_count = attempted_count;
        state.ready.known_prepared_capacity_bytes = attempted_bytes;
        state.update_high_water();
        Ok(())
    }

    fn assign(self: &Arc<Self>, known_prepared_capacity_bytes: usize) -> ExecutionPressureLease {
        let mut state = self.state.lock().expect("pipeline pressure lock");
        state.ready.packet_count = state
            .ready
            .packet_count
            .checked_sub(1)
            .expect("assigned packet must have a ready count charge");
        state.ready.known_prepared_capacity_bytes = state
            .ready
            .known_prepared_capacity_bytes
            .checked_sub(known_prepared_capacity_bytes)
            .expect("assigned packet must have a ready capacity charge");
        state.executing_packet_count = state.executing_packet_count.saturating_add(1);
        state.executing_prepared_capacity_bytes = state
            .executing_prepared_capacity_bytes
            .saturating_add(known_prepared_capacity_bytes);
        state.update_high_water();
        ExecutionPressureLease {
            tracker: Arc::clone(self),
            known_prepared_capacity_bytes,
        }
    }

    fn observe(&self) -> PipelinePressureObservation {
        self.state.lock().expect("pipeline pressure lock").observe()
    }
}

#[derive(Debug)]
struct ExecutionPressureLease {
    tracker: Arc<PipelinePressureTracker>,
    known_prepared_capacity_bytes: usize,
}

impl Drop for ExecutionPressureLease {
    fn drop(&mut self) {
        let mut state = self.tracker.state.lock().expect("pipeline pressure lock");
        state.executing_packet_count = state
            .executing_packet_count
            .checked_sub(1)
            .expect("execution lease must have a packet charge");
        state.executing_prepared_capacity_bytes = state
            .executing_prepared_capacity_bytes
            .checked_sub(self.known_prepared_capacity_bytes)
            .expect("execution lease must have a capacity charge");
    }
}

#[derive(Debug)]
struct QueuedExecutionPacket {
    packet: PreparedExecutionPacket,
    admitted_at: Instant,
}

#[derive(Debug)]
pub(super) struct AssignedExecutionPacket {
    packet: PreparedExecutionPacket,
    ready_queue_residence_time: Duration,
    pressure_lease: ExecutionPressureLease,
}

impl AssignedExecutionPacket {
    pub(super) fn execute(self) -> Result<ResultEntry, ExecutionFailure> {
        self.execute_observed().result
    }

    pub(super) fn execute_observed(self) -> ObservedExecution {
        let Self {
            packet,
            ready_queue_residence_time,
            pressure_lease,
        } = self;
        let preparation_service_time = packet.preparation_service_time;
        let execution_started = Instant::now();
        let result = packet.execute();
        let execution_service_time = execution_started.elapsed();
        drop(pressure_lease);
        ObservedExecution {
            result,
            phases: PacketPhaseObservation {
                preparation_service: preparation_service_time,
                ready_queue_residence: ready_queue_residence_time,
                execution_service: execution_service_time,
            },
        }
    }
}

#[derive(Debug)]
pub(super) struct BoundedReadyQueue {
    limits: ReadyQueueLimits,
    packets: VecDeque<QueuedExecutionPacket>,
    pressure: Arc<PipelinePressureTracker>,
}

impl BoundedReadyQueue {
    pub(super) fn new(limits: ReadyQueueLimits) -> Self {
        Self {
            limits,
            packets: VecDeque::new(),
            pressure: Arc::new(PipelinePressureTracker::default()),
        }
    }

    pub(super) fn push(
        &mut self,
        packet: PreparedExecutionPacket,
    ) -> Result<(), ReadyQueueAdmissionFailure> {
        self.pressure
            .admit_ready(packet.known_prepared_capacity_bytes, self.limits)?;
        self.packets.push_back(QueuedExecutionPacket {
            packet,
            admitted_at: Instant::now(),
        });
        Ok(())
    }

    pub(super) fn pop(&mut self) -> Option<AssignedExecutionPacket> {
        let queued = self.packets.pop_front()?;
        let pressure_lease = self
            .pressure
            .assign(queued.packet.known_prepared_capacity_bytes);
        Some(AssignedExecutionPacket {
            packet: queued.packet,
            ready_queue_residence_time: queued.admitted_at.elapsed(),
            pressure_lease,
        })
    }

    pub(super) fn observe(&self) -> ReadyQueueObservation {
        self.pressure.observe().ready
    }

    pub(super) fn observe_pressure(&self) -> PipelinePressureObservation {
        self.pressure.observe()
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, HashSet},
        fmt::Write as _,
        sync::Arc,
    };

    use crate::execution_control_experiment::{
        CancellationToken, ControlFailure, WorkDomain, WorkLimits,
    };
    use crate::resources::{ResourceLimits, ResourceSetBuilder};

    use super::super::transform_set_experiment::{
        InvocationEntry, TransformSetBuilder, execute_transform_set,
    };
    use super::*;

    const SOURCE_ID: &str = "urn:fastxslt:ar-0020:source";
    const STYLE_ID: &str = "urn:fastxslt:ar-0020:stylesheet";

    fn snapshot() -> ResourceSnapshot {
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
        resources
            .admit(
                SOURCE_ID,
                include_bytes!("../../../../corpus/golden/hello/input.xml").to_vec(),
            )
            .expect("admit AR-0020 source");
        resources
            .admit(
                STYLE_ID,
                include_bytes!("../../../../corpus/golden/hello/stylesheet.xsl").to_vec(),
            )
            .expect("admit AR-0020 stylesheet");
        resources.seal()
    }

    fn policy() -> ExecutionPolicy {
        ExecutionPolicy {
            denied_sources: HashSet::new(),
            serialized_byte_limit: 4_096,
            work_limits: WorkLimits::unbounded(),
        }
    }

    fn source_snapshot(source: Option<&[u8]>) -> ResourceSnapshot {
        let resource_count = usize::from(source.is_some()) + 1;
        let mut resources =
            ResourceSetBuilder::new(ResourceLimits::new(resource_count, 4_096, 8_192));
        if let Some(source) = source {
            resources
                .admit(SOURCE_ID, source.to_vec())
                .expect("admit AR-0020 selected source");
        }
        resources
            .admit(
                STYLE_ID,
                include_bytes!("../../../../corpus/golden/hello/stylesheet.xsl").to_vec(),
            )
            .expect("admit AR-0020 stylesheet");
        resources.seal()
    }

    fn request(identity: &str, cancellation: CancellationToken) -> TransformRequest {
        TransformRequest {
            identity: identity.to_owned(),
            result_identity: format!("result:{identity}"),
            entry: InvocationEntry::PrincipalSource {
                resource: SOURCE_ID.to_owned(),
            },
            parameters: BTreeMap::default(),
            cancellation,
            cancellation_fault: None,
        }
    }

    fn prepare_packet(
        snapshot: ResourceSnapshot,
        stylesheet: Arc<StylesheetProgram>,
        identity: &str,
    ) -> Result<PreparedExecutionPacket, PacketPreparationFailure> {
        PreparedExecutionPacket::prepare_duplicate(
            snapshot,
            stylesheet,
            policy(),
            MultipleMatchPolicy::UseLast,
            request(identity, CancellationToken::new()),
            super::super::XML_LIMITS,
        )
    }

    #[test]
    fn prepared_packet_executes_the_same_semantics_as_worker_local_preparation() {
        let snapshot = snapshot();
        let stylesheet = super::super::compile_resource(&snapshot, STYLE_ID)
            .expect("compile AR-0020 stylesheet");
        let mut reference =
            TransformSetBuilder::new(snapshot.clone(), stylesheet.clone(), 1, policy());
        reference
            .add(request("reference", CancellationToken::new()))
            .expect("admit reference request");
        let reference = execute_transform_set(reference.seal())
            .expect("execute worker-local preparation reference");

        let packet = prepare_packet(snapshot, Arc::new(stylesheet), "prepared")
            .expect("prepare immutable execution packet");
        let prepared = packet.execute().expect("execute prepared packet");
        let reference = reference
            .by_request
            .get("reference")
            .expect("reference result");
        assert_eq!(prepared.semantic, reference.semantic);
        assert_eq!(prepared.serialized, reference.serialized);
        assert_eq!(prepared.result_id, "result:prepared");
    }

    #[test]
    fn shared_source_requests_duplicate_preparation_before_single_flight_is_considered() {
        let snapshot = snapshot();
        let stylesheet = Arc::new(
            super::super::compile_resource(&snapshot, STYLE_ID)
                .expect("compile AR-0020 stylesheet"),
        );
        let first = prepare_packet(snapshot.clone(), stylesheet.clone(), "first")
            .expect("prepare first packet");
        let second = prepare_packet(snapshot, stylesheet, "second").expect("prepare second packet");
        let first_source = first.source.as_ref().expect("first source");
        let second_source = second.source.as_ref().expect("second source");

        assert!(!Arc::ptr_eq(first_source, second_source));
        assert_eq!(
            first.known_prepared_capacity_bytes,
            second.known_prepared_capacity_bytes
        );
        assert_eq!(first_source.node_count(), second_source.node_count());
    }

    #[test]
    fn ready_queue_checks_count_then_known_prepared_capacity_without_mutation() {
        let snapshot = snapshot();
        let stylesheet = Arc::new(
            super::super::compile_resource(&snapshot, STYLE_ID)
                .expect("compile AR-0020 stylesheet"),
        );
        let first = prepare_packet(snapshot.clone(), stylesheet.clone(), "first")
            .expect("prepare first packet");
        let capacity = first.known_prepared_capacity_bytes;
        let second = prepare_packet(snapshot.clone(), stylesheet.clone(), "second")
            .expect("prepare second packet");
        let mut count_limited = BoundedReadyQueue::new(ReadyQueueLimits {
            packet_count: 1,
            known_prepared_capacity_bytes: usize::MAX,
        });
        count_limited.push(first).expect("admit first packet");
        assert_eq!(
            count_limited.push(second),
            Err(ReadyQueueAdmissionFailure::PacketCount {
                limit: 1,
                attempted: 2,
            })
        );
        assert_eq!(
            count_limited.observe(),
            ReadyQueueObservation {
                packet_count: 1,
                known_prepared_capacity_bytes: capacity,
            }
        );
        count_limited.pop().expect("release first packet");
        assert_eq!(count_limited.observe(), ReadyQueueObservation::default());

        let byte_rejected = prepare_packet(snapshot, stylesheet, "byte-rejected")
            .expect("prepare byte-limited packet");
        let mut byte_limited = BoundedReadyQueue::new(ReadyQueueLimits {
            packet_count: 1,
            known_prepared_capacity_bytes: capacity - 1,
        });
        assert_eq!(
            byte_limited.push(byte_rejected),
            Err(ReadyQueueAdmissionFailure::KnownPreparedCapacity {
                limit: capacity - 1,
                current: 0,
                attempted: capacity,
            })
        );
        assert_eq!(byte_limited.observe(), ReadyQueueObservation::default());
    }

    #[test]
    fn observes_phase_service_time_and_moves_prepared_pressure_between_stages() {
        let snapshot = snapshot();
        let stylesheet = Arc::new(
            super::super::compile_resource(&snapshot, STYLE_ID)
                .expect("compile AR-0020 stylesheet"),
        );
        let first = prepare_packet(snapshot.clone(), stylesheet.clone(), "first")
            .expect("prepare first packet");
        let capacity = first.known_prepared_capacity_bytes;
        let second = prepare_packet(snapshot, stylesheet, "second").expect("prepare second packet");
        let mut queue = BoundedReadyQueue::new(ReadyQueueLimits {
            packet_count: 2,
            known_prepared_capacity_bytes: capacity * 2,
        });
        queue.push(first).expect("admit first packet");
        queue.push(second).expect("admit second packet");

        assert_eq!(
            queue.observe_pressure(),
            PipelinePressureObservation {
                ready: ReadyQueueObservation {
                    packet_count: 2,
                    known_prepared_capacity_bytes: capacity * 2,
                },
                high_water_ready_packet_count: 2,
                high_water_ready_prepared_capacity_bytes: capacity * 2,
                high_water_total_prepared_capacity_bytes: capacity * 2,
                ..PipelinePressureObservation::default()
            }
        );

        let assigned = queue.pop().expect("assign first packet");
        let assigned_pressure = queue.observe_pressure();
        assert_eq!(assigned_pressure.ready.packet_count, 1);
        assert_eq!(
            assigned_pressure.ready.known_prepared_capacity_bytes,
            capacity
        );
        assert_eq!(assigned_pressure.executing_packet_count, 1);
        assert_eq!(
            assigned_pressure.executing_prepared_capacity_bytes,
            capacity
        );
        assert_eq!(assigned_pressure.high_water_executing_packet_count, 1);
        assert_eq!(
            assigned_pressure.high_water_executing_prepared_capacity_bytes,
            capacity
        );
        assert_eq!(
            assigned_pressure.high_water_total_prepared_capacity_bytes,
            capacity * 2
        );

        let observed = assigned.execute_observed();
        assert!(observed.result.is_ok());
        assert!(observed.phases.preparation_service > Duration::ZERO);
        assert!(observed.phases.ready_queue_residence > Duration::ZERO);
        assert!(observed.phases.execution_service > Duration::ZERO);
        let after_execution = queue.observe_pressure();
        assert_eq!(after_execution.ready.packet_count, 1);
        assert_eq!(after_execution.executing_packet_count, 0);
        assert_eq!(
            after_execution.executing_prepared_capacity_bytes, 0,
            "execution charge must be released after success"
        );

        let abandoned = queue.pop().expect("assign second packet");
        assert_eq!(queue.observe_pressure().executing_packet_count, 1);
        drop(abandoned);
        let after_abandonment = queue.observe_pressure();
        assert_eq!(after_abandonment.ready, ReadyQueueObservation::default());
        assert_eq!(after_abandonment.executing_packet_count, 0);
        assert_eq!(after_abandonment.executing_prepared_capacity_bytes, 0);
    }

    #[test]
    fn cancelled_preparation_publishes_no_packet_and_a_new_attempt_can_retry() {
        let snapshot = snapshot();
        let stylesheet = Arc::new(
            super::super::compile_resource(&snapshot, STYLE_ID)
                .expect("compile AR-0020 stylesheet"),
        );
        let cancelled = CancellationToken::new();
        cancelled.cancel();
        let failure = PreparedExecutionPacket::prepare_duplicate(
            snapshot.clone(),
            stylesheet.clone(),
            policy(),
            MultipleMatchPolicy::UseLast,
            request("cancelled", cancelled),
            super::super::XML_LIMITS,
        )
        .expect_err("cancelled preparation must fail before packet publication");
        assert_eq!(
            failure,
            PacketPreparationFailure::Preparation(PreparationFailure::Control(
                ControlFailure::Cancelled {
                    domain: WorkDomain::XmlEvent,
                }
            ))
        );

        let retry = prepare_packet(snapshot, stylesheet, "retry")
            .expect("a new attempt can prepare after cancellation");
        assert!(retry.known_prepared_capacity_bytes > 0);
    }

    #[test]
    fn denied_missing_and_invalid_sources_fail_before_packet_publication() {
        let valid = snapshot();
        let stylesheet = Arc::new(
            super::super::compile_resource(&valid, STYLE_ID).expect("compile AR-0020 stylesheet"),
        );
        let mut denied_policy = policy();
        denied_policy.denied_sources.insert(SOURCE_ID.to_owned());
        let denied = PreparedExecutionPacket::prepare_duplicate(
            valid,
            stylesheet.clone(),
            denied_policy,
            MultipleMatchPolicy::UseLast,
            request("denied", CancellationToken::new()),
            super::super::XML_LIMITS,
        )
        .expect_err("denied source must fail before packet publication");
        assert_eq!(
            denied,
            PacketPreparationFailure::DeniedSource {
                identity: SOURCE_ID.to_owned(),
            }
        );

        let missing = source_snapshot(None);
        let missing = PreparedExecutionPacket::prepare_duplicate(
            missing,
            stylesheet.clone(),
            policy(),
            MultipleMatchPolicy::UseLast,
            request("missing", CancellationToken::new()),
            super::super::XML_LIMITS,
        )
        .expect_err("missing source must fail before packet publication");
        assert_eq!(
            missing,
            PacketPreparationFailure::Preparation(PreparationFailure::MissingResource {
                identity: SOURCE_ID.to_owned(),
            })
        );

        let invalid = source_snapshot(Some(b"<broken></mismatch>"));
        let failure = PreparedExecutionPacket::prepare_duplicate(
            invalid,
            stylesheet,
            policy(),
            MultipleMatchPolicy::UseLast,
            request("invalid", CancellationToken::new()),
            super::super::XML_LIMITS,
        )
        .expect_err("invalid XML must fail before packet publication");
        assert!(matches!(
            failure,
            PacketPreparationFailure::Preparation(PreparationFailure::InvalidXml {
                identity,
                ..
            }) if identity == SOURCE_ID
        ));
    }

    #[test]
    fn cancellation_while_queued_is_observed_by_the_same_invocation_control() {
        let snapshot = snapshot();
        let stylesheet = Arc::new(
            super::super::compile_resource(&snapshot, STYLE_ID)
                .expect("compile AR-0020 stylesheet"),
        );
        let cancellation = CancellationToken::new();
        let packet = PreparedExecutionPacket::prepare_duplicate(
            snapshot,
            stylesheet,
            policy(),
            MultipleMatchPolicy::UseLast,
            request("queued-cancellation", cancellation.clone()),
            super::super::XML_LIMITS,
        )
        .expect("prepare packet before cancellation");
        let capacity = packet.known_prepared_capacity_bytes;
        let mut queue = BoundedReadyQueue::new(ReadyQueueLimits {
            packet_count: 1,
            known_prepared_capacity_bytes: capacity,
        });
        queue
            .push(packet)
            .expect("admit packet before cancellation");

        cancellation.cancel();
        let failure = queue
            .pop()
            .expect("take cancelled packet")
            .execute()
            .expect_err("queued cancellation must reach execution");
        assert_eq!(failure.category, FailureCategory::Cancelled);
        assert_eq!(failure.request_id.as_deref(), Some("queued-cancellation"));
        assert!(failure.work_domain.is_some());
        assert_eq!(queue.observe(), ReadyQueueObservation::default());
        let pressure = queue.observe_pressure();
        assert_eq!(pressure.executing_packet_count, 0);
        assert_eq!(pressure.executing_prepared_capacity_bytes, 0);
    }

    #[test]
    fn packet_retains_only_its_original_resource_generation() {
        let original = snapshot();
        let replacement = snapshot();
        let stylesheet = Arc::new(
            super::super::compile_resource(&original, STYLE_ID)
                .expect("compile AR-0020 stylesheet"),
        );
        let packet = prepare_packet(original.clone(), stylesheet, "generation")
            .expect("prepare original-generation packet");

        assert!(packet.snapshot.same_generation(&original));
        assert!(!packet.snapshot.same_generation(&replacement));
        drop(original);
        assert_eq!(
            packet
                .execute()
                .expect("retained original generation remains executable")
                .result_id,
            "result:generation"
        );
    }

    #[test]
    #[ignore = "manual release-mode AR-0020 raw preparation/execution phase probe"]
    fn measures_pinned_for004_raw_phase_ratio() {
        const SAMPLES: usize = 51;
        const STYLE_ID: &str = "urn:w3c:xslt30:for-004:stylesheet";
        const SOURCE_ID: &str = "urn:w3c:xslt30:for-004:generated-source";

        for items in [5, 50, 500] {
            let mut source = String::from("<order>");
            for _ in 0..items {
                source.push_str("<order-item price='1.00' qty='1'/>");
            }
            source.push_str("</order>");
            let stylesheet =
                include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl");
            let maximum_resource_bytes = source.len().max(stylesheet.len());
            let mut resources = ResourceSetBuilder::new(ResourceLimits::new(
                2,
                maximum_resource_bytes,
                source.len() + stylesheet.len(),
            ));
            resources
                .admit(SOURCE_ID, source.into_bytes())
                .expect("admit generated for-004 source");
            resources
                .admit(STYLE_ID, stylesheet.to_vec())
                .expect("admit pinned for-004 stylesheet");
            let snapshot = resources.seal();
            let stylesheet = Arc::new(
                super::super::compile_resource(&snapshot, STYLE_ID)
                    .expect("compile pinned for-004 stylesheet"),
            );
            let mut preparation = Vec::with_capacity(SAMPLES);
            let mut execution = Vec::with_capacity(SAMPLES);
            let mut known_prepared_capacity_bytes = 0;
            for sample in 0..SAMPLES {
                let packet = PreparedExecutionPacket::prepare_duplicate(
                    snapshot.clone(),
                    stylesheet.clone(),
                    policy(),
                    MultipleMatchPolicy::UseLast,
                    TransformRequest {
                        identity: format!("for004-{items}-{sample}"),
                        result_identity: format!("result:for004-{items}-{sample}"),
                        entry: InvocationEntry::PrincipalSource {
                            resource: SOURCE_ID.to_owned(),
                        },
                        parameters: BTreeMap::default(),
                        cancellation: CancellationToken::new(),
                        cancellation_fault: None,
                    },
                    super::super::XML_LIMITS,
                )
                .expect("prepare measured for-004 packet");
                known_prepared_capacity_bytes = packet.known_prepared_capacity_bytes;
                preparation.push(packet.preparation_service_time);
                let execution_started = Instant::now();
                let result = packet.execute().expect("execute measured for-004 packet");
                execution.push(execution_started.elapsed());
                let mut expected = String::new();
                write!(&mut expected, "<out>{items}.00</out>").expect("format expected result");
                assert!(
                    result.serialized.contains(&expected),
                    "generated for-004 semantic sentinel"
                );
            }
            preparation.sort_unstable();
            execution.sort_unstable();
            let preparation_median = preparation[SAMPLES / 2];
            let execution_median = execution[SAMPLES / 2];
            println!(
                "items={items} samples={SAMPLES} prepared_capacity_bytes={known_prepared_capacity_bytes} preparation_median_us={:.3} execution_median_us={:.3} preparation_to_execution_ratio={:.3}",
                preparation_median.as_secs_f64() * 1_000_000.0,
                execution_median.as_secs_f64() * 1_000_000.0,
                preparation_median.as_secs_f64() / execution_median.as_secs_f64(),
            );
        }
    }
}
