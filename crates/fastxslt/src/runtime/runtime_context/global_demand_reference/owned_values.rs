//! Owned demand-value experiment using existing temporary/source semantic owners.

use std::sync::Arc;

use super::super::super::retained_source_values::RetainedSourceSelection;
use super::{
    Document, ExecutionFailure, InvocationControl, TemporaryTree, control_failure,
    temporary_tree_string_value,
};
use crate::xpath::path_experiment::{LocationPath, evaluate_location_path_controlled};

#[derive(Debug)]
pub(super) enum OwnedValue {
    Temporary(Arc<TemporaryTree>),
    Source(RetainedSourceSelection),
}

impl OwnedValue {
    pub(super) fn select_source(
        owner: &Arc<Document>,
        path: &LocationPath,
        control: &mut InvocationControl,
    ) -> Result<Self, ExecutionFailure> {
        let nodes = evaluate_location_path_controlled(owner, owner.document_node(), path, control)
            .map_err(|error| control_failure(error, "demand-shared-reference"))?;
        Ok(Self::Source(RetainedSourceSelection {
            owner: Arc::clone(owner),
            nodes,
        }))
    }

    pub(super) fn string_value(
        &self,
        control: &mut InvocationControl,
    ) -> Result<String, ExecutionFailure> {
        match self {
            Self::Temporary(tree) => {
                temporary_tree_string_value(tree, "demand-shared-reference", control)
            }
            Self::Source(selection) => selection.nodes.first().map_or_else(
                || Ok(String::new()),
                |node| {
                    selection
                        .owner
                        .string_value_controlled(*node, control)
                        .map_err(|error| control_failure(error, "demand-shared-reference"))
                },
            ),
        }
    }
}

#[cfg(test)]
#[path = "owned_values/comparisons.rs"]
mod comparisons;

#[cfg(test)]
mod tests {
    use super::super::{Driver, RuntimeVariables};
    use super::*;
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::xdm::qualified_nodes::QualifiedSourceNode;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
    use std::collections::BTreeMap;

    pub(super) fn document(xml: &str) -> Arc<Document> {
        Arc::new(
            Document::from_parsed(
                parse_document(
                    "urn:owned-demand",
                    xml.as_bytes(),
                    ParseLimits {
                        max_events: 512,
                        max_depth: 32,
                    },
                )
                .unwrap(),
            )
            .unwrap(),
        )
    }

    pub(super) fn driver(
        source: Arc<Document>,
        body: &str,
        control: &mut InvocationControl,
    ) -> std::rc::Rc<Driver> {
        let stylesheet = document(&format!(
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
            <xsl:variable name="source" select="/r/b"/>
            <xsl:variable name="first" select="'placeholder'"/>
            <xsl:variable name="second" select="'placeholder'"/>
            <xsl:variable name="kept" select="'placeholder'"/>
            {body}
          </xsl:stylesheet>"#
        ));
        Driver::new(
            Arc::new(compile_stylesheet(&stylesheet).unwrap()),
            source,
            &BTreeMap::new(),
            false,
            control,
        )
        .unwrap()
    }

