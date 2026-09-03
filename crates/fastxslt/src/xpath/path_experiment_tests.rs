use std::ops::Range;

use super::{
    ExistencePredicate, FinalContextPredicate, PathFailure, PathOrigin, PositionPredicate,
    PredicateAxis, evaluate_location_path, evaluate_location_path_controlled, parse_location_path,
    parse_qualified_child_path,
};
use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeKind, SourceLocation};
use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

fn location() -> SourceLocation {
    SourceLocation {
        resource: "memory:stylesheet.xsl".to_owned(),
        span: Range { start: 12, end: 25 },
    }
}

#[test]
fn qualified_child_steps_match_expanded_names() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<doc xmlns:xs="http://www.w3.org/2001/XMLSchema"><string1><xs:a>selected</xs:a><a>other</a></string1></doc>"#,
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let path = parse_qualified_child_path("doc/string1/xs:a", location(), |prefix| {
        (prefix == "xs").then(|| "http://www.w3.org/2001/XMLSchema".to_owned())
    })
    .expect("qualified child path should parse");

    let selected = evaluate_location_path(&document, document.document_node(), &path);

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "selected");
}

#[test]
fn qualified_child_steps_reject_unbound_prefixes() {
    let failure = parse_qualified_child_path("doc/missing:a", location(), |_| None)
        .expect_err("unbound prefix must fail statically");

    assert!(matches!(
        failure,
        PathFailure::Invalid {
            standard_code: "XPST0081",
            ..
        }
    ));
}

#[test]
fn selects_the_context_item_without_navigation() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<item>value</item>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let item = document.children(document.document_node())[0];
    let path = parse_location_path(".", location()).expect("context item should parse");

    let selected = evaluate_location_path(&document, item, &path);

    assert_eq!(selected, [item]);
    assert_eq!(path.origin, PathOrigin::ContextItem);
}

#[test]
fn explicit_context_descendant_path_stays_inside_the_context_subtree() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><outside/><scope><inside><leaf/></inside></scope></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 5,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let scope = document.children(root)[1];
    let inside = document.children(scope)[0];
    let leaf = document.children(inside)[0];
    let path = parse_location_path(".//*", location()).expect("context descendant path");

    let selected = evaluate_location_path(&document, scope, &path);

    assert_eq!(path.origin, PathOrigin::ContextDescendant);
    assert_eq!(selected, [inside, leaf]);
}

#[test]
fn root_path_selects_the_document_node_from_an_element_context() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><item/></root>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let path = parse_location_path("/", location()).expect("root path should parse");
    let mut control = InvocationControl::unbounded();

    let selected = evaluate_location_path_controlled(&document, root, &path, &mut control)
        .expect("root path should execute");

    assert_eq!(path.origin, PathOrigin::DocumentNode);
    assert_eq!(selected, [document.document_node()]);
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 1);
}

#[test]
fn self_steps_preserve_typed_element_attribute_and_text_contexts() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root a=\"1\">text<child/></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let attribute_path =
        parse_location_path("attribute::a", location()).expect("named attribute step should parse");
    let text_path = parse_location_path("text()", location()).expect("text kind test should parse");
    let attribute = evaluate_location_path(&document, root, &attribute_path)[0];
    let text = evaluate_location_path(&document, root, &text_path)[0];
    let self_element =
        parse_location_path("self::*", location()).expect("self element wildcard should parse");
    let self_named =
        parse_location_path("self::root", location()).expect("named self step should parse");
    let self_node =
        parse_location_path("self::node()", location()).expect("self node test should parse");
    let descendant_or_self_node = parse_location_path("descendant-or-self::node()", location())
        .expect("descendant-or-self node test should parse");
    let mut control = InvocationControl::unbounded();

    assert_eq!(
        evaluate_location_path(&document, root, &self_element),
        [root]
    );
    assert_eq!(evaluate_location_path(&document, root, &self_named), [root]);
    assert!(evaluate_location_path(&document, attribute, &self_element).is_empty());
    assert_eq!(
        evaluate_location_path(&document, attribute, &self_node),
        [attribute]
    );
    assert_eq!(evaluate_location_path(&document, text, &self_node), [text]);
    assert_eq!(
        evaluate_location_path(&document, attribute, &descendant_or_self_node),
        [attribute]
    );
    assert_eq!(
        evaluate_location_path(&document, text, &descendant_or_self_node),
        [text]
    );
    assert_eq!(document.kind(text), NodeKind::Text);
    assert_eq!(document.string_value(text), "text");
    assert_eq!(
        evaluate_location_path_controlled(&document, text, &self_node, &mut control)
            .expect("controlled self selection should succeed"),
        [text]
    );
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 1);

    for expression in ["attribute::text()", "parent::text()", "self::text()"] {
        assert!(matches!(
            parse_location_path(expression, location()),
            Err(PathFailure::Unsupported { .. })
        ));
    }
}

