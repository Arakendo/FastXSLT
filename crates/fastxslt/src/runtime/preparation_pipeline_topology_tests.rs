//! Threaded AR-0020 mechanics over the topology-free packet reference.

use std::{
    collections::{BTreeMap, HashSet},
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::resources::{ResourceLimits, ResourceSetBuilder, ResourceSnapshot};

use super::preparation_pipeline_experiment::{
    AssignedExecutionPacket, BoundedReadyQueue, PipelinePressureObservation,
    PreparedExecutionPacket, ReadyQueueLimits,
};
use super::transform_set_experiment::{ExecutionPolicy, InvocationEntry, TransformRequest};
use super::{MultipleMatchPolicy, XML_LIMITS, compile_resource};

const SOURCE_ID: &str = "urn:fastxslt:ar-0020:threaded-source";
const STYLE_ID: &str = "urn:fastxslt:ar-0020:threaded-stylesheet";

struct BlockingReadyQueue {
    limits: ReadyQueueLimits,
    queue: Mutex<BoundedReadyQueue>,
    not_empty: Condvar,
    not_full: Condvar,
}

impl BlockingReadyQueue {
    fn new(limits: ReadyQueueLimits) -> Self {
        Self {
            limits,
            queue: Mutex::new(BoundedReadyQueue::new(limits)),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        }
    }

    fn push(&self, packet: PreparedExecutionPacket) {
        assert!(self.limits.packet_count > 0);
        assert!(
            packet.known_prepared_capacity_bytes <= self.limits.known_prepared_capacity_bytes,
            "one packet must fit the configured byte envelope"
        );
        let mut packet = Some(packet);
        let mut queue = self.queue.lock().expect("blocking ready queue lock");
        loop {
            let observed = queue.observe();
            let fits_count = observed.packet_count < self.limits.packet_count;
            let fits_bytes = observed.known_prepared_capacity_bytes.saturating_add(
                packet
                    .as_ref()
                    .expect("packet remains owned while blocked")
                    .known_prepared_capacity_bytes,
            ) <= self.limits.known_prepared_capacity_bytes;
            if fits_count && fits_bytes {
                queue
                    .push(packet.take().expect("publish packet exactly once"))
                    .expect("preflight and admission share the queue lock");
                self.not_empty.notify_one();
                return;
            }
            queue = self
                .not_full
                .wait(queue)
                .expect("blocking ready queue lock");
        }
    }

    fn pop(&self) -> AssignedExecutionPacket {
        self.pop_observed().0
    }

    fn pop_observed(&self) -> (AssignedExecutionPacket, Duration, usize) {
        let mut queue = self.queue.lock().expect("blocking ready queue lock");
        let mut wait_time = Duration::ZERO;
        let mut waits = 0;
        loop {
            if let Some(packet) = queue.pop() {
                self.not_full.notify_all();
                return (packet, wait_time, waits);
            }
            let started = Instant::now();
            queue = self
                .not_empty
                .wait(queue)
                .expect("blocking ready queue lock");
            wait_time += started.elapsed();
            waits += 1;
        }
    }

    fn observe(&self) -> PipelinePressureObservation {
        self.queue
            .lock()
            .expect("blocking ready queue lock")
            .observe_pressure()
    }
}

fn snapshot(items: usize) -> ResourceSnapshot {
    let mut source = String::from("<order>");
    for _ in 0..items {
        source.push_str("<order-item price='1.00' qty='1'/>");
    }
    source.push_str("</order>");
    let stylesheet = include_bytes!("../../../../vendor/xslt30-test/tests/expr/for/for-004.xsl");
    let maximum_resource_bytes = source.len().max(stylesheet.len());
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(
        2,
        maximum_resource_bytes,
        source.len() + stylesheet.len(),
    ));
    resources
        .admit(SOURCE_ID, source.into_bytes())
        .expect("admit threaded source");
    resources
        .admit(STYLE_ID, stylesheet.to_vec())
        .expect("admit threaded stylesheet");
    resources.seal()
}

fn policy() -> ExecutionPolicy {
    ExecutionPolicy {
        denied_sources: HashSet::new(),
        serialized_byte_limit: 4_096,
        work_limits: WorkLimits::unbounded(),
    }
}

