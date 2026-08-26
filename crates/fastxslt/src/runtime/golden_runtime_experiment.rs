use std::collections::{BTreeMap, HashSet};

use crate::compile::golden_stylesheet_experiment::compile_stylesheet;
use crate::resources::ResourceSnapshot;
use crate::xdm::owned_tree_experiment::Document;
use crate::xml::quick_xml_experiment::{ExpandedName, ParseLimits, parse_document};
use crate::xpath::path_experiment::evaluate_child_path;
use crate::xslt::golden_semantics_experiment::{Instruction, OutputSettings, StylesheetProgram};

const XML_LIMITS: ParseLimits = ParseLimits {
    max_events: 1_024,
    max_depth: 64,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResultNode {
    Element {
        name: ExpandedName,
        children: Vec<ResultNode>,
    },
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SemanticResult {
    children: Vec<ResultNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransformRequest {
    identity: String,
    result_identity: String,
    source_resource: String,
}

#[derive(Debug)]
struct TransformSetBuilder {
    snapshot: ResourceSnapshot,
    stylesheet: StylesheetProgram,
    requests: Vec<TransformRequest>,
    request_ids: HashSet<String>,
    result_ids: HashSet<String>,
    request_limit: usize,
    policy: ExecutionPolicy,
}

#[derive(Debug)]
struct TransformSet {
    snapshot: ResourceSnapshot,
    stylesheet: StylesheetProgram,
    requests: Vec<TransformRequest>,
    policy: ExecutionPolicy,
}

#[derive(Debug, Clone)]
struct ExecutionPolicy {
    denied_sources: HashSet<String>,
    serialized_byte_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResultEntry {
    result_id: String,
    semantic: SemanticResult,
    serialized: String,
}

#[derive(Debug, PartialEq, Eq)]
struct ResultSet {
    by_request: BTreeMap<String, ResultEntry>,
    completion_order: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FailureCategory {
    Invalid,
    Unsupported,
    MissingResource,
    Denied,
    Limit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExecutionFailure {
    code: &'static str,
    category: FailureCategory,
    request_id: Option<String>,
    detail: String,
}

impl TransformSetBuilder {
    fn new(
        snapshot: ResourceSnapshot,
        stylesheet: StylesheetProgram,
        request_limit: usize,
        policy: ExecutionPolicy,
    ) -> Self {
        Self {
            snapshot,
            stylesheet,
            requests: Vec::new(),
            request_ids: HashSet::new(),
            result_ids: HashSet::new(),
            request_limit,
            policy,
        }
    }

    fn add(&mut self, request: TransformRequest) -> Result<(), ExecutionFailure> {
        if self.requests.len() >= self.request_limit {
            return Err(failure(
                "FXBT0001",
                FailureCategory::Limit,
                Some(&request.identity),
                format!("transform-set request limit is {}", self.request_limit),
            ));
        }
        if !self.request_ids.insert(request.identity.clone()) {
            return Err(failure(
                "FXBT0002",
                FailureCategory::Invalid,
                Some(&request.identity),
                "duplicate request identity",
            ));
        }
        if !self.result_ids.insert(request.result_identity.clone()) {
            self.request_ids.remove(&request.identity);
            return Err(failure(
                "FXBT0003",
                FailureCategory::Invalid,
                Some(&request.identity),
                "duplicate result identity",
            ));
        }
        if self
            .policy
            .denied_sources
            .contains(&request.source_resource)
        {
            self.request_ids.remove(&request.identity);
            self.result_ids.remove(&request.result_identity);
            return Err(failure(
                "FXRS0003",
                FailureCategory::Denied,
                Some(&request.identity),
                format!("source authority is denied: {}", request.source_resource),
            ));
        }
        if self.snapshot.get(&request.source_resource).is_none() {
            self.request_ids.remove(&request.identity);
            self.result_ids.remove(&request.result_identity);
            return Err(failure(
                "FXRS0001",
                FailureCategory::MissingResource,
                Some(&request.identity),
                format!("source is not admitted: {}", request.source_resource),
            ));
        }
        self.requests.push(request);
        Ok(())
    }

    fn seal(self) -> TransformSet {
        TransformSet {
            snapshot: self.snapshot,
            stylesheet: self.stylesheet,
            requests: self.requests,
            policy: self.policy,
        }
    }
}

fn compile_resource(
    snapshot: &ResourceSnapshot,
    stylesheet_id: &str,
) -> Result<StylesheetProgram, ExecutionFailure> {
    let bytes = snapshot.get(stylesheet_id).ok_or_else(|| {
        failure(
            "FXRS0002",
            FailureCategory::MissingResource,
            None,
            format!("stylesheet is not admitted: {stylesheet_id}"),
        )
    })?;
    let parsed = parse_document(stylesheet_id, bytes, XML_LIMITS).map_err(|error| {
        failure(
            "FXXM0001",
            FailureCategory::Invalid,
            None,
            format!("stylesheet XML is invalid: {error:?}"),
        )
    })?;
    let document = Document::from_parsed(parsed).map_err(|error| {
        failure(
            "FXXD0001",
            FailureCategory::Invalid,
            None,
            format!("stylesheet XDM construction failed: {error:?}"),
        )
    })?;
    compile_stylesheet(&document).map_err(|error| {
        failure(
            error.code,
            match error.category {
                crate::compile::golden_stylesheet_experiment::CompileCategory::Invalid => {
                    FailureCategory::Invalid
                }
                crate::compile::golden_stylesheet_experiment::CompileCategory::Unsupported => {
                    FailureCategory::Unsupported
                }
            },
            None,
            format!(
                "{} at {}:{}..{}",
                error.detail,
                error.location.resource,
                error.location.span.start,
                error.location.span.end
            ),
        )
    })
}

fn execute_transform_set(set: TransformSet) -> Result<ResultSet, ExecutionFailure> {
    let mut by_request = BTreeMap::new();
    let mut completion_order = Vec::new();

    for request in set.requests.into_iter().rev() {
        let bytes = set
            .snapshot
            .get(&request.source_resource)
            .expect("sealed transform sets contain admitted sources");
        let parsed =
            parse_document(&request.source_resource, bytes, XML_LIMITS).map_err(|error| {
                failure(
                    "FXXM0002",
                    FailureCategory::Invalid,
                    Some(&request.identity),
                    format!("source XML is invalid: {error:?}"),
                )
            })?;
        let source = Document::from_parsed(parsed).map_err(|error| {
            failure(
                "FXXD0002",
                FailureCategory::Invalid,
                Some(&request.identity),
                format!("source XDM construction failed: {error:?}"),
            )
        })?;
        let semantic = execute_program(&set.stylesheet, &source, &request.identity)?;
        let serialized = serialize_xml(
            &semantic,
            &set.stylesheet.output,
            &request.identity,
            set.policy.serialized_byte_limit,
        )?;
        completion_order.push(request.identity.clone());
        by_request.insert(
            request.identity,
            ResultEntry {
                result_id: request.result_identity,
                semantic,
                serialized,
            },
        );
    }
    Ok(ResultSet {
        by_request,
        completion_order,
    })
}

fn execute_program(
    program: &StylesheetProgram,
    source: &Document,
    request_id: &str,
) -> Result<SemanticResult, ExecutionFailure> {
    Ok(SemanticResult {
        children: execute_sequence(
            &program.root_template.body,
            source,
            source.document_node(),
            request_id,
        )?,
    })
}

fn execute_sequence(
    instructions: &[Instruction],
    source: &Document,
    context: crate::xdm::owned_tree_experiment::NodeId,
    request_id: &str,
) -> Result<Vec<ResultNode>, ExecutionFailure> {
    let mut result = Vec::new();
    for instruction in instructions {
        match instruction {
            Instruction::LiteralElement { name, body, .. } => {
                result.push(ResultNode::Element {
                    name: name.clone(),
                    children: execute_sequence(body, source, context, request_id)?,
                });
            }
            Instruction::Text { value, .. } => append_text(&mut result, value),
            Instruction::ValueOf { select, .. } => {
                let selected = evaluate_child_path(source, context, select);
                if selected.len() > 1 {
                    return Err(failure(
                        "FXRT1001",
                        FailureCategory::Unsupported,
                        Some(request_id),
                        "the private value-of slice does not define multi-node conversion",
                    ));
                }
                let value = selected
                    .first()
                    .map_or_else(String::new, |node| source.string_value(*node));
                append_text(&mut result, &value);
            }
        }
    }
    Ok(result)
}

fn append_text(nodes: &mut Vec<ResultNode>, value: &str) {
    if value.is_empty() {
        return;
    }
    if let Some(ResultNode::Text(existing)) = nodes.last_mut() {
        existing.push_str(value);
    } else {
        nodes.push(ResultNode::Text(value.to_owned()));
    }
}

fn serialize_xml(
    result: &SemanticResult,
    settings: &OutputSettings,
    request_id: &str,
    byte_limit: usize,
) -> Result<String, ExecutionFailure> {
    let first_significant = result.children.iter().find(|node| match node {
        ResultNode::Text(value) => !value.chars().all(char::is_whitespace),
        ResultNode::Element { .. } => true,
    });
    let inferred_html = settings.method.is_none()
        && matches!(
            first_significant,
            Some(ResultNode::Element { name, .. })
                if name.namespace.is_none() && name.local.eq_ignore_ascii_case("html")
        );
    if inferred_html
        || settings
            .method
            .as_deref()
            .is_some_and(|method| method != "xml")
    {
        return Err(failure(
            "FXSR1001",
            FailureCategory::Unsupported,
            Some(request_id),
            "the selected output method is outside the private XML serialization slice",
        ));
    }
    let mut output = BudgetedString::new(byte_limit, request_id);
    if !settings.omit_xml_declaration {
        output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    }
    for node in &result.children {
        serialize_node(node, &mut output)?;
    }
    Ok(output.finish())
}

fn serialize_node(node: &ResultNode, output: &mut BudgetedString) -> Result<(), ExecutionFailure> {
    match node {
        ResultNode::Text(value) => escape_text(value, output)?,
        ResultNode::Element { name, children } => {
            if name.namespace.is_some() {
                return Err(failure(
                    "FXSR1002",
                    FailureCategory::Unsupported,
                    Some(&output.request_id),
                    "namespaced result serialization is outside the private slice",
                ));
            }
            output.push('<')?;
            output.push_str(&name.local)?;
            output.push('>')?;
            for child in children {
                serialize_node(child, output)?;
            }
            output.push_str("</")?;
            output.push_str(&name.local)?;
            output.push('>')?;
        }
    }
    Ok(())
}

fn escape_text(value: &str, output: &mut BudgetedString) -> Result<(), ExecutionFailure> {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;")?,
            '<' => output.push_str("&lt;")?,
            '>' => output.push_str("&gt;")?,
            _ => output.push(character)?,
        }
    }
    Ok(())
}

struct BudgetedString {
    value: String,
    byte_limit: usize,
    request_id: String,
}

impl BudgetedString {
    fn new(byte_limit: usize, request_id: &str) -> Self {
        Self {
            value: String::new(),
            byte_limit,
            request_id: request_id.to_owned(),
        }
    }

    fn push_str(&mut self, value: &str) -> Result<(), ExecutionFailure> {
        let attempted = self.value.len().checked_add(value.len()).ok_or_else(|| {
            failure(
                "FXSR0001",
                FailureCategory::Limit,
                Some(&self.request_id),
                "serialized result byte count overflowed",
            )
        })?;
        if attempted > self.byte_limit {
            return Err(failure(
                "FXSR0002",
                FailureCategory::Limit,
                Some(&self.request_id),
                format!(
                    "serialized result requires at least {attempted} bytes; limit is {}",
                    self.byte_limit
                ),
            ));
        }
        self.value.push_str(value);
        Ok(())
    }

    fn push(&mut self, character: char) -> Result<(), ExecutionFailure> {
        let mut encoded = [0_u8; 4];
        self.push_str(character.encode_utf8(&mut encoded))
    }

    fn finish(self) -> String {
        self.value
    }
}

fn failure(
    code: &'static str,
    category: FailureCategory,
    request_id: Option<&str>,
    detail: impl Into<String>,
) -> ExecutionFailure {
    ExecutionFailure {
        code,
        category,
        request_id: request_id.map(str::to_owned),
        detail: detail.into(),
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, fs, path::PathBuf};

    use crate::resources::{ResourceLimits, ResourceSetBuilder};
    use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind};
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    use super::{
        ExecutionPolicy, FailureCategory, ResultNode, SemanticResult, TransformRequest,
        TransformSetBuilder, compile_resource, execute_transform_set, serialize_xml,
    };

    const SOURCE_ID: &str = "urn:fastxslt:golden:hello:source";
    const STYLESHEET_ID: &str = "urn:fastxslt:golden:hello:stylesheet";

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
            source_resource: source_id.to_owned(),
        }
    }

    fn policy(serialized_byte_limit: usize) -> ExecutionPolicy {
        ExecutionPolicy {
            denied_sources: HashSet::new(),
            serialized_byte_limit,
        }
    }

    fn suite_test_set() -> (Document, PathBuf) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../vendor/xslt30-test/tests/decl/template/_template-test-set.xml");
        let bytes = fs::read(&path).expect("read pinned XSLT30 test set and close handle");
        let parsed = parse_document(
            "urn:w3c:xslt30:decl:template:test-set",
            &bytes,
            ParseLimits {
                max_events: 4_096,
                max_depth: 64,
            },
        )
        .expect("parse pinned XSLT30 test set");
        (
            Document::from_parsed(parsed).expect("build test-set document"),
            path,
        )
    }

    fn attribute<'a>(document: &'a Document, node: NodeId, local: &str) -> Option<&'a str> {
        document
            .attributes(node)
            .iter()
            .copied()
            .find(|attribute| {
                document
                    .name(*attribute)
                    .is_some_and(|name| name.local == local)
            })
            .and_then(|attribute| document.value(attribute))
    }

    fn find_element(
        document: &Document,
        parent: NodeId,
        local: &str,
        required_attribute: Option<(&str, &str)>,
    ) -> Option<NodeId> {
        for child in document.children(parent).iter().copied() {
            if document.kind(child) != NodeKind::Element {
                continue;
            }
            let matches_name = document.name(child).is_some_and(|name| name.local == local);
            let matches_attribute = required_attribute
                .is_none_or(|(name, value)| attribute(document, child, name) == Some(value));
            if matches_name && matches_attribute {
                return Some(child);
            }
            if let Some(found) = find_element(document, child, local, required_attribute) {
                return Some(found);
            }
        }
        None
    }

    fn assert_same_empty_document_element(actual: &str, expected: &str) {
        let limits = ParseLimits {
            max_events: 32,
            max_depth: 8,
        };
        let actual = Document::from_parsed(
            parse_document("urn:fastxslt:actual", actual.as_bytes(), limits)
                .expect("actual result should parse"),
        )
        .expect("actual result document should build");
        let expected = Document::from_parsed(
            parse_document("urn:w3c:expected", expected.as_bytes(), limits)
                .expect("expected result should parse"),
        )
        .expect("expected result document should build");
        let actual_root = find_element(&actual, actual.document_node(), "o", None)
            .expect("actual document element");
        let expected_root = find_element(&expected, expected.document_node(), "o", None)
            .expect("expected document element");

        assert_eq!(actual.name(actual_root), expected.name(expected_root));
        assert!(actual.children(actual_root).is_empty());
        assert!(expected.children(expected_root).is_empty());
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
    fn executes_pinned_xslt30_template_006_from_its_upstream_test_set() {
        const CASE_NAME: &str = "template-006";
        let overlay = include_str!("../../../../corpus/overlays/xslt30/private-slice-v0.toml");
        assert!(overlay.contains("case_name = \"template-006\""));

        let (test_set, set_path) = suite_test_set();
        let test_case = find_element(
            &test_set,
            test_set.document_node(),
            "test-case",
            Some(("name", CASE_NAME)),
        )
        .expect("overlay case should exist in pinned suite");
        let environment_ref = find_element(&test_set, test_case, "environment", None)
            .and_then(|node| attribute(&test_set, node, "ref"))
            .expect("case should reference an environment");
        let environment = find_element(
            &test_set,
            test_set.document_node(),
            "environment",
            Some(("name", environment_ref)),
        )
        .expect("referenced environment should exist");
        let source = find_element(&test_set, environment, "content", None)
            .map(|node| test_set.string_value(node))
            .expect("environment should contain the principal source");
        let stylesheet_file = find_element(&test_set, test_case, "stylesheet", None)
            .and_then(|node| attribute(&test_set, node, "file"))
            .expect("case should name a stylesheet");
        let expected = find_element(&test_set, test_case, "assert-xml", None)
            .map(|node| test_set.string_value(node))
            .expect("case should provide an XML assertion");
        let stylesheet = fs::read(
            set_path
                .parent()
                .expect("test set should have a directory")
                .join(stylesheet_file),
        )
        .expect("read upstream stylesheet and close handle");

        let source_id = "urn:w3c:xslt30:template-006:source";
        let stylesheet_id = "urn:w3c:xslt30:template-006:stylesheet";
        let mut resources = ResourceSetBuilder::new(ResourceLimits::new(2, 4_096, 8_192));
        resources
            .admit(source_id, source.into_bytes())
            .expect("admit upstream source");
        resources
            .admit(stylesheet_id, stylesheet)
            .expect("admit upstream stylesheet");
        let snapshot = resources.seal();
        let program = compile_resource(&snapshot, stylesheet_id).expect("compile suite case");
        assert_eq!(program.output.method, None);
        let mut set = TransformSetBuilder::new(snapshot, program, 1, policy(4_096));
        set.add(request(CASE_NAME, "result:template-006", source_id))
            .expect("admit suite request");

        let results = execute_transform_set(set.seal()).expect("execute suite case");
        let actual = &results.by_request[CASE_NAME].serialized;

        assert_eq!(actual, "<?xml version=\"1.0\" encoding=\"UTF-8\"?><o></o>");
        assert_same_empty_document_element(actual, expected.trim());
    }

    #[test]
    fn absent_output_declaration_does_not_silently_apply_html_serialization() {
        let result = SemanticResult {
            children: vec![ResultNode::Element {
                name: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: "html".to_owned(),
                },
                children: Vec::new(),
            }],
        };
        let settings = crate::xslt::golden_semantics_experiment::OutputSettings {
            method: None,
            omit_xml_declaration: false,
        };

        let failure = serialize_xml(&result, &settings, "html-result", 4_096)
            .expect_err("adaptive HTML output remains unsupported");

        assert_eq!(failure.code, "FXSR1001");
        assert_eq!(failure.category, FailureCategory::Unsupported);
        assert_eq!(failure.request_id.as_deref(), Some("html-result"));
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
    }
}
