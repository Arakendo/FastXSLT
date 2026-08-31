//! Private AR-0010 experiment for invocation-local cooperative control.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WorkDomain {
    XmlEvent,
    XdmNode,
    XdmStringValueNode,
    XPathNodeVisit,
    XPathOperation,
    XsltInstruction,
    XsltTemplateCandidate,
    ResultNode,
    ResultTextByte,
    SerializedByte,
}

impl WorkDomain {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::XmlEvent => "xml-event",
            Self::XdmNode => "xdm-node",
            Self::XdmStringValueNode => "xdm-string-value-node",
            Self::XPathNodeVisit => "xpath-node-visit",
            Self::XPathOperation => "xpath-operation",
            Self::XsltInstruction => "xslt-instruction",
            Self::XsltTemplateCandidate => "xslt-template-candidate",
            Self::ResultNode => "result-node",
            Self::ResultTextByte => "result-text-byte",
            Self::SerializedByte => "serialized-byte",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WorkLimits {
    pub(crate) xml_events: usize,
    pub(crate) xdm_nodes: usize,
    pub(crate) xdm_string_value_nodes: usize,
    pub(crate) xpath_node_visits: usize,
    pub(crate) xpath_operations: usize,
    pub(crate) xslt_instructions: usize,
    pub(crate) xslt_template_candidates: usize,
    pub(crate) result_nodes: usize,
    pub(crate) result_text_bytes: usize,
    pub(crate) serialized_bytes: usize,
}

impl WorkLimits {
    pub(crate) const fn unbounded() -> Self {
        Self {
            xml_events: usize::MAX,
            xdm_nodes: usize::MAX,
            xdm_string_value_nodes: usize::MAX,
            xpath_node_visits: usize::MAX,
            xpath_operations: usize::MAX,
            xslt_instructions: usize::MAX,
            xslt_template_candidates: usize::MAX,
            result_nodes: usize::MAX,
            result_text_bytes: usize::MAX,
            serialized_bytes: usize::MAX,
        }
    }

    const fn limit(self, domain: WorkDomain) -> usize {
        match domain {
            WorkDomain::XmlEvent => self.xml_events,
            WorkDomain::XdmNode => self.xdm_nodes,
            WorkDomain::XdmStringValueNode => self.xdm_string_value_nodes,
            WorkDomain::XPathNodeVisit => self.xpath_node_visits,
            WorkDomain::XPathOperation => self.xpath_operations,
            WorkDomain::XsltInstruction => self.xslt_instructions,
            WorkDomain::XsltTemplateCandidate => self.xslt_template_candidates,
            WorkDomain::ResultNode => self.result_nodes,
            WorkDomain::ResultTextByte => self.result_text_bytes,
            WorkDomain::SerializedByte => self.serialized_bytes,
        }
    }

