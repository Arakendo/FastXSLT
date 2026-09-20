//! Private compilation of XSLT sequence constructors and instructions.

use crate::xdm::atomic_value_experiment::{AtomicValue, BuiltinAtomicType};
use crate::xdm::owned_tree_experiment::{Document, NodeId, NodeKind, SourceLocation};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::case_conversion_experiment::{
    CaseConversionParseFailure, parse as parse_case_conversion,
    recognizes as recognizes_case_conversion,
};
use crate::xpath::castable_experiment::{parse as parse_castable, parse_cast};
use crate::xpath::constant_boolean_experiment::{
    BooleanParseFailure, ScalarExpression, parse_literal_comparison,
    parse_scalar as parse_source_free_scalar, recognizes_scalar as recognizes_source_free_scalar,
};
use crate::xpath::context_requirement_experiment::classify as classify_missing_context;
use crate::xpath::decimal_sum_for_experiment::parse as parse_decimal_sum_for;
use crate::xpath::deep_equal_boolean_experiment::{
    parse as parse_deep_equal, recognizes as recognizes_deep_equal,
};
use crate::xpath::deep_equal_experiment::DeepEqualFailureKind;
use crate::xpath::default_collation_experiment::{
    DefaultCollationParseFailure, parse as parse_default_collation,
    recognizes as recognizes_default_collation,
};
use crate::xpath::duration_component_experiment::{
    DurationComponentParseFailure, parse as parse_duration_component,
    recognizes as recognizes_duration_component,
};
use crate::xpath::effective_boolean_value_experiment::{
    EffectiveBooleanFailure, parse as parse_document_boolean,
    recognizes as recognizes_document_boolean,
};
use crate::xpath::empty_experiment::{
    SequenceCardinalityParseFailure, parse_source_free as parse_sequence_cardinality,
    recognizes_source_free as recognizes_sequence_cardinality,
};
use crate::xpath::encode_for_uri_expression::{
    EncodeForUriParseFailure, parse as parse_encode_for_uri,
    recognizes as recognizes_encode_for_uri,
};
use crate::xpath::escape_html_uri_expression::{
    EscapeHtmlUriParseFailure, parse as parse_escape_html_uri,
    recognizes as recognizes_escape_html_uri,
};
use crate::xpath::focus_sum_for_experiment::parse as parse_focus_sum_for;
use crate::xpath::for_distinct_values_experiment::{
    ForExpressionFailure, parse as parse_for_distinct_values,
};
use crate::xpath::format_number_experiment::{
    FormatNumberFailureKind, parse as parse_format_number,
};
use crate::xpath::integer_for_experiment::parse as parse_integer_for;
use crate::xpath::iri_to_uri_expression::{
    IriToUriParseFailure, parse as parse_iri_to_uri, recognizes as recognizes_iri_to_uri,
};
use crate::xpath::path_experiment::{
    LocationPath, PathFailure, PathStep, parse_location_path, parse_qualified_child_path,
    parse_xslt10_location_path,
};
use crate::xpath::path_operand_type_experiment::classify as classify_atomic_path_operand;
use crate::xpath::string_length_experiment::{
    StringLengthParseFailure, parse as parse_string_length, recognizes as recognizes_string_length,
};
use crate::xslt::golden_semantics_experiment::{
    BooleanExpression, ChooseBranch, ComputedAttribute, DynamicElementName,
    ElementConstructorOrigin, FocusEqualityOperand, Instruction, LiteralAttributeValue,
    SequenceItemExpression, SortDataType, SortKey, SortOrder, SortSelect, StringComparison,
    TemplateArgument, ValueExpression,
};

#[path = "instruction_compiler/computed_attribute_compiler.rs"]
mod computed_attribute_compiler;
use computed_attribute_compiler::{compile_computed_attribute, compile_computed_attributes};
#[path = "instruction_compiler/boolean_expression_compiler.rs"]
mod boolean_expression_compiler;
#[path = "instruction_compiler/conditional_expression_compiler.rs"]
mod conditional_expression_compiler;
#[path = "instruction_compiler/value_expression_compiler.rs"]
mod value_expression_compiler;
#[path = "instruction_compiler/xslt10_static_introspection_compiler.rs"]
mod xslt10_static_introspection_compiler;
pub(super) use value_expression_compiler::compile_value_expression;
use value_expression_compiler::generated_root_argument;
#[path = "instruction_compiler/literal_attribute_compiler.rs"]
mod literal_attribute_compiler;
pub(super) use literal_attribute_compiler::compile_literal_result_attributes;
#[path = "instruction_compiler/number_compiler.rs"]
mod number_compiler;
use number_compiler::compile as compile_number;
#[path = "instruction_compiler/source_copy_compiler.rs"]
mod source_copy_compiler;
use source_copy_compiler::compile_copy;
#[path = "instruction_compiler/attribute_set_compiler.rs"]
mod attribute_set_compiler;
#[path = "instruction_compiler/template_invocation_compiler.rs"]
mod template_invocation_compiler;
use attribute_set_compiler::compile_local_attribute_sets;

fn parse_context_focus_equality(
    expression: &str,
) -> Option<(FocusEqualityOperand, FocusEqualityOperand)> {
    let (left, right) = expression.trim().split_once('=')?;
    if left.contains(['=', '!']) || right.contains('=') {
        return None;
    }
    let left = parse_context_focus_operand(left.trim())?;
    let right = parse_context_focus_operand(right.trim())?;
    (!matches!(left, FocusEqualityOperand::Static(_))
        || !matches!(right, FocusEqualityOperand::Static(_)))
    .then_some((left, right))
}

fn parse_context_focus_operand(operand: &str) -> Option<FocusEqualityOperand> {
    match operand {
        "position()" => Some(FocusEqualityOperand::Position),
        "last()" => Some(FocusEqualityOperand::Size),
        _ if parse_ceiling_half_size(operand) => Some(FocusEqualityOperand::CeilingHalfSize),
        _ => operand.parse().ok().map(FocusEqualityOperand::Static),
    }
}

fn parse_ceiling_half_size(operand: &str) -> bool {
    let Some(inner) = operand
        .strip_prefix("ceiling(")
        .and_then(|value| value.strip_suffix(')'))
    else {
        return false;
    };
    let Some((left, right)) = inner.split_once("div") else {
        return false;
    };
    left.trim() == "last()" && right.trim() == "2"
}

pub(super) fn parse_mode(
    document: &Document,
    element: NodeId,
    mode: &str,
) -> Result<String, CompileFailure> {
    template_invocation_compiler::parse_mode(document, element, mode)
}

use super::{
    CompileCategory, CompileFailure, XML_SCHEMA_NAMESPACE, XSLT_NAMESPACE, effective_default_mode,
    effective_xpath_default_namespace, ensure_no_meaningful_children, ensure_only_attributes,
    invalid, is_ascii_ncname, is_xslt_element, map_path_failure, meaningful_children,
    normalize_named_template_name, normalize_variable_qname, optional_attribute,
    required_attribute, unsupported,
};

pub(super) fn validate_local_attribute_set(
    document: &Document,
    element: NodeId,
) -> Result<ExpandedName, CompileFailure> {
    attribute_set_compiler::validate_local_attribute_set(document, element)
}

pub(super) fn validate_local_attribute_set_graph(
    document: &Document,
    stylesheet: NodeId,
) -> Result<(), CompileFailure> {
    attribute_set_compiler::validate_local_attribute_set_graph(document, stylesheet)
}

fn compile_sequence(
    document: &Document,
    parent: NodeId,
) -> Result<Vec<Instruction>, CompileFailure> {
    compile_sequence_excluding(document, parent, &[])
}

pub(super) fn compile_sequence_excluding(
    document: &Document,
    parent: NodeId,
    excluded: &[NodeId],
) -> Result<Vec<Instruction>, CompileFailure> {
    compile_sequence_excluding_with_bindings(document, parent, excluded, &[])
}

fn xslt10_text_constructor_exclusions(document: &Document, children: &[NodeId]) -> Vec<NodeId> {
    children
        .iter()
        .copied()
        .filter(|child| {
            if document.kind(*child) != NodeKind::Element {
                return false;
            }
            let name = document.name(*child).expect("element nodes have names");
            name.namespace.as_deref() != Some(XSLT_NAMESPACE)
                || matches!(
                    name.local.as_str(),
                    "attribute" | "comment" | "copy" | "element" | "processing-instruction"
                )
        })
        .collect()
}

