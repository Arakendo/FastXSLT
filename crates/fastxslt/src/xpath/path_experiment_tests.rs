use std::ops::Range;

use super::{
    AxisPredicate, FinalContextPredicate, PathFailure, PathOrigin, PositionPredicate,
    PredicateAxis, StepPredicate, evaluate_location_path, evaluate_location_path_controlled,
    parse_location_path, parse_qualified_child_path, parse_xslt10_location_path,
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
fn qualified_child_and_attribute_steps_match_expanded_names() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<doc xmlns:a="urn:element" xmlns:b="urn:attribute"><a:item b:code="selected" code="other"/></doc>"#,
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let path =
        parse_qualified_child_path("doc/a:item/@b:code", location(), |prefix| match prefix {
            "a" => Some("urn:element".to_owned()),
            "b" => Some("urn:attribute".to_owned()),
            _ => None,
        })
        .expect("qualified child and attribute path should parse");

    let selected = evaluate_location_path(&document, document.document_node(), &path);

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "selected");
}

#[test]
fn qualified_descendant_paths_preserve_document_and_context_origins() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<outer xmlns:p="urn:items"><p:item>outer</p:item><inner><p:item>inner</p:item></inner></outer>"#,
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let resolve = |prefix: &str| (prefix == "p").then(|| "urn:items".to_owned());
    let document_path = parse_qualified_child_path("//p:item", location(), resolve)
        .expect("document descendant path should parse");
    let context_path = parse_qualified_child_path(".//p:item", location(), resolve)
        .expect("context descendant path should parse");
    let outer = document.children(document.document_node())[0];
    let inner = document
        .children(outer)
        .iter()
        .copied()
        .find(|node| {
            document
                .name(*node)
                .is_some_and(|name| name.local == "inner")
        })
        .expect("inner element");

    let document_selected =
        evaluate_location_path(&document, document.document_node(), &document_path);
    let context_selected = evaluate_location_path(&document, inner, &context_path);

    assert_eq!(document_selected.len(), 2);
    assert_eq!(context_selected.len(), 1);
    assert_eq!(document.string_value(context_selected[0]), "inner");
}

#[test]
fn qualified_descendant_namespace_wildcards_filter_elements_and_attributes() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<outer xmlns:p="urn:selected" xmlns:q="urn:other"><p:item p:code="yes" q:code="no"/><q:item p:code="also"/></outer>"#,
        ParseLimits {
            max_events: 16,
            max_depth: 3,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let resolve = |prefix: &str| (prefix == "p").then(|| "urn:selected".to_owned());
    let elements = parse_qualified_child_path("//p:*", location(), resolve)
        .expect("element namespace wildcard should parse");
    let attributes = parse_qualified_child_path("//@p:*", location(), resolve)
        .expect("attribute namespace wildcard should parse");

    let selected_elements = evaluate_location_path(&document, document.document_node(), &elements);
    let selected_attributes =
        evaluate_location_path(&document, document.document_node(), &attributes);

    assert_eq!(selected_elements.len(), 1);
    assert_eq!(selected_attributes.len(), 2);
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
fn local_name_predicate_selects_namespaced_and_unnamespaced_children() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<doc xmlns:p="urn:selected"><bar>one</bar><p:bar>two</p:bar><other>no</other></doc>"#,
        ParseLimits {
            max_events: 16,
            max_depth: 3,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let path = parse_location_path("*[local-name()='bar']", location())
        .expect("literal local-name predicate should parse");

    let selected = evaluate_location_path(&document, doc, &path);

    assert_eq!(selected.len(), 2);
    assert_eq!(document.string_value(selected[0]), "one");
    assert_eq!(document.string_value(selected[1]), "two");
}

#[test]
fn reverse_axes_apply_positions_in_axis_order_then_normalize_document_order() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><outside/><scope><first/><second/><context/></scope></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let scope = document.children(root)[1];
    let context = document.children(scope)[2];

    let sibling_nearest = parse_location_path("preceding-sibling::*[1]", location())
        .expect("preceding-sibling nearest path");
    let sibling_farthest = parse_location_path("preceding-sibling::*[last()]", location())
        .expect("preceding-sibling farthest path");
    let preceding_nearest =
        parse_location_path("preceding::*[1]", location()).expect("preceding nearest path");
    let preceding_farthest =
        parse_location_path("preceding::*[last()]", location()).expect("preceding farthest path");

    let selected_name = |path| {
        let selected = evaluate_location_path(&document, context, path);
        document
            .name(selected[0])
            .expect("selected element")
            .local
            .clone()
    };
    assert_eq!(selected_name(&sibling_nearest), "second");
    assert_eq!(selected_name(&sibling_farthest), "first");
    assert_eq!(selected_name(&preceding_nearest), "second");
    assert_eq!(selected_name(&preceding_farthest), "outside");
}