fn request(index: usize) -> TransformRequest {
    TransformRequest {
        identity: format!("threaded-{index}"),
        result_identity: format!("result:threaded-{index}"),
        entry: InvocationEntry::PrincipalSource {
            resource: SOURCE_ID.to_owned(),
        },
        parameters: BTreeMap::default(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    }
}

#[derive(Debug)]
struct TopologyRun {
    elapsed: Duration,
    consumer_wait: Duration,
    consumer_waits: usize,
    pressure: PipelinePressureObservation,
}

#[derive(Debug)]
struct EqualBudgetRun {
    elapsed: Duration,
    latencies: Vec<Duration>,
    consumer_wait: Duration,
    total_prepared_high_water: usize,
}

fn prepare(
    snapshot: ResourceSnapshot,
    stylesheet: Arc<crate::xslt::golden_semantics_experiment::StylesheetProgram>,
    index: usize,
) -> PreparedExecutionPacket {
    PreparedExecutionPacket::prepare_duplicate(
        snapshot,
        stylesheet,
        policy(),
        MultipleMatchPolicy::UseLast,
        request(index),
        XML_LIMITS,
    )
    .expect("prepare topology packet")
}

fn assert_result(result: &super::transform_set_experiment::ResultEntry, items: usize) {
    assert!(
        result
            .serialized
            .contains(&format!("<out>{items}.00</out>"))
    );
}

fn run_direct(
    snapshot: &ResourceSnapshot,
    stylesheet: &Arc<crate::xslt::golden_semantics_experiment::StylesheetProgram>,
    items: usize,
    requests: usize,
) -> Duration {
    let started = Instant::now();
    for index in 0..requests {
        let result = prepare(snapshot.clone(), Arc::clone(stylesheet), index)
            .execute()
            .expect("execute direct topology reference");
        assert_result(&result, items);
    }
    started.elapsed()
}

fn run_pipelined(
    snapshot: &ResourceSnapshot,
    stylesheet: &Arc<crate::xslt::golden_semantics_experiment::StylesheetProgram>,
    items: usize,
    requests: usize,
    preparer_count: usize,
    queue_depth: usize,
    packet_capacity: usize,
) -> TopologyRun {
    let queue = Arc::new(BlockingReadyQueue::new(ReadyQueueLimits {
        packet_count: queue_depth,
        known_prepared_capacity_bytes: packet_capacity * queue_depth,
    }));
    let next = Arc::new(AtomicUsize::new(0));
    let started = Instant::now();
    let (consumer_wait, consumer_waits) = thread::scope(|scope| {
        for _ in 0..preparer_count {
            let queue = Arc::clone(&queue);
            let next = Arc::clone(&next);
            let snapshot = snapshot.clone();
            let stylesheet = Arc::clone(stylesheet);
            scope.spawn(move || {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    if index >= requests {
                        break;
                    }
                    queue.push(prepare(snapshot.clone(), stylesheet.clone(), index));
                }
            });
        }

        let mut consumer_wait = Duration::ZERO;
        let mut consumer_waits = 0;
        for _ in 0..requests {
            let (packet, wait, waits) = queue.pop_observed();
            consumer_wait += wait;
            consumer_waits += waits;
            let result = packet.execute().expect("execute pipelined topology packet");
            assert_result(&result, items);
        }
        (consumer_wait, consumer_waits)
    });
    TopologyRun {
        elapsed: started.elapsed(),
        consumer_wait,
        consumer_waits,
        pressure: queue.observe(),
    }
}