pub(super) fn compile_sequence_excluding_with_bindings(
    document: &Document,
    parent: NodeId,
    excluded: &[NodeId],
    initial_local_bindings: &[String],
) -> Result<Vec<Instruction>, CompileFailure> {
    let mut instructions = Vec::new();
    let mut local_variables = initial_local_bindings.to_vec();
    let preserve_whitespace = effective_xml_space_preserved(document, parent)?;
    let children = document.children(parent).to_vec();
    for (index, child) in children.iter().copied().enumerate() {
        if excluded.contains(&child) {
            continue;
        }
        match document.kind(child) {
            NodeKind::Text => {
                if let Some(instruction) = compile_literal_text_node(
                    document,
                    child,
                    preserve_whitespace
                        || text_run_contains_non_whitespace(document, &children, index),
                ) {
                    instructions.push(instruction);
                }
            }
            NodeKind::Comment | NodeKind::ProcessingInstruction => {}
            NodeKind::Element => {
                let name = document.name(child).expect("element nodes have names");
                if name.namespace.as_deref() == Some(XSLT_NAMESPACE) {
                    instructions.push(compile_xslt_instruction(
                        document,
                        child,
                        &name.local,
                        &mut local_variables,
                    )?);
                } else {
                    instructions.push(compile_literal_element(document, child)?);
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

fn compile_xslt_instruction(
    document: &Document,
    element: NodeId,
    local_name: &str,
    local_variables: &mut Vec<String>,
) -> Result<Instruction, CompileFailure> {
    let instruction = match local_name {
        "text" => compile_text(document, element)?,
        "element" => compile_static_computed_element(document, element)?,
        "comment" => compile_comment(document, element)?,
        "attribute" => compile_attribute(document, element)?,
        "processing-instruction" => compile_processing_instruction(document, element)?,
        "value-of" => compile_value_of(document, element)?,
        "number" => compile_number(document, element)?,
        "variable" => {
            let variable = compile_variable(document, element)?;
            let name = local_variable_name(&variable);
            if local_variables.contains(name) {
                return Err(invalid(
                    "FXST0017",
                    format!("duplicate local variable binding: ${name}"),
                    document.location(element),
                ));
            }
            local_variables.push(name.clone());
            variable
        }
        "sequence" => compile_sequence_nodes(document, element)?,
        "apply-templates" => compile_apply_templates(document, element)?,
        "next-match" => {
            ensure_only_attributes(document, element, &[], "xsl:next-match")?;
            Instruction::NextMatch {
                arguments: compile_with_params(document, element, "xsl:next-match", true)?,
                location: document.location(element).clone(),
            }
        }
        "apply-imports" => compile_apply_imports(document, element)?,
        "for-each" => compile_for_each(document, element)?,
        "if" => compile_if(document, element)?,
        "choose" => compile_choose(document, element)?,
        "call-template" => compile_call_template(document, element)?,
        "copy" => compile_copy(document, element)?,
        "copy-of" => compile_copy_of(document, element)?,
        _ => {
            return Err(unsupported(
                "FXST1006",
                format!("unsupported XSLT instruction: xsl:{local_name}"),
                document.location(element),
            ));
        }
    };
    Ok(instruction)
}

fn compile_literal_text_node(
    document: &Document,
    node: NodeId,
    preserve_whitespace: bool,
) -> Option<Instruction> {
    let value = document.value(node).unwrap_or_default();
    (preserve_whitespace || !value.chars().all(char::is_whitespace)).then(|| Instruction::Text {
        value: value.to_owned(),
        location: document.location(node).clone(),
    })
}

fn text_run_contains_non_whitespace(
    document: &Document,
    siblings: &[NodeId],
    text_index: usize,
) -> bool {
    let mut start = text_index;
    while start > 0
        && matches!(
            document.kind(siblings[start - 1]),
            NodeKind::Text | NodeKind::Comment | NodeKind::ProcessingInstruction
        )
    {
        start -= 1;
    }
    let mut end = text_index + 1;
    while end < siblings.len()
        && matches!(
            document.kind(siblings[end]),
            NodeKind::Text | NodeKind::Comment | NodeKind::ProcessingInstruction
        )
    {
        end += 1;
    }
    siblings[start..end].iter().any(|sibling| {
        document.kind(*sibling) == NodeKind::Text
            && document
                .value(*sibling)
                .is_some_and(|value| !value.chars().all(char::is_whitespace))
    })
}

fn local_variable_name(variable: &Instruction) -> &String {
    let (Instruction::Variable { name, .. }
    | Instruction::StaticAtomicVariable { name, .. }
    | Instruction::AtomicVariableAlias { name, .. }
    | Instruction::ContextPositionVariable { name, .. }
    | Instruction::ContextNodeNameVariable { name, .. }
    | Instruction::ContextCountPathVariable { name, .. }
    | Instruction::Xslt10BinaryNumericVariable { name, .. }
    | Instruction::SourceNodeVariable { name, .. }
    | Instruction::SourceVariablePathVariable { name, .. }
    | Instruction::SourceNodeUnionVariable { name, .. }
    | Instruction::IntegerRangeVariable { name, .. }
    | Instruction::TemporaryTreeVariable { name, .. }
    | Instruction::Xslt10TextTreeVariable { name, .. }
    | Instruction::Xslt10ValueOfTreeVariable { name, .. }
    | Instruction::Xslt10ForEachTextTreeVariable { name, .. }
    | Instruction::Xslt10SequenceTreeVariable { name, .. }) = variable
    else {
        unreachable!("compile_variable returns a variable instruction")
    };
    name
}

fn compile_attribute(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["name", "select"], "xsl:attribute")?;
    if optional_attribute(document, element, None, "select").is_none() {
        let attribute = compile_computed_attribute(document, element)?;
        return Ok(Instruction::Attribute {
            attribute,
            location: document.location(element).clone(),
        });
    }
    ensure_no_meaningful_children(document, element, "xsl:attribute")?;
    let name = required_attribute(document, element, None, "name")?;
    if !is_ascii_ncname(name) {
        return Err(unsupported(
            "FXST1033",
            "the private standalone xsl:attribute slice requires an unprefixed static NCName",
            document.location(element),
        ));
    }
    let select = required_attribute(document, element, None, "select")?;
    let value = match select.split_whitespace().collect::<String>().as_str() {
        ".+1" => LiteralAttributeValue::ContextIntegerIncrement(1),
        _ => {
            return Err(unsupported(
                "FXXP1012",
                format!("unsupported standalone xsl:attribute value expression: {select}"),
                document.location(element),
            ));
        }
    };
    Ok(Instruction::Attribute {
        attribute: ComputedAttribute {
            name: crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: name.to_owned(),
            },
            dynamic_name: None,
            value,
            location: document.location(element).clone(),
        },
        location: document.location(element).clone(),
    })
}

pub(super) fn compile_literal_element(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    validate_extension_element_prefixes(document, element)?;
    if is_declared_extension_element(document, element) {
        return Err(unsupported(
            "FXST1059",
            "extension element execution and xsl:fallback are outside the admitted slice",
            document.location(element),
        ));
    }
    ensure_literal_result_control_attributes(document, element)?;
    validate_exclude_result_prefixes(document, element)?;
    let (mut computed_attributes, computed_attribute_nodes) =
        compile_computed_attributes(document, element)?;
    let mut attributes = compile_literal_result_attributes(document, element)?;
    if uses_xslt10_compatibility(document, element) {
        attributes.retain(|literal| {
            !computed_attributes
                .iter()
                .any(|computed| computed.dynamic_name.is_none() && computed.name == literal.name)
        });
    } else {
        ensure_distinct_result_attributes(
            &attributes,
            &computed_attributes,
            document.location(element),
        )?;
    }
    let mut attribute_set_values =
        compile_local_attribute_sets(document, element, Some(XSLT_NAMESPACE))?;
    attribute_set_values.retain(|set_attribute| {
        !attributes.iter().any(|attribute| {
            set_attribute.dynamic_name.is_none() && attribute.name == set_attribute.name
        }) && !computed_attributes.iter().any(|attribute| {
            attribute.dynamic_name.is_none()
                && set_attribute.dynamic_name.is_none()
                && attribute.name == set_attribute.name
        })
    });
    attribute_set_values.append(&mut computed_attributes);
    let computed_attributes = attribute_set_values;
    let mut namespaces = literal_result_namespaces(document, element);
    retain_computed_attribute_namespace_bindings(&mut namespaces, &computed_attributes);
    Ok(Instruction::LiteralElement {
        origin: ElementConstructorOrigin::Literal,
        name: document
            .name(element)
            .expect("literal result element has a name")
            .clone(),
        namespaces: namespaces.into(),
        attributes,
        computed_attributes,
        body: compile_sequence_excluding(document, element, &computed_attribute_nodes)?,
        location: document.location(element).clone(),
    })
}

fn is_declared_extension_element(document: &Document, element: NodeId) -> bool {
    let Some(element_namespace) = document
        .name(element)
        .and_then(|name| name.namespace.as_deref())
    else {
        return false;
    };
    let mut current = Some(element);
    while let Some(node) = current {
        let declaration = if document.name(node).is_some_and(|name| {
            name.namespace.as_deref() == Some(XSLT_NAMESPACE)
                && matches!(name.local.as_str(), "stylesheet" | "transform")
        }) {
            optional_attribute(document, node, None, "extension-element-prefixes")
        } else {
            optional_attribute(
                document,
                node,
                Some(XSLT_NAMESPACE),
                "extension-element-prefixes",
            )
        };
        if declaration.is_some_and(|prefixes| {
            prefixes.split_whitespace().any(|prefix| {
                if prefix == "#default" {
                    document.namespace_declarations(node).iter().any(|binding| {
                        binding.prefix.is_none() && binding.namespace == element_namespace
                    })
                } else {
                    namespace_for_prefix(document, node, prefix) == Some(element_namespace)
                }
            })
        }) {
            return true;
        }
        current = document.parent(node);
    }
    false
}

fn compile_static_computed_element(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &["name", "namespace", "use-attribute-sets"],
        "xsl:element",
    )?;
    let name = required_attribute(document, element, None, "name")?;
    let namespace = optional_attribute(document, element, None, "namespace");
    if namespace.is_some_and(|value| value.contains(['{', '}'])) {
        return Err(unsupported(
            "FXST1045",
            "the private xsl:element namespace slice requires a static URI",
            document.location(element),
        ));
    }
    if matches!(name.trim(), "{name()}" | "{name(.)}") {
        let (mut computed_attributes, computed_attribute_nodes) =
            compile_computed_attributes(document, element)?;
        let mut attribute_set_values = compile_local_attribute_sets(document, element, None)?;
        attribute_set_values.retain(|set_attribute| {
            !computed_attributes.iter().any(|attribute| {
                attribute.dynamic_name.is_none()
                    && set_attribute.dynamic_name.is_none()
                    && attribute.name == set_attribute.name
            })
        });
        attribute_set_values.append(&mut computed_attributes);
        return Ok(Instruction::ContextNameElement {
            namespace_override: namespace.map(str::to_owned),
            static_namespaces: document.in_scope_namespaces(element).into(),
            computed_attributes: attribute_set_values,
            body: compile_sequence_excluding(document, element, &computed_attribute_nodes)?,
            location: document.location(element).clone(),
        });
    }
    if uses_xslt10_compatibility(document, element)
        && let Some(name) = compile_dynamic_element_name(document, element, name)
    {
        let (mut computed_attributes, computed_attribute_nodes) =
            compile_computed_attributes(document, element)?;
        let mut attribute_set_values = compile_local_attribute_sets(document, element, None)?;
        attribute_set_values.retain(|set_attribute| {
            !computed_attributes.iter().any(|attribute| {
                attribute.dynamic_name.is_none()
                    && set_attribute.dynamic_name.is_none()
                    && attribute.name == set_attribute.name
            })
        });
        attribute_set_values.append(&mut computed_attributes);
        return Ok(Instruction::DynamicNameElement {
            name,
            namespace_override: namespace.map(str::to_owned),
            static_namespaces: document.in_scope_namespaces(element).into(),
            computed_attributes: attribute_set_values,
            body: compile_sequence_excluding(document, element, &computed_attribute_nodes)?,
            location: document.location(element).clone(),
        });
    }
    let (name, mut namespaces) =
        compile_static_computed_element_name(document, element, name, namespace)?;
    let (mut computed_attributes, computed_attribute_nodes) =
        compile_computed_attributes(document, element)?;
    let mut attribute_set_values = compile_local_attribute_sets(document, element, None)?;
    attribute_set_values.retain(|set_attribute| {
        !computed_attributes.iter().any(|attribute| {
            attribute.dynamic_name.is_none()
                && set_attribute.dynamic_name.is_none()
                && attribute.name == set_attribute.name
        })
    });
    attribute_set_values.append(&mut computed_attributes);
    let computed_attributes = attribute_set_values;
    retain_computed_attribute_namespace_bindings(&mut namespaces, &computed_attributes);
    Ok(Instruction::LiteralElement {
        origin: ElementConstructorOrigin::ComputedStatic,
        name,
        namespaces: namespaces.into(),
        attributes: Vec::new(),
        computed_attributes,
        body: compile_sequence_excluding(document, element, &computed_attribute_nodes)?,
        location: document.location(element).clone(),
    })
}

fn compile_dynamic_element_name(
    document: &Document,
    element: NodeId,
    lexical: &str,
) -> Option<DynamicElementName> {
    if let Some((prefix, suffix)) = lexical.split_once("{position()}")
        && !prefix.contains(['{', '}'])
        && !suffix.contains(['{', '}'])
    {
        return Some(DynamicElementName::FocusPosition {
            prefix: prefix.to_owned(),
            suffix: suffix.to_owned(),
        });
    }
    let expression = lexical.strip_prefix('{')?.strip_suffix('}')?.trim();
    (!expression.contains(['{', '}']))
        .then(|| compile_sort_path(document, element, expression, document.location(element)).ok())
        .flatten()
        .map(DynamicElementName::Path)
}

fn retain_computed_attribute_namespace_bindings(
    namespaces: &mut Vec<NamespaceBinding>,
    attributes: &[ComputedAttribute],
) {
    const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
    let mut generated_index = 0_usize;
    for namespace in attributes.iter().filter_map(|attribute| {
        attribute.name.namespace.as_deref().or_else(|| {
            attribute.dynamic_name.as_ref().and_then(|name| match name {
                crate::xslt::golden_semantics_experiment::DynamicAttributeName::Path {
                    namespace_override,
                    ..
                }
                | crate::xslt::golden_semantics_experiment::DynamicAttributeName::ContextName {
                    namespace_override,
                    ..
                }
                | crate::xslt::golden_semantics_experiment::DynamicAttributeName::Literal {
                    namespace_override,
                    ..
                }
                | crate::xslt::golden_semantics_experiment::DynamicAttributeName::VariableAvt {
                    namespace_override,
                    ..
                } => namespace_override
                    .as_deref()
                    .filter(|value| !value.is_empty()),
            })
        })
    }) {
        if namespace == XML_NAMESPACE
            || namespaces
                .iter()
                .any(|binding| binding.prefix.is_some() && binding.namespace == namespace)
        {
            continue;
        }
        let prefix = loop {
            let candidate = format!("ns{generated_index}");
            generated_index += 1;
            if !namespaces
                .iter()
                .any(|binding| binding.prefix.as_deref() == Some(&candidate))
            {
                break candidate;
            }
        };
        namespaces.push(NamespaceBinding {
            prefix: Some(prefix),
            namespace: namespace.to_owned(),
        });
    }
}

fn compile_static_computed_element_name(
    document: &Document,
    element: NodeId,
    lexical: &str,
    namespace_override: Option<&str>,
) -> Result<(ExpandedName, Vec<NamespaceBinding>), CompileFailure> {
    const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";
    if namespace_override == Some(XMLNS_NAMESPACE) {
        return Err(invalid(
            "XTDE0835",
            "xsl:element cannot construct a name in the reserved xmlns namespace",
            document.location(element),
        ));
    }
    if lexical.contains(['{', '}']) {
        return Err(unsupported(
            "FXST1047",
            "the private xsl:element slice requires a static QName",
            document.location(element),
        ));
    }
    if is_ascii_ncname(lexical) {
        let namespace = match namespace_override {
            Some("") => None,
            Some(namespace) => Some(namespace),
            None => namespace_for_prefix(document, element, "").filter(|value| !value.is_empty()),
        };
        return Ok((
            ExpandedName {
                namespace: namespace.map(str::to_owned),
                local: lexical.to_owned(),
            },
            namespace
                .map(|namespace| {
                    vec![NamespaceBinding {
                        prefix: None,
                        namespace: namespace.to_owned(),
                    }]
                })
                .unwrap_or_default(),
        ));
    }
    let Some((prefix, local)) = lexical.split_once(':') else {
        return Err(invalid(
            "XTDE0820",
            format!("xsl:element name is not a lexical QName: {lexical}"),
            document.location(element),
        ));
    };
    if !is_ascii_ncname(prefix) || !is_ascii_ncname(local) || local.contains(':') {
        return Err(invalid(
            "XTDE0820",
            format!("xsl:element name is not a lexical QName: {lexical}"),
            document.location(element),
        ));
    }
    let namespace = match namespace_override {
        Some("") => {
            return Err(invalid(
                "XTDE0835",
                "a prefixed xsl:element name cannot use an empty namespace",
                document.location(element),
            ));
        }
        Some(namespace) => namespace,
        None => namespace_for_prefix(document, element, prefix).ok_or_else(|| {
            invalid(
                "XTDE0830",
                format!("xsl:element name uses an unbound prefix: {prefix}"),
                document.location(element),
            )
        })?,
    };
    Ok((
        ExpandedName {
            namespace: Some(namespace.to_owned()),
            local: local.to_owned(),
        },
        vec![NamespaceBinding {
            prefix: Some(prefix.to_owned()),
            namespace: namespace.to_owned(),
        }],
    ))
}

fn ensure_distinct_result_attributes(
    literal: &[crate::xslt::golden_semantics_experiment::LiteralAttribute],
    computed: &[crate::xslt::golden_semantics_experiment::ComputedAttribute],
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    for (index, attribute) in computed.iter().enumerate() {
        if attribute.dynamic_name.is_some() {
            continue;
        }
        if literal
            .iter()
            .any(|existing| existing.name == attribute.name)
            || computed[..index]
                .iter()
                .any(|existing| existing.dynamic_name.is_none() && existing.name == attribute.name)
        {
            return Err(invalid(
                "XTDE0410",
                format!("duplicate result attribute: {}", attribute.name.local),
                location,
            ));
        }
    }
    Ok(())
}

fn compile_apply_imports(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    template_invocation_compiler::compile_apply_imports(document, element)
}

fn compile_copy_of(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["select"], "xsl:copy-of")?;
    ensure_no_meaningful_children(document, element, "xsl:copy-of")?;
    let select = required_attribute(document, element, None, "select")?;
    if uses_xslt10_compatibility(document, element) && select.trim_start().starts_with("key(") {
        return value_expression_compiler::compile_xslt10_literal_key_lookup(
            document,
            element,
            select,
            document.location(element),
        )
        .map(|select| Instruction::CopyOfXslt10KeyLookup {
            select: Box::new(select),
            location: document.location(element).clone(),
        });
    }
    if let Some(value) = static_copy_of_text(select.trim()) {
        return Ok(Instruction::CopyOfStaticAtomicText {
            value,
            location: document.location(element).clone(),
        });
    }
    if let Some(variable) = select
        .trim()
        .strip_prefix('$')
        .filter(|name| is_ascii_ncname(name))
    {
        return Ok(Instruction::CopyOfVariable {
            variable: variable.to_owned(),
            location: document.location(element).clone(),
        });
    }
    if let Some(select) = value_expression_compiler::compile_xslt10_binary_numeric(
        document,
        element,
        select.trim(),
        document.location(element),
    ) {
        return Ok(Instruction::CopyOfAtomicValue {
            select: Box::new(select),
            location: document.location(element).clone(),
        });
    }
    if let Some(alternatives) = split_top_level_union(select) {
        let alternatives = alternatives
            .into_iter()
            .map(str::trim)
            .map(|alternative| {
                if alternative.is_empty() {
                    return Err(invalid(
                        "XPST0003",
                        "xsl:copy-of path union contains an empty alternative",
                        document.location(element),
                    ));
                }
                parse_copy_of_path(document, element, alternative)
            })
            .collect::<Result<Vec<_>, CompileFailure>>()?;
        return Ok(Instruction::CopyOfPathUnion {
            alternatives,
            location: document.location(element).clone(),
        });
    }
    match select.trim() {
        "." | "current()" => Ok(Instruction::CopyOfCurrent {
            location: document.location(element).clone(),
        }),
        "*" => Ok(Instruction::CopyOfChildElements {
            location: document.location(element).clone(),
        }),
        "ancestor-or-self::*" => Ok(Instruction::CopyOfAncestorOrSelfElements {
            location: document.location(element).clone(),
        }),
        expression => parse_copy_of_path(document, element, expression).map(|select| {
            Instruction::CopyOfLocationPath {
                select,
                location: document.location(element).clone(),
            }
        }),
    }
}

pub(super) fn split_top_level_union(expression: &str) -> Option<Vec<&str>> {
    let bytes = expression.as_bytes();
    let mut alternatives = Vec::new();
    let mut quote = None;
    let mut parentheses = 0_usize;
    let mut brackets = 0_usize;
    let mut start = 0_usize;

    for (index, byte) in bytes.iter().copied().enumerate() {
        if let Some(expected) = quote {
            if byte == expected {
                quote = None;
            }
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => parentheses += 1,
            b')' => parentheses = parentheses.saturating_sub(1),
            b'[' => brackets += 1,
            b']' => brackets = brackets.saturating_sub(1),
            b'|' if parentheses == 0 && brackets == 0 => {
                alternatives.push(&expression[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    if alternatives.is_empty() {
        None
    } else {
        alternatives.push(&expression[start..]);
        Some(alternatives)
    }
}

pub(super) fn parse_xslt10_normalize_space_path(
    expression: &str,
    location: crate::xdm::owned_tree_experiment::SourceLocation,
) -> Option<LocationPath> {
    let path = expression
        .trim()
        .strip_prefix("normalize-space(")?
        .strip_suffix(')')?
        .trim();
    (!path.is_empty())
        .then(|| parse_location_path(path, location).ok())
        .flatten()
}

fn parse_copy_of_path(
    document: &Document,
    element: NodeId,
    expression: &str,
) -> Result<LocationPath, CompileFailure> {
    let location = document.location(element).clone();
    let path = match parse_location_path(expression, location.clone()) {
        Ok(path) => Ok(path),
        Err(PathFailure::Unsupported { .. }) if expression.contains(':') => {
            parse_qualified_child_path(expression, location, |prefix| {
                namespace_for_prefix(document, element, prefix).map(str::to_owned)
            })
        }
        Err(failure) => Err(failure),
    };
    path.map_err(|failure| match failure {
        PathFailure::Invalid {
            standard_code,
            detail,
            location,
        } => invalid(standard_code, detail, &location),
        PathFailure::Unsupported {
            detail, location, ..
        } => unsupported(
            "FXXP1003",
            format!("unsupported xsl:copy-of selection: {detail}"),
            &location,
        ),
    })
}

fn static_copy_of_text(expression: &str) -> Option<String> {
    for delimiter in ['\'', '"'] {
        let literal = expression
            .strip_prefix(delimiter)
            .and_then(|value| value.strip_suffix(delimiter));
        if let Some(literal) = literal.filter(|value| !value.contains(delimiter)) {
            return Some(literal.to_owned());
        }
    }
    match expression {
        "true()" => Some("true".to_owned()),
        "false()" => Some("false".to_owned()),
        _ => expression
            .parse::<i64>()
            .ok()
            .map(|value| value.to_string()),
    }
}

fn compile_for_each(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    let select = required_attribute(document, element, None, "select")?;
    let (sorts, sort_nodes) = compile_sort_keys(document, element)?;
    if let Some(variable) = select
        .trim()
        .strip_prefix('$')
        .filter(|name| is_ascii_ncname(name))
    {
        ensure_only_attributes(
            document,
            element,
            &["select", "default-mode"],
            "xsl:for-each",
        )?;
        return Ok(Instruction::ForEachVariable {
            variable: variable.to_owned(),
            sorts,
            body: compile_sequence_excluding(document, element, &sort_nodes)?,
            location: document.location(element).clone(),
        });
    }
    if let Some((start, end)) = parse_static_integer_range(select) {
        if !sorts.is_empty() {
            return Err(unsupported(
                "FXST1044",
                "xsl:sort over atomic integer ranges is outside the admitted sorting slice",
                document.location(element),
            ));
        }
        for child in meaningful_children(document, element) {
            if is_xslt_element(document, child, "apply-templates") {
                let apply_select = optional_attribute(document, child, None, "select");
                if apply_select.is_none_or(|value| value.trim() == ".") {
                    return Err(invalid(
                        "XTTE0510",
                        "xsl:apply-templates requires nodes, but the statically known focus is an integer",
                        document.location(child),
                    ));
                }
            }
        }
        ensure_only_attributes(
            document,
            element,
            &["select", "default-mode"],
            "xsl:for-each",
        )?;
        let body = compile_sequence_excluding(document, element, &sort_nodes)?;
        if !is_context_independent_static_range_body(&body) {
            return Err(unsupported(
                "FXST1007",
                "the admitted static integer-range xsl:for-each body is limited to literal result elements and text",
                document.location(element),
            ));
        }
        return Ok(Instruction::ForEachStaticIntegerRange {
            start,
            end,
            body,
            location: document.location(element).clone(),
        });
    }
    ensure_only_attributes(
        document,
        element,
        &["select", "default-mode"],
        "xsl:for-each",
    )?;
    let location = document.location(element).clone();
    Ok(Instruction::ForEachNodes {
        select: template_invocation_compiler::parse_apply_selection(
            document,
            element,
            select,
            location.clone(),
        )?,
        sorts,
        body: compile_sequence_excluding(document, element, &sort_nodes)?,
        location,
    })
}

pub(super) fn compile_sort_keys(
    document: &Document,
    parent: NodeId,
) -> Result<(Vec<SortKey>, Vec<NodeId>), CompileFailure> {
    let xslt10_numeric_conversion = uses_xslt10_compatibility(document, parent);
    let children = meaningful_children(document, parent);
    let mut sorts = Vec::new();
    let mut sort_nodes = Vec::new();
    let mut saw_body = false;
    for child in children {
        if !is_xslt_element(document, child, "sort") {
            saw_body = true;
            continue;
        }
        if saw_body {
            return Err(invalid(
                "XTSE0010",
                "xsl:sort must precede every other sequence-constructor child",
                document.location(child),
            ));
        }
        ensure_only_attributes(
            document,
            child,
            &["select", "data-type", "order", "lang", "case-order"],
            "xsl:sort",
        )?;
        ensure_no_meaningful_children(document, child, "xsl:sort")?;
        let location = document.location(child).clone();
        let select = compile_sort_select(
            document,
            child,
            optional_attribute(document, child, None, "select").unwrap_or("."),
            xslt10_numeric_conversion,
            &location,
        )?;
        let data_type = compile_sort_data_type(
            optional_attribute(document, child, None, "data-type"),
            &location,
        )?;
        validate_sort_collation_metadata(document, child, &data_type, &location)?;
        let order = compile_sort_order(
            optional_attribute(document, child, None, "order"),
            &location,
        )?;
        sorts.push(SortKey {
            select,
            data_type,
            xslt10_numeric_conversion,
            order,
            location,
        });
        sort_nodes.push(child);
    }
    Ok((sorts, sort_nodes))
}

fn compile_sort_select(
    document: &Document,
    sort: NodeId,
    select: &str,
    xslt10_compatibility: bool,
    location: &SourceLocation,
) -> Result<SortSelect, CompileFailure> {
    if xslt10_compatibility && select.trim_start().starts_with("key(") {
        return Ok(SortSelect::Xslt10KeyLookup(Box::new(
            value_expression_compiler::compile_xslt10_literal_key_lookup(
                document, sort, select, location,
            )?,
        )));
    }
    if let Some(alternatives) = split_top_level_union(select) {
        let alternatives = alternatives
            .into_iter()
            .map(str::trim)
            .map(|alternative| {
                if alternative.is_empty() {
                    return Err(invalid(
                        "XPST0003",
                        "xsl:sort path union contains an empty alternative",
                        location,
                    ));
                }
                compile_sort_path(document, sort, alternative, location)
            })
            .collect::<Result<Vec<_>, CompileFailure>>()?;
        return Ok(SortSelect::PathUnion(alternatives));
    }
    if let Some(value) = xpath_string_literal(select.trim()) {
        return Ok(SortSelect::Literal(value.to_owned()));
    }
    if let Some(variable) = select.trim().strip_prefix('$')
        && is_ascii_ncname(variable)
    {
        return Ok(SortSelect::Variable(variable.to_owned()));
    }
    if xslt10_compatibility
        && let Some((path, variable, explicit_position_comparison)) =
            value_expression_compiler::compile_xslt10_variable_position_path(
                document, sort, select, location,
            )?
    {
        return Ok(SortSelect::Xslt10VariablePositionPath {
            path,
            variable,
            explicit_position_comparison,
        });
    }
    match select.trim() {
        "position()" => Ok(SortSelect::ContextPosition),
        "last()" => Ok(SortSelect::ContextSize),
        "name()" | "name(.)" => Ok(SortSelect::ContextNodeName),
        "string-length()" | "string-length(.)" => Ok(SortSelect::ContextStringLength),
        _ => {
            if let Some(path) =
                compile_sort_function_path(document, sort, select, "count", location)?
            {
                return Ok(SortSelect::CountPath(path));
            }
            if let Some(path) =
                compile_sort_function_path(document, sort, select, "number", location)?
            {
                return Ok(SortSelect::NumberPath(path));
            }
            Ok(SortSelect::LocationPath(compile_sort_path(
                document, sort, select, location,
            )?))
        }
    }
}

fn compile_sort_data_type(
    value: Option<&str>,
    location: &SourceLocation,
) -> Result<SortDataType, CompileFailure> {
    match value.map(fold_static_sort_control_avt) {
        None | Some(Some("text")) => Ok(SortDataType::Text),
        Some(Some("number")) => Ok(SortDataType::Number),
        Some(Some(value)) => Err(unsupported(
            "FXST1044",
            format!("unsupported xsl:sort data-type: {value}"),
            location,
        )),
        Some(None) => compile_sort_control_variable(value.expect("dynamic sort control exists"))
            .map(SortDataType::Variable)
            .ok_or_else(|| {
                unsupported(
                    "FXST1044",
                    "dynamic xsl:sort data-type is outside the admitted variable-only slice",
                    location,
                )
            }),
    }
}

fn compile_sort_order(
    value: Option<&str>,
    location: &SourceLocation,
) -> Result<SortOrder, CompileFailure> {
    match value.map(fold_static_sort_control_avt) {
        None | Some(Some("ascending")) => Ok(SortOrder::Ascending),
        Some(Some("descending")) => Ok(SortOrder::Descending),
        Some(Some(value)) => Err(invalid(
            "XTDE0030",
            format!("invalid xsl:sort order: {value}"),
            location,
        )),
        Some(None) => compile_sort_control_variable(value.expect("dynamic sort control exists"))
            .map(SortOrder::Variable)
            .ok_or_else(|| {
                unsupported(
                    "FXST1044",
                    "dynamic xsl:sort order is outside the admitted variable-only slice",
                    location,
                )
            }),
    }
}

fn fold_static_sort_control_avt(value: &str) -> Option<&str> {
    if !value.contains(['{', '}']) {
        return Some(value);
    }
    let expression = value.strip_prefix('{')?.strip_suffix('}')?.trim();
    xpath_string_literal(expression)
}

fn compile_sort_control_variable(value: &str) -> Option<String> {
    let expression = value.strip_prefix('{')?.strip_suffix('}')?.trim();
    let variable = expression.strip_prefix('$')?;
    is_ascii_ncname(variable).then(|| variable.to_owned())
}

fn validate_sort_collation_metadata(
    document: &Document,
    sort: NodeId,
    data_type: &SortDataType,
    location: &SourceLocation,
) -> Result<(), CompileFailure> {
    let lang = optional_attribute(document, sort, None, "lang");
    let case_order = optional_attribute(document, sort, None, "case-order");
    if matches!(data_type, SortDataType::Text | SortDataType::Variable(_))
        && (lang.is_some() || case_order.is_some())
    {
        return Err(unsupported(
            "FXST1063",
            "language-sensitive text collation is outside the admitted xsl:sort slice",
            location,
        ));
    }
    if matches!(data_type, SortDataType::Number)
        && lang
            .into_iter()
            .chain(case_order)
            .any(|value| value.contains(['{', '}']))
    {
        return Err(unsupported(
            "FXST1064",
            "dynamic ignored xsl:sort collation attributes are outside the admitted numeric slice",
            location,
        ));
    }
    Ok(())
}

fn compile_sort_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<LocationPath, CompileFailure> {
    match parse_location_path(expression, location.clone()) {
        Ok(path) => Ok(path),
        Err(PathFailure::Unsupported { .. }) if expression.contains(':') => {
            parse_qualified_child_path(expression, location.clone(), |prefix| {
                namespace_for_prefix(document, element, prefix).map(str::to_owned)
            })
            .map_err(map_path_failure)
        }
        Err(failure) => Err(map_path_failure(failure)),
    }
}

fn compile_sort_function_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    function: &str,
    location: &SourceLocation,
) -> Result<Option<LocationPath>, CompileFailure> {
    let expression = expression.trim();
    let Some(argument) = expression
        .strip_prefix(function)
        .and_then(|value| value.strip_prefix('('))
        .and_then(|value| value.strip_suffix(')'))
        .map(str::trim)
    else {
        return Ok(None);
    };
    if argument.is_empty() {
        return Ok(None);
    }
    let mut path = parse_location_path(argument, location.clone()).map_err(map_path_failure)?;
    if let Some(namespace) = effective_xpath_default_namespace(document, element) {
        for step in &mut path.steps {
            if let PathStep::ChildNamed(local) = step {
                *step = PathStep::ChildExpandedName(ExpandedName {
                    namespace: Some(namespace.to_owned()),
                    local: local.clone(),
                });
            }
        }
    }
    Ok(Some(path))
}

pub(super) fn uses_xslt10_compatibility(document: &Document, element: NodeId) -> bool {
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(version) = optional_attribute(document, node, Some(XSLT_NAMESPACE), "version") {
            return version == "1.0";
        }
        if document.name(node).is_some_and(|name| {
            name.namespace.as_deref() == Some(XSLT_NAMESPACE)
                && matches!(name.local.as_str(), "stylesheet" | "transform")
        }) {
            return optional_attribute(document, node, None, "version") == Some("1.0");
        }
        current = document.parent(node);
    }
    false
}

fn parse_static_integer_range(expression: &str) -> Option<(i64, i64)> {
    let (start, end) = expression.split_once(" to ")?;
    Some((
        start.trim().parse::<i64>().ok()?,
        end.trim().parse::<i64>().ok()?,
    ))
}

fn is_context_independent_static_range_body(body: &[Instruction]) -> bool {
    body.iter().all(|instruction| match instruction {
        Instruction::Text { .. } => true,
        Instruction::LiteralElement { body, .. } => is_context_independent_static_range_body(body),
        _ => false,
    })
}

fn ensure_literal_result_control_attributes(
    document: &Document,
    element: NodeId,
) -> Result<(), CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document
            .name(*attribute)
            .expect("attribute nodes have expanded names");
        if name.namespace.as_deref() == Some(XSLT_NAMESPACE)
            && !matches!(
                name.local.as_str(),
                "version"
                    | "xpath-default-namespace"
                    | "default-mode"
                    | "use-attribute-sets"
                    | "exclude-result-prefixes"
                    | "extension-element-prefixes"
            )
        {
            if uses_xslt10_compatibility(document, element) {
                continue;
            }
            return Err(unsupported(
                "FXST1007",
                "unsupported XSLT control attribute on a literal result element",
                document.location(*attribute),
            ));
        }
    }
    Ok(())
}

pub(super) fn compile_text(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    let value = compile_text_value(document, element)?;
    Ok(Instruction::Text {
        value,
        location: document.location(element).clone(),
    })
}

pub(super) fn compile_text_value(
    document: &Document,
    element: NodeId,
) -> Result<String, CompileFailure> {
    compile_text_value_with_ignored_escaping(document, element, false)
}

fn compile_text_value_with_ignored_escaping(
    document: &Document,
    element: NodeId,
    ignore_disable_output_escaping: bool,
) -> Result<String, CompileFailure> {
    ensure_text_attributes(document, element)?;
    match optional_attribute(document, element, None, "disable-output-escaping") {
        Some("yes") if !ignore_disable_output_escaping => {
            return Err(unsupported(
                "FXST1060",
                "disable-output-escaping='yes' is outside the semantic result-tree slice",
                document.location(element),
            ));
        }
        None | Some("no" | "yes") => {}
        Some(_) => {
            return Err(invalid(
                "XTSE0020",
                "disable-output-escaping must be 'yes' or 'no'",
                document.location(element),
            ));
        }
    }
    let mut value = String::new();
    for child in document.children(element).iter().copied() {
        match document.kind(child) {
            NodeKind::Text => value.push_str(document.value(child).unwrap_or_default()),
            NodeKind::Comment | NodeKind::ProcessingInstruction => {}
            NodeKind::Element | NodeKind::Document | NodeKind::Attribute => {
                return Err(invalid(
                    "FXST0026",
                    "the private xsl:text slice permits character content only",
                    document.location(child),
                ));
            }
        }
    }
    Ok(value)
}

fn ensure_text_attributes(document: &Document, element: NodeId) -> Result<(), CompileFailure> {
    const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
    for attribute in document.attributes(element) {
        let name = document
            .name(*attribute)
            .expect("attribute nodes have expanded names");
        if name.namespace.is_none() && name.local == "disable-output-escaping" {
            continue;
        }
        if name.namespace.as_deref() == Some(XML_NAMESPACE) && name.local == "space" {
            match document.value(*attribute).unwrap_or_default() {
                "default" | "preserve" => continue,
                _ => {
                    return Err(invalid(
                        "XTSE0020",
                        "xml:space must be 'default' or 'preserve'",
                        document.location(*attribute),
                    ));
                }
            }
        }
        return Err(unsupported(
            "FXST1009",
            format!(
                "unsupported attribute on xsl:text: {{{}}}{}",
                name.namespace.as_deref().unwrap_or(""),
                name.local
            ),
            document.location(*attribute),
        ));
    }
    Ok(())
}

pub(super) fn compile_processing_instruction(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["name"], "xsl:processing-instruction")?;
    let target = required_attribute(document, element, None, "name")?;
    if !is_ascii_ncname(target) || target.eq_ignore_ascii_case("xml") {
        return Err(invalid(
            "FXST0036",
            "the static processing-instruction target must be an NCName other than XML",
            document.location(element),
        ));
    }
    let children = meaningful_children(document, element);
    let mut value = String::new();
    let mut requires_dynamic_content = false;
    for child in children.iter().copied() {
        match document.kind(child) {
            NodeKind::Text => value.push_str(document.value(child).unwrap_or_default()),
            NodeKind::Comment | NodeKind::ProcessingInstruction => {}
            NodeKind::Element => {
                if is_xslt_element(document, child, "text") {
                    value.push_str(&compile_text_value_with_ignored_escaping(
                        document, child, true,
                    )?);
                } else if is_xslt_element(document, child, "value-of")
                    && let Some(static_value) = compile_static_node_content_value(document, child)?
                {
                    value.push_str(&static_value);
                } else if uses_xslt10_compatibility(document, element) {
                    requires_dynamic_content = true;
                } else {
                    return Err(unsupported(
                        "FXST1034",
                        "computed processing-instruction content is outside the private slice",
                        document.location(child),
                    ));
                }
            }
            NodeKind::Document | NodeKind::Attribute => {
                return Err(invalid(
                    "FXST0006",
                    "unexpected node kind in xsl:processing-instruction",
                    document.location(child),
                ));
            }
        }
    }
    if requires_dynamic_content {
        const MAX_SEQUENCE_CONSTRUCTOR_CHILDREN: usize = 64;
        if children.len() > MAX_SEQUENCE_CONSTRUCTOR_CHILDREN {
            return Err(unsupported(
                "FXST1034",
                format!(
                    "the private XSLT 1.0 processing-instruction sequence constructor is limited to {MAX_SEQUENCE_CONSTRUCTOR_CHILDREN} children"
                ),
                document.location(element),
            ));
        }
        let excluded = xslt10_text_constructor_exclusions(document, &children);
        return Ok(Instruction::Xslt10ProcessingInstructionNode {
            target: target.to_owned(),
            body: compile_sequence_excluding(document, element, &excluded)?.into_boxed_slice(),
            location: document.location(element).clone(),
        });
    }
    if value.contains("?>") {
        if uses_xslt10_compatibility(document, element) {
            value = recover_xslt10_processing_instruction_content(&value);
        } else {
            return Err(unsupported(
                "FXST1035",
                "processing-instruction data containing ?> requires recovery outside the private slice",
                document.location(element),
            ));
        }
    }
    Ok(Instruction::ProcessingInstructionNode {
        target: target.to_owned(),
        value,
        location: document.location(element).clone(),
    })
}

fn recover_xslt10_processing_instruction_content(value: &str) -> String {
    value.replace("?>", "? >")
}

pub(super) fn compile_comment(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["select"], "xsl:comment")?;
    let mut value = if let Some(select) = optional_attribute(document, element, None, "select") {
        ensure_no_meaningful_children(document, element, "xsl:comment")?;
        crate::xpath::static_string_experiment::fold(select).ok_or_else(|| {
            unsupported(
                "FXXP1013",
                format!("unsupported static comment expression: {select}"),
                document.location(element),
            )
        })?
    } else {
        let mut value = String::new();
        for child in meaningful_children(document, element) {
            match document.kind(child) {
                NodeKind::Text => value.push_str(document.value(child).unwrap_or_default()),
                NodeKind::Comment | NodeKind::ProcessingInstruction => {}
                NodeKind::Element => {
                    if is_xslt_element(document, child, "text") {
                        value.push_str(&compile_text_value_with_ignored_escaping(
                            document, child, true,
                        )?);
                    } else if is_xslt_element(document, child, "value-of")
                        && let Some(static_value) =
                            compile_static_node_content_value(document, child)?
                    {
                        value.push_str(&static_value);
                    } else {
                        return Err(unsupported(
                            "FXST1036",
                            "computed comment content is outside the private slice",
                            document.location(child),
                        ));
                    }
                }
                NodeKind::Document | NodeKind::Attribute => {
                    return Err(invalid(
                        "FXST0006",
                        "unexpected node kind in xsl:comment",
                        document.location(child),
                    ));
                }
            }
        }
        value
    };
    if value.contains("--") || value.ends_with('-') {
        if uses_xslt10_compatibility(document, element) {
            value = recover_xslt10_comment_content(&value);
        } else {
            return Err(unsupported(
                "FXST1037",
                "comment content requiring lexical recovery is outside the private slice",
                document.location(element),
            ));
        }
    }
    Ok(Instruction::CommentNode {
        value,
        location: document.location(element).clone(),
    })
}

