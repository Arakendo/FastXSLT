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
}

#[derive(Debug)]
struct TransformSet {
    snapshot: ResourceSnapshot,
    stylesheet: StylesheetProgram,
    requests: Vec<TransformRequest>,
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
    ) -> Self {
        Self {
            snapshot,
            stylesheet,
            requests: Vec::new(),
            request_ids: HashSet::new(),
            result_ids: HashSet::new(),
            request_limit,
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
        let serialized = serialize_xml(&semantic, &set.stylesheet.output, &request.identity)?;
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
) -> Result<String, ExecutionFailure> {
    if settings.method != "xml" {
        return Err(failure(
            "FXSR1001",
            FailureCategory::Unsupported,
            Some(request_id),
            "only XML serialization is available in the private slice",
        ));
    }
    let mut output = String::new();
    if !settings.omit_xml_declaration {
        output.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    }
    for node in &result.children {
        serialize_node(node, &mut output, request_id)?;
    }
    Ok(output)
}

fn serialize_node(
    node: &ResultNode,
    output: &mut String,
    request_id: &str,
) -> Result<(), ExecutionFailure> {
    match node {
        ResultNode::Text(value) => escape_text(value, output),
        ResultNode::Element { name, children } => {
            if name.namespace.is_some() {
                return Err(failure(
                    "FXSR1002",
                    FailureCategory::Unsupported,
                    Some(request_id),
                    "namespaced result serialization is outside the private slice",
                ));
            }
            output.push('<');
            output.push_str(&name.local);
            output.push('>');
            for child in children {
                serialize_node(child, output, request_id)?;
            }
            output.push_str("</");
            output.push_str(&name.local);
            output.push('>');
        }
    }
    Ok(())
}

fn escape_text(value: &str, output: &mut String) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            _ => output.push(character),
        }
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
    use crate::resources::{ResourceLimits, ResourceSetBuilder};

    use super::{
        FailureCategory, ResultNode, TransformRequest, TransformSetBuilder, compile_resource,
        execute_transform_set,
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

    #[test]
    fn golden_transform_executes_through_an_unordered_identified_set() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 4);
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
        let mut builder = TransformSetBuilder::new(snapshot, program, 1);
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
    fn builder_rejects_duplicates_limits_and_unadmitted_sibling_results() {
        let snapshot = snapshot();
        let program = compile_resource(&snapshot, STYLESHEET_ID).expect("compile once");
        let mut builder = TransformSetBuilder::new(snapshot, program, 2);
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
}
