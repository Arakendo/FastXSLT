//! Opt-in, bounded reference read paths. Unknown string-read locations remain
//! unknown. Each failed cell captures its active path, not a flattened causal
//! chain or a complete cycle trace. No production diagnostic policy is selected.

use super::super::failure_context_reference::{Dependency, FailureContext};
use super::*;
use crate::execution_control_experiment::WorkDomain;
use crate::xdm::owned_tree_experiment::SourceLocation;
use std::cell::{Cell, OnceCell};

#[path = "cycle_provenance.rs"]
mod cycle_provenance;

#[path = "string_provenance.rs"]
mod string_provenance;

#[path = "boolean_provenance.rs"]
mod boolean_provenance;

type CycleObservation = Arc<FailureContext<Failure<ExecutionFailure>>>;

#[derive(Default)]
pub(super) struct ReadProvenance {
    enabled: Cell<bool>,
    path: RefCell<Vec<Dependency>>,
    // One bounded first-cycle observation, not a lookup map, failure attachment
    // or complete invocation error history. Experimental inspection only.
    first_cycle: OnceCell<CycleObservation>,
}

pub(super) struct ActiveRead<'a>(Option<&'a ReadProvenance>);

impl Drop for ActiveRead<'_> {
    fn drop(&mut self) {
        if let Some(owner) = self.0 {
            owner.path.borrow_mut().pop();
        }
    }
}

impl ReadProvenance {
    pub(super) fn observe_cycle(
        &self,
        error: &Arc<Failure<ExecutionFailure>>,
        control: &mut InvocationControl,
    ) {
        if self.enabled.get() && matches!(error.as_ref(), Failure::Circularity(_)) {
            self.first_cycle.get_or_init(|| {
                Arc::new(FailureContext::capture(
                    Arc::clone(error),
                    &self.path.borrow(),
                    32,
                    control,
                ))
            });
        }
    }

    pub(super) fn enter(
        &self,
        binding: usize,
        location: Option<&SourceLocation>,
        program: &StylesheetProgram,
        control: &mut InvocationControl,
    ) -> Result<ActiveRead<'_>, ExecutionFailure> {
        if !self.enabled.get() {
            return Ok(ActiveRead(None));
        }
        if self.path.borrow().len() == 32 {
            return Err(failure(
                "FXAR0030",
                FailureCategory::Limit,
                Some("demand-shared-reference"),
                "reference read-path depth limit",
            ));
        }
        // Provisional entry work, not a byte/allocator estimate. Charge before
        // retaining owned metadata; release the borrow before shared execution.
        control
            .charge(WorkDomain::XdmNode, 1)
            .map_err(|error| control_failure(error, "demand-shared-reference"))?;
        self.path.borrow_mut().push(Dependency {
            binding,
            declaration: Arc::new(
                program.global_bindings[binding]
                    .declaration_reference
                    .clone(),
            ),
            read: location.map(|location| Arc::new(location.clone())),
        });
        Ok(ActiveRead(Some(self)))
    }

    pub(super) fn capture(
        &self,
        resolver: &DemandReference<OwnedValue, ExecutionFailure>,
        binding: usize,
        control: &mut InvocationControl,
    ) {
        if self.enabled.get() {
            resolver.failed_context(binding, &self.path.borrow(), 32, control);
        }
    }
}

#[test]
fn demand_read_sites_follow_recursive_copies_without_relabeling_origin() {
    let mut control = InvocationControl::unbounded();
    let (driver, xml) = super::failure_observations::driver(
        r#"
      <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>
      <xsl:template name="first_constructor"><xsl:copy-of select="$second"/></xsl:template>
      <xsl:template name="second_constructor"><xsl:message terminate="yes">failure</xsl:message></xsl:template>"#,
        &mut control,
    );
    driver.provenance.enabled.set(true);
    let original = driver.run(&mut control).unwrap_err();
    assert_eq!(original.code, "XTMM9000");
    assert!(driver.provenance.path.borrow().is_empty());
    let context = driver
        .resolver
        .failed_context(1, &[], 0, &mut control)
        .unwrap();
    assert_eq!(context.dependencies.len(), 2);
    for (index, edge) in context.dependencies.iter().enumerate() {
        assert_eq!(edge.binding, index);
        assert!(xml[edge.declaration.span.clone()].starts_with("<xsl:variable"));
        assert!(xml[edge.read.as_ref().unwrap().span.clone()].starts_with("<xsl:copy-of"));
    }
    assert_eq!(
        driver.read_string("second", &mut control).unwrap_err(),
        original
    );
    let repeated = driver
        .resolver
        .failed_context(1, &[], 0, &mut control)
        .unwrap();
    assert!(Arc::ptr_eq(&context, &repeated));
    assert_eq!(control.messages(), &["failure"]);
    assert!(driver.provenance.path.borrow().is_empty());
}