fn recover_xslt10_comment_content(value: &str) -> String {
    let characters = value.chars().collect::<Vec<_>>();
    let mut recovered = String::with_capacity(value.len() + 1);
    for (index, character) in characters.iter().copied().enumerate() {
        recovered.push(character);
        if character == '-'
            && (characters.get(index + 1) == Some(&'-') || index + 1 == characters.len())
        {
            recovered.push(' ');
        }
    }
    recovered
}

fn compile_static_node_content_value(
    document: &Document,
    element: NodeId,
) -> Result<Option<String>, CompileFailure> {
    for attribute in document.attributes(element) {
        let name = document
            .name(*attribute)
            .expect("attribute nodes have expanded names");
        if name.namespace.is_none()
            && matches!(name.local.as_str(), "select" | "disable-output-escaping")
        {
            continue;
        }
        return Err(unsupported(
            "FXST1009",
            format!(
                "unsupported attribute on xsl:value-of: {{{}}}{}",
                name.namespace.as_deref().unwrap_or(""),
                name.local
            ),
            document.location(*attribute),
        ));
    }
    match optional_attribute(document, element, None, "disable-output-escaping") {
        None | Some("no" | "yes") => {}
        Some(_) => {
            return Err(invalid(
                "XTSE0020",
                "disable-output-escaping must be 'yes' or 'no'",
                document.location(element),
            ));
        }
    }
    ensure_no_meaningful_children(document, element, "xsl:value-of")?;
    let select = required_attribute(document, element, None, "select")?;
    Ok(crate::xpath::static_string_experiment::fold(select)
        .or_else(|| crate::xpath::static_string_experiment::fold_concat_literals(select))
        .or_else(|| crate::xpath::static_string_experiment::fold_substring_literals(select)))
}