#[test]
fn descendant_steps_filter_one_charged_document_order_traversal() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root>lead<a><b/>tail</a><!--c--><?p x?></root>",
        ParseLimits {
            max_events: 20,
            max_depth: 5,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let elements = parse_location_path("descendant::*", location())
        .expect("descendant element wildcard should parse");
    let named = parse_location_path("descendant::b", location())
        .expect("named descendant step should parse");
    let nodes = parse_location_path("descendant::node()", location())
        .expect("descendant node test should parse");
    let mut control = InvocationControl::unbounded();

    let selected_elements = evaluate_location_path(&document, root, &elements);
    let selected_nodes = evaluate_location_path_controlled(&document, root, &nodes, &mut control)
        .expect("controlled descendant traversal should succeed");

    assert_eq!(selected_elements.len(), 2);
    assert_eq!(document.name(selected_elements[0]).unwrap().local, "a");
    assert_eq!(document.name(selected_elements[1]).unwrap().local, "b");
    assert_eq!(
        evaluate_location_path(&document, root, &named),
        [selected_elements[1]]
    );
    assert_eq!(selected_nodes.len(), 6);
    assert_eq!(document.kind(selected_nodes[0]), NodeKind::Text);
    assert_eq!(document.kind(selected_nodes[1]), NodeKind::Element);
    assert_eq!(document.kind(selected_nodes[2]), NodeKind::Element);
    assert_eq!(document.kind(selected_nodes[4]), NodeKind::Comment);
    assert_eq!(
        document.kind(selected_nodes[5]),
        NodeKind::ProcessingInstruction
    );
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 6);
    assert!(matches!(
        parse_location_path("descendant::text()", location()),
        Err(PathFailure::Unsupported { .. })
    ));
}

#[test]
fn leading_descendant_origin_unifies_explicit_and_abbreviated_child_steps() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><!--c--><a>text<b/></a></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 5,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let explicit_elements = parse_location_path("//child::*", location())
        .expect("explicit leading descendant child wildcard should parse");
    let abbreviated_elements =
        parse_location_path("//*", location()).expect("abbreviated wildcard should parse");
    let explicit_nodes = parse_location_path("//child::node()", location())
        .expect("explicit leading descendant child node test should parse");
    let abbreviated_nodes =
        parse_location_path("//node()", location()).expect("abbreviated node test should parse");
    let named =
        parse_location_path("//b", location()).expect("abbreviated named child should parse");
    let mut control = InvocationControl::unbounded();

    assert_eq!(explicit_elements.steps, abbreviated_elements.steps);
    assert_eq!(explicit_nodes.steps, abbreviated_nodes.steps);
    assert_eq!(explicit_elements.origin, PathOrigin::Descendant);
    assert_eq!(
        evaluate_location_path(&document, document.document_node(), &explicit_elements).len(),
        3
    );
    assert_eq!(
        evaluate_location_path(&document, document.document_node(), &named).len(),
        1
    );
    assert_eq!(
        evaluate_location_path_controlled(
            &document,
            document.children(document.document_node())[0],
            &abbreviated_nodes,
            &mut control,
        )
        .expect("controlled leading descendant traversal should execute")
        .len(),
        5
    );
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 5);

    let self_nodes =
        parse_location_path("//self::node()", location()).expect("self node test should parse");
    let self_elements =
        parse_location_path("//self::*", location()).expect("self wildcard should parse");
    let mut self_control = InvocationControl::unbounded();
    let selected = evaluate_location_path_controlled(
        &document,
        document.children(document.document_node())[0],
        &self_nodes,
        &mut self_control,
    )
    .expect("leading descendant self expansion should execute");

    assert_eq!(selected.len(), 6);
    assert_eq!(selected[0], document.document_node());
    assert_eq!(
        evaluate_location_path(&document, selected[1], &self_elements).len(),
        3
    );
    assert_eq!(self_control.consumed(WorkDomain::XPathNodeVisit), 6);
}