#[test]
fn preceding_axis_filters_non_element_kinds_before_reverse_positions() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root>lead<!--first--><?one ready?><a/>middle<!--nearest--><?two ready?><context/></root>",
        ParseLimits {
            max_events: 20,
            max_depth: 3,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let context = *document.children(root).last().expect("context child");
    let nearest_comment =
        parse_location_path("preceding::comment()[1]", location()).expect("preceding comment path");
    let farthest_pi =
        parse_location_path("preceding::processing-instruction()[last()]", location())
            .expect("preceding processing-instruction path");
    let all_text =
        parse_location_path("preceding::text()", location()).expect("preceding text path");

    let selected_comment = evaluate_location_path(&document, context, &nearest_comment);
    let selected_pi = evaluate_location_path(&document, context, &farthest_pi);
    let selected_text = evaluate_location_path(&document, context, &all_text);

    assert_eq!(document.value(selected_comment[0]), Some("nearest"));
    assert_eq!(
        document.name(selected_pi[0]).expect("PI target").local,
        "one"
    );
    assert_eq!(
        selected_text
            .into_iter()
            .map(|node| document.value(node).expect("text value"))
            .collect::<Vec<_>>(),
        ["lead", "middle"]
    );
}

#[test]
fn ancestor_axes_apply_positions_in_reverse_axis_order() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><middle><leaf/></middle></root>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let middle = document.children(root)[0];
    let leaf = document.children(middle)[0];

    for (expression, expected) in [
        ("ancestor::*[1]", "middle"),
        ("ancestor::*[last()]", "root"),
        ("ancestor-or-self::*[1]", "leaf"),
        ("ancestor-or-self::*[last()]", "root"),
    ] {
        let path = parse_location_path(expression, location()).expect("ancestor path");
        let selected = evaluate_location_path(&document, leaf, &path);
        assert_eq!(selected.len(), 1, "{expression}");
        assert_eq!(
            document.name(selected[0]).expect("selected element").local,
            expected,
            "{expression}"
        );
    }

    let all = parse_location_path("ancestor-or-self::node()", location())
        .expect("ancestor-or-self node path");
    assert_eq!(
        evaluate_location_path(&document, leaf, &all),
        [document.document_node(), root, middle, leaf]
    );
}

#[test]
fn following_axis_excludes_context_descendants_and_applies_forward_positions() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><context><excluded/></context><after><inside/></after><tail/></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let context = document.children(root)[0];
    let all = parse_location_path("following::*", location()).expect("following path");
    let first = parse_location_path("following::*[1]", location()).expect("first following path");
    let last =
        parse_location_path("following::*[last()]", location()).expect("last following path");

    let names = |path| {
        evaluate_location_path(&document, context, path)
            .into_iter()
            .map(|node| document.name(node).expect("selected element").local.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(names(&all), ["after", "inside", "tail"]);
    assert_eq!(names(&first), ["after"]);
    assert_eq!(names(&last), ["tail"]);
}

#[test]
fn following_axis_filters_explicit_non_element_node_kinds() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><context/><!--note--><?work ready?>tail<after/></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 3,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let context = document.children(root)[0];

    for (expression, expected) in [
        ("following::comment()", "note"),
        ("following::processing-instruction()", "ready"),
        ("following::text()", "tail"),
    ] {
        let path = parse_location_path(expression, location()).expect("following kind-test path");
        let selected = evaluate_location_path(&document, context, &path);
        assert_eq!(selected.len(), 1, "{expression}");
        assert_eq!(document.string_value(selected[0]), expected, "{expression}");
    }
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
fn abbreviated_self_step_composes_inside_a_path() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root a=\"1\" b=\"2\"/>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let path = parse_location_path("@*/.", location()).expect("composed self step should parse");
    let mut control = InvocationControl::unbounded();

    let selected = evaluate_location_path_controlled(&document, root, &path, &mut control)
        .expect("composed self step should execute");

    assert_eq!(selected, document.attributes(root));
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 4);
}

#[test]
fn axis_separator_allows_xpath_whitespace() {
    let child = parse_location_path("child \t::\r\n sub", location())
        .expect("whitespace before and after the child-axis separator should parse");
    let attribute = parse_location_path("attribute :: *", location())
        .expect("whitespace around the attribute-axis separator should parse");

    assert_eq!(child.steps, ["sub"]);
    assert_eq!(attribute.steps, ["*"]);
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
        b"<root a=\"1\">text<!--c--><?p x?><child/></root>",
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
    let comment = document.children(root)[1];
    let processing_instruction = document.children(root)[2];
    let self_element =
        parse_location_path("self::*", location()).expect("self element wildcard should parse");
    let self_named =
        parse_location_path("self::root", location()).expect("named self step should parse");
    let self_node =
        parse_location_path("self::node()", location()).expect("self node test should parse");
    let self_text = parse_location_path("self::text()", location()).expect("self text test");
    let self_comment =
        parse_location_path("self::comment()", location()).expect("self comment test");
    let self_pi = parse_location_path("self::processing-instruction()", location())
        .expect("self processing-instruction test");
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
    assert_eq!(evaluate_location_path(&document, text, &self_text), [text]);
    assert_eq!(
        evaluate_location_path(&document, comment, &self_comment),
        [comment]
    );
    assert_eq!(
        evaluate_location_path(&document, processing_instruction, &self_pi),
        [processing_instruction]
    );
    assert!(evaluate_location_path(&document, root, &self_text).is_empty());
    assert!(evaluate_location_path(&document, text, &self_comment).is_empty());
    assert!(evaluate_location_path(&document, text, &self_pi).is_empty());
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

    for expression in ["attribute::text()", "parent::text()"] {
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
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 11);

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
    assert_eq!(self_control.consumed(WorkDomain::XPathNodeVisit), 12);
}