pub(super) fn validate_exclude_result_prefixes(
    document: &Document,
    element: NodeId,
) -> Result<(), CompileFailure> {
    let Some(exclusions) = optional_attribute(document, element, None, "exclude-result-prefixes")
        .or_else(|| {
            optional_attribute(
                document,
                element,
                Some(XSLT_NAMESPACE),
                "exclude-result-prefixes",
            )
        })
    else {
        return Ok(());
    };
    for prefix in exclusions.split_whitespace() {
        if prefix == "#all" {
            continue;
        }
        let namespace_prefix = if prefix == "#default" { "" } else { prefix };
        if (prefix != "#default" && !is_ascii_ncname(prefix))
            || namespace_for_prefix(document, element, namespace_prefix).is_none()
        {
            return Err(invalid(
                "XTSE0808",
                format!("invalid or unbound excluded result prefix: {prefix}"),
                document.location(element),
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_extension_element_prefixes(
    document: &Document,
    element: NodeId,
) -> Result<(), CompileFailure> {
    let Some(prefixes) = optional_attribute(document, element, None, "extension-element-prefixes")
        .or_else(|| {
            optional_attribute(
                document,
                element,
                Some(XSLT_NAMESPACE),
                "extension-element-prefixes",
            )
        })
    else {
        return Ok(());
    };
    for prefix in prefixes.split_whitespace() {
        if prefix == "#default" {
            if namespace_for_prefix(document, element, "").is_none_or(str::is_empty) {
                return Err(invalid(
                    "XTSE1430",
                    "extension-element-prefixes names #default without a bound default namespace",
                    document.location(element),
                ));
            }
            continue;
        }
        if !is_ascii_ncname(prefix) || namespace_for_prefix(document, element, prefix).is_none() {
            return Err(invalid(
                "XTSE1430",
                format!("invalid or unbound extension-element prefix: {prefix}"),
                document.location(element),
            ));
        }
    }
    Ok(())
}

pub(super) fn literal_result_namespaces(
    document: &Document,
    element: NodeId,
) -> Vec<NamespaceBinding> {
    let mut namespaces = Vec::new();
    let mut excluded_prefixes = Vec::new();
    let mut exclude_all = false;
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(exclusions) =
            optional_attribute(document, node, None, "exclude-result-prefixes").or_else(|| {
                optional_attribute(
                    document,
                    node,
                    Some(XSLT_NAMESPACE),
                    "exclude-result-prefixes",
                )
            })
        {
            for prefix in exclusions.split_whitespace() {
                if prefix == "#all" {
                    exclude_all = true;
                } else if !excluded_prefixes.contains(&prefix) {
                    excluded_prefixes.push(prefix);
                }
            }
        }
        if let Some(extensions) =
            optional_attribute(document, node, None, "extension-element-prefixes").or_else(|| {
                optional_attribute(
                    document,
                    node,
                    Some(XSLT_NAMESPACE),
                    "extension-element-prefixes",
                )
            })
        {
            for prefix in extensions.split_whitespace() {
                if !excluded_prefixes.contains(&prefix) {
                    excluded_prefixes.push(prefix);
                }
            }
        }
        current = document.parent(node);
    }
    let mut current = Some(element);
    while let Some(node) = current {
        for binding in document.namespace_declarations(node) {
            let prefix = binding.prefix.as_deref();
            let required_for_element = document.name(element).is_some_and(|name| {
                name.namespace.as_deref() == Some(binding.namespace.as_str())
                    && document.prefix(element) == prefix
            });
            let required_for_attribute = document.attributes(element).iter().any(|attribute| {
                document.name(*attribute).is_some_and(|name| {
                    name.namespace.as_deref() == Some(binding.namespace.as_str())
                        && name.namespace.as_deref() != Some(XSLT_NAMESPACE)
                        && document.prefix(*attribute) == prefix
                })
            });
            let excluded = match prefix {
                Some(prefix) => excluded_prefixes.contains(&prefix),
                None => excluded_prefixes.contains(&"#default"),
            };
            if prefix != Some("xml")
                && binding.namespace != XSLT_NAMESPACE
                && !binding.namespace.is_empty()
                && (required_for_element || required_for_attribute || (!exclude_all && !excluded))
                && !namespaces
                    .iter()
                    .any(|existing: &NamespaceBinding| existing.prefix.as_deref() == prefix)
            {
                namespaces.push(binding.clone());
            }
        }
        current = document.parent(node);
    }
    if let Some(preferred_prefix) = document.prefix(element)
        && let Some(index) = namespaces
            .iter()
            .position(|binding| binding.prefix.as_deref() == Some(preferred_prefix))
    {
        namespaces[..=index].rotate_right(1);
    }
    namespaces
}

fn compile_apply_templates(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    template_invocation_compiler::compile_apply_templates(document, element)
}

fn compile_with_params(
    document: &Document,
    parent: NodeId,
    parent_label: &str,
    allow_fallback: bool,
) -> Result<Vec<TemplateArgument>, CompileFailure> {
    template_invocation_compiler::compile_with_params(
        document,
        parent,
        parent_label,
        allow_fallback,
    )
}

pub(super) fn parse_template_modes(
    document: &Document,
    element: NodeId,
    mode: &str,
) -> Result<Vec<String>, CompileFailure> {
    template_invocation_compiler::parse_template_modes(document, element, mode)
}

fn compile_value_of(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &[
            "select",
            "separator",
            "xpath-default-namespace",
            "disable-output-escaping",
        ],
        "xsl:value-of",
    )?;
    match optional_attribute(document, element, None, "disable-output-escaping") {
        None | Some("no") => {}
        Some("yes") => {
            return Err(unsupported(
                "FXST1060",
                "disable-output-escaping='yes' is outside the semantic result-tree slice",
                document.location(element),
            ));
        }
        Some(_) => {
            return Err(invalid(
                "XTSE0020",
                "disable-output-escaping must be 'yes' or 'no'",
                document.location(element),
            ));
        }
    }
    ensure_no_meaningful_children(document, element, "xsl:value-of")?;
    let location = document.location(element).clone();
    let expression = required_attribute(document, element, None, "select")?;
    let select = compile_value_expression(document, element, expression, &location)?;
    let separator = optional_attribute(document, element, None, "separator")
        .unwrap_or(" ")
        .to_owned();
    Ok(Instruction::ValueOf {
        select,
        separator,
        location,
    })
}

fn compile_variable(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["name", "select", "as"], "xsl:variable")?;
    let location = document.location(element).clone();
    let lexical_name = required_attribute(document, element, None, "name")?;
    let name = normalize_variable_qname(document, element, lexical_name).map_err(|_| {
        invalid(
            "FXST0016",
            format!("invalid local variable name: {lexical_name}"),
            &location,
        )
    })?;
    let Some(expression) = optional_attribute(document, element, None, "select") else {
        return compile_content_variable(document, element, &name, location);
    };
    ensure_no_meaningful_children(document, element, "xsl:variable")?;
    if optional_attribute(document, element, None, "as").is_some() {
        return Err(unsupported(
            "FXST1016",
            "typed select-based local variables are outside the private slice",
            &location,
        ));
    }
    if let Some(offset) = parse_context_position_offset(expression) {
        return Ok(Instruction::ContextPositionVariable {
            name: name.clone(),
            offset,
            location,
        });
    }
    if matches!(expression.trim(), "name()" | "name(.)") {
        return Ok(Instruction::ContextNodeNameVariable {
            name: name.clone(),
            location,
        });
    }
    if let Some(path) = expression
        .trim()
        .strip_prefix("count(")
        .and_then(|path| path.strip_suffix(')'))
    {
        if let Ok(select) = parse_location_path(path.trim(), location.clone()) {
            return Ok(Instruction::ContextCountPathVariable {
                name: name.clone(),
                select,
                location,
            });
        }
    }
    if let Some(variable) = compile_source_node_union_variable(&name, expression, &location)? {
        return Ok(variable);
    }
    if let Some(source) = expression
        .trim()
        .strip_prefix('$')
        .filter(|source| is_ascii_ncname(source))
    {
        return Ok(Instruction::AtomicVariableAlias {
            name: name.clone(),
            source: source.to_owned(),
            location,
        });
    }
    if let Some(variable) = compile_static_atomic_variable(&name, expression, &location) {
        return Ok(variable);
    }
    if let Some(variable) =
        compile_local_binary_numeric_variable(document, element, &name, expression, &location)
    {
        return Ok(variable);
    }
    if let Some(variable) =
        compile_local_variable_path_variable(document, element, &name, expression, &location)?
    {
        return Ok(variable);
    }
    compile_local_node_or_cast_variable(document, element, &name, expression, location)
}

fn parse_context_position_offset(expression: &str) -> Option<usize> {
    let expression = expression.trim();
    if expression == "position()" {
        return Some(0);
    }
    let (position, offset) = expression.split_once('+')?;
    (position.trim() == "position()")
        .then(|| offset.trim().parse().ok())
        .flatten()
}

fn compile_local_node_or_cast_variable(
    document: &Document,
    element: NodeId,
    name: &str,
    expression: &str,
    location: SourceLocation,
) -> Result<Instruction, CompileFailure> {
    let node_path = match parse_location_path(expression, location.clone()) {
        Ok(path) => Some(path),
        Err(PathFailure::Unsupported { .. }) if expression.contains(':') => Some(
            parse_qualified_child_path(expression, location.clone(), |prefix| {
                namespace_for_prefix(document, element, prefix).map(str::to_owned)
            })
            .map_err(map_path_failure)?,
        ),
        Err(_) => None,
    };
    if let Some(select) = node_path {
        return Ok(Instruction::SourceNodeVariable {
            name: name.to_owned(),
            select,
            location,
        });
    }
    let select = parse_cast(expression, &location).map_err(|failure| CompileFailure {
        code: "FXXP1008",
        category: CompileCategory::Unsupported,
        detail: failure.detail,
        location: failure.location,
    })?;
    Ok(Instruction::Variable {
        name: name.to_owned(),
        select: Box::new(select),
        location,
    })
}

fn compile_local_variable_path_variable(
    document: &Document,
    element: NodeId,
    name: &str,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<Instruction>, CompileFailure> {
    let Some((source, select)) = value_expression_compiler::compile_xslt10_variable_path(
        document, element, expression, location,
    )?
    else {
        return Ok(None);
    };
    Ok(Some(Instruction::SourceVariablePathVariable {
        name: name.to_owned(),
        source,
        select,
        location: location.clone(),
    }))
}

fn compile_local_binary_numeric_variable(
    document: &Document,
    element: NodeId,
    name: &str,
    expression: &str,
    location: &SourceLocation,
) -> Option<Instruction> {
    let select = value_expression_compiler::compile_xslt10_binary_numeric(
        document,
        element,
        expression.trim(),
        location,
    )?;
    Some(Instruction::Xslt10BinaryNumericVariable {
        name: name.to_owned(),
        select: Box::new(select),
        location: location.clone(),
    })
}

fn compile_content_variable(
    document: &Document,
    element: NodeId,
    name: &str,
    location: SourceLocation,
) -> Result<Instruction, CompileFailure> {
    if optional_attribute(document, element, None, "as").is_none()
        && document.children(element).is_empty()
    {
        return Ok(Instruction::StaticAtomicVariable {
            name: name.to_owned(),
            value: AtomicValue::string(String::new()),
            location,
        });
    }
    if let Some(variable) = compile_xslt10_text_tree_variable(document, element, name, &location)? {
        return Ok(variable);
    }
    if let Some(variable) =
        compile_xslt10_value_of_tree_variable(document, element, name, &location)?
    {
        return Ok(variable);
    }
    if let Some(variable) =
        compile_xslt10_for_each_text_variable(document, element, name, &location)?
    {
        return Ok(variable);
    }
    if optional_attribute(document, element, None, "as").is_none()
        && meaningful_children(document, element)
            .iter()
            .all(|child| document.kind(*child) == NodeKind::Element)
    {
        match super::compile_constructed_elements(document, element) {
            Ok(elements) => {
                return Ok(Instruction::TemporaryTreeVariable {
                    name: name.to_owned(),
                    elements,
                    location,
                });
            }
            Err(failure)
                if failure.code == "FXST1015" && uses_xslt10_compatibility(document, element) =>
            {
                // The compact static constructor is an optimization. XSLT 1.0
                // content variables may contain the ordinary instruction
                // sequence, so fall through to the complete runtime path.
            }
            Err(failure) => return Err(failure),
        }
    }
    if optional_attribute(document, element, None, "as").is_none()
        && uses_xslt10_compatibility(document, element)
    {
        return Ok(Instruction::Xslt10SequenceTreeVariable {
            name: name.to_owned(),
            body: compile_sequence(document, element)?,
            location,
        });
    }
    compile_integer_range_variable(document, element, name, &location)
}

fn compile_xslt10_text_tree_variable(
    document: &Document,
    element: NodeId,
    name: &str,
    location: &SourceLocation,
) -> Result<Option<Instruction>, CompileFailure> {
    if optional_attribute(document, element, None, "as").is_some()
        || !uses_xslt10_compatibility(document, element)
    {
        return Ok(None);
    }
    Ok(
        compile_xslt10_static_text_tree(document, element)?.map(|value| {
            Instruction::Xslt10TextTreeVariable {
                name: name.to_owned(),
                value,
                location: location.clone(),
            }
        }),
    )
}

pub(super) fn compile_xslt10_static_text_tree(
    document: &Document,
    element: NodeId,
) -> Result<Option<String>, CompileFailure> {
    if !uses_xslt10_compatibility(document, element) {
        return Ok(None);
    }
    let children = meaningful_children(document, element);
    if children.is_empty()
        || !children.iter().all(|child| {
            document.kind(*child) == NodeKind::Text || is_xslt_element(document, *child, "text")
        })
    {
        return Ok(None);
    }
    let mut value = String::new();
    for child in children {
        if document.kind(child) == NodeKind::Text {
            value.push_str(document.value(child).unwrap_or_default());
            continue;
        }
        let Instruction::Text { value: text, .. } = compile_text(document, child)? else {
            unreachable!("compile_text returns one text instruction")
        };
        value.push_str(&text);
    }
    Ok(Some(value))
}

fn compile_xslt10_value_of_tree_variable(
    document: &Document,
    element: NodeId,
    name: &str,
    location: &SourceLocation,
) -> Result<Option<Instruction>, CompileFailure> {
    if optional_attribute(document, element, None, "as").is_some()
        || !uses_xslt10_compatibility(document, element)
    {
        return Ok(None);
    }
    let children = meaningful_children(document, element);
    let [value_of] = children.as_slice() else {
        return Ok(None);
    };
    if !is_xslt_element(document, *value_of, "value-of") {
        return Ok(None);
    }
    ensure_only_attributes(document, *value_of, &["select"], "xsl:value-of")?;
    ensure_no_meaningful_children(document, *value_of, "xsl:value-of")?;
    let select = required_attribute(document, *value_of, None, "select")?;
    Ok(Some(Instruction::Xslt10ValueOfTreeVariable {
        name: name.to_owned(),
        select: compile_value_expression(
            document,
            *value_of,
            select,
            document.location(*value_of),
        )?,
        location: location.clone(),
    }))
}

pub(super) fn compile_xslt10_for_each_text_path(
    document: &Document,
    element: NodeId,
) -> Result<Option<LocationPath>, CompileFailure> {
    if !uses_xslt10_compatibility(document, element) {
        return Ok(None);
    }
    let children = meaningful_children(document, element);
    let [for_each] = children.as_slice() else {
        return Ok(None);
    };
    if !is_xslt_element(document, *for_each, "for-each") {
        return Ok(None);
    }
    ensure_only_attributes(document, *for_each, &["select"], "xsl:for-each")?;
    let body = meaningful_children(document, *for_each);
    let [value_of] = body.as_slice() else {
        return Ok(None);
    };
    if !is_xslt_element(document, *value_of, "value-of")
        || required_attribute(document, *value_of, None, "select")?.trim() != "."
    {
        return Ok(None);
    }
    ensure_only_attributes(document, *value_of, &["select"], "xsl:value-of")?;
    ensure_no_meaningful_children(document, *value_of, "xsl:value-of")?;
    let select = required_attribute(document, *for_each, None, "select")?;
    parse_location_path(select, document.location(*for_each).clone())
        .map(Some)
        .map_err(map_path_failure)
}

fn compile_xslt10_for_each_text_variable(
    document: &Document,
    element: NodeId,
    name: &str,
    location: &SourceLocation,
) -> Result<Option<Instruction>, CompileFailure> {
    Ok(
        compile_xslt10_for_each_text_path(document, element)?.map(|select| {
            Instruction::Xslt10ForEachTextTreeVariable {
                name: name.to_owned(),
                select,
                location: location.clone(),
            }
        }),
    )
}

fn compile_source_node_union_variable(
    name: &str,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<Instruction>, CompileFailure> {
    let Some(alternatives) = split_top_level_union(expression) else {
        return Ok(None);
    };
    let sources = alternatives
        .into_iter()
        .map(str::trim)
        .map(|alternative| {
            alternative
                .strip_prefix('$')
                .filter(|source| is_ascii_ncname(source))
                .map(str::to_owned)
                .ok_or_else(|| {
                    unsupported(
                        "FXXP1001",
                        "local node-set union currently requires variable-only operands",
                        location,
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(Instruction::SourceNodeUnionVariable {
        name: name.to_owned(),
        sources,
        location: location.clone(),
    }))
}

fn compile_static_atomic_variable(
    name: &str,
    expression: &str,
    location: &SourceLocation,
) -> Option<Instruction> {
    let value = if let Some(value) = xpath_string_literal(expression.trim()) {
        AtomicValue::string(value.to_owned())
    } else if let Ok(value) = expression.trim().parse::<i64>() {
        AtomicValue::from_validated_lexical(BuiltinAtomicType::Integer, value.to_string())
    } else if let Some(value) = parse_static_contains(expression) {
        AtomicValue::from_validated_lexical(BuiltinAtomicType::Boolean, value.to_string())
    } else {
        let value =
            crate::xpath::constant_numeric_experiment::fold_exact_integral_arithmetic(expression)?;
        AtomicValue::from_validated_lexical(BuiltinAtomicType::Integer, value)
    };
    Some(Instruction::StaticAtomicVariable {
        name: name.to_owned(),
        value,
        location: location.clone(),
    })
}

fn parse_static_contains(expression: &str) -> Option<bool> {
    let arguments = expression
        .trim()
        .strip_prefix("contains(")?
        .strip_suffix(')')?;
    let arguments = crate::xpath::static_string_experiment::split_arguments(arguments, 2)?;
    let [haystack, needle] = arguments.as_slice() else {
        return None;
    };
    Some(xpath_string_literal(haystack)?.contains(xpath_string_literal(needle)?))
}

fn compile_integer_range_variable(
    document: &Document,
    element: NodeId,
    name: &str,
    location: &SourceLocation,
) -> Result<Instruction, CompileFailure> {
    let sequence_type = optional_attribute(document, element, None, "as").ok_or_else(|| {
        unsupported(
            "FXST1016",
            "constructed local variables require an admitted sequence type",
            location,
        )
    })?;
    if sequence_type != "xs:integer *"
        || namespace_for_prefix(document, element, "xs") != Some(XML_SCHEMA_NAMESPACE)
    {
        return Err(unsupported(
            "FXST1016",
            format!("unsupported constructed local variable type: {sequence_type}"),
            location,
        ));
    }
    let children = meaningful_children(document, element);
    let [for_each] = children.as_slice() else {
        return Err(unsupported(
            "FXST1017",
            "the admitted constructed integer sequence requires one xsl:for-each",
            location,
        ));
    };
    if !is_xslt_element(document, *for_each, "for-each") {
        return Err(unsupported(
            "FXST1017",
            "the admitted constructed integer sequence requires xsl:for-each",
            document.location(*for_each),
        ));
    }
    ensure_only_attributes(document, *for_each, &["select"], "xsl:for-each")?;
    let range = required_attribute(document, *for_each, None, "select")?;
    let (start, end) = parse_integer_range(range, document.location(*for_each))?;
    validate_atomized_range_body(document, *for_each)?;
    Ok(Instruction::IntegerRangeVariable {
        name: name.to_owned(),
        start,
        end,
        location: location.clone(),
    })
}

fn parse_integer_range(
    expression: &str,
    location: &SourceLocation,
) -> Result<(i64, i64), CompileFailure> {
    let Some((start, end)) = expression.split_once(" to ") else {
        return Err(unsupported(
            "FXXP1010",
            format!("unsupported integer range: {expression}"),
            location,
        ));
    };
    let start = start.trim().parse::<i64>().map_err(|_| {
        invalid(
            "FXXP0004",
            format!("invalid integer range start: {start}"),
            location,
        )
    })?;
    let end = end.trim().parse::<i64>().map_err(|_| {
        invalid(
            "FXXP0004",
            format!("invalid integer range end: {end}"),
            location,
        )
    })?;
    Ok((start, end))
}

fn validate_atomized_range_body(
    document: &Document,
    for_each: NodeId,
) -> Result<(), CompileFailure> {
    let children = meaningful_children(document, for_each);
    let [wrapper] = children.as_slice() else {
        return Err(unsupported(
            "FXST1017",
            "the admitted integer range body requires one literal wrapper",
            document.location(for_each),
        ));
    };
    if document
        .name(*wrapper)
        .is_none_or(|name| name.namespace.as_deref() == Some(XSLT_NAMESPACE))
        || !document.attributes(*wrapper).is_empty()
    {
        return Err(unsupported(
            "FXST1017",
            "the admitted integer range body requires an attribute-free literal wrapper",
            document.location(*wrapper),
        ));
    }
    let body = meaningful_children(document, *wrapper);
    let [value_of] = body.as_slice() else {
        return Err(unsupported(
            "FXST1017",
            "the admitted integer range wrapper requires one xsl:value-of",
            document.location(*wrapper),
        ));
    };
    if !is_xslt_element(document, *value_of, "value-of")
        || required_attribute(document, *value_of, None, "select")? != "."
    {
        return Err(unsupported(
            "FXST1017",
            "the admitted integer range wrapper must atomize the range item",
            document.location(*value_of),
        ));
    }
    ensure_only_attributes(document, *value_of, &["select"], "xsl:value-of")?;
    ensure_no_meaningful_children(document, *value_of, "xsl:value-of")
}

fn namespace_for_prefix<'a>(
    document: &'a Document,
    element: NodeId,
    prefix: &str,
) -> Option<&'a str> {
    if prefix == "xml" {
        return Some("http://www.w3.org/XML/1998/namespace");
    }
    let requested_prefix = (!prefix.is_empty()).then_some(prefix);
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(binding) = document
            .namespace_declarations(node)
            .iter()
            .find(|binding| binding.prefix.as_deref() == requested_prefix)
        {
            return Some(binding.namespace.as_str());
        }
        current = document.parent(node);
    }
    None
}

fn compile_sequence_nodes(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(document, element, &["select"], "xsl:sequence")?;
    ensure_no_meaningful_children(document, element, "xsl:sequence")?;
    let location = document.location(element).clone();
    let expression = required_attribute(document, element, None, "select")?;
    let sequence_items: Vec<_> = expression.split(',').map(str::trim).collect();
    if sequence_items
        .iter()
        .all(|item| *item == "*" || item.starts_with('$'))
    {
        let mut select = Vec::new();
        for item in sequence_items {
            if item == "*" {
                select.push(SequenceItemExpression::ChildElements);
            } else if let Some(variable) = item.strip_prefix('$') {
                select.push(SequenceItemExpression::Variable(normalize_variable_qname(
                    document, element, variable,
                )?));
            } else {
                return Err(unsupported(
                    "FXXP1003",
                    format!("unsupported xsl:sequence item expression: {item}"),
                    &location,
                ));
            }
        }
        return Ok(Instruction::SequenceItems { select, location });
    }
    let select =
        parse_for_distinct_values(expression, location.clone()).map_err(
            |failure| match failure {
                ForExpressionFailure::Invalid { detail, location } => CompileFailure {
                    code: "FXXP0003",
                    category: CompileCategory::Invalid,
                    detail,
                    location,
                },
                ForExpressionFailure::Unsupported { detail, location } => CompileFailure {
                    code: "FXXP1003",
                    category: CompileCategory::Unsupported,
                    detail,
                    location,
                },
            },
        )?;
    Ok(Instruction::SequenceNodes {
        select: Box::new(select),
        location,
    })
}

fn compile_if(document: &Document, element: NodeId) -> Result<Instruction, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &["test", "default-mode", "xpath-default-namespace"],
        "xsl:if",
    )?;
    let location = document.location(element).clone();
    let expression = required_conditional_test(document, element)?;
    Ok(Instruction::If {
        test: compile_boolean_test(document, element, expression, &location)?,
        body: compile_sequence(document, element)?,
        location,
    })
}

pub(super) fn compile_choose(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    ensure_choose_attributes(document, element)?;
    let children = meaningful_children(document, element);
    validate_choose_structure(document, element, &children)?;
    let mut branches = Vec::new();
    let mut otherwise = None;
    for child in children {
        if is_xslt_element(document, child, "when") {
            ensure_only_attributes(
                document,
                child,
                &["test", "xpath-default-namespace"],
                "xsl:when",
            )?;
            let expression = required_conditional_test(document, child)?;
            branches.push(ChooseBranch {
                test: compile_boolean_test(document, child, expression, document.location(child))?,
                body: compile_sequence(document, child)?,
            });
        } else if is_xslt_element(document, child, "otherwise") {
            ensure_only_attributes(
                document,
                child,
                &["xpath-default-namespace"],
                "xsl:otherwise",
            )?;
            otherwise = Some(compile_sequence(document, child)?);
        } else {
            unreachable!("validate_choose_structure rejects other children")
        }
    }
    Ok(Instruction::Choose {
        branches,
        otherwise: otherwise.unwrap_or_default(),
        location: document.location(element).clone(),
    })
}

fn compile_boolean_test(
    document: &Document,
    element: NodeId,
    expression: &str,
    location: &crate::xdm::owned_tree_experiment::SourceLocation,
) -> Result<BooleanExpression, CompileFailure> {
    if uses_xslt10_compatibility(document, element)
        && let Some(value) = xslt10_static_introspection_compiler::fold_effective_boolean(
            document, element, expression,
        )
    {
        return Ok(BooleanExpression::Constant(value));
    }
    boolean_expression_compiler::compile(
        document,
        element,
        expression,
        location,
        effective_string_comparison(document, element)?,
        uses_xslt10_compatibility(document, element),
    )
}

fn ensure_choose_attributes(document: &Document, element: NodeId) -> Result<(), CompileFailure> {
    const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
    for attribute in document.attributes(element) {
        let name = document
            .name(*attribute)
            .expect("attribute nodes have expanded names");
        if name.namespace.is_none() && name.local == "xpath-default-namespace" {
            continue;
        }
        if name.namespace.is_none() && name.local == "default-collation" {
            continue;
        }
        if name.namespace.as_deref() == Some(XML_NAMESPACE) && name.local == "space" {
            match document.value(*attribute).unwrap_or_default() {
                "default" | "preserve" => continue,
                _ => {
                    return Err(invalid(
                        "XTSE0020",
                        "xml:space must be 'default' or 'preserve'",
                        document.location(*attribute),
                    ));
                }
            }
        }
        return Err(unsupported(
            "FXST1009",
            format!(
                "unsupported attribute on xsl:choose: {{{}}}{}",
                name.namespace.as_deref().unwrap_or(""),
                name.local
            ),
            document.location(*attribute),
        ));
    }
    Ok(())
}

fn effective_string_comparison(
    document: &Document,
    element: NodeId,
) -> Result<StringComparison, CompileFailure> {
    const CODEPOINT: &str = "http://www.w3.org/2005/xpath-functions/collation/codepoint";
    const HTML_ASCII: &str =
        "http://www.w3.org/2005/xpath-functions/collation/html-ascii-case-insensitive";
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(collations) = optional_attribute(document, node, None, "default-collation") {
            for collation in collations.split_whitespace() {
                match collation {
                    CODEPOINT => return Ok(StringComparison::Codepoint),
                    HTML_ASCII => return Ok(StringComparison::HtmlAsciiCaseInsensitive),
                    _ => {}
                }
            }
            return Err(invalid(
                "XTSE0125",
                "default-collation does not name an available collation",
                document.location(node),
            ));
        }
        current = document.parent(node);
    }
    Ok(StringComparison::Codepoint)
}

fn effective_xml_space_preserved(
    document: &Document,
    element: NodeId,
) -> Result<bool, CompileFailure> {
    const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(value) = optional_attribute(document, node, Some(XML_NAMESPACE), "space") {
            return match value {
                "preserve" => Ok(true),
                "default" => Ok(false),
                _ => Err(invalid(
                    "XTSE0020",
                    "xml:space must be 'default' or 'preserve'",
                    document.location(node),
                )),
            };
        }
        current = document.parent(node);
    }
    Ok(false)
}

fn validate_choose_structure(
    document: &Document,
    element: NodeId,
    children: &[NodeId],
) -> Result<(), CompileFailure> {
    let mut when_count = 0_usize;
    let mut saw_otherwise = false;
    for child in children {
        if is_xslt_element(document, *child, "when") {
            if saw_otherwise {
                return Err(invalid(
                    "XTSE0010",
                    "xsl:when cannot follow xsl:otherwise",
                    document.location(*child),
                ));
            }
            required_conditional_test(document, *child)?;
            when_count += 1;
        } else if is_xslt_element(document, *child, "otherwise") {
            if saw_otherwise {
                return Err(invalid(
                    "XTSE0010",
                    "xsl:choose permits at most one xsl:otherwise",
                    document.location(*child),
                ));
            }
            saw_otherwise = true;
        } else {
            return Err(invalid(
                "XTSE0010",
                "xsl:choose permits only xsl:when and xsl:otherwise children",
                document.location(*child),
            ));
        }
    }
    if when_count == 0 {
        return Err(invalid(
            "XTSE0010",
            "xsl:choose requires at least one xsl:when",
            document.location(element),
        ));
    }
    Ok(())
}

fn required_conditional_test(document: &Document, element: NodeId) -> Result<&str, CompileFailure> {
    optional_attribute(document, element, None, "test").ok_or_else(|| {
        invalid(
            "XTSE0010",
            "xsl:if and xsl:when require a test attribute",
            document.location(element),
        )
    })
}

fn xpath_string_literal(expression: &str) -> Option<&str> {
    if expression.len() < 2 {
        return None;
    }
    let quote = expression.as_bytes()[0];
    if !matches!(quote, b'\'' | b'"') || expression.as_bytes().last() != Some(&quote) {
        return None;
    }
    let value = &expression[1..expression.len() - 1];
    (!value.as_bytes().contains(&quote)).then_some(value)
}

fn parse_generated_temporary_root(expression: &str) -> Option<(&str, Option<&str>)> {
    let argument = generated_root_argument(expression)?;
    let argument = argument.strip_prefix('$')?;
    let (variable, descendant) = argument
        .split_once("//")
        .map_or((argument, None), |(variable, descendant)| {
            (variable, Some(descendant))
        });
    if !is_ascii_ncname(variable)
        || descendant.is_some_and(|descendant| !is_ascii_ncname(descendant))
    {
        return None;
    }
    Some((variable, descendant))
}

fn parse_generated_document_root(expression: &str) -> Option<(&str, Option<&str>)> {
    let argument = generated_root_argument(expression)?;
    let argument = argument.strip_prefix("document('")?;
    let (reference, suffix) = argument.split_once("')")?;
    if reference.is_empty() || reference.contains('\'') {
        return None;
    }
    let descendant = if suffix.is_empty() {
        None
    } else {
        Some(suffix.strip_prefix("//")?)
    };
    if descendant.is_some_and(|name| !is_ascii_ncname(name)) {
        return None;
    }
    Some((reference, descendant))
}

fn unsupported_boolean_expression(expression: &str, location: &SourceLocation) -> CompileFailure {
    unsupported(
        "FXXP1002",
        format!("unsupported conditional expression: {expression}"),
        location,
    )
}

fn compile_call_template(
    document: &Document,
    element: NodeId,
) -> Result<Instruction, CompileFailure> {
    template_invocation_compiler::compile_call_template(document, element)
}
