use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::xpath::path_experiment::{PathFailure, parse_child_path};
use crate::xslt::golden_semantics_experiment::{
    ElementTemplate, Instruction, OutputSettings, StylesheetProgram, Template,
};

const XSLT_NAMESPACE: &str = "http://www.w3.org/1999/XSL/Transform";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompileCategory {
    Invalid,
    Unsupported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompileFailure {
    pub(crate) code: &'static str,
    pub(crate) category: CompileCategory,
    pub(crate) detail: String,
    pub(crate) location: SourceLocation,
}

pub(crate) fn compile_stylesheet(document: &Document) -> Result<StylesheetProgram, CompileFailure> {
    let root = document_element(document)?;
    require_name(document, root, Some(XSLT_NAMESPACE), "stylesheet")?;
    let declared_version = required_attribute(document, root, None, "version")?.to_owned();

    let mut output = None;
    let mut root_template = None;
    let mut element_templates = Vec::new();
    for child in meaningful_children(document, root) {
        let Some(name) = document.name(child) else {
            continue;
        };
        match (name.namespace.as_deref(), name.local.as_str()) {
            (Some(XSLT_NAMESPACE), "output") => {
                if output.is_some() {
                    return Err(invalid(
                        "FXST0002",
                        "the private slice permits one xsl:output declaration",
                        document.location(child),
                    ));
                }
                output = Some(compile_output(document, child)?);
            }
            (Some(XSLT_NAMESPACE), "template") => {
                if let Some(name) = optional_attribute(document, child, None, "name") {
                    return Err(unsupported(
                        "FXST1010",
                        format!("named templates are outside the private slice: {name}"),
                        document.location(child),
                    ));
                }
                let pattern = required_attribute(document, child, None, "match")?;
                if pattern == "/" {
                    if root_template.is_some() {
                        return Err(unsupported(
                            "FXST1001",
                            "the private slice permits one root template",
                            document.location(child),
                        ));
                    }
                    root_template = Some(compile_template(document, child)?);
                } else {
                    let element_template = compile_element_template(document, child, pattern)?;
                    if element_templates.iter().any(|existing: &ElementTemplate| {
                        existing.match_name == element_template.match_name
                    }) {
                        return Err(unsupported(
                            "FXST1008",
                            format!(
                                "template priority for duplicate match pattern is outside the private slice: {pattern}"
                            ),
                            document.location(child),
                        ));
                    }
                    element_templates.push(element_template);
                }
            }
            (Some(XSLT_NAMESPACE), local) => {
                return Err(unsupported(
                    "FXST1002",
                    format!("unsupported top-level XSLT declaration: xsl:{local}"),
                    document.location(child),
                ));
            }
            _ => {
                return Err(unsupported(
                    "FXST1003",
                    "literal top-level elements are outside the private slice",
                    document.location(child),
                ));
            }
        }
    }

    Ok(StylesheetProgram {
        declared_version,
        output: output.unwrap_or(OutputSettings {
            method: None,
            omit_xml_declaration: false,
        }),
        root_template: root_template.ok_or_else(|| {
            invalid(
                "FXST0004",
                "the private slice requires a root template",
                document.location(root),
            )
        })?,
        element_templates,
    })
}

fn compile_output(document: &Document, element: NodeId) -> Result<OutputSettings, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &["method", "omit-xml-declaration"],
        "xsl:output",
    )?;
    ensure_no_meaningful_children(document, element, "xsl:output")?;
    let method = required_attribute(document, element, None, "method")?;
    if method != "xml" {
        return Err(unsupported(
            "FXST1004",
            format!("unsupported output method: {method}"),
            document.location(element),
        ));
    }
    let omit = required_attribute(document, element, None, "omit-xml-declaration")?;
    let omit_xml_declaration = match omit {
        "yes" => true,
        "no" => false,
        _ => {
            return Err(invalid(
                "FXST0005",
                "omit-xml-declaration must be 'yes' or 'no'",
                document.location(element),
            ));
        }
    };
    Ok(OutputSettings {
        method: Some(method.to_owned()),
        omit_xml_declaration,
    })
}

