//! Compiled single-owner grouped filters and retained attribute-path values.

use super::*;

fn generated_operand_fixture(
    body: &str,
    source: &[u8],
) -> (StylesheetProgram, ResourceSnapshot, Document) {
    selected_document_fixture("", body, source)
}

fn selected_document_fixture(
    globals: &str,
    body: &str,
    source: &[u8],
) -> (StylesheetProgram, ResourceSnapshot, Document) {
    selected_document_fixture_with_lookup(
        globals,
        body,
        source,
        b"<r> <n att='x'>value</n><n/><!--comment--></r>",
    )
}

fn selected_document_fixture_with_lookup(
    globals: &str,
    body: &str,
    source: &[u8],
    lookup: &[u8],
) -> (StylesheetProgram, ResourceSnapshot, Document) {
    let style = format!(
        r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:strip-space elements="*"/>{globals}<xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#
    );
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(3, 16384, 32768));
    builder.admit(STYLE, style.into_bytes()).unwrap();
    for name in ["a.xml", "b.xml"] {
        builder
            .admit(
                format!("https://example.test/bound/{name}"),
                lookup.to_vec(),
            )
            .unwrap();
    }
    let snapshot = builder.seal();
    let source = Document::from_parsed(
        parse_document(
            "https://example.test/bound/source.xml",
            source,
            ParseLimits {
                max_events: 100,
                max_depth: 16,
            },
        )
        .unwrap(),
    )
    .unwrap();
    (
        compile_resource(&snapshot, STYLE).unwrap(),
        snapshot,
        source,
    )
}

#[test]
fn global_selected_document_values_retain_effective_owner() {
    let fixture = selected_document_fixture(
        r#"<xsl:variable name="v" select="document('a.xml')/r/n"/><xsl:variable name="empty" select="document('a.xml')/missing"/>"#,
        r#"<xsl:if test="$v">nonempty|</xsl:if><xsl:if test="$empty">wrong</xsl:if><xsl:value-of select="$v"/><xsl:text>|</xsl:text><xsl:variable name="alias" select="$v"/><xsl:value-of select="$alias/@att"/><xsl:text>|</xsl:text><xsl:if test="generate-id($v) = generate-id(document('a.xml')/r/n)">same</xsl:if><xsl:for-each select="$v"><xsl:value-of select="."/></xsl:for-each>"#,
        b"<r><n att='wrong'>principal</n></r>",
    );
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        assert_eq!(
            run(
                &fixture,
                representation,
                None,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "nonempty|value|x|samevalue"
        );
    }
}

#[test]
fn current_variable_lookup_shares_sort_and_concat_owner_semantics() {
    let fixture = selected_document_fixture(
        r#"<xsl:variable name="v" select="document('a.xml')/r"/>"#,
        r#"<xsl:for-each select="r/ref"><xsl:sort select="$v/n[@att = (current())/@key]/@att"/><xsl:value-of select="concat(@key, ':', $v/n[@att=current()/@key]/@att, '|')"/></xsl:for-each>"#,
        b"<r><ref key='x'/><ref key='z'/><ref/><ref key=''/></r>",
    );
    for view in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        let mut baseline = InvocationControl::unbounded();
        assert_eq!(
            run(&fixture, view, None, &mut baseline).unwrap(),
            "z:|:|:|x:x|"
        );
        let visits = baseline.consumed(WorkDomain::XPathNodeVisit);
        for allowed in [visits - 1, visits] {
            let mut limits = WorkLimits::unbounded();
            limits.xpath_node_visits = allowed;
            let result = run(
                &fixture,
                view,
                None,
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            if allowed == visits {
                assert_eq!(result.unwrap(), "z:|:|:|x:x|");
            } else {
                assert_eq!(result.unwrap_err().code, "FXCT0002");
            }
        }
        assert_eq!(
            run(
                &fixture,
                view,
                None,
                &mut InvocationControl::unbounded()
                    .cancelling_on_charge(WorkDomain::XPathNodeVisit, 10)
            )
            .unwrap_err()
            .code,
            "FXCT0001"
        );
    }
}

