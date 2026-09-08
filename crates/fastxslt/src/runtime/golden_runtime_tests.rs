//! General runtime contract tests retained separately from execution semantics.

use std::collections::{BTreeMap, HashSet};

use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::atomic_value_experiment::AtomicValue;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ExpandedName, ParseLimits, parse_document};

use super::{
    ExecutionPolicy, FailureCategory, InvocationEntry, InvocationParameter, MultipleMatchPolicy,
    ResultAttribute, ResultNode, SemanticResult, TransformRequest, TransformSetBuilder,
    WhitespaceRepresentation, compile_resource, execute_program,
    execute_program_with_parameters_using, execute_transform_set, materialize_integer_range,
    serialize_xml, serialize_xml_bytes,
};

const SOURCE_ID: &str = "urn:fastxslt:golden:hello:source";
const STYLESHEET_ID: &str = "urn:fastxslt:golden:hello:stylesheet";
type ConfigureWorkLimits = fn(&mut WorkLimits);

fn snapshot() -> crate::resources::ResourceSnapshot {
    let mut builder = ResourceSetBuilder::new(ResourceLimits::new(8, 4_096, 8_192));
    builder
        .admit(
            SOURCE_ID,
            include_bytes!("../../../../corpus/golden/hello/input.xml").to_vec(),
        )
        .expect("admit source");
    builder
        .admit(
            STYLESHEET_ID,
            include_bytes!("../../../../corpus/golden/hello/stylesheet.xsl").to_vec(),
        )
        .expect("admit stylesheet");
    builder.seal()
}

fn request(request_id: &str, result_id: &str, source_id: &str) -> TransformRequest {
    TransformRequest {
        identity: request_id.to_owned(),
        result_identity: result_id.to_owned(),
        entry: InvocationEntry::PrincipalSource {
            resource: source_id.to_owned(),
        },
        parameters: BTreeMap::new(),
        cancellation: CancellationToken::new(),
        cancellation_fault: None,
    }
}

fn policy(serialized_byte_limit: usize) -> ExecutionPolicy {
    ExecutionPolicy {
        denied_sources: HashSet::new(),
        serialized_byte_limit,
        work_limits: WorkLimits::unbounded(),
    }
}

fn execute_with_work_limits(request_id: &str, work_limits: WorkLimits) -> super::ExecutionFailure {
    let snapshot = snapshot();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
    let mut builder = TransformSetBuilder::new(
        snapshot,
        program,
        1,
        ExecutionPolicy {
            denied_sources: HashSet::new(),
            serialized_byte_limit: 4_096,
            work_limits,
        },
    );
    builder
        .add(request(request_id, "controlled-result", SOURCE_ID))
        .expect("admit controlled request");
    execute_transform_set(builder.seal()).expect_err("work limit should stop execution")
}

#[test]
fn golden_transform_executes_through_an_unordered_identified_set() {
    let snapshot = snapshot();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
    let mut builder = TransformSetBuilder::new(snapshot, program, 4, policy(4_096));
    builder
        .add(request("request-a", "result-a.html", SOURCE_ID))
        .expect("add first request");
    builder
        .add(request("request-b", "result-b.html", SOURCE_ID))
        .expect("add second request");

    let results = execute_transform_set(builder.seal()).expect("execute set");

    assert_eq!(results.completion_order, ["request-b", "request-a"]);
    let first = &results.by_request["request-a"];
    assert_eq!(first.result_id, "result-a.html");
    assert_eq!(
        first.semantic.children,
        [ResultNode::Element {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: "message".to_owned(),
            },
            namespaces: Vec::new().into(),
            attributes: Vec::new(),
            children: vec![ResultNode::Text("Hello, FastXSLT!".to_owned())],
        }]
    );
    assert_eq!(first.serialized, "<message>Hello, FastXSLT!</message>");
    assert_eq!(
        format!("{}\n", first.serialized),
        include_str!("../../../../corpus/golden/hello/expected.xml")
    );
    assert_eq!(results.by_request["request-b"].semantic, first.semantic);
}