#[test]
fn leading_descendant_expands_contexts_before_evaluating_an_arbitrary_axis() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><leaf/><middle><nested/></middle><sibling/></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 5,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let path = parse_location_path("//ancestor::*", location())
        .expect("leading descendant ancestor step should parse");

    let selected = evaluate_location_path(&document, document.document_node(), &path);
    let names: Vec<_> = selected
        .into_iter()
        .map(|node| {
            document
                .name(node)
                .expect("element should have a name")
                .local
                .clone()
        })
        .collect();

    assert_eq!(names, ["root", "middle"]);
}

#[test]
fn leading_descendant_applies_a_step_predicate_per_expanded_context() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><a id='one'/><a id='two'/><group><a id='three'/><a id='four'/></group></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 5,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let path = parse_location_path("//a[1]", location())
        .expect("leading descendant positional step should parse");

    let selected = evaluate_location_path(&document, document.document_node(), &path);
    let ids: Vec<_> = selected
        .into_iter()
        .map(|node| {
            let attribute = document.attributes(node)[0];
            document
                .value(attribute)
                .expect("id should have a value")
                .to_owned()
        })
        .collect();

    assert_eq!(ids, ["one", "three"]);
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
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 20);
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
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 21);
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
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 15);
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
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 15);
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
    assert_eq!(descendant_control.consumed(WorkDomain::XPathNodeVisit), 6);
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
fn named_processing_instruction_test_filters_the_target_on_child_and_descendant_paths() {
    let parsed = parse_document(
        "memory:source.xml",
        br"<root><?other first?><a><?work selected?></a><?work selected-too?></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];

    let child = parse_location_path("processing-instruction('work')", location())
        .expect("named child PI test should parse");
    let descendants = parse_location_path(".//processing-instruction(\"work\")", location())
        .expect("named descendant PI test should parse");

    let child_selected = evaluate_location_path(&document, root, &child);
    let descendant_selected = evaluate_location_path(&document, root, &descendants);
    assert_eq!(child_selected.len(), 1);
    assert_eq!(document.value(child_selected[0]), Some("selected-too"));
    assert_eq!(descendant_selected.len(), 2);
    assert!(descendant_selected.iter().all(|node| {
        document
            .name(*node)
            .is_some_and(|name| name.local == "work")
    }));

    let literal_star = parse_location_path("processing-instruction('*')", location())
        .expect("XPath 1.0 literal-star PI test should parse");
    assert!(evaluate_location_path(&document, root, &literal_star).is_empty());
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
        path.final_predicate.as_deref(),
        Some(&AxisPredicate {
            axis: PredicateAxis::Child,
            name: "child2".to_owned(),
            value: None,
            position: None,
            conjunct: None,
        })
    );
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 5);
}

#[test]
fn abbreviated_named_child_predicate_reuses_child_axis_semantics() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><item><near-south/></item><item><north/></item></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let path = parse_location_path("self::*[near-south]", location())
        .expect("abbreviated named child predicate should parse");

    let first_item = document.children(root)[0];
    let second_item = document.children(root)[1];
    assert_eq!(
        evaluate_location_path(&document, first_item, &path),
        vec![first_item]
    );
    assert!(evaluate_location_path(&document, second_item, &path).is_empty());
    assert_eq!(
        path.final_predicate.as_ref().map(|value| value.axis),
        Some(PredicateAxis::Child)
    );
}

#[test]
fn text_child_predicate_uses_node_set_effective_boolean_value() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><item>present</item><item><child/></item><item><![CDATA[]]></item></root>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];

    for expression in ["item[text()]", "item[child::text()]"] {
        let path =
            parse_location_path(expression, location()).expect("text predicate should parse");
        let selected = evaluate_location_path(&document, root, &path);
        assert_eq!(selected.len(), 2);
        assert_eq!(document.string_value(selected[0]), "present");
        assert_eq!(document.string_value(selected[1]), "");
        assert_eq!(
            path.final_predicate.as_ref().map(|value| value.axis),
            Some(PredicateAxis::ChildText)
        );
    }
}

#[test]
fn language_predicate_reuses_context_language_semantics() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<doc xml:lang="en-US"><para id="1">A</para><para id="2" xml:lang="fr">B</para></doc>"#,
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let path = parse_location_path("para[@id='1' and lang('EN')]", location())
        .expect("conjoined attribute and language predicate should parse");
    let mut control = InvocationControl::unbounded();

    let selected = evaluate_location_path_controlled(&document, doc, &path, &mut control)
        .expect("unbounded evaluation should succeed");

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "A");
    assert_eq!(
        path.final_predicate.as_deref(),
        Some(&AxisPredicate {
            axis: PredicateAxis::Attribute,
            name: "id".to_owned(),
            value: Some("1".to_owned()),
            position: None,
            conjunct: Some(Box::new(AxisPredicate {
                axis: PredicateAxis::ContextLanguage,
                name: "EN".to_owned(),
                value: None,
                position: None,
                conjunct: None,
            })),
        })
    );
}