#[test]
fn demand_boolean_string_literal_read_retains_condition_location() {
    let mut control = InvocationControl::unbounded();
    let (driver, xml) = super::failure_observations::driver(
        r#"
      <xsl:template match="/"><xsl:if test="$first = 'x'">unused</xsl:if></xsl:template>
      <xsl:template name="first_constructor"><xsl:message terminate="yes">failure</xsl:message></xsl:template>"#,
        &mut control,
    );
    driver.provenance.enabled.set(true);
    assert_eq!(driver.run(&mut control).unwrap_err().code, "XTMM9000");
    let context = driver
        .resolver
        .failed_context(0, &[], 0, &mut control)
        .unwrap();
    assert_eq!(context.dependencies.len(), 1);
    assert!(
        xml[context.dependencies[0].read.as_ref().unwrap().span.clone()].starts_with("<xsl:if")
    );
    assert!(driver.provenance.path.borrow().is_empty());
}

#[test]
fn demand_direct_string_read_retains_its_own_compiled_instruction() {
    for disable in ["yes", "no"] {
        let mut control = InvocationControl::unbounded();
        let (driver, xml) = super::failure_observations::driver(
            &format!(
                r#"
          <xsl:template match="/"><xsl:copy-of select="$first"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:value-of select="$second" disable-output-escaping="{disable}"/></xsl:template>
          <xsl:template name="second_constructor"><xsl:message terminate="yes">failure</xsl:message></xsl:template>"#
            ),
            &mut control,
        );
        driver.provenance.enabled.set(true);
        let original = driver.run(&mut control).unwrap_err();
        assert_eq!(original.code, "XTMM9000");
        let context = driver
            .resolver
            .failed_context(1, &[], 0, &mut control)
            .unwrap();
        assert_eq!(context.dependencies.len(), 2);
        let read = context.dependencies[1].read.as_ref().unwrap();
        assert!(xml[read.span.clone()].starts_with("<xsl:value-of"));
        assert_ne!(Some(read.as_ref()), original.location.as_ref());
        assert_ne!(context.dependencies[0].read, context.dependencies[1].read);
        assert!(driver.provenance.path.borrow().is_empty());
    }
}

#[test]
fn demand_read_path_bounds_and_unwind_release_retained_metadata() {
    use crate::execution_control_experiment::{CancellationToken, WorkLimits};
    let mut control = InvocationControl::unbounded();
    let (driver, _) = super::failure_observations::driver("", &mut control);
    driver.provenance.enabled.set(true);
    let mut limits = WorkLimits::unbounded();
    limits.xdm_nodes = 0;
    let mut denied = InvocationControl::new(CancellationToken::new(), limits);
    assert!(
        driver
            .provenance
            .enter(0, None, &driver.program, &mut denied)
            .is_err()
    );
    assert!(driver.provenance.path.borrow().is_empty());
    let guards = (0..32)
        .map(|_| {
            driver
                .provenance
                .enter(0, None, &driver.program, &mut control)
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert!(
        driver
            .provenance
            .enter(0, None, &driver.program, &mut control)
            .is_err()
    );
    assert_eq!(driver.provenance.path.borrow().len(), 32);
    drop(guards);
    assert!(driver.provenance.path.borrow().is_empty());
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = driver
            .provenance
            .enter(0, None, &driver.program, &mut control)
            .unwrap();
        panic!("reference cleanup probe");
    }));
    assert!(unwind.is_err());
    assert!(driver.provenance.path.borrow().is_empty());
}
