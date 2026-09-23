//! Runtime `QName` resolution for the bounded path-valued `xsl:attribute` name slice.

use crate::execution_control_experiment::InvocationControl;
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::path_experiment::{LocationPath, evaluate_location_path_controlled};
use crate::xslt::golden_semantics_experiment::{DynamicAttributeName, DynamicAttributeNamePart};

use super::runtime_context::{RuntimeVariables, SequenceInputs};
use super::{
    ExecutionFailure, FailureCategory, control_failure, failure_at, required_source_context,
};

pub(super) fn resolve(
    inputs: &SequenceInputs<'_>,
    context: Option<crate::xdm::owned_tree_experiment::NodeId>,
    focus_position: usize,
    name: &DynamicAttributeName,
    variables: &RuntimeVariables,
    location: &SourceLocation,
    control: &mut InvocationControl,
) -> Result<ExpandedName, ExecutionFailure> {
    let (lexical, namespace_override, static_namespaces) = match name {
        DynamicAttributeName::Path {
            path,
            namespace_override,
            static_namespaces,
        } => (
            path_name_value(inputs, context, path, control)?,
            namespace_override,
            static_namespaces,
        ),
        DynamicAttributeName::ContextName {
            namespace_override,
            static_namespaces,
        } => (
            context_lexical_name(inputs, context, control)?,
            namespace_override,
            static_namespaces,
        ),
        DynamicAttributeName::Literal {
            value,
            namespace_override,
            static_namespaces,
        } => (value.clone(), namespace_override, static_namespaces),
        DynamicAttributeName::VariableAvt {
            parts,
            namespace_override,
            static_namespaces,
        } => (
            variable_avt_value(inputs, variables, parts, focus_position, location, control)?,
            namespace_override,
            static_namespaces,
        ),
    };
    resolve_lexical_name(
        &lexical,
        namespace_override.as_deref(),
        static_namespaces,
        location,
        inputs.request_id,
    )
}

pub(super) fn variable_avt_value(
    inputs: &SequenceInputs<'_>,
    variables: &RuntimeVariables,
    parts: &[DynamicAttributeNamePart],
    focus_position: usize,
    location: &SourceLocation,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let mut value = String::new();
    for part in parts {
        match part {
            DynamicAttributeNamePart::Text(text) => value.push_str(text),
            DynamicAttributeNamePart::Variable(name) => {
                if let Some(atomic) = variables.atomics.get(name).or_else(|| {
                    variables
                        .allows_global_fallback(name)
                        .then(|| inputs.globals.atomics.get(name))
                        .flatten()
                }) {
                    value.push_str(atomic.lexical());
                } else if let Some(tree) = variables.temporary_tree(inputs.globals, name) {
                    value.push_str(&super::runtime_context::temporary_tree_string_value(
                        tree,
                        inputs.request_id,
                        control,
                    )?);
                } else {
                    return Err(failure_at(
                        "FXRT0002",
                        FailureCategory::Invalid,
                        Some(inputs.request_id),
                        location.clone(),
                        format!("unbound variable in computed-attribute name: ${name}"),
                    ));
                }
            }
            DynamicAttributeNamePart::Position => {
                control
                    .charge(
                        crate::execution_control_experiment::WorkDomain::XPathOperation,
                        1,
                    )
                    .map_err(|failure| control_failure(failure, inputs.request_id))?;
                value.push_str(&focus_position.to_string());
            }
        }
    }
    Ok(value)
}

fn context_lexical_name(
    inputs: &SequenceInputs<'_>,
    context: Option<crate::xdm::owned_tree_experiment::NodeId>,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    control
        .charge(
            crate::execution_control_experiment::WorkDomain::XPathNodeVisit,
            1,
        )
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    let Some(name) = source.name(context) else {
        return Ok(String::new());
    };
    Ok(source.prefix(context).map_or_else(
        || name.local.clone(),
        |prefix| format!("{prefix}:{}", name.local),
    ))
}

fn path_name_value(
    inputs: &SequenceInputs<'_>,
    context: Option<crate::xdm::owned_tree_experiment::NodeId>,
    path: &LocationPath,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, context)?;
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(selected
        .first()
        .map(|node| source.string_value(*node))
        .unwrap_or_default())
}

fn resolve_lexical_name(
    lexical: &str,
    namespace_override: Option<&str>,
    static_namespaces: &[NamespaceBinding],
    location: &SourceLocation,
    request_id: &str,
) -> Result<ExpandedName, ExecutionFailure> {
    const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
    const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";
    let (prefix, local) = match lexical.split_once(':') {
        Some((prefix, local))
            if is_ascii_ncname(prefix) && is_ascii_ncname(local) && !local.contains(':') =>
        {
            (Some(prefix), local)
        }
        None if is_ascii_ncname(lexical) => (None, lexical),
        _ => {
            return Err(dynamic_name_failure(
                "XTDE0850",
                "xsl:attribute name AVT did not produce a lexical QName",
                location,
                request_id,
            ));
        }
    };
    if prefix == Some("xmlns") || (prefix.is_none() && local == "xmlns") {
        return Err(dynamic_name_failure(
            "XTDE0855",
            "xsl:attribute cannot construct a name in the reserved xmlns namespace",
            location,
            request_id,
        ));
    }
    let namespace = match namespace_override {
        Some("") if prefix.is_some() => {
            return Err(dynamic_name_failure(
                "XTDE0860",
                "a prefixed xsl:attribute name cannot use an empty namespace",
                location,
                request_id,
            ));
        }
        Some("") => None,
        Some(namespace) => Some(namespace.to_owned()),
        None if prefix.is_some() => static_namespaces
            .iter()
            .find(|binding| binding.prefix.as_deref() == prefix)
            .map(|binding| binding.namespace.clone()),
        None => None,
    };
    if prefix.is_some() && namespace.is_none() {
        return Err(dynamic_name_failure(
            "XTDE0860",
            "xsl:attribute dynamic name uses an unbound prefix",
            location,
            request_id,
        ));
    }
    if (prefix == Some("xml")) != (namespace.as_deref() == Some(XML_NAMESPACE))
        || namespace.as_deref() == Some(XMLNS_NAMESPACE)
    {
        return Err(dynamic_name_failure(
            "XTDE0860",
            "the xml prefix and namespace must be used together",
            location,
            request_id,
        ));
    }
    Ok(ExpandedName {
        namespace,
        local: local.to_owned(),
    })
}

fn dynamic_name_failure(
    code: &'static str,
    detail: &'static str,
    location: &SourceLocation,
    request_id: &str,
) -> ExecutionFailure {
    failure_at(
        code,
        FailureCategory::Invalid,
        Some(request_id),
        location.clone(),
        detail,
    )
}

fn is_ascii_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        })
}