fn compile_element_template(
    document: &Document,
    element: NodeId,
    pattern: &str,
) -> Result<ElementTemplate, CompileFailure> {
    if !is_ascii_ncname(pattern) {
        return Err(unsupported(
            "FXST1005",
            format!("unsupported template match pattern: {pattern}"),
            document.location(element),
        ));
    }
    Ok(ElementTemplate {
        match_name: crate::xml::quick_xml_experiment::ExpandedName {
            namespace: None,
            local: pattern.to_owned(),
        },
        template: compile_template(document, element)?,
    })
}

fn compile_template(document: &Document, element: NodeId) -> Result<Template, CompileFailure> {
    ensure_only_attributes(document, element, &["match"], "xsl:template")?;
    Ok(Template {
        body: compile_sequence(document, element)?,
        location: document.location(element).clone(),
    })
}

fn compile_sequence(
    document: &Document,
    parent: NodeId,
) -> Result<Vec<Instruction>, CompileFailure> {
    let mut instructions = Vec::new();
    for child in document.children(parent).iter().copied() {
        match document.kind(child) {
            NodeKind::Text => {
                let value = document.value(child).unwrap_or_default();
                if !value.chars().all(char::is_whitespace) {
                    instructions.push(Instruction::Text {
                        value: value.to_owned(),
                        location: document.location(child).clone(),
                    });
                }
            }
            NodeKind::Comment | NodeKind::ProcessingInstruction => {}
            NodeKind::Element => {
                let name = document.name(child).expect("element nodes have names");
                if name.namespace.as_deref() == Some(XSLT_NAMESPACE) {
                    if name.local == "value-of" {
                        instructions.push(compile_value_of(document, child)?);
                    } else if name.local == "apply-templates" {
                        instructions.push(compile_apply_templates(document, child)?);
                    } else {
                        return Err(unsupported(
                            "FXST1006",
                            format!("unsupported XSLT instruction: xsl:{}", name.local),
                            document.location(child),
                        ));
                    }
                } else {
                    if !document.attributes(child).is_empty() {
                        return Err(unsupported(
                            "FXST1007",
                            "literal result attributes are outside the private slice",
                            document.location(child),
                        ));
                    }
                    instructions.push(Instruction::LiteralElement {
                        name: name.clone(),
                        body: compile_sequence(document, child)?,
                        location: document.location(child).clone(),
                    });
                }
            }
            NodeKind::Document | NodeKind::Attribute => {
                return Err(invalid(
                    "FXST0006",
                    "unexpected node kind in stylesheet sequence",
                    document.location(child),
                ));
            }
        }
    }
    Ok(instructions)
}

fn compile_apply_templates(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["select"], "xsl:apply-templates")?;
    ensure_no_meaningful_children(document, element, "xsl:apply-templates")?;
    let location = document.location(element).clone();
    let select = optional_attribute(document, element, None, "select")
        .map(|expression| parse_child_path(expression, location.clone()).map_err(map_path_failure))
        .transpose()?;
    Ok(Instruction::ApplyTemplates { select, location })
}

fn compile_value_of(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["select"], "xsl:value-of")?;
    ensure_no_meaningful_children(document, element, "xsl:value-of")?;
    let location = document.location(element).clone();
    let expression = required_attribute(document, element, None, "select")?;
    let select = parse_child_path(expression, location.clone()).map_err(map_path_failure)?;
    Ok(Instruction::ValueOf { select, location })
}

fn ensure_only_attributes(
    document: &Document,
    element: NodeId,
    allowed: &[&str],
    display_name: &str,
) -> Result<(), CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document
            .name(*attribute)
            .expect("attribute nodes have expanded names");
        if name.namespace.is_some() || !allowed.contains(&name.local.as_str()) {
            return Err(unsupported(
                "FXST1009",
                format!(
                    "unsupported attribute on {display_name}: {{{}}}{}",
                    name.namespace.as_deref().unwrap_or(""),
                    name.local
                ),
                document.location(*attribute),
            ));
        }
    }
    Ok(())
}

fn document_element(document: &Document) -> Result<NodeId, CompileFailure> {
    let root = document.document_node();
    let elements: Vec<_> = document
        .children(root)
        .iter()
        .copied()
        .filter(|node| document.kind(*node) == NodeKind::Element)
        .collect();
    if let [element] = elements.as_slice() {
        return Ok(*element);
    }
    Err(invalid(
        "FXST0001",
        "a stylesheet document must contain exactly one document element",
        document.location(root),
    ))
}