#[test]
fn conjoined_position_uses_the_unfiltered_candidate_focus() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<doc><item test="yes">one</item><item>two</item><item test="yes">three</item><item>four</item><item test="yes">five</item></doc>"#,
        ParseLimits {
            max_events: 24,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let third = parse_location_path("*[@test and position()=3]", location())
        .expect("conjoined attribute and position predicate should parse");
    let second = parse_location_path("*[@test and position()=2]", location())
        .expect("nonmatching candidate position should parse");

    let selected = evaluate_location_path(&document, doc, &third);

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "three");
    assert!(evaluate_location_path(&document, doc, &second).is_empty());
    assert_eq!(
        third.final_predicate.as_deref(),
        Some(&AxisPredicate {
            axis: PredicateAxis::Attribute,
            name: "test".to_owned(),
            value: None,
            position: Some(PositionPredicate::Select(3)),
            conjunct: None,
        })
    );
}

#[test]
fn missing_attribute_predicate_composes_with_a_chained_position() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<doc><item test="yes">one</item><item>two</item><item test="yes">three</item><item>four</item></doc>"#,
        ParseLimits {
            max_events: 20,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let path = parse_location_path("*[not(@test)][last()=position()]", location())
        .expect("missing-attribute predicate and symmetric last position should parse");

    let selected = evaluate_location_path(&document, doc, &path);

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "four");
    assert!(matches!(
        path.step_axis_predicates.as_slice(),
        [Some(AxisPredicate {
            axis: PredicateAxis::MissingAttribute,
            name,
            ..
        })] if name == "test"
    ));
    assert_eq!(
        path.step_position_predicates,
        vec![vec![StepPredicate::Position(PositionPredicate::Last)]]
    );
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
        path.final_predicate.as_deref(),
        Some(&AxisPredicate {
            axis: PredicateAxis::Ancestor,
            name: "element2".to_owned(),
            value: None,
            position: None,
            conjunct: None,
        })
    );
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 19);
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
        self_path.final_predicate.as_deref(),
        Some(&AxisPredicate {
            axis: PredicateAxis::AncestorOrSelf,
            name: "element2".to_owned(),
            value: None,
            position: None,
            conjunct: None,
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
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 10);
    assert_eq!(
        path.final_predicate.as_deref(),
        Some(&AxisPredicate {
            axis: PredicateAxis::Attribute,
            name: "attr1".to_owned(),
            value: None,
            position: None,
            conjunct: None,
        })
    );
}

#[test]
fn abbreviated_attribute_existence_predicate_uses_the_same_typed_path() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><item/><item selected=\"yes\">right</item></doc>",
        ParseLimits {
            max_events: 24,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let path = parse_location_path("*[@selected]", location())
        .expect("abbreviated attribute predicate should parse");

    let selected = evaluate_location_path_controlled(
        &document,
        doc,
        &path,
        &mut InvocationControl::unbounded(),
    )
    .expect("unbounded evaluation should succeed");

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "right");
    assert_eq!(
        path.final_predicate.as_deref(),
        Some(&AxisPredicate {
            axis: PredicateAxis::Attribute,
            name: "selected".to_owned(),
            value: None,
            position: None,
            conjunct: None,
        })
    );
}

#[test]
fn literal_attribute_value_predicate_filters_the_final_step() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><item selected=\"no\">wrong</item><item selected=\"yes\">right</item></doc>",
        ParseLimits {
            max_events: 24,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let path = parse_location_path("*[@selected='yes']", location())
        .expect("literal attribute-value predicate should parse");

    let selected = evaluate_location_path_controlled(
        &document,
        doc,
        &path,
        &mut InvocationControl::unbounded(),
    )
    .expect("unbounded evaluation should succeed");

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "right");
    assert_eq!(
        path.final_predicate.as_deref(),
        Some(&AxisPredicate {
            axis: PredicateAxis::Attribute,
            name: "selected".to_owned(),
            value: Some("yes".to_owned()),
            position: None,
            conjunct: None,
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
        descendant_path.final_predicate.as_deref(),
        Some(&AxisPredicate {
            axis: PredicateAxis::DescendantOrSelf,
            name: "child2".to_owned(),
            value: None,
            position: None,
            conjunct: None,
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
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 17);
    assert_eq!(
        path.final_predicate.as_deref(),
        Some(&AxisPredicate {
            axis: PredicateAxis::Parent,
            name: "element1".to_owned(),
            value: None,
            position: None,
            conjunct: None,
        })
    );
}

#[test]
fn xslt10_boolean_following_sibling_comparison_uses_axis_existence() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><a>1</a><a>2</a><a>3</a><last/></doc>",
        ParseLimits {
            max_events: 24,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let path = parse_xslt10_location_path("a[false() != following-sibling::*]", location())
        .expect("XSLT 1.0 boolean/node-set comparison should parse");
    let mut control = InvocationControl::unbounded();

    let selected = evaluate_location_path_controlled(&document, doc, &path, &mut control)
        .expect("unbounded evaluation should succeed");

    assert_eq!(selected.len(), 3);
    assert_eq!(document.string_value(selected[0]), "1");
    assert_eq!(document.string_value(selected[2]), "3");
    assert!(control.consumed(WorkDomain::XPathNodeVisit) > 0);
}

#[test]
fn attribute_wildcard_predicates_test_attribute_presence() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><a id='1'>present</a><a>missing</a></doc>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];

    let present = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("a[@*]", location()).expect("presence predicate should parse"),
    );
    let missing = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("a[not(@*)]", location()).expect("absence predicate should parse"),
    );

    assert_eq!(present.len(), 1);
    assert_eq!(document.string_value(present[0]), "present");
    assert_eq!(missing.len(), 1);
    assert_eq!(document.string_value(missing[0]), "missing");
}