fn run_combined_budget(
    snapshot: &ResourceSnapshot,
    stylesheet: &Arc<crate::xslt::golden_semantics_experiment::StylesheetProgram>,
    items: usize,
    requests: usize,
    workers: usize,
) -> EqualBudgetRun {
    let next = AtomicUsize::new(0);
    let active_prepared = AtomicUsize::new(0);
    let prepared_high_water = AtomicUsize::new(0);
    let latencies = Mutex::new(Vec::with_capacity(requests));
    let started = Instant::now();
    thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    if index >= requests {
                        break;
                    }
                    let request_started = Instant::now();
                    let packet = prepare(snapshot.clone(), Arc::clone(stylesheet), index);
                    let charge = packet.known_prepared_capacity_bytes;
                    let current = active_prepared.fetch_add(charge, Ordering::Relaxed) + charge;
                    prepared_high_water.fetch_max(current, Ordering::Relaxed);
                    let result = packet.execute().expect("execute combined worker packet");
                    active_prepared.fetch_sub(charge, Ordering::Relaxed);
                    assert_result(&result, items);
                    latencies
                        .lock()
                        .expect("combined latency lock")
                        .push(request_started.elapsed());
                }
            });
        }
    });
    EqualBudgetRun {
        elapsed: started.elapsed(),
        latencies: latencies.into_inner().expect("combined latency ownership"),
        consumer_wait: Duration::ZERO,
        total_prepared_high_water: prepared_high_water.load(Ordering::Relaxed),
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "the private topology probe keeps its fixed workload and pool dimensions explicit"
)]
fn run_staged_budget(
    snapshot: &ResourceSnapshot,
    stylesheet: &Arc<crate::xslt::golden_semantics_experiment::StylesheetProgram>,
    items: usize,
    requests: usize,
    preparers: usize,
    executors: usize,
    queue_depth: usize,
    packet_capacity: usize,
) -> EqualBudgetRun {
    let queue = Arc::new(BlockingReadyQueue::new(ReadyQueueLimits {
        packet_count: queue_depth,
        known_prepared_capacity_bytes: packet_capacity * queue_depth,
    }));
    let next_preparation = AtomicUsize::new(0);
    let next_execution = AtomicUsize::new(0);
    let latencies = Mutex::new(Vec::with_capacity(requests));
    let consumer_wait = Mutex::new(Duration::ZERO);
    let started = Instant::now();
    thread::scope(|scope| {
        let next_preparation = &next_preparation;
        let next_execution = &next_execution;
        let latencies = &latencies;
        let consumer_wait = &consumer_wait;
        for _ in 0..preparers {
            let queue = Arc::clone(&queue);
            let snapshot = snapshot.clone();
            let stylesheet = Arc::clone(stylesheet);
            scope.spawn(move || {
                loop {
                    let index = next_preparation.fetch_add(1, Ordering::Relaxed);
                    if index >= requests {
                        break;
                    }
                    queue.push(prepare(snapshot.clone(), stylesheet.clone(), index));
                }
            });
        }
        for _ in 0..executors {
            let queue = Arc::clone(&queue);
            scope.spawn(move || {
                loop {
                    let index = next_execution.fetch_add(1, Ordering::Relaxed);
                    if index >= requests {
                        break;
                    }
                    let (packet, wait, _) = queue.pop_observed();
                    let observed = packet.execute_observed();
                    let request_latency = observed.phases.preparation_service
                        + observed.phases.ready_queue_residence
                        + observed.phases.execution_service;
                    let result = observed.result.expect("execute staged worker packet");
                    assert_result(&result, items);
                    latencies
                        .lock()
                        .expect("staged latency lock")
                        .push(request_latency);
                    *consumer_wait.lock().expect("consumer wait lock") += wait;
                }
            });
        }
    });
    let pressure = queue.observe();
    EqualBudgetRun {
        elapsed: started.elapsed(),
        latencies: latencies.into_inner().expect("staged latency ownership"),
        consumer_wait: consumer_wait.into_inner().expect("consumer wait ownership"),
        total_prepared_high_water: pressure.high_water_total_prepared_capacity_bytes,
    }
}

fn percentile(latencies: &[Duration], percentile: usize) -> Duration {
    let index = (latencies.len() - 1) * percentile / 100;
    latencies[index]
}

fn report_equal_budget(
    name: &str,
    items: usize,
    total_threads: usize,
    requests: usize,
    samples: usize,
    mut runs: Vec<EqualBudgetRun>,
) {
    runs.sort_by_key(|run| run.elapsed);
    let run = &mut runs[samples / 2];
    run.latencies.sort_unstable();
    let request_count = f64::from(u32::try_from(requests).expect("request count fits u32"));
    println!(
        "items={items} topology={name} total_threads={total_threads} requests={requests} samples={samples} throughput_per_second={:.1} p50_us={:.1} p95_us={:.1} p99_us={:.1} consumer_wait_us={:.1} prepared_high_water_bytes={}",
        request_count / run.elapsed.as_secs_f64(),
        percentile(&run.latencies, 50).as_secs_f64() * 1_000_000.0,
        percentile(&run.latencies, 95).as_secs_f64() * 1_000_000.0,
        percentile(&run.latencies, 99).as_secs_f64() * 1_000_000.0,
        run.consumer_wait.as_secs_f64() * 1_000_000.0,
        run.total_prepared_high_water,
    );
}

