//! Private schedule attribution, not a production or host performance benchmark.

use super::*;
use std::{
    fmt::Write,
    hint::black_box,
    time::{Duration, Instant},
};

#[test]
#[ignore = "manual release-mode AR-0030 schedule cost probe"]
fn demand_declaration_schedule_cost_probe() {
    for (label, count, used) in [("one", 1, 1), ("all16", 16, 16), ("one_of16", 16, 1)] {
        let mut body = String::new();
        for index in 0..count {
            write!(
                body,
                r#"<xsl:variable name="v{index}"><box>payload</box></xsl:variable>"#,
            )
            .unwrap();
        }
        body.push_str(r#"<xsl:template match="/">"#);
        for index in 0..used {
            write!(body, r#"<xsl:value-of select="$v{index}"/>"#).unwrap();
        }
        body.push_str("</xsl:template>");
        let compiled = declaration_program(&body).unwrap();
        let program = Arc::new(compiled.program);
        let source = document("urn:schedule-cost:source", "<r/>");
        let expected = "payload".repeat(used);
        let parameters = BTreeMap::new();
        let invocation = |initialize| {
            let mut control = InvocationControl::unbounded();
            let driver = Driver::with_constructor_bodies(
                Arc::clone(&program),
                Arc::clone(&source),
                &parameters,
                false,
                compiled.bodies.clone(),
                &mut control,
            )
            .unwrap();
            if initialize {
                eager(&driver, &mut control).unwrap();
            }
            assert_eq!(driver.run(&mut control).unwrap(), expected);
            assert_eq!(
                driver.evaluations.borrow().iter().sum::<usize>(),
                if initialize { count } else { used }
            );
            assert!(driver.resolver.stack.borrow().is_empty());
            black_box(control.consumed(WorkDomain::XPathOperation))
        };
        // Identical fresh-invocation setup, map cloning, checking and teardown
        // are timed in both lanes. XML preparation and compilation are excluded.
        for initialize in [false, true] {
            let start = Instant::now();
            while start.elapsed() < Duration::from_millis(100) {
                invocation(initialize);
            }
        }
        let mut windows = [Vec::new(), Vec::new()];
        for round in 0..7 {
            for lane in [round % 2, 1 - round % 2] {
                let start = Instant::now();
                let mut iterations = 0_u32;
                while start.elapsed() < Duration::from_millis(100) {
                    invocation(lane == 1);
                    iterations += 1;
                }
                let ns = start.elapsed().as_secs_f64() * 1e9 / f64::from(iterations);
                windows[lane].push(ns);
                println!(
                    "AR0030_COST window {label} lane={lane} round={round} ns={ns:.1} n={iterations}"
                );
            }
        }
        for (lane, values) in windows.iter_mut().enumerate() {
            values.sort_by(f64::total_cmp);
            println!(
                "AR0030_COST summary {label} lane={lane} min_ns={:.1} median_ns={:.1} max_ns={:.1} xpath_work={}",
                values[0],
                values[3],
                values[6],
                invocation(lane == 1)
            );
        }
    }
}