#[test]
fn internal_descendant_abbreviation_lowers_to_a_typed_step_and_deduplicates() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><center><a/><center><b/></center></center></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 6,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let explicit = parse_location_path("//center//child::*", location())
        .expect("explicit child step after internal descendant separator should parse");
    let abbreviated = parse_location_path("//center//*", location())
        .expect("abbreviated child step after internal descendant separator should parse");
    let mut control = InvocationControl::unbounded();

    assert_eq!(explicit.steps, abbreviated.steps);
    assert_eq!(explicit.steps[1], "node()");
    let selected = evaluate_location_path_controlled(
        &document,
        document.children(document.document_node())[0],
        &abbreviated,
        &mut control,
    )
    .expect("internal descendant abbreviation should execute");

    assert_eq!(selected.len(), 3);
    assert_eq!(document.name(selected[0]).expect("a name").local, "a");
    assert_eq!(
        document.name(selected[1]).expect("center name").local,
        "center"
    );
    assert_eq!(document.name(selected[2]).expect("b name").local, "b");
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 14);
}

#[test]
fn internal_descendant_abbreviation_composes_with_attribute_steps() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<root xmlns:n="urn:test"><center p="1"><a q="2"/><center p="3"><b q="4"/></center></center></root>"#,
        ParseLimits {
            max_events: 20,
            max_depth: 6,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let explicit = parse_location_path("//center//attribute::*", location())
        .expect("explicit attributes after internal descendant separator should parse");
    let abbreviated = parse_location_path("//center//@*", location())
        .expect("abbreviated attributes after internal descendant separator should parse");
    let mut control = InvocationControl::unbounded();

    assert_eq!(explicit.steps, abbreviated.steps);
    let selected = evaluate_location_path_controlled(
        &document,
        document.children(document.document_node())[0],
        &abbreviated,
        &mut control,
    )
    .expect("internal descendant attribute composition should execute");

    assert_eq!(selected.len(), 4);
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 15);
    assert!(
        selected
            .iter()
            .all(|node| document.kind(*node) == NodeKind::Attribute)
    );
}

#[test]
fn normalize_space_text_predicate_uses_xml_whitespace_effective_boolean_value() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root>  <a> value </a><b>\n\t</b>tail</root>",
        ParseLimits {
            max_events: 20,
            max_depth: 5,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let path = parse_location_path("//text()[normalize-space()]", location())
        .expect("bounded text predicate should parse");
    let mut control = InvocationControl::unbounded();

    assert_eq!(
        path.final_context_predicate,
        Some(FinalContextPredicate::TextHasNonWhitespace)
    );

    let selected =
        evaluate_location_path_controlled(&document, document.document_node(), &path, &mut control)
            .expect("bounded text predicate should execute");

    assert_eq!(selected.len(), 2);
    assert_eq!(document.value(selected[0]), Some(" value "));
    assert_eq!(document.value(selected[1]), Some("tail"));
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 7);
}