#[test]
fn current_variable_lookup_preserves_empty_namespaced_and_first_result_rules() {
    let fixture = selected_document_fixture_with_lookup(
        r#"<xsl:variable name="v" select="document('a.xml')/r"/>"#,
        r#"<xsl:for-each xmlns:p="urn:p" select="r/ref"><xsl:value-of select="concat($v/p:n[@p:key=current()/@p:key]/@p:out, '|')"/></xsl:for-each>"#,
        br#"<r xmlns:q="urn:p"><ref q:key="x"/><ref q:key=""/><ref/><ref q:key="z"/></r>"#,
        br#"<r xmlns:q="urn:p"><q:n q:key="x"/><q:n q:key="x" q:out="first"/><q:n q:key="x" q:out="second"/><q:n q:key="" q:out="empty"/><q:n q:out="missing"/></r>"#,
    );
    std::thread::scope(|scope| {
        let handles = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    run(
                        &fixture,
                        WhitespaceRepresentation::VisibilityView,
                        None,
                        &mut InvocationControl::unbounded().with_complete_sequence_frame_clones(),
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            assert_eq!(handle.join().unwrap(), "first|empty|||");
        }
    });
    let temporary = selected_document_fixture(
        r#"<xsl:variable name="v" select="document('a.xml')/r"/><xsl:variable name="t"><box key="x"/></xsl:variable>"#,
        r#"<xsl:for-each select="$t/box"><xsl:value-of select="concat($v/n[@att=current()/@key]/@att, '|')"/></xsl:for-each>"#,
        b"<r key='x'/>",
    );
    assert_eq!(
        run(
            &temporary,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRT0007"
    );
}

#[test]
fn constructed_global_text_compares_as_string_in_selection_predicates() {
    let fixture = selected_document_fixture(
        r#"<xsl:variable name="group">yes</xsl:variable><xsl:variable name="empty"><xsl:text/></xsl:variable>"#,
        r#"<xsl:for-each select="r/n[@name=$group]"><xsl:value-of select="@id"/></xsl:for-each><xsl:text>|</xsl:text><xsl:for-each select="r/n[@name=$empty]"><xsl:value-of select="@id"/></xsl:for-each>"#,
        b"<r><n name='no' id='bad'/><n name='yes' id='right'/><n name='' id='empty'/><n id='missing'/></r>",
    );
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "right|empty"
    );
}

#[test]
fn generated_identity_document_operands_share_owner_and_first_node_semantics() {
    for path in [
        "/r/n",
        "/r/n/@att",
        "/r/n/text()",
        "/r/comment()",
        "/missing",
    ] {
        let body = format!(
            r#"<xsl:variable name="v" select="document('a.xml'){path}"/><xsl:if test="generate-id($v) = generate-id(document(r/ref){path})">same</xsl:if><xsl:if test="generate-id(document('a.xml'){path}) != generate-id(document('b.xml'){path})">different</xsl:if><xsl:text>|</xsl:text><xsl:value-of select="generate-id($v)"/><xsl:text>|</xsl:text><xsl:value-of select="generate-id(document('a.xml'){path})"/>"#
        );
        let fixture = generated_operand_fixture(&body, b"<r><ref>a.xml</ref><ref>a.xml</ref></r>");
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            let result = run(
                &fixture,
                representation,
                None,
                &mut InvocationControl::unbounded().with_complete_sequence_frame_clones(),
            )
            .unwrap();
            let parts = result.split('|').collect::<Vec<_>>();
            assert_eq!(
                parts[0],
                if path == "/missing" {
                    "same"
                } else {
                    "samedifferent"
                }
            );
            assert_eq!(parts[1], parts[2]);
            assert_eq!(parts[1].is_empty(), path == "/missing");
        }
    }
}

