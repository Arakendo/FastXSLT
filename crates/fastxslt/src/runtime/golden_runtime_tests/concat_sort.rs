//! Legacy concat sort keys use the shared lexical evaluator, not result construction.

use super::{
    BTreeMap, CancellationToken, Document, InvocationControl, MultipleMatchPolicy, ParseLimits,
    ResourceLimits, ResourceSetBuilder, WhitespaceRepresentation, WorkDomain, WorkLimits,
    compile_resource, execute_program_with_parameters_using, parse_document, serialize_xml,
};
use crate::xslt::golden_semantics_experiment::StylesheetProgram;

fn compile(select: &str) -> StylesheetProgram {
    compile_output(select, "@id")
}

fn compile_output(select: &str, output: &str) -> StylesheetProgram {
    compile_options(select, output, "'key:'")
}

fn compile_options(select: &str, output: &str, prefix: &str) -> StylesheetProgram {
    let stylesheet = format!(
        r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:strip-space elements="*"/><xsl:key name="names" match="item" use="@id"/><xsl:template match="/"><xsl:variable name="prefix" select="{prefix}"/><xsl:for-each select="r/item"><xsl:sort select="{select}"/><xsl:value-of select="{output}"/></xsl:for-each></xsl:template></xsl:stylesheet>"#
    );
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
    resources
        .admit("urn:concat-sort:style", stylesheet.into_bytes())
        .unwrap();
    compile_resource(&resources.seal(), "urn:concat-sort:style").unwrap()
}

fn run(
    program: &StylesheetProgram,
    control: &mut InvocationControl,
    representation: WhitespaceRepresentation,
) -> Result<String, super::super::ExecutionFailure> {
    run_source(program, b"<r> <item id='Z'><name>Zulu</name></item><item id='A'><name>Alpha</name><name>ignored</name></item><item id='B'><name>Alpha</name></item><item id='E'/> </r>", control, representation)
}

fn run_source(
    program: &StylesheetProgram,
    bytes: &[u8],
    control: &mut InvocationControl,
    representation: WhitespaceRepresentation,
) -> Result<String, super::super::ExecutionFailure> {
    let source = Document::from_parsed(
        parse_document(
            "urn:concat-sort:source",
            bytes,
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
        "concat-sort",
        representation,
        None,
        None,
        control,
    )?;
    serialize_xml(&result, &program.output, "concat-sort", 8192, control)
}

#[test]
fn concat_sort_preserves_first_node_conversion_empty_keys_stability_and_views() {
    let program = compile("concat($prefix,name,':')");
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        assert_eq!(
            run(
                &program,
                &mut InvocationControl::unbounded(),
                representation
            )
            .unwrap(),
            "EABZ"
        );
    }
}

#[test]
fn concat_sort_does_not_charge_internal_keys_as_result_text() {
    let mut literal = InvocationControl::unbounded();
    let mut concat = InvocationControl::unbounded();
    let expected = run(
        &compile("'same'"),
        &mut literal,
        WhitespaceRepresentation::VisibilityView,
    )
    .unwrap();
    assert_eq!(
        run(
            &compile("concat('sa','me')"),
            &mut concat,
            WhitespaceRepresentation::VisibilityView
        )
        .unwrap(),
        expected
    );
    assert_eq!(
        concat.consumed(WorkDomain::ResultTextByte),
        literal.consumed(WorkDomain::ResultTextByte)
    );
}

