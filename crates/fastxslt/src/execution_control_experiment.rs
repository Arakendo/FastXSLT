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
    XsltInstruction,
    SerializedByte,
}

impl WorkDomain {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::XmlEvent => "xml-event",
            Self::XdmNode => "xdm-node",
            Self::XdmStringValueNode => "xdm-string-value-node",
            Self::XPathNodeVisit => "xpath-node-visit",
            Self::XsltInstruction => "xslt-instruction",
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
    pub(crate) xslt_instructions: usize,
    pub(crate) serialized_bytes: usize,
}

impl WorkLimits {
    pub(crate) const fn unbounded() -> Self {
        Self {
            xml_events: usize::MAX,
            xdm_nodes: usize::MAX,
            xdm_string_value_nodes: usize::MAX,
            xpath_node_visits: usize::MAX,
            xslt_instructions: usize::MAX,
            serialized_bytes: usize::MAX,
        }
    }

    const fn limit(self, domain: WorkDomain) -> usize {
        match domain {
            WorkDomain::XmlEvent => self.xml_events,
            WorkDomain::XdmNode => self.xdm_nodes,
            WorkDomain::XdmStringValueNode => self.xdm_string_value_nodes,
            WorkDomain::XPathNodeVisit => self.xpath_node_visits,
            WorkDomain::XsltInstruction => self.xslt_instructions,
            WorkDomain::SerializedByte => self.serialized_bytes,
        }
    }

    fn remaining_mut(&mut self, domain: WorkDomain) -> &mut usize {
        match domain {
            WorkDomain::XmlEvent => &mut self.xml_events,
            WorkDomain::XdmNode => &mut self.xdm_nodes,
            WorkDomain::XdmStringValueNode => &mut self.xdm_string_value_nodes,
            WorkDomain::XPathNodeVisit => &mut self.xpath_node_visits,
            WorkDomain::XsltInstruction => &mut self.xslt_instructions,
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
}

impl InvocationControl {
    pub(crate) fn new(cancellation: CancellationToken, limits: WorkLimits) -> Self {
        Self {
            cancellation,
            limits,
            remaining: limits,
        }
    }

    pub(crate) fn unbounded() -> Self {
        Self::new(CancellationToken::new(), WorkLimits::unbounded())
    }

    pub(crate) fn charge(
        &mut self,
        domain: WorkDomain,
        units: usize,
    ) -> Result<(), ControlFailure> {
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
}
