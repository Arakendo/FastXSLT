//! General runtime contract tests retained separately from execution semantics.

use std::collections::{BTreeMap, HashSet};

use crate::execution_control_experiment::{
    CancellationToken, InvocationControl, WorkDomain, WorkLimits,
};
use crate::resources::{ResourceLimits, ResourceSetBuilder};
use crate::xdm::atomic_value_experiment::AtomicValue;

use super::{
    ExecutionPolicy, FailureCategory, InvocationEntry, InvocationParameter, ResultNode,
    SemanticResult, TransformRequest, TransformSetBuilder, compile_resource, execute_transform_set,
    materialize_integer_range, serialize_xml,
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
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:template match="/" mode="audit"><out>mode</out></xsl:template></xsl:stylesheet>"#.to_vec(),
        )
        .expect("admit initial-mode stylesheet");
    let snapshot = resources.seal();
    let program = compile_resource(&snapshot, MODE_STYLESHEET).expect("compile initial mode");
    let mut builder = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));

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
    let results = execute_transform_set(builder.seal()).expect("execute initial mode");
    assert_eq!(
        results.by_request["known-mode"].serialized,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><out>mode</out>"
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
        encoding: None,
        media_type: None,
        include_content_type: None,
        byte_order_mark: None,
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
fn requested_indentation_is_preserved_as_an_explicit_serialization_boundary() {
    let result = SemanticResult {
        children: vec![ResultNode::Element {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: "out".to_owned(),
            },
            namespaces: Vec::new(),
            attributes: Vec::new(),
            children: Vec::new(),
        }],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("xml".to_owned()),
        encoding: None,
        media_type: None,
        include_content_type: None,
        byte_order_mark: None,
        omit_xml_declaration: false,
        indent: Some(true),
    };

    let mut control = InvocationControl::unbounded();
    let failure = serialize_xml(&result, &settings, "indented-result", 4_096, &mut control)
        .expect_err("indentation must not be silently ignored");

    assert_eq!(failure.code, "FXSR1003");
    assert_eq!(failure.category, FailureCategory::Unsupported);
    assert_eq!(failure.request_id.as_deref(), Some("indented-result"));
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
        encoding: None,
        media_type: None,
        include_content_type: None,
        byte_order_mark: None,
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
        encoding: None,
        media_type: None,
        include_content_type: Some(true),
        byte_order_mark: None,
        omit_xml_declaration: false,
        indent: None,
    };
    let mut control = InvocationControl::unbounded();

    let serialized = serialize_xml(&result, &settings, "text", 4_096, &mut control)
        .expect("serialize text result");

    assert_eq!(serialized, "A < B & C + nested");
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
        encoding: None,
        media_type: None,
        include_content_type: None,
        byte_order_mark: None,
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
fn string_serialization_accepts_utf8_without_bom_and_rejects_bom_emission() {
    let result = SemanticResult {
        children: vec![ResultNode::Text("result".to_owned())],
    };
    let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
        method: Some("xml".to_owned()),
        encoding: Some("UTF-8".to_owned()),
        media_type: None,
        include_content_type: None,
        byte_order_mark: Some(false),
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
