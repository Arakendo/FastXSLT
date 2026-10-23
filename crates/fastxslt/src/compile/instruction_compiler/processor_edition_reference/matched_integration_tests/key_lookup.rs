use super::*;
use crate::execution_control_experiment::{CancellationToken, WorkDomain, WorkLimits};
use crate::runtime::golden_runtime_experiment::execute_compiled_stylesheet_controlled_for_test as execute;
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

fn stylesheet(version: &str) -> Document {
    document("urn:selected-key", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="{version}"><xsl:output method="text"/><xsl:key name="id" match="item" use="@id"/><xsl:key name="ann" match="ann" use="@of"/><xsl:template match="r"><xsl:apply-templates select="node"/></xsl:template><xsl:template match="node"><xsl:variable name="id" select="@id"/><xsl:value-of select="count(key('id',$id))"/><xsl:text>:</xsl:text><xsl:apply-templates select="key('id',$id)"/><xsl:text>|</xsl:text></xsl:template><xsl:template match="item"><xsl:choose><xsl:when test="key('ann',@id)">Y</xsl:when><xsl:otherwise>N</xsl:otherwise></xsl:choose></xsl:template></xsl:stylesheet>"#).as_bytes())
}

fn check_limits(program: &StylesheetProgram, source: &Document, required: usize, expected: &str) {
    let mut limits = WorkLimits::unbounded();
    limits.xpath_node_visits = required;
    assert_eq!(
        execute(
            program,
            source,
            "exact",
            &mut InvocationControl::new(CancellationToken::new(), limits)
        )
        .unwrap(),
        expected
    );
    limits.xpath_node_visits -= 1;
    assert!(
        execute(
            program,
            source,
            "short",
            &mut InvocationControl::new(CancellationToken::new(), limits)
        )
        .unwrap_err()
        .contains("FXCT0002")
    );
    let mut cancel = InvocationControl::unbounded()
        .cancelling_on_charge(WorkDomain::XPathNodeVisit, required / 2);
    assert!(
        execute(program, source, "cancel", &mut cancel)
            .unwrap_err()
            .contains("FXCT0001")
    );
    assert_eq!(
        execute(
            program,
            source,
            "reuse",
            &mut InvocationControl::unbounded()
        )
        .unwrap(),
        expected
    );
}

#[test]
fn selected_keys_preserve_lookup_dispatch_controls_and_source_isolation() {
    let source = document("urn:same-source", b"<r><definition><item id='a'/><item id='a'/><item id='b'/><item id=''/></definition><ann of='a'/><node id='a'/><node id='b'/><node id='missing'/><node id=''/><node/></r>");
    let changed = document("urn:same-source", b"<r><item id='a'/><node id='a'/></r>");
    let expected = "2:YY|1:N|0:|1:N|0:|";
    let reference = compiler::compile_stylesheet(&stylesheet("1.0")).unwrap();
    let mut baseline = InvocationControl::unbounded();
    assert_eq!(
        execute(&reference, &source, "reference", &mut baseline).unwrap(),
        expected
    );
    for version in ["1.0", "2.0", "3.0"] {
        let style = stylesheet(version);
        let selected =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).unwrap();
        drop(style);
        let mut actual = InvocationControl::unbounded();
        assert_eq!(
            execute(&selected, &source, "selected", &mut actual).unwrap(),
            expected
        );
        for domain in [
            WorkDomain::XPathNodeVisit,
            WorkDomain::XPathOperation,
            WorkDomain::XsltInstruction,
            WorkDomain::XsltTemplateCandidate,
            WorkDomain::ResultTextByte,
        ] {
            assert_eq!(actual.consumed(domain), baseline.consumed(domain));
        }
        check_limits(
            &selected,
            &source,
            actual.consumed(WorkDomain::XPathNodeVisit),
            expected,
        );
        assert_eq!(
            execute(
                &selected,
                &changed,
                "changed",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            "1:N|"
        );
        assert_eq!(
            execute(
                &selected,
                &source,
                "original",
                &mut InvocationControl::unbounded()
            )
            .unwrap(),
            expected
        );
    }
}

#[test]
fn selected_keys_keep_unbound_names_and_general_shapes_explicit() {
    for body in [
        r#"<xsl:value-of select="count(key('id',$missing))"/>"#,
        r#"<xsl:if test="key('id',$missing)">bad</xsl:if>"#,
        r#"<xsl:apply-templates select="key('id',$missing)"/>"#,
    ] {
        let style = document("urn:unbound-key", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="/">{body}</xsl:template></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(error)) =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("unbound key variable")
        };
        assert_eq!(error.code, "XPST0008");
        assert_eq!(error.location.resource, "urn:unbound-key");
    }
    for declaration in [
        r#"<xsl:key name="id" match="item[@id]" use="@id"/>"#,
        r#"<xsl:key name="id" match="item" use="concat(@id,'x')"/>"#,
    ] {
        let style = document("urn:key-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0">{declaration}<xsl:template match="/"/></xsl:stylesheet>"#).as_bytes());
        let Err(StrictPreprocessingFailure::Compile(error)) =
            compile(&style, Edition::Legacy, &mut InvocationControl::unbounded())
        else {
            panic!("guarded declaration")
        };
        assert_eq!(error.code, "FXAR0031");
    }
    assert!(
        compile(
            &stylesheet("3.0"),
            Edition::Modern,
            &mut InvocationControl::unbounded()
        )
        .is_err()
    );
}

#[test]
fn selected_key_dispatch_keeps_modes_arguments_and_sorts_guarded() {
    for instruction in [
        r#"<xsl:apply-templates select="key('id',@id)" mode="other"/>"#,
        r#"<xsl:apply-templates select="key('id',@id)"><xsl:sort/></xsl:apply-templates>"#,
        r#"<xsl:apply-templates select="key('id',@id)"><xsl:with-param name="p" select="'x'"/></xsl:apply-templates>"#,
    ] {
        let style = document("urn:key-dispatch-guard", format!(r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"><xsl:template match="/">{instruction}</xsl:template></xsl:stylesheet>"#).as_bytes());
        assert!(compile(&style, Edition::Legacy, &mut InvocationControl::unbounded()).is_err());
    }
}