#[test]
fn descendant_or_self_steps_include_self_and_deduplicate_overlapping_contexts() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><center><center><center/></center></center></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 6,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let outer = document.children(root)[0];
    let descendants =
        parse_location_path("descendant::*", location()).expect("descendant wildcard should parse");
    let descendant_or_self = parse_location_path("descendant-or-self::*", location())
        .expect("descendant-or-self wildcard should parse");
    let overlapping = parse_location_path("//center/descendant-or-self::center", location())
        .expect("overlapping named descendant-or-self path should parse");
    let mut control = InvocationControl::unbounded();

    assert_eq!(
        evaluate_location_path(&document, outer, &descendants).len(),
        2
    );
    assert_eq!(
        evaluate_location_path(&document, outer, &descendant_or_self).len(),
        3
    );
    let selected = evaluate_location_path_controlled(
        &document,
        document.document_node(),
        &overlapping,
        &mut control,
    )
    .expect("overlapping descendant-or-self path should execute");
    assert_eq!(selected.len(), 3);
    assert_eq!(selected[0], outer);
    assert_eq!(selected[1], document.children(outer)[0]);
    assert_eq!(selected[2], document.children(selected[1])[0]);
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 10);
    assert!(matches!(
        parse_location_path("descendant-or-self::text()", location()),
        Err(PathFailure::Unsupported { .. })
    ));
}

#[test]
fn absolute_and_parent_steps_preserve_document_node_distinctions() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><child/></root>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let document_node = document.document_node();
    let root = document.children(document_node)[0];
    let child = document.children(root)[0];

    let absolute = parse_location_path("/root", location()).expect("absolute path should parse");
    assert_eq!(absolute.origin, PathOrigin::DocumentNode);
    assert_eq!(evaluate_location_path(&document, child, &absolute), [root]);
    let explicit_any_element =
        parse_location_path("/child::*", location()).expect("explicit absolute child parses");
    let abbreviated_any_element =
        parse_location_path("/*", location()).expect("abbreviated absolute child parses");
    let explicit_any_node = parse_location_path("/child::node()", location())
        .expect("explicit absolute child node test parses");
    let abbreviated_any_node =
        parse_location_path("/node()", location()).expect("abbreviated child node test parses");
    assert_eq!(explicit_any_element.steps, abbreviated_any_element.steps);
    assert_eq!(explicit_any_node.steps, abbreviated_any_node.steps);
    assert_eq!(
        evaluate_location_path(&document, child, &explicit_any_element),
        [root]
    );
    assert_eq!(
        evaluate_location_path(&document, child, &abbreviated_any_node),
        [root]
    );
    let document_self = parse_location_path("/self::node()", location())
        .expect("absolute document self test parses");
    let all_elements = parse_location_path("/descendant::*", location())
        .expect("absolute descendant wildcard parses");
    let all_elements_with_self = parse_location_path("/descendant-or-self::*", location())
        .expect("absolute descendant-or-self wildcard parses");
    let all_nodes_with_self = parse_location_path("/descendant-or-self::node()", location())
        .expect("absolute descendant-or-self node test parses");
    assert_eq!(
        evaluate_location_path(&document, child, &document_self),
        [document_node]
    );
    assert_eq!(
        evaluate_location_path(&document, child, &all_elements),
        [root, child]
    );
    assert_eq!(
        evaluate_location_path(&document, child, &all_elements_with_self),
        [root, child]
    );
    assert_eq!(
        evaluate_location_path(&document, child, &all_nodes_with_self),
        [document_node, root, child]
    );

    let explicit = parse_location_path("parent::node()", location())
        .expect("explicit parent node test should parse");
    let abbreviated = parse_location_path("..", location()).expect("parent abbreviation parses");
    assert_eq!(explicit.steps, abbreviated.steps);
    assert_eq!(
        evaluate_location_path(&document, root, &explicit),
        [document_node]
    );
    assert!(
        evaluate_location_path(
            &document,
            root,
            &parse_location_path("parent::*", location()).expect("wildcard should parse"),
        )
        .is_empty()
    );

    let mut control = InvocationControl::unbounded();
    let named = evaluate_location_path_controlled(
        &document,
        child,
        &parse_location_path("parent::root", location()).expect("named parent should parse"),
        &mut control,
    )
    .expect("named parent should execute");
    assert_eq!(named, [root]);
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 1);
}