#[test]
fn concat_sort_work_limits_cancellation_and_failure_leave_program_reusable() {
    let program = compile("concat($prefix,name,':')");
    let mut measured = InvocationControl::unbounded();
    assert_eq!(
        run(
            &program,
            &mut measured,
            WhitespaceRepresentation::VisibilityView
        )
        .unwrap(),
        "EABZ"
    );
    let charge = measured.consumed(WorkDomain::XPathOperation);
    for allowance in [charge - 1, charge] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = allowance;
        let result = run(
            &program,
            &mut InvocationControl::new(CancellationToken::new(), limits),
            WhitespaceRepresentation::VisibilityView,
        );
        if allowance == charge {
            assert_eq!(result.unwrap(), "EABZ");
        } else {
            assert_eq!(result.unwrap_err().code, "FXCT0002");
        }
    }
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 2);
    assert_eq!(
        run(
            &program,
            &mut cancelled,
            WhitespaceRepresentation::VisibilityView
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    let failure = run(
        &compile("concat($missing,name)"),
        &mut InvocationControl::unbounded(),
        WhitespaceRepresentation::VisibilityView,
    )
    .unwrap_err();
    assert_eq!(failure.code, "FXRT0002");
    assert!(failure.location.is_some());
    assert_eq!(
        run(
            &program,
            &mut InvocationControl::unbounded(),
            WhitespaceRepresentation::VisibilityView
        )
        .unwrap(),
        "EABZ"
    );
}

#[test]
fn concat_sort_does_not_reinterpret_secondary_document_ids() {
    const STYLE: &str = "urn:concat-sort:style";
    const SOURCE: &str = "urn:concat-sort:source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(3, 8192, 16384));
    resources.admit(STYLE, br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:for-each select="document('urn:concat-sort:extra')/r/item"><xsl:sort select="concat('key:',@id)"/><xsl:value-of select="@id"/></xsl:for-each></xsl:template></xsl:stylesheet>"#.to_vec()).unwrap();
    resources.admit(SOURCE, b"<principal/>".to_vec()).unwrap();
    resources
        .admit(
            "urn:concat-sort:extra",
            b"<r><item id='B'/><item id='A'/></r>".to_vec(),
        )
        .unwrap();
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLE).unwrap();
    let mut builder = super::TransformSetBuilder::new(snapshot, program, 1, super::policy(8192));
    builder
        .add(super::request("concat-sort-secondary", "result", SOURCE))
        .unwrap();
    let failure = super::execute_transform_set(builder.seal()).unwrap_err();
    assert_eq!(failure.code, "FXRT0007");
    assert_eq!(failure.category, super::FailureCategory::Unsupported);
    assert_eq!(failure.location.unwrap().resource, STYLE);
}

#[test]
fn concat_substring_shares_output_conversion_rounding_and_sort_controls() {
    let reference = compile_output("'same'", "substring(name,1.5,2.5)");
    let composed = compile_output("'same'", "concat(substring(name,1.5,2.5),'')");
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        let mut reference_control = InvocationControl::unbounded();
        let mut composed_control = InvocationControl::unbounded();
        assert_eq!(
            run(&reference, &mut reference_control, representation).unwrap(),
            "ululphlph"
        );
        assert_eq!(
            run(&composed, &mut composed_control, representation).unwrap(),
            "ululphlph"
        );
        assert_eq!(
            reference_control.consumed(WorkDomain::ResultTextByte),
            composed_control.consumed(WorkDomain::ResultTextByte)
        );
    }
    let program = compile("concat(substring(name,1,1),':')");
    let mut measured = InvocationControl::unbounded();
    assert_eq!(
        run(
            &program,
            &mut measured,
            WhitespaceRepresentation::VisibilityView
        )
        .unwrap(),
        "EABZ"
    );
    let charged = measured.consumed(WorkDomain::XPathOperation);
    for allowance in [charged - 1, charged] {
        let mut limits = WorkLimits::unbounded();
        limits.xpath_operations = allowance;
        let result = run(
            &program,
            &mut InvocationControl::new(CancellationToken::new(), limits),
            WhitespaceRepresentation::VisibilityView,
        );
        if allowance == charged {
            assert_eq!(result.unwrap(), "EABZ");
        } else {
            assert_eq!(result.unwrap_err().code, "FXCT0002");
        }
    }
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XPathOperation, 3);
    assert_eq!(
        run(
            &program,
            &mut cancelled,
            WhitespaceRepresentation::VisibilityView
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    assert_eq!(
        run(
            &program,
            &mut InvocationControl::unbounded(),
            WhitespaceRepresentation::VisibilityView
        )
        .unwrap(),
        "EABZ"
    );
}

#[test]
fn concat_substring_keeps_zero_length_empty_and_two_argument_behavior() {
    for (expression, expected) in [
        ("concat(substring(name,1,0),'|')", "||||"),
        ("concat(substring(name,2),'|')", "ulu|lpha|lpha||"),
        ("concat(substring(missing,-2,5),'|')", "||||"),
    ] {
        assert_eq!(
            run(
                &compile_output("'same'", expression),
                &mut InvocationControl::unbounded(),
                WhitespaceRepresentation::VisibilityView
            )
            .unwrap(),
            expected
        );
    }
}

#[test]
fn direct_substring_sort_charges_the_same_string_traversal_as_concat() {
    let direct = compile("substring(name,1,1)");
    let composed = compile("concat(substring(name,1,1),'')");
    let mut direct_control = InvocationControl::unbounded();
    let mut composed_control = InvocationControl::unbounded();
    assert_eq!(
        run(
            &direct,
            &mut direct_control,
            WhitespaceRepresentation::VisibilityView
        )
        .unwrap(),
        "EABZ"
    );
    assert_eq!(
        run(
            &composed,
            &mut composed_control,
            WhitespaceRepresentation::VisibilityView
        )
        .unwrap(),
        "EABZ"
    );
    assert_eq!(
        direct_control.consumed(WorkDomain::XdmStringValueNode),
        composed_control.consumed(WorkDomain::XdmStringValueNode)
    );
    let charge = direct_control.consumed(WorkDomain::XdmStringValueNode);
    for allowance in [charge - 1, charge] {
        let mut limits = WorkLimits::unbounded();
        limits.xdm_string_value_nodes = allowance;
        let result = run(
            &direct,
            &mut InvocationControl::new(CancellationToken::new(), limits),
            WhitespaceRepresentation::VisibilityView,
        );
        if allowance == charge {
            assert_eq!(result.unwrap(), "EABZ");
        } else {
            let failure = result.unwrap_err();
            assert_eq!(failure.code, "FXCT0002");
            assert_eq!(failure.work_domain, Some(WorkDomain::XdmStringValueNode));
        }
    }
    let mut cancelled =
        InvocationControl::unbounded().cancelling_on_charge(WorkDomain::XdmStringValueNode, 1);
    assert_eq!(
        run(
            &direct,
            &mut cancelled,
            WhitespaceRepresentation::VisibilityView
        )
        .unwrap_err()
        .code,
        "FXCT0001"
    );
    assert_eq!(
        run(
            &direct,
            &mut InvocationControl::unbounded(),
            WhitespaceRepresentation::CompleteReference
        )
        .unwrap(),
        "EABZ"
    );
}

#[test]
fn all_selected_source_sort_strings_charge_traversal_and_recover() {
    for (select, prefix, expected) in [
        ("name", "1", "EABZ"),
        ("name|missing", "1", "EABZ"),
        ("*[$prefix]", "1", "EABZ"),
        ("*[position()=$prefix]", "1", "EABZ"),
        ("*[name(.)=$prefix]", "'name'", "EABZ"),
        ("substring-after(name,'l')", "1", "EABZ"),
        ("key('names',@id)", "1", "EBAZ"),
    ] {
        let program = compile_options(select, "@id", prefix);
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            let mut measured = InvocationControl::unbounded();
            assert_eq!(
                run(&program, &mut measured, representation).unwrap(),
                expected,
                "{select}"
            );
            let visits = measured.consumed(WorkDomain::XdmStringValueNode);
            assert!(visits > 4, "{select}: only output traversal was charged");
            for allowance in [visits - 1, visits] {
                let mut limits = WorkLimits::unbounded();
                limits.xdm_string_value_nodes = allowance;
                let result = run(
                    &program,
                    &mut InvocationControl::new(CancellationToken::new(), limits),
                    representation,
                );
                if allowance == visits {
                    assert_eq!(result.unwrap(), expected);
                } else {
                    let failure = result.unwrap_err();
                    assert_eq!(failure.code, "FXCT0002");
                    assert_eq!(failure.work_domain, Some(WorkDomain::XdmStringValueNode));
                }
            }
            let mut cancelled = InvocationControl::unbounded()
                .cancelling_on_charge(WorkDomain::XdmStringValueNode, 1);
            assert_eq!(
                run(&program, &mut cancelled, representation)
                    .unwrap_err()
                    .code,
                "FXCT0001"
            );
            assert_eq!(
                run(
                    &program,
                    &mut InvocationControl::unbounded(),
                    representation
                )
                .unwrap(),
                expected
            );
        }
    }
}

