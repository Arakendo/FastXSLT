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
            WorkDomain::ResultNode => &mut self.result_nodes,
            WorkDomain::ResultTextByte => &mut self.result_text_bytes,
            WorkDomain::SerializedByte => &mut self.serialized_bytes,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CancellationToken(Arc<AtomicBool>);

impl CancellationToken {
    pub(crate) fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }

    pub(crate) fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }

    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
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

    pub(crate) fn charge(
        &mut self,
        domain: WorkDomain,
        units: usize,
    ) -> Result<(), ControlFailure> {
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
    use std::{hint::black_box, time::Instant};

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
