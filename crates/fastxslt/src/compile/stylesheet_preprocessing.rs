//! Stylesheet text preservation, independent of local expression compatibility.
//!
//! ADR-0023 integration checkpoint: shared text producers use this owner. The
//! test-only strict graph reference derives immutable relationships before
//! lowering. Production promotion, residual-consumer audit and the complete
//! modern preprocessing stage remain open; this is not a public profile API.

use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, is_xml_whitespace_only};

use super::{CompileFailure, invalid, optional_attribute};
use crate::xslt::golden_semantics_experiment::{
    ProcessorPolicyStamp, StylesheetPreprocessingStamp, StylesheetProgram,
};

#[cfg(test)]
#[path = "stylesheet_preprocessing/canonical_content.rs"]
pub(super) mod canonical_content;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StylesheetPreprocessingProfile {
    Modern,
    #[cfg(test)]
    StrictXslt10,
}

/// Compiler-owned qualification for modules in one already-preprocessed graph.
/// Processor and preprocessing stamps are independent fields, not inferred from
/// one another. This token does not implement either policy, validate a view,
/// infer a declared version or authorize resource access. The graph owner
/// supplies effective documents and the compiler must apply its selected policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StylesheetCompilationContext {
    preprocessing: StylesheetPreprocessingStamp,
    processor: ProcessorPolicyStamp,
}

impl StylesheetCompilationContext {
    pub(crate) const fn baseline() -> Self {
        Self {
            preprocessing: StylesheetPreprocessingStamp::ModernBaseline,
            processor: ProcessorPolicyStamp::DeclarationDrivenBaseline,
        }
    }

    #[cfg(test)]
    pub(crate) const fn for_reference(
        profile: StylesheetPreprocessingProfile,
        modern_reference: bool,
    ) -> Self {
        let preprocessing = match profile {
            StylesheetPreprocessingProfile::StrictXslt10 => {
                StylesheetPreprocessingStamp::StrictXslt10Reference
            }
            StylesheetPreprocessingProfile::Modern if modern_reference => {
                StylesheetPreprocessingStamp::ModernReference
            }
            StylesheetPreprocessingProfile::Modern => StylesheetPreprocessingStamp::ModernBaseline,
        };
        Self {
            preprocessing,
            ..Self::baseline()
        }
    }

    /// Only a compiler that actually applies the selected reference policy may
    /// use this context. This is not a public relabel operation on artifacts.
    #[cfg(test)]
    pub(crate) const fn with_processor_reference(
        mut self,
        processor: ProcessorPolicyStamp,
    ) -> Self {
        self.processor = processor;
        self
    }

    /// Finish a newly compiled module under the caller's actual context. Do not
    /// use this on an imported reusable artifact: composition checks its retained
    /// qualification without rewriting it.
    pub(crate) fn qualify_module(self, mut program: StylesheetProgram) -> StylesheetProgram {
        program.preprocessing = self.preprocessing;
        program.processor = self.processor;
        program
    }
}

/// Common constructor text eligibility. A strict view has already removed
/// strip-eligible fragments before any lowering path observes its relationships.
pub(super) fn constructor_children(
    document: &Document,
    parent: NodeId,
) -> Result<Vec<NodeId>, CompileFailure> {
    let preserve = super::is_xslt_element(document, parent, "text")
        || effective_xml_space_preserved(document, parent)?;
    let children = document.children(parent);
    Ok(children
        .iter()
        .enumerate()
        .filter_map(|(index, child)| match document.kind(*child) {
            NodeKind::Comment | NodeKind::ProcessingInstruction => None,
            NodeKind::Text => preserves_literal_text(
                document,
                children,
                index,
                preserve,
                StylesheetPreprocessingProfile::Modern,
            )
            .then_some(*child),
            _ => Some(*child),
        })
        .collect())
}

/// Safe strict-profile reference. Only the selected stylesheet subtree changes;
/// embedded XML ancestry and every original node's payload/provenance survive.
#[cfg(test)]
pub(crate) fn strict_stylesheet_view(
    document: &Document,
    root: NodeId,
    control: &mut crate::execution_control_experiment::InvocationControl,
) -> Result<Document, StrictPreprocessingFailure> {
    stylesheet_visibility_view(
        document,
        root,
        StylesheetPreprocessingProfile::StrictXslt10,
        control,
    )
}