#[test]
fn source_node_global_paths_execute_in_for_each_without_temporary_tree_dispatch() {
    let source = parse_document(
        "memory:source-node-global.xml",
        b"<root><group><item>A</item><item>B</item></group></root>",
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:source-node-global.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:param name="all" select="//group"/>
          <xsl:template match="/">
            <out><xsl:for-each select="$all/item"><xsl:value-of select="."/></xsl:for-each></out>
          </xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 64,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("source-node global path should compile");

    let result = execute_program(
        &program,
        &source,
        "source-node-global-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("source-node global path should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "source-node-global-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(
        serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>AB</out>"
    );
}

#[test]
fn source_dependent_global_count_uses_the_principal_document_focus() {
    let source = parse_document(
        "memory:global-count.xml",
        b"<docs><item/><item/></docs>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:global-count.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:param name="children" select="count(*)"/>
          <xsl:template match="/"><out><xsl:value-of select="$children"/></out></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("global count should compile");

    let result = execute_program(
        &program,
        &source,
        "global-count-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("global count should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "global-count-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>1</out>");
}

#[test]
fn static_contains_local_variable_reuses_atomic_boolean_execution() {
    let source = parse_document(
        "memory:static-contains.xml",
        b"<doc/>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:static-contains.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:template match="/"><xsl:variable name="found" select="contains('foo','o')"/><out><xsl:if test="$found">yes</xsl:if></out></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("static contains variable should compile");

    let result = execute_program(
        &program,
        &source,
        "static-contains-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("static contains variable should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "static-contains-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>yes</out>");
}

#[test]
fn static_string_function_can_supply_a_template_parameter_default() {
    let source = parse_document(
        "memory:static-template-parameter.xml",
        b"<doc/>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:static-template-parameter.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:variable name="implicit-empty"/>
          <xsl:template match="doc"><xsl:param name="explicit-empty" select="substring-before('a','z')"/><out><xsl:value-of select="$explicit-empty=$implicit-empty"/></out></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("static template parameter default should compile");

    let result = execute_program(
        &program,
        &source,
        "static-template-parameter-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("static template parameter default should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "static-template-parameter-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>true</out>");
}

#[test]
fn xslt10_variable_pair_comparison_rejects_general_node_set_pairs() {
    let source = parse_document(
        "memory:node-set-pair.xml",
        b"<doc><left>x</left><right>x</right></doc>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:node-set-pair.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:variable name="left" select="/doc/left"/>
          <xsl:variable name="right" select="/doc/right"/>
          <xsl:template match="/"><xsl:value-of select="$left=$right"/></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("variable pair comparison should compile to an explicit runtime boundary");

    let failure = execute_program(
        &program,
        &source,
        "node-set-pair-request",
        &mut InvocationControl::unbounded(),
    )
    .expect_err("general node-set pair comparison should remain unsupported");

    assert_eq!(failure.category, FailureCategory::Unsupported);
    assert_eq!(failure.code, "FXRT1015");
}

#[test]
fn static_integral_arithmetic_local_variable_copies_as_an_atomic_value() {
    let source = parse_document(
        "memory:static-arithmetic.xml",
        b"<doc/>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:static-arithmetic.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:template match="/"><xsl:variable name="value" select="10+7"/><out><xsl:copy-of select="$value"/></out></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("static arithmetic variable should compile");

    let result = execute_program(
        &program,
        &source,
        "static-arithmetic-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("static arithmetic variable should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "static-arithmetic-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>17</out>");
}

#[test]
fn xslt10_binary_numeric_tree_composes_paths_and_a_global_variable() {
    let source = parse_document(
        "memory:variable-arithmetic.xml",
        b"<doc><n2>2</n2><n5>5</n5></doc>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:variable-arithmetic.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:variable name="offset" select="10"/>
          <xsl:template match="doc"><out><xsl:value-of select="n2+3+$offset+7+n5"/></out></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("variable-bearing arithmetic should compile");

    let result = execute_program(
        &program,
        &source,
        "variable-arithmetic-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("variable-bearing arithmetic should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "variable-arithmetic-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>27</out>");
}

#[test]
fn standalone_exact_numeric_literal_uses_the_checked_numeric_value_path() {
    let source = parse_document(
        "memory:standalone-numeric.xml",
        b"<doc/>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:standalone-numeric.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:template match="doc"><out><xsl:value-of select="9876543210"/></out></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 24,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("standalone exact numeric literal should compile");

    let result = execute_program(
        &program,
        &source,
        "standalone-numeric-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("standalone exact numeric literal should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "standalone-numeric-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>9876543210</out>");
}

#[test]
fn xslt10_value_path_navigates_from_a_source_node_parameter() {
    let source = parse_document(
        "memory:variable-path.xml",
        br#"<page><hotel><location country="US"/></hotel><hotel><location country="CA"/></hotel></page>"#,
        ParseLimits {
            max_events: 24,
            max_depth: 6,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:variable-path.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:template match="page"><xsl:call-template name="emit"><xsl:with-param name="hotels" select="hotel"/></xsl:call-template></xsl:template>
          <xsl:template name="emit"><xsl:param name="hotels"/><out><xsl:value-of select="$hotels/location/@country"/></out></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 48,
            max_depth: 10,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("source-variable path should compile");

    let result = execute_program(
        &program,
        &source,
        "variable-path-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("source-variable path should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "variable-path-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>US</out>");
}

#[test]
fn static_string_local_variable_shadows_an_outer_binding_lexically() {
    let source = parse_document(
        "memory:static-string-local.xml",
        b"<doc><child/></doc>",
        ParseLimits {
            max_events: 12,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:static-string-local.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:template match="/"><xsl:variable name="value" select="'outer'"/><out><xsl:value-of select="$value"/><xsl:apply-templates/></out></xsl:template>
          <xsl:template match="doc"><xsl:variable name="value" select="'inner'"/><xsl:value-of select="$value"/></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 48,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("static string locals should compile");

    let result = execute_program(
        &program,
        &source,
        "static-string-local-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("static string locals should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "static-string-local-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>outerinner</out>");
}

#[test]
fn local_atomic_variable_alias_preserves_the_bound_type_and_value() {
    let source = parse_document(
        "memory:atomic-alias.xml",
        b"<doc/>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:atomic-alias.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:template match="doc"><xsl:variable name="first" select="'value'"/><xsl:variable name="second" select="$first"/><out><xsl:value-of select="$second"/></out></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("local atomic aliases should compile");

    let result = execute_program(
        &program,
        &source,
        "atomic-alias-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("local atomic aliases should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "atomic-alias-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>value</out>");
}

#[test]
fn xslt10_named_call_ignores_undeclared_arguments_and_retains_global_fallback() {
    let source = parse_document(
        "memory:ignored-argument.xml",
        b"<doc/>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:ignored-argument.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:param name="test" select="'global'"/>
          <xsl:template match="/"><out><xsl:call-template name="target"><xsl:with-param name="test" select="'local'"/></xsl:call-template></out></xsl:template>
          <xsl:template name="target"><xsl:choose><xsl:when test="$test = 'global'">global</xsl:when><xsl:otherwise>local</xsl:otherwise></xsl:choose></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 64,
            max_depth: 12,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("undeclared argument should be ignored");

    let result = execute_program(
        &program,
        &source,
        "ignored-argument-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("global comparison should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "ignored-argument-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>global</out>");
}

#[test]
fn template_arguments_preserve_existential_source_path_equality() {
    let source = parse_document(
        "memory:path-equality-argument.xml",
        b"<doc><a>x</a><b>x</b><b>y</b></doc>",
        ParseLimits {
            max_events: 20,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:path-equality-argument.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:template match="doc"><out><xsl:apply-templates select="a"><xsl:with-param name="eq" select="a=b"/><xsl:with-param name="ne" select="a!=b"/></xsl:apply-templates></out></xsl:template>
          <xsl:template match="a"><xsl:param name="eq" select="0"/><xsl:param name="ne" select="0"/><xsl:if test="$eq">equal|</xsl:if><xsl:if test="$ne">not-equal</xsl:if></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 64,
            max_depth: 10,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("source-path equality arguments should compile");

    let result = execute_program(
        &program,
        &source,
        "path-equality-argument-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("source-path equality arguments should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "path-equality-argument-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>equal|not-equal</out>");
}

#[test]
fn xslt10_sum_paths_compose_through_template_arguments_and_computed_attributes() {
    let source = parse_document(
        "memory:sum-argument.xml",
        br#"<doc><group rank="2"><a>2</a><a>3</a></group></doc>"#,
        ParseLimits {
            max_events: 20,
            max_depth: 5,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:sum-argument.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:template match="doc"><out><xsl:apply-templates select="group"><xsl:sort select="@rank" data-type="number"/><xsl:with-param name="total" select="sum(group/a)"/></xsl:apply-templates></out></xsl:template>
          <xsl:template match="group"><xsl:param name="total" select="0"/><item rank="{@rank}"><xsl:attribute name="portion"><xsl:value-of select="concat(sum(a),'/', $total)"/></xsl:attribute></item></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 64,
            max_depth: 10,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("sum-path arguments and computed attributes should compile");

    let result = execute_program(
        &program,
        &source,
        "sum-argument-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("sum-path arguments and computed attributes should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "sum-argument-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(
        serialized,
        r#"<out><item rank="2" portion="5/5"></item></out>"#
    );
}

#[test]
fn xslt10_node_set_string_equal_and_not_equal_are_independently_existential() {
    let source = parse_document(
        "memory:node-set-comparison.xml",
        b"<doc><value/><value>x</value></doc>",
        ParseLimits {
            max_events: 16,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:node-set-comparison.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:variable name="values" select="/doc/value"/>
          <xsl:template match="/"><out><xsl:if test="$values = ''">equal|</xsl:if><xsl:if test="$values != ''">not-equal</xsl:if></out></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 48,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("node-set string comparisons should compile");

    let result = execute_program(
        &program,
        &source,
        "node-set-comparison-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("node-set string comparisons should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "node-set-comparison-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>equal|not-equal</out>");
}

#[test]
fn xslt10_source_node_set_comparisons_are_independently_existential() {
    let source = parse_document(
        "memory:source-node-set-comparison.xml",
        br#"<doc><j l="12" w="33">first</j><j l="17" w="45">second</j><j l="12" w="33">fourth</j></doc>"#,
        ParseLimits {
            max_events: 24,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:source-node-set-comparison.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:template match="doc"><out><xsl:if test="j[@l='12'] = j[@w='33']">if|</xsl:if><xsl:value-of select="j[@l='12'] = j[@w='33']"/>|<xsl:value-of select="j[@l='12'] = j[@w='45']"/>|<xsl:value-of select="j[@l='12'] != j[@l='17']"/></out></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 48,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("source node-set comparisons should compile");

    let result = execute_program(
        &program,
        &source,
        "source-node-set-comparison-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("source node-set comparisons should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "source-node-set-comparison-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out>if|true|false|true</out>");
}

#[test]
fn xslt10_computed_attributes_count_typed_source_paths() {
    let source = parse_document(
        "memory:computed-attribute-count.xml",
        b"<doc><item><child><leaf/></child></item><item/></doc>",
        ParseLimits {
            max_events: 20,
            max_depth: 6,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:computed-attribute-count.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output omit-xml-declaration="yes"/>
          <xsl:template match="doc"><out><xsl:for-each select="item"><n><xsl:attribute name="descendants"><xsl:value-of select="count(descendant::*)"/></xsl:attribute><xsl:attribute name="with-self"><xsl:value-of select="count(descendant-or-self::*)"/></xsl:attribute></n></xsl:for-each></out></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 48,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("computed source-path counts should compile");

    let result = execute_program(
        &program,
        &source,
        "computed-attribute-count-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("computed source-path counts should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "computed-attribute-count-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(
        serialized,
        r#"<out><n descendants="2" with-self="3"></n><n descendants="0" with-self="1"></n></out>"#
    );
}

#[test]
fn source_node_local_variables_execute_directly_in_for_each() {
    const SOURCE: &str = "urn:fastxslt:local-node-variable:source";
    const STYLESHEET: &str = "urn:fastxslt:local-node-variable:stylesheet";
    let stylesheet =
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
      <xsl:output method="xml" omit-xml-declaration="yes"/>
      <xsl:template match="/">
        <xsl:variable name="which" select="root/group/item"/>
        <out><first><xsl:value-of select="$which"/></first><all><xsl:for-each select="$which"><xsl:value-of select="."/></xsl:for-each></all><applied><xsl:apply-templates select="$which" mode="local"/></applied></out>
      </xsl:template>
      <xsl:template match="item" mode="local"><xsl:value-of select="."/></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            b"<root><group><item>A</item><item>B</item></group></root>".to_vec(),
        )
        .expect("admit local node-variable source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit local node-variable stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile local node variable");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("local-node-variable", "result", SOURCE))
        .expect("admit local node-variable request");

    let results = execute_transform_set(builder.seal()).expect("execute local node variable");
    assert_eq!(
        results.by_request["local-node-variable"].serialized,
        "<out><first>A</first><all>AB</all><applied>AB</applied></out>"
    );
}

#[test]
fn source_node_union_variables_deduplicate_before_variable_count() {
    const SOURCE: &str = "urn:fastxslt:local-node-union:source";
    const STYLESHEET: &str = "urn:fastxslt:local-node-union:stylesheet";
    let stylesheet =
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
      <xsl:output method="xml" omit-xml-declaration="yes"/>
      <xsl:template match="/">
        <xsl:variable name="left" select="root/item[1]"/>
        <xsl:variable name="right" select="root/item[2]"/>
        <xsl:variable name="joined" select="$right | $left | $right"/>
        <out><xsl:value-of select="count($joined)"/><xsl:text>:</xsl:text><xsl:for-each select="$joined"><xsl:value-of select="."/></xsl:for-each></out>
      </xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            b"<root><item>A</item><item>B</item></root>".to_vec(),
        )
        .expect("admit local node-union source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit local node-union stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile local node-set union");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("local-node-union", "result", SOURCE))
        .expect("admit local node-union request");

    let results = execute_transform_set(builder.seal()).expect("execute local node-set union");
    assert_eq!(
        results.by_request["local-node-union"].serialized,
        "<out>2:AB</out>"
    );
}

#[test]
fn current_source_node_arguments_compose_with_computed_attributes() {
    const SOURCE: &str = "urn:fastxslt:current-argument:source";
    const STYLESHEET: &str = "urn:fastxslt:current-argument:stylesheet";
    let stylesheet =
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
      <xsl:output method="xml" omit-xml-declaration="yes"/>
      <xsl:template match="doc"><out><xsl:apply-templates select="foo"><xsl:with-param name="node" select="current()"/></xsl:apply-templates></out></xsl:template>
      <xsl:template match="foo"><xsl:param name="node"/><content><xsl:attribute name="from"><xsl:value-of select="@name"/></xsl:attribute><xsl:attribute name="size"><xsl:value-of select="count($node)"/></xsl:attribute><xsl:value-of select="normalize-space($node)"/></content></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(SOURCE, b"<doc>alpha <foo name=\"x\"/> omega</doc>".to_vec())
        .expect("admit current-node argument source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit current-node argument stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile current-node argument");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("current-node-argument", "result", SOURCE))
        .expect("admit current-node argument request");

    let results = execute_transform_set(builder.seal()).expect("execute current-node argument");
    assert_eq!(
        results.by_request["current-node-argument"].serialized,
        r#"<out><content from="x" size="1">alpha omega</content></out>"#
    );
}

#[test]
fn source_variable_paths_apply_predicate_focus_per_root() {
    const SOURCE: &str = "urn:fastxslt:source-variable-predicate:source";
    const STYLESHEET: &str = "urn:fastxslt:source-variable-predicate:stylesheet";
    let stylesheet =
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
      <xsl:output method="xml" omit-xml-declaration="yes"/>
      <xsl:variable name="all" select="//OL[@real='yes']"/>
      <xsl:template match="/"><out><xsl:apply-templates select="$all/LI[@flag][last()]"/></out></xsl:template>
      <xsl:template match="LI[@flag]"><xsl:value-of select="."/></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            b"<doc><OL real=\"yes\"><LI flag=\"yes\">A</LI><LI flag=\"yes\">B</LI></OL><OL real=\"yes\"><LI flag=\"yes\">C</LI><LI>D</LI></OL></doc>".to_vec(),
        )
        .expect("admit source-variable predicate source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit source-variable predicate stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile source-variable predicate path");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("source-variable-predicate", "result", SOURCE))
        .expect("admit source-variable predicate request");

    let results =
        execute_transform_set(builder.seal()).expect("execute source-variable predicate path");
    assert_eq!(
        results.by_request["source-variable-predicate"].serialized,
        "<out>BC</out>"
    );
}

#[test]
fn xslt10_current_predicate_retains_the_outer_source_focus() {
    const SOURCE: &str = "urn:fastxslt:current-predicate:source";
    const STYLESHEET: &str = "urn:fastxslt:current-predicate:stylesheet";
    let stylesheet =
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
      <xsl:output method="xml" omit-xml-declaration="yes"/>
      <xsl:template match="doc"><out><xsl:apply-templates select="mark"/></out></xsl:template>
      <xsl:template match="mark"><count><xsl:value-of select="count(current())"/></count><direct><xsl:value-of select="following-sibling::ch[current()]"/></direct><filtered><xsl:value-of select="(following-sibling::ch[current()])[1]"/></filtered><numeric><xsl:value-of select="following-sibling::*[count(current())]"/></numeric></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            b"<doc><mark/><ch>first</ch><ch>second</ch></doc>".to_vec(),
        )
        .expect("admit current-predicate source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit current-predicate stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile current predicate");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("current-predicate", "result", SOURCE))
        .expect("admit current-predicate request");

    let results = execute_transform_set(builder.seal()).expect("execute current predicate");
    assert_eq!(
        results.by_request["current-predicate"].serialized,
        "<out><count>1</count><direct>first</direct><filtered>first</filtered><numeric>first</numeric></out>"
    );
}

#[test]
fn xslt_current_predicate_does_not_enter_the_general_xpath_path_parser() {
    let stylesheet = parse_document(
        "urn:fastxslt:modern-current-predicate:stylesheet",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="mark"><xsl:value-of select="following-sibling::ch[current()]"/></xsl:template></xsl:stylesheet>"#,
        ParseLimits {
            max_events: 16,
            max_depth: 5,
        },
    )
    .expect("modern stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");

    let failure = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect_err("XSLT 1.0 compatibility must not widen the general XPath parser");

    assert_eq!(failure.code, "FXXP1001");
}

#[test]
fn generate_id_uses_stable_distinct_source_node_identity() {
    const SOURCE: &str = "urn:fastxslt:generate-id:source";
    const STYLESHEET: &str = "urn:fastxslt:generate-id:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(SOURCE, b"<doc><a/><b/></doc>".to_vec())
        .expect("admit source document");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><same><xsl:value-of select="generate-id(/doc/a)=generate-id(/doc/a)"/></same><different><xsl:value-of select="generate-id(/doc/a)=generate-id(/doc/b)"/></different><first><xsl:value-of select="generate-id(/doc/a)"/></first><again><xsl:value-of select="generate-id(/doc/a)"/></again><empty><xsl:value-of select="generate-id(/doc/missing)"/></empty></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(16_384));
    builder
        .add(request("generate-id", "generate-id-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute generate-id case");

    assert_eq!(
        results.by_request["generate-id"].serialized,
        "<out><same>true</same><different>false</different><first>fastxslt-principal-n2</first><again>fastxslt-principal-n2</again><empty></empty></out>"
    );
}

#[test]
fn generate_id_rejects_more_than_one_source_node() {
    const SOURCE: &str = "urn:fastxslt:generate-id-many:source";
    const STYLESHEET: &str = "urn:fastxslt:generate-id-many:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><a/><a/></doc>".to_vec())
        .expect("admit source document");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="generate-id(/doc/a)"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request(
            "generate-id-many",
            "generate-id-many-result",
            SOURCE,
        ))
        .expect("admit request");

    let failure = execute_transform_set(builder.seal()).expect_err("cardinality must be enforced");

    assert_eq!(failure.code, "XPTY0004");
    assert_eq!(failure.category, FailureCategory::Invalid);
}

#[test]
fn static_xsl_element_executes_nested_sequence_constructors() {
    let source = parse_document(
        "memory:static-computed-element.xml",
        b"<docs><a>X</a><a>Y</a><a>Z</a></docs>",
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:static-computed-element.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:template match="docs">
            <out><xsl:element name="all"><xsl:for-each select="a"><xsl:value-of select="."/></xsl:for-each></xsl:element></out>
          </xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 64,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("static xsl:element should compile");

    let result = execute_program(
        &program,
        &source,
        "static-computed-element-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("static xsl:element should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "static-computed-element-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(
        serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out><all>XYZ</all></out>"
    );
}

#[test]
fn static_xsl_element_namespace_serializes_a_default_result_binding() {
    let source = parse_document(
        "memory:static-computed-element-namespace.xml",
        b"<doc/>",
        ParseLimits {
            max_events: 8,
            max_depth: 4,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:static-computed-element-namespace.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output method="xml" omit-xml-declaration="yes"/>
          <xsl:template match="/"><xsl:element name="out" namespace="urn:result"/></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("static namespace should compile");

    let result = execute_program(
        &program,
        &source,
        "static-computed-element-namespace-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("static namespaced xsl:element should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "static-computed-element-namespace-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(serialized, "<out xmlns=\"urn:result\"></out>");
}

#[test]
fn descendant_match_path_accepts_nonadjacent_ancestor() {
    let source = parse_document(
        "memory:descendant-match.xml",
        b"<a><b><c/></b><b><e><c/></e></b></a>",
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("source should parse");
    let source = Document::from_parsed(source).expect("source XDM should build");
    let stylesheet = parse_document(
        "memory:descendant-match.xsl",
        br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0">
          <xsl:output method="xml" omit-xml-declaration="yes"/>
          <xsl:template match="/"><out><xsl:apply-templates/></out></xsl:template>
          <xsl:template match="a|b|e"><xsl:apply-templates/></xsl:template>
          <xsl:template match="a//c"><descendant/></xsl:template>
          <xsl:template match="a/*/c"><grandchild/></xsl:template>
        </xsl:stylesheet>"#,
        ParseLimits {
            max_events: 64,
            max_depth: 8,
        },
    )
    .expect("stylesheet should parse");
    let stylesheet = Document::from_parsed(stylesheet).expect("stylesheet XDM should build");
    let program = crate::compile::golden_stylesheet_experiment::compile_stylesheet(&stylesheet)
        .expect("descendant match paths should compile");

    let result = execute_program(
        &program,
        &source,
        "descendant-match-request",
        &mut InvocationControl::unbounded(),
    )
    .expect("descendant match path should execute");
    let serialized = serialize_xml(
        &result,
        &program.output,
        "descendant-match-request",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("result should serialize");

    assert_eq!(
        serialized,
        "<out><grandchild></grandchild><descendant></descendant></out>"
    );
}

#[test]
fn one_prepared_source_supports_preserving_and_stripping_stylesheets_without_mutation() {
    let parsed_source = parse_document(
        "memory:shared-source.xml",
        b"<root>  <a>A</a>\n  <b>B</b>  </root>",
        ParseLimits {
            max_events: 64,
            max_depth: 8,
        },
    )
    .expect("shared source should parse");
    let source = Document::from_parsed(parsed_source).expect("shared source should prepare");
    let compile = |identity: &str, declaration: &str| {
        let xml = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">{declaration}<xsl:template match="/"><out><xsl:value-of select="."/></out></xsl:template></xsl:stylesheet>"#
        );
        let parsed = parse_document(
            identity,
            xml.as_bytes(),
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .expect("stylesheet should parse");
        let document = Document::from_parsed(parsed).expect("stylesheet XDM should build");
        crate::compile::golden_stylesheet_experiment::compile_stylesheet(&document)
            .expect("stylesheet should compile")
    };
    let preserving = compile("memory:preserving.xsl", "");
    let stripping = compile("memory:stripping.xsl", r#"<xsl:strip-space elements="*"/>"#);

    let execute = |program: &crate::xslt::golden_semantics_experiment::StylesheetProgram,
                   request_id: &str| {
        let mut control = InvocationControl::unbounded();
        execute_program(program, &source, request_id, &mut control)
            .expect("shared prepared source should execute")
    };
    let preserved = execute(&preserving, "preserving-request");
    let stripped = execute(&stripping, "stripping-request");
    let mut reference_control = InvocationControl::unbounded();
    let reference = execute_program_with_parameters_using(
        &stripping,
        &source,
        &BTreeMap::new(),
        MultipleMatchPolicy::UseLast,
        "stripping-reference-request",
        WhitespaceRepresentation::CompleteReference,
        None,
        None,
        &mut reference_control,
    )
    .expect("complete reference should execute");

    assert_eq!(
        preserved.children,
        [ResultNode::Element {
            name: ExpandedName {
                namespace: None,
                local: "out".to_owned(),
            },
            namespaces: Vec::new().into(),
            attributes: Vec::new(),
            children: vec![ResultNode::Text("  A\n  B  ".to_owned())],
        }]
    );
    assert_eq!(
        stripped.children,
        [ResultNode::Element {
            name: ExpandedName {
                namespace: None,
                local: "out".to_owned(),
            },
            namespaces: Vec::new().into(),
            attributes: Vec::new(),
            children: vec![ResultNode::Text("AB".to_owned())],
        }]
    );
    assert_eq!(stripped, reference);
    assert_eq!(source.string_value(source.document_node()), "  A\n  B  ");

    std::thread::scope(|scope| {
        let preserving_run = scope.spawn(|| {
            for iteration in 0..100 {
                assert_eq!(
                    execute(&preserving, &format!("concurrent-preserve-{iteration}")),
                    preserved
                );
            }
        });
        let stripping_run = scope.spawn(|| {
            for iteration in 0..100 {
                assert_eq!(
                    execute(&stripping, &format!("concurrent-strip-{iteration}")),
                    stripped
                );
            }
        });
        preserving_run
            .join()
            .expect("preserving worker should join");
        stripping_run.join().expect("stripping worker should join");
    });
    assert_eq!(source.string_value(source.document_node()), "  A\n  B  ");
}

#[test]
fn exact_element_templates_dispatch_repeated_nodes_in_document_order() {
    const DISPATCH_SOURCE: &str = "urn:fastxslt:golden:template-dispatch:source";
    const DISPATCH_STYLESHEET: &str = "urn:fastxslt:golden:template-dispatch:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(4, 4_096, 8_192));
    resources
        .admit(
            DISPATCH_SOURCE,
            include_bytes!("../../../../corpus/golden/template-dispatch/input.xml").to_vec(),
        )
        .expect("admit dispatch source");
    resources
        .admit(
            DISPATCH_STYLESHEET,
            include_bytes!("../../../../corpus/golden/template-dispatch/stylesheet.xsl").to_vec(),
        )
        .expect("admit dispatch stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, DISPATCH_STYLESHEET).expect("compile dispatch stylesheet once");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request(
            "dispatch-request",
            "dispatch-result",
            DISPATCH_SOURCE,
        ))
        .expect("add dispatch request");

    let results = execute_transform_set(builder.seal()).expect("execute dispatch set");

    assert_eq!(
        results.by_request["dispatch-request"].serialized,
        include_str!("../../../../corpus/golden/template-dispatch/expected.xml").trim()
    );
}

#[test]
fn principal_template_after_two_includes_wins_same_precedence_conflict() {
    const SOURCE: &str = "urn:fastxslt:include-order:source";
    const PRINCIPAL: &str = "https://example.invalid/include-order/main.xsl";
    const MODULES: [(&str, &[u8]); 4] = [
        (
            "https://example.invalid/include-order/b.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:import href="d.xsl"/><xsl:template match="unused-b">B</xsl:template></xsl:stylesheet>"#,
        ),
        (
            "https://example.invalid/include-order/c.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:import href="e.xsl"/><xsl:template match="title">C</xsl:template></xsl:stylesheet>"#,
        ),
        (
            "https://example.invalid/include-order/d.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="unused-d">D</xsl:template></xsl:stylesheet>"#,
        ),
        (
            "https://example.invalid/include-order/e.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="title">E</xsl:template></xsl:stylesheet>"#,
        ),
    ];
    let principal = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/title"/></out></xsl:template><xsl:include href="b.xsl"/><xsl:include href="c.xsl"/><xsl:template match="title">MAIN</xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(6, 8_192, 32_768));
    resources
        .admit(SOURCE, b"<doc><title>value</title></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(PRINCIPAL, principal.to_vec())
        .expect("admit principal stylesheet");
    for (identity, bytes) in MODULES {
        resources
            .admit(identity, bytes.to_vec())
            .expect("admit stylesheet dependency");
    }
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, PRINCIPAL).expect("compile include graph");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request("include-order", "include-order-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute include-order case");

    assert_eq!(
        results.by_request["include-order"].serialized,
        "<out>MAIN</out>"
    );
}

#[test]
fn named_processing_instruction_pattern_outranks_generic_node_test() {
    const SOURCE: &str = "urn:fastxslt:named-pi-pattern:source";
    const STYLESHEET: &str = "urn:fastxslt:named-pi-pattern:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="//processing-instruction()"/></out></xsl:template><xsl:template match="processing-instruction()"><generic/></xsl:template><xsl:template match="processing-instruction('work')"><named><xsl:value-of select="name()"/></named></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(SOURCE, b"<doc><?other no?><?work yes?></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request("named-pi-pattern", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute stylesheet");

    assert_eq!(
        results.by_request["named-pi-pattern"].serialized,
        "<out><generic></generic><named>work</named></out>"
    );
}

#[test]
fn ignored_stylesheet_comments_do_not_split_literal_text_runs() {
    const SOURCE: &str = "urn:fastxslt:stylesheet-comment-text:source";
    const STYLESHEET: &str = "urn:fastxslt:stylesheet-comment-text:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out>x
    <!-- ignored -->y
    <!-- ignored -->
    <z><!-- ignored --> </z></out></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(SOURCE, b"<doc/>".to_vec())
        .expect("admit source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request("stylesheet-comment-text", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute stylesheet");

    assert_eq!(
        results.by_request["stylesheet-comment-text"].serialized,
        "<out>x\n    y\n    \n    <z></z></out>"
    );
}

#[test]
fn positional_patterns_and_avts_share_the_apply_templates_focus() {
    const POSITION_SOURCE: &str = "urn:fastxslt:position-focus:source";
    const POSITION_STYLESHEET: &str = "urn:fastxslt:position-focus:stylesheet";
    let source = b"<doc><simplelist>\n<member>1</member>\n<member>2</member>\n<member>3</member>\n<member>4</member>\n</simplelist></doc>";
    let stylesheet = br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:template match="/"><xsl:apply-templates/></xsl:template>
        <xsl:template match="doc"><xsl:apply-templates/></xsl:template>
        <xsl:template match="simplelist"><out><xsl:apply-templates/></out></xsl:template>
        <xsl:template match="member[position()&lt;last()]"><member pos="{position()}" last="{last()}"/></xsl:template>
        <xsl:template match="member[position()=last()]"><member final="yes" pos="{position()}" last="{last()}"/></xsl:template>
        <xsl:template match="text()"/>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(POSITION_SOURCE, source.to_vec())
        .expect("admit positional source");
    resources
        .admit(POSITION_STYLESHEET, stylesheet.to_vec())
        .expect("admit positional stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, POSITION_STYLESHEET)
        .expect("compile positional patterns and context AVTs");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("position-focus", "result", POSITION_SOURCE))
        .expect("admit positional request");

    let results = execute_transform_set(builder.seal()).expect("execute positional request");
    assert_eq!(
        results.by_request["position-focus"].serialized,
        "<out><member pos=\"2\" last=\"9\"></member><member pos=\"4\" last=\"9\"></member><member pos=\"6\" last=\"9\"></member><member final=\"yes\" pos=\"8\" last=\"9\"></member></out>"
    );
}

#[test]
fn exact_positional_match_patterns_use_named_source_sibling_position() {
    const SOURCE: &str = "urn:fastxslt:positional-match-source-order:source";
    const STYLESHEET: &str = "urn:fastxslt:positional-match-source-order:stylesheet";
    let source = b"<doc><a z='4'>A</a><a z='3'>B</a><a z='2'>C</a><a z='1'>D</a><b>first</b><b>second</b></doc>";
    let stylesheet = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="doc"><out><xsl:apply-templates select="a"><xsl:sort select="@z" data-type="number"/></xsl:apply-templates><xsl:apply-templates select="b"/></out></xsl:template><xsl:template match="a[position()=1]"><a1><xsl:value-of select="."/></a1></xsl:template><xsl:template match="a[position()=2]"><a2><xsl:value-of select="."/></a2></xsl:template><xsl:template match="a[position()=3]"><a3><xsl:value-of select="."/></a3></xsl:template><xsl:template match="a[position()=4]"><a4><xsl:value-of select="."/></a4></xsl:template><xsl:template match="b[position()&lt;2]"><first><xsl:value-of select="."/></first></xsl:template><xsl:template match="b"/></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(SOURCE, source.to_vec())
        .expect("admit positional source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit positional stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile positional patterns");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("positional-match-source-order", "result", SOURCE))
        .expect("admit positional request");

    let results = execute_transform_set(builder.seal()).expect("execute positional request");
    assert_eq!(
        results.by_request["positional-match-source-order"].serialized,
        "<out><a4>D</a4><a3>C</a3><a2>B</a2><a1>A</a1><first>first</first></out>"
    );
}

#[test]
fn temporary_tree_builtins_preserve_mixed_text_in_document_order() {
    const TEMP_SOURCE: &str = "urn:fastxslt:temporary-text:source";
    const TEMP_STYLESHEET: &str = "urn:fastxslt:temporary-text:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:variable name="temporary"><x>head<y>middle</y>tail</x></xsl:variable>
        <xsl:template match="/"><out><xsl:apply-templates select="$temporary" mode="temporary"/></out></xsl:template>
        <xsl:template match="/" mode="temporary"><tree><xsl:apply-templates mode="temporary"/></tree></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(TEMP_SOURCE, b"<principal/>".to_vec())
        .expect("admit principal source");
    resources
        .admit(TEMP_STYLESHEET, stylesheet.to_vec())
        .expect("admit mixed temporary-tree stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, TEMP_STYLESHEET)
        .expect("compile mixed temporary-tree stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("temporary-text", "result", TEMP_SOURCE))
        .expect("admit temporary-text request");

    let results = execute_transform_set(builder.seal()).expect("execute temporary text traversal");
    assert_eq!(
        results.by_request["temporary-text"].serialized,
        "<out><tree>headmiddletail</tree></out>"
    );
}

#[test]
fn temporary_tree_shallow_skip_traverses_elements_and_drops_unmatched_text() {
    const SOURCE: &str = "urn:fastxslt:temporary-shallow-skip:source";
    const STYLESHEET: &str = "urn:fastxslt:temporary-shallow-skip:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:mode name="temporary" on-no-match="shallow-skip"/>
        <xsl:variable name="temporary"><x>head<keep>middle</keep>tail</x></xsl:variable>
        <xsl:template match="/"><out><xsl:apply-templates select="$temporary" mode="temporary"/></out></xsl:template>
        <xsl:template match="keep" mode="temporary"><kept/></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<principal/>".to_vec())
        .expect("admit principal source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit temporary shallow-skip stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile temporary shallow-skip stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("temporary-shallow-skip", "result", SOURCE))
        .expect("admit temporary shallow-skip request");

    let results =
        execute_transform_set(builder.seal()).expect("execute temporary shallow-skip traversal");
    assert_eq!(
        results.by_request["temporary-shallow-skip"].serialized,
        "<out><kept></kept></out>"
    );
}

#[test]
fn temporary_copy_is_shallow_and_executes_its_compiled_attributes_and_body() {
    const SOURCE: &str = "urn:fastxslt:temporary-copy:source";
    const STYLESHEET: &str = "urn:fastxslt:temporary-copy:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:variable name="temporary"><outer><inner><lost/></inner></outer></xsl:variable>
        <xsl:template match="/"><xsl:apply-templates select="$temporary/outer" mode="temporary"/></xsl:template>
        <xsl:template match="outer" mode="temporary"><xsl:copy><xsl:attribute name="marker">kept</xsl:attribute><xsl:apply-templates mode="temporary"/></xsl:copy></xsl:template>
        <xsl:template match="inner" mode="temporary"><xsl:copy/></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(SOURCE, b"<principal/>".to_vec())
        .expect("admit principal source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit temporary-copy stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile temporary xsl:copy");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("temporary-copy", "result", SOURCE))
        .expect("admit temporary-copy request");

    let results = execute_transform_set(builder.seal()).expect("execute temporary xsl:copy");

    assert_eq!(
        results.by_request["temporary-copy"].serialized,
        "<outer marker=\"kept\"><inner></inner></outer>"
    );
}

#[test]
fn temporary_path_templates_receive_the_selected_sequence_focus() {
    const SOURCE: &str = "urn:fastxslt:temporary-focus:source";
    const STYLESHEET: &str = "urn:fastxslt:temporary-focus:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:variable name="temporary"><items><item/><item/></items></xsl:variable>
        <xsl:template match="/"><out><xsl:apply-templates select="$temporary/items/item" mode="temporary"/></out></xsl:template>
        <xsl:template match="item" mode="temporary"><seen p="{position()}" n="{last()}"/></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(SOURCE, b"<principal/>".to_vec())
        .expect("admit principal source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit temporary-focus stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile temporary focus AVTs");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("temporary-focus", "result", SOURCE))
        .expect("admit temporary-focus request");

    let results = execute_transform_set(builder.seal()).expect("execute temporary focus");

    assert_eq!(
        results.by_request["temporary-focus"].serialized,
        "<out><seen p=\"1\" n=\"2\"></seen><seen p=\"2\" n=\"2\"></seen></out>"
    );
}

#[test]
fn context_string_value_avts_share_source_and_temporary_node_semantics() {
    const SOURCE: &str = "urn:fastxslt:context-string-avt:source";
    const STYLESHEET: &str = "urn:fastxslt:context-string-avt:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:variable name="temporary"><item>alpha<part>beta</part>gamma</item></xsl:variable>
        <xsl:template match="/"><out><xsl:apply-templates select="doc"/><xsl:apply-templates select="$temporary/item" mode="temporary"/></out></xsl:template>
        <xsl:template match="doc"><source value="{.}"/></xsl:template>
        <xsl:template match="item" mode="temporary"><temporary value="{.}"/></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(SOURCE, b"<doc>head<part>middle</part>tail</doc>".to_vec())
        .expect("admit source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit context string-value AVT stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile context string-value AVTs");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("context-string-avt", "result", SOURCE))
        .expect("admit context string-value AVT request");

    let results = execute_transform_set(builder.seal()).expect("execute string-value AVTs");
    assert_eq!(
        results.by_request["context-string-avt"].serialized,
        "<out><source value=\"headmiddletail\"></source><temporary value=\"alphabetagamma\"></temporary></out>"
    );
}

#[test]
fn local_position_variable_uses_each_selected_source_node_focus() {
    const SOURCE: &str = "urn:fastxslt:position-variable:source";
    const STYLESHEET: &str = "urn:fastxslt:position-variable:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:template match="/"><out><xsl:for-each select="doc/item"><xsl:variable name="p" select="position()"/><xsl:value-of select="$p"/></xsl:for-each></out></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><item/><item/></doc>".to_vec())
        .expect("admit position-variable source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit position-variable stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile position variable");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("position-variable", "result", SOURCE))
        .expect("admit position-variable request");

    let results = execute_transform_set(builder.seal()).expect("execute position variable");
    assert_eq!(
        results.by_request["position-variable"].serialized,
        "<out>12</out>"
    );
}

#[test]
fn xslt10_sort_orders_for_each_and_apply_templates_with_stable_multiple_keys() {
    const SOURCE: &str = "urn:fastxslt:xslt10-sort:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-sort:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:template match="/"><out>
          <numbers><xsl:for-each select="doc/item"><xsl:sort select="@rank" data-type="number"/><xsl:value-of select="@name"/></xsl:for-each></numbers>
          <names><xsl:apply-templates select="doc/item" mode="named"><xsl:sort select="@group"/><xsl:sort select="@name" order="descending"/></xsl:apply-templates></names>
          <positions><xsl:for-each select="doc/item"><xsl:sort select="position()" data-type="number" order="descending"/><xsl:value-of select="@name"/></xsl:for-each></positions>
          <node-names><xsl:for-each select="doc/names/*"><xsl:sort select="name(.)"/><xsl:value-of select="name()"/><xsl:text>|</xsl:text></xsl:for-each></node-names>
          <attribute-names><xsl:for-each select="doc/names/@*"><xsl:sort select="name(.)"/><xsl:value-of select="name()"/><xsl:text>|</xsl:text></xsl:for-each></attribute-names>
          <lengths><xsl:for-each select="doc/lengths/*"><xsl:sort select="string-length(.)" data-type="number"/><xsl:value-of select="name()"/><xsl:text>|</xsl:text></xsl:for-each></lengths>
          <counts><xsl:for-each select="doc/counts/*"><xsl:sort select="count(*)" data-type="number"/><xsl:value-of select="name()"/><xsl:text>|</xsl:text></xsl:for-each></counts>
          <number-conversion><xsl:for-each select="doc/number-conversion/n"><xsl:sort select="number(@x)"/><xsl:value-of select="@x"/><xsl:text>|</xsl:text></xsl:for-each></number-conversion>
        </out></xsl:template>
        <xsl:template match="item" mode="named"><xsl:value-of select="@name"/></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            br#"<doc xmlns:p="urn:p"><item rank="10" group="b" name="one"/><item rank="2" group="a" name="two"/><item rank="2" group="a" name="three"/><item rank="3" group="a" name="XML"/><names p:c="3" b="2" a="1"><p:c/><b/><a/></names><lengths><ccc>123</ccc><b>1</b><aa>12</aa></lengths><counts><c3><i/><i/><i/></c3><c1><i/></c1><c2><i/><i/></c2></counts><number-conversion><n x="3"/><n x="2"/><n x="a"/><n x="1"/></number-conversion></doc>"#.to_vec(),
        )
        .expect("admit sorting source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit sorting stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile xsl:sort");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("xslt10-sort", "result", SOURCE))
        .expect("admit sorting request");

    let results = execute_transform_set(builder.seal()).expect("execute xsl:sort");
    assert_eq!(
        results.by_request["xslt10-sort"].serialized,
        "<out><numbers>twothreeXMLone</numbers><names>XMLtwothreeone</names><positions>XMLthreetwoone</positions><node-names>a|b|p:c|</node-names><attribute-names>a|b|p:c|</attribute-names><lengths>b|aa|ccc|</lengths><counts>c1|c2|c3|</counts><number-conversion>1|2|3|a|</number-conversion></out>"
    );
}

#[test]
fn xslt10_sort_uses_the_first_document_order_node_from_a_path_union() {
    const SOURCE: &str = "urn:fastxslt:xslt10-sort-path-union:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-sort-path-union:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/entry"><xsl:sort select="name/last | primary/name/last"/></xsl:apply-templates></out></xsl:template><xsl:template match="entry"><xsl:value-of select="name/last"/><xsl:value-of select="primary/name/last"/><xsl:text>|</xsl:text></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            br"<doc><entry><name><last>Zulu</last></name></entry><entry><primary><name><last>Alpha</last></name></primary></entry><entry><primary><name><last>Mike</last></name></primary></entry></doc>".to_vec(),
        )
        .expect("admit path-union sorting source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit path-union sorting stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile path-union sort key");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("path-union-sort", "result", SOURCE))
        .expect("admit path-union sorting request");

    let results = execute_transform_set(builder.seal()).expect("execute path-union sort");
    assert_eq!(
        results.by_request["path-union-sort"].serialized,
        "<out>Alpha|Mike|Zulu|</out>"
    );
}

#[test]
fn xslt10_sort_resolves_a_qualified_attribute_key() {
    const SOURCE: &str = "urn:fastxslt:xslt10-qualified-sort:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-qualified-sort:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            br#"<doc xmlns:p="urn:rank"><item p:rank="2">B</item><item p:rank="1">A</item></doc>"#
                .to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:rank"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:for-each select="doc/item"><xsl:sort select="@p:rank" data-type="number"/><xsl:value-of select="."/></xsl:for-each></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile qualified sort key");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("qualified-sort", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute qualified sort key");

    assert_eq!(
        results.by_request["qualified-sort"].serialized,
        "<out xmlns:p=\"urn:rank\">AB</out>"
    );
}

#[test]
fn xslt10_default_single_number_counts_matching_preceding_siblings() {
    const SOURCE: &str = "urn:fastxslt:xslt10-number:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-number:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:template match="/"><out><xsl:for-each select="doc/item"><n><xsl:number level="single"/></n><p><xsl:number value="position()"/></p></xsl:for-each><values><xsl:number value="1999.499999"/>|<xsl:number value="1999.5"/>|<xsl:number value="'bad'"/>|<xsl:number value="0.42"/></values></out></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><item/><other/><item/><item/></doc>".to_vec())
        .expect("admit numbering source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit numbering stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile xsl:number");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("xslt10-number", "result", SOURCE))
        .expect("admit numbering request");

    let results = execute_transform_set(builder.seal()).expect("execute xsl:number");
    assert_eq!(
        results.by_request["xslt10-number"].serialized,
        "<out><n>1</n><p>1</p><n>2</n><p>2</p><n>3</n><p>3</p><values>1999|2000|NaN|0.42</values></out>"
    );
}

#[test]
fn xslt10_number_applies_bounded_decimal_format_tokens() {
    const STYLESHEET: &str = "urn:fastxslt:number-format";
    const SOURCE: &str = "urn:fastxslt:number-format-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:number value="position()" format="01"/>|<xsl:number value="12" format="(001) "/>|<xsl:number value="'bad'" format="[1]"/>|<xsl:for-each select="doc/n"><xsl:number value="." format="(1) "/></xsl:for-each></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(SOURCE, b"<doc><n> 7 </n><n>bad</n></doc>".to_vec())
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile xsl:number format");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("number-format", "number-format-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute xsl:number format");

    assert_eq!(
        results.by_request["number-format"].serialized,
        "<out>01|(012) |[NaN]|(7) (NaN) </out>"
    );
}

#[test]
fn xslt10_number_applies_alphabetic_and_roman_format_tokens() {
    const STYLESHEET: &str = "urn:fastxslt:number-named-format";
    const SOURCE: &str = "urn:fastxslt:number-named-format-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:number value="1" format="A"/>|<xsl:number value="26" format="a"/>|<xsl:number value="27" format="A-"/>|<xsl:number value="4" format="(I)"/>|<xsl:number value="19" format="i"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(SOURCE, b"<doc/>".to_vec())
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile named number formats");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request(
            "number-named-format",
            "number-named-format-result",
            SOURCE,
        ))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute named number formats");

    assert_eq!(
        results.by_request["number-named-format"].serialized,
        "<out>A|z|AA-|(IV)|xix</out>"
    );
}

#[test]
fn xslt10_multiple_number_applies_compiled_token_and_separator_sequence() {
    const STYLESHEET: &str = "urn:fastxslt:number-token-sequence";
    const SOURCE: &str = "urn:fastxslt:number-token-sequence-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates/></out></xsl:template><xsl:template match="a"><n><xsl:number level="multiple" format="(A--1)"/></n><xsl:apply-templates/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(SOURCE, b"<a><a/><a><a/></a></a>".to_vec())
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile number token sequence");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request(
            "number-token-sequence",
            "number-token-sequence-result",
            SOURCE,
        ))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute number token sequence");

    assert_eq!(
        results.by_request["number-token-sequence"].serialized,
        "<out><n>(A)</n><n>(A--1)</n><n>(A--2)</n><n>(A--2--1)</n></out>"
    );
}

#[test]
fn xslt10_number_applies_static_decimal_grouping() {
    const STYLESHEET: &str = "urn:fastxslt:number-grouping";
    const SOURCE: &str = "urn:fastxslt:number-grouping-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:number value="12345" format="000001" grouping-separator="," grouping-size="3"/>|<xsl:number value="12345" grouping-separator=","/>|<xsl:number value="12345" grouping-size="2"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(SOURCE, b"<doc/>".to_vec())
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile number grouping");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("number-grouping", "number-grouping-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute number grouping");

    assert_eq!(
        results.by_request["number-grouping"].serialized,
        "<out>012,345|12345|12345</out>"
    );
}

#[test]
fn xslt10_single_number_honors_static_count_and_from_patterns() {
    const STYLESHEET: &str = "urn:fastxslt:number-pattern";
    const SOURCE: &str = "urn:fastxslt:number-pattern-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/chapter/*"/></out></xsl:template><xsl:template match="note"><n><xsl:number level="single" count="note" from="chapter" format="(01)"/></n></xsl:template><xsl:template match="inside"><n><xsl:number level="single" count="note" from="chapter"/></n></xsl:template><xsl:template match="other"/></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(
            SOURCE,
            b"<doc><chapter><note/><other/><note><inside/></note><note/></chapter></doc>".to_vec(),
        )
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile number patterns");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("number-pattern", "number-pattern-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute number patterns");

    assert_eq!(
        results.by_request["number-pattern"].serialized,
        "<out><n>(01)</n><n>(02)</n><n>(03)</n></out>"
    );
}

#[test]
fn xslt10_single_number_requires_and_includes_the_from_boundary() {
    const STYLESHEET: &str = "urn:fastxslt:number-from-boundary";
    const SOURCE: &str = "urn:fastxslt:number-from-boundary-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/*/title"/></out></xsl:template><xsl:template match="title"><n><xsl:number level="single" count="section|chapter" from="section"/></n></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(
            SOURCE,
            b"<doc><section><title/></section><chapter><title/></chapter></doc>".to_vec(),
        )
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile number boundary");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request(
            "number-from-boundary",
            "number-from-boundary-result",
            SOURCE,
        ))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute number boundary");

    assert_eq!(
        results.by_request["number-from-boundary"].serialized,
        "<out><n>1</n><n></n></out>"
    );
}

#[test]
fn xslt10_any_number_counts_document_order_and_resets_at_from_boundary() {
    const STYLESHEET: &str = "urn:fastxslt:any-number";
    const SOURCE: &str = "urn:fastxslt:any-number-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/chapter/a"/></out></xsl:template><xsl:template match="a"><n><xsl:number level="any" count="a" from="chapter"/></n></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(
            SOURCE,
            b"<doc><chapter><a/><b/><a/></chapter><chapter><a/></chapter></doc>".to_vec(),
        )
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile level-any numbering");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("any-number", "any-number-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute level-any numbering");

    assert_eq!(
        results.by_request["any-number"].serialized,
        "<out><n>1</n><n>2</n><n>1</n></out>"
    );
}

#[test]
fn xslt10_any_number_formats_an_empty_number_list_without_a_zero_token() {
    const STYLESHEET: &str = "urn:fastxslt:any-number-empty";
    const SOURCE: &str = "urn:fastxslt:any-number-empty-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/chapter"/></out></xsl:template><xsl:template match="chapter"><xsl:number level="any" count="section" format="i."/><xsl:text>chapter</xsl:text></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(SOURCE, b"<doc><chapter/></doc>".to_vec())
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile empty any numbering");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request(
            "any-number-empty",
            "any-number-empty-result",
            SOURCE,
        ))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute empty any numbering");

    assert_eq!(
        results.by_request["any-number-empty"].serialized,
        "<out>.chapter</out>"
    );
}

#[test]
fn xslt10_multiple_number_formats_matching_ancestor_lineage() {
    const STYLESHEET: &str = "urn:fastxslt:multiple-number";
    const SOURCE: &str = "urn:fastxslt:multiple-number-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates/></out></xsl:template><xsl:template match="a"><n><xsl:number level="multiple"/></n><xsl:apply-templates/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(SOURCE, b"<a><a/><a><a/></a></a>".to_vec())
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile multiple numbering");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("multiple-number", "multiple-number-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute multiple numbering");

    assert_eq!(
        results.by_request["multiple-number"].serialized,
        "<out><n>1</n><n>1.1</n><n>1.2</n><n>1.2.1</n></out>"
    );
}

#[test]
fn xslt10_number_count_and_from_accept_static_pattern_unions() {
    const STYLESHEET: &str = "urn:fastxslt:number-union";
    const SOURCE: &str = "urn:fastxslt:number-union-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/chapter/*"/></out></xsl:template><xsl:template match="a"><n><xsl:number level="any" count="a | b" from="chapter | appendix"/></n></xsl:template><xsl:template match="b"><n><xsl:number level="multiple" count="a|b" from="chapter|appendix"/></n></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(
            SOURCE,
            b"<doc><chapter><a/><b/><a/></chapter><appendix><a/></appendix></doc>".to_vec(),
        )
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile number unions");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("number-union", "number-union-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute number unions");

    assert_eq!(
        results.by_request["number-union"].serialized,
        "<out><n>1</n><n>2</n><n>3</n></out>"
    );
}

#[test]
fn xslt10_number_patterns_distinguish_nodes_elements_and_attributes() {
    const STYLESHEET: &str = "urn:fastxslt:number-node-kinds";
    const SOURCE: &str = "urn:fastxslt:number-node-kinds-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/a/@*"/></out></xsl:template><xsl:template match="@*"><n><xsl:number level="any" count="node() | / | @*"/></n></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(SOURCE, br#"<doc><a first="x" second="y"/></doc>"#.to_vec())
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile node-kind patterns");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request(
            "number-node-kinds",
            "number-node-kinds-result",
            SOURCE,
        ))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute node-kind patterns");

    assert_eq!(
        results.by_request["number-node-kinds"].serialized,
        "<out><n>4</n><n>5</n></out>"
    );
}

