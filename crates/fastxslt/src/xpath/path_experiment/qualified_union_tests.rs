//! Same-document mixed normalization; no cross-document ranking is selected.

use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

fn source() -> Document {
    Document::from_parsed(
        parse_document(
            "urn:mixed",
            br#"<r xmlns="urn:default" xmlns:p="urn:p" a="attr"> <child/> </r>"#,
            ParseLimits {
                max_events: 32,
                max_depth: 8,
            },
        )
        .unwrap(),
    )
    .unwrap()
}

const SELECT: &str = "/*/@* | /*/* | /*/namespace::* | /* | /*/namespace::p";

#[test]
fn qualified_union_order_and_identity_survive_views_and_do_not_rank_other_origins() {
    let source = source();
    let view = source
        .view_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
        .unwrap();
    let complete = source
        .derive_stripping_all_element_whitespace(&mut InvocationControl::unbounded())
        .unwrap();
    let path = parse(SELECT, source.location(source.document_node()).clone()).unwrap();
    let select = |document| {
        evaluate(
            document,
            source.document_node(),
            &path,
            8,
            16,
            &mut InvocationControl::unbounded(),
        )
        .unwrap()
    };
    let original = select(&source);
    assert_eq!(original.len(), 6);
    assert!(matches!(original[0], QualifiedSourceNode::Tree { .. }));
    assert_eq!(
        original[1..4]
            .iter()
            .map(|node| node.namespace().unwrap().prefix())
            .collect::<Vec<_>>(),
        ["", "p", "xml"]
    );
    let QualifiedSourceNode::Tree { document, id } = original[4] else {
        panic!("attribute")
    };
    assert_eq!(
        document.kind(id),
        crate::xdm::owned_tree_experiment::NodeKind::Attribute
    );
    for effective in [&view, &complete] {
        let nodes = select(effective);
        for (left, right) in original.iter().zip(nodes) {
            assert!(left.same_node(right));
            assert_eq!(
                left.compare_same_origin(right),
                Some(std::cmp::Ordering::Equal)
            );
        }
    }
    for pair in original.windows(2) {
        assert_eq!(
            pair[0].compare_same_origin(pair[1]),
            Some(std::cmp::Ordering::Less)
        );
    }
    let unrelated = self::source();
    let unrelated_nodes = select(&unrelated);
    assert_eq!(original[1].compare_same_origin(unrelated_nodes[1]), None);
    assert!(!original[1].same_node(unrelated_nodes[1]));
}

#[test]
fn qualified_union_bounds_charge_normalization_and_allow_clean_reuse() {
    let source = source();
    let path = parse(SELECT, source.location(source.document_node()).clone()).unwrap();
    let mut measured = InvocationControl::unbounded();
    assert_eq!(
        evaluate(&source, source.document_node(), &path, 8, 16, &mut measured)
            .unwrap()
            .len(),
        6
    );
    let charge = measured.consumed(WorkDomain::XPathNodeVisit);
    for budget in [charge - 1, charge] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = budget;
        let result = evaluate(
            &source,
            source.document_node(),
            &path,
            8,
            16,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        );
        assert_eq!(result.is_ok(), budget == charge);
    }
    // Capacity is enforced before normalization, including duplicate products.
    assert_eq!(
        evaluate(
            &source,
            source.document_node(),
            &path,
            8,
            6,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err(),
        QualifiedPathFailure::SequenceCapacity
    );
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 5);
    assert!(matches!(
        evaluate(
            &source,
            source.document_node(),
            &path,
            8,
            16,
            &mut cancelled
        ),
        Err(QualifiedPathFailure::Control(
            ControlFailure::Cancelled { .. }
        ))
    ));
    assert_eq!(
        evaluate(
            &source,
            source.document_node(),
            &path,
            8,
            16,
            &mut InvocationControl::unbounded()
        )
        .unwrap()
        .len(),
        6
    );
}

#[test]
fn qualified_union_split_respects_quoted_pipes_and_rejects_empty_arms() {
    let source = source();
    let location = source.location(source.document_node()).clone();
    assert_eq!(
        union_alternatives("namespace::p[.='a|b'] | @*"),
        Some(vec!["namespace::p[.='a|b'] ", " @*"])
    );
    for expression in ["namespace::* |", "| namespace::*", "namespace::* || @*"] {
        assert!(matches!(
            parse(expression, location.clone()),
            Err(PathFailure::Invalid {
                standard_code: "XPST0003",
                ..
            })
        ));
    }
}