#[test]
fn attribute_axis_selects_attributes_but_not_namespace_nodes() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<root plain="1" n:other="2" xmlns:n="urn:test"><child plain="3"/></root>"#,
        ParseLimits {
            max_events: 12,
            max_depth: 3,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let mut control = InvocationControl::unbounded();

    let wildcard = evaluate_location_path_controlled(
        &document,
        root,
        &parse_location_path("attribute::*", location()).expect("wildcard should parse"),
        &mut control,
    )
    .expect("attribute wildcard should execute");

    assert_eq!(wildcard.len(), 2);
    assert!(
        wildcard
            .iter()
            .all(|node| document.kind(*node) == NodeKind::Attribute)
    );
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 2);
    assert_eq!(
        evaluate_location_path(
            &document,
            root,
            &parse_location_path("attribute::plain", location()).expect("name should parse"),
        ),
        [wildcard[0]]
    );
    assert_eq!(
        evaluate_location_path(
            &document,
            root,
            &parse_location_path("attribute::node()", location()).expect("node test should parse"),
        ),
        wildcard
    );
    let explicit = parse_location_path("//attribute::*", location())
        .expect("leading explicit attribute expansion should parse");
    let abbreviated =
        parse_location_path("//@*", location()).expect("leading abbreviated attributes parse");
    let named =
        parse_location_path("//@plain", location()).expect("leading named attribute parses");
    assert_eq!(explicit.steps, abbreviated.steps);
    let mut descendant_control = InvocationControl::unbounded();
    let all_attributes =
        evaluate_location_path_controlled(&document, root, &abbreviated, &mut descendant_control)
            .expect("leading attribute expansion should execute");
    assert_eq!(all_attributes.len(), 3);
    assert_eq!(evaluate_location_path(&document, root, &named).len(), 2);
    assert_eq!(descendant_control.consumed(WorkDomain::XPathNodeVisit), 5);
}

#[test]
fn explicit_child_wildcard_selects_elements_across_namespaces() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<root>text<a/><n:b xmlns:n="urn:test"/><!-- comment --></root>"#,
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let path = parse_location_path("child::*", location()).expect("child wildcard should parse");
    let mut control = InvocationControl::unbounded();

    let selected = evaluate_location_path_controlled(&document, root, &path, &mut control)
        .expect("unbounded evaluation should succeed");

    assert_eq!(selected.len(), 2);
    assert_eq!(document.name(selected[0]).expect("a name").local, "a");
    assert_eq!(document.name(selected[1]).expect("b name").local, "b");
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 4);
}

#[test]
fn explicit_child_node_test_selects_every_child_node_kind() {
    let parsed = parse_document(
        "memory:source.xml",
        br"<root>text<a/><?work item?><!-- comment --></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let path =
        parse_location_path("child::node()", location()).expect("child node test should parse");
    let mut control = InvocationControl::unbounded();

    let selected = evaluate_location_path_controlled(&document, root, &path, &mut control)
        .expect("unbounded evaluation should succeed");

    assert_eq!(
        selected
            .iter()
            .map(|node| document.kind(*node))
            .collect::<Vec<_>>(),
        [
            NodeKind::Text,
            NodeKind::Element,
            NodeKind::ProcessingInstruction,
            NodeKind::Comment,
        ]
    );
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 4);
}

#[test]
fn child_kind_tests_select_only_their_declared_node_kinds() {
    let parsed = parse_document(
        "memory:source.xml",
        br"<root>text<a/><?work item?><!-- comment --></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];

    for (expression, kind) in [
        ("element()", NodeKind::Element),
        ("text()", NodeKind::Text),
        ("comment()", NodeKind::Comment),
        ("processing-instruction()", NodeKind::ProcessingInstruction),
    ] {
        let path = parse_location_path(expression, location()).expect("kind test should parse");
        let selected = evaluate_location_path(&document, root, &path);
        assert_eq!(selected.len(), 1, "{expression}");
        assert_eq!(document.kind(selected[0]), kind, "{expression}");
    }
}

#[test]
fn evaluates_the_golden_path_from_the_document_node() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<greeting><name>FastXSLT</name></greeting>",
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let path = parse_location_path("greeting/name", location()).expect("path should parse");

    let selected = evaluate_location_path(&document, document.document_node(), &path);

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "FastXSLT");
}