fn meaningful_children(document: &Document, parent: NodeId) -> Vec<NodeId> {
    document
        .children(parent)
        .iter()
        .copied()
        .filter(|child| match document.kind(*child) {
            NodeKind::Comment | NodeKind::ProcessingInstruction => false,
            NodeKind::Text => !document
                .value(*child)
                .unwrap_or_default()
                .chars()
                .all(char::is_whitespace),
            _ => true,
        })
        .collect()
}

fn ensure_no_meaningful_children(
    document: &Document,
    element: NodeId,
    display_name: &str,
) -> Result<(), CompileFailure> {
    if meaningful_children(document, element).is_empty() {
        Ok(())
    } else {
        Err(invalid(
            "FXST0007",
            format!("{display_name} must be empty in the private slice"),
            document.location(element),
        ))
    }
}

fn required_attribute<'a>(
    document: &'a Document,
    element: NodeId,
    namespace: Option<&str>,
    local: &str,
) -> Result<&'a str, CompileFailure> {
    document
        .attributes(element)
        .iter()
        .copied()
        .find(|attribute| {
            document
                .name(*attribute)
                .is_some_and(|name| name.namespace.as_deref() == namespace && name.local == local)
        })
        .and_then(|attribute| document.value(attribute))
        .ok_or_else(|| {
            invalid(
                "FXST0008",
                format!("missing required attribute: {local}"),
                document.location(element),
            )
        })
}

fn optional_attribute<'a>(
    document: &'a Document,
    element: NodeId,
    namespace: Option<&str>,
    local: &str,
) -> Option<&'a str> {
    document
        .attributes(element)
        .iter()
        .copied()
        .find(|attribute| {
            document
                .name(*attribute)
                .is_some_and(|name| name.namespace.as_deref() == namespace && name.local == local)
        })
        .and_then(|attribute| document.value(attribute))
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}

fn require_name(
    document: &Document,
    node: NodeId,
    namespace: Option<&str>,
    local: &str,
) -> Result<(), CompileFailure> {
    if document
        .name(node)
        .is_some_and(|name| name.namespace.as_deref() == namespace && name.local == local)
    {
        Ok(())
    } else {
        Err(invalid(
            "FXST0009",
            format!("expected element: {{{}}}{local}", namespace.unwrap_or("")),
            document.location(node),
        ))
    }
}

fn map_path_failure(failure: PathFailure) -> CompileFailure {
    match failure {
        PathFailure::Invalid { detail, location } => CompileFailure {
            code: "FXXP0001",
            category: CompileCategory::Invalid,
            detail,
            location,
        },
        PathFailure::Unsupported { detail, location } => CompileFailure {
            code: "FXXP1001",
            category: CompileCategory::Unsupported,
            detail,
            location,
        },
    }
}

fn invalid(
    code: &'static str,
    detail: impl Into<String>,
    location: &SourceLocation,
) -> CompileFailure {
    CompileFailure {
        code,
        category: CompileCategory::Invalid,
        detail: detail.into(),
        location: location.clone(),
    }
}

fn unsupported(
    code: &'static str,
    detail: impl Into<String>,
    location: &SourceLocation,
) -> CompileFailure {
    CompileFailure {
        code,
        category: CompileCategory::Unsupported,
        detail: detail.into(),
        location: location.clone(),
    }
}

#[cfg(test)]
mod tests {
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
    use crate::xslt::golden_semantics_experiment::Instruction;

    use super::{CompileCategory, compile_stylesheet};

    const LIMITS: ParseLimits = ParseLimits {
        max_events: 256,
        max_depth: 32,
    };

    fn parse_stylesheet(resource: &str, bytes: &[u8]) -> Document {
        let parsed = parse_document(resource, bytes, LIMITS).expect("stylesheet XML should parse");
        Document::from_parsed(parsed).expect("stylesheet XDM should build")
    }

