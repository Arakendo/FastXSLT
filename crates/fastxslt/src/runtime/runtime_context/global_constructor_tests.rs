//! Production shared-executor admission, controls and recursive ownership.
use super::super::tests::{document, program};
use super::tests::with_owner;
use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkLimits};

#[path = "global_constructor_tests/comparisons.rs"]
mod comparisons;

#[path = "global_constructor_tests/string_operations.rs"]
mod string_operations;

#[path = "global_constructor_tests/source_iteration.rs"]
mod source_iteration;

#[path = "global_constructor_tests/literal_attributes.rs"]
mod literal_attributes;

#[path = "global_constructor_tests/attribute_focus.rs"]
mod attribute_focus;

#[path = "global_constructor_tests/comment_pi.rs"]
mod comment_pi;

#[path = "global_constructor_tests/source_copy.rs"]
mod source_copy;

fn value(owner: &GlobalInitialization<'_>, name: &str, control: &mut InvocationControl) -> String {
    owner.read_legacy_string(name, control).unwrap()
}

#[test]
fn shared_constructor_dispatch_reads_forward_once_with_fresh_root_focus() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:apply-templates select="//b"/></xsl:variable>
      <xsl:variable name="inner"><xsl:call-template name="build"/></xsl:variable>
      <xsl:variable name="local" select="'global'"/>
      <xsl:template match="b"><xsl:variable name="local" select="'caller'"/>
        <xsl:value-of select="$inner"/><xsl:value-of select="$inner"/>
      </xsl:template>
      <xsl:template name="build"><xsl:message>once</xsl:message>
        <xsl:value-of select="name(.)"/><xsl:value-of select="position()"/>
        <xsl:value-of select="last()"/><xsl:value-of select="r/@v"/>
        <xsl:value-of select="$local"/>
      </xsl:template>"#,
    );
    let source = document("<r v='root'><b/></r>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(
            value(&owner, "outer", &mut control),
            "11rootglobal11rootglobal"
        );
        assert_eq!(
            value(&owner, "outer", &mut control),
            "11rootglobal11rootglobal"
        );
        assert_eq!(control.messages(), &["once"]);
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_cycle_and_termination_do_not_publish_or_replay() {
    for (body, code) in [
        (r#"<xsl:value-of select="$outer"/>"#, "XTDE0640"),
        (
            r#"<xsl:message terminate="yes">stop</xsl:message>"#,
            "XTMM9000",
        ),
    ] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>
          <xsl:template name="build">partial{body}</xsl:template>"#
        ));
        let source = document("<r/>");
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::unbounded();
            let error = owner.read_legacy_string("outer", &mut control).unwrap_err();
            assert_eq!(error.code, code);
            assert_eq!(
                error.location.as_ref().unwrap().resource,
                "urn:initializer-handoff"
            );
            let messages = control.messages().to_vec();
            assert_eq!(
                owner.read_legacy_string("outer", &mut control).unwrap_err(),
                error
            );
            assert_eq!(control.messages(), messages);
            assert!(owner.completed.borrow().temporary_trees.is_empty());
            assert_eq!(owner.depth.get(), 0);
        });
    }
}

#[test]
fn shared_constructor_retention_limits_precede_publication() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>
      <xsl:template name="build"><part>payload</part></xsl:template>"#,
    );
    let source = document("<r/>");
    let measured = with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(value(&owner, "outer", &mut control), "payload");
        [
            control.consumed(WorkDomain::ResultNode),
            control.consumed(WorkDomain::ResultTextByte),
        ]
    });
    for (dimension, count) in measured.into_iter().enumerate() {
        assert!(count > 0);
        for (limit, succeeds) in [(count, true), (count - 1, false)] {
            with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
                let mut limits = WorkLimits::unbounded();
                if dimension == 0 {
                    limits.result_nodes = limit;
                } else {
                    limits.result_text_bytes = limit;
                }
                let mut control = InvocationControl::new(CancellationToken::new(), limits);
                assert_eq!(
                    owner.read_legacy_string("outer", &mut control).is_ok(),
                    succeeds
                );
                assert_eq!(
                    owner
                        .completed
                        .borrow()
                        .temporary_trees
                        .contains_key("outer"),
                    succeeds
                );
                assert_eq!(owner.depth.get(), 0);
            });
        }
    }
}