#[test]
fn global_selected_document_values_preserve_controls_and_initializer_guard() {
    let globals = r#"<xsl:variable name="v" select="document('a.xml')/r/n"/>"#;
    let fixture = selected_document_fixture(globals, r#"<xsl:value-of select="$v"/>"#, b"<r/>");
    let mut measured = InvocationControl::unbounded();
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut measured
        )
        .unwrap(),
        "value"
    );
    let visits = measured.consumed(WorkDomain::XPathNodeVisit);
    assert!(visits > 0);
    for allowed in [visits - 1, visits] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = allowed;
        let result = run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        );
        if allowed == visits {
            assert_eq!(result.unwrap(), "value");
        } else {
            assert_eq!(result.unwrap_err().code, "FXCT0002");
        }
    }
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 1)
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    let denied = HashSet::from(["https://example.test/bound/a.xml".to_owned()]);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            Some(&denied),
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRS0003"
    );
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "value"
    );
    let dependent = selected_document_fixture(
        &format!(r#"{globals}<xsl:variable name="alias" select="$v"/>"#),
        r#"<xsl:value-of select="$alias"/>"#,
        b"<r/>",
    );
    let failure = run(
        &dependent,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut InvocationControl::unbounded(),
    )
    .unwrap_err();
    assert_eq!(failure.code, "FXRT1004");
}

#[test]
fn generated_identity_document_operands_keep_multiple_owners_guarded() {
    let body = r#"<xsl:value-of select="generate-id(document(r/ref)/r/n)"/>"#;
    for (source, expected) in [
        (b"<r/>".as_slice(), None),
        (
            b"<r><ref>a.xml</ref><ref>b.xml</ref></r>".as_slice(),
            Some("FXRT0007"),
        ),
    ] {
        let fixture = generated_operand_fixture(body, source);
        let result = run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded(),
        );
        if let Some(code) = expected {
            assert_eq!(result.unwrap_err().code, code);
        } else {
            assert_eq!(result.unwrap(), "");
        }
    }
}

#[test]
fn generated_identity_document_operands_preserve_control_and_denial() {
    let fixture = generated_operand_fixture(
        r#"<xsl:variable name="v" select="document('a.xml')/r/n"/><xsl:if test="generate-id($v) = generate-id(document(r/ref)/r/n)">same</xsl:if>"#,
        b"<r><ref>a.xml</ref></r>",
    );
    let mut measured = InvocationControl::unbounded();
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut measured
        )
        .unwrap(),
        "same"
    );
    let visits = measured.consumed(WorkDomain::XPathNodeVisit);
    for allowed in [visits - 1, visits] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = allowed;
        let result = run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        );
        if allowed == visits {
            assert_eq!(result.unwrap(), "same");
        } else {
            assert_eq!(result.unwrap_err().code, "FXCT0002");
        }
    }
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
                .cancelling_on_charge(WorkDomain::XPathNodeVisit, visits - 1)
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    let denied = HashSet::from(["https://example.test/bound/a.xml".to_owned()]);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            Some(&denied),
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRS0003"
    );
}

fn identity_owner_fixture() -> (StylesheetProgram, ResourceSnapshot, Document) {
    let style = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:strip-space elements="*"/><xsl:variable name="a" select="document('a.xml')"/><xsl:variable name="b" select="document('b.xml')"/><xsl:template match="/"><xsl:apply-templates select="$a" mode="external"/><xsl:text>|</xsl:text><xsl:apply-templates select="$b" mode="external"/><xsl:text>|</xsl:text><xsl:variable name="again" select="document('a.xml')"/><xsl:apply-templates select="$again" mode="external"/><xsl:text>|</xsl:text><xsl:value-of select="generate-id(r/n)"/></xsl:template><xsl:template match="/" mode="external"><xsl:value-of select="generate-id(r/n)"/></xsl:template></xsl:stylesheet>"#;
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(3, 8192, 16384));
    builder.admit(STYLE, style.to_vec()).unwrap();
    for resource in [
        "https://example.test/bound/a.xml",
        "https://example.test/bound/b.xml",
    ] {
        builder.admit(resource, b"<r> <n/> </r>".to_vec()).unwrap();
    }
    let snapshot = builder.seal();
    let source = Document::from_parsed(
        parse_document(
            "urn:bound:principal",
            b"<r> <n/> </r>",
            ParseLimits {
                max_events: 32,
                max_depth: 8,
            },
        )
        .unwrap(),
    )
    .unwrap();
    (
        compile_resource(&snapshot, STYLE).unwrap(),
        snapshot,
        source,
    )
}

