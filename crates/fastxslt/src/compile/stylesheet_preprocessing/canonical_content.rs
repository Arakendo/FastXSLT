//! Test-only canonical child-content reference. Logical text runs retain their
//! original fragments; no replacement XDM identities or source spans are made.

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};

use super::{
    StrictPreprocessingFailure, StylesheetPreprocessingProfile, stylesheet_visibility_view,
};

#[derive(Debug)]
pub(in crate::compile::golden_stylesheet_experiment) struct CanonicalContent<'a> {
    pub(in crate::compile::golden_stylesheet_experiment) source: &'a Document,
    pub(in crate::compile::golden_stylesheet_experiment) children: Vec<CanonicalChild<'a>>,
}

#[derive(Debug)]
pub(in crate::compile::golden_stylesheet_experiment) enum CanonicalChild<'a> {
    Node(NodeId),
    Text(Vec<TextFragment<'a>>),
}

#[derive(Debug)]
pub(in crate::compile::golden_stylesheet_experiment) struct TextFragment<'a> {
    pub(in crate::compile::golden_stylesheet_experiment) original_node: NodeId,
    pub(in crate::compile::golden_stylesheet_experiment) value: &'a str,
    pub(in crate::compile::golden_stylesheet_experiment) location: &'a SourceLocation,
}

#[derive(Debug)]
struct TextPosition<'a> {
    original_node: NodeId,
    location: &'a SourceLocation,
    decoded_byte_offset: usize,
}

/// Locate a UTF-8 boundary in the logical run without concatenating payloads.
/// At a fragment boundary, the following nonempty fragment owns the position;
/// EOF belongs to the final fragment. Offsets are into decoded text, NOT raw
/// XML: entity references and CDATA make adding them to a source span unsound.
/// The original fragment span remains the diagnostic anchor until a tokenizer
/// supplies a finer original-byte map. Invalid offsets produce no anchor.
fn locate_text_position<'a>(
    fragments: &[TextFragment<'a>],
    mut offset: usize,
    control: &mut InvocationControl,
) -> Result<Option<TextPosition<'a>>, StrictPreprocessingFailure> {
    for (index, fragment) in fragments.iter().enumerate() {
        control.charge(WorkDomain::XdmNode, 1)?;
        let last = index + 1 == fragments.len();
        if offset < fragment.value.len() || (last && offset == fragment.value.len()) {
            return Ok(fragment
                .value
                .is_char_boundary(offset)
                .then_some(TextPosition {
                    original_node: fragment.original_node,
                    location: fragment.location,
                    decoded_byte_offset: offset,
                }));
        }
        offset -= fragment.value.len();
    }
    Ok(None)
}

/// Apply the selected profile before coalescing surviving text. All text payload
/// and locations borrow the original immutable owner; only child/fragment slots
/// are retained. This is a local reference, not the graph lowering interface.
fn canonical_content<'a>(
    document: &'a Document,
    parent: NodeId,
    profile: StylesheetPreprocessingProfile,
    control: &mut InvocationControl,
) -> Result<CanonicalContent<'a>, StrictPreprocessingFailure> {
    let view = stylesheet_visibility_view(document, parent, profile, control)?;
    coalesce_visible_children(document, &view, parent, control)
}