#[test]
fn composed_substrings_share_output_and_concat_semantics() {
    for (expression, expected) in [
        (
            "substring(name,1,string-length(name))",
            "Zulu|Alpha|Alpha||",
        ),
        ("substring(substring-after(name,'l'),1,2)", "u|ph|ph||"),
    ] {
        let output = format!("concat({expression},'|')");
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            assert_eq!(
                run(
                    &compile_output("'same'", &output),
                    &mut InvocationControl::unbounded(),
                    representation
                )
                .unwrap(),
                expected
            );
        }
    }
    let dynamic = "substring(name,1,number(starts-with(name,'Al'))*string-length(name))";
    let direct = compile_output("'same'", dynamic);
    let composed = compile_output("'same'", &format!("concat({dynamic},'')"));
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        assert_eq!(
            run(&direct, &mut InvocationControl::unbounded(), representation).unwrap(),
            "AlphaAlpha"
        );
        assert_eq!(
            run(
                &composed,
                &mut InvocationControl::unbounded(),
                representation
            )
            .unwrap(),
            "AlphaAlpha"
        );
    }
}

#[test]
fn composed_substring_sort_is_controlled_and_reusable() {
    let dynamic = "substring(name,1,number(not(starts-with(name,'Z')))*string-length(name))";
    let program = compile(&format!("concat({dynamic},'')"));
    for representation in [
        WhitespaceRepresentation::VisibilityView,
        WhitespaceRepresentation::CompleteReference,
    ] {
        let mut measured = InvocationControl::unbounded();
        assert_eq!(
            run(&program, &mut measured, representation).unwrap(),
            "ZEAB"
        );
        for domain in [WorkDomain::XPathOperation, WorkDomain::XdmStringValueNode] {
            let visits = measured.consumed(domain);
            for allowance in [visits - 1, visits] {
                let mut limits = WorkLimits::unbounded();
                match domain {
                    WorkDomain::XPathOperation => limits.xpath_operations = allowance,
                    WorkDomain::XdmStringValueNode => limits.xdm_string_value_nodes = allowance,
                    _ => unreachable!(),
                }
                let result = run(
                    &program,
                    &mut InvocationControl::new(CancellationToken::new(), limits),
                    representation,
                );
                if allowance == visits {
                    assert_eq!(result.unwrap(), "ZEAB");
                } else {
                    let failure = result.unwrap_err();
                    assert_eq!(failure.code, "FXCT0002");
                    assert_eq!(failure.work_domain, Some(domain));
                }
            }
            let mut cancelled = InvocationControl::unbounded().cancelling_on_charge(domain, 1);
            assert_eq!(
                run(&program, &mut cancelled, representation)
                    .unwrap_err()
                    .code,
                "FXCT0001"
            );
        }
        assert_eq!(
            run(
                &program,
                &mut InvocationControl::unbounded(),
                representation
            )
            .unwrap(),
            "ZEAB"
        );
    }
}