/// Test-only modern visibility reference. Original text fragments remain stored;
/// merged runs govern eligibility, not physical node replacement or provenance.
#[cfg(test)]
pub(crate) fn stylesheet_visibility_view(
    document: &Document,
    root: NodeId,
    profile: StylesheetPreprocessingProfile,
    control: &mut crate::execution_control_experiment::InvocationControl,
) -> Result<Document, StrictPreprocessingFailure> {
    use crate::execution_control_experiment::WorkDomain;
    use std::collections::HashSet;

    control.charge(WorkDomain::XdmNode, 1)?;
    let mut pending = vec![root];
    let mut parents = HashSet::new();
    let mut retained = HashSet::new();
    while let Some(parent) = pending.pop() {
        control.charge(WorkDomain::XdmNode, 1)?;
        parents.insert(parent);
        let explicit = super::is_xslt_element(document, parent, "text")
            || effective_xml_space_preserved(document, parent)?;
        let children = document.children(parent);
        let requires_empty = profile == StylesheetPreprocessingProfile::Modern
            && requires_empty_content(document, parent);
        for (index, child) in children.iter().copied().enumerate() {
            control.charge(WorkDomain::XdmNode, 1)?;
            if profile == StylesheetPreprocessingProfile::Modern
                && document.kind(child) == NodeKind::Text
            {
                // Conservative reference bound: merged-run membership/value
                // scans and neighboring-run lookup stay charged before visits.
                for _ in 0..4 {
                    control.charge(WorkDomain::XdmNode, children.len())?;
                }
            }
            let keep = match document.kind(child) {
                NodeKind::Comment | NodeKind::ProcessingInstruction => false,
                NodeKind::Text => {
                    preserves_literal_text(document, children, index, explicit, profile)
                        && !(profile == StylesheetPreprocessingProfile::Modern
                            && modern_forces_whitespace_removal(document, parent, children, index))
                }
                _ => true,
            };
            if requires_empty && keep {
                return Err(invalid(
                    "XTSE0260",
                    "XSLT element required to be empty has content after preprocessing",
                    document.location(parent),
                )
                .into());
            }
            if keep {
                retained.insert(child);
            }
            if document.kind(child) == NodeKind::Element {
                pending.push(child);
            }
        }
    }
    document
        .view_filtering_children(
            |parent, child| !parents.contains(&parent) || retained.contains(&child),
            control,
        )
        .map_err(StrictPreprocessingFailure::Control)
}

/// Unconditionally empty declarations/instructions in the currently reviewed
/// slice. Conditional content models (for example select-bearing constructors)
/// remain owned by their instruction validators, not this lexical name table.
#[cfg(test)]
fn requires_empty_content(document: &Document, element: NodeId) -> bool {
    document.name(element).is_some_and(|name| {
        name.namespace.as_deref() == Some(super::XSLT_NAMESPACE)
            && matches!(
                name.local.as_str(),
                "include"
                    | "import"
                    | "output"
                    | "strip-space"
                    | "preserve-space"
                    | "decimal-format"
                    | "namespace-alias"
                    | "output-character"
                    | "mode"
                    | "context-item"
                    | "copy-of"
                    | "number"
            )
    })
}

/// Applied only to whitespace-only merged runs, after comment/PI removal.
#[cfg(test)]
fn modern_forces_whitespace_removal(
    document: &Document,
    parent: NodeId,
    children: &[NodeId],
    index: usize,
) -> bool {
    if preserves_literal_text(
        document,
        children,
        index,
        false,
        StylesheetPreprocessingProfile::Modern,
    ) {
        return false;
    }
    if document.name(parent).is_some_and(|name| {
        name.namespace.as_deref() == Some(super::XSLT_NAMESPACE)
            && matches!(
                name.local.as_str(),
                "accumulator"
                    | "analyze-string"
                    | "apply-imports"
                    | "apply-templates"
                    | "attribute-set"
                    | "call-template"
                    | "character-map"
                    | "choose"
                    | "evaluate"
                    | "fork"
                    | "merge"
                    | "merge-source"
                    | "mode"
                    | "next-iteration"
                    | "next-match"
                    | "override"
                    | "package"
                    | "stylesheet"
                    | "transform"
                    | "use-package"
            )
    }) {
        return true;
    }
    let next = children[index + 1..]
        .iter()
        .find(|node| !belongs_to_text_run(document.kind(**node)));
    let previous = children[..index]
        .iter()
        .rev()
        .find(|node| !belongs_to_text_run(document.kind(**node)));
    next.is_some_and(|node| {
        ["param", "sort", "context-item", "on-completion"]
            .iter()
            .any(|name| super::is_xslt_element(document, *node, name))
    }) || previous.is_some_and(|node| super::is_xslt_element(document, *node, "catch"))
}