#[test]
fn xslt10_number_pattern_supports_exact_attribute_value_predicate() {
    const STYLESHEET: &str = "urn:fastxslt:number-attribute-predicate";
    const SOURCE: &str = "urn:fastxslt:number-attribute-predicate-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/chapter/note"/></out></xsl:template><xsl:template match="note"><n><xsl:number level="any" count="note[@flag='yes']" from="chapter"/></n></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(
            SOURCE,
            br#"<doc><chapter><note flag="yes"/><note flag="no"/><note flag="yes"/></chapter></doc>"#.to_vec(),
        )
        .expect("admit source");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile attribute predicate pattern");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request(
            "number-attribute-predicate",
            "number-attribute-predicate-result",
            SOURCE,
        ))
        .expect("admit request");

    let results =
        execute_transform_set(builder.seal()).expect("execute attribute predicate pattern");

    assert_eq!(
        results.by_request["number-attribute-predicate"].serialized,
        "<out><n>1</n><n>1</n><n>2</n></out>"
    );
}

#[test]
fn xslt10_number_pattern_supports_bounded_sibling_position_predicates() {
    const STYLESHEET: &str = "urn:fastxslt:number-position-predicate";
    const SOURCE: &str = "urn:fastxslt:number-position-predicate-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/note"/></out></xsl:template><xsl:template match="note"><odd><xsl:number level="single" count="note[position() mod 2 = 1]"/></odd><second><xsl:number level="single" count="note[2]"/></second></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(SOURCE, b"<doc><note/><note/><note/><note/></doc>".to_vec())
        .expect("admit source");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile position predicate patterns");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request(
            "number-position-predicate",
            "number-position-predicate-result",
            SOURCE,
        ))
        .expect("admit request");

    let results =
        execute_transform_set(builder.seal()).expect("execute position predicate patterns");

    assert_eq!(
        results.by_request["number-position-predicate"].serialized,
        "<out><odd>1</odd><second></second><odd></odd><second>1</second><odd>2</odd><second></second><odd></odd><second></second></out>"
    );
}

#[test]
fn xslt10_number_pattern_supports_one_parent_child_relationship() {
    const STYLESHEET: &str = "urn:fastxslt:number-parent-child-pattern";
    const SOURCE: &str = "urn:fastxslt:number-parent-child-pattern-source";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="1.0"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/*/note"/></out></xsl:template><xsl:template match="note"><chapter><xsl:number count="chapter/note"/></chapter><first><xsl:number count="*/note[1]"/></first></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(
            SOURCE,
            b"<doc><chapter><note/><note/></chapter><appendix><note/></appendix></doc>".to_vec(),
        )
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile parent-child patterns");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request(
            "number-parent-child-pattern",
            "number-parent-child-pattern-result",
            SOURCE,
        ))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute parent-child patterns");

    assert_eq!(
        results.by_request["number-parent-child-pattern"].serialized,
        "<out><chapter>1</chapter><first>1</first><chapter>2</chapter><first></first><chapter></chapter><first>1</first></out>"
    );
}

#[test]
fn typed_string_globals_retain_atomic_identity_for_effective_boolean_value() {
    const SOURCE: &str = "urn:fastxslt:typed-global-ebv:source";
    const STYLESHEET: &str = "urn:fastxslt:typed-global-ebv:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="2.0"
        xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
        xmlns:xs="http://www.w3.org/2001/XMLSchema"
        exclude-result-prefixes="xs">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:variable name="present" as="xs:untypedAtomic">value</xsl:variable>
        <xsl:variable name="empty" as="xs:string" select="''"/>
        <xsl:template match="/"><out><xsl:if test="$present">yes</xsl:if><xsl:if test="$empty">no</xsl:if><xsl:value-of select="boolean($present)"/><xsl:value-of select="boolean($empty)"/></out></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc/>".to_vec())
        .expect("admit typed-global source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit typed-global stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile typed globals");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("typed-global-ebv", "result", SOURCE))
        .expect("admit typed-global request");

    let results = execute_transform_set(builder.seal()).expect("execute typed global EBV");
    assert_eq!(
        results.by_request["typed-global-ebv"].serialized,
        "<out>yestruefalse</out>"
    );
}

