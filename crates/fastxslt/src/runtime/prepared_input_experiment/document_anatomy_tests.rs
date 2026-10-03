//! AR-0027 shape matrix over the unchanged safe prepared-document owner.

use std::fmt::Write as _;

use crate::execution_control_experiment::InvocationControl;
use crate::resources::{ResourceLimits, ResourceSetBuilder, ResourceSnapshot};
use crate::xml::quick_xml_experiment::ParseLimits;

use super::prepare_document;

const IDENTITY: &str = "urn:ar0027:layout-source";
const LIMITS: ParseLimits = ParseLimits {
    max_events: 100_000,
    max_depth: 512,
};

#[path = "capacity_comparison_tests.rs"]
mod capacity_comparison_tests;

#[path = "capacity_lifecycle_tests.rs"]
mod capacity_lifecycle_tests;

#[path = "capacity_cancellation_gap_tests.rs"]
mod capacity_cancellation_gap_tests;

#[path = "capacity_checkpoint_tests.rs"]
mod capacity_checkpoint_tests;

struct Fixture {
    name: &'static str,
    xml: String,
    nodes: usize,
    text_bytes: usize,
}

fn fixtures() -> Vec<Fixture> {
    let wide = format!(
        "<r>{}</r>",
        "<item code='same'>shared text</item>".repeat(1000)
    );
    let deep = format!("{}leaf{}", "<n>".repeat(256), "</n>".repeat(256));
    let mut attributes = String::new();
    for index in 0..16 {
        write!(attributes, " a{index}='same'").unwrap();
    }
    let attribute_heavy = format!("<r>{}</r>", format!("<item{attributes}/>").repeat(64));
    let text_heavy = format!(
        "<r>{}</r>",
        format!("<item>{}</item>", "x".repeat(4096)).repeat(64)
    );
    let mut bindings = String::new();
    for index in 0..8 {
        write!(bindings, " xmlns:p{index}='urn:ns:{index}'").unwrap();
    }
    let namespace_heavy = format!("<r>{}</r>", format!("<p0:item{bindings}/>").repeat(128));
    let mut low_repetition = String::from("<r>");
    for index in 0..512 {
        write!(
            low_repetition,
            "<item{index} code{index}='v{index:04}'>t{index:04}</item{index}>"
        )
        .unwrap();
    }
    low_repetition.push_str("</r>");
    vec![
        Fixture {
            name: "wide",
            xml: wide,
            nodes: 3002,
            text_bytes: 11_000,
        },
        Fixture {
            name: "deep",
            xml: deep,
            nodes: 258,
            text_bytes: 4,
        },
        Fixture {
            name: "attribute-heavy",
            xml: attribute_heavy,
            nodes: 1090,
            text_bytes: 0,
        },
        Fixture {
            name: "text-heavy",
            xml: text_heavy,
            nodes: 130,
            text_bytes: 262_144,
        },
        Fixture {
            name: "namespace-heavy",
            xml: namespace_heavy,
            nodes: 130,
            text_bytes: 0,
        },
        Fixture {
            name: "low-repetition",
            xml: low_repetition,
            nodes: 1538,
            text_bytes: 2560,
        },
    ]
}

fn snapshot(fixture: &Fixture) -> ResourceSnapshot {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(1, 1_048_576, 1_048_576));
    builder
        .admit(IDENTITY, fixture.xml.as_bytes().to_vec())
        .unwrap();
    builder.seal()
}

#[test]
fn shape_matrix_conserves_capacity_totals_and_relationship_lengths() {
    for fixture in fixtures() {
        let snapshot = snapshot(&fixture);
        let (document, _) = prepare_document(
            &snapshot,
            LIMITS,
            IDENTITY,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let anatomy = document.capacity_anatomy();
        assert_eq!(
            anatomy.total_capacity_bytes(),
            document.owned_capacity_bytes()
        );
        assert_eq!(document.node_count(), fixture.nodes);
        assert_eq!(anatomy.node_count, fixture.nodes);
        assert_eq!(
            anatomy.live_node_records,
            anatomy.node_count * anatomy.node_record_size
        );
        assert_eq!(
            anatomy.node_records,
            anatomy.node_capacity * anatomy.node_record_size
        );
        assert_eq!(
            anatomy.live_node_records + anatomy.unused_node_records,
            anatomy.node_records
        );
        assert!(anatomy.live_child_ids <= anatomy.child_ids);
        assert!(anatomy.live_attribute_ids <= anatomy.attribute_ids);
        assert!(anatomy.live_namespace_records <= anatomy.namespace_records);
        let mut children = 0;
        let mut attributes = 0;
        let mut namespaces = 0;
        let mut pending = vec![document.document_node()];
        while let Some(node) = pending.pop() {
            children += document.children(node).len();
            attributes += document.attributes(node).len();
            namespaces += document.namespace_declarations(node).len();
            assert_eq!(document.location(node).resource, IDENTITY);
            pending.extend(document.children(node));
            pending.extend(document.attributes(node));
        }
        assert_eq!(
            anatomy.live_child_ids,
            children * std::mem::size_of::<crate::xdm::owned_tree_experiment::NodeId>()
        );
        assert_eq!(
            anatomy.live_attribute_ids,
            attributes * std::mem::size_of::<crate::xdm::owned_tree_experiment::NodeId>()
        );
        assert_eq!(
            anatomy.live_namespace_records,
            namespaces * std::mem::size_of::<crate::xml::quick_xml_experiment::NamespaceBinding>()
        );
        assert_eq!(
            document.string_value(document.document_node()).len(),
            fixture.text_bytes
        );
        if fixture.name == "namespace-heavy" {
            assert_eq!(namespaces, 1024);
            // Bindings are declarations, not eagerly materialized namespace nodes.
            assert_eq!(document.node_count(), 130);
        }
    }
}

#[test]
#[ignore = "manual release AR-0027 shape anatomy; no layout optimization"]
fn measure_document_shape_anatomy() {
    for fixture in fixtures() {
        let snapshot = snapshot(&fixture);
        let (document, parsed_capacity) = prepare_document(
            &snapshot,
            LIMITS,
            IDENTITY,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let anatomy = document.capacity_anatomy();
        println!(
            "ar0027 shape={} source_bytes={} parsed_capacity={} total_capacity={} anatomy={anatomy:?}",
            fixture.name,
            fixture.xml.len(),
            parsed_capacity,
            anatomy.total_capacity_bytes()
        );
    }
}

#[cfg(feature = "allocation-observation")]
#[test]
#[ignore = "manual release complete XML/XDM allocation scope; separate from timing"]
fn measure_document_shape_allocations() {
    for fixture in fixtures() {
        let snapshot = snapshot(&fixture);
        let mut prepared = None;
        // Both XML parsing and XDM construction occur inside the scope. No
        // prebuilt parser allocations are moved in from outside the counter.
        let allocations = allocation_counter::measure(|| {
            prepared = Some(
                prepare_document(
                    &snapshot,
                    LIMITS,
                    IDENTITY,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap(),
            );
        });
        let (document, _) = prepared.unwrap();
        assert_eq!(document.node_count(), fixture.nodes);
        assert!(allocations.bytes_current > 0);
        assert!(allocations.bytes_max >= u64::try_from(allocations.bytes_current).unwrap());
        println!(
            "ar0027 shape={} source_bytes={} xdm_capacity={} allocations={allocations:?}",
            fixture.name,
            fixture.xml.len(),
            document.owned_capacity_bytes()
        );
    }
}