#[derive(Clone)]
struct MixedWorkItem {
    snapshot: ResourceSnapshot,
    items: usize,
    identity: usize,
}

#[derive(Debug)]
struct MixedRun {
    elapsed: Duration,
    latencies: Vec<(usize, Duration)>,
    prepared_high_water: usize,
}

fn mixed_work(interleaved: bool) -> Vec<MixedWorkItem> {
    const CYCLES: usize = 256;
    let small = snapshot(5);
    let medium = snapshot(50);
    let large = snapshot(500);
    let mut sizes = Vec::with_capacity(CYCLES * 8);
    if interleaved {
        for _ in 0..CYCLES {
            sizes.extend([5, 5, 50, 5, 50, 5, 50, 500]);
        }
    } else {
        sizes.extend(std::iter::repeat_n(5, CYCLES * 4));
        sizes.extend(std::iter::repeat_n(50, CYCLES * 3));
        sizes.extend(std::iter::repeat_n(500, CYCLES));
    }
    sizes
        .into_iter()
        .enumerate()
        .map(|(identity, items)| MixedWorkItem {
            snapshot: match items {
                5 => small.clone(),
                50 => medium.clone(),
                500 => large.clone(),
                _ => unreachable!("fixed mixed workload size"),
            },
            items,
            identity,
        })
        .collect()
}

fn uniform_work(items: usize, requests: usize) -> Vec<MixedWorkItem> {
    let snapshot = snapshot(items);
    (0..requests)
        .map(|identity| MixedWorkItem {
            snapshot: snapshot.clone(),
            items,
            identity,
        })
        .collect()
}

fn prepare_mixed(
    item: &MixedWorkItem,
    stylesheet: Arc<crate::xslt::golden_semantics_experiment::StylesheetProgram>,
) -> PreparedExecutionPacket {
    prepare(item.snapshot.clone(), stylesheet, item.identity)
}

fn run_mixed_combined(
    work: &[MixedWorkItem],
    stylesheet: &Arc<crate::xslt::golden_semantics_experiment::StylesheetProgram>,
) -> MixedRun {
    let next = AtomicUsize::new(0);
    let active_prepared = AtomicUsize::new(0);
    let prepared_high_water = AtomicUsize::new(0);
    let latencies = Mutex::new(Vec::with_capacity(work.len()));
    let started = Instant::now();
    thread::scope(|scope| {
        for _ in 0..10 {
            scope.spawn(|| {
                loop {
                    let index = next.fetch_add(1, Ordering::Relaxed);
                    let Some(item) = work.get(index) else { break };
                    let request_started = Instant::now();
                    let packet = prepare_mixed(item, Arc::clone(stylesheet));
                    let charge = packet.known_prepared_capacity_bytes;
                    let current = active_prepared.fetch_add(charge, Ordering::Relaxed) + charge;
                    prepared_high_water.fetch_max(current, Ordering::Relaxed);
                    let result = packet.execute().expect("execute mixed combined packet");
                    active_prepared.fetch_sub(charge, Ordering::Relaxed);
                    assert_result(&result, item.items);
                    latencies
                        .lock()
                        .expect("mixed combined latency lock")
                        .push((item.items, request_started.elapsed()));
                }
            });
        }
    });
    MixedRun {
        elapsed: started.elapsed(),
        latencies: latencies
            .into_inner()
            .expect("mixed combined latency ownership"),
        prepared_high_water: prepared_high_water.load(Ordering::Relaxed),
    }
}