#[test]
fn xslt10_empty_bindings_are_strings_while_constructed_content_is_a_temporary_tree() {
    const SOURCE: &str = "urn:fastxslt:xslt10-variable-ebv:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-variable-ebv:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0"
        xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:variable name="empty"></xsl:variable>
        <xsl:variable name="present">value</xsl:variable>
        <xsl:variable name="source-value"><xsl:value-of select="/doc/value"/></xsl:variable>
        <xsl:template match="/"><xsl:variable name="local-empty"/><out><xsl:value-of select="boolean($local-empty)"/><xsl:value-of select="boolean($empty)"/><xsl:value-of select="boolean($present)"/><xsl:value-of select="boolean($source-value)"/></out></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><value>source</value></doc>".to_vec())
        .expect("admit variable EBV source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit variable EBV stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile variable EBV");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("xslt10-variable-ebv", "result", SOURCE))
        .expect("admit variable EBV request");

    let results = execute_transform_set(builder.seal()).expect("execute variable EBV");
    assert_eq!(
        results.by_request["xslt10-variable-ebv"].serialized,
        "<out>falsefalsetruetrue</out>"
    );
}

#[test]
fn xslt10_node_set_boolean_comparisons_use_effective_boolean_value() {
    const SOURCE: &str = "urn:fastxslt:xslt10-node-set-boolean:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-node-set-boolean:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0"
        xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:template match="doc">
            <xsl:variable name="nodes" select="items/item"/>
            <xsl:variable name="words" select="words/word"/>
            <xsl:variable name="numbers" select="numbers/number"/>
            <xsl:variable name="missing" select="missing/item"/>
            <out>
                <e><xsl:value-of select="$nodes=true()"/></e>
                <ne><xsl:value-of select="not($nodes=true())"/></ne>
                <n><xsl:value-of select="true()!=$nodes"/></n>
                <nn><xsl:value-of select="not(true()!=$nodes)"/></nn>
                <se><xsl:value-of select="$words='foo'"/></se>
                <sn><xsl:value-of select="not($words!='foo')"/></sn>
                <ne><xsl:value-of select="$numbers=34"/></ne>
                <nn2><xsl:value-of select="$numbers!=34"/></nn2>
                <empty><xsl:value-of select="not($missing!='foo')"/></empty>
            </out>
        </xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            b"<doc><items><item/></items><words><word>foo</word><word>bar</word></words><numbers><number>10</number><number>34</number></numbers></doc>".to_vec(),
        )
        .expect("admit node-set boolean source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit node-set boolean stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile node-set boolean");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("xslt10-node-set-boolean", "result", SOURCE))
        .expect("admit node-set boolean request");

    let results = execute_transform_set(builder.seal()).expect("execute node-set boolean");
    assert_eq!(
        results.by_request["xslt10-node-set-boolean"].serialized,
        "<out><e>true</e><ne>false</ne><n>false</n><nn>true</nn><se>true</se><sn>false</sn><ne>true</ne><nn2>true</nn2><empty>true</empty></out>"
    );
}

#[test]
fn xslt10_temporary_tree_compares_by_string_or_number_as_required() {
    const SOURCE: &str = "urn:fastxslt:xslt10-temporary-comparison:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-temporary-comparison:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0"
        xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:variable name="word"><xsl:value-of select="/doc/word"/></xsl:variable>
        <xsl:variable name="number"><xsl:value-of select="/doc/number"/></xsl:variable>
        <xsl:template match="doc"><out>
            <s><xsl:value-of select="$word='found'"/></s>
            <sn><xsl:value-of select="'found'!=$word"/></sn>
            <n><xsl:value-of select="$number=17"/></n>
            <nn><xsl:value-of select="not(17=$number)"/></nn>
        </out></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            b"<doc><word>found</word><number>17</number></doc>".to_vec(),
        )
        .expect("admit temporary comparison source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit temporary comparison stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile temporary comparisons");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("xslt10-temporary-comparison", "result", SOURCE))
        .expect("admit temporary comparison request");

    let results = execute_transform_set(builder.seal()).expect("execute temporary comparisons");
    assert_eq!(
        results.by_request["xslt10-temporary-comparison"].serialized,
        "<out><s>true</s><sn>false</sn><n>true</n><nn>false</nn></out>"
    );
}

#[test]
fn xslt10_string_and_number_convert_variable_values() {
    const SOURCE: &str = "urn:fastxslt:xslt10-variable-conversion:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-variable-conversion:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0"
        xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="text"/>
        <xsl:variable name="atomic" select="'7.5'"/>
        <xsl:variable name="empty"><xsl:value-of select="/doc/missing"/></xsl:variable>
        <xsl:variable name="temporary"><xsl:value-of select="/doc/value"/></xsl:variable>
        <xsl:template match="/"><xsl:variable name="nodes" select="doc/n"/><xsl:value-of select="string($atomic)"/>|<xsl:value-of select="number($atomic)"/>|<xsl:value-of select="string($empty)"/>|<xsl:value-of select="number($empty)"/>|<xsl:value-of select="string($temporary)"/>|<xsl:value-of select="number($temporary)"/>|<xsl:value-of select="string($nodes)"/>|<xsl:value-of select="number($nodes)"/></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            b"<doc><value>12.25</value><n>34</n><n>56</n></doc>".to_vec(),
        )
        .expect("admit variable conversion source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit variable conversion stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile variable conversions");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request("xslt10-variable-conversion", "result", SOURCE))
        .expect("admit variable conversion request");

    let results = execute_transform_set(builder.seal()).expect("execute variable conversions");
    assert_eq!(
        results.by_request["xslt10-variable-conversion"].serialized,
        "7.5|7.5||NaN|12.25|12.25|34|34"
    );
}

#[test]
fn xslt10_numeric_variables_select_path_positions() {
    const SOURCE: &str = "urn:fastxslt:xslt10-variable-position:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-variable-position:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0"
        xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="text"/>
        <xsl:variable name="first" select="1"/>
        <xsl:variable name="third" select="3"/>
        <xsl:variable name="fraction" select="'2.5'"/>
        <xsl:template match="doc"><xsl:variable name="rtf">2</xsl:variable><xsl:value-of select="a[position() = $first]"/>|<xsl:value-of select="a[$third]"/>|<xsl:value-of select="a[$third = position()]"/>|<xsl:value-of select="a[$fraction]"/>|<xsl:value-of select="a[$rtf]"/></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(SOURCE, b"<doc><a>A</a><a>B</a><a>C</a></doc>".to_vec())
        .expect("admit variable-position source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit variable-position stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile variable positions");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request("xslt10-variable-position", "result", SOURCE))
        .expect("admit variable-position request");

    let results = execute_transform_set(builder.seal()).expect("execute variable positions");
    assert_eq!(
        results.by_request["xslt10-variable-position"].serialized,
        "A|C|C||A"
    );
}

#[test]
fn xslt10_binary_string_functions_convert_first_path_node() {
    const SOURCE: &str = "urn:fastxslt:xslt10-path-string-functions:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-path-string-functions:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0"
        xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
        <xsl:output method="text"/>
        <xsl:variable name="suffix" select="'!'"/>
        <xsl:template match="doc"><xsl:value-of select="starts-with(value, 'alpha')"/>|<xsl:value-of select="contains(value, '/beta')"/>|<xsl:value-of select="substring-before(value, '/')"/>|<xsl:value-of select="substring-after(value, '/')"/>|<xsl:value-of select="contains(missing, '')"/>|<xsl:value-of select="substring(value, 2, 4)"/>|<xsl:value-of select="substring(value, 2.5, 3.6)"/>|<xsl:value-of select="substring(missing, 1)"/>|<xsl:value-of select="translate(value, 'a/', 'A-')"/>|<xsl:value-of select="translate(value, 'ab', 'X')"/>|<xsl:value-of select="concat(first, '-', second, 34, missing, $suffix)"/></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            b"<doc><value>alpha/beta</value><value>ignored</value><first>A</first><second>B</second></doc>".to_vec(),
        )
        .expect("admit path string-function source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit path string-function stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile path string functions");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request("xslt10-path-string-functions", "result", SOURCE))
        .expect("admit path string-function request");

    let results = execute_transform_set(builder.seal()).expect("execute path string functions");
    assert_eq!(
        results.by_request["xslt10-path-string-functions"].serialized,
        "true|true|alpha|beta|true|lpha|pha/||AlphA-betA|XlphX/etX|A-B34!"
    );
}

#[test]
fn xslt10_sum_converts_every_selected_node_and_empty_to_zero() {
    const SOURCE: &str = "urn:fastxslt:xslt10-sum-path:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-sum-path:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="doc"><xsl:value-of select="sum(n)"/>|<xsl:value-of select="sum(n/@value)"/>|<xsl:value-of select="sum(missing)"/>|<xsl:value-of select="sum(bad)"/></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            b"<doc><n value='1.25'>2.5</n><n value='2.75'>3.5</n><bad>not-a-number</bad></doc>"
                .to_vec(),
        )
        .expect("admit sum source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit sum stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile sum paths");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request("xslt10-sum-path", "result", SOURCE))
        .expect("admit sum request");

    let results = execute_transform_set(builder.seal()).expect("execute sum paths");
    assert_eq!(
        results.by_request["xslt10-sum-path"].serialized,
        "6|4|0|NaN"
    );
}

#[test]
fn qualified_temporary_path_dispatches_a_matching_union_alternative() {
    const PATH_SOURCE: &str = "urn:fastxslt:temporary-path:source";
    const PATH_STYLESHEET: &str = "urn:fastxslt:temporary-path:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="2.0"
        xmlns:xsl="http://www.w3.org/1999/XSL/Transform"
        xmlns:db="http://docbook.org/docbook-ng"
        xmlns:m="http://docbook.org/xslt/ns/mode"
        exclude-result-prefixes="db m">
        <xsl:output method="xml" omit-xml-declaration="yes"/>
        <xsl:variable name="dummy"><db:book><db:info><db:title>Book Title</db:title></db:info><db:chapter><db:info><db:title>ChapterTitle</db:title></db:info></db:chapter></db:book></xsl:variable>
        <xsl:template match="/"><xsl:apply-templates select="$dummy/db:book/db:chapter/db:info/db:title" mode="m:titlepage-mode"/></xsl:template>
        <xsl:template match="db:chapter/db:info/db:title | db:appendix/db:info/db:title | db:preface/db:info/db:title | db:bibliography/db:info/db:title" mode="m:titlepage-mode" priority="100"><high><xsl:apply-templates/></high></xsl:template>
        <xsl:template match="db:title" mode="m:titlepage-mode"><low/></xsl:template>
    </xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(PATH_SOURCE, b"<principal/>".to_vec())
        .expect("admit principal source");
    resources
        .admit(PATH_STYLESHEET, stylesheet.to_vec())
        .expect("admit qualified temporary-path stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, PATH_STYLESHEET)
        .expect("compile qualified temporary-path stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("temporary-path", "result", PATH_SOURCE))
        .expect("admit temporary-path request");

    let results = execute_transform_set(builder.seal()).expect("execute qualified temporary path");
    assert_eq!(
        results.by_request["temporary-path"].serialized,
        "<high>ChapterTitle</high>"
    );
}

#[test]
fn context_node_name_uses_the_retained_source_lexical_prefix() {
    const SOURCE: &str = "urn:fastxslt:name-context:source";
    const STYLESHEET: &str = "urn:fastxslt:name-context:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            br#"<doc xmlns:p="urn:example"><p:item/></doc>"#.to_vec(),
        )
        .expect("admit namespaced source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:example"><xsl:template match="/"><xsl:apply-templates/></xsl:template><xsl:template match="doc"><xsl:apply-templates select="*"/></xsl:template><xsl:template match="p:item"><out><xsl:value-of select="name(.)"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile exact name operation");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("namespaced-name", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute lexical node name");
    assert_eq!(
        results.by_request["namespaced-name"].serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out xmlns:p=\"urn:example\">p:item</out>"
    );
}

#[test]
fn node_name_path_returns_retained_element_and_attribute_lexical_names() {
    const SOURCE: &str = "urn:fastxslt:name-path:source";
    const STYLESHEET: &str = "urn:fastxslt:name-path:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            br#"<doc xmlns:p="urn:example"><p:item p:code="x"/></doc>"#.to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:example"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:value-of select="name(doc/p:item)"/>|<xsl:value-of select="name(doc/p:item/@p:code)"/>|<xsl:value-of select="name(doc/missing)"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile node-name paths");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("name-path", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute node-name paths");

    assert_eq!(
        results.by_request["name-path"].serialized,
        "<out xmlns:p=\"urn:example\">p:item|p:code|</out>"
    );
}

#[test]
fn value_of_resolves_simple_qualified_element_and_attribute_paths() {
    const SOURCE: &str = "urn:fastxslt:qualified-value:source";
    const STYLESHEET: &str = "urn:fastxslt:qualified-value:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            br#"<doc xmlns:p="urn:example" xml:lang="en"><p:item p:code="selected">value</p:item></doc>"#
                .to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:example"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:value-of select="doc/p:item"/>|<xsl:value-of select="doc/p:item/@p:code"/>|<xsl:value-of select="doc/@xml:lang"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile qualified value paths");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("qualified-value", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute qualified value paths");

    assert_eq!(
        results.by_request["qualified-value"].serialized,
        "<out xmlns:p=\"urn:example\">value|selected|en</out>"
    );
}

#[test]
fn for_each_and_apply_templates_resolve_simple_qualified_paths() {
    const SOURCE: &str = "urn:fastxslt:qualified-selection:source";
    const STYLESHEET: &str = "urn:fastxslt:qualified-selection:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            br#"<doc xmlns:p="urn:example"><p:item>A</p:item><p:item>B</p:item></doc>"#.to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:example"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:for-each select="doc/p:item"><each><xsl:value-of select="."/></each></xsl:for-each><xsl:apply-templates select="doc/p:item"/></out></xsl:template><xsl:template match="p:item"><applied><xsl:value-of select="."/></applied></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile qualified node selections");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("qualified-selection", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute qualified selections");

    assert_eq!(
        results.by_request["qualified-selection"].serialized,
        "<out xmlns:p=\"urn:example\"><each>A</each><each>B</each><applied>A</applied><applied>B</applied></out>"
    );
}

#[test]
fn node_name_path_rejects_more_than_one_node() {
    const SOURCE: &str = "urn:fastxslt:name-path-many:source";
    const STYLESHEET: &str = "urn:fastxslt:name-path-many:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><item/><item/></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="name(doc/item)"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile node-name path");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("name-path-many", "result", SOURCE))
        .expect("admit request");

    let failure = execute_transform_set(builder.seal()).expect_err("cardinality must be enforced");

    assert_eq!(failure.code, "XPTY0004");
    assert_eq!(failure.category, FailureCategory::Invalid);
}

#[test]
fn context_local_name_and_namespace_uri_use_the_expanded_name() {
    const SOURCE: &str = "urn:fastxslt:expanded-name-context:source";
    const STYLESHEET: &str = "urn:fastxslt:expanded-name-context:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            br#"<doc xmlns:p="urn:example"><p:item/></doc>"#.to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:example"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:apply-templates select="doc/*"/></xsl:template><xsl:template match="p:item"><out><xsl:value-of select="local-name()"/>|<xsl:value-of select="namespace-uri()"/>|<xsl:value-of select="local-name(.)"/>|<xsl:value-of select="namespace-uri(.)"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile expanded-name operations");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("expanded-name", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute expanded-name operations");
    assert_eq!(
        results.by_request["expanded-name"].serialized,
        "<out xmlns:p=\"urn:example\">item|urn:example|item|urn:example</out>"
    );
}

#[test]
fn path_local_name_and_namespace_uri_use_the_selected_expanded_name() {
    const SOURCE: &str = "urn:fastxslt:expanded-name-path:source";
    const STYLESHEET: &str = "urn:fastxslt:expanded-name-path:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            br#"<doc xmlns:p="urn:example"><p:item/><plain/></doc>"#.to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:example"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:value-of select="local-name(doc/p:item)"/>|<xsl:value-of select="namespace-uri(doc/p:item)"/>|<xsl:value-of select="local-name(doc/plain)"/>|<xsl:value-of select="namespace-uri(doc/plain)"/>|<xsl:value-of select="local-name(doc/missing)"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile expanded-name paths");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("expanded-name-path", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute expanded-name paths");

    assert_eq!(
        results.by_request["expanded-name-path"].serialized,
        "<out xmlns:p=\"urn:example\">item|urn:example|plain||</out>"
    );
}

#[test]
fn expanded_name_path_rejects_more_than_one_node() {
    const SOURCE: &str = "urn:fastxslt:expanded-name-many:source";
    const STYLESHEET: &str = "urn:fastxslt:expanded-name-many:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><item/><item/></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="local-name(doc/item)"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile expanded-name path");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("expanded-name-many", "result", SOURCE))
        .expect("admit request");

    let failure = execute_transform_set(builder.seal()).expect_err("cardinality must be enforced");

    assert_eq!(failure.code, "XPTY0004");
    assert_eq!(failure.category, FailureCategory::Invalid);
}

#[test]
fn context_string_function_reuses_the_context_item_string_value() {
    const SOURCE: &str = "urn:fastxslt:context-string:source";
    const STYLESHEET: &str = "urn:fastxslt:context-string:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc>alpha<part>beta</part>gamma</doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:apply-templates select="doc"/></xsl:template><xsl:template match="doc"><out><xsl:value-of select="string()"/>|<xsl:value-of select="string(.)"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile context string operations");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("context-string", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute context string operations");
    assert_eq!(
        results.by_request["context-string"].serialized,
        "<out>alphabetagamma|alphabetagamma</out>"
    );
}

#[test]
fn xslt10_value_of_converts_a_node_set_from_its_first_node_in_document_order() {
    const SOURCE: &str = "urn:fastxslt:xslt10-value-of-many:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-value-of-many:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            b"<doc><item>first</item><item>second</item></doc>".to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="doc/item"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile XSLT 1.0 conversion");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("xslt10-value-of-many", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute XSLT 1.0 conversion");

    assert_eq!(
        results.by_request["xslt10-value-of-many"].serialized,
        "first"
    );
}

#[test]
fn modern_value_of_does_not_inherit_xslt10_first_node_conversion() {
    const SOURCE: &str = "urn:fastxslt:modern-value-of-many:source";
    const STYLESHEET: &str = "urn:fastxslt:modern-value-of-many:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            b"<doc><item>first</item><item>second</item></doc>".to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="doc/item"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile modern selection");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("modern-value-of-many", "result", SOURCE))
        .expect("admit request");

    let failure =
        execute_transform_set(builder.seal()).expect_err("cardinality must remain explicit");

    assert_eq!(failure.code, "FXRT1001");
    assert_eq!(failure.category, FailureCategory::Unsupported);
}

#[test]
fn context_normalize_space_streams_across_descendant_text_boundaries() {
    const SOURCE: &str = "urn:fastxslt:context-normalize-space:source";
    const STYLESHEET: &str = "urn:fastxslt:context-normalize-space:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            b"<doc>  alpha <part> beta </part>\n gamma  </doc>".to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:apply-templates select="doc"/></xsl:template><xsl:template match="doc"><out><xsl:value-of select="normalize-space()"/>|<xsl:value-of select="normalize-space(.)"/>|<xsl:value-of select="normalize-space(part)"/>|<xsl:value-of select="normalize-space(missing)"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile context normalization operations");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("context-normalize-space", "result", SOURCE))
        .expect("admit request");

    let results =
        execute_transform_set(builder.seal()).expect("execute context normalization operations");
    assert_eq!(
        results.by_request["context-normalize-space"].serialized,
        "<out>alpha beta gamma|alpha beta gamma|beta|</out>"
    );
}

#[test]
fn normalize_space_path_rejects_more_than_one_node() {
    const SOURCE: &str = "urn:fastxslt:normalize-space-many:source";
    const STYLESHEET: &str = "urn:fastxslt:normalize-space-many:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><part>a</part><part>b</part></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="normalize-space(/doc/part)"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request(
            "normalize-space-many",
            "normalize-space-many-result",
            SOURCE,
        ))
        .expect("admit request");

    let failure = execute_transform_set(builder.seal()).expect_err("cardinality must be enforced");

    assert_eq!(failure.code, "XPTY0004");
    assert_eq!(failure.category, FailureCategory::Invalid);
}

#[test]
fn context_string_length_counts_unicode_across_descendant_text_boundaries() {
    const SOURCE: &str = "urn:fastxslt:context-string-length:source";
    const STYLESHEET: &str = "urn:fastxslt:context-string-length:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, "<doc>a<part>🦀</part>é</doc>".as_bytes().to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:apply-templates select="doc"/></xsl:template><xsl:template match="doc"><out><xsl:value-of select="string-length()"/>|<xsl:value-of select="string-length ()"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile context string length");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("context-string-length", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute context string length");
    assert_eq!(
        results.by_request["context-string-length"].serialized,
        "<out>3|3</out>"
    );
}