#[test]
fn filters_the_final_child_step_by_an_explicit_named_child_axis() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><child1/><child1><child2/></child1><child1><other/></child1></doc>",
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let path = parse_location_path("child1[child::child2]", location())
        .expect("named child-axis predicate should parse");

    let mut control = InvocationControl::unbounded();
    let selected = evaluate_location_path_controlled(&document, doc, &path, &mut control)
        .expect("unbounded evaluation should succeed");

    assert_eq!(selected.len(), 1);
    assert_eq!(
        path.final_predicate,
        Some(ExistencePredicate {
            axis: PredicateAxis::Child,
            name: "child2".to_owned(),
        })
    );
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 5);
}

#[test]
fn searches_descendants_and_filters_by_a_named_ancestor() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><element1><child2>wrong</child2></element1><element2><child2>right</child2></element2></doc>",
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let path = parse_location_path("//child2[ancestor::element2]", location())
        .expect("path-002 expression should parse");

    let mut control = InvocationControl::unbounded();
    let selected = evaluate_location_path_controlled(&document, doc, &path, &mut control)
        .expect("unbounded evaluation should succeed");

    assert_eq!(path.origin, PathOrigin::Descendant);
    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "right");
    assert_eq!(
        path.final_predicate,
        Some(ExistencePredicate {
            axis: PredicateAxis::Ancestor,
            name: "element2".to_owned(),
        })
    );
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 11);
}

#[test]
fn ancestor_or_self_predicate_checks_the_candidate_before_its_parent() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><element2><child2>right</child2></element2></doc>",
        ParseLimits {
            max_events: 24,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let self_path = parse_location_path("//element2[ancestor-or-self::element2]", location())
        .expect("ancestor-or-self self match should parse");
    let ancestor_path = parse_location_path("//child2[ancestor-or-self::element2]", location())
        .expect("path-003 expression should parse");

    let self_selected = evaluate_location_path(&document, doc, &self_path);
    let ancestor_selected = evaluate_location_path(&document, doc, &ancestor_path);

    assert_eq!(self_selected.len(), 1);
    assert_eq!(document.string_value(ancestor_selected[0]), "right");
    assert_eq!(
        self_path.final_predicate,
        Some(ExistencePredicate {
            axis: PredicateAxis::AncestorOrSelf,
            name: "element2".to_owned(),
        })
    );
}

#[test]
fn attribute_predicate_inspects_attributes_without_making_them_children() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><child2/><child2 attr1=\"yes\">right</child2></doc>",
        ParseLimits {
            max_events: 24,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let path = parse_location_path("//child2[attribute::attr1]", location())
        .expect("path-004 expression should parse");

    let mut control = InvocationControl::unbounded();
    let selected = evaluate_location_path_controlled(&document, doc, &path, &mut control)
        .expect("unbounded evaluation should succeed");

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "right");
    assert_eq!(document.children(selected[0]).len(), 1);
    assert_eq!(document.attributes(selected[0]).len(), 1);
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 5);
    assert_eq!(
        path.final_predicate,
        Some(ExistencePredicate {
            axis: PredicateAxis::Attribute,
            name: "attr1".to_owned(),
        })
    );
}

#[test]
fn descendant_or_self_predicate_checks_self_then_document_order_descendants() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><element1><child2>right</child2></element1><element1><child1/></element1><child2>self</child2></doc>",
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let descendant_path = parse_location_path("element1[descendant-or-self::child2]", location())
        .expect("path-005 expression should parse");
    let self_path = parse_location_path("child2[descendant-or-self::child2]", location())
        .expect("descendant-or-self self match should parse");
    let mut control = InvocationControl::unbounded();

    let descendant_selected =
        evaluate_location_path_controlled(&document, doc, &descendant_path, &mut control)
            .expect("unbounded evaluation should succeed");
    let self_selected = evaluate_location_path(&document, doc, &self_path);

    assert_eq!(descendant_selected.len(), 1);
    assert_eq!(document.string_value(descendant_selected[0]), "right");
    assert_eq!(document.string_value(self_selected[0]), "self");
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 7);
    assert_eq!(
        descendant_path.final_predicate,
        Some(ExistencePredicate {
            axis: PredicateAxis::DescendantOrSelf,
            name: "child2".to_owned(),
        })
    );
}

