use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkLimits};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
use crate::xpath::path_experiment::parse_xslt10_location_path as parse_location_path;

fn location() -> SourceLocation {
    SourceLocation {
        resource: "memory:predicate.xsl".to_owned(),
        span: 12..90,
    }
}

fn source() -> Document {
    Document::from_parsed(parse_document("memory:source.xml",
        "<r att='not-empty'><left>hé😃-tail</left><left>ignored</left><n>hé😃</n><n>other</n></r>".as_bytes(),
        ParseLimits { max_events: 40, max_depth: 8 }).unwrap()).unwrap()
}

#[test]
fn composed_predicates_preserve_first_node_unicode_and_scalar_comparison() {
    let document = source();
    for (expression, count) in [
        (
            "r/n[string(.) = concat(substring-before(/r/left, '-'), '')]",
            1,
        ),
        (
            "r/n[concat(substring-before(/r/left, '-'), '') != string(.)]",
            1,
        ),
        ("r/n[string-length(.) = ((3 + 6 - 8) * 3)]", 1),
        ("r/@*[string-length(self::*) != ((3 + 6 - 8) * 5)]", 1),
        ("r/n[substring-before('abc', '') = '']", 2),
        ("r/n[substring-before('abc', 'missing') = '']", 2),
        ("r/n[string(/r/missing) = '']", 2),
        ("r/n[concat('a,=', 'b') = 'a,=b']", 2),
        ("r/n[string(.) = 'hé😃' and not(string(.) = 'other')]", 1),
    ] {
        let path = parse_location_path(expression, location()).unwrap();
        let nodes = evaluate_location_path_controlled(
            &document,
            document.document_node(),
            &path,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert_eq!(nodes.len(), count, "{expression}");
        assert!(path.known_owned_capacity_bytes() > 0);
    }
}

#[test]
fn composed_predicate_refuses_adjacent_unsupported_semantics() {
    assert!(
        crate::xpath::path_experiment::parse_location_path(
            "r/n[string(.) = concat('a','b')]",
            location()
        )
        .is_err()
    );
    for expression in [
        "child = concat('a','b')",
        "string(child[1]) = 'a'",
        "string($v) = 'a'",
        "string(p:child) = 'a'",
        "concat('a') = 'a'",
        "substring-before('a','b','c') = ''",
        "string-length(.) = (9007199254740993 - 9007199254740992)",
        "string-length(.) = (1 div 2)",
        "string(document('remote')) = ''",
    ] {
        assert!(parse(expression, &location()).is_none(), "{expression}");
    }
    let deep = format!("{}'x'{} = 'x'", "string(".repeat(17), ")".repeat(17));
    assert!(parse(&deep, &location()).is_none());
    let deep_constant = format!("string-length(.) = {}1+1{}", "(".repeat(70), ")".repeat(70));
    assert!(parse(&deep_constant, &location()).is_none());
    let failure = parse_location_path("r/n[string($v) = 'a']", location()).unwrap_err();
    assert!(
        matches!(failure, super::super::PathFailure::Unsupported { location: actual, .. } if actual == location())
    );
}

#[test]
fn composed_predicate_controls_fail_without_retained_partial_results() {
    let document = source();
    let path = parse_location_path(
        "r/n[string(.) = concat(substring-before(/r/left, '-'), '')]",
        location(),
    )
    .unwrap();
    let evaluate = |control: &mut InvocationControl| {
        evaluate_location_path_controlled(&document, document.document_node(), &path, control)
    };
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(evaluate(&mut baseline).unwrap().len(), 1);
    for domain in [
        WorkDomain::XPathNodeVisit,
        WorkDomain::XdmStringValueNode,
        WorkDomain::XPathOperation,
        WorkDomain::ResultTextByte,
    ] {
        let consumed = baseline.consumed(domain);
        assert!(consumed > 0);
        for limit in [consumed - 1, consumed] {
            let mut limits = WorkLimits::unbounded();
            match domain {
                WorkDomain::XPathNodeVisit => limits.xpath_node_visits = limit,
                WorkDomain::XdmStringValueNode => limits.xdm_string_value_nodes = limit,
                WorkDomain::XPathOperation => limits.xpath_operations = limit,
                WorkDomain::ResultTextByte => limits.result_text_bytes = limit,
                _ => unreachable!(),
            }
            let mut control = InvocationControl::new(CancellationToken::new(), limits);
            assert_eq!(evaluate(&mut control).is_ok(), limit == consumed);
        }
        let mut control = InvocationControl::unbounded().cancelling_on_charge(domain, 2);
        assert!(evaluate(&mut control).is_err());
        assert_eq!(
            evaluate(&mut InvocationControl::unbounded()).unwrap().len(),
            1
        );
    }
    #[cfg(feature = "allocation-observation")]
    for cancel in [false, true] {
        let allocation = allocation_counter::measure(|| {
            let mut control = InvocationControl::unbounded();
            if cancel {
                control = control.cancelling_on_charge(WorkDomain::ResultTextByte, 2);
            }
            let outcome = evaluate(&mut control);
            assert_eq!(outcome.is_ok(), !cancel);
        });
        assert_eq!(allocation.bytes_current, 0);
        assert_eq!(allocation.count_current, 0);
    }
}