#[test]
fn value_of_position_and_last_use_the_current_sequence_focus() {
    const SOURCE: &str = "urn:fastxslt:value-focus:source";
    const STYLESHEET: &str = "urn:fastxslt:value-focus:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><item/><item/><item/></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/item"/></out></xsl:template><xsl:template match="item"><at><xsl:value-of select="position()"/>/<xsl:value-of select="last()"/></at></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile focus operations");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("value-focus", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute focus operations");
    assert_eq!(
        results.by_request["value-focus"].serialized,
        "<out><at>1/3</at><at>2/3</at><at>3/3</at></out>"
    );
}

#[test]
fn value_of_focus_equality_uses_apply_and_for_each_focus() {
    const SOURCE: &str = "urn:fastxslt:value-focus-equality:source";
    const STYLESHEET: &str = "urn:fastxslt:value-focus-equality:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><item/><item/><item/></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/item"/><xsl:for-each select="doc/item"><size><xsl:value-of select="3 = last()"/></size></xsl:for-each></out></xsl:template><xsl:template match="item"><at><xsl:value-of select="position() = 2"/>/<xsl:value-of select="last() = 3"/></at></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile focus equalities");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("value-focus-equality", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute focus equalities");
    assert_eq!(
        results.by_request["value-focus-equality"].serialized,
        "<out><at>false/true</at><at>true/true</at><at>false/true</at><size>true</size><size>true</size><size>true</size></out>"
    );
}

#[test]
fn focus_boolean_comparisons_use_the_current_sequence_focus() {
    const SOURCE: &str = "urn:fastxslt:boolean-focus:source";
    const STYLESHEET: &str = "urn:fastxslt:boolean-focus:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            b"<doc><item>A</item><item>B</item><item>C</item></doc>".to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:for-each select="doc/item"><xsl:if test="position()=1">[</xsl:if><xsl:value-of select="."/><xsl:if test="position()!=last()">,</xsl:if><xsl:if test="position()=last()">]</xsl:if></xsl:for-each></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile focus comparison");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("boolean-focus", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute focus comparison");
    assert_eq!(
        results.by_request["boolean-focus"].serialized,
        "<out>[A,B,C]</out>"
    );
}

#[test]
fn focus_relations_compose_with_short_circuit_and() {
    const SOURCE: &str = "urn:fastxslt:boolean-focus-range:source";
    const STYLESHEET: &str = "urn:fastxslt:boolean-focus-range:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            b"<doc><item>A</item><item>B</item><item>C</item><item>D</item><item>E</item><item>F</item><item>G</item></doc>".to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:for-each select="doc/item"><xsl:if test="position() &gt;= 2 and position() &lt;= 6"><xsl:value-of select="."/></xsl:if></xsl:for-each></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile focus range");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("boolean-focus-range", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute focus range");
    assert_eq!(
        results.by_request["boolean-focus-range"].serialized,
        "<out>BCDEF</out>"
    );
}

#[test]
fn focus_equality_supports_the_ceiling_of_half_the_sequence_size() {
    const SOURCE: &str = "urn:fastxslt:boolean-focus-middle:source";
    const STYLESHEET: &str = "urn:fastxslt:boolean-focus-middle:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            b"<doc><item>A</item><item>B</item><item>C</item><item>D</item><item>E</item><item>F</item><item>G</item></doc>".to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:for-each select="doc/item"><xsl:if test="position() = ceiling(last() div 2)"><xsl:value-of select="."/></xsl:if></xsl:for-each></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile focus middle");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("boolean-focus-middle", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute focus middle");
    assert_eq!(
        results.by_request["boolean-focus-middle"].serialized,
        "<out>D</out>"
    );
}

#[test]
fn count_path_equality_drives_instruction_conditions() {
    const SOURCE: &str = "urn:fastxslt:count-condition:source";
    const STYLESHEET: &str = "urn:fastxslt:count-condition:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            b"<doc><item>A</item><item><child>B</child></item></doc>".to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/item"/></out></xsl:template><xsl:template match="item"><xsl:choose><xsl:when test="count(./*)=0"><leaf/></xsl:when><xsl:otherwise><branch/></xsl:otherwise></xsl:choose></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile count condition");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("count-condition", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute count condition");
    assert_eq!(
        results.by_request["count-condition"].serialized,
        "<out><leaf></leaf><branch></branch></out>"
    );
}

#[test]
fn apply_templates_path_union_normalizes_identity_and_document_order() {
    const SOURCE: &str = "urn:fastxslt:apply-union:source";
    const STYLESHEET: &str = "urn:fastxslt:apply-union:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br#"<doc a="A" b="B"/>"#.to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc"/></out></xsl:template><xsl:template match="doc"><xsl:apply-templates select="@b | @a | @b"/></xsl:template><xsl:template match="@*"><xsl:value-of select="."/><xsl:if test="position()!=last()">,</xsl:if></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile apply union");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("apply-union", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute apply union");
    assert_eq!(
        results.by_request["apply-union"].serialized,
        "<out>A,B</out>"
    );
}

#[test]
fn apply_templates_variable_path_union_normalizes_identity_and_document_order() {
    const SOURCE: &str = "urn:fastxslt:apply-variable-union:source";
    const STYLESHEET: &str = "urn:fastxslt:apply-variable-union:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            br"<doc><first>A</first><second>B</second></doc>".to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:variable name="chosen" select="doc/second"/><out><xsl:apply-templates select="$chosen | doc/first | $chosen"/></out></xsl:template><xsl:template match="first"><xsl:value-of select="."/><xsl:if test="position()!=last()">,</xsl:if></xsl:template><xsl:template match="second"><xsl:value-of select="."/><xsl:if test="position()!=last()">,</xsl:if></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile variable/path apply union");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("apply-variable-union", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute variable/path apply union");
    assert_eq!(
        results.by_request["apply-variable-union"].serialized,
        "<out>A,B</out>"
    );
}

#[test]
fn source_attribute_copy_uses_the_existing_pending_attribute_owner() {
    const SOURCE: &str = "urn:fastxslt:source-attribute-copy:source";
    const STYLESHEET: &str = "urn:fastxslt:source-attribute-copy:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br#"<doc a="A" b="B"/>"#.to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="doc/@b | doc/@a"/></out></xsl:template><xsl:template match="@*"><xsl:copy/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile source attribute copy");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("source-attribute-copy", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute source attribute copy");
    assert_eq!(
        results.by_request["source-attribute-copy"].serialized,
        "<out a=\"A\" b=\"B\"></out>"
    );
}

#[test]
fn xslt10_literal_attribute_composes_text_with_first_source_path_node() {
    const SOURCE: &str = "urn:fastxslt:mixed-path-avt:source";
    const STYLESHEET: &str = "urn:fastxslt:mixed-path-avt:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            br#"<docs><doc2><doc3><a level="first"/><a level="second"/></doc3></doc2></docs>"#
                .to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out href="before:{.//doc2/doc3/a/@level}:after" empty="before:{missing/@value}:after"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile mixed path AVT");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("mixed-path-avt", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute mixed path AVT");
    assert_eq!(
        results.by_request["mixed-path-avt"].serialized,
        "<out href=\"before:first:after\" empty=\"before::after\"></out>"
    );
}

#[test]
fn xslt10_literal_attribute_composes_text_with_source_attribute_integer_offset() {
    const SOURCE: &str = "urn:fastxslt:attribute-offset-avt:source";
    const STYLESHEET: &str = "urn:fastxslt:attribute-offset-avt:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br#"<response indice="1"/>"#.to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="response"><out y="before{@indice - 1}after"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile attribute-offset AVT");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("attribute-offset-avt", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute attribute-offset AVT");
    assert_eq!(
        results.by_request["attribute-offset-avt"].serialized,
        "<out y=\"before0after\"></out>"
    );
}

#[test]
fn xslt10_literal_attribute_concatenates_two_source_attributes_inside_text() {
    const SOURCE: &str = "urn:fastxslt:attribute-concat-avt:source";
    const STYLESHEET: &str = "urn:fastxslt:attribute-concat-avt:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br#"<doc a="Front" b="Back"/>"#.to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="doc"><out z="Before{concat(@a,@b)}After"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile attribute-concat AVT");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("attribute-concat-avt", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute attribute-concat AVT");
    assert_eq!(
        results.by_request["attribute-concat-avt"].serialized,
        "<out z=\"BeforeFrontBackAfter\"></out>"
    );
}

#[test]
fn xslt10_literal_attribute_compares_two_source_attributes_with_starts_with() {
    const SOURCE: &str = "urn:fastxslt:attribute-starts-with-avt:source";
    const STYLESHEET: &str = "urn:fastxslt:attribute-starts-with-avt:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br#"<doc a="fools" b="foo"/>"#.to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="doc"><out z="Before{starts-with(@a,@b)}After"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile attribute starts-with AVT");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("attribute-starts-with-avt", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute attribute starts-with AVT");
    assert_eq!(
        results.by_request["attribute-starts-with-avt"].serialized,
        "<out z=\"BeforetrueAfter\"></out>"
    );
}

#[test]
fn xslt10_literal_attribute_concatenates_literal_with_global_variable() {
    const SOURCE: &str = "urn:fastxslt:literal-variable-concat-avt:source";
    const STYLESHEET: &str = "urn:fastxslt:literal-variable-concat-avt:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br"<doc/>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:variable name="color" select="'red'"/><xsl:template match="doc"><out style="{concat('border: solid ',$color)}"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile literal-variable concat AVT");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("literal-variable-concat-avt", "result", SOURCE))
        .expect("admit request");

    let results =
        execute_transform_set(builder.seal()).expect("execute literal-variable concat AVT");
    assert_eq!(
        results.by_request["literal-variable-concat-avt"].serialized,
        "<out style=\"border: solid red\"></out>"
    );
}

#[test]
fn xslt10_text_tree_variable_composes_with_source_path_avts() {
    const SOURCE: &str = "urn:fastxslt:text-tree-variable-avt:source";
    const STYLESHEET: &str = "urn:fastxslt:text-tree-variable-avt:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            br#"<photograph><href>headquarters.jpg</href><size width="300"/></photograph>"#
                .to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="photograph"><xsl:variable name="image-dir">/images</xsl:variable><out src="{$image-dir}/{href}" width="{size/@width}"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile text-tree variable AVT");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request("text-tree-variable-avt", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute text-tree variable AVT");
    assert_eq!(
        results.by_request["text-tree-variable-avt"].serialized,
        "<out src=\"/images/headquarters.jpg\" width=\"300\"></out>"
    );
}

#[test]
fn xslt10_text_tree_variable_avts_shadow_across_nested_scope() {
    const SOURCE: &str = "urn:fastxslt:text-tree-variable-shadow:source";
    const STYLESHEET: &str = "urn:fastxslt:text-tree-variable-shadow:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(SOURCE, br"<doc/>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><xsl:variable name="bar">outer</xsl:variable><outer bar="{$bar}"><xsl:for-each select="./*"><xsl:variable name="bar">inner</xsl:variable><inner bar="{$bar}"/></xsl:for-each></outer></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile text-tree shadowing");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request("text-tree-variable-shadow", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute text-tree shadowing");
    assert_eq!(
        results.by_request["text-tree-variable-shadow"].serialized,
        "<outer bar=\"outer\"><inner bar=\"inner\"></inner></outer>"
    );
}

#[test]
fn parent_name_value_uses_the_typed_singleton_parent_path() {
    const SOURCE: &str = "urn:fastxslt:parent-name-value:source";
    const STYLESHEET: &str = "urn:fastxslt:parent-name-value:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><group><item/></group></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><xsl:apply-templates select="doc/group/item"/></xsl:template><xsl:template match="item"><out><xsl:value-of select="name(..)"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile parent name operation");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("parent-name-value", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute parent name operation");
    assert_eq!(
        results.by_request["parent-name-value"].serialized,
        "<out>group</out>"
    );
}

#[test]
fn unqualified_name_comparison_does_not_match_a_namespaced_parent() {
    const SOURCE: &str = "urn:fastxslt:parent-name:source";
    const STYLESHEET: &str = "urn:fastxslt:parent-name:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            br#"<outer xmlns:p="urn:example"><item/><p:holder><p:item/></p:holder></outer>"#
                .to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="outer/item"/><xsl:apply-templates select="outer/*/*"/></out></xsl:template><xsl:template match="*"><xsl:if test="name(..)='outer'"><hit/></xsl:if><xsl:if test="name(..)='holder'"><bad/></xsl:if></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile parent-name tests");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("parent-name", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute parent-name tests");
    assert_eq!(
        results.by_request["parent-name"].serialized,
        "<out><hit></hit></out>"
    );
}

#[test]
fn context_string_length_counts_unicode_codepoints_and_not_utf8_bytes() {
    const SOURCE: &str = "urn:fastxslt:string-length:source";
    const STYLESHEET: &str = "urn:fastxslt:string-length:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, "<doc>é</doc>".as_bytes().to_vec())
        .expect("admit Unicode source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:if test="string-length(.) = 1"><codepoint/></xsl:if><xsl:if test="string-length(.) = 2"><bytes/></xsl:if></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile string-length tests");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("string-length", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute string-length tests");
    assert_eq!(
        results.by_request["string-length"].serialized,
        "<out><codepoint></codepoint></out>"
    );
}

#[test]
fn xpath10_literal_comparisons_preserve_atomic_semantics() {
    const SOURCE: &str = "urn:fastxslt:numeric-comparison:source";
    const STYLESHEET: &str = "urn:fastxslt:numeric-comparison:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br"<doc/>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="1=1"/>|<xsl:value-of select="1!=1.00"/>|<xsl:value-of select="0 = -0"/>|<xsl:value-of select="1.9999999 &lt; 2"/>|<xsl:value-of select="2.0000001 &lt; 2.0"/>|<xsl:value-of select="false()!=true()"/>|<xsl:value-of select="'ace' != 'abc'"/>|<xsl:value-of select="number(true())=1"/>|<xsl:value-of select="number(false())=1"/>|<xsl:value-of select="false() and 1 div 0"/>|<xsl:value-of select="true() or 1 div 0"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile numeric comparison tests");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("numeric-comparison", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute numeric comparisons");
    assert_eq!(
        results.by_request["numeric-comparison"].serialized,
        "true|false|true|true|false|true|true|true|false|false|true"
    );
}

#[test]
fn boolean_relative_name_path_uses_the_dynamic_context() {
    const SOURCE: &str = "urn:fastxslt:boolean-path:source";
    const STYLESHEET: &str = "urn:fastxslt:boolean-path:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br"<doc><present/></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:apply-templates select="doc"/></xsl:template><xsl:template match="doc"><xsl:value-of select="boolean(present)"/>|<xsl:value-of select="boolean(missing)"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile boolean path");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("boolean-path", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute boolean path");
    assert_eq!(results.by_request["boolean-path"].serialized, "true|false");
}