#[test]
fn shared_constructor_exact_instruction_budget_and_cancellation_cleanup() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>
      <xsl:template name="build">payload</xsl:template>"#,
    );
    let source = document("<r/>");
    let count = with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(value(&owner, "outer", &mut control), "payload");
        control.consumed(WorkDomain::XsltInstruction)
    });
    assert!(count > 0);
    for (limit, succeeds) in [(count, true), (count - 1, false)] {
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xslt_instructions: limit,
                    ..WorkLimits::unbounded()
                },
            );
            assert_eq!(
                owner.read_legacy_string("outer", &mut control).is_ok(),
                succeeds
            );
            assert_eq!(owner.depth.get(), 0);
            assert_eq!(
                owner
                    .completed
                    .borrow()
                    .temporary_trees
                    .contains_key("outer"),
                succeeds
            );
        });
    }
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XsltInstruction, 1);
        assert_eq!(
            owner
                .read_legacy_string("outer", &mut control)
                .unwrap_err()
                .code,
            "FXCT0001"
        );
        assert!(owner.completed.borrow().temporary_trees.is_empty());
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_supplied_parameter_skips_default_and_unsupported_stays_explicit() {
    let program = program(
        r#"
      <xsl:param name="outer"><xsl:call-template name="build"/></xsl:param>
      <xsl:template name="build"><xsl:copy-of select="ancestor-or-self::*"/></xsl:template>"#,
    );
    let source = document("<r/>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        assert_eq!(
            owner
                .read_legacy_string("outer", &mut InvocationControl::unbounded())
                .unwrap_err()
                .category,
            FailureCategory::Unsupported
        );
    });
    let supplied = BTreeMap::from([(
        "outer".into(),
        InvocationParameter {
            value: InvocationParameterValue::Atomic(
                crate::xdm::atomic_value_experiment::AtomicValue::string("host"),
            ),
            tunnel: false,
        },
    )]);
    with_owner(&program, None, &supplied, |owner| {
        assert_eq!(
            value(&owner, "outer", &mut InvocationControl::unbounded()),
            "host"
        );
    });
}

#[test]
fn shared_constructor_preserves_multiple_match_policy() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:apply-templates select="//b"/></xsl:variable>
      <xsl:template match="b">first</xsl:template>
      <xsl:template match="b">last</xsl:template>"#,
    );
    let source = document("<r><b/></r>");
    let parameters = BTreeMap::new();
    let dynamic = RefCell::new(BTreeMap::new());
    for policy in [
        crate::runtime::golden_runtime_experiment::MultipleMatchPolicy::UseLast,
        crate::runtime::golden_runtime_experiment::MultipleMatchPolicy::Error,
    ] {
        let documents = GlobalDocumentContext {
            multiple_match_policy: policy,
            resource_snapshot: None,
            denied_resources: None,
            dynamic_documents: &dynamic,
        };
        let owner =
            GlobalInitialization::new(&program, Some(&source), &parameters, "policy", &documents);
        let result = owner.read_legacy_string("outer", &mut InvocationControl::unbounded());
        if policy == crate::runtime::golden_runtime_experiment::MultipleMatchPolicy::UseLast {
            assert_eq!(result.unwrap(), "last");
        } else {
            assert!(result.is_err());
            assert!(owner.completed.borrow().temporary_trees.is_empty());
        }
    }
}

#[test]
fn shared_constructor_rejects_unwired_match_and_parameter_consumers() {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    for template in [
        r#"<xsl:template match="b"><xsl:param name="p" select="$later"/><xsl:value-of select="$p"/></xsl:template>"#,
        r#"<xsl:template match="b[@v]">unproved</xsl:template>"#,
    ] {
        let stylesheet = document(&format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:variable name="outer"><xsl:apply-templates select="//b"/></xsl:variable>
          <xsl:variable name="later" select="'right'"/>{template}
        </xsl:stylesheet>"#
        ));
        assert_eq!(
            compile_stylesheet(&stylesheet).unwrap_err().code,
            "FXST1015"
        );
    }
}

