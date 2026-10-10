//! Private static and dynamic XSLT computed-element compilation.

use super::computed_attribute_compiler::parse_xslt10_name_avt_parts;
use super::{
    CompileFailure, ComputedAttribute, Document, DynamicNamespaceValue, ElementConstructorOrigin,
    ExpandedName, Instruction, NamespaceBinding, NodeId, compile_attribute_set_use_names,
    compile_computed_attributes, compile_sequence_excluding, compile_sort_path,
    ensure_only_attributes, invalid, is_ascii_ncname, namespace_for_prefix, optional_attribute,
    required_attribute, retain_computed_attribute_namespace_bindings, unsupported,
    uses_xslt10_compatibility, value_expression_compiler,
};
use crate::xslt::golden_semantics_experiment::DynamicElementName;

pub(super) fn compile_static_computed_element(
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
    let namespace_override = compile_dynamic_element_namespace(document, element, namespace)?;
    if matches!(name.trim(), "{name()}" | "{name(.)}") {
        let ComputedElementContent {
            attribute_set_names,
            computed_attributes,
            body,
        } = compile_computed_element_content(document, element)?;
        return Ok(Instruction::ContextNameElement {
            xslt10_compatibility: uses_xslt10_compatibility(document, element),
            namespace_override,
            static_namespaces: document.in_scope_namespaces(element).into(),
            attribute_set_names,
            computed_attributes,
            body,
            location: document.location(element).clone(),
        });
    }
    if uses_xslt10_compatibility(document, element)
        && let Some(name) = compile_dynamic_element_name(document, element, name)?
    {
        let ComputedElementContent {
            attribute_set_names,
            computed_attributes,
            body,
        } = compile_computed_element_content(document, element)?;
        return Ok(Instruction::DynamicNameElement {
            xslt10_compatibility: true,
            name,
            namespace_override,
            static_namespaces: document.in_scope_namespaces(element).into(),
            attribute_set_names,
            computed_attributes,
            body,
            location: document.location(element).clone(),
        });
    }
    if matches!(namespace_override, Some(DynamicNamespaceValue::Path(_))) {
        let ComputedElementContent {
            attribute_set_names,
            computed_attributes,
            body,
        } = compile_computed_element_content(document, element)?;
        return Ok(Instruction::DynamicNameElement {
            xslt10_compatibility: uses_xslt10_compatibility(document, element),
            name: DynamicElementName::Literal(name.to_owned()),
            namespace_override,
            static_namespaces: document.in_scope_namespaces(element).into(),
            attribute_set_names,
            computed_attributes,
            body,
            location: document.location(element).clone(),
        });
    }
    let static_namespace = namespace_override
        .as_ref()
        .map(|namespace| match namespace {
            DynamicNamespaceValue::Static(value) => value.as_str(),
            DynamicNamespaceValue::Path(_) => unreachable!("dynamic namespace returned above"),
        });
    let (name, mut namespaces) =
        compile_static_computed_element_name(document, element, name, static_namespace)?;
    let (computed_attributes, computed_attribute_nodes) =
        compile_computed_attributes(document, element)?;
    let attribute_set_names = compile_attribute_set_use_names(document, element, None)?;
    retain_computed_attribute_namespace_bindings(&mut namespaces, &computed_attributes);
    Ok(Instruction::LiteralElement {
        origin: ElementConstructorOrigin::ComputedStatic,
        name,
        namespaces: namespaces.into(),
        attributes: Vec::new(),
        attribute_set_names,
        computed_attributes,
        body: compile_sequence_excluding(document, element, &computed_attribute_nodes)?,
        location: document.location(element).clone(),
    })
}

fn compile_computed_element_content(
    document: &Document,
    element: NodeId,
) -> Result<ComputedElementContent, CompileFailure> {
    let (computed_attributes, computed_attribute_nodes) =
        compile_computed_attributes(document, element)?;
    let attribute_set_names = compile_attribute_set_use_names(document, element, None)?;
    let body = compile_sequence_excluding(document, element, &computed_attribute_nodes)?;
    Ok(ComputedElementContent {
        attribute_set_names,
        computed_attributes,
        body,
    })
}

struct ComputedElementContent {
    attribute_set_names: Vec<ExpandedName>,
    computed_attributes: Vec<ComputedAttribute>,
    body: Vec<Instruction>,
}

fn compile_dynamic_element_namespace(
    document: &Document,
    element: NodeId,
    lexical: Option<&str>,
) -> Result<Option<DynamicNamespaceValue>, CompileFailure> {
    let Some(lexical) = lexical else {
        return Ok(None);
    };
    if !lexical.contains(['{', '}']) {
        return Ok(Some(DynamicNamespaceValue::Static(lexical.to_owned())));
    }
    if uses_xslt10_compatibility(document, element)
        && let Some(expression) = lexical
            .strip_prefix('{')
            .and_then(|value| value.strip_suffix('}'))
        && !expression.contains(['{', '}'])
        && let Ok(path) = compile_sort_path(
            document,
            element,
            expression.trim(),
            document.location(element),
        )
    {
        return Ok(Some(DynamicNamespaceValue::Path(Box::new(path))));
    }
    Err(unsupported(
        "FXST1045",
        "the private xsl:element namespace slice requires a static URI or one XSLT 1.0 path AVT",
        document.location(element),
    ))
}

fn compile_dynamic_element_name(
    document: &Document,
    element: NodeId,
    lexical: &str,
) -> Result<Option<DynamicElementName>, CompileFailure> {
    if let Some((prefix, suffix)) = lexical.split_once("{position()}")
        && !prefix.contains(['{', '}'])
        && !suffix.contains(['{', '}'])
    {
        return Ok(Some(DynamicElementName::FocusPosition {
            prefix: prefix.to_owned(),
            suffix: suffix.to_owned(),
        }));
    }
    if let Some(parts) = parse_xslt10_name_avt_parts(lexical) {
        return Ok(Some(DynamicElementName::VariableAvt(parts)));
    }
    let Some(expression) = lexical
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .map(str::trim)
    else {
        return Ok(None);
    };
    if expression.contains(['{', '}']) {
        return Ok(None);
    }
    if let Some((variable, path)) = value_expression_compiler::compile_xslt10_variable_path(
        document,
        element,
        expression,
        document.location(element),
    )? {
        return Ok(Some(DynamicElementName::SourceVariablePath {
            variable,
            path,
        }));
    }
    Ok(
        compile_sort_path(document, element, expression, document.location(element))
            .ok()
            .map(DynamicElementName::Path),
    )
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
        if uses_xslt10_compatibility(document, element)
            && let Some(namespace) = namespace_override
        {
            return Ok(explicit_xslt10_name(None, lexical, namespace));
        }
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
    if uses_xslt10_compatibility(document, element)
        && let Some(namespace) = namespace_override
    {
        return Ok(explicit_xslt10_name(Some(prefix), local, namespace));
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

fn explicit_xslt10_name(
    prefix: Option<&str>,
    local: &str,
    namespace: &str,
) -> (ExpandedName, Vec<NamespaceBinding>) {
    const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
    let prefix = if namespace == XML_NAMESPACE {
        Some("xml")
    } else {
        prefix.filter(|prefix| !matches!(*prefix, "xml" | "xmlns"))
    };
    (
        ExpandedName {
            namespace: (!namespace.is_empty()).then(|| namespace.to_owned()),
            local: local.to_owned(),
        },
        if namespace.is_empty() {
            Vec::new()
        } else {
            vec![NamespaceBinding {
                prefix: prefix.map(str::to_owned),
                namespace: namespace.to_owned(),
            }]
        },
    )
}