#[test]
fn xpath10_mixed_boolean_equality_is_selected_at_compilation() {
    const SOURCE: &str = "urn:fastxslt:xpath10-boolean-coercion:source";
    const LEGACY: &str = "urn:fastxslt:xpath10-boolean-coercion:legacy";
    const MODERN: &str = "urn:fastxslt:xpath10-boolean-coercion:modern";
    let body = r#"<xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="true()='0'"/>|<xsl:value-of select="false()=''"/>|<xsl:value-of select="true()=2"/>|<xsl:value-of select="false()=0"/>|<xsl:value-of select="0=false()"/>|<xsl:value-of select="'0'=true()"/>|<xsl:value-of select="1='001'"/>|<xsl:value-of select="0='false'"/>|<xsl:value-of select="0!='false'"/></xsl:template>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(3, 8_192, 16_384));
    resources
        .admit(SOURCE, br"<doc/>".to_vec())
        .expect("admit source");
    resources
        .admit(
            LEGACY,
            format!(r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">{body}</xsl:stylesheet>"#).into_bytes(),
        )
        .expect("admit legacy stylesheet");
    resources
        .admit(
            MODERN,
            format!(r#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">{body}</xsl:stylesheet>"#).into_bytes(),
        )
        .expect("admit modern stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, LEGACY).expect("compile XPath 1.0 coercion");
    let modern = compile_resource(&snapshot, MODERN).expect_err("modern coercion stays rejected");
    assert_eq!(modern.code, "FXXP1019");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("xpath10-coercion", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute XPath 1.0 coercion");
    assert_eq!(
        results.by_request["xpath10-coercion"].serialized,
        "true|true|true|true|true|true|true|false|true"
    );
}

#[test]
fn source_free_literal_boolean_composition_uses_shared_modern_semantics() {
    const SOURCE: &str = "urn:fastxslt:literal-boolean-composition:source";
    const STYLESHEET: &str = "urn:fastxslt:literal-boolean-composition:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br"<doc/>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="'foo' and 'fop'"/>|<xsl:value-of select="'1' and '0'"/>|<xsl:value-of select="0 or ''"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile boolean composition");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("literal-boolean-composition", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute boolean composition");
    assert_eq!(
        results.by_request["literal-boolean-composition"].serialized,
        "true|true|false"
    );
}

#[test]
fn xpath10_non_finite_literal_division_is_selected_at_compilation() {
    const SOURCE: &str = "urn:fastxslt:xpath10-non-finite:source";
    const LEGACY: &str = "urn:fastxslt:xpath10-non-finite:legacy";
    const MODERN: &str = "urn:fastxslt:xpath10-non-finite:modern";
    let body = r#"<xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="1 div 0"/>|<xsl:value-of select="-1 div 0"/>|<xsl:value-of select="0 div 0"/>|<xsl:value-of select="boolean(1 div 0)"/>|<xsl:value-of select="boolean(0 div 0)"/></xsl:template>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(3, 8_192, 16_384));
    resources
        .admit(SOURCE, br"<doc/>".to_vec())
        .expect("admit source");
    resources
        .admit(
            LEGACY,
            format!(r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">{body}</xsl:stylesheet>"#).into_bytes(),
        )
        .expect("admit legacy stylesheet");
    resources
        .admit(
            MODERN,
            format!(r#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">{body}</xsl:stylesheet>"#).into_bytes(),
        )
        .expect("admit modern stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, LEGACY).expect("compile XPath 1.0 division");
    compile_resource(&snapshot, MODERN).expect_err("modern decimal division by zero stays invalid");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("xpath10-non-finite", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute XPath 1.0 division");
    assert_eq!(
        results.by_request["xpath10-non-finite"].serialized,
        "Infinity|-Infinity|NaN|true|false"
    );
}

#[test]
fn xpath10_ordered_literal_comparison_is_selected_at_compilation() {
    const SOURCE: &str = "urn:fastxslt:xpath10-ordered-literal:source";
    const LEGACY: &str = "urn:fastxslt:xpath10-ordered-literal:legacy";
    const MODERN: &str = "urn:fastxslt:xpath10-ordered-literal:modern";
    let body = r#"<xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="'2' &gt; '1'"/>|<xsl:value-of select="'10' &gt; '2'"/>|<xsl:value-of select="2 &lt; '10'"/>|<xsl:value-of select="'bad' &lt; 4"/></xsl:template>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(3, 8_192, 16_384));
    resources
        .admit(SOURCE, br"<doc/>".to_vec())
        .expect("admit source");
    resources
        .admit(
            LEGACY,
            format!(r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">{body}</xsl:stylesheet>"#).into_bytes(),
        )
        .expect("admit legacy stylesheet");
    resources
        .admit(
            MODERN,
            format!(r#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">{body}</xsl:stylesheet>"#).into_bytes(),
        )
        .expect("admit modern stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, LEGACY).expect("compile XPath 1.0 comparison");
    compile_resource(&snapshot, MODERN).expect_err("modern mixed comparison stays rejected");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("xpath10-ordered-literal", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute XPath 1.0 comparison");
    assert_eq!(
        results.by_request["xpath10-ordered-literal"].serialized,
        "true|true|true|false"
    );
}

#[test]
fn binary_numeric_paths_share_execution_with_compiled_cardinality_policy() {
    const SOURCE: &str = "urn:fastxslt:binary-numeric:source";
    const LEGACY: &str = "urn:fastxslt:binary-numeric:legacy";
    const MODERN: &str = "urn:fastxslt:binary-numeric:modern";
    let body = r#"<xsl:output method="text"/><xsl:template match="/"><xsl:apply-templates select="doc"/></xsl:template><xsl:template match="doc"><xsl:value-of select="n1+n2"/>|<xsl:value-of select="(n1/@attrib)*(n2/@attrib)"/>|<xsl:value-of select="n-2 - n-1"/>|<xsl:value-of select="div div mod"/>|<xsl:value-of select="n-2+-n-1"/>|<xsl:value-of select="n-2 - -n-1"/>|<xsl:value-of select="-n-2 --n-1"/>|<xsl:value-of select="-n-2/@attrib --n-1/@attrib"/>|<xsl:value-of select="-(n-2/@attrib) - -(n-1/@attrib)"/>|<xsl:value-of select="n-2 mod n-1"/>|<xsl:value-of select="div mod mod"/>|<xsl:value-of select="n1*n2*n3"/>|<xsl:value-of select="n0 div n1 div n2"/>|<xsl:value-of select="(n0*n1) div n2"/>|<xsl:value-of select="n1+5"/>|<xsl:value-of select="(n1+5)*3"/>|<xsl:value-of select="-(4-6)"/>|<xsl:value-of select="n1*.5"/></xsl:template>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(3, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            br"<doc><n0>36</n0><n1 attrib='5'>3</n1><n1 attrib='100'>100</n1><n2 attrib='5'>6</n2><n3>2</n3><n-1 attrib='9'>3</n-1><n-2 attrib='1'>7</n-2><div>8</div><mod>4</mod></doc>"
                .to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            LEGACY,
            format!(r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">{body}</xsl:stylesheet>"#).into_bytes(),
        )
        .expect("admit legacy stylesheet");
    resources
        .admit(
            MODERN,
            format!(r#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform">{body}</xsl:stylesheet>"#).into_bytes(),
        )
        .expect("admit modern stylesheet");
    let snapshot = resources.seal();
    let legacy = compile_resource(&snapshot, LEGACY).expect("compile XPath 1.0 arithmetic");
    let modern = compile_resource(&snapshot, MODERN).expect("compile modern arithmetic");

    let mut legacy_builder = TransformSetBuilder::new(snapshot.clone(), legacy, 1, policy(4_096));
    legacy_builder
        .add(request("legacy-binary", "legacy-result", SOURCE))
        .expect("admit legacy request");
    let results = execute_transform_set(legacy_builder.seal()).expect("execute legacy arithmetic");
    assert_eq!(
        results.by_request["legacy-binary"].serialized,
        "9|25|4|2|4|10|-4|8|8|1|0|36|2|18|8|24|2|1.5"
    );

    let mut modern_builder = TransformSetBuilder::new(snapshot, modern, 1, policy(4_096));
    modern_builder
        .add(request("modern-binary", "modern-result", SOURCE))
        .expect("admit modern request");
    let failure = execute_transform_set(modern_builder.seal())
        .expect_err("modern path cardinality remains enforced");
    assert_eq!(failure.code, "XPTY0004");
}

#[test]
fn xpath_constant_integral_numeric_expressions_fold_exact_results() {
    const SOURCE: &str = "urn:fastxslt:integral-functions:source";
    const STYLESHEET: &str = "urn:fastxslt:integral-functions:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            br"<doc><low>-1.5</low><high>2.999999</high></doc>".to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="floor(1.9)"/>|<xsl:value-of select="floor(-1.5)"/>|<xsl:value-of select="ceiling(1.1)"/>|<xsl:value-of select="ceiling(-1.5)"/>|<xsl:value-of select="round(2.5)"/>|<xsl:value-of select="round(-2.5)"/>|<xsl:value-of select="floor(1.9)=1"/>|<xsl:value-of select="round(-1.5)=-1"/>|<xsl:value-of select="floor(doc/low)"/>|<xsl:value-of select="ceiling(doc/high)"/>|<xsl:value-of select="round(doc/high)"/>|<xsl:value-of select="floor(doc/missing)"/>|<xsl:value-of select="2*3"/>|<xsl:value-of select="7 - -3"/>|<xsl:value-of select="6 div -2"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile exact integral functions");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("integral-functions", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute integral functions");
    assert_eq!(
        results.by_request["integral-functions"].serialized,
        "1|-2|2|-1|3|-2|true|true|-2|3|3||6|10|-3"
    );
}

#[test]
fn xpath_number_conversion_handles_finite_and_nan_results() {
    const SOURCE: &str = "urn:fastxslt:number-conversion:source";
    const STYLESHEET: &str = "urn:fastxslt:number-conversion:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            br"<doc><n>04.2500</n><invalid>not-a-number</invalid></doc>".to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="number(2)"/>|<xsl:value-of select="number('003.500')"/>|<xsl:value-of select="number(-0)"/>|<xsl:value-of select="number(doc/n)"/>|<xsl:value-of select="number(doc/missing)"/>|<xsl:value-of select="number(doc/invalid)"/>|<xsl:value-of select="number()"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program =
        compile_resource(&snapshot, STYLESHEET).expect("compile finite number conversions");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("number-conversion", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute number conversions");
    assert_eq!(
        results.by_request["number-conversion"].serialized,
        "2|3.5|0|4.25|NaN|NaN|NaN"
    );
}

#[test]
fn xpath_number_path_rejects_more_than_one_node() {
    const SOURCE: &str = "urn:fastxslt:number-many:source";
    const STYLESHEET: &str = "urn:fastxslt:number-many:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br"<doc><n>1</n><n>2</n></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="number(doc/n)"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile number path");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("number-many", "result", SOURCE))
        .expect("admit request");

    let failure = execute_transform_set(builder.seal()).expect_err("cardinality must be enforced");
    assert_eq!(failure.code, "XPTY0004");
    assert_eq!(failure.category, FailureCategory::Invalid);
}

#[test]
fn xpath_integral_path_rejects_more_than_one_node() {
    const SOURCE: &str = "urn:fastxslt:integral-many:source";
    const STYLESHEET: &str = "urn:fastxslt:integral-many:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br"<doc><n>1</n><n>2</n></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="floor(doc/n)"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile integral path");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("integral-many", "result", SOURCE))
        .expect("admit request");

    let failure = execute_transform_set(builder.seal()).expect_err("cardinality must be enforced");
    assert_eq!(failure.code, "XPTY0004");
    assert_eq!(failure.category, FailureCategory::Invalid);
}

#[test]
fn xpath_concat_folds_bounded_static_atomic_arguments() {
    const SOURCE: &str = "urn:fastxslt:static-concat:source";
    const STYLESHEET: &str = "urn:fastxslt:static-concat:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br"<doc/>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="concat('a,b', false(), string(34), 'c')"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile static concat");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("static-concat", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute static concat");
    assert_eq!(
        results.by_request["static-concat"].serialized,
        "a,bfalse34c"
    );
}

#[test]
fn xpath_static_string_functions_preserve_typed_results() {
    const SOURCE: &str = "urn:fastxslt:static-string-functions:source";
    const STYLESHEET: &str = "urn:fastxslt:static-string-functions:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br"<doc/>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            r#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="contains('ENCYCLOPEDIA', 'CYCL')"/>|<xsl:value-of select="starts-with('abc', '')"/>|<xsl:value-of select="substring-before('1999/04/01', '/')"/>|<xsl:value-of select="substring-after('é😀', 'é')"/>|<xsl:value-of select="translate('zzaaazzz', 'abcz', 'ABC')"/>|<xsl:value-of select="substring('é😀x', 2, 1)"/></xsl:template></xsl:stylesheet>"#.as_bytes().to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile static string functions");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("static-string-functions", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute static string functions");
    assert_eq!(
        results.by_request["static-string-functions"].serialized,
        "true|true|1999|😀|AAA|😀"
    );
}

#[test]
fn xpath_string_function_composes_static_atoms_and_typed_paths() {
    const SOURCE: &str = "urn:fastxslt:string-function:source";
    const STYLESHEET: &str = "urn:fastxslt:string-function:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br"<doc>alpha<part>beta</part></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="string(0)"/>|<xsl:value-of select="string('test')"/>|<xsl:value-of select="string(doc)"/>|<xsl:value-of select="string(doc/missing)"/>|<xsl:value-of select="string(boolean(0))"/>|<xsl:value-of select="string(boolean(1))"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile string function");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("string-function", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute string function");
    assert_eq!(
        results.by_request["string-function"].serialized,
        "0|test|alphabeta||false|true"
    );
}

#[test]
fn xpath_string_path_rejects_more_than_one_node() {
    const SOURCE: &str = "urn:fastxslt:string-many:source";
    const STYLESHEET: &str = "urn:fastxslt:string-many:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br"<doc><n>one</n><n>two</n></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:value-of select="string(doc/n)"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile string path");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("string-many", "result", SOURCE))
        .expect("admit request");

    let failure = execute_transform_set(builder.seal()).expect_err("cardinality must be enforced");
    assert_eq!(failure.code, "XPTY0004");
    assert_eq!(failure.category, FailureCategory::Invalid);
}

#[test]
fn xpath_lang_uses_the_nearest_inherited_xml_language() {
    const SOURCE: &str = "urn:fastxslt:lang:source";
    const STYLESHEET: &str = "urn:fastxslt:lang:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(
            SOURCE,
            br#"<doc xml:lang="EN-us"><p id="1">P</p><q id="2" xml:lang="fr">Q</q></doc>"#.to_vec(),
        )
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="text"/><xsl:template match="/"><xsl:value-of select="doc/*[@id='1' and lang('en')]"/>|<xsl:apply-templates select="doc/*"/></xsl:template><xsl:template match="p"><xsl:if test="lang('en')">yes:</xsl:if><xsl:value-of select="lang('en')"/>|</xsl:template><xsl:template match="q"><xsl:if test="lang('en')">wrong:</xsl:if><xsl:value-of select="lang('en')"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile lang function");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("lang", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute lang function");
    assert_eq!(results.by_request["lang"].serialized, "P|yes:true|false");
}

#[test]
fn default_selection_uses_built_in_element_and_text_rules() {
    const BUILT_IN_SOURCE: &str = "urn:fastxslt:golden:built-in-rules:source";
    const BUILT_IN_STYLESHEET: &str = "urn:fastxslt:golden:built-in-rules:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(4, 4_096, 8_192));
    resources
        .admit(
            BUILT_IN_SOURCE,
            include_bytes!("../../../../corpus/golden/built-in-template-rules/input.xml").to_vec(),
        )
        .expect("admit built-in-rule source");
    resources
        .admit(
            BUILT_IN_STYLESHEET,
            include_bytes!("../../../../corpus/golden/built-in-template-rules/stylesheet.xsl")
                .to_vec(),
        )
        .expect("admit built-in-rule stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, BUILT_IN_STYLESHEET)
        .expect("compile built-in-rule stylesheet once");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request(
            "built-in-request",
            "built-in-result",
            BUILT_IN_SOURCE,
        ))
        .expect("add built-in-rule request");

    let results = execute_transform_set(builder.seal()).expect("execute built-in-rule set");

    assert_eq!(
        results.by_request["built-in-request"].serialized,
        include_str!("../../../../corpus/golden/built-in-template-rules/expected.xml").trim()
    );
}

#[test]
fn unmatched_attributes_use_the_built_in_string_value_rule() {
    const SOURCE: &str = "urn:fastxslt:built-in-attribute:source";
    const STYLESHEET: &str = "urn:fastxslt:built-in-attribute:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, br#"<root value="kept"/>"#.to_vec())
        .expect("admit attribute source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="root/@value"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit attribute stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile attribute stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("attribute-request", "attribute-result", SOURCE))
        .expect("add attribute request");

    let results = execute_transform_set(builder.seal()).expect("execute attribute request");
    assert_eq!(
        results.by_request["attribute-request"].serialized,
        "<out>kept</out>"
    );
}

#[test]
fn apply_templates_executes_a_convergent_path_node_once() {
    const SOURCE: &str = "urn:fastxslt:path-normalization:source";
    const STYLESHEET: &str = "urn:fastxslt:path-normalization:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<r><a/><a/></r>".to_vec())
        .expect("admit convergent source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="/r/a/.."/></out></xsl:template><xsl:template match="r"><hit/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit convergent stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile convergent stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("convergent", "result", SOURCE))
        .expect("admit convergent request");

    let results = execute_transform_set(builder.seal()).expect("execute convergent request");

    assert_eq!(
        results.by_request["convergent"].serialized,
        "<out><hit></hit></out>"
    );
}

#[test]
fn isolated_descendant_copies_retain_required_namespace_bindings() {
    const SOURCE: &str = "urn:fastxslt:namespace-fixup:source";
    const STYLESHEET: &str = "urn:fastxslt:namespace-fixup:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            br#"<root xmlns:p="urn:example"><p:item/></root>"#.to_vec(),
        )
        .expect("admit namespaced descendant source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:example" exclude-result-prefixes="p"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><xsl:apply-templates/></xsl:template><xsl:template match="root"><out xsl:xpath-default-namespace="urn:example"><xsl:apply-templates select="item"/></out></xsl:template><xsl:template match="p:item"><deep><xsl:copy-of select="."/></deep><shallow><xsl:copy/></shallow></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit namespace-copy stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile namespace copies");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(8_192));
    builder
        .add(request("namespace-fixup", "result", SOURCE))
        .expect("admit namespace-copy request");

    let results = execute_transform_set(builder.seal()).expect("execute namespace copies");

    assert_eq!(
        results.by_request["namespace-fixup"].serialized,
        "<out><deep><p:item xmlns:p=\"urn:example\"></p:item></deep><shallow><p:item xmlns:p=\"urn:example\"></p:item></shallow></out>"
    );
}

#[test]
fn shallow_copy_preserves_comments_and_attribute_template_results() {
    const SOURCE: &str = "urn:fastxslt:shallow-copy-boundary:source";
    const STYLESHEET: &str = "urn:fastxslt:shallow-copy-boundary:stylesheet";
    let cases: [(&[u8], &[u8], &str); 2] = [
        (
            b"<root><!--retained source comment--></root>",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:mode on-no-match="shallow-copy"/></xsl:stylesheet>"#,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><root><!--retained source comment--></root>",
        ),
        (
            br#"<root code="intercepted"/>"#,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:mode on-no-match="shallow-copy"/><xsl:template match="@code"><xsl:value-of select="."/></xsl:template></xsl:stylesheet>"#,
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?><root>intercepted</root>",
        ),
    ];

    for (source, stylesheet, expected) in cases {
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
        resources
            .admit(SOURCE, source.to_vec())
            .expect("admit shallow-copy boundary source");
        resources
            .admit(STYLESHEET, stylesheet.to_vec())
            .expect("admit shallow-copy boundary stylesheet");
        let snapshot = resources.seal();
        let program =
            compile_resource(&snapshot, STYLESHEET).expect("compile bounded shallow-copy policy");
        let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
        builder
            .add(request("shallow-copy", "result", SOURCE))
            .expect("admit shallow-copy boundary request");

        let results = execute_transform_set(builder.seal())
            .expect("represented shallow-copy result path must execute");
        assert_eq!(results.by_request["shallow-copy"].serialized, expected);
    }
}

#[test]
fn mode_owned_multiple_match_failure_overrides_host_recovery() {
    const SOURCE: &str = "urn:fastxslt:mode-multiple-match:source";
    const STYLESHEET: &str = "urn:fastxslt:mode-multiple-match:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><para>text<foo/></para></doc>".to_vec())
        .expect("admit ambiguous mode source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:mode name="c" on-multiple-match="fail"/><xsl:template match="/" mode="c"><xsl:apply-templates mode="c"/></xsl:template><xsl:template match="para[foo]" mode="c"><a/></xsl:template><xsl:template match="para[text()]" mode="c"><b/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit ambiguous mode stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile mode-owned fail policy");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096))
        .with_multiple_match_policy(MultipleMatchPolicy::UseLast);
    builder
        .add(TransformRequest {
            identity: "ambiguous-mode".to_owned(),
            result_identity: "ambiguous-mode-result".to_owned(),
            entry: InvocationEntry::InitialMode {
                resource: SOURCE.to_owned(),
                name: "c".to_owned(),
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect("admit ambiguous mode request");

    let failure = execute_transform_set(builder.seal())
        .expect_err("mode-owned fail policy must reject distinct equal-rank rules");
    assert_eq!(failure.code, "XTDE0540");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(failure.request_id.as_deref(), Some("ambiguous-mode"));
    assert!(failure.location.is_some());
}

#[test]
fn named_template_recursion_stops_at_the_private_depth_limit() {
    const SOURCE: &str = "urn:fastxslt:recursion:source";
    const STYLESHEET: &str = "urn:fastxslt:recursion:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc/>".to_vec())
        .expect("admit recursion source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="loop"><xsl:call-template name="loop"/></xsl:template><xsl:template match="/"><xsl:call-template name="loop"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit recursive stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile recursive stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("recursive", "recursive-result", SOURCE))
        .expect("admit recursive request");

    let failure = execute_transform_set(builder.seal())
        .expect_err("recursive call chain must stop at the private depth limit");

    assert_eq!(failure.code, "FXRT0003");
    assert_eq!(failure.category, FailureCategory::Limit);
    assert_eq!(failure.request_id.as_deref(), Some("recursive"));
}

#[test]
fn named_template_parameters_apply_defaults_and_atomic_select_arguments() {
    const SOURCE: &str = "urn:fastxslt:named-template-parameters:source";
    const STYLESHEET: &str = "urn:fastxslt:named-template-parameters:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc/>".to_vec())
        .expect("admit parameter source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:call-template name="emit"/><xsl:call-template name="emit"><xsl:with-param name="value" select="7"/></xsl:call-template><xsl:call-template name="emit"><xsl:with-param name="value" select="'literal'"/></xsl:call-template><xsl:call-template name="emit"><xsl:with-param name="value" select="true()"/></xsl:call-template><xsl:call-template name="emit"><xsl:with-param name="value" select="position()"/></xsl:call-template><xsl:call-template name="emit"><xsl:with-param name="value" select="last()"/></xsl:call-template></out></xsl:template><xsl:template name="emit">
                <xsl:param name="value" select="5"/>
                <xsl:value-of select="$value"/>
            </xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit parameter stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile parameter stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("parameters", "parameters-result", SOURCE))
        .expect("admit parameter request");

    let results = execute_transform_set(builder.seal()).expect("execute parameter calls");

    assert_eq!(
        results.by_request["parameters"].serialized,
        "<out>57literaltrue11</out>"
    );
}

#[test]
fn node_template_parameter_shadows_same_named_global_atomic_in_both_frame_paths() {
    const SOURCE: &str = "urn:fastxslt:parameter-shadow:source";
    const STYLESHEET: &str = "urn:fastxslt:parameter-shadow:stylesheet";
    let source_bytes = b"<doc><a>node</a></doc>";
    let stylesheet = br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:variable name="value">global</xsl:variable><xsl:template match="/"><xsl:call-template name="emit"><xsl:with-param name="value" select="doc/a"/></xsl:call-template></xsl:template><xsl:template name="emit"><xsl:param name="value" select="0"/><out><xsl:value-of select="$value"/></out></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, source_bytes.to_vec())
        .expect("admit parameter-shadow source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit parameter-shadow stylesheet");
    let program = compile_resource(&resources.seal(), STYLESHEET)
        .expect("compile parameter-shadow stylesheet");
    let parsed = parse_document(
        SOURCE,
        source_bytes,
        ParseLimits {
            max_events: 32,
            max_depth: 8,
        },
    )
    .expect("parse parameter-shadow source");
    let source = Document::from_parsed(parsed).expect("prepare parameter-shadow source");

    for mut control in [
        InvocationControl::unbounded(),
        InvocationControl::unbounded().with_complete_atomic_frame_clones(),
    ] {
        let result = execute_program_with_parameters_using(
            &program,
            &source,
            &BTreeMap::new(),
            MultipleMatchPolicy::UseLast,
            "parameter-shadow",
            WhitespaceRepresentation::VisibilityView,
            None,
            None,
            &mut control,
        )
        .expect("execute parameter-shadow stylesheet");
        assert_eq!(
            result.children,
            [ResultNode::Element {
                name: ExpandedName {
                    namespace: None,
                    local: "out".to_owned(),
                },
                namespaces: Vec::new().into(),
                attributes: Vec::new(),
                children: vec![ResultNode::Text("node".to_owned())],
            }]
        );
    }
}

#[test]
fn batch_of_one_matches_the_same_semantic_and_serialization_path() {
    let snapshot = snapshot();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("only", "only-result", SOURCE_ID))
        .expect("add request");

    let results = execute_transform_set(builder.seal()).expect("execute one");

    assert_eq!(results.completion_order, ["only"]);
    assert_eq!(
        results.by_request["only"].serialized,
        "<message>Hello, FastXSLT!</message>"
    );
}

#[test]
fn initial_mode_uses_a_source_and_rejects_unknown_compiled_identity() {
    const MODE_STYLESHEET: &str = "urn:fastxslt:initial-mode:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE_ID, br"<doc/>".to_vec())
        .expect("admit initial-mode source");
    resources
        .admit(
            MODE_STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/" mode="audit"><out>mode</out></xsl:template><xsl:template match="doc" mode="audit"><out>element</out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit initial-mode stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, MODE_STYLESHEET).expect("compile initial mode");
    let mut missing = TransformSetBuilder::new(snapshot.clone(), program.clone(), 1, policy(4_096));
    missing
        .add(TransformRequest {
            identity: "missing-element".to_owned(),
            result_identity: "missing-element-result".to_owned(),
            entry: InvocationEntry::InitialModeElement {
                resource: SOURCE_ID.to_owned(),
                name: "audit".to_owned(),
                element: ExpandedName {
                    namespace: None,
                    local: "missing".to_owned(),
                },
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect("known mode and admitted source should pass admission");
    let failure = execute_transform_set(missing.seal())
        .expect_err("missing initial context element should fail execution");
    assert_eq!(failure.code, "FXRT0005");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(failure.request_id.as_deref(), Some("missing-element"));

    let mut builder = TransformSetBuilder::new(snapshot, program, 2, policy(4_096));

    let failure = builder
        .add(TransformRequest {
            identity: "unknown-mode".to_owned(),
            result_identity: "unknown-mode-result".to_owned(),
            entry: InvocationEntry::InitialMode {
                resource: SOURCE_ID.to_owned(),
                name: "missing".to_owned(),
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect_err("unknown initial mode should fail request admission");
    assert_eq!(failure.code, "XTDE0045");
    assert_eq!(failure.category, FailureCategory::Invalid);

    builder
        .add(TransformRequest {
            identity: "known-mode".to_owned(),
            result_identity: "known-mode-result".to_owned(),
            entry: InvocationEntry::InitialMode {
                resource: SOURCE_ID.to_owned(),
                name: "audit".to_owned(),
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect("failed admission must not poison the builder");
    builder
        .add(TransformRequest {
            identity: "element-mode".to_owned(),
            result_identity: "element-mode-result".to_owned(),
            entry: InvocationEntry::InitialModeElement {
                resource: SOURCE_ID.to_owned(),
                name: "audit".to_owned(),
                element: ExpandedName {
                    namespace: None,
                    local: "doc".to_owned(),
                },
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect("admit element initial context");
    let results = execute_transform_set(builder.seal()).expect("execute initial mode");
    assert_eq!(
        results.by_request["known-mode"].serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>mode</out>"
    );
    assert_eq!(
        results.by_request["element-mode"].serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>element</out>"
    );
}

#[test]
fn invocation_parameters_override_global_defaults_without_cross_request_state() {
    const PARAMETER_STYLESHEET: &str = "urn:fastxslt:invocation-parameter:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE_ID, br"<doc/>".to_vec())
        .expect("admit invocation-parameter source");
    resources
        .admit(
            PARAMETER_STYLESHEET,
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:param name="message">default</xsl:param><xsl:template match="/"><out><xsl:value-of select="$message"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit invocation-parameter stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, PARAMETER_STYLESHEET)
        .expect("compile invocation-parameter stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 2, policy(4_096));

    builder
        .add(request("default", "default-result", SOURCE_ID))
        .expect("admit defaulted request");
    let mut overridden = request("overridden", "overridden-result", SOURCE_ID);
    overridden.parameters.insert(
        "message".to_owned(),
        InvocationParameter {
            value: AtomicValue::string("host supplied").into(),
            tunnel: false,
        },
    );
    builder
        .add(overridden)
        .expect("admit parameterized request");

    let results = execute_transform_set(builder.seal()).expect("execute parameterized set");
    assert_eq!(
        results.by_request["default"].serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>default</out>"
    );
    assert_eq!(
        results.by_request["overridden"].serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>host supplied</out>"
    );
}

#[test]
fn absent_output_declaration_does_not_silently_apply_html_serialization() {
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: "html".to_owned(),
            },
            namespaces: Vec::new().into(),
            attributes: Vec::new(),
            children: Vec::new(),
        }],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: None,
        version: None,
        html_version: None,
        encoding: None,
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: false,
        indent: None,
    };

    let mut control = InvocationControl::unbounded();
    let failure = serialize_xml(&result, &settings, "html-result", 4_096, &mut control)
        .expect_err("adaptive HTML output remains unsupported");

    assert_eq!(failure.code, "FXSR1001");
    assert_eq!(failure.category, FailureCategory::Unsupported);
    assert_eq!(failure.request_id.as_deref(), Some("html-result"));
}

#[test]
fn legacy_html_serialization_recognizes_uppercase_script_and_void_elements() {
    let element = |local: &str, children: Vec<ResultNode>| ResultNode::Element {
        name: crate::xml::quick_xml_experiment::ExpandedName {
            namespace: None,
            local: local.to_owned(),
        },
        namespaces: Vec::new().into(),
        attributes: Vec::new(),
        children,
    };
    let result = SemanticResult {
        children: vec![element(
            "HTML",
            vec![
                element(
                    "HEAD",
                    vec![element(
                        "SCRIPT",
                        vec![ResultNode::Text("if (a < b && c > d) {}".to_owned())],
                    )],
                ),
                element("BODY", Vec::new()),
            ],
        )],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("html".to_owned()),
        version: None,
        html_version: None,
        encoding: None,
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: false,
        indent: None,
    };

    let mut control = InvocationControl::unbounded();
    let actual = serialize_xml(&result, &settings, "uppercase-html", 4_096, &mut control)
        .expect("serialize bounded uppercase HTML");

    assert_eq!(
        actual,
        "<HTML><HEAD><meta http-equiv=\"Content-Type\" content=\"text/html; charset=UTF-8\"><SCRIPT>if (a < b && c > d) {}</SCRIPT></HEAD><BODY></BODY></HTML>"
    );
}

#[test]
fn legacy_html_serialization_admits_one_bare_uri_link() {
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: "HTML".to_owned(),
            },
            namespaces: Vec::new().into(),
            attributes: Vec::new(),
            children: vec![ResultNode::Element {
                name: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: "a".to_owned(),
                },
                namespaces: Vec::new().into(),
                attributes: vec![ResultAttribute {
                    name: crate::xml::quick_xml_experiment::ExpandedName {
                        namespace: None,
                        local: "href".to_owned(),
                    },
                    value: "/cgi-bin/app?p_parm1=Out1".to_owned(),
                }],
                children: Vec::new(),
            }],
        }],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("html".to_owned()),
        version: None,
        html_version: None,
        encoding: None,
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: false,
        indent: None,
    };

    let mut control = InvocationControl::unbounded();
    let actual = serialize_xml(&result, &settings, "bare-html-link", 4_096, &mut control)
        .expect("serialize bounded bare link document");

    assert_eq!(
        actual,
        "<HTML><a href=\"/cgi-bin/app?p_parm1=Out1\"></a></HTML>"
    );
}

#[test]
fn xsl_copy_preserves_a_source_comment_as_the_current_node() {
    const SOURCE: &str = "urn:fastxslt:copy-comment:source";
    const STYLESHEET: &str = "urn:fastxslt:copy-comment:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><!--first--><!--second--></doc>".to_vec())
        .expect("admit source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:for-each select="//comment()"><xsl:copy/></xsl:for-each></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile comment copy");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("copy-comment", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute comment copy");

    assert_eq!(
        results.by_request["copy-comment"].serialized,
        "<out><!--first--><!--second--></out>"
    );
}

#[test]
fn source_copy_computed_attribute_reads_the_current_source_attribute() {
    const SOURCE: &str = "urn:fastxslt:source-copy-attribute:source";
    const STYLESHEET: &str = "urn:fastxslt:source-copy-attribute:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:for-each select="doc/item"><xsl:copy><xsl:attribute name="copied"><xsl:value-of select="@source"/></xsl:attribute></xsl:copy></xsl:for-each></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(
            SOURCE,
            b"<doc><item source=\"first\"/><item/></doc>".to_vec(),
        )
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile source-copy attribute");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request(
            "source-copy-attribute",
            "source-copy-attribute-result",
            SOURCE,
        ))
        .expect("admit request");

    let results =
        execute_transform_set(builder.seal()).expect("execute source-copy attribute request");

    assert_eq!(
        results.by_request["source-copy-attribute"].serialized,
        "<out><item copied=\"first\"></item><item copied=\"\"></item></out>"
    );
}

#[test]
fn xslt10_numeric_sort_uses_xpath_number_lexicals_and_equal_zero_keys() {
    const SOURCE: &str = "urn:fastxslt:xslt10-numeric-sort:source";
    const STYLESHEET: &str = "urn:fastxslt:xslt10-numeric-sort:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:for-each select="doc/n"><xsl:sort data-type="number"/><xsl:value-of select="."/><xsl:text>|</xsl:text></xsl:for-each></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    resources
        .admit(
            SOURCE,
            b"<doc><n>+16</n><n>+15</n><n>0</n><n>-0</n></doc>".to_vec(),
        )
        .expect("admit source");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile XSLT 1.0 numeric sort");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request(
            "xslt10-numeric-sort",
            "numeric-sort-result",
            SOURCE,
        ))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute XSLT 1.0 numeric sort");

    assert_eq!(
        results.by_request["xslt10-numeric-sort"].serialized,
        "<out>+16|+15|0|-0|</out>"
    );
}

#[test]
fn absent_method_selects_xhtml_for_an_xhtml_html_document_element() {
    let xhtml_name = |local: &str| crate::xml::quick_xml_experiment::ExpandedName {
        namespace: Some("http://www.w3.org/1999/xhtml".to_owned()),
        local: local.to_owned(),
    };
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: xhtml_name("html"),
            namespaces: vec![crate::xml::quick_xml_experiment::NamespaceBinding {
                prefix: None,
                namespace: "http://www.w3.org/1999/xhtml".to_owned(),
            }]
            .into(),
            attributes: Vec::new(),
            children: vec![ResultNode::Element {
                name: xhtml_name("br"),
                namespaces: Vec::new().into(),
                attributes: Vec::new(),
                children: Vec::new(),
            }],
        }],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: None,
        version: None,
        html_version: None,
        encoding: None,
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: false,
        indent: None,
    };

    let serialized = serialize_xml(
        &result,
        &settings,
        "inferred-xhtml",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("infer XHTML from the expanded name of the document element");

    assert_eq!(
        serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><html xmlns=\"http://www.w3.org/1999/xhtml\"><br /></html>"
    );
}

#[test]
fn requested_indentation_formats_only_element_only_child_sequences() {
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: "out".to_owned(),
            },
            namespaces: Vec::new().into(),
            attributes: Vec::new(),
            children: vec![
                ResultNode::Element {
                    name: crate::xml::quick_xml_experiment::ExpandedName {
                        namespace: None,
                        local: "group".to_owned(),
                    },
                    namespaces: Vec::new().into(),
                    attributes: Vec::new(),
                    children: vec![ResultNode::Element {
                        name: crate::xml::quick_xml_experiment::ExpandedName {
                            namespace: None,
                            local: "item".to_owned(),
                        },
                        namespaces: Vec::new().into(),
                        attributes: Vec::new(),
                        children: vec![ResultNode::Text("value".to_owned())],
                    }],
                },
                ResultNode::Element {
                    name: crate::xml::quick_xml_experiment::ExpandedName {
                        namespace: None,
                        local: "mixed".to_owned(),
                    },
                    namespaces: Vec::new().into(),
                    attributes: Vec::new(),
                    children: vec![
                        ResultNode::Text("left".to_owned()),
                        ResultNode::Element {
                            name: crate::xml::quick_xml_experiment::ExpandedName {
                                namespace: None,
                                local: "em".to_owned(),
                            },
                            namespaces: Vec::new().into(),
                            attributes: Vec::new(),
                            children: Vec::new(),
                        },
                        ResultNode::Text("right".to_owned()),
                    ],
                },
            ],
        }],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("xml".to_owned()),
        version: None,
        html_version: None,
        encoding: None,
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: false,
        indent: Some(true),
    };

    let mut control = InvocationControl::unbounded();
    let serialized = serialize_xml(&result, &settings, "indented-result", 4_096, &mut control)
        .expect("bounded element-only indentation should serialize");

    assert_eq!(
        serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>\n  <group>\n    <item>value</item>\n  </group>\n  <mixed>left<em></em>right</mixed>\n</out>"
    );
}

#[test]
fn xhtml_content_type_replaces_an_existing_meta_without_mutating_result_content() {
    let xhtml_name = |local: &str| crate::xml::quick_xml_experiment::ExpandedName {
        namespace: Some("http://www.w3.org/1999/xhtml".to_owned()),
        local: local.to_owned(),
    };
    let attribute = |local: &str, value: &str| ResultAttribute {
        name: crate::xml::quick_xml_experiment::ExpandedName {
            namespace: None,
            local: local.to_owned(),
        },
        value: value.to_owned(),
    };
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: xhtml_name("head"),
            namespaces: vec![crate::xml::quick_xml_experiment::NamespaceBinding {
                prefix: None,
                namespace: "http://www.w3.org/1999/xhtml".to_owned(),
            }]
            .into(),
            attributes: Vec::new(),
            children: vec![
                ResultNode::Element {
                    name: xhtml_name("meta"),
                    namespaces: Vec::new().into(),
                    attributes: vec![
                        attribute("http-equiv", "Content-Type"),
                        attribute("media-type", "stale/type"),
                    ],
                    children: Vec::new(),
                },
                ResultNode::Text("authored head text".to_owned()),
            ],
        }],
    };
    let mut settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("xhtml".to_owned()),
        version: None,
        html_version: None,
        encoding: Some("UTF-8".to_owned()),
        media_type: Some("application/example+xml".to_owned()),
        doctype_system: None,
        doctype_public: None,
        include_content_type: Some(true),
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: true,
        indent: Some(false),
    };

    let serialized = serialize_xml(
        &result,
        &settings,
        "xhtml-content-type",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("replace XHTML content-type metadata");
    assert_eq!(
        serialized,
        "<head xmlns=\"http://www.w3.org/1999/xhtml\"><meta http-equiv=\"Content-Type\" content=\"application/example+xml; charset=UTF-8\" />authored head text</head>"
    );

    settings.include_content_type = Some(false);
    let retained = serialize_xml(
        &result,
        &settings,
        "xhtml-content-type-disabled",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("retain authored metadata when content-type handling is disabled");
    assert!(retained.contains("media-type=\"stale/type\""));
    assert!(!retained.contains("content=\"application/example+xml"));
}

#[test]
fn serializer_uses_the_predefined_xml_prefix_without_a_namespace_declaration() {
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: "out".to_owned(),
            },
            namespaces: Vec::new().into(),
            attributes: vec![ResultAttribute {
                name: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: Some("http://www.w3.org/XML/1998/namespace".to_owned()),
                    local: "lang".to_owned(),
                },
                value: "en".to_owned(),
            }],
            children: Vec::new(),
        }],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("xml".to_owned()),
        version: None,
        html_version: None,
        encoding: None,
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: true,
        indent: Some(false),
    };

    let serialized = serialize_xml(
        &result,
        &settings,
        "predefined-xml-prefix",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("serialize an XML-namespaced attribute without an authored binding");

    assert_eq!(serialized, "<out xml:lang=\"en\"></out>");
}