#[cfg(feature = "allocation-observation")]
#[test]
fn generated_identity_document_operands_release_after_success_and_failure() {
    let fixture = generated_operand_fixture(
        r#"<xsl:variable name="v" select="document('a.xml')/r/n"/><xsl:if test="generate-id($v) = generate-id(document(r/ref)/r/n)">same</xsl:if>"#,
        b"<r><ref>a.xml</ref></r>",
    );
    let mut measured = InvocationControl::unbounded();
    run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut measured,
    )
    .unwrap();
    let late_charge = measured.consumed(WorkDomain::XPathNodeVisit) - 1;
    for cancelled in [false, true] {
        let allocation = allocation_counter::measure(|| {
            let mut control = InvocationControl::unbounded();
            if cancelled {
                control = control.cancelling_on_charge(WorkDomain::XPathNodeVisit, late_charge);
            }
            let result = run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut control,
            );
            assert_eq!(result.is_ok(), !cancelled);
        });
        assert_eq!(allocation.count_current, 0);
        assert_eq!(allocation.bytes_current, 0);
    }
}

#[test]
fn generated_identity_secondary_owners_preserve_reuse_isolation_and_controls() {
    let fixture = identity_owner_fixture();
    let mut measured = InvocationControl::unbounded();
    let expected = run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut measured,
    )
    .unwrap();
    let ids = expected.split('|').collect::<Vec<_>>();
    assert_eq!(ids[0], ids[2]);
    assert_ne!(ids[0], ids[1]);
    assert_ne!(ids[0], ids[3]);
    assert_ne!(ids[1], ids[3]);
    for id in ids {
        assert!(id.bytes().all(|byte| byte.is_ascii_alphanumeric()));
        assert!(id.starts_with(|c: char| c.is_ascii_alphabetic()));
    }
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        assert_eq!(
            run(
                &fixture,
                representation,
                None,
                &mut InvocationControl::unbounded().with_complete_sequence_frame_clones()
            )
            .unwrap(),
            expected
        );
    }
    let visits = measured.consumed(WorkDomain::XPathNodeVisit);
    for allowance in [visits, visits - 1] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = allowance;
        let result = run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        );
        if allowance == visits {
            assert_eq!(result.unwrap(), expected);
        } else {
            assert_eq!(result.unwrap_err().code, "FXCT0002");
        }
    }
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
                .cancelling_on_charge(WorkDomain::XPathNodeVisit, visits - 1)
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    std::thread::scope(|scope| {
        let handles = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    run(
                        &fixture,
                        WhitespaceRepresentation::VisibilityView,
                        None,
                        &mut InvocationControl::unbounded(),
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            assert_eq!(handle.join().unwrap(), expected);
        }
    });
}

#[test]
fn generated_identity_on_secondary_focus_never_aliases_principal_focus() {
    let fixture = fixture_with_external_body(
        true,
        r#"<xsl:value-of select="generate-id(.)"/><xsl:text>|</xsl:text><xsl:for-each select="$principal"><xsl:value-of select="generate-id(.)"/></xsl:for-each>"#,
    );
    let result = run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut InvocationControl::unbounded(),
    );
    let result = result.unwrap();
    let (secondary, principal) = result.split_once('|').unwrap();
    assert_ne!(secondary, principal);
    assert_eq!(principal, "fastxsltPrincipalN0");
}

