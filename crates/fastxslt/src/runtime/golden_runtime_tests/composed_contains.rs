//! Composed string conditions reuse value conversion without a second evaluator.

use super::{
    BTreeMap, CancellationToken, Document, InvocationControl, MultipleMatchPolicy, ParseLimits,
    ResourceLimits, ResourceSetBuilder, WhitespaceRepresentation, WorkDomain, WorkLimits,
    compile_resource, execute_program_with_parameters_using, parse_document, serialize_xml,
};

fn compile(
    body: &str,
    version: &str,
) -> Result<
    crate::xslt::golden_semantics_experiment::StylesheetProgram,
    super::super::ExecutionFailure,
> {
    let stylesheet = format!(
        r#"<xsl:stylesheet version="{version}" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:strip-space elements="*"/><xsl:template match="/"><xsl:variable name="haystack" select="'é+abc,+z'"/><xsl:variable name="needle" select="'abc,'"/>{body}</xsl:template></xsl:stylesheet>"#
    );
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources
        .admit("urn:contains:style", stylesheet.into_bytes())
        .unwrap();
    compile_resource(&resources.seal(), "urn:contains:style")
}

fn run(
    program: &crate::xslt::golden_semantics_experiment::StylesheetProgram,
    control: &mut InvocationControl,
) -> Result<String, super::super::ExecutionFailure> {
    run_using(program, control, WhitespaceRepresentation::VisibilityView)
}