    fn remaining_mut(&mut self, domain: WorkDomain) -> &mut usize {
        match domain {
            WorkDomain::XmlEvent => &mut self.xml_events,
            WorkDomain::XdmNode => &mut self.xdm_nodes,
            WorkDomain::XdmStringValueNode => &mut self.xdm_string_value_nodes,
            WorkDomain::XPathNodeVisit => &mut self.xpath_node_visits,
            WorkDomain::XPathOperation => &mut self.xpath_operations,
            WorkDomain::XsltInstruction => &mut self.xslt_instructions,
            WorkDomain::XsltTemplateCandidate => &mut self.xslt_template_candidates,
            WorkDomain::ResultNode => &mut self.result_nodes,
            WorkDomain::ResultTextByte => &mut self.result_text_bytes,
            WorkDomain::SerializedByte => &mut self.serialized_bytes,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CancellationToken {
    cancelled: Arc<AtomicBool>,
    first_charge_barrier: Option<FirstChargeBarrier>,
}

#[derive(Debug, Clone)]
struct FirstChargeBarrier {
    observed: Arc<AtomicBool>,
    released: Arc<AtomicBool>,
    passed: Arc<AtomicBool>,
}

impl CancellationToken {
    pub(crate) fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            first_charge_barrier: None,
        }
    }

    pub(crate) fn with_first_charge_barrier() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            first_charge_barrier: Some(FirstChargeBarrier {
                observed: Arc::new(AtomicBool::new(false)),
                released: Arc::new(AtomicBool::new(false)),
                passed: Arc::new(AtomicBool::new(false)),
            }),
        }
    }

    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        if let Some(barrier) = &self.first_charge_barrier {
            barrier.released.store(true, Ordering::Release);
        }
    }

    pub(crate) fn first_charge_observed(&self) -> bool {
        self.first_charge_barrier
            .as_ref()
            .is_some_and(|barrier| barrier.observed.load(Ordering::Acquire))
    }

    fn is_cancelled(&self) -> bool {
        if let Some(barrier) = &self.first_charge_barrier
            && !barrier.passed.swap(true, Ordering::AcqRel)
        {
            barrier.observed.store(true, Ordering::Release);
            while !barrier.released.load(Ordering::Acquire) {
                std::thread::yield_now();
            }
        }
        self.cancelled.load(Ordering::Acquire)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ControlFailure {
    Cancelled {
        domain: WorkDomain,
    },
    BudgetExhausted {
        domain: WorkDomain,
        limit: usize,
        consumed: usize,
        attempted: usize,
    },
}

impl ControlFailure {
    pub(crate) const fn domain(&self) -> WorkDomain {
        match self {
            Self::Cancelled { domain } | Self::BudgetExhausted { domain, .. } => *domain,
        }
    }
}

#[derive(Debug)]
pub(crate) struct InvocationControl {
    cancellation: CancellationToken,
    limits: WorkLimits,
    remaining: WorkLimits,
    cancellation_fault: Option<CancellationFault>,
    #[cfg(test)]
    observations: InvocationObservations,
}

#[cfg(test)]
#[derive(Debug, Default)]
struct InvocationObservations {
    template_candidates_considered: usize,
    template_candidates_since_charge: usize,
    maximum_template_candidates_between_charges: usize,
    cancel_after_template_candidates: Option<usize>,
    template_candidate_signal_sent: bool,
    template_candidates_after_signal: usize,
    skip_template_candidate_charging: bool,
    document_rooted_match_evaluations: usize,
    global_atomic_frames_cloned: usize,
    global_atomic_entries_cloned: usize,
    skip_document_rooted_match_cache: bool,
    document_rooted_match_cache_builds: usize,
    document_rooted_match_cache_hits: usize,
    document_rooted_match_cache_bytes: usize,
}

#[derive(Debug, Clone, Copy)]
struct CancellationFault {
    domain: WorkDomain,
    accepted_charges_before_signal: usize,
}

impl InvocationControl {
    pub(crate) fn new(cancellation: CancellationToken, limits: WorkLimits) -> Self {
        Self {
            cancellation,
            limits,
            remaining: limits,
            cancellation_fault: None,
            #[cfg(test)]
            observations: InvocationObservations::default(),
        }
    }

    pub(crate) fn unbounded() -> Self {
        Self::new(CancellationToken::new(), WorkLimits::unbounded())
    }

    /// Installs a deterministic test fault at a real charge point.
    ///
    /// This is not a deadline or production cancellation mechanism. It lets the
    /// private experiment prove phase-specific failure behavior after a chosen
    /// number of matching charges have already succeeded.
    #[cfg(test)]
    pub(crate) fn cancelling_on_charge(
        mut self,
        domain: WorkDomain,
        accepted_charges_before_signal: usize,
    ) -> Self {
        self.cancellation_fault = Some(CancellationFault {
            domain,
            accepted_charges_before_signal,
        });
        self
    }

    #[cfg(test)]
    pub(crate) fn consumed(&self, domain: WorkDomain) -> usize {
        self.limits
            .limit(domain)
            .saturating_sub(self.remaining.limit(domain))
    }

    /// Records selection fanout without selecting a budget unit or adding a
    /// cancellation check to the measured path.
    pub(crate) fn observe_template_candidate(&mut self) {
        #[cfg(not(test))]
        let _ = self;
        #[cfg(test)]
        {
            self.observations.template_candidates_considered += 1;
            self.observations.template_candidates_since_charge += 1;
            if self.observations.template_candidate_signal_sent {
                self.observations.template_candidates_after_signal += 1;
            } else if let Some(remaining) = &mut self.observations.cancel_after_template_candidates
            {
                *remaining -= 1;
                if *remaining == 0 {
                    self.cancellation.cancel();
                    self.observations.template_candidate_signal_sent = true;
                    self.observations.cancel_after_template_candidates = None;
                }
            }
        }
    }

    pub(crate) fn charge_template_candidate(&mut self) -> Result<(), ControlFailure> {
        self.observe_template_candidate();
        #[cfg(test)]
        if self.observations.skip_template_candidate_charging {
            return Ok(());
        }
        self.charge(WorkDomain::XsltTemplateCandidate, 1)
    }

    #[cfg(test)]
    pub(crate) fn without_template_candidate_charging(mut self) -> Self {
        self.observations.skip_template_candidate_charging = true;
        self
    }

    #[cfg(test)]
    pub(crate) fn cancelling_after_template_candidates(mut self, candidates: usize) -> Self {
        assert!(
            candidates > 0,
            "candidate cancellation requires a positive offset"
        );
        self.observations.cancel_after_template_candidates = Some(candidates);
        self
    }

    #[cfg(test)]
    pub(crate) fn template_candidate_observation(&self) -> (usize, usize) {
        (
            self.observations.template_candidates_considered,
            self.observations
                .maximum_template_candidates_between_charges
                .max(self.observations.template_candidates_since_charge),
        )
    }

    #[cfg(test)]
    pub(crate) fn template_candidates_after_cancellation_signal(&self) -> usize {
        self.observations.template_candidates_after_signal
    }

    /// Attributes repeated document-rooted match-path evaluation without
    /// selecting or retaining an optimized membership representation.
    pub(crate) fn observe_document_rooted_match_evaluation(&mut self) {
        #[cfg(not(test))]
        let _ = self;
        #[cfg(test)]
        {
            self.observations.document_rooted_match_evaluations += 1;
        }
    }

    #[cfg(test)]
    pub(crate) fn document_rooted_match_evaluations(&self) -> usize {
        self.observations.document_rooted_match_evaluations
    }

    /// Attributes the current reference runtime's complete global-atomic map
    /// clone when it creates a named-template frame.
    pub(crate) fn observe_global_atomic_frame_clone(&mut self, entries: usize) {
        #[cfg(not(test))]
        let _ = (self, entries);
        #[cfg(test)]
        {
            self.observations.global_atomic_frames_cloned += 1;
            self.observations.global_atomic_entries_cloned += entries;
        }
    }

    #[cfg(test)]
    pub(crate) fn global_atomic_frame_clone_observation(&self) -> (usize, usize) {
        (
            self.observations.global_atomic_frames_cloned,
            self.observations.global_atomic_entries_cloned,
        )
    }

    pub(crate) fn document_rooted_match_cache_enabled(&self) -> bool {
        #[cfg(not(test))]
        let _ = self;
        #[cfg(test)]
        if self.observations.skip_document_rooted_match_cache {
            return false;
        }
        true
    }

    pub(crate) fn observe_document_rooted_match_cache_build(&mut self, retained_bytes: usize) {
        #[cfg(not(test))]
        let _ = (self, retained_bytes);
        #[cfg(test)]
        {
            self.observations.document_rooted_match_cache_builds += 1;
            self.observations.document_rooted_match_cache_bytes += retained_bytes;
        }
    }

    pub(crate) fn observe_document_rooted_match_cache_hit(&mut self) {
        #[cfg(not(test))]
        let _ = self;
        #[cfg(test)]
        {
            self.observations.document_rooted_match_cache_hits += 1;
        }
    }

    #[cfg(test)]
    pub(crate) fn without_document_rooted_match_cache(mut self) -> Self {
        self.observations.skip_document_rooted_match_cache = true;
        self
    }

    #[cfg(test)]
    pub(crate) fn document_rooted_match_cache_observation(&self) -> (usize, usize, usize) {
        (
            self.observations.document_rooted_match_cache_builds,
            self.observations.document_rooted_match_cache_hits,
            self.observations.document_rooted_match_cache_bytes,
        )
    }

    pub(crate) fn charge(
        &mut self,
        domain: WorkDomain,
        units: usize,
    ) -> Result<(), ControlFailure> {
        #[cfg(test)]
        {
            self.observations
                .maximum_template_candidates_between_charges = self
                .observations
                .maximum_template_candidates_between_charges
                .max(self.observations.template_candidates_since_charge);
            self.observations.template_candidates_since_charge = 0;
        }
        if let Some(fault) = &mut self.cancellation_fault
            && fault.domain == domain
        {
            if fault.accepted_charges_before_signal == 0 {
                self.cancellation.cancel();
                self.cancellation_fault = None;
            } else {
                fault.accepted_charges_before_signal -= 1;
            }
        }
        if self.cancellation.is_cancelled() {
            return Err(ControlFailure::Cancelled { domain });
        }

        let limit = self.limits.limit(domain);
        let remaining = self.remaining.remaining_mut(domain);
        if units > *remaining {
            return Err(ControlFailure::BudgetExhausted {
                domain,
                limit,
                consumed: limit.saturating_sub(*remaining),
                attempted: units,
            });
        }
        *remaining -= units;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        hint::black_box,
        sync::mpsc,
        time::{Duration, Instant},
    };

    use super::{CancellationToken, ControlFailure, InvocationControl, WorkDomain, WorkLimits};

    #[test]
    fn cancellation_and_budget_exhaustion_remain_distinct() {
        let token = CancellationToken::new();
        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = 2;
        let mut control = InvocationControl::new(token.clone(), limits);

        control
            .charge(WorkDomain::XPathNodeVisit, 2)
            .expect("boundary-sized charge should pass");
        assert_eq!(
            control.charge(WorkDomain::XPathNodeVisit, 1),
            Err(ControlFailure::BudgetExhausted {
                domain: WorkDomain::XPathNodeVisit,
                limit: 2,
                consumed: 2,
                attempted: 1,
            })
        );

        token.cancel();
        assert_eq!(
            control.charge(WorkDomain::XmlEvent, 1),
            Err(ControlFailure::Cancelled {
                domain: WorkDomain::XmlEvent,
            })
        );
    }

    #[test]
    fn first_charge_barrier_observes_then_releases_into_cancellation() {
        let token = CancellationToken::with_first_charge_barrier();
        let worker_token = token.clone();
        let (result, completed) = mpsc::channel();
        std::thread::spawn(move || {
            let mut control = InvocationControl::new(worker_token, WorkLimits::unbounded());
            result
                .send(control.charge(WorkDomain::XsltInstruction, 1))
                .expect("test receiver should remain available");
        });

        while !token.first_charge_observed() {
            std::thread::yield_now();
        }
        assert!(completed.try_recv().is_err());
        token.cancel();
        assert_eq!(
            completed
                .recv_timeout(Duration::from_secs(1))
                .expect("cancelled charge should finish"),
            Err(ControlFailure::Cancelled {
                domain: WorkDomain::XsltInstruction,
            })
        );
    }

    #[test]
    #[ignore = "manual release-mode accounting-cost probe"]
    fn measures_unexhausted_charge_cost() {
        const ITERATIONS: usize = 10_000_000;
        const ITERATIONS_F64: f64 = 10_000_000.0;
        const SAMPLES: usize = 7;
        let mut baseline_ns = Vec::with_capacity(SAMPLES);
        let mut charged_ns = Vec::with_capacity(SAMPLES);

        for _ in 0..SAMPLES {
            let baseline_start = Instant::now();
            for value in 0..ITERATIONS {
                black_box(value);
            }
            baseline_ns
                .push(baseline_start.elapsed().as_secs_f64() * 1_000_000_000.0 / ITERATIONS_F64);

            let mut control = InvocationControl::unbounded();
            let charged_start = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(control.charge(WorkDomain::XPathNodeVisit, 1))
                    .expect("unbounded charge should succeed");
            }
            charged_ns
                .push(charged_start.elapsed().as_secs_f64() * 1_000_000_000.0 / ITERATIONS_F64);
            assert_eq!(
                black_box(control.consumed(WorkDomain::XPathNodeVisit)),
                ITERATIONS
            );
        }

        baseline_ns.sort_by(f64::total_cmp);
        charged_ns.sort_by(f64::total_cmp);
        println!(
            "iterations={ITERATIONS} samples={SAMPLES} baseline_median_ns={:.3} charge_median_ns={:.3}",
            baseline_ns[SAMPLES / 2],
            charged_ns[SAMPLES / 2]
        );
    }
}
