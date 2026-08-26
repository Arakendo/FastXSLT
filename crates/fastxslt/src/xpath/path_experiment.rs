use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChildPath {
    pub(crate) steps: Vec<String>,
    pub(crate) selects_context_item: bool,
    pub(crate) location: SourceLocation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PathFailure {
    Invalid {
        detail: String,
        location: SourceLocation,
    },
    Unsupported {
        detail: String,
        location: SourceLocation,
    },
}

pub(crate) fn parse_child_path(
    expression: &str,
    location: SourceLocation,
) -> Result<ChildPath, PathFailure> {
    if expression.is_empty() {
        return Err(PathFailure::Invalid {
            detail: "the path expression is empty".to_owned(),
            location,
        });
    }
    if expression == "." {
        return Ok(ChildPath {
            steps: Vec::new(),
            selects_context_item: true,
            location,
        });
    }
    if expression.starts_with('/')
        || expression.ends_with('/')
        || expression.contains("//")
        || expression.contains(['[', ']', '(', ')', '@', ':', '*'])
    {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice supports only relative unprefixed child-name paths: {expression}"
            ),
            location,
        });
    }

    if !expression.is_ascii() {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice does not yet classify or evaluate non-ASCII child names: {expression}"
            ),
            location,
        });
    }

    let steps: Vec<_> = expression.split('/').map(str::to_owned).collect();
    if steps.iter().any(|step| step == "." || step == "..") {
        return Err(PathFailure::Unsupported {
            detail: format!(
                "the private slice supports the context item only as the complete expression: {expression}"
            ),
            location,
        });
    }
    if steps.iter().any(|step| !is_ascii_ncname(step)) {
        return Err(PathFailure::Invalid {
            detail: format!("the private slice found an invalid child name in: {expression}"),
            location,
        });
    }
    Ok(ChildPath {
        steps,
        selects_context_item: false,
        location,
    })
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}

pub(crate) fn evaluate_child_path(
    document: &Document,
    context: NodeId,
    path: &ChildPath,
) -> Vec<NodeId> {
    let mut control = InvocationControl::unbounded();
    evaluate_child_path_controlled(document, context, path, &mut control)
        .expect("unbounded private control cannot reject XPath work")
}

pub(crate) fn evaluate_child_path_controlled(
    document: &Document,
    context: NodeId,
    path: &ChildPath,
    control: &mut InvocationControl,
) -> Result<Vec<NodeId>, ControlFailure> {
    if path.selects_context_item {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        return Ok(vec![context]);
    }
    let mut current = vec![context];
    for step in &path.steps {
        let mut next = Vec::new();
        for node in current {
            for child in document.children(node).iter().copied() {
                control.charge(WorkDomain::XPathNodeVisit, 1)?;
                if document.kind(child) == NodeKind::Element
                    && document
                        .name(child)
                        .is_some_and(|name| name.namespace.is_none() && name.local == *step)
                {
                    next.push(child);
                }
            }
        }
        current = next;
    }
    Ok(current)
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::{PathFailure, evaluate_child_path, parse_child_path};
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xdm::owned_tree_experiment::SourceLocation;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn location() -> SourceLocation {
        SourceLocation {
            resource: "memory:stylesheet.xsl".to_owned(),
            span: Range { start: 12, end: 25 },
        }
    }

    #[test]
    fn parses_the_golden_relative_child_path() {
        let path = parse_child_path("greeting/name", location()).expect("path should parse");

        assert_eq!(path.steps, ["greeting", "name"]);
        assert!(!path.selects_context_item);
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
        let path = parse_child_path(".", location()).expect("context item should parse");

        let selected = evaluate_child_path(&document, item, &path);

        assert_eq!(selected, [item]);
        assert!(path.selects_context_item);
    }

    #[test]
    fn distinguishes_invalid_from_unsupported_path_syntax() {
        assert!(matches!(
            parse_child_path("", location()),
            Err(PathFailure::Invalid { .. })
        ));
        let invalid = parse_child_path("1greeting/name", location())
            .expect_err("an invalid child-name path must fail");
        let unsupported = parse_child_path("greeting//name", location())
            .expect_err("a descendant path is outside the private slice");

        assert!(matches!(invalid, PathFailure::Invalid { .. }));
        assert!(matches!(unsupported, PathFailure::Unsupported { .. }));
        assert_eq!(failure_location(&invalid), &location());
        assert_eq!(failure_location(&unsupported), &location());
    }

    #[test]
    fn accepts_supported_ncname_punctuation_without_claiming_unicode_names() {
        let path = parse_child_path("catalog/item.name/item-2/_value", location())
            .expect("ASCII NCName punctuation belongs to the private grammar");
        assert_eq!(path.steps, ["catalog", "item.name", "item-2", "_value"]);

        assert!(matches!(
            parse_child_path("café/name", location()),
            Err(PathFailure::Unsupported { .. })
        ));
        assert!(matches!(
            parse_child_path("catalog/..", location()),
            Err(PathFailure::Unsupported { .. })
        ));
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
        let path = parse_child_path("greeting/name", location()).expect("path should parse");

        let selected = evaluate_child_path(&document, document.document_node(), &path);

        assert_eq!(selected.len(), 1);
        assert_eq!(document.string_value(selected[0]), "FastXSLT");
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
        let items = parse_child_path("item", location()).expect("item path should parse");
        let dotted = parse_child_path("item.name", location()).expect("dotted name should parse");
        let missing = parse_child_path("missing", location()).expect("missing path should parse");

        let selected = evaluate_child_path(&document, catalog, &items);

        assert_eq!(selected.len(), 2);
        assert_eq!(document.string_value(selected[0]), "first");
        assert_eq!(document.string_value(selected[1]), "second");
        assert_eq!(
            document.string_value(evaluate_child_path(&document, catalog, &dotted)[0]),
            "dotted"
        );
        assert!(evaluate_child_path(&document, catalog, &missing).is_empty());
    }

    fn failure_location(failure: &PathFailure) -> &SourceLocation {
        match failure {
            PathFailure::Invalid { location, .. } | PathFailure::Unsupported { location, .. } => {
                location
            }
        }
    }
}
