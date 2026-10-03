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
        WhitespaceRepresentation::VisibilityView,
        None,
        None,
        control,
    )?;
    serialize_xml(&result, &program.output, "composed-contains", 8192, control)
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