#[test]
fn namespaced_element_names_use_retained_bindings_and_undeclare_defaults() {
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: Some("urn:prefixed".to_owned()),
                local: "root".to_owned(),
            },
            namespaces: vec![
                crate::xml::quick_xml_experiment::NamespaceBinding {
                    prefix: Some("p".to_owned()),
                    namespace: "urn:prefixed".to_owned(),
                },
                crate::xml::quick_xml_experiment::NamespaceBinding {
                    prefix: None,
                    namespace: "urn:default".to_owned(),
                },
            ]
            .into(),
            attributes: Vec::new(),
            children: vec![ResultNode::Element {
                name: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: "child".to_owned(),
                },
                namespaces: Vec::new().into(),
                attributes: Vec::new(),
                children: Vec::new(),
            }],
        }],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("xml".to_owned()),
        version: None,
        html_version: None,
        encoding: None,
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: true,
        indent: None,
    };

    let mut control = InvocationControl::unbounded();
    let serialized = serialize_xml(&result, &settings, "namespaced", 4_096, &mut control)
        .expect("serialize retained namespace bindings");

    assert_eq!(
        serialized,
        "<p:root xmlns:p=\"urn:prefixed\" xmlns=\"urn:default\"><child xmlns=\"\"></child></p:root>"
    );
}

#[test]
fn text_output_concatenates_descendant_text_without_markup_or_escaping() {
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: "root".to_owned(),
            },
            namespaces: Vec::new().into(),
            attributes: Vec::new(),
            children: vec![
                ResultNode::Text("A < B & C".to_owned()),
                ResultNode::Element {
                    name: crate::xml::quick_xml_experiment::ExpandedName {
                        namespace: None,
                        local: "nested".to_owned(),
                    },
                    namespaces: Vec::new().into(),
                    attributes: Vec::new(),
                    children: vec![ResultNode::Text(" + nested".to_owned())],
                },
            ],
        }],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("text".to_owned()),
        version: None,
        html_version: None,
        encoding: None,
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: Some(true),
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: false,
        indent: None,
    };
    let mut control = InvocationControl::unbounded();

    let serialized = serialize_xml(&result, &settings, "text", 4_096, &mut control)
        .expect("serialize text result");

    assert_eq!(serialized, "A < B & C + nested");
}

#[test]
fn processing_instruction_serializes_as_markup_but_not_as_text_value() {
    let result = SemanticResult {
        children: vec![ResultNode::ProcessingInstruction {
            target: "my-pi".to_owned(),
            value: "href=\"book.css\" type=\"text/css\"".to_owned(),
        }],
    };
    let mut settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("xml".to_owned()),
        version: None,
        html_version: None,
        encoding: None,
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: true,
        indent: None,
    };
    let mut control = InvocationControl::unbounded();
    assert_eq!(
        serialize_xml(&result, &settings, "pi", 4_096, &mut control)
            .expect("serialize processing instruction"),
        "<?my-pi href=\"book.css\" type=\"text/css\"?>"
    );

    settings.method = Some("text".to_owned());
    let mut control = InvocationControl::unbounded();
    assert_eq!(
        serialize_xml(&result, &settings, "pi-text", 4_096, &mut control)
            .expect("serialize PI document string value"),
        ""
    );
}

#[test]
fn xml_compatible_xhtml_output_honors_explicit_declaration_omission() {
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: Some("http://www.w3.org/1999/xhtml".to_owned()),
                local: "out".to_owned(),
            },
            namespaces: vec![crate::xml::quick_xml_experiment::NamespaceBinding {
                prefix: None,
                namespace: "http://www.w3.org/1999/xhtml".to_owned(),
            }]
            .into(),
            attributes: Vec::new(),
            children: Vec::new(),
        }],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("xhtml".to_owned()),
        version: None,
        html_version: None,
        encoding: None,
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: true,
        indent: Some(false),
    };
    let mut control = InvocationControl::unbounded();

    let serialized = serialize_xml(&result, &settings, "xhtml", 4_096, &mut control)
        .expect("serialize XML-compatible XHTML result");

    assert_eq!(
        serialized,
        "<out xmlns=\"http://www.w3.org/1999/xhtml\"></out>"
    );
}

#[test]
fn xhtml_doctype_bytes_are_bounded_with_the_rest_of_serialization() {
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: Some("http://www.w3.org/1999/xhtml".to_owned()),
                local: "html".to_owned(),
            },
            namespaces: vec![crate::xml::quick_xml_experiment::NamespaceBinding {
                prefix: None,
                namespace: "http://www.w3.org/1999/xhtml".to_owned(),
            }]
            .into(),
            attributes: Vec::new(),
            children: Vec::new(),
        }],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("xhtml".to_owned()),
        version: None,
        html_version: None,
        encoding: None,
        media_type: None,
        doctype_system: Some("out.dtd".to_owned()),
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: None,
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: true,
        indent: Some(false),
    };
    let expected =
        "<!DOCTYPE html SYSTEM \"out.dtd\"><html xmlns=\"http://www.w3.org/1999/xhtml\"></html>";
    let serialized = serialize_xml(
        &result,
        &settings,
        "bounded-doctype",
        expected.len(),
        &mut InvocationControl::unbounded(),
    )
    .expect("serialize a DOCTYPE at the exact byte limit");
    assert_eq!(serialized, expected);

    let failure = serialize_xml(
        &result,
        &settings,
        "bounded-doctype",
        expected.len() - 1,
        &mut InvocationControl::unbounded(),
    )
    .expect_err("DOCTYPE bytes must not bypass the serialized result limit");
    assert_eq!(failure.code, "FXSR0002");
    assert_eq!(failure.category, FailureCategory::Limit);
}