#[test]
fn attribute_boolean_predicates_preserve_and_or_precedence() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><a squish='x' squash='x'>1</a><a squish='x' squeesh='x'>2</a><a squash='x' squeesh='x'>3</a><a squish='x'>4</a><a squeesh='x'>5</a><a squash='x'>6</a></doc>",
        ParseLimits {
            max_events: 40,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let values = |expression: &str| {
        evaluate_location_path(
            &document,
            doc,
            &parse_location_path(expression, location()).expect("boolean predicate should parse"),
        )
        .into_iter()
        .map(|node| document.string_value(node))
        .collect::<String>()
    };

    assert_eq!(values("a[@squeesh or (@squish and @squash)]"), "1235");
    assert_eq!(values("a[(@squeesh or @squish) and @squash]"), "13");
    assert_eq!(values("a[@squeesh or @squish and @squash]"), "1235");
}

#[test]
fn path_boolean_predicates_compose_descendant_equality_and_negation() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><a squish='heavy'>1</a><a>2<child>target</child></a><a>3</a><a>target</a><a>4<child>missed</child></a></doc>",
        ParseLimits {
            max_events: 30,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let values = |expression: &str| {
        evaluate_location_path(
            &document,
            doc,
            &parse_location_path(expression, location()).expect("boolean predicate should parse"),
        )
        .into_iter()
        .map(|node| document.string_value(node))
        .collect::<String>()
    };

    assert_eq!(values("a[descendant::*='target']"), "2target");
    assert_eq!(values("a[descendant::*!='target']"), "4missed");
    assert_eq!(
        values("a[not(('target'=descendant::*) or @squish)]"),
        "3target4missed"
    );
}

#[test]
fn path_boolean_predicate_compares_following_sibling_node_set_with_integer() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><a>1</a><a>2</a><a>3</a><a>4</a></doc>",
        ParseLimits {
            max_events: 20,
            max_depth: 3,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let values = |expression: &str| {
        evaluate_location_path(
            &document,
            doc,
            &parse_location_path(expression, location())
                .expect("numeric node-set predicate should parse"),
        )
        .into_iter()
        .map(|node| document.string_value(node))
        .collect::<String>()
    };

    assert_eq!(values("a[following-sibling::*=3]"), "12");
    assert_eq!(values("a[following-sibling::*!=4]"), "12");
    assert_eq!(values("a[following-sibling::* < 3]"), "1");
    assert_eq!(values("a[3 < following-sibling::*]"), "123");
    assert_eq!(values("a[following-sibling::* >= 3]"), "123");
    assert_eq!(values("a[3 >= following-sibling::*]"), "12");
    assert_eq!(values("a[following-sibling::* > 3]"), "123");
    assert_eq!(values("a[3 > following-sibling::*]"), "1");
    assert_eq!(values("a[following-sibling::* <= 3]"), "12");
    assert_eq!(values("a[3 <= following-sibling::*]"), "123");
}

#[test]
fn following_sibling_text_kind_test_selects_only_text_nodes() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><h1/>alpha<x/>beta<!--ignored--><?pi ignored?></doc>",
        ParseLimits {
            max_events: 16,
            max_depth: 3,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let selected = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("h1/following-sibling::text()", location())
            .expect("following-sibling text kind test should parse"),
    );

    assert_eq!(selected.len(), 2);
    assert_eq!(document.string_value(selected[0]), "alpha");
    assert_eq!(document.string_value(selected[1]), "beta");
}

#[test]
fn path_boolean_predicate_compares_named_and_wildcard_children_with_integer() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><a s='named'><div>9</div></a><a s='wildcard'><other>9</other></a><a s='miss'><div>8</div></a></doc>",
        ParseLimits {
            max_events: 20,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];

    let named = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("a[div=9]", location())
            .expect("operator-shaped child name should parse as a node test"),
    );
    let wildcard = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("a[*=9]", location())
            .expect("wildcard child test should parse before multiplication"),
    );

    assert_eq!(named, [document.children(doc)[0]]);
    assert_eq!(
        wildcard,
        [document.children(doc)[0], document.children(doc)[1]]
    );
}