#[cfg(test)]
#[derive(Debug)]
pub(crate) enum StrictPreprocessingFailure {
    Compile(CompileFailure),
    Control(crate::execution_control_experiment::ControlFailure),
}

#[cfg(test)]
impl From<CompileFailure> for StrictPreprocessingFailure {
    fn from(failure: CompileFailure) -> Self {
        Self::Compile(failure)
    }
}

#[cfg(test)]
impl From<crate::execution_control_experiment::ControlFailure> for StrictPreprocessingFailure {
    fn from(failure: crate::execution_control_experiment::ControlFailure) -> Self {
        Self::Control(failure)
    }
}

/// Classify original text fragments without copying text or changing provenance.
/// The caller has already resolved effective `xml:space`/`xsl:text` preservation.
/// Modern merged-run eligibility does not erase original fragment locations.
pub(super) fn preserves_literal_text(
    document: &Document,
    siblings: &[NodeId],
    text_index: usize,
    explicit_preservation: bool,
    profile: StylesheetPreprocessingProfile,
) -> bool {
    debug_assert_eq!(document.kind(siblings[text_index]), NodeKind::Text);
    if explicit_preservation {
        return true;
    }
    match profile {
        StylesheetPreprocessingProfile::Modern => {}
        #[cfg(test)]
        StylesheetPreprocessingProfile::StrictXslt10 => {
            return contains_non_whitespace(document, siblings[text_index]);
        }
    }
    let mut start = text_index;
    while start > 0 && belongs_to_text_run(document.kind(siblings[start - 1])) {
        start -= 1;
    }
    let mut end = text_index + 1;
    while end < siblings.len() && belongs_to_text_run(document.kind(siblings[end])) {
        end += 1;
    }
    siblings[start..end]
        .iter()
        .any(|sibling| contains_non_whitespace(document, *sibling))
}

fn belongs_to_text_run(kind: NodeKind) -> bool {
    matches!(
        kind,
        NodeKind::Text | NodeKind::Comment | NodeKind::ProcessingInstruction
    )
}

fn contains_non_whitespace(document: &Document, node: NodeId) -> bool {
    document.kind(node) == NodeKind::Text
        && document
            .value(node)
            .is_some_and(|value| !is_xml_whitespace_only(value))
}