#[test]
fn string_serialization_accepts_utf8_without_bom_and_rejects_bom_emission() {
    let result = SemanticResult {
        children: vec![ResultNode::Text("result".to_owned())],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("xml".to_owned()),
        version: None,
        html_version: None,
        encoding: Some("UTF-8".to_owned()),
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: Some(false),
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: false,
        indent: Some(false),
    };
    let mut control = InvocationControl::unbounded();
    let serialized = serialize_xml(&result, &settings, "utf8", 4_096, &mut control)
        .expect("serialize UTF-8 without a byte-order mark");

    let mut bom_settings = settings;
    bom_settings.byte_order_mark = Some(true);
    let failure = serialize_xml(
        &result,
        &bom_settings,
        "utf8-bom",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect_err("the string lane must not pretend to emit byte metadata");

    assert_eq!(
        serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>result"
    );
    assert_eq!(failure.code, "FXSR1005");
    assert_eq!(failure.category, FailureCategory::Unsupported);
}

#[test]
fn byte_serialization_emits_bounded_ascii_iso_8859_1() {
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: "out".to_owned(),
            },
            namespaces: Vec::new().into(),
            attributes: Vec::new(),
            children: vec![ResultNode::Text("ASCII result".to_owned())],
        }],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("xml".to_owned()),
        version: None,
        html_version: None,
        encoding: Some("ISO-8859-1".to_owned()),
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: Some(false),
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: false,
        indent: Some(false),
    };
    let mut control = InvocationControl::unbounded();
    let bytes = serialize_xml_bytes(&result, &settings, "latin1", 4_096, &mut control)
        .expect("serialize the bounded ASCII subset as ISO-8859-1 bytes");
    assert_eq!(
        bytes,
        b"<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><out>ASCII result</out>"
    );

    let non_ascii = SemanticResult {
        children: vec![ResultNode::Text("\u{e9}".to_owned())],
    };
    let failure = serialize_xml_bytes(
        &non_ascii,
        &settings,
        "latin1-non-ascii",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect_err("the bounded lane must not replace or misencode non-ASCII text");
    assert_eq!(failure.code, "FXSR1006");
    assert_eq!(failure.category, FailureCategory::Unsupported);
}

#[test]
fn us_ascii_cdata_expansion_is_bounded_and_rejects_other_non_ascii_content() {
    let name = crate::xml::quick_xml_experiment::ExpandedName {
        namespace: Some("http://www.w3.org/1999/xhtml".to_owned()),
        local: "example".to_owned(),
    };
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: name.clone(),
            namespaces: vec![crate::xml::quick_xml_experiment::NamespaceBinding {
                prefix: None,
                namespace: "http://www.w3.org/1999/xhtml".to_owned(),
            }]
            .into(),
            attributes: Vec::new(),
            children: vec![ResultNode::Text("ç".to_owned())],
        }],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("xhtml".to_owned()),
        version: None,
        html_version: None,
        encoding: Some("US-ASCII".to_owned()),
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: Some(false),
        normalization_form: Some("NFC".to_owned()),
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: vec![name],
        omit_xml_declaration: false,
        indent: Some(false),
    };
    let bytes = serialize_xml_bytes(
        &result,
        &settings,
        "ascii-cdata",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect("serialize bounded non-ASCII CDATA as US-ASCII");
    assert!(bytes.is_ascii());
    assert!(String::from_utf8_lossy(&bytes).contains("]]>&#xE7;<![CDATA["));

    serialize_xml_bytes(
        &result,
        &settings,
        "ascii-cdata-exact-limit",
        bytes.len(),
        &mut InvocationControl::unbounded(),
    )
    .expect("the exact final expanded-byte limit must pass");
    let failure = serialize_xml_bytes(
        &result,
        &settings,
        "ascii-cdata-short-limit",
        bytes.len() - 1,
        &mut InvocationControl::unbounded(),
    )
    .expect_err("the final expanded-byte limit must reject one byte less");
    assert_eq!(failure.code, "FXSR0002");

    let mut ordinary = settings;
    ordinary.cdata_section_elements.clear();
    let failure = serialize_xml_bytes(
        &result,
        &ordinary,
        "ascii-non-cdata",
        4_096,
        &mut InvocationControl::unbounded(),
    )
    .expect_err("non-ASCII outside the admitted CDATA shape must stay unsupported");
    assert_eq!(failure.code, "FXSR1009");
    assert_eq!(failure.category, FailureCategory::Unsupported);
}

#[test]
fn byte_serialization_emits_and_accounts_for_a_utf8_byte_order_mark() {
    let result = SemanticResult {
        children: vec![ResultNode::Text("result".to_owned())],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("text".to_owned()),
        version: None,
        html_version: None,
        encoding: Some("UTF-8".to_owned()),
        media_type: None,
        doctype_system: None,
        doctype_public: None,
        include_content_type: None,
        escape_uri_attributes: None,
        byte_order_mark: Some(true),
        normalization_form: None,
        character_map: Vec::new(),
        undeclare_prefixes: None,
        standalone: None,
        suppress_indentation_elements: Vec::new(),
        cdata_section_elements: Vec::new(),
        omit_xml_declaration: false,
        indent: Some(false),
    };
    let bytes = serialize_xml_bytes(
        &result,
        &settings,
        "utf8-bom-bytes",
        9,
        &mut InvocationControl::unbounded(),
    )
    .expect("the byte lane should prepend the UTF-8 byte-order mark");
    assert_eq!(bytes, b"\xef\xbb\xbfresult");

    let failure = serialize_xml_bytes(
        &result,
        &settings,
        "utf8-bom-limit",
        8,
        &mut InvocationControl::unbounded(),
    )
    .expect_err("the byte limit must include the byte-order mark");
    assert_eq!(failure.code, "FXSR0002");
    assert_eq!(failure.category, FailureCategory::Limit);
}

#[test]
fn integer_range_materialization_charges_each_atomic_item_before_retention() {
    let mut limits = WorkLimits::unbounded();
    limits.xpath_operations = 9;
    let mut control = InvocationControl::new(CancellationToken::new(), limits);

    let failure = materialize_integer_range(1, 10, "range-request", &mut control)
        .expect_err("the tenth item must exceed the nine-operation budget");

    assert_eq!(failure.category, FailureCategory::Limit);
    assert_eq!(failure.work_domain, Some(WorkDomain::XPathOperation));
    assert_eq!(failure.request_id.as_deref(), Some("range-request"));
}

#[test]
fn atomic_apply_range_observes_control_before_span_proportional_retention() {
    const SOURCE: &str = "urn:fastxslt:controlled-atomic-range:source";
    const STYLESHEET: &str = "urn:fastxslt:controlled-atomic-range:stylesheet";
    let stylesheet = br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:apply-templates select="1 to 1000000000"/></xsl:template></xsl:stylesheet>"#;
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc/>".to_vec())
        .expect("admit controlled range source");
    resources
        .admit(STYLESHEET, stylesheet.to_vec())
        .expect("admit controlled range stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile controlled range");
    let mut work_limits = WorkLimits::unbounded();
    work_limits.xpath_operations = 0;
    let mut builder = TransformSetBuilder::new(
        snapshot,
        program,
        1,
        ExecutionPolicy {
            denied_sources: HashSet::new(),
            serialized_byte_limit: 4_096,
            work_limits,
        },
    );
    builder
        .add(request("controlled-range", "result", SOURCE))
        .expect("admit controlled range request");

    let failure = execute_transform_set(builder.seal())
        .expect_err("control must stop before retaining the hostile range");

    assert_eq!(failure.category, FailureCategory::Limit);
    assert_eq!(failure.work_domain, Some(WorkDomain::XPathOperation));
    assert_eq!(failure.request_id.as_deref(), Some("controlled-range"));
}

#[test]
fn builder_rejects_duplicates_limits_and_unadmitted_sibling_results() {
    let snapshot = snapshot();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
    let mut builder = TransformSetBuilder::new(snapshot, program, 2, policy(4_096));
    builder
        .add(request("first", "future.xml", SOURCE_ID))
        .expect("add first request");

    let failure = builder
        .add(request("first", "other.xml", SOURCE_ID))
        .expect_err("duplicate request should fail");
    assert_eq!(failure.code, "FXBT0002");
    assert_eq!(failure.category, FailureCategory::Invalid);

    let failure = builder
        .add(request("second", "future.xml", SOURCE_ID))
        .expect_err("duplicate result should fail");
    assert_eq!(failure.code, "FXBT0003");

    let failure = builder
        .add(request("second", "second-result", "future.xml"))
        .expect_err("a sibling result is not an admitted source");
    assert_eq!(failure.code, "FXRS0001");
    assert_eq!(failure.category, FailureCategory::MissingResource);

    builder
        .add(request("second", "second-result", SOURCE_ID))
        .expect("failed additions do not mutate the builder");
    let failure = builder
        .add(request("third", "third-result", SOURCE_ID))
        .expect_err("request limit should fail");
    assert_eq!(failure.code, "FXBT0001");
    assert_eq!(failure.category, FailureCategory::Limit);
}

#[test]
fn initial_template_entry_rejects_an_unknown_compiled_name_without_a_source() {
    let snapshot = snapshot();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));

    let failure = builder
        .add(TransformRequest {
            identity: "unknown-entry".to_owned(),
            result_identity: "unknown-result".to_owned(),
            entry: InvocationEntry::InitialTemplate {
                name: "missing".to_owned(),
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect_err("unknown initial-template entry should fail admission");

    assert_eq!(failure.code, "XTDE0040");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(failure.request_id.as_deref(), Some("unknown-entry"));
}

#[test]
fn initial_template_position_requires_a_dynamic_focus() {
    const SOURCE: &str = "urn:fastxslt:focusless-initial-template:source";
    const STYLESHEET: &str = "urn:fastxslt:focusless-initial-template:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<unused/>".to_vec())
        .expect("admit unused source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="start"><xsl:value-of select="position()"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile focus operation");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(TransformRequest {
            identity: "focusless".to_owned(),
            result_identity: "result".to_owned(),
            entry: InvocationEntry::InitialTemplate {
                name: "start".to_owned(),
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect("admit focusless initial template");

    let failure = execute_transform_set(builder.seal())
        .expect_err("position without a dynamic focus must fail");
    assert_eq!(failure.code, "XPDY0002");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert!(failure.location.is_some());
}

#[test]
fn initial_template_position_comparison_requires_a_dynamic_focus() {
    const SOURCE: &str = "urn:fastxslt:focusless-comparison:source";
    const STYLESHEET: &str = "urn:fastxslt:focusless-comparison:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<unused/>".to_vec())
        .expect("admit unused source");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="start"><xsl:value-of select="position() = 1"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile focus comparison");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(TransformRequest {
            identity: "focusless-comparison".to_owned(),
            result_identity: "result".to_owned(),
            entry: InvocationEntry::InitialTemplate {
                name: "start".to_owned(),
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect("admit focusless initial template");

    let failure = execute_transform_set(builder.seal())
        .expect_err("position comparison without a dynamic focus must fail");
    assert_eq!(failure.code, "XPDY0002");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert!(failure.location.is_some());
}

#[test]
fn initial_template_copy_of_current_copies_a_document_nodes_children() {
    const SOURCE: &str = "urn:fastxslt:initial-template-document-copy:source";
    const STYLESHEET: &str = "urn:fastxslt:initial-template-document-copy:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<doc><a/></doc>".to_vec())
        .expect("admit source document");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template name="start"><out><xsl:copy-of select="."/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(TransformRequest {
            identity: "document-copy".to_owned(),
            result_identity: "document-copy-result".to_owned(),
            entry: InvocationEntry::InitialTemplateWithSource {
                resource: SOURCE.to_owned(),
                name: "start".to_owned(),
            },
            parameters: BTreeMap::new(),
            cancellation: CancellationToken::new(),
            cancellation_fault: None,
        })
        .expect("admit initial-template request");

    let results = execute_transform_set(builder.seal()).expect("execute document copy");

    assert_eq!(
        results.by_request["document-copy"].serialized,
        "<out><doc><a></a></doc></out>"
    );
}

#[test]
fn copy_of_static_atomic_values_construct_bounded_text() {
    const SOURCE: &str = "urn:fastxslt:static-string-copy:source";
    const STYLESHEET: &str = "urn:fastxslt:static-string-copy:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(SOURCE, b"<doc/>".to_vec())
        .expect("admit source document");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:copy-of select="'test &amp; value'"/>:<xsl:copy-of select="32"/>:<xsl:copy-of select="true()"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(16_384));
    builder
        .add(request("static-string-copy", "result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute static string copy");

    assert_eq!(
        results.by_request["static-string-copy"].serialized,
        "<out>test &amp; value:32:true</out>"
    );
}

#[test]
fn copy_of_variables_preserves_atomic_source_node_and_temporary_tree_values() {
    const SOURCE: &str = "urn:fastxslt:variable-copy:source";
    const STYLESHEET: &str = "urn:fastxslt:variable-copy:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 16_384, 32_768));
    resources
        .admit(
            SOURCE,
            b"<doc><item id=\"1\">source</item><item id=\"2\"/></doc>".to_vec(),
        )
        .expect("admit source document");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:kept" exclude-result-prefixes="p"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:variable name="atomic" select="'value'"/><xsl:template match="/"><xsl:variable name="nodes" select="/doc/item"/><xsl:variable name="tree"><p:held marker="yes"><inner>temporary</inner></p:held></xsl:variable><out><xsl:copy-of select="$atomic"/><xsl:copy-of select="$nodes"/><xsl:copy-of select="$tree"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile variable copies");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(32_768));
    builder
        .add(request("variable-copy", "variable-copy-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute variable copies");

    assert_eq!(
        results.by_request["variable-copy"].serialized,
        "<out>value<item id=\"1\">source</item><item id=\"2\"></item><p:held xmlns:p=\"urn:kept\" marker=\"yes\"><inner>temporary</inner></p:held></out>"
    );
}

#[test]
fn copy_of_path_union_deduplicates_and_restores_document_order() {
    const SOURCE: &str = "urn:fastxslt:union-copy:source";
    const STYLESHEET: &str = "urn:fastxslt:union-copy:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 12_288, 24_576));
    resources
        .admit(
            SOURCE,
            br#"<doc xmlns:p="urn:items" z="last-expression" x="first-expression"><p:a/><a/></doc>"#.to_vec(),
        )
        .expect("admit source document");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:items" exclude-result-prefixes="p"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><xsl:apply-templates select="doc"/></xsl:template><xsl:template match="doc"><out><xsl:copy-of select="@x | p:a | @z | p:a | a"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile copy-of path union");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(24_576));
    builder
        .add(request("union-copy", "union-copy-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute copy-of path union");

    assert_eq!(
        results.by_request["union-copy"].serialized,
        "<out z=\"last-expression\" x=\"first-expression\"><p:a xmlns:p=\"urn:items\"></p:a><a xmlns:p=\"urn:items\"></a></out>"
    );
}

#[test]
fn copy_of_location_path_copies_each_selected_subtree_in_document_order() {
    const SOURCE: &str = "urn:fastxslt:path-copy:source";
    const STYLESHEET: &str = "urn:fastxslt:path-copy:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            b"<doc><selected id=\"1\"><value>A</value></selected><skip/><selected id=\"2\"><value>B</value></selected></doc>".to_vec(),
        )
        .expect("admit source document");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:copy-of select="/doc/selected"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(16_384));
    builder
        .add(request("path-copy", "path-copy-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute location-path copy");

    assert_eq!(
        results.by_request["path-copy"].serialized,
        "<out><selected id=\"1\"><value>A</value></selected><selected id=\"2\"><value>B</value></selected></out>"
    );
}

#[test]
fn copy_of_location_paths_preserve_attribute_comment_and_processing_instruction_kinds() {
    const SOURCE: &str = "urn:fastxslt:node-kind-copy:source";
    const STYLESHEET: &str = "urn:fastxslt:node-kind-copy:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 8_192, 16_384));
    resources
        .admit(
            SOURCE,
            b"<doc id=\"kept\"><!--note--><?work ready?></doc>".to_vec(),
        )
        .expect("admit source document");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:copy-of select="/doc/@id"/><xsl:copy-of select="/doc/comment()"/><xsl:copy-of select="/doc/processing-instruction()"/></out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(16_384));
    builder
        .add(request("node-kind-copy", "node-kind-copy-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute node-kind copy");

    assert_eq!(
        results.by_request["node-kind-copy"].serialized,
        "<out id=\"kept\"><!--note--><?work ready?></out>"
    );
}

#[test]
fn copy_of_ancestor_or_self_elements_preserves_reverse_axis_order() {
    const SOURCE: &str = "urn:fastxslt:ancestor-copy:source";
    const STYLESHEET: &str = "urn:fastxslt:ancestor-copy:stylesheet";
    let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
    resources
        .admit(SOURCE, b"<root><mid><leaf/></mid></root>".to_vec())
        .expect("admit source document");
    resources
        .admit(
            STYLESHEET,
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><out><xsl:apply-templates select="root/mid/leaf"/></out></xsl:template><xsl:template match="leaf"><xsl:copy-of select="ancestor-or-self::*"/></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, STYLESHEET).expect("compile stylesheet");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(request("ancestor-copy", "ancestor-copy-result", SOURCE))
        .expect("admit request");

    let results = execute_transform_set(builder.seal()).expect("execute ancestor copy");

    assert_eq!(
        results.by_request["ancestor-copy"].serialized,
        "<out><leaf></leaf><mid><leaf></leaf></mid><root><mid><leaf></leaf></mid></root></out>"
    );
}

#[test]
fn explicit_source_denial_is_distinct_from_missing_resource() {
    let snapshot = snapshot();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
    let mut denied_sources = HashSet::new();
    denied_sources.insert(SOURCE_ID.to_owned());
    let mut builder = TransformSetBuilder::new(
        snapshot,
        program,
        1,
        ExecutionPolicy {
            denied_sources,
            serialized_byte_limit: 4_096,
            work_limits: WorkLimits::unbounded(),
        },
    );

    let failure = builder
        .add(request("denied", "denied-result", SOURCE_ID))
        .expect_err("admitted source should still be deniable");

    assert_eq!(failure.code, "FXRS0003");
    assert_eq!(failure.category, FailureCategory::Denied);
    assert_eq!(failure.request_id.as_deref(), Some("denied"));
}

#[test]
fn serialization_stops_before_exceeding_the_host_byte_limit() {
    let snapshot = snapshot();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(16));
    builder
        .add(request("limited", "limited-result", SOURCE_ID))
        .expect("add limited request");

    let failure = execute_transform_set(builder.seal()).expect_err("output should be limited");

    assert_eq!(failure.code, "FXSR0002");
    assert_eq!(failure.category, FailureCategory::Limit);
    assert_eq!(failure.request_id.as_deref(), Some("limited"));
    assert_eq!(failure.work_domain, None);
}

#[test]
fn host_cancellation_is_observed_as_cooperative_control_not_a_budget_failure() {
    let snapshot = snapshot();
    let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
    let token = CancellationToken::new();
    let mut controlled_request = request("cancelled", "cancelled-result", SOURCE_ID);
    controlled_request.cancellation = token.clone();
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
    builder
        .add(controlled_request)
        .expect("admit cancellable request");
    token.cancel();

    let failure = execute_transform_set(builder.seal()).expect_err("cancelled work should stop");

    assert_eq!(failure.code, "FXCT0001");
    assert_eq!(failure.category, FailureCategory::Cancelled);
    assert_eq!(failure.request_id.as_deref(), Some("cancelled"));
    assert_eq!(failure.work_domain, Some(WorkDomain::XmlEvent));
}

#[test]
fn each_implemented_layer_charges_its_own_work_domain() {
    let cases: [(WorkDomain, ConfigureWorkLimits); 8] = [
        (WorkDomain::XmlEvent, |limits: &mut WorkLimits| {
            limits.xml_events = 0;
        }),
        (WorkDomain::XdmNode, |limits: &mut WorkLimits| {
            limits.xdm_nodes = 1;
        }),
        (WorkDomain::XPathNodeVisit, |limits: &mut WorkLimits| {
            limits.xpath_node_visits = 0;
        }),
        (WorkDomain::XdmStringValueNode, |limits: &mut WorkLimits| {
            limits.xdm_string_value_nodes = 0;
        }),
        (WorkDomain::XsltInstruction, |limits: &mut WorkLimits| {
            limits.xslt_instructions = 0;
        }),
        (WorkDomain::ResultNode, |limits: &mut WorkLimits| {
            limits.result_nodes = 0;
        }),
        (WorkDomain::ResultTextByte, |limits: &mut WorkLimits| {
            limits.result_text_bytes = 0;
        }),
        (WorkDomain::SerializedByte, |limits: &mut WorkLimits| {
            limits.serialized_bytes = 0;
        }),
    ];

    for (domain, configure) in cases {
        let mut limits = WorkLimits::unbounded();
        configure(&mut limits);
        let failure = execute_with_work_limits(domain.name(), limits);

        assert_eq!(failure.code, "FXCT0002");
        assert_eq!(failure.category, FailureCategory::Limit);
        assert_eq!(failure.request_id.as_deref(), Some(domain.name()));
        assert_eq!(failure.work_domain, Some(domain));
    }
}