#[test]
fn parent_predicate_checks_only_the_immediate_parent() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><element1><child1>right</child1></element1><element2><child1>wrong</child1></element2></doc>",
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let path = parse_location_path("//child1[parent::element1]", location())
        .expect("path-006 expression should parse");
    let mut control = InvocationControl::unbounded();

    let selected = evaluate_location_path_controlled(&document, doc, &path, &mut control)
        .expect("unbounded evaluation should succeed");

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "right");
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 9);
    assert_eq!(
        path.final_predicate,
        Some(ExistencePredicate {
            axis: PredicateAxis::Parent,
            name: "element1".to_owned(),
        })
    );
}

#[test]
fn constant_integer_arithmetic_selects_the_matching_node_position() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><element1>wrong</element1><skip/><element1>right</element1><element1>wrong</element1></doc>",
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let path = parse_location_path("element1[(((((2*10)-4)+9) div 5) mod 3 )]", location())
        .expect("path-007 expression should parse");
    let mut control = InvocationControl::unbounded();

    let selected = evaluate_location_path_controlled(&document, doc, &path, &mut control)
        .expect("unbounded evaluation should succeed");

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "right");
    assert_eq!(
        path.step_position_predicates[0],
        Some(PositionPredicate::Select(2))
    );
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 4);
}

#[test]
fn applies_positions_to_individual_steps_and_last_to_the_matched_sequence() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><element1>wrong</element1><element1><child1>wrong</child1><child1>wrong</child1><child1>right</child1></element1><element1>wrong</element1></doc>",
        ParseLimits {
            max_events: 40,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let path = parse_location_path(
        "doc/element1[(((((2*10)-4)+9) div 5) mod 3)]/child1[last()]",
        location(),
    )
    .expect("path-010 selection should parse");
    let mut control = InvocationControl::unbounded();

    let selected =
        evaluate_location_path_controlled(&document, document.document_node(), &path, &mut control)
            .expect("unbounded evaluation should succeed");

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "right");
    assert_eq!(
        path.step_position_predicates,
        [
            None,
            Some(PositionPredicate::Select(2)),
            Some(PositionPredicate::Last),
        ]
    );
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 7);
}

#[test]
fn following_sibling_axis_filters_then_applies_position() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><a/><skip/><target/><target/></doc>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let a = document.children(doc)[0];
    let any = parse_location_path("following-sibling::*[1]", location())
        .expect("following sibling wildcard should parse");
    let named = parse_location_path("following-sibling::target[2]", location())
        .expect("following sibling name should parse");

    let first = evaluate_location_path(&document, a, &any);
    let second_target = evaluate_location_path(&document, a, &named);

    assert_eq!(document.name(first[0]).expect("element name").local, "skip");
    assert_eq!(second_target, [document.children(doc)[3]]);
}

#[test]
fn evaluation_preserves_document_order_and_requires_no_namespace() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<catalog xmlns:n="urn:other"><item>first</item><n:item>namespaced</n:item><skip/><item>second</item><item.name>dotted</item.name></catalog>"#,
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let catalog = document.children(document.document_node())[0];
    let items = parse_location_path("item", location()).expect("item path should parse");
    let dotted = parse_location_path("item.name", location()).expect("dotted name should parse");
    let missing = parse_location_path("missing", location()).expect("missing path should parse");

    let selected = evaluate_location_path(&document, catalog, &items);

    assert_eq!(selected.len(), 2);
    assert_eq!(document.string_value(selected[0]), "first");
    assert_eq!(document.string_value(selected[1]), "second");
    assert_eq!(
        document.string_value(evaluate_location_path(&document, catalog, &dotted)[0]),
        "dotted"
    );
    assert!(evaluate_location_path(&document, catalog, &missing).is_empty());
}

#[test]
fn each_path_step_normalizes_convergent_nodes_in_document_order() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<r><a/><a/></r>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let path = parse_location_path("/r/a/..", location()).expect("parent path should parse");

    let selected = evaluate_location_path(&document, document.document_node(), &path);

    assert_eq!(selected, [root]);
}
