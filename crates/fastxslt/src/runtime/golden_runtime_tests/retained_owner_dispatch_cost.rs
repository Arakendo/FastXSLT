//! Integrated single-document control; not an interop performance claim.

use super::*;

fn ordinary_fixture(retained: bool) -> (StylesheetProgram, ResourceSnapshot, Document) {
    let (_, _, source) = fixture(false);
    let global = if retained {
        r#"<xsl:variable name="principal" select="/"/>"#
    } else {
        ""
    };
    let select = if retained { "$principal" } else { "/" };
    let style = format!(
        r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/>{global}<xsl:template match="/"><xsl:apply-templates select="{select}" mode="out"/></xsl:template><xsl:template match="/" mode="out"><out><xsl:value-of select="external/value"/></out></xsl:template></xsl:stylesheet>"#
    );
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    builder.admit(STYLE, style.into_bytes()).unwrap();
    let snapshot = builder.seal();
    let program = compile_resource(&snapshot, STYLE).unwrap();
    (program, snapshot, source)
}

#[cfg(feature = "allocation-observation")]
#[test]
fn ordinary_retained_dispatch_allocation_and_release() {
    for retained in [false, true] {
        let fixture = ordinary_fixture(retained);
        let info = allocation_counter::measure(|| {
            assert_eq!(
                run(
                    &fixture,
                    WhitespaceRepresentation::VisibilityView,
                    None,
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                "<out>wrong</out>"
            );
        });
        assert_eq!(info.bytes_current, 0);
        assert_eq!(info.count_current, 0);
        println!(
            "ordinary-retained-dispatch retained={retained} total={} peak={} allocations={}",
            info.bytes_total, info.bytes_max, info.count_total
        );
    }
}

#[test]
#[ignore = "manual rotated release single-document dispatch timings; omit allocation-observation"]
fn ordinary_retained_dispatch_timing() {
    let fixtures = [ordinary_fixture(false), ordinary_fixture(true)];
    for fixture in &fixtures {
        for _ in 0..100 {
            assert_eq!(
                run(
                    fixture,
                    WhitespaceRepresentation::VisibilityView,
                    None,
                    &mut InvocationControl::unbounded()
                )
                .unwrap(),
                "<out>wrong</out>"
            );
        }
    }
    for window in 0..5 {
        for offset in 0..2 {
            let lane = (window + offset) % 2;
            let start = std::time::Instant::now();
            let mut iterations = 0;
            while start.elapsed() < std::time::Duration::from_millis(30) {
                let actual = run(
                    &fixtures[lane],
                    WhitespaceRepresentation::VisibilityView,
                    None,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap();
                std::hint::black_box(actual);
                iterations += 1;
            }
            let nanos = start.elapsed().as_secs_f64() * 1e9 / f64::from(iterations);
            println!(
                "ordinary-retained-dispatch window={window} retained={} iterations={iterations} ns_per_call={nanos:.1}",
                lane == 1
            );
        }
    }
}