    #[test]
    fn compiles_the_golden_stylesheet_into_owned_semantics() {
        let bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/golden/hello/stylesheet.xsl"
        ));
        let document = parse_stylesheet("golden:hello/stylesheet.xsl", bytes);

        let program = compile_stylesheet(&document).expect("golden stylesheet should compile");

        assert_eq!(program.declared_version, "1.0");
        assert_eq!(program.output.method.as_deref(), Some("xml"));
        assert!(program.output.omit_xml_declaration);
        let [Instruction::LiteralElement { name, body, .. }] =
            program.root_template.body.as_slice()
        else {
            panic!("root template should contain one literal result element");
        };
        assert_eq!(name.namespace, None);
        assert_eq!(name.local, "message");
        assert!(matches!(
            body.as_slice(),
            [
                Instruction::Text { value: first, .. },
                Instruction::ValueOf { select, .. },
                Instruction::Text { value: last, .. }
            ] if first == "Hello, " && select.steps == ["greeting", "name"] && last == "!"
        ));
        assert_eq!(
            program.root_template.location.resource,
            "golden:hello/stylesheet.xsl"
        );
    }

    #[test]
    fn preserves_absent_output_declaration_for_runtime_method_inference() {
        let stylesheet = parse_stylesheet(
            "memory:default-output.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><o/></xsl:template></xsl:stylesheet>"#,
        );

        let program = compile_stylesheet(&stylesheet).expect("stylesheet should compile");

        assert_eq!(program.output.method, None);
        assert!(!program.output.omit_xml_declaration);
    }

    #[test]
    fn compiles_exact_element_template_dispatch_without_priority_semantics() {
        let bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../corpus/golden/template-dispatch/stylesheet.xsl"
        ));
        let document = parse_stylesheet("golden:template-dispatch/stylesheet.xsl", bytes);

        let program = compile_stylesheet(&document).expect("dispatch stylesheet should compile");

        assert_eq!(program.element_templates.len(), 1);
        assert_eq!(program.element_templates[0].match_name.local, "item");
        assert!(matches!(
            program.root_template.body.as_slice(),
            [Instruction::LiteralElement { body, .. }]
                if matches!(body.as_slice(), [Instruction::ApplyTemplates { select: Some(select), .. }]
                    if select.steps == ["catalog", "item"])
        ));

        let duplicate = parse_stylesheet(
            "memory:duplicate-pattern.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><out/></xsl:template><xsl:template match="item"><a/></xsl:template><xsl:template match="item"><b/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&duplicate)
            .expect_err("priority conflict must remain visibly unsupported");
        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.code, "FXST1008");

        let mode = parse_stylesheet(
            "memory:mode.xsl",
            br#"<xsl:stylesheet version="3.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template match="/"><xsl:apply-templates select="root/item" mode="detail"/></xsl:template><xsl:template match="item"><out/></xsl:template></xsl:stylesheet>"#,
        );
        let failure =
            compile_stylesheet(&mode).expect_err("mode semantics must remain unsupported");
        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.code, "FXST1009");
    }

    #[test]
    fn distinguishes_invalid_stylesheet_from_unsupported_instruction() {
        let invalid = parse_stylesheet(
            "memory:invalid.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><xsl:value-of/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&invalid).expect_err("missing select should fail");
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.code, "FXST0008");
        assert_eq!(failure.location.resource, "memory:invalid.xsl");

        let unsupported = parse_stylesheet(
            "memory:unsupported.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><xsl:for-each select="item"/></xsl:template></xsl:stylesheet>"#,
        );
        let failure =
            compile_stylesheet(&unsupported).expect_err("unsupported instruction should fail");
        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.code, "FXST1006");
        assert_eq!(failure.location.resource, "memory:unsupported.xsl");

        let named_template = parse_stylesheet(
            "memory:named-template.xsl",
            br#"<xsl:stylesheet version="2.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:template name="worker"><out/></xsl:template><xsl:template match="/"><xsl:call-template name="worker"/></xsl:template></xsl:stylesheet>"#,
        );
        let failure = compile_stylesheet(&named_template)
            .expect_err("valid named-template syntax should be visibly unsupported");
        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.code, "FXST1010");
    }

    #[test]
    fn classifies_xpath_outside_the_private_child_path_slice_as_unsupported() {
        let stylesheet = parse_stylesheet(
            "memory:path.xsl",
            br#"<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform"><xsl:output method="xml" omit-xml-declaration="yes"/><xsl:template match="/"><value><xsl:value-of select="greeting//name"/></value></xsl:template></xsl:stylesheet>"#,
        );

        let failure = compile_stylesheet(&stylesheet).expect_err("unsupported XPath should fail");

        assert_eq!(failure.category, CompileCategory::Unsupported);
        assert_eq!(failure.code, "FXXP1001");
        assert_eq!(failure.location.resource, "memory:path.xsl");
    }
}