fn run_mixed_staged(
    work: &[MixedWorkItem],
    stylesheet: &Arc<crate::xslt::golden_semantics_experiment::StylesheetProgram>,
    preparers: usize,
    executors: usize,
    ready_byte_limit: usize,
) -> MixedRun {
    let queue = Arc::new(BlockingReadyQueue::new(ReadyQueueLimits {
        packet_count: 10,
        known_prepared_capacity_bytes: ready_byte_limit,
    }));
    let next_preparation = AtomicUsize::new(0);
    let next_execution = AtomicUsize::new(0);
    let latencies = Mutex::new(Vec::with_capacity(work.len()));
    let started = Instant::now();
    thread::scope(|scope| {
        let next_preparation = &next_preparation;
        let next_execution = &next_execution;
        let latencies = &latencies;
        for _ in 0..preparers {
            let queue = Arc::clone(&queue);
            scope.spawn(move || {
                loop {
                    let index = next_preparation.fetch_add(1, Ordering::Relaxed);
                    let Some(item) = work.get(index) else { break };
                    queue.push(prepare_mixed(item, Arc::clone(stylesheet)));
                }
            });
        }
        for _ in 0..executors {
            let queue = Arc::clone(&queue);
            scope.spawn(move || {
                loop {
                    let index = next_execution.fetch_add(1, Ordering::Relaxed);
                    if index >= work.len() {
                        break;
                    }
                    let observed = queue.pop().execute_observed();
                    let result = observed.result.expect("execute mixed staged packet");
                    let items = result
                        .serialized
                        .contains("<out>500.00</out>")
                        .then_some(500)
                        .or_else(|| result.serialized.contains("<out>50.00</out>").then_some(50))
                        .unwrap_or(5);
                    assert_result(&result, items);
                    latencies.lock().expect("mixed staged latency lock").push((
                        items,
                        observed.phases.preparation_service
                            + observed.phases.ready_queue_residence
                            + observed.phases.execution_service,
                    ));
                }
            });
        }
    });
    MixedRun {
        elapsed: started.elapsed(),
        latencies: latencies
            .into_inner()
            .expect("mixed staged latency ownership"),
        prepared_high_water: queue.observe().high_water_total_prepared_capacity_bytes,
    }
}