#[test]
fn shared_constructor_keeps_modern_and_module_graph_admission_closed() {
    use crate::compile::golden_stylesheet_experiment::{
        compile_stylesheet, compile_stylesheet_with_single_include,
    };
    let declarations = r#"<xsl:variable name="outer"><xsl:call-template name="build"/></xsl:variable>
        <xsl:template name="build">payload</xsl:template>"#;
    let modern = document(&format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">{declarations}</xsl:stylesheet>"#
    ));
    assert_eq!(compile_stylesheet(&modern).unwrap_err().code, "FXST1015");
    let included = document(&format!(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">{declarations}</xsl:stylesheet>"#
    ));
    let principal = document(
        r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:include href="module"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#,
    );
    let root = included.children(included.document_node())[0];
    assert_eq!(
        compile_stylesheet_with_single_include(&principal, &included, root)
            .unwrap_err()
            .code,
        "FXST1015"
    );
}

#[test]
fn shared_constructor_secondary_document_read_is_not_an_unbound_variable() {
    let program = program("");
    with_owner(&program, None, &BTreeMap::new(), |owner| {
        owner.completed.borrow_mut().literal_documents.insert(
            "secondary".into(),
            crate::runtime::golden_runtime_experiment::dynamic_document::DynamicDocument {
                identity: 7,
                document: std::sync::Arc::new(document("<r>secondary</r>")),
            },
        );
        let error = owner
            .read_legacy_string("secondary", &mut InvocationControl::unbounded())
            .unwrap_err();
        assert_eq!(error.category, FailureCategory::Unsupported);
        assert_eq!(error.code, "FXRT1004");
    });
}

#[test]
fn shared_constructor_conditional_dependency_is_not_a_predicted_cycle() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="choose"/></xsl:variable>
      <xsl:variable name="inner"><xsl:call-template name="read-outer"/></xsl:variable>
      <xsl:template name="choose"><xsl:choose>
        <xsl:when test="r/@cycle = 'yes'"><xsl:value-of select="$inner"/></xsl:when>
        <xsl:otherwise>safe</xsl:otherwise>
      </xsl:choose></xsl:template>
      <xsl:template name="read-outer"><xsl:message>inner</xsl:message><xsl:value-of select="$outer"/></xsl:template>"#,
    );
    for cycle in [false, true] {
        let source = document(if cycle { "<r cycle='yes'/>" } else { "<r/>" });
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::unbounded();
            let result = owner.read_legacy_string("outer", &mut control);
            if cycle {
                assert_eq!(result.unwrap_err().code, "XTDE0640");
                assert!(owner.completed.borrow().temporary_trees.is_empty());
            } else {
                assert_eq!(result.unwrap(), "safe");
                assert!(!owner.is_complete("inner"));
                assert!(control.messages().is_empty());
                // Eager initialization is unchanged: the unused constructor
                // runs later, after outer has completed, without a cycle.
                owner.initialize_all(&mut control).unwrap();
                assert_eq!(control.messages(), &["inner"]);
            }
        });
    }
}

#[test]
fn shared_constructor_source_condition_preserves_modes_and_untaken_bodies() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="choose"/></xsl:variable>
      <xsl:template name="choose">
        <xsl:if test="r/@enabled = 'yes' and not(r/@blocked = 'yes')"><xsl:apply-templates select="r/b" mode="selected"/></xsl:if>
        <xsl:if test="r/@never = 'yes'"><xsl:copy-of select="/"/></xsl:if>
      </xsl:template>
      <xsl:template match="b" mode="selected"><xsl:value-of select="."/></xsl:template>
      <xsl:template match="b">wrong-mode</xsl:template>"#,
    );
    for (xml, expected) in [
        ("<r enabled='yes'><b>selected</b></r>", "selected"),
        ("<r enabled='yes' blocked='yes'><b>selected</b></r>", ""),
        ("<r><b>selected</b></r>", ""),
    ] {
        let source = document(xml);
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            assert_eq!(
                value(&owner, "outer", &mut InvocationControl::unbounded()),
                expected
            );
        });
    }
}