pub(super) fn effective_xml_space_preserved(
    document: &Document,
    element: NodeId,
) -> Result<bool, CompileFailure> {
    const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(value) = optional_attribute(document, node, Some(XML_NAMESPACE), "space") {
            return match value {
                "preserve" => Ok(true),
                "default" => Ok(false),
                _ => Err(invalid(
                    "XTSE0020",
                    "xml:space must be 'default' or 'preserve'",
                    document.location(node),
                )),
            };
        }
        current = document.parent(node);
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn document(xml: &str) -> Document {
        Document::from_parsed(
            parse_document(
                "urn:preprocessing",
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

    fn element(document: &Document, local: &str) -> NodeId {
        let mut pending = vec![document.document_node()];
        while let Some(node) = pending.pop() {
            if document.kind(node) == NodeKind::Element
                && document.name(node).is_some_and(|name| name.local == local)
            {
                return node;
            }
            pending.extend(document.children(node));
        }
        panic!("test element {local} not found")
    }

    fn selected_text(
        document: &Document,
        parent: NodeId,
        profile: StylesheetPreprocessingProfile,
        explicit: bool,
    ) -> String {
        let children = document.children(parent);
        children
            .iter()
            .enumerate()
            .filter(|(index, node)| {
                document.kind(**node) == NodeKind::Text
                    && preserves_literal_text(document, children, *index, explicit, profile)
            })
            .map(|(_, node)| document.value(*node).unwrap_or_default())
            .collect()
    }

    #[test]
    fn profile_not_local_version_owns_comment_and_pi_boundaries() {
        for version in ["1.0", "3.0"] {
            let tree = document(&format!(
                r#"<s xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xsl:version="{version}"><out>A<!--ignored--> <?ignored yes?><z/> <!--ignored-->B</out></s>"#
            ));
            let parent = element(&tree, "out");
            assert_eq!(
                selected_text(&tree, parent, StylesheetPreprocessingProfile::Modern, false),
                "A  B"
            );
            assert_eq!(
                selected_text(
                    &tree,
                    parent,
                    StylesheetPreprocessingProfile::StrictXslt10,
                    false
                ),
                "AB"
            );
        }
    }

    #[test]
    fn elements_end_runs_and_comment_values_are_not_text() {
        let tree = document("<out> <!--not literal--> <?pi text?><z/>A<z/> </out>");
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            assert_eq!(
                selected_text(&tree, element(&tree, "out"), profile, false),
                "A"
            );
        }
    }

    #[test]
    fn inherited_space_and_default_reset_are_shared() {
        let tree = document(
            r#"<root xml:space="preserve"><out> <!--ignored--> </out><reset xml:space="default"> <!--ignored--> </reset></root>"#,
        );
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            let out = element(&tree, "out");
            assert_eq!(
                selected_text(
                    &tree,
                    out,
                    profile,
                    effective_xml_space_preserved(&tree, out).unwrap()
                ),
                "  "
            );
            let reset = element(&tree, "reset");
            assert_eq!(
                selected_text(
                    &tree,
                    reset,
                    profile,
                    effective_xml_space_preserved(&tree, reset).unwrap()
                ),
                ""
            );
        }
    }

    #[test]
    fn explicit_text_preservation_and_xml_whitespace_definition() {
        let tree = document("<out> \t<!--ignored-->\n&#160;</out>");
        let out = element(&tree, "out");
        assert_eq!(
            selected_text(
                &tree,
                out,
                StylesheetPreprocessingProfile::StrictXslt10,
                false
            ),
            "\n\u{a0}"
        );
        assert_eq!(
            selected_text(
                &tree,
                out,
                StylesheetPreprocessingProfile::StrictXslt10,
                true
            ),
            " \t\n\u{a0}"
        );
    }

    #[test]
    fn selection_does_not_mutate_original_nodes_or_locations() {
        let tree = document("<out>A<!--ignored--> <z/></out>");
        let parent = element(&tree, "out");
        let before: Vec<_> = tree
            .children(parent)
            .iter()
            .map(|id| {
                (
                    *id,
                    tree.value(*id).map(str::to_owned),
                    tree.location(*id).clone(),
                )
            })
            .collect();
        for profile in [
            StylesheetPreprocessingProfile::Modern,
            StylesheetPreprocessingProfile::StrictXslt10,
        ] {
            let _ = selected_text(&tree, parent, profile, false);
        }
        let after: Vec<_> = tree
            .children(parent)
            .iter()
            .map(|id| {
                (
                    *id,
                    tree.value(*id).map(str::to_owned),
                    tree.location(*id).clone(),
                )
            })
            .collect();
        assert_eq!(before, after);
        assert!(
            before
                .iter()
                .all(|(_, _, location)| location.resource == "urn:preprocessing")
        );
    }

    #[test]
    fn invalid_space_keeps_original_diagnostic_location() {
        let tree = document(r#"<root xml:space="invalid"><out/></root>"#);
        let failure = effective_xml_space_preserved(&tree, element(&tree, "out")).unwrap_err();
        assert_eq!(failure.code, "XTSE0020");
        assert_eq!(failure.location, *tree.location(element(&tree, "root")));
    }

    #[test]
    fn strict_view_is_immutable_and_keeps_original_identity_and_provenance() {
        use crate::execution_control_experiment::InvocationControl;
        let tree =
            document("<outer><out>A<!--ignored--> <z/></out><untouched> </untouched></outer>");
        let out = element(&tree, "out");
        let children = tree.children(out).to_vec();
        let view = strict_stylesheet_view(&tree, out, &mut InvocationControl::unbounded()).unwrap();
        assert!(view.same_origin(&tree));
        assert!(view.shares_node_storage_with(&tree));
        assert_eq!(tree.children(out), children);
        assert_eq!(view.children(out), &[children[0], children[3]]);
        assert_eq!(view.location(children[0]), tree.location(children[0]));
        let untouched = element(&tree, "untouched");
        assert_eq!(view.children(untouched), tree.children(untouched));
    }

    #[test]
    fn strict_view_exact_budget_one_less_and_cancellation_leave_source_intact() {
        use crate::execution_control_experiment::{
            CancellationToken, ControlFailure, InvocationControl, WorkDomain, WorkLimits,
        };
        let tree = document("<out>A<!--ignored--> <z/></out>");
        let out = element(&tree, "out");
        let original = tree.children(out).to_vec();
        let mut measured = InvocationControl::unbounded();
        strict_stylesheet_view(&tree, out, &mut measured).unwrap();
        let required = measured.consumed(WorkDomain::XdmNode);
        let mut limits = WorkLimits::unbounded();
        limits.xdm_nodes = required;
        strict_stylesheet_view(
            &tree,
            out,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        )
        .unwrap();
        limits.xdm_nodes -= 1;
        assert!(matches!(
            strict_stylesheet_view(
                &tree,
                out,
                &mut InvocationControl::new(CancellationToken::new(), limits)
            ),
            Err(StrictPreprocessingFailure::Control(
                ControlFailure::BudgetExhausted {
                    domain: WorkDomain::XdmNode,
                    ..
                }
            ))
        ));
        let token = CancellationToken::new();
        token.cancel();
        assert!(matches!(
            strict_stylesheet_view(
                &tree,
                out,
                &mut InvocationControl::new(token, WorkLimits::unbounded())
            ),
            Err(StrictPreprocessingFailure::Control(
                ControlFailure::Cancelled {
                    domain: WorkDomain::XdmNode
                }
            ))
        ));
        assert_eq!(tree.children(out), original);
    }

    #[test]
    fn modern_view_forced_rules_override_space_without_changing_strict_rules() {
        use crate::execution_control_experiment::InvocationControl;
        for name in [
            "accumulator",
            "analyze-string",
            "apply-imports",
            "apply-templates",
            "attribute-set",
            "call-template",
            "character-map",
            "choose",
            "evaluate",
            "fork",
            "merge",
            "merge-source",
            "mode",
            "next-iteration",
            "next-match",
            "override",
            "package",
            "stylesheet",
            "transform",
            "use-package",
        ] {
            let tree = document(&format!(
                r#"<xsl:{name} xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve"> <!--ignored--> </xsl:{name}>"#
            ));
            let root = element(&tree, name);
            let original = tree.children(root).to_vec();
            let modern = stylesheet_visibility_view(
                &tree,
                root,
                StylesheetPreprocessingProfile::Modern,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            let strict =
                strict_stylesheet_view(&tree, root, &mut InvocationControl::unbounded()).unwrap();
            assert!(modern.children(root).is_empty(), "{name}");
            assert_eq!(strict.children(root).len(), 2, "{name}");
            assert_eq!(tree.children(root), original);
            assert!(modern.same_origin(&tree));
            assert!(modern.shares_node_storage_with(&tree));
        }
    }

    #[test]
    fn modern_view_sibling_rules_use_merged_runs_and_expanded_names() {
        use crate::execution_control_experiment::InvocationControl;
        let tree = document(
            r#"<out xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve"> <!--ignored--> <xsl:param/> <?ignored yes?> <xsl:sort/> <xsl:context-item/> <xsl:on-completion/><xsl:catch/> <!--ignored--> <keep/> <param/>A<!--ignored--> <xsl:param/><xsl:text> </xsl:text></out>"#,
        );
        let root = element(&tree, "out");
        let modern = stylesheet_visibility_view(
            &tree,
            root,
            StylesheetPreprocessingProfile::Modern,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        let values: String = modern
            .children(root)
            .iter()
            .filter_map(|node| modern.value(*node))
            .collect();
        assert_eq!(values, " A ");
        let text = element(&tree, "text");
        assert_eq!(modern.children(text), tree.children(text));
        let strict =
            strict_stylesheet_view(&tree, root, &mut InvocationControl::unbounded()).unwrap();
        assert!(strict.children(root).len() > modern.children(root).len());
        for child in modern.children(root) {
            assert_eq!(modern.location(*child), tree.location(*child));
        }
    }

    #[test]
    fn modern_empty_content_validation_uses_preprocessed_content() {
        use crate::execution_control_experiment::InvocationControl;
        for name in [
            "include",
            "import",
            "output",
            "strip-space",
            "preserve-space",
            "decimal-format",
            "namespace-alias",
            "output-character",
            "mode",
            "context-item",
            "copy-of",
            "number",
        ] {
            for content in [
                " <!--ignored--> <?ignored yes?>",
                "A<!--ignored--> ",
                "<child/>",
            ] {
                let tree = document(&format!(
                    r#"<xsl:{name} xmlns:xsl="http://www.w3.org/1999/XSL/Transform">{content}</xsl:{name}>"#
                ));
                let root = element(&tree, name);
                let original = tree.children(root).to_vec();
                let result = stylesheet_visibility_view(
                    &tree,
                    root,
                    StylesheetPreprocessingProfile::Modern,
                    &mut InvocationControl::unbounded(),
                );
                if content.starts_with(' ') {
                    assert!(result.unwrap().children(root).is_empty(), "{name}");
                } else {
                    let StrictPreprocessingFailure::Compile(failure) = result.unwrap_err() else {
                        panic!("expected static error for {name}");
                    };
                    assert_eq!(failure.code, "XTSE0260", "{name}");
                    assert_eq!(failure.category, super::super::CompileCategory::Invalid);
                    assert_eq!(failure.location, *tree.location(root));
                }
                assert_eq!(tree.children(root), original);
            }
        }
    }

    #[test]
    fn modern_empty_content_preservation_and_expanded_name_boundaries() {
        use crate::execution_control_experiment::InvocationControl;
        for xml in [
            r#"<xsl:output xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve"> <!--ignored--> </xsl:output>"#,
            r#"<root xml:space="preserve" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output> </xsl:output></root>"#,
        ] {
            let tree = document(xml);
            let root = element(&tree, "output");
            assert!(matches!(
                stylesheet_visibility_view(
                    &tree,
                    root,
                    StylesheetPreprocessingProfile::Modern,
                    &mut InvocationControl::unbounded()
                ),
                Err(StrictPreprocessingFailure::Compile(CompileFailure {
                    code: "XTSE0260",
                    ..
                }))
            ));
            // The strict reference does not borrow a modern static-error rule.
            strict_stylesheet_view(&tree, root, &mut InvocationControl::unbounded()).unwrap();
        }
        for xml in [
            r#"<output xml:space="preserve"> </output>"#,
            r#"<xsl:output xmlns:xsl="urn:not-xslt" xml:space="preserve"> </xsl:output>"#,
            r#"<root xml:space="preserve" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output xml:space="default"> </xsl:output></root>"#,
        ] {
            let tree = document(xml);
            let root = element(&tree, "output");
            stylesheet_visibility_view(
                &tree,
                root,
                StylesheetPreprocessingProfile::Modern,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
        }
        // Modern xsl:key can have a sequence constructor when @use is absent;
        // existing narrow key lowering must not redefine its content model.
        let tree = document(
            r#"<xsl:key xmlns:xsl="http://www.w3.org/1999/XSL/Transform" name="k" match="r">K</xsl:key>"#,
        );
        let root = element(&tree, "key");
        let view = stylesheet_visibility_view(
            &tree,
            root,
            StylesheetPreprocessingProfile::Modern,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
        assert_eq!(view.string_value(root), "K");
    }

    #[test]
    fn modern_view_exact_budget_cancellation_and_original_tree_recovery() {
        use crate::execution_control_experiment::{
            CancellationToken, ControlFailure, InvocationControl, WorkDomain, WorkLimits,
        };
        let tree = document(
            r#"<out xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xml:space="preserve"> <!--ignored--> <xsl:param/>A<!--ignored--> </out>"#,
        );
        let root = element(&tree, "out");
        let original = tree.children(root).to_vec();
        let mut measured = InvocationControl::unbounded();
        stylesheet_visibility_view(
            &tree,
            root,
            StylesheetPreprocessingProfile::Modern,
            &mut measured,
        )
        .unwrap();
        let required = measured.consumed(WorkDomain::XdmNode);
        for limit in [required, required - 1] {
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xdm_nodes: limit,
                    ..WorkLimits::unbounded()
                },
            );
            let result = stylesheet_visibility_view(
                &tree,
                root,
                StylesheetPreprocessingProfile::Modern,
                &mut control,
            );
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
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmNode, 3);
        assert!(matches!(
            stylesheet_visibility_view(
                &tree,
                root,
                StylesheetPreprocessingProfile::Modern,
                &mut control
            ),
            Err(StrictPreprocessingFailure::Control(
                ControlFailure::Cancelled {
                    domain: WorkDomain::XdmNode
                }
            ))
        ));
        assert_eq!(tree.children(root), original);
        stylesheet_visibility_view(
            &tree,
            root,
            StylesheetPreprocessingProfile::Modern,
            &mut InvocationControl::unbounded(),
        )
        .unwrap();
    }
}