fn report_mixed(name: &str, order: &str, samples: usize, mut runs: Vec<MixedRun>) {
    runs.sort_by_key(|run| run.elapsed);
    let run = &runs[samples / 2];
    let request_count =
        f64::from(u32::try_from(run.latencies.len()).expect("mixed request count fits u32"));
    let throughput = request_count / run.elapsed.as_secs_f64();
    let latency = |items| {
        let mut values: Vec<_> = run
            .latencies
            .iter()
            .filter_map(|(candidate, latency)| (*candidate == items).then_some(*latency))
            .collect();
        values.sort_unstable();
        percentile(&values, 95).as_secs_f64() * 1_000_000.0
    };
    println!(
        "order={order} topology={name} samples={samples} throughput_per_second={throughput:.1} small_p95_us={:.1} medium_p95_us={:.1} large_p95_us={:.1} prepared_high_water_bytes={}",
        latency(5),
        latency(50),
        latency(500),
        run.prepared_high_water,
    );
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TrialAssessment {
    accepted: bool,
    throughput_ratio: f64,
    worst_tail_ratio: f64,
    prepared_high_water_ratio: f64,
}

fn p95_for_items(run: &MixedRun, items: usize) -> Option<Duration> {
    let mut values: Vec<_> = run
        .latencies
        .iter()
        .filter_map(|(candidate, latency)| (*candidate == items).then_some(*latency))
        .collect();
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    Some(percentile(&values, 95))
}

fn assess_trial(
    combined: &MixedRun,
    staged: &MixedRun,
    minimum_throughput_ratio: f64,
    maximum_tail_ratio: f64,
    maximum_prepared_high_water_ratio: f64,
) -> TrialAssessment {
    let throughput_ratio = combined.elapsed.as_secs_f64() / staged.elapsed.as_secs_f64();
    let worst_tail_ratio = [5, 50, 500]
        .into_iter()
        .filter_map(|items| {
            let combined = p95_for_items(combined, items)?;
            let staged = p95_for_items(staged, items)?;
            Some(staged.as_secs_f64() / combined.as_secs_f64())
        })
        .fold(0.0_f64, f64::max);
    let prepared_high_water_ratio = if combined.prepared_high_water == 0 {
        f64::INFINITY
    } else {
        f64::from(
            u32::try_from(staged.prepared_high_water)
                .expect("trial staged prepared high-water fits u32"),
        ) / f64::from(
            u32::try_from(combined.prepared_high_water)
                .expect("trial combined prepared high-water fits u32"),
        )
    };
    TrialAssessment {
        accepted: throughput_ratio >= minimum_throughput_ratio
            && worst_tail_ratio <= maximum_tail_ratio
            && prepared_high_water_ratio <= maximum_prepared_high_water_ratio,
        throughput_ratio,
        worst_tail_ratio,
        prepared_high_water_ratio,
    }
}

fn median_mixed_run(mut runs: Vec<MixedRun>) -> MixedRun {
    runs.sort_by_key(|run| run.elapsed);
    runs.swap_remove(runs.len() / 2)
}

#[test]
fn bounded_trial_requires_gain_without_buying_tail_or_memory_regression() {
    let run = |elapsed_ms, p95_ms, prepared_high_water| MixedRun {
        elapsed: Duration::from_millis(elapsed_ms),
        latencies: (0..20)
            .map(|_| (50, Duration::from_millis(p95_ms)))
            .collect(),
        prepared_high_water,
    };
    let combined = run(100, 10, 1_000);

    assert!(assess_trial(&combined, &run(90, 11, 1_200), 1.05, 1.25, 1.5).accepted);
    assert!(!assess_trial(&combined, &run(98, 10, 1_000), 1.05, 1.25, 1.5).accepted);
    assert!(!assess_trial(&combined, &run(90, 13, 1_000), 1.05, 1.25, 1.5).accepted);
    assert!(!assess_trial(&combined, &run(90, 10, 1_600), 1.05, 1.25, 1.5).accepted);
}

#[test]
fn one_transform_worker_accepts_one_or_many_bounded_preparation_workers() {
    const REQUESTS: usize = 32;
    const QUEUE_DEPTH: usize = 2;

    for preparer_count in [1, 4] {
        let snapshot = snapshot(50);
        let stylesheet =
            Arc::new(compile_resource(&snapshot, STYLE_ID).expect("compile threaded stylesheet"));
        let probe = PreparedExecutionPacket::prepare_duplicate(
            snapshot.clone(),
            stylesheet.clone(),
            policy(),
            MultipleMatchPolicy::UseLast,
            request(usize::MAX),
            XML_LIMITS,
        )
        .expect("prepare capacity probe");
        let packet_capacity = probe.known_prepared_capacity_bytes;
        drop(probe);
        let queue = Arc::new(BlockingReadyQueue::new(ReadyQueueLimits {
            packet_count: QUEUE_DEPTH,
            known_prepared_capacity_bytes: packet_capacity * QUEUE_DEPTH,
        }));
        let next = Arc::new(AtomicUsize::new(0));

        thread::scope(|scope| {
            for _ in 0..preparer_count {
                let queue = Arc::clone(&queue);
                let next = Arc::clone(&next);
                let snapshot = snapshot.clone();
                let stylesheet = Arc::clone(&stylesheet);
                scope.spawn(move || {
                    loop {
                        let index = next.fetch_add(1, Ordering::Relaxed);
                        if index >= REQUESTS {
                            break;
                        }
                        let packet = PreparedExecutionPacket::prepare_duplicate(
                            snapshot.clone(),
                            stylesheet.clone(),
                            policy(),
                            MultipleMatchPolicy::UseLast,
                            request(index),
                            XML_LIMITS,
                        )
                        .expect("prepare threaded packet");
                        queue.push(packet);
                    }
                });
            }

            for _ in 0..REQUESTS {
                let result = queue
                    .pop()
                    .execute()
                    .expect("execute on the single transform worker");
                assert!(result.serialized.contains("<out>50.00</out>"));
            }
        });

        let observed = queue.observe();
        assert_eq!(observed.ready.packet_count, 0);
        assert_eq!(observed.executing_packet_count, 0);
        assert!(observed.high_water_ready_packet_count <= QUEUE_DEPTH);
        assert!(observed.high_water_ready_prepared_capacity_bytes <= packet_capacity * QUEUE_DEPTH);
        assert_eq!(observed.high_water_executing_packet_count, 1);
        assert!(
            observed.high_water_total_prepared_capacity_bytes
                <= packet_capacity * (QUEUE_DEPTH + 1)
        );
    }
}

#[test]
#[ignore = "manual release-mode AR-0020 A/B/C topology throughput comparison"]
fn measures_direct_and_single_transform_worker_pipeline_topologies() {
    const REQUESTS: usize = 256;
    const SAMPLES: usize = 7;
    const QUEUE_DEPTH: usize = 2;

    for items in [50, 500] {
        let snapshot = snapshot(items);
        let stylesheet =
            Arc::new(compile_resource(&snapshot, STYLE_ID).expect("compile topology stylesheet"));
        let probe = prepare(snapshot.clone(), stylesheet.clone(), usize::MAX);
        let packet_capacity = probe.known_prepared_capacity_bytes;
        drop(probe);

        let mut direct = Vec::with_capacity(SAMPLES);
        let mut pipeline_one = Vec::with_capacity(SAMPLES);
        let mut pipeline_four = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            direct.push(run_direct(&snapshot, &stylesheet, items, REQUESTS));
            pipeline_one.push(run_pipelined(
                &snapshot,
                &stylesheet,
                items,
                REQUESTS,
                1,
                QUEUE_DEPTH,
                packet_capacity,
            ));
            pipeline_four.push(run_pipelined(
                &snapshot,
                &stylesheet,
                items,
                REQUESTS,
                4,
                QUEUE_DEPTH,
                packet_capacity,
            ));
        }
        direct.sort_unstable();
        pipeline_one.sort_by_key(|run| run.elapsed);
        pipeline_four.sort_by_key(|run| run.elapsed);
        let direct = direct[SAMPLES / 2];
        let pipeline_one = &pipeline_one[SAMPLES / 2];
        let pipeline_four = &pipeline_four[SAMPLES / 2];
        let request_count =
            f64::from(u32::try_from(REQUESTS).expect("measurement request count fits u32"));
        let throughput = |elapsed: Duration| request_count / elapsed.as_secs_f64();
        println!(
            "items={items} requests={REQUESTS} samples={SAMPLES} packet_capacity_bytes={packet_capacity} direct_per_second={:.1} pipeline_one_per_second={:.1} pipeline_one_speedup={:.3} pipeline_one_consumer_wait_us={:.1} pipeline_one_waits={} pipeline_one_high_water_bytes={} pipeline_four_per_second={:.1} pipeline_four_speedup={:.3} pipeline_four_consumer_wait_us={:.1} pipeline_four_waits={} pipeline_four_high_water_bytes={}",
            throughput(direct),
            throughput(pipeline_one.elapsed),
            direct.as_secs_f64() / pipeline_one.elapsed.as_secs_f64(),
            pipeline_one.consumer_wait.as_secs_f64() * 1_000_000.0,
            pipeline_one.consumer_waits,
            pipeline_one
                .pressure
                .high_water_total_prepared_capacity_bytes,
            throughput(pipeline_four.elapsed),
            direct.as_secs_f64() / pipeline_four.elapsed.as_secs_f64(),
            pipeline_four.consumer_wait.as_secs_f64() * 1_000_000.0,
            pipeline_four.consumer_waits,
            pipeline_four
                .pressure
                .high_water_total_prepared_capacity_bytes,
        );
    }
}

