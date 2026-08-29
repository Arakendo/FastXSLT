use std::ops::Range;

use super::{
    ExistencePredicate, PathFailure, PathOrigin, PositionPredicate, PredicateAxis,
    evaluate_location_path, evaluate_location_path_controlled, parse_location_path,
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
fn parses_the_golden_relative_child_path() {
    let path = parse_location_path("greeting/name", location()).expect("path should parse");

    assert_eq!(path.steps, ["greeting", "name"]);
    assert_eq!(path.origin, PathOrigin::Relative);
    assert_eq!(path.final_predicate, None);
    assert_eq!(path.location, location());
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
fn distinguishes_invalid_from_unsupported_path_syntax() {
    assert!(matches!(
        parse_location_path("", location()),
        Err(PathFailure::Invalid { .. })
    ));
    let invalid = parse_location_path("1greeting/name", location())
        .expect_err("an invalid child-name path must fail");
    let unsupported = parse_location_path("greeting//name", location())
        .expect_err("a descendant path is outside the private slice");

    assert!(matches!(invalid, PathFailure::Invalid { .. }));
    assert!(matches!(unsupported, PathFailure::Unsupported { .. }));
    assert_eq!(failure_location(&invalid), &location());
    assert_eq!(failure_location(&unsupported), &location());
    assert!(matches!(
        parse_location_path("sum(for $i in item return $i)", location()),
        Err(PathFailure::Unsupported { .. })
    ));
}

#[test]
fn accepts_supported_ncname_punctuation_without_claiming_unicode_names() {
    let path = parse_location_path("catalog/item.name/item-2/_value", location())
        .expect("ASCII NCName punctuation belongs to the private grammar");
    assert_eq!(path.steps, ["catalog", "item.name", "item-2", "_value"]);

    assert!(matches!(
        parse_location_path("café/name", location()),
        Err(PathFailure::Unsupported { .. })
    ));
    assert!(matches!(
        parse_location_path("catalog/ancestor::node()", location()),
        Err(PathFailure::Unsupported { .. })
    ));
    let self_node = parse_location_path("catalog/self::node()", location())
        .expect("the admitted self node test should parse");
    assert_eq!(self_node.steps[1], "node()");
}

#[test]
fn explicit_and_abbreviated_axes_share_typed_step_semantics() {
    let implicit = parse_location_path("//center/south-east", location())
        .expect("implicit child steps should parse");
    let explicit = parse_location_path("//center/child::south-east", location())
        .expect("explicit named child-axis step should parse");

    assert_eq!(explicit.steps, implicit.steps);
    assert_eq!(explicit.origin, PathOrigin::Descendant);
    assert_eq!(
        parse_location_path("//center/*", location())
            .expect("abbreviated wildcard should parse")
            .steps,
        parse_location_path("//center/child::*", location())
            .expect("explicit wildcard should parse")
            .steps
    );
    assert_eq!(
        parse_location_path("//center/node()", location())
            .expect("abbreviated node test should parse")
            .steps,
        parse_location_path("//center/child::node()", location())
            .expect("explicit node test should parse")
            .steps
    );
    assert_eq!(
        parse_location_path("//west/@*", location())
            .expect("abbreviated attribute wildcard should parse")
            .steps,
        parse_location_path("//west/attribute::*", location())
            .expect("explicit attribute wildcard should parse")
            .steps
    );
    assert_eq!(
        parse_location_path("//west/@west-attr-2", location())
            .expect("abbreviated named attribute should parse")
            .steps,
        parse_location_path("//west/attribute::west-attr-2", location())
            .expect("explicit named attribute should parse")
            .steps
    );
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

fn failure_location(failure: &PathFailure) -> &SourceLocation {
    match failure {
        PathFailure::Invalid { location, .. } | PathFailure::Unsupported { location, .. } => {
            location
        }
    }
}