#[test]
fn generated_identity_secondary_focus_agrees_across_legacy_string_routes() {
    let canonical = fixture_with_external_body(
        true,
        r#"<xsl:value-of select="generate-id(root(document('extra.xml')))"/><xsl:text>|</xsl:text><xsl:value-of select="generate-id(external/value)"/>"#,
    );
    let canonical = run(
        &canonical,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut InvocationControl::unbounded(),
    )
    .unwrap();
    let (root, value) = canonical.split_once('|').unwrap();
    assert_ne!(root, value);
    for (body, expected) in [
        (r#"<xsl:value-of select="generate-id()"/>"#, root.to_owned()),
        (
            r#"<xsl:value-of select="generate-id(root(.))"/>"#,
            root.to_owned(),
        ),
        (
            r#"<xsl:value-of select="concat('id:',generate-id(.))"/>"#,
            format!("id:{root}"),
        ),
        (
            r#"<out id="{generate-id()}"/>"#,
            format!("<out id=\"{root}\"></out>"),
        ),
        (
            r#"<out id="{generate-id(key('which', 'selected'))}"/>"#,
            format!("<out id=\"{value}\"></out>"),
        ),
        (
            r#"<xsl:variable name="id" select="'fastxsltPrincipalN0'"/><xsl:if test="generate-id(root(.)) = $id"><bad/></xsl:if>"#,
            String::new(),
        ),
    ] {
        let fixture = fixture_with_templates(
            true,
            body,
            r#"<xsl:key name="which" match="value" use="'selected'"/>"#,
        );
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            let actual = run(
                &fixture,
                representation,
                None,
                &mut InvocationControl::unbounded(),
            )
            .unwrap();
            assert_eq!(actual, expected, "{body}");
        }
    }
}

#[test]
fn generated_identity_principal_focus_is_valid_after_secondary_admission() {
    let fixture = fixture_with_external_body(
        true,
        r#"<xsl:for-each select="$principal"><out id="{generate-id()}"><xsl:value-of select="generate-id(.)"/><xsl:text>|</xsl:text><xsl:value-of select="concat('id:', generate-id(.))"/></out></xsl:for-each>"#,
    );
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        assert_eq!(
            run(
                &fixture,
                representation,
                None,
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "<out id=\"fastxsltPrincipalN0\">fastxsltPrincipalN0|id:fastxsltPrincipalN0</out>"
        );
    }
}

#[test]
fn literal_document_path_variable_retains_effective_owner_and_empty_selection() {
    let fixture = fixture_with_external_body(
        true,
        r#"<xsl:variable name="selected" select="document('extra.xml')//value/text()[1]"/><xsl:variable name="empty" select="document('extra.xml')/missing"/><xsl:variable name="alias" select="$selected"/><out><xsl:value-of select="$alias"/>:<xsl:value-of select="count($empty)"/><xsl:for-each select="$principal"><checked><xsl:value-of select="$selected"/>:<xsl:value-of select="external/value"/></checked></xsl:for-each></out>"#,
    );
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        for complete in [false, true] {
            let mut control = InvocationControl::unbounded();
            if complete {
                control = control.with_complete_sequence_frame_clones();
            }
            assert_eq!(
                run(&fixture, representation, None, &mut control).unwrap(),
                "<out>right:0<checked>right:wrong</checked></out>"
            );
        }
    }
}

#[test]
fn literal_document_path_variable_preserves_controls_and_denial() {
    let fixture = fixture_with_external_body(
        false,
        r#"<xsl:variable name="selected" select="document('extra.xml')/external/node()"/><out><xsl:value-of select="$selected"/></out>"#,
    );
    let mut baseline = InvocationControl::unbounded();
    let expected = run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut baseline,
    )
    .unwrap();
    assert_eq!(expected, "<out>right</out>");
    let charged = baseline.consumed(WorkDomain::XPathNodeVisit);
    assert!(charged > 0);
    for allowance in [charged, charged - 1] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_node_visits = allowance;
        let result = run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        );
        if allowance == charged {
            assert_eq!(result.unwrap(), expected);
        } else {
            assert_eq!(result.unwrap_err().code, "FXCT0002");
        }
    }
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 0)
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            Some(&HashSet::from([EXTRA.to_owned()])),
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRS0003"
    );
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
}