#[test]
fn path_boolean_predicate_compares_parent_attributes_with_string_literals() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><group pick='yes'><item>kept</item></group><group pick='no'><item>missed</item></group></doc>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let selected = evaluate_location_path(
        &document,
        document.document_node(),
        &parse_location_path("doc//item[../@pick='yes']", location())
            .expect("parent attribute comparison should parse"),
    );

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "kept");
}

#[test]
fn path_boolean_predicate_compares_dynamic_node_sets_and_positional_children() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><a><inner>match</inner></a><peer>miss</peer><peer>match</peer><foo><bar>first</bar><bar>this</bar></foo><foo><bar>this</bar><bar>other</bar></foo><foo><bar><baz>x</baz><baz>goodbye</baz></bar></foo><foo><bar>x</bar><bar><baz>x</baz><baz>goodbye</baz></bar></foo></doc>",
        ParseLimits {
            max_events: 64,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];

    let sibling_match = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("a[following-sibling::*=descendant::*]", location())
            .expect("node-set equality predicate should parse"),
    );
    let positional_match = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("foo[(bar[2])='this']", location())
            .expect("positional child equality predicate should parse"),
    );
    let nested_any = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("foo[(bar[(baz[2])='goodbye'])]", location())
            .expect("nested child equality predicate should parse"),
    );
    let nested_second = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("foo[(bar[2][(baz[2])='goodbye'])]", location())
            .expect("nested positioned child equality predicate should parse"),
    );

    assert_eq!(sibling_match, [document.children(doc)[0]]);
    assert_eq!(positional_match, [document.children(doc)[3]]);
    assert_eq!(
        nested_any,
        [document.children(doc)[5], document.children(doc)[6]]
    );
    assert_eq!(nested_second, [document.children(doc)[6]]);
}

#[test]
fn path_boolean_predicate_compares_child_node_sets_with_string_literals() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><book><title>Book 1</title><author real='yes'><last>Smith</last></author></book><book><title>Book 2</title><author real='no'><last>Jones</last><last>Smith</last></author></book><book/></doc>",
        ParseLimits {
            max_events: 40,
            max_depth: 5,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let values = |expression: &str| {
        evaluate_location_path(
            &document,
            doc,
            &parse_location_path(expression, location())
                .expect("child node-set comparison should parse"),
        )
    };

    assert_eq!(values("book[title='Book 2']"), [document.children(doc)[1]]);
    assert_eq!(
        values("book['Smith'=author/last]"),
        [document.children(doc)[0], document.children(doc)[1]]
    );
    assert_eq!(
        values("book[author/last!='Smith']"),
        [document.children(doc)[1]]
    );
    assert_eq!(
        values("book[author/@real='no']"),
        [document.children(doc)[1]]
    );
}

#[test]
fn path_boolean_predicate_compares_text_children_with_a_string_literal() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><item>Mary</item><item><child>Mary</child></item><item>Other</item></doc>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];

    let selected = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("item[text()='Mary']", location())
            .expect("text child comparison should parse"),
    );

    assert_eq!(selected, [document.children(doc)[0]]);
}

#[test]
fn path_boolean_predicate_negates_context_string_equality() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><a>value</a><a></a></doc>",
        ParseLimits {
            max_events: 12,
            max_depth: 3,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let selected = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("a[not(.='')]", location())
            .expect("context string predicate should parse"),
    );

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "value");
}

#[test]
fn path_boolean_predicates_project_the_context_lexical_name() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc xmlns:p='urn:test'><foo/><bar/><p:fizz/></doc>",
        ParseLimits {
            max_events: 16,
            max_depth: 3,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let starts_with = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("*[starts-with(name(.),'f')]", location())
            .expect("name prefix predicate should parse"),
    );
    let length = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("*[string-length(name(.))=3]", location())
            .expect("name length predicate should parse"),
    );
    let equal = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("*[name()='p:fizz']", location())
            .expect("name equality predicate should parse"),
    );
    let not_equal = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("*['bar'!=name(.)]", location())
            .expect("reversed name inequality predicate should parse"),
    );

    assert_eq!(starts_with.len(), 1);
    assert_eq!(document.name(starts_with[0]).unwrap().local, "foo");
    assert_eq!(length.len(), 2);
    assert_eq!(document.name(length[0]).unwrap().local, "foo");
    assert_eq!(document.name(length[1]).unwrap().local, "bar");
    assert_eq!(equal.len(), 1);
    assert_eq!(document.name(equal[0]).unwrap().local, "fizz");
    assert_eq!(not_equal, [starts_with[0], equal[0]]);
}