#[test]
fn shared_constructor_conditional_work_and_failure_cleanup_are_bounded() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="choose"/></xsl:variable>
      <xsl:template name="choose"><xsl:if test="r/@enabled = 'yes' or r/@other = 'yes'">payload</xsl:if></xsl:template>"#,
    );
    let source = document("<r enabled='yes'/>");
    let consumed = with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(value(&owner, "outer", &mut control), "payload");
        control.consumed(WorkDomain::XPathNodeVisit)
    });
    assert!(consumed > 0);
    for (limit, succeeds) in [(consumed, true), (consumed - 1, false)] {
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xpath_node_visits: limit,
                    ..WorkLimits::unbounded()
                },
            );
            assert_eq!(
                owner.read_legacy_string("outer", &mut control).is_ok(),
                succeeds
            );
            assert_eq!(owner.is_complete("outer"), succeeds);
            assert_eq!(owner.depth.get(), 0);
        });
    }
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 1);
        let error = owner.read_legacy_string("outer", &mut control).unwrap_err();
        assert_eq!(error.code, "FXCT0001");
        assert_eq!(
            owner.read_legacy_string("outer", &mut control).unwrap_err(),
            error
        );
        assert_eq!(owner.depth.get(), 0);
        assert!(!owner.is_complete("outer"));
    });
}

#[test]
fn shared_constructor_conditional_invocations_do_not_share_source_or_values() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="choose"/></xsl:variable>
      <xsl:template name="choose"><xsl:choose>
        <xsl:when test="r/@enabled = 'yes'"><xsl:value-of select="r/@value"/></xsl:when>
        <xsl:otherwise>disabled</xsl:otherwise>
      </xsl:choose></xsl:template>"#,
    );
    std::thread::scope(|scope| {
        let handles = (0..8)
            .map(|index| {
                let program = &program;
                scope.spawn(move || {
                    let source = document(&format!("<r enabled='yes' value='{index}'/>"));
                    with_owner(program, Some(&source), &BTreeMap::new(), |owner| {
                        value(&owner, "outer", &mut InvocationControl::unbounded())
                    })
                })
            })
            .collect::<Vec<_>>();
        for (index, handle) in handles.into_iter().enumerate() {
            assert_eq!(handle.join().unwrap(), index.to_string());
        }
    });
}

#[test]
fn shared_constructor_conditions_match_ordinary_execution_in_both_profiles() {
    use crate::compile::golden_stylesheet_experiment::{
        compile_stylesheet, stylesheet_preprocessing::strict_stylesheet_view,
    };
    use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_for_test;
    let source = document("<r enabled='yes'/>");
    for strict in [false, true] {
        let mut results = Vec::new();
        for global in [false, true] {
            let declaration = if global {
                r#"<xsl:variable name="outer"><xsl:call-template name="choose"/></xsl:variable>"#
            } else {
                ""
            };
            let entry = if global {
                r#"<xsl:value-of select="$outer"/>"#
            } else {
                r#"<xsl:call-template name="choose"/>"#
            };
            let stylesheet = document(&format!(
                r#"
              <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
                <xsl:output method="text"/>{declaration}
                <xsl:template match="/">{entry}</xsl:template>
                <xsl:template name="choose"><xsl:choose>
                  <xsl:when test="r/@enabled = 'yes'">A<!--split-->   <!--split-->B</xsl:when>
                  <xsl:otherwise>disabled</xsl:otherwise>
                </xsl:choose></xsl:template>
              </xsl:stylesheet>"#
            ));
            let root = stylesheet
                .children(stylesheet.document_node())
                .iter()
                .copied()
                .find(|node| stylesheet.name(*node).is_some())
                .unwrap();
            let effective;
            let stylesheet = if strict {
                effective =
                    strict_stylesheet_view(&stylesheet, root, &mut InvocationControl::unbounded())
                        .unwrap();
                &effective
            } else {
                &stylesheet
            };
            let compiled = compile_stylesheet(stylesheet).unwrap();
            results.push(
                execute_compiled_stylesheet_for_test(&compiled, &source, "conditional-parity")
                    .unwrap(),
            );
        }
        assert_eq!(results[0], results[1]);
        assert_eq!(results[0], if strict { "AB" } else { "A   B" });
    }
}