#[test]
fn grouped_variable_scalars_keep_the_actual_document_owner_and_lexical_scope() {
    let fixture = fixture_with_external_body(
        true,
        r#"<xsl:variable name="alias" select="$doc"/><out><xsl:value-of select="($doc)"/>:<xsl:value-of select="$alias"/>:<xsl:value-of select="($principal)"/><nested><xsl:variable name="doc" select="'shadow'"/><xsl:value-of select="( $doc )"/></nested><xsl:value-of select="($doc)"/></out>"#,
    );
    let expected = "<out>right:right:wrong<nested>shadow</nested>right</out>";
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        for complete in [false, true] {
            let mut control = InvocationControl::unbounded();
            if complete {
                control = control
                    .with_complete_sequence_frame_clones()
                    .with_complete_atomic_frame_clones();
            }
            assert_eq!(
                run(&fixture, representation, None, &mut control).unwrap(),
                expected
            );
        }
    }
    std::thread::scope(|scope| {
        let handles = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    run(
                        &fixture,
                        WhitespaceRepresentation::VisibilityView,
                        None,
                        &mut InvocationControl::unbounded(),
                    )
                    .unwrap()
                })
            })
            .collect::<Vec<_>>();
        for handle in handles {
            assert_eq!(handle.join().unwrap(), expected);
        }
    });
}

#[test]
fn grouped_document_scalar_preserves_exact_controls_and_diagnostics() {
    let fixture =
        fixture_with_external_body(false, r#"<out><xsl:value-of select="($doc)"/></out>"#);
    let mut baseline = InvocationControl::unbounded();
    let expected = run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut baseline,
    )
    .unwrap();
    assert_eq!(expected, "<out>right</out>");
    for domain in [
        WorkDomain::XdmStringValueNode,
        WorkDomain::XdmNode,
        WorkDomain::ResultTextByte,
    ] {
        let charge = baseline.consumed(domain);
        assert!(charge > 0);
        for allowance in [charge, charge - 1] {
            let mut limits = WorkLimits::unbounded();
            match domain {
                WorkDomain::XdmStringValueNode => limits.xdm_string_value_nodes = allowance,
                WorkDomain::XdmNode => limits.xdm_nodes = allowance,
                WorkDomain::ResultTextByte => limits.result_text_bytes = allowance,
                _ => unreachable!(),
            }
            let result = run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut InvocationControl::new(CancellationToken::new(), limits),
            );
            if allowance == charge {
                assert_eq!(result.unwrap(), expected);
            } else {
                assert_eq!(result.unwrap_err().code, "FXCT0002");
            }
        }
        assert_eq!(
            run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut InvocationControl::unbounded().cancelling_on_charge(domain, 0)
            )
            .unwrap_err()
            .code,
            "FXCT0001"
        );
    }
    let absent = fixture_with_external_body(false, r#"<xsl:value-of select="($missing)"/>"#);
    let denied = HashSet::from([EXTRA.to_owned()]);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            Some(&denied),
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRS0003"
    );
    assert_eq!(
        run(
            &absent,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRT0002"
    );
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
}

fn lookup_fixture(language: &str) -> (StylesheetProgram, ResourceSnapshot, Document) {
    lookup_fixture_with_select(
        language,
        "($terms/table/expand[@language=$lang]/gentext[@key=$key])[1]",
    )
}

fn lookup_fixture_with_select(
    language: &str,
    select: &str,
) -> (StylesheetProgram, ResourceSnapshot, Document) {
    let (_, _, source) = fixture(false);
    let style = format!(
        r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:strip-space elements="*"/><xsl:param name="terms" select="document('extra.xml')"/><xsl:template match="/"><xsl:variable name="lang" select="{language}"/><xsl:variable name="key" select="'title'"/><xsl:variable name="lookup" select="{select}"/><xsl:variable name="text" select="$lookup/@text"/><xsl:variable name="alias" select="$text"/><out><xsl:value-of select="$alias"/>:<xsl:value-of select="$lookup/@text"/><xsl:apply-templates select="$lookup" mode="lookup"/></out></xsl:template><xsl:template match="gentext" mode="lookup"><selected><xsl:value-of select="@text"/>:<xsl:value-of select="position()"/>/<xsl:value-of select="last()"/></selected></xsl:template></xsl:stylesheet>"#
    );
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(2, 8192, 16384));
    builder.admit(STYLE, style.into_bytes()).unwrap();
    builder.admit(EXTRA, br#"<table> <expand><gentext key="title" text="absent"/></expand><expand language=""><gentext key="title" text="empty"/></expand><expand language="fr"><gentext key="title" text="wrong"/></expand><expand language="en"><gentext key="other" text="wrong"/><gentext key="title" text="first"/><gentext key="title" text="second"/></expand><expand language="en"><gentext key="title" text="third"/></expand> </table>"#.to_vec()).unwrap();
    let snapshot = builder.seal();
    (
        compile_resource(&snapshot, STYLE).unwrap(),
        snapshot,
        source,
    )
}