#[test]
fn path_boolean_predicates_count_children_and_measure_attributes() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><row ex=''><td/><td/><td/></row><row ex='abc'><td/><td/></row><row><td/><td/><td/></row></doc>",
        ParseLimits {
            max_events: 32,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let three_children = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("row[count(./td)=3]", location())
            .expect("child count predicate should parse"),
    );
    let empty_attribute = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("row[string-length(@ex)=0]", location())
            .expect("attribute length predicate should parse"),
    );
    let nonempty_attribute = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("row[@ex!='']", location())
            .expect("attribute inequality predicate should parse"),
    );
    let positive_attribute_length = evaluate_location_path(
        &document,
        doc,
        &parse_location_path("row[string-length(@ex)>0]", location())
            .expect("attribute length comparison should parse"),
    );

    assert_eq!(three_children.len(), 2);
    assert_eq!(empty_attribute.len(), 2);
    assert_eq!(empty_attribute, [three_children[0], three_children[1]]);
    assert_eq!(nonempty_attribute.len(), 1);
    assert_eq!(nonempty_attribute[0], document.children(doc)[1]);
    assert_eq!(positive_attribute_length, nonempty_attribute);
}

#[test]
fn path_boolean_predicates_compare_relative_element_path_counts() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><empty/><nested><child><leaf/></child></nested><one><child/></one></doc>",
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let doc = document.children(document.document_node())[0];
    let path = parse_location_path("*[count(./*/*) > 0]", location())
        .expect("relative element count predicate should parse");
    let reversed = parse_location_path("*[0 < count(./*/*)]", location())
        .expect("reversed relative element count predicate should parse");
    let mut control = InvocationControl::unbounded();

    let selected = evaluate_location_path_controlled(&document, doc, &path, &mut control)
        .expect("relative element count evaluation should succeed");
    let reversed_selected = evaluate_location_path(&document, doc, &reversed);

    assert_eq!(selected, [document.children(doc)[1]]);
    assert_eq!(reversed_selected, selected);
    assert!(control.consumed(WorkDomain::XPathNodeVisit) > 0);
    assert_eq!(control.consumed(WorkDomain::XPathOperation), 3);
}

#[test]
fn path_boolean_predicates_compare_ancestor_element_counts() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<far-north><north><near-north><center><near-south><south><far-south/></south></near-south></center></near-north></north></far-north>",
        ParseLimits {
            max_events: 32,
            max_depth: 12,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let path = parse_location_path("//*[count(ancestor::*) >= 2]/../parent::*", location())
        .expect("ancestor element count predicate should parse");
    let mut control = InvocationControl::unbounded();

    let selected =
        evaluate_location_path_controlled(&document, document.document_node(), &path, &mut control)
            .expect("ancestor element count evaluation should succeed");
    let selected_names = selected
        .iter()
        .map(|node| document.name(*node).expect("element name").local.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        selected_names,
        ["far-north", "north", "near-north", "center", "near-south"]
    );
    assert!(control.consumed(WorkDomain::XPathNodeVisit) > 0);
}

#[test]
fn path_boolean_predicates_select_nested_relative_path_existence() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<root><doc><element1><foo><bar/></foo></element1></doc><doc><element1><foo/></element1></doc></root>",
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let path = parse_location_path("doc[(element1/foo)[bar]]", location())
        .expect("nested relative-path existence predicate should parse");
    let mut control = InvocationControl::unbounded();

    let selected = evaluate_location_path_controlled(&document, root, &path, &mut control)
        .expect("nested relative-path existence evaluation should succeed");

    assert_eq!(selected, [document.children(root)[0]]);
    assert!(control.consumed(WorkDomain::XPathNodeVisit) > 0);
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
        [StepPredicate::Position(PositionPredicate::Select(2))]
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
            vec![],
            vec![StepPredicate::Position(PositionPredicate::Select(2))],
            vec![StepPredicate::Position(PositionPredicate::Last)],
        ]
    );
    assert_eq!(control.consumed(WorkDomain::XPathNodeVisit), 7);
}

#[test]
fn chained_axis_then_position_predicates_preserve_lexical_filter_order() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<docs><doc att1='outer'><doc att1='inner'><leaf/></doc></doc></docs>",
        ParseLimits {
            max_events: 16,
            max_depth: 6,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let docs = document.children(document.document_node())[0];
    let outer = document.children(docs)[0];
    let inner = document.children(outer)[0];
    let leaf = document.children(inner)[0];
    let path = parse_location_path("ancestor-or-self::*[@att1][1]/@att1", location())
        .expect("axis predicate followed by position should parse");

    let selected = evaluate_location_path(&document, leaf, &path);

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "inner");
    assert!(path.step_axis_predicates[0].is_some());
    assert_eq!(
        path.step_position_predicates[0],
        [StepPredicate::Position(PositionPredicate::Select(1))]
    );
    assert!(matches!(
        parse_location_path("ancestor-or-self::*[1][@att1]/@att1", location()),
        Err(PathFailure::Unsupported { .. })
    ));

    let filtered_path = parse_location_path("(ancestor-or-self::*)[@att1][1]/@att1", location())
        .expect("parenthesized reverse-axis filter should parse");
    let filtered = evaluate_location_path(&document, leaf, &filtered_path);
    assert_eq!(filtered.len(), 1);
    assert_eq!(document.string_value(filtered[0]), "outer");
    assert!(filtered_path.first_step_predicates_use_document_order);

    let enclosed_step = parse_location_path("(((ancestor::doc[1]))/@att1)", location())
        .expect("parenthesized reverse-axis step should preserve proximity order");
    let enclosed = evaluate_location_path(&document, leaf, &enclosed_step);
    assert_eq!(enclosed.len(), 1);
    assert_eq!(document.string_value(enclosed[0]), "inner");
    assert!(!enclosed_step.first_step_predicates_use_document_order);

    let grouped_axis = parse_location_path("((ancestor::doc))[1]/@att1", location())
        .expect("predicate outside a grouped reverse axis should use document order");
    let grouped = evaluate_location_path(&document, leaf, &grouped_axis);
    assert_eq!(grouped.len(), 1);
    assert_eq!(document.string_value(grouped[0]), "outer");
    assert!(grouped_axis.first_step_predicates_use_document_order);
}