#[test]
fn composed_substring_counts_unicode_characters_and_preserves_empty_delimiters() {
    let source = "<r><item><name>é🙂界</name><name>ignored</name></item><item/></r>";
    for (expression, expected) in [
        (
            "concat(substring(name,2,string-length(name)),'|')",
            "🙂界||",
        ),
        (
            "concat(substring(substring-after(name,''),1,2),'|')",
            "é🙂||",
        ),
        (
            "concat(substring(name,2.5,string-length(name)),'|')",
            "界||",
        ),
        (
            "concat(substring(name,1,number(not(starts-with(name,'é')))*string-length(name)),'|')",
            "||",
        ),
    ] {
        let program = compile_output("'same'", expression);
        for representation in [
            WhitespaceRepresentation::VisibilityView,
            WhitespaceRepresentation::CompleteReference,
        ] {
            assert_eq!(
                run_source(
                    &program,
                    source.as_bytes(),
                    &mut InvocationControl::unbounded(),
                    representation
                )
                .unwrap(),
                expected
            );
        }
    }
}

#[test]
fn shared_path_starts_with_preserves_conditions_and_non_source_rejection() {
    for (select, predicate, expected) in [
        ("r/item", "starts-with(name,'Al')", Ok("AB")),
        ("r/item", "not(starts-with(name,'Al'))", Ok("ZE")),
        (
            "r/item/namespace::*",
            "starts-with(.,'urn:')",
            Err("FXST1007"),
        ),
    ] {
        let stylesheet = format!(
            r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:for-each select="{select}"><xsl:if test="{predicate}"><xsl:value-of select="@id"/></xsl:if></xsl:for-each></xsl:template></xsl:stylesheet>"#
        );
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(1, 8192, 8192));
        resources
            .admit("urn:concat-sort:style", stylesheet.into_bytes())
            .unwrap();
        let compiled = compile_resource(&resources.seal(), "urn:concat-sort:style");
        if let Err(code) = expected {
            assert_eq!(compiled.unwrap_err().code, code);
            continue;
        }
        let program = compiled.unwrap();
        let result = run(
            &program,
            &mut InvocationControl::unbounded(),
            WhitespaceRepresentation::VisibilityView,
        );
        match expected {
            Ok(expected) => assert_eq!(result.unwrap(), expected),
            Err(code) => assert_eq!(result.unwrap_err().code, code),
        }
    }
}
