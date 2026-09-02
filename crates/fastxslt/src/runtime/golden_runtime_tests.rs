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
            namespaces: Vec::new(),
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
            namespaces: Vec::new(),
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
            namespaces: Vec::new(),
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
fn context_node_name_refuses_to_fabricate_a_namespaced_lexical_qname() {
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

    let failure = execute_transform_set(builder.seal())
        .expect_err("prefix-free expanded-name storage cannot preserve fn:name lexical identity");
    assert_eq!(failure.code, "FXRT1008");
    assert_eq!(failure.category, FailureCategory::Unsupported);
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
            value: AtomicValue::string("host supplied"),
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
            namespaces: Vec::new(),
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
            }],
            attributes: Vec::new(),
            children: vec![ResultNode::Element {
                name: xhtml_name("br"),
                namespaces: Vec::new(),
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
            namespaces: Vec::new(),
            attributes: Vec::new(),
            children: vec![
                ResultNode::Element {
                    name: crate::xml::quick_xml_experiment::ExpandedName {
                        namespace: None,
                        local: "group".to_owned(),
                    },
                    namespaces: Vec::new(),
                    attributes: Vec::new(),
                    children: vec![ResultNode::Element {
                        name: crate::xml::quick_xml_experiment::ExpandedName {
                            namespace: None,
                            local: "item".to_owned(),
                        },
                        namespaces: Vec::new(),
                        attributes: Vec::new(),
                        children: vec![ResultNode::Text("value".to_owned())],
                    }],
                },
                ResultNode::Element {
                    name: crate::xml::quick_xml_experiment::ExpandedName {
                        namespace: None,
                        local: "mixed".to_owned(),
                    },
                    namespaces: Vec::new(),
                    attributes: Vec::new(),
                    children: vec![
                        ResultNode::Text("left".to_owned()),
                        ResultNode::Element {
                            name: crate::xml::quick_xml_experiment::ExpandedName {
                                namespace: None,
                                local: "em".to_owned(),
                            },
                            namespaces: Vec::new(),
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
            }],
            attributes: Vec::new(),
            children: vec![
                ResultNode::Element {
                    name: xhtml_name("meta"),
                    namespaces: Vec::new(),
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
            namespaces: Vec::new(),
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
            ],
            attributes: Vec::new(),
            children: vec![ResultNode::Element {
                name: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: "child".to_owned(),
                },
                namespaces: Vec::new(),
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
            namespaces: Vec::new(),
            attributes: Vec::new(),
            children: vec![
                ResultNode::Text("A < B & C".to_owned()),
                ResultNode::Element {
                    name: crate::xml::quick_xml_experiment::ExpandedName {
                        namespace: None,
                        local: "nested".to_owned(),
                    },
                    namespaces: Vec::new(),
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
            }],
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
            }],
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
            namespaces: Vec::new(),
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
            }],
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

    assert_eq!(failure.code, "FXRT0004");
    assert_eq!(failure.category, FailureCategory::Invalid);
    assert_eq!(failure.request_id.as_deref(), Some("unknown-entry"));
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