#[test]
fn shared_constructor_condition_uses_effective_source_and_result_outlives_program() {
    use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
    use crate::runtime::golden_runtime_experiment::{execute_program, serialize_xml};
    for strip in [false, true] {
        let stylesheet = document(&format!(
            r#"
          <xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
            {}<xsl:output method="text"/>
            <xsl:variable name="outer"><xsl:call-template name="choose"/></xsl:variable>
            <xsl:template name="choose"><xsl:choose>
              <xsl:when test="r = 'x'">stripped</xsl:when>
              <xsl:otherwise>preserved</xsl:otherwise>
            </xsl:choose></xsl:template>
            <xsl:template match="/"><xsl:value-of select="$outer"/></xsl:template>
          </xsl:stylesheet>"#,
            if strip {
                r#"<xsl:strip-space elements="*"/>"#
            } else {
                ""
            }
        ));
        let compiled = compile_stylesheet(&stylesheet).unwrap();
        let source = document("<r>  <b>x</b> </r>");
        let mut control = InvocationControl::unbounded();
        let result = execute_program(&compiled, &source, "effective", &mut control).unwrap();
        let output = compiled.output.clone();
        drop(compiled);
        drop(source);
        drop(stylesheet);
        let actual = serialize_xml(&result, &output, "effective", 1024, &mut control).unwrap();
        assert_eq!(actual, if strip { "stripped" } else { "preserved" });
    }
}

#[test]
fn shared_constructor_global_boolean_preserves_value_kind() {
    for (declaration, expected) in [
        (r#"<xsl:variable name="flag" select="false()"/>"#, "no"),
        (r#"<xsl:variable name="flag" select="0"/>"#, "no"),
        (r#"<xsl:variable name="flag" select="''"/>"#, "no"),
        (r#"<xsl:variable name="flag" select="'false'"/>"#, "yes"),
        (r#"<xsl:variable name="flag" select="/r/absent"/>"#, "no"),
        (r#"<xsl:variable name="flag" select="/r/empty"/>"#, "yes"),
        (
            r#"<xsl:variable name="flag"><xsl:call-template name="empty"/></xsl:variable>"#,
            "yes",
        ),
    ] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="choose"/></xsl:variable>
          {declaration}
          <xsl:template name="empty"/>
          <xsl:template name="choose"><xsl:choose>
            <xsl:when test="$flag">yes</xsl:when><xsl:otherwise>no</xsl:otherwise>
          </xsl:choose></xsl:template>"#
        ));
        let source = document("<r><empty/></r>");
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::unbounded();
            assert_eq!(
                value(&owner, "outer", &mut control),
                expected,
                "{declaration}"
            );
            let globals = owner.initialize_all(&mut control).unwrap();
            assert_eq!(
                super::super::super::completed_global_boolean(&globals, "flag", "test").unwrap(),
                Some(expected == "yes")
            );
        });
    }
}

#[test]
fn shared_constructor_global_boolean_short_circuits_without_skipping_eager_initialization() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="choose"/></xsl:variable>
      <xsl:variable name="flag"><xsl:call-template name="build"/></xsl:variable>
      <xsl:template name="choose"><xsl:if test="false() and $flag">wrong</xsl:if>
        <xsl:if test="true() or $flag">safe</xsl:if></xsl:template>
      <xsl:template name="build"><xsl:message>once</xsl:message><xsl:value-of select="$outer"/></xsl:template>"#,
    );
    let source = document("<r/>");
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(value(&owner, "outer", &mut control), "safe");
        assert!(!owner.is_complete("flag"));
        assert!(control.messages().is_empty());
        owner.initialize_all(&mut control).unwrap();
        assert_eq!(control.messages(), &["once"]);
    });
}