pub(in crate::compile::golden_stylesheet_experiment) fn coalesce_visible_children<'a>(
    document: &'a Document,
    view: &Document,
    parent: NodeId,
    control: &mut InvocationControl,
) -> Result<CanonicalContent<'a>, StrictPreprocessingFailure> {
    debug_assert!(document.same_origin(view));
    let mut children = Vec::new();
    for child in view.children(parent).iter().copied() {
        control.charge(WorkDomain::XdmNode, 1)?;
        if document.kind(child) == NodeKind::Text {
            // Charge a fragment slot before retaining any borrowed payload.
            control.charge(WorkDomain::XdmNode, 1)?;
            let fragment = TextFragment {
                original_node: child,
                value: document.value(child).unwrap_or_default(),
                location: document.location(child),
            };
            if let Some(CanonicalChild::Text(fragments)) = children.last_mut() {
                fragments.push(fragment);
            } else {
                control.charge(WorkDomain::XdmNode, 1)?;
                children.push(CanonicalChild::Text(vec![fragment]));
            }
        } else {
            debug_assert!(!matches!(
                document.kind(child),
                NodeKind::Comment | NodeKind::ProcessingInstruction
            ));
            control.charge(WorkDomain::XdmNode, 1)?;
            children.push(CanonicalChild::Node(child));
        }
    }
    Ok(CanonicalContent {
        source: document,
        children,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_control_experiment::{CancellationToken, ControlFailure, WorkLimits};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn document(xml: &str) -> Document {
        Document::from_parsed(
            parse_document(
                "urn:canonical-stylesheet",
                xml.as_bytes(),
                ParseLimits {
                    max_events: 1000,
                    max_depth: 50,
                },
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn text(fragments: &[TextFragment<'_>]) -> String {
        fragments.iter().map(|fragment| fragment.value).collect()
    }

    #[test]
    fn canonical_preprocessing_merges_only_surviving_text_and_preserves_fragments() {
        let tree = document("<out> <!--ignored-->A<?ignored yes?> <z/> B<!--ignored--> </out>");
        let root = tree.children(tree.document_node())[0];
        let original = tree.children(root).to_vec();
        for (profile, expected) in [
            (StylesheetPreprocessingProfile::Modern, [" A ", " B "]),
            (StylesheetPreprocessingProfile::StrictXslt10, ["A", " B"]),
        ] {
            let content =
                canonical_content(&tree, root, profile, &mut InvocationControl::unbounded())
                    .unwrap();
            assert!(std::ptr::eq(
                std::ptr::from_ref(content.source),
                std::ptr::from_ref(&tree),
            ));
            assert_eq!(content.children.len(), 3);
            let CanonicalChild::Text(first) = &content.children[0] else {
                panic!("first run")
            };
            let CanonicalChild::Node(middle) = content.children[1] else {
                panic!("element boundary")
            };
            let CanonicalChild::Text(last) = &content.children[2] else {
                panic!("last run")
            };
            assert_eq!([text(first), text(last)], expected);
            assert_eq!(tree.name(middle).unwrap().local, "z");
            for fragment in first.iter().chain(last) {
                assert_eq!(fragment.location, tree.location(fragment.original_node));
                assert_eq!(Some(fragment.value), tree.value(fragment.original_node));
                assert!(std::ptr::eq(
                    fragment.location,
                    tree.location(fragment.original_node)
                ));
            }
            assert_eq!(tree.children(root), original);
        }
    }

    #[test]
    fn canonical_preprocessing_keeps_space_text_and_utf8_without_copying_payload() {
        let tree = document(
            r#"<xsl:text xmlns:xsl="http://www.w3.org/1999/XSL/Transform"> <!--ignored-->é<?ignored yes?>&#160; </xsl:text>"#,
        );
        let root = tree.children(tree.document_node())[0];
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            let content =
                canonical_content(&tree, root, profile, &mut InvocationControl::unbounded())
                    .unwrap();
            let [CanonicalChild::Text(fragments)] = content.children.as_slice() else {
                panic!("one logical text run")
            };
            assert_eq!(text(fragments), " é\u{a0} ");
            assert_eq!(fragments.len(), 3);
            for fragment in fragments {
                assert_eq!(
                    fragment.value.as_ptr(),
                    tree.value(fragment.original_node).unwrap().as_ptr()
                );
            }
        }
    }

    #[test]
    fn canonical_preprocessing_does_not_resurrect_forced_stripped_text() {
        let tree = document(
            r#"<out xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve"> <!--ignored--> <xsl:param/>A<!--ignored--> </out>"#,
        );
        let root = tree.children(tree.document_node())[0];
        let content = canonical_content(
            &tree,
            root,
            StylesheetPreprocessingProfile::Modern,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert_eq!(content.children.len(), 2);
        let CanonicalChild::Text(fragments) = &content.children[1] else {
            panic!("text after param")
        };
        assert_eq!(text(fragments), "A ");
        let strict = canonical_content(
            &tree,
            root,
            StylesheetPreprocessingProfile::StrictXslt10,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert_eq!(strict.children.len(), 3);
    }

    #[test]
    fn canonical_preprocessing_charges_retention_and_releases_failed_partial_content() {
        let tree = document("<out>A<!--ignored--> B<z/>C<?ignored yes?> D</out>");
        let root = tree.children(tree.document_node())[0];
        let original = tree.children(root).to_vec();
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            let mut visibility = InvocationControl::unbounded();
            stylesheet_visibility_view(&tree, root, profile, &mut visibility).unwrap();
            let view_work = visibility.consumed(WorkDomain::XdmNode);
            let mut measured = InvocationControl::unbounded();
            canonical_content(&tree, root, profile, &mut measured).unwrap();
            let required = measured.consumed(WorkDomain::XdmNode);
            assert!(required > view_work);
            for limit in [required, required - 1] {
                let mut control = InvocationControl::new(
                    CancellationToken::new(),
                    WorkLimits {
                        xdm_nodes: limit,
                        ..WorkLimits::unbounded()
                    },
                );
                let result = canonical_content(&tree, root, profile, &mut control);
                if limit == required {
                    result.unwrap();
                } else {
                    assert!(matches!(
                        result,
                        Err(StrictPreprocessingFailure::Control(
                            ControlFailure::BudgetExhausted {
                                domain: WorkDomain::XdmNode,
                                ..
                            }
                        ))
                    ));
                }
            }
            // Exhaustion after the view succeeds exercises partial canonical
            // retention, not only an early visibility-pass failure.
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xdm_nodes: view_work + 3,
                    ..WorkLimits::unbounded()
                },
            );
            assert!(canonical_content(&tree, root, profile, &mut control).is_err());
            let view = stylesheet_visibility_view(
                &tree,
                root,
                profile,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let mut cancelled =
                InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 3);
            assert!(matches!(
                coalesce_visible_children(&tree, &view, root, &mut cancelled),
                Err(StrictPreprocessingFailure::Control(
                    ControlFailure::Cancelled {
                        domain: WorkDomain::XdmNode
                    }
                ))
            ));
            assert_eq!(tree.children(root), original);
            canonical_content(&tree, root, profile, &mut InvocationControl::unbounded()).unwrap();
        }
    }

    #[test]
    fn canonical_preprocessing_text_positions_keep_fragment_ownership_and_utf8_boundaries() {
        let tree = document("<out>é<!--removed-->B<?removed yes?>😀</out>");
        let root = tree.children(tree.document_node())[0];
        let content = canonical_content(
            &tree,
            root,
            StylesheetPreprocessingProfile::Modern,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let [CanonicalChild::Text(fragments)] = content.children.as_slice() else {
            panic!("one logical run")
        };
        assert_eq!(text(fragments), "éB😀");
        for (offset, fragment_index, local_offset) in [(0, 0, 0), (2, 1, 0), (3, 2, 0), (7, 2, 4)] {
            let position =
                locate_text_position(fragments, offset, &mut InvocationControl::unbounded())
                    .unwrap()
                    .unwrap();
            assert_eq!(
                position.original_node,
                fragments[fragment_index].original_node
            );
            assert_eq!(position.decoded_byte_offset, local_offset);
            assert!(std::ptr::eq(
                position.location,
                fragments[fragment_index].location
            ));
        }
        for offset in [1, 4, 5, 6, 8, usize::MAX] {
            assert!(
                locate_text_position(fragments, offset, &mut InvocationControl::unbounded())
                    .unwrap()
                    .is_none()
            );
        }
    }

    #[test]
    fn canonical_preprocessing_decoded_offsets_do_not_fabricate_raw_xml_spans() {
        let tree = document("<out>&amp;<!--removed--><![CDATA[é]]></out>");
        let root = tree.children(tree.document_node())[0];
        let content = canonical_content(
            &tree,
            root,
            StylesheetPreprocessingProfile::Modern,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let [CanonicalChild::Text(fragments)] = content.children.as_slice() else {
            panic!("entity and CDATA logical run")
        };
        assert_eq!(text(fragments), "&é");
        let position = locate_text_position(fragments, 1, &mut InvocationControl::unbounded())
            .unwrap()
            .unwrap();
        assert_eq!(position.original_node, fragments[1].original_node);
        assert_eq!(position.decoded_byte_offset, 0);
        assert_eq!(position.location, tree.location(fragments[1].original_node));
        assert_ne!(position.location.span, fragments[0].location.span);
    }

    #[test]
    fn canonical_preprocessing_position_mapping_follows_selected_edition() {
        let tree = document("<out> <!--removed-->A<?removed yes?> </out>");
        let root = tree.children(tree.document_node())[0];
        let original_a = tree.children(root)[2];
        for (profile, offset) in [
            (StylesheetPreprocessingProfile::Modern, 1),
            (StylesheetPreprocessingProfile::StrictXslt10, 0),
        ] {
            let content =
                canonical_content(&tree, root, profile, &mut InvocationControl::unbounded())
                    .unwrap();
            let [CanonicalChild::Text(fragments)] = content.children.as_slice() else {
                panic!("one logical run")
            };
            let position =
                locate_text_position(fragments, offset, &mut InvocationControl::unbounded())
                    .unwrap()
                    .unwrap();
            assert_eq!(position.original_node, original_a);
            assert_eq!(position.decoded_byte_offset, 0);
            assert_eq!(position.location, tree.location(original_a));
        }
    }

    #[test]
    fn canonical_preprocessing_position_mapping_is_bounded_cancellable_and_reusable() {
        let tree = document("<out>A<!--removed-->B<?removed yes?>C</out>");
        let root = tree.children(tree.document_node())[0];
        let content = canonical_content(
            &tree,
            root,
            StylesheetPreprocessingProfile::Modern,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let [CanonicalChild::Text(fragments)] = content.children.as_slice() else {
            panic!("three fragments")
        };
        for limit in [3, 2] {
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xdm_nodes: limit,
                    ..WorkLimits::unbounded()
                },
            );
            let result = locate_text_position(fragments, 2, &mut control);
            if limit == 3 {
                assert_eq!(
                    result.unwrap().unwrap().original_node,
                    fragments[2].original_node
                );
                assert_eq!(control.consumed(WorkDomain::XdmNode), 3);
            } else {
                assert!(matches!(
                    result,
                    Err(StrictPreprocessingFailure::Control(
                        ControlFailure::BudgetExhausted {
                            domain: WorkDomain::XdmNode,
                            ..
                        }
                    ))
                ));
            }
        }
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 2);
        assert!(matches!(
            locate_text_position(fragments, 2, &mut control),
            Err(StrictPreprocessingFailure::Control(
                ControlFailure::Cancelled {
                    domain: WorkDomain::XdmNode
                }
            ))
        ));
        assert_eq!(
            locate_text_position(fragments, 2, &mut InvocationControl::unbounded())
                .unwrap()
                .unwrap()
                .original_node,
            fragments[2].original_node
        );
        assert!(
            locate_text_position(&[], 0, &mut InvocationControl::unbounded())
                .unwrap()
                .is_none()
        );
    }
}