#[test]
#[ignore = "manual release-mode AR-0020 equal-thread-budget topology comparison"]
fn measures_equal_ten_thread_combined_and_staged_topologies() {
    const REQUESTS: usize = 256;
    const SAMPLES: usize = 5;
    const TOTAL_THREADS: usize = 10;
    const QUEUE_DEPTH: usize = 10;

    for items in [50, 500] {
        let snapshot = snapshot(items);
        let stylesheet =
            Arc::new(compile_resource(&snapshot, STYLE_ID).expect("compile topology stylesheet"));
        let probe = prepare(snapshot.clone(), stylesheet.clone(), usize::MAX);
        let packet_capacity = probe.known_prepared_capacity_bytes;
        drop(probe);

        let mut combined = Vec::with_capacity(SAMPLES);
        let mut staged_9_1 = Vec::with_capacity(SAMPLES);
        let mut staged_8_2 = Vec::with_capacity(SAMPLES);
        let mut staged_7_3 = Vec::with_capacity(SAMPLES);
        let mut staged_5_5 = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            combined.push(run_combined_budget(
                &snapshot,
                &stylesheet,
                items,
                REQUESTS,
                TOTAL_THREADS,
            ));
            for ((preparers, executors), runs) in [
                ((9, 1), &mut staged_9_1),
                ((8, 2), &mut staged_8_2),
                ((7, 3), &mut staged_7_3),
                ((5, 5), &mut staged_5_5),
            ] {
                runs.push(run_staged_budget(
                    &snapshot,
                    &stylesheet,
                    items,
                    REQUESTS,
                    preparers,
                    executors,
                    QUEUE_DEPTH,
                    packet_capacity,
                ));
            }
        }

        for (name, runs) in [
            ("combined-10", combined),
            ("staged-9-1", staged_9_1),
            ("staged-8-2", staged_8_2),
            ("staged-7-3", staged_7_3),
            ("staged-5-5", staged_5_5),
        ] {
            report_equal_budget(name, items, TOTAL_THREADS, REQUESTS, SAMPLES, runs);
        }
    }
}