#[test]
fn shared_constructor_global_boolean_cycle_failure_and_local_shadowing() {
    for shadow in [false, true] {
        let program = program(&format!(
            r#"
          <xsl:variable name="outer"><xsl:call-template name="choose"/></xsl:variable>
          <xsl:template name="choose">{}
            <xsl:if test="$outer">wrong</xsl:if>safe
          </xsl:template>"#,
            if shadow {
                r#"<xsl:variable name="outer" select="''"/>"#
            } else {
                ""
            }
        ));
        let source = document("<r/>");
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::unbounded();
            let result = owner.read_legacy_string("outer", &mut control);
            if shadow {
                assert!(result.unwrap().contains("safe"));
            } else {
                let error = result.unwrap_err();
                assert_eq!(error.code, "XTDE0640");
                assert_eq!(
                    owner.read_legacy_string("outer", &mut control).unwrap_err(),
                    error
                );
                assert!(!owner.is_complete("outer"));
            }
            assert_eq!(owner.depth.get(), 0);
        });
    }
}

#[test]
fn shared_constructor_global_boolean_exact_work_and_cancel_cleanup() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="choose"/></xsl:variable>
      <xsl:variable name="flag"><xsl:call-template name="build"/></xsl:variable>
      <xsl:template name="choose"><xsl:if test="$flag">yes</xsl:if></xsl:template>
      <xsl:template name="build"/>"#,
    );
    let source = document("<r/>");
    let count = with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(value(&owner, "outer", &mut control), "yes");
        control.consumed(WorkDomain::XPathOperation)
    });
    assert!(count > 0);
    for (limit, succeeds) in [(count, true), (count - 1, false)] {
        with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
            let mut control = InvocationControl::new(
                CancellationToken::new(),
                WorkLimits {
                    xpath_operations: limit,
                    ..WorkLimits::unbounded()
                },
            );
            assert_eq!(
                owner.read_legacy_string("outer", &mut control).is_ok(),
                succeeds
            );
            assert_eq!(owner.is_complete("outer"), succeeds);
            assert_eq!(owner.depth.get(), 0);
        });
    }
    with_owner(&program, Some(&source), &BTreeMap::new(), |owner| {
        let mut control =
            InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 1);
        let error = owner.read_legacy_string("outer", &mut control).unwrap_err();
        assert_eq!(error.code, "FXCT0001");
        assert_eq!(
            owner.read_legacy_string("outer", &mut control).unwrap_err(),
            error
        );
        assert!(!owner.is_complete("outer"));
        assert_eq!(owner.depth.get(), 0);
    });
}

#[test]
fn shared_constructor_global_boolean_parameter_override_suppresses_default() {
    let program = program(
        r#"
      <xsl:variable name="outer"><xsl:call-template name="choose"/></xsl:variable>
      <xsl:param name="flag"><xsl:call-template name="fail"/></xsl:param>
      <xsl:template name="choose"><xsl:if test="$flag">yes</xsl:if></xsl:template>
      <xsl:template name="fail"><xsl:message terminate="yes">default</xsl:message></xsl:template>"#,
    );
    let source = document("<r/>");
    let supplied = BTreeMap::from([(
        "flag".into(),
        InvocationParameter {
            value: InvocationParameterValue::Atomic(
                crate::xdm::atomic_value_experiment::AtomicValue::string("supplied"),
            ),
            tunnel: false,
        },
    )]);
    with_owner(&program, Some(&source), &supplied, |owner| {
        let mut control = InvocationControl::unbounded();
        assert_eq!(value(&owner, "outer", &mut control), "yes");
        owner.initialize_all(&mut control).unwrap();
        assert!(control.messages().is_empty());
    });
}
