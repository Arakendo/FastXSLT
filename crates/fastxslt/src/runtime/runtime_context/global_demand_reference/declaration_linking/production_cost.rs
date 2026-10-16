//! Same-program ordinary-global control; the reference is not production code.

use super::*;
use crate::runtime::golden_runtime_experiment::{ResultNode, execute_program};
use std::{
    fmt::Write,
    hint::black_box,
    time::{Duration, Instant},
};

struct Fixture {
    program: Arc<StylesheetProgram>,
    source: Arc<Document>,
    expected: Vec<ResultNode>,
}

impl Fixture {
    fn new(count: usize, reads: usize) -> Self {
        let mut xml = String::from(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">"#,
        );
        for index in 0..count {
            write!(xml, r#"<xsl:variable name="v{index}" select="/r/b"/>"#).unwrap();
        }
        xml.push_str(r#"<xsl:template match="/"><xsl:text>start</xsl:text>"#);
        for index in 0..reads {
            write!(xml, r#"<xsl:value-of select="$v{}"/>"#, index % count).unwrap();
        }
        xml.push_str("</xsl:template></xsl:stylesheet>");
        let program =
            Arc::new(compile_stylesheet(&document("urn:ordinary-cost:style", &xml)).unwrap());
        let source = document(
            "urn:ordinary-cost:source",
            "<r><b>payload</b><b>other</b></r>",
        );
        let expected = execute_program(
            &program,
            &source,
            "ordinary-cost",
            &mut InvocationControl::unbounded(),
        )
        .unwrap()
        .children;
        Self {
            program,
            source,
            expected,
        }
    }

    fn invoke(&self, lane: usize) -> usize {
        let mut control = InvocationControl::unbounded();
        let nodes = if lane == 0 {
            execute_program(&self.program, &self.source, "ordinary-cost", &mut control)
                .unwrap()
                .children
        } else {
            let driver = Driver::new(
                Arc::clone(&self.program),
                Arc::clone(&self.source),
                &BTreeMap::new(),
                false,
                &mut control,
            )
            .unwrap();
            if lane == 2 {
                eager(&driver, &mut control).unwrap();
            }
            let nodes = driver
                .execute_nodes_with_frame(
                    &self.program.root_template.as_ref().unwrap().body,
                    &RuntimeVariables::default(),
                    &self.source,
                    &mut control,
                )
                .unwrap();
            assert!(driver.resolver.stack.borrow().is_empty());
            assert!(driver.evaluations.borrow().iter().all(|count| *count <= 1));
            nodes
        };
        assert_eq!(nodes, self.expected);
        black_box(control.consumed(WorkDomain::XPathOperation))
    }
}

#[test]
fn ordinary_global_production_and_demand_cost_fixtures_have_semantic_parity() {
    for (count, reads) in [(0, 0), (1, 1), (1, 64), (16, 16), (16, 1)] {
        let fixture = Fixture::new(count, reads);
        for lane in 0..3 {
            fixture.invoke(lane);
        }
    }
}

#[test]
#[ignore = "manual release-mode AR-0030 ordinary-global production comparison"]
fn ordinary_global_production_cost_probe() {
    for (count, reads) in [(0, 0), (1, 1), (1, 64), (16, 16), (16, 1)] {
        let fixture = Fixture::new(count, reads);
        for lane in 0..3 {
            let start = Instant::now();
            while start.elapsed() < Duration::from_millis(100) {
                fixture.invoke(lane);
            }
        }
        let mut windows = [Vec::new(), Vec::new(), Vec::new()];
        for round in 0..6 {
            for offset in 0..3 {
                let lane = (round + offset) % 3;
                let start = Instant::now();
                let mut iterations = 0_u32;
                while start.elapsed() < Duration::from_millis(100) {
                    fixture.invoke(lane);
                    iterations += 1;
                }
                let ns = start.elapsed().as_secs_f64() * 1e9 / f64::from(iterations);
                windows[lane].push(ns);
                println!(
                    "AR0030_ORDINARY window globals={count} reads={reads} lane={lane} round={round} ns={ns:.1} n={iterations}"
                );
            }
        }
        for (lane, values) in windows.iter_mut().enumerate() {
            values.sort_by(f64::total_cmp);
            println!(
                "AR0030_ORDINARY summary globals={count} reads={reads} lane={lane} min_ns={:.1} median_ns={:.1} max_ns={:.1} xpath_work={}",
                values[0],
                f64::midpoint(values[2], values[3]),
                values[5],
                fixture.invoke(lane)
            );
        }
    }
}