fn run_using(
    program: &crate::xslt::golden_semantics_experiment::StylesheetProgram,
    control: &mut InvocationControl,
    representation: WhitespaceRepresentation,
) -> Result<String, super::super::ExecutionFailure> {
    let source = Document::from_parsed(
        parse_document(
            "urn:contains:source",
            b"<r> <n>abc,</n><n>other</n> </r>",
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .unwrap(),
    )
    .unwrap();
    let result = execute_program_with_parameters_using(
        program,
        &source,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        "composed-contains",
        representation,
        None,
        None,
        control,
    )?;
    serialize_xml(&result, &program.output, "composed-contains", 8192, control)
}

#[test]
fn generated_source_and_temporary_root_strings_are_ascii_alphanumeric() {
    let program = compile(
        r#"<xsl:variable name="tree"><leaf/></xsl:variable><xsl:value-of select="generate-id(/)"/><xsl:text>|</xsl:text><xsl:value-of select="generate-id(root($tree))"/><xsl:text>|</xsl:text><xsl:value-of select="generate-id(root($tree))"/>"#,
        "3.0",
    )
    .unwrap();
    let actual = run(&program, &mut InvocationControl::unbounded()).unwrap();
    let fields = actual.split('|').collect::<Vec<_>>();
    assert_eq!(fields.len(), 3);
    assert_ne!(fields[0], fields[1]);
    assert_eq!(fields[1], fields[2]);
    for identity in fields {
        assert!(identity.as_bytes()[0].is_ascii_alphabetic());
        assert!(identity.bytes().all(|byte| byte.is_ascii_alphanumeric()));
    }
}

#[test]
fn composed_contains_generated_ids_keep_source_identity_and_legacy_first_node_conversion() {
    let body = r#"<xsl:value-of select="concat('+',generate-id(/),'+')"/><xsl:text>|</xsl:text><xsl:value-of select="generate-id(/)"/><xsl:text>|</xsl:text><xsl:value-of select="concat('empty=',generate-id(r/missing))"/><xsl:text>|</xsl:text><xsl:value-of select="contains(concat('+',generate-id(r/n),'+'),concat('+',generate-id(r/n[1]),'+'))"/><xsl:text>|</xsl:text><xsl:value-of select="contains(concat('+',generate-id(/),'+'),concat('+',generate-id(r),'+'))"/><xsl:text>|</xsl:text><xsl:for-each select="r/n[1]"><xsl:value-of select="concat(generate-id(),':',generate-id(.))"/></xsl:for-each>"#;
    let program = compile(body, "1.0").unwrap();
    let actual = run(&program, &mut InvocationControl::unbounded()).unwrap();
    let fields = actual.split('|').collect::<Vec<_>>();
    assert_eq!(fields.len(), 6);
    assert_eq!(fields[0], format!("+{}+", fields[1]));
    assert_eq!(fields[2..5], ["empty=", "true", "false"]);
    let (implicit, explicit) = fields[5].split_once(':').unwrap();
    assert_eq!(implicit, explicit);
    assert_ne!(implicit, fields[1]);
    for identity in [fields[1], implicit] {
        assert!(identity.as_bytes()[0].is_ascii_alphabetic());
        assert!(identity.bytes().all(|byte| byte.is_ascii_alphanumeric()));
    }
    assert_eq!(
        run_using(
            &program,
            &mut InvocationControl::unbounded(),
            WhitespaceRepresentation::CompleteReference,
        )
        .unwrap(),
        actual,
    );
    assert_eq!(
        compile(body, "3.0").unwrap_err().category,
        super::FailureCategory::Unsupported
    );
    let namespace = compile(
        r#"<xsl:value-of select="concat('+',generate-id(r/namespace::*),'+')"/>"#,
        "1.0",
    )
    .unwrap();
    let value = run(&namespace, &mut InvocationControl::unbounded()).unwrap();
    assert!(value.starts_with("+fastxsltPrincipalNs"));
    assert!(value.ends_with('+'));
}

#[test]
fn composed_contains_generated_ids_preserve_control_and_short_circuiting() {
    let program = compile(r#"<xsl:if test="false() and contains(concat('+',generate-id(r/n),'+'),'+')"><xsl:text>bad</xsl:text></xsl:if><xsl:value-of select="concat('+',generate-id(/),'+')"/>"#, "1.0").unwrap();
    let mut measured = InvocationControl::unbounded();
    let expected = run(&program, &mut measured).unwrap();
    let charge = measured.consumed(WorkDomain::XPathOperation);
    let plain = compile(r#"<xsl:if test="false() and contains(concat('x','y'),'+')"><xsl:text>bad</xsl:text></xsl:if><xsl:value-of select="concat('+',generate-id(/),'+')"/>"#, "1.0").unwrap();
    let mut plain_control = InvocationControl::unbounded();
    assert_eq!(run(&plain, &mut plain_control).unwrap(), expected);
    assert_eq!(charge, plain_control.consumed(WorkDomain::XPathOperation));
    for allowance in [charge - 1, charge] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = allowance;
        let actual = run(
            &program,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        );
        if allowance == charge {
            assert_eq!(actual.unwrap(), expected);
        } else {
            let failure = actual.unwrap_err();
            assert_eq!(failure.code, "FXCT0002");
            assert_eq!(failure.request_id.as_deref(), Some("composed-contains"));
        }
    }
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 2);
    assert_eq!(run(&program, &mut cancelled).unwrap_err().code, "FXCT0001");
    assert_eq!(
        run(&program, &mut InvocationControl::unbounded()).unwrap(),
        expected
    );
}

#[test]
fn composed_contains_conditions_convert_variables_literals_and_paths() {
    let program = compile(r#"<xsl:if test="contains($haystack,concat('+',$needle,'+'))"><xsl:text>A</xsl:text></xsl:if><xsl:if test="contains(concat('é',r/n),$needle)"><xsl:text>B</xsl:text></xsl:if><xsl:if test="contains($haystack,'')"><xsl:text>C</xsl:text></xsl:if><xsl:choose><xsl:when test="not(contains($haystack,'missing')) and contains($haystack,'é')"><xsl:text>D</xsl:text></xsl:when><xsl:otherwise><xsl:text>bad</xsl:text></xsl:otherwise></xsl:choose><xsl:value-of select="contains(concat('é',r/n),concat('abc',','))"/>"#, "1.0").unwrap();
    assert_eq!(
        run(&program, &mut InvocationControl::unbounded()).unwrap(),
        "ABCDtrue"
    );
}

#[test]
fn composed_contains_short_circuits_and_does_not_admit_modern_conversion() {
    let program = compile(r#"<xsl:if test="true() or contains($missing,concat('a','b'))"><xsl:text>yes</xsl:text></xsl:if><xsl:if test="false() and contains($missing,concat('a','b'))"><xsl:text>bad</xsl:text></xsl:if>"#, "1.0").unwrap();
    assert_eq!(
        run(&program, &mut InvocationControl::unbounded()).unwrap(),
        "yes"
    );
    let failure = compile(r#"<xsl:if test="contains($haystack,concat('+',$needle,'+'))"><xsl:text>bad</xsl:text></xsl:if>"#, "3.0").unwrap_err();
    assert_eq!(failure.category, super::FailureCategory::Unsupported);
    let program = compile(
        r#"<xsl:if test="contains($missing,concat('a','b'))"><xsl:text>bad</xsl:text></xsl:if>"#,
        "1.0",
    )
    .unwrap();
    let failure = run(&program, &mut InvocationControl::unbounded()).unwrap_err();
    assert_eq!(failure.category, super::FailureCategory::Invalid);
    assert_eq!(failure.request_id.as_deref(), Some("composed-contains"));
}

#[test]
fn composed_contains_control_failure_leaves_compiled_state_reusable() {
    let program = compile(r#"<xsl:if test="contains(concat('é',r/n),concat('abc',','))"><xsl:text>yes</xsl:text></xsl:if>"#, "1.0").unwrap();
    let mut measured = InvocationControl::unbounded();
    assert_eq!(run(&program, &mut measured).unwrap(), "yes");
    let charge = measured.consumed(WorkDomain::XPathOperation);
    for allowance in [charge - 1, charge] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = allowance;
        let result = run(
            &program,
            &mut InvocationControl::new(CancellationToken::new(), limits),
        );
        if allowance == charge {
            assert_eq!(result.unwrap(), "yes");
        } else {
            assert_eq!(result.unwrap_err().code, "FXCT0002");
        }
    }
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 2);
    assert_eq!(run(&program, &mut cancelled).unwrap_err().code, "FXCT0001");
    assert_eq!(
        run(&program, &mut InvocationControl::unbounded()).unwrap(),
        "yes"
    );
}