#[test]
fn chained_axis_then_explicit_position_equality_reuses_the_typed_focus() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><item/><item test='yes'><num>1</num></item><item test='yes'><num>2</num></item></doc>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let path = parse_location_path("*[@test][position() = 2]/num", location())
        .expect("bounded explicit position equality should compile");
    let selected = evaluate_location_path(&document, root, &path);

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "2");
}

#[test]
fn explicit_position_relations_filter_the_typed_step_focus() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><a>1</a><a>2</a><a>3</a><a>4</a></doc>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];

    for (expression, expected) in [
        ("a[position()>2]", "34"),
        ("a[position()<3]", "12"),
        ("a[position()>=2]", "234"),
        ("a[position()<=3]", "123"),
        ("a[position()!=2]", "134"),
    ] {
        let path = parse_location_path(expression, location()).expect("position relation");
        let actual = evaluate_location_path(&document, root, &path)
            .into_iter()
            .map(|node| document.string_value(node))
            .collect::<String>();
        assert_eq!(actual, expected, "{expression}");
    }
}

#[test]
fn last_minus_constant_selects_relative_to_the_typed_step_focus() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><a>1</a><a>2</a><a>3</a><a>4</a></doc>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];

    for (expression, expected) in [
        ("a[last()-0]", "4"),
        ("a[last()-1]", "3"),
        ("a[last()-3]", "1"),
        ("a[last()-4]", ""),
        ("a[last()][last()]", "4"),
        ("a[1][last()]", "1"),
        ("a[last()-1][1]", "3"),
        ("a[last()-1][last()]", "3"),
        ("a[number('3')]", "3"),
    ] {
        let path = parse_location_path(expression, location()).expect("last-minus predicate");
        let actual = evaluate_location_path(&document, root, &path)
            .into_iter()
            .map(|node| document.string_value(node))
            .collect::<String>();
        assert_eq!(actual, expected, "{expression}");
    }
}

#[test]
fn trailing_name_predicate_observes_the_position_filtered_focus() {
    let parsed = parse_document(
        "memory:source.xml",
        b"<doc><alpha><z/></alpha><alpha><z/><z/></alpha><alpha><z/><e/></alpha></doc>",
        ParseLimits {
            max_events: 32,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let document = Document::from_parsed(parsed).expect("source XDM should build");
    let root = document.children(document.document_node())[0];
    let path = parse_location_path("alpha/*[last()][name()='z']", location())
        .expect("ordered position/name predicate chain");

    let selected = evaluate_location_path(&document, root, &path);

    assert_eq!(selected.len(), 2);
    assert!(
        selected
            .iter()
            .all(|node| { document.name(*node).is_some_and(|name| name.local == "z") })
    );
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
fn unicode_ncname_steps_match_unqualified_elements() {
    let parsed = parse_document(
        "memory:source.xml",
        "<文書><日本>selected</日本><別名>other</別名></文書>".as_bytes(),
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("Unicode source should parse");
    let document = Document::from_parsed(parsed).expect("Unicode source XDM should build");
    let path = parse_location_path("文書/日本", location())
        .expect("XML NCName location steps should parse");

    let selected = evaluate_location_path(&document, document.document_node(), &path);

    assert_eq!(selected.len(), 1);
    assert_eq!(document.string_value(selected[0]), "selected");
}

#[test]
fn xml_lang_attribute_steps_preserve_expanded_name_and_document_order() {
    let parsed = parse_document(
        "memory:source.xml",
        br#"<doc xml:lang="en"><group><item xml:lang="en-us"/></group></doc>"#,
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("xml:lang source should parse");
    let document = Document::from_parsed(parsed).expect("xml:lang source XDM should build");
    let doc = document.children(document.document_node())[0];
    let group = document.children(doc)[0];
    let item = document.children(group)[0];
    let path = parse_location_path("ancestor-or-self::*[@xml:lang]/@xml:lang", location())
        .expect("predeclared XML attribute path should parse");

    let selected = evaluate_location_path(&document, item, &path);

    assert_eq!(selected.len(), 2);
    assert_eq!(document.string_value(selected[0]), "en");
    assert_eq!(document.string_value(selected[1]), "en-us");
    assert!(selected.iter().all(|node| {
        document.name(*node).is_some_and(|name| {
            name.namespace.as_deref() == Some("http://www.w3.org/XML/1998/namespace")
                && name.local == "lang"
        })
    }));
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