#[test]
fn bound_variable_path_selects_first_across_parents_and_retains_attributes() {
    let fixture = lookup_fixture("'en'");
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        for complete in [false, true] {
            let mut control = InvocationControl::unbounded();
            if complete {
                control = control.with_complete_sequence_frame_clones();
            }
            assert_eq!(
                run(&fixture, representation, None, &mut control).unwrap(),
                "<out>first:first<selected>first:1/1</selected></out>"
            );
        }
    }
    let missing = lookup_fixture("'missing'");
    assert_eq!(
        run(
            &missing,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<out>:</out>"
    );
    let empty = lookup_fixture("''");
    assert_eq!(
        run(
            &empty,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        "<out>empty:empty<selected>empty:1/1</selected></out>"
    );
    std::thread::scope(|scope| {
        let threads = [&fixture, &missing, &fixture, &missing].map(|fixture| {
            scope.spawn(move || {
                run(
                    fixture,
                    WhitespaceRepresentation::VisibilityView,
                    None,
                    &mut InvocationControl::unbounded(),
                )
                .unwrap()
            })
        });
        for (index, thread) in threads.into_iter().enumerate() {
            assert_eq!(
                thread.join().unwrap(),
                if index % 2 == 0 {
                    "<out>first:first<selected>first:1/1</selected></out>"
                } else {
                    "<out>:</out>"
                }
            );
        }
    });
}

#[test]
fn bound_variable_path_keeps_non_string_predicates_unsupported_and_controls_exact() {
    for select in [
        "($terms/table/expand[@language=$absent]/gentext)[1]",
        "($absent/table/expand)[1]",
    ] {
        let fixture = lookup_fixture_with_select("'en'", select);
        assert_eq!(
            run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut InvocationControl::unbounded()
            )
            .unwrap_err()
            .code,
            "FXRT0002"
        );
    }
    let numeric = lookup_fixture("1");
    assert_eq!(
        run(
            &numeric,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRT1017"
    );
    let fixture = lookup_fixture("'en'");
    let mut baseline = InvocationControl::unbounded();
    run(
        &fixture,
        WhitespaceRepresentation::VisibilityView,
        None,
        &mut baseline,
    )
    .unwrap();
    let consumed = baseline.consumed(WorkDomain::XdmNode);
    for limit in [consumed - 1, consumed] {
        let mut control = InvocationControl::new(
            CancellationToken::new(),
            WorkLimits {
                xdm_nodes: limit,
                ..WorkLimits::unbounded()
            },
        );
        assert_eq!(
            run(
                &fixture,
                WhitespaceRepresentation::VisibilityView,
                None,
                &mut control
            )
            .is_ok(),
            limit == consumed
        );
    }
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathNodeVisit, 4);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            None,
            &mut cancelled
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    let denied = HashSet::from([EXTRA.to_owned()]);
    assert_eq!(
        run(
            &fixture,
            WhitespaceRepresentation::VisibilityView,
            Some(&denied),
            &mut InvocationControl::unbounded()
        )
        .unwrap_err()
        .code,
        "FXRS0003"
    );
}

#[cfg(feature = "allocation-observation")]
#[test]
fn bound_variable_path_releases_success_and_partial_failure_allocations() {
    for language in ["'en'", "1", "'missing'"] {
        let fixture = lookup_fixture(language);
        for cancelled in [false, true] {
            let info = allocation_counter::measure(|| {
                let mut control = InvocationControl::unbounded();
                if cancelled {
                    control = control.cancelling_on_charge(WorkDomain::XPathNodeVisit, 20);
                }
                let outcome = run(
                    &fixture,
                    WhitespaceRepresentation::VisibilityView,
                    None,
                    &mut control,
                );
                if !cancelled {
                    assert_eq!(outcome.is_ok(), language != "1");
                }
            });
            assert_eq!(info.bytes_current, 0);
            assert_eq!(info.count_current, 0);
        }
    }
}