#[test]
#[ignore = "manual release-mode AR-0020 mixed-size topology comparison"]
fn measures_clustered_and_interleaved_mixed_size_batches() {
    const SAMPLES: usize = 7;

    let compile_snapshot = snapshot(5);
    let stylesheet = Arc::new(
        compile_resource(&compile_snapshot, STYLE_ID).expect("compile mixed topology stylesheet"),
    );
    let large_probe = prepare(snapshot(500), stylesheet.clone(), usize::MAX);
    let ready_byte_limit = large_probe.known_prepared_capacity_bytes * 3;
    drop(large_probe);

    for (order, interleaved) in [("clustered", false), ("interleaved", true)] {
        let work = mixed_work(interleaved);
        let mut combined = Vec::with_capacity(SAMPLES);
        let mut staged_8_2 = Vec::with_capacity(SAMPLES);
        let mut staged_7_3 = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            combined.push(run_mixed_combined(&work, &stylesheet));
            staged_8_2.push(run_mixed_staged(&work, &stylesheet, 8, 2, ready_byte_limit));
            staged_7_3.push(run_mixed_staged(&work, &stylesheet, 7, 3, ready_byte_limit));
        }
        report_mixed("combined-10", order, SAMPLES, combined);
        report_mixed("staged-8-2", order, SAMPLES, staged_8_2);
        report_mixed("staged-7-3", order, SAMPLES, staged_7_3);
    }
}

#[test]
#[ignore = "manual release-mode AR-0020 bounded staged-trial policy replay"]
fn measures_bounded_staged_trial_policy_replay() {
    const SAMPLES: usize = 7;
    const UNIFORM_REQUESTS: usize = 512;
    const MINIMUM_THROUGHPUT_RATIO: f64 = 1.05;
    const MAXIMUM_TAIL_RATIO: f64 = 1.25;
    const MAXIMUM_PREPARED_HIGH_WATER_RATIO: f64 = 1.5;

    let compile_snapshot = snapshot(5);
    let stylesheet =
        Arc::new(compile_resource(&compile_snapshot, STYLE_ID).expect("compile trial stylesheet"));
    let large_probe = prepare(snapshot(500), stylesheet.clone(), usize::MAX);
    let ready_byte_limit = large_probe.known_prepared_capacity_bytes * 3;
    drop(large_probe);

    for (name, work) in [
        ("uniform-50", uniform_work(50, UNIFORM_REQUESTS)),
        ("uniform-500", uniform_work(500, UNIFORM_REQUESTS)),
        ("mixed-clustered", mixed_work(false)),
        ("mixed-interleaved", mixed_work(true)),
    ] {
        let mut combined = Vec::with_capacity(SAMPLES);
        let mut staged = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            combined.push(run_mixed_combined(&work, &stylesheet));
            staged.push(run_mixed_staged(&work, &stylesheet, 7, 3, ready_byte_limit));
        }
        let combined = median_mixed_run(combined);
        let staged = median_mixed_run(staged);
        let assessment = assess_trial(
            &combined,
            &staged,
            MINIMUM_THROUGHPUT_RATIO,
            MAXIMUM_TAIL_RATIO,
            MAXIMUM_PREPARED_HIGH_WATER_RATIO,
        );
        let request_count =
            f64::from(u32::try_from(work.len()).expect("trial request count fits u32"));
        println!(
            "workload={name} samples={SAMPLES} trial=staged-7-3 accepted={} combined_per_second={:.1} staged_per_second={:.1} throughput_ratio={:.3} worst_tail_ratio={:.3} prepared_high_water_ratio={:.3} rejected_trial_exposure_ms={:.3}",
            assessment.accepted,
            request_count / combined.elapsed.as_secs_f64(),
            request_count / staged.elapsed.as_secs_f64(),
            assessment.throughput_ratio,
            assessment.worst_tail_ratio,
            assessment.prepared_high_water_ratio,
            if assessment.accepted {
                0.0
            } else {
                staged.elapsed.as_secs_f64() * 1_000.0
            },
        );
    }
}