    #[test]
    fn demand_owned_tree_borrow_survives_recursive_reads_and_owner_teardown() {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document("<r><b>source</b></r>"),
            r#"
          <xsl:template match="/"><xsl:value-of select="$second"/></xsl:template>
          <xsl:template name="first_constructor"><box><xsl:text>first</xsl:text></box></xsl:template>
          <xsl:template name="second_constructor"><xsl:value-of select="$first"/><xsl:text>:</xsl:text><xsl:value-of select="$source"/></xsl:template>"#,
            &mut control,
        );
        let first = driver.read_value("first", &mut control).unwrap().unwrap();
        let OwnedValue::Temporary(tree) = first.as_ref() else {
            panic!("temporary value");
        };
        let identity = tree.identity;
        let second = driver.read_value("second", &mut control).unwrap().unwrap();
        assert_eq!(second.string_value(&mut control).unwrap(), "first:source");
        assert_eq!(tree.identity, identity);
        assert!(Arc::ptr_eq(
            &first,
            &driver.read_value("first", &mut control).unwrap().unwrap()
        ));
        assert_eq!(driver.evaluations("first"), 1);
        drop(driver);
        assert_eq!(
            temporary_tree_string_value(tree, "after-drop", &mut control).unwrap(),
            "first"
        );
    }

    #[test]
    fn demand_owned_source_keeps_identity_during_recursive_evaluation_and_focus_switch() {
        let mut control = InvocationControl::unbounded();
        let original = document("<r><b>original</b></r>");
        let foreign = document("<r><b>foreign</b></r>");
        let weak = Arc::downgrade(&original);
        let driver = driver(
            original,
            r#"
          <xsl:template match="/"><xsl:value-of select="$kept"/></xsl:template>
          <xsl:template name="first_constructor"><xsl:value-of select="$source"/></xsl:template>"#,
            &mut control,
        );
        let value = driver.read_value("source", &mut control).unwrap().unwrap();
        let OwnedValue::Source(selection) = value.as_ref() else {
            panic!("source value");
        };
        let id = selection.nodes[0];
        let foreign_id = foreign.children(foreign.children(foreign.document_node())[0])[0];
        assert_eq!(id, foreign_id);
        assert!(
            !QualifiedSourceNode::Tree {
                document: &selection.owner,
                id
            }
            .same_node(QualifiedSourceNode::Tree {
                document: &foreign,
                id: foreign_id
            })
        );
        assert_eq!(
            driver
                .read_string("first", &mut control)
                .unwrap()
                .as_deref(),
            Some("original")
        );
        let mut frame = RuntimeVariables::default();
        control
            .charge(
                crate::execution_control_experiment::WorkDomain::XdmNode,
                selection.nodes.len(),
            )
            .unwrap();
        frame.bind_retained_source_nodes(
            "kept".into(),
            Arc::clone(&selection.owner),
            selection.nodes.clone(),
        );
        for complete in [false, true] {
            let frame = if complete {
                frame.clone_complete_for_sequence()
            } else {
                frame.clone()
            };
            let result = driver
                .execute_with_frame(
                    &driver.program.root_template.as_ref().unwrap().body,
                    &frame,
                    &foreign,
                    &mut control,
                )
                .unwrap();
            assert_eq!(
                temporary_tree_string_value(&result, "foreign-focus", &mut control).unwrap(),
                "original"
            );
        }
        drop(driver);
        drop(frame);
        assert!(weak.upgrade().is_some());
        assert_eq!(selection.owner.string_value(id), "original");
        drop(value);
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn demand_owned_source_copy_owns_result_and_foreign_consumer_stays_guarded() {
        use super::super::super::super::TemporaryNodeKind;
        let mut control = InvocationControl::unbounded();
        let original = document("<r xmlns:p='urn:source'><b p:flag='x'>original</b></r>");
        let foreign = document("<r><b>foreign</b></r>");
        let weak = Arc::downgrade(&original);
        let driver = driver(
            original,
            r#"
          <xsl:template match="/"><xsl:copy-of select="$kept"/></xsl:template>"#,
            &mut control,
        );
        let value = driver.read_value("source", &mut control).unwrap().unwrap();
        let OwnedValue::Source(selection) = value.as_ref() else {
            panic!("source value");
        };
        let mut frame = RuntimeVariables::default();
        control
            .charge(
                crate::execution_control_experiment::WorkDomain::XdmNode,
                selection.nodes.len(),
            )
            .unwrap();
        frame.bind_retained_source_nodes(
            "kept".into(),
            Arc::clone(&selection.owner),
            selection.nodes.clone(),
        );
        let rejected = driver
            .execute_with_frame(
                &driver.program.root_template.as_ref().unwrap().body,
                &frame,
                &foreign,
                &mut control,
            )
            .unwrap_err();
        assert_eq!(rejected.code, "FXRT1017");
        let result = driver
            .execute_with_frame(
                &driver.program.root_template.as_ref().unwrap().body,
                &frame,
                &driver.source,
                &mut control,
            )
            .unwrap();
        drop(value);
        drop(frame);
        drop(driver);
        assert!(weak.upgrade().is_none());
        assert_eq!(
            temporary_tree_string_value(&result, "copied", &mut control).unwrap(),
            "original"
        );
        let TemporaryNodeKind::Element {
            name,
            namespaces,
            attributes,
        } = &result.nodes[result.roots[0]].kind
        else {
            panic!("copied element");
        };
        assert_eq!(name.local, "b");
        assert!(
            namespaces
                .iter()
                .any(|binding| binding.prefix.as_deref() == Some("p")
                    && binding.namespace == "urn:source")
        );
        assert_eq!(attributes.len(), 1);
    }

    #[test]
    fn demand_owned_views_preserve_identity_and_effective_navigation() {
        let mut control = InvocationControl::unbounded();
        let original = document("<r><b> <c/> </b></r>");
        let view = Arc::new(
            original
                .view_stripping_all_element_whitespace(&mut control)
                .unwrap(),
        );
        let complete = Arc::new(
            original
                .derive_stripping_all_element_whitespace(&mut control)
                .unwrap(),
        );
        let mut values = Vec::new();
        for source in [Arc::clone(&original), view, complete] {
            let driver = driver(source, r#"<xsl:template match="/"/>"#, &mut control);
            values.push(driver.read_value("source", &mut control).unwrap().unwrap());
        }
        let OwnedValue::Source(preserved) = values[0].as_ref() else {
            panic!("source");
        };
        let id = preserved.nodes[0];
        assert_eq!(preserved.owner.children(id).len(), 3);
        assert_eq!(values[0].string_value(&mut control).unwrap(), "  ");
        for value in &values[1..] {
            let OwnedValue::Source(stripped) = value.as_ref() else {
                panic!("source");
            };
            assert!(
                QualifiedSourceNode::Tree {
                    document: &preserved.owner,
                    id
                }
                .same_node(QualifiedSourceNode::Tree {
                    document: &stripped.owner,
                    id: stripped.nodes[0]
                })
            );
            assert_eq!(stripped.owner.children(id).len(), 1);
            assert_eq!(stripped.owner.location(id), preserved.owner.location(id));
            assert_eq!(value.string_value(&mut control).unwrap(), "");
        }
        assert_eq!(original.children(id).len(), 3);
    }

    #[test]
    fn demand_owned_failed_dependency_preserves_previously_completed_tree_borrow() {
        let mut control = InvocationControl::unbounded();
        let driver = driver(
            document("<r/>"),
            r#"
          <xsl:template match="/"/>
          <xsl:template name="first_constructor"><box>kept</box></xsl:template>
          <xsl:template name="second_constructor"><xsl:value-of select="$first"/><xsl:message terminate="yes">failed</xsl:message></xsl:template>"#,
            &mut control,
        );
        let first = driver.read_value("first", &mut control).unwrap().unwrap();
        let OwnedValue::Temporary(tree) = first.as_ref() else {
            panic!("temporary");
        };
        let identity = tree.identity;
        let error = driver.read_value("second", &mut control).unwrap_err();
        assert_eq!(error.code, "XTMM9000");
        assert_eq!(tree.identity, identity);
        assert_eq!(
            temporary_tree_string_value(tree, "kept", &mut control).unwrap(),
            "kept"
        );
        assert!(driver.resolver.stack.borrow().is_empty());
        assert_eq!(driver.evaluations("first"), 1);
        assert_eq!(
            driver.read_value("second", &mut control).unwrap_err(),
            error
        );
        assert_eq!(driver.evaluations("second"), 1);
    }

    #[test]
    fn demand_owned_source_limits_and_mid_selection_cancellation_publish_no_partial_owner() {
        use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
        let source = document("<r><b>A</b><b>B</b><b>C</b></r>");
        let body = r#"<xsl:template match="/"/>"#;
        let mut baseline = InvocationControl::unbounded();
        let reference = driver(Arc::clone(&source), body, &mut baseline);
        reference.read_value("source", &mut baseline).unwrap();
        let exact = baseline.consumed(WorkDomain::XPathNodeVisit);
        drop(reference);
        for (limit, cancel, success) in [
            (exact, false, true),
            (exact - 1, false, false),
            (exact, true, false),
        ] {
            let mut limits = WorkLimits::unbounded();
            limits.xpath_node_visits = limit;
            let mut control = InvocationControl::new(CancellationToken::new(), limits);
            if cancel {
                control = control.cancelling_on_charge(WorkDomain::XPathNodeVisit, 1);
            }
            let driver = driver(Arc::clone(&source), body, &mut control);
            let owner_count = Arc::strong_count(&source);
            let result = driver.read_value("source", &mut control);
            assert_eq!(result.is_ok(), success);
            if !success {
                assert_eq!(
                    result.unwrap_err().code,
                    if cancel { "FXCT0001" } else { "FXCT0002" }
                );
                assert_eq!(Arc::strong_count(&source), owner_count);
                assert!(
                    !driver
                        .resolver
                        .states
                        .borrow()
                        .iter()
                        .any(|state| matches!(state, super::super::super::State::Complete(_)))
                );
            }
            assert!(driver.resolver.stack.borrow().is_empty());
        }
        assert_eq!(Arc::strong_count(&source), 1);
    }
}
