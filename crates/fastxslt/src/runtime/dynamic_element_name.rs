//! Runtime `QName` resolution for path-valued computed-element names.

use std::sync::Arc;

use crate::execution_control_experiment::InvocationControl;
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::path_experiment::{LocationPath, evaluate_location_path_controlled};
use crate::xslt::golden_semantics_experiment::DynamicElementName;

use super::runtime_context::{RuntimeVariables, SequenceInputs};
use super::{
    ExecutionFailure, FailureCategory, SequenceContext, control_failure, failure_at,
    required_source_context,
};

#[derive(Clone, Copy)]
pub(super) struct DynamicElementNameRequest<'a> {
    pub(super) name: &'a DynamicElementName,
    pub(super) variables: &'a RuntimeVariables,
    pub(super) namespace_override: Option<&'a str>,
    pub(super) static_namespaces: &'a [NamespaceBinding],
    pub(super) location: &'a SourceLocation,
}

pub(super) fn resolve_dynamic_element_name(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    request: DynamicElementNameRequest<'_>,
    control: &mut InvocationControl,
) -> Result<(ExpandedName, Arc<[NamespaceBinding]>), ExecutionFailure> {
    let lexical = match request.name {
        DynamicElementName::Path(path) => path_name_value(inputs, execution, path, control)?,
        DynamicElementName::FocusPosition { prefix, suffix } => {
            format!("{prefix}{}{suffix}", execution.focus_position)
        }
        DynamicElementName::VariableAvt(parts) => {
            super::dynamic_attribute_name::variable_avt_value(
                inputs,
                request.variables,
                parts,
                execution.focus_position,
                request.location,
                control,
            )?
        }
    };
    resolve_lexical_element_name(
        &lexical,
        request.namespace_override,
        request.static_namespaces,
        request.location,
        inputs.request_id,
    )
}

fn path_name_value(
    inputs: &SequenceInputs<'_>,
    execution: SequenceContext<'_>,
    path: &LocationPath,
    control: &mut InvocationControl,
) -> Result<String, ExecutionFailure> {
    let (source, context) = required_source_context(inputs, execution.node)?;
    let selected = evaluate_location_path_controlled(source, context, path, control)
        .map_err(|failure| control_failure(failure, inputs.request_id))?;
    Ok(selected
        .first()
        .map(|node| source.string_value(*node))
        .unwrap_or_default())
}

fn resolve_lexical_element_name(
    lexical: &str,
    namespace_override: Option<&str>,
    static_namespaces: &[NamespaceBinding],
    location: &SourceLocation,
    request_id: &str,
) -> Result<(ExpandedName, Arc<[NamespaceBinding]>), ExecutionFailure> {
    const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";
    if namespace_override == Some(XMLNS_NAMESPACE) {
        return Err(dynamic_name_failure(
            "XTDE0835",
            "xsl:element cannot construct a name in the reserved xmlns namespace",
            location,
            request_id,
        ));
    }
    let (prefix, local) = match lexical.split_once(':') {
        Some((prefix, local))
            if is_ascii_ncname(prefix) && is_ascii_ncname(local) && !local.contains(':') =>
        {
            (Some(prefix), local)
        }
        None if is_ascii_ncname(lexical) => (None, lexical),
        _ => {
            return Err(dynamic_name_failure(
                "XTDE0820",
                "xsl:element name AVT did not produce a lexical QName",
                location,
                request_id,
            ));
        }
    };
    let namespace = match namespace_override {
        Some("") if prefix.is_some() => {
            return Err(dynamic_name_failure(
                "XTDE0835",
                "a prefixed xsl:element name cannot use an empty namespace",
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
            "XTDE0830",
            "xsl:element dynamic name uses an unbound prefix",
            location,
            request_id,
        ));
    }
    let namespaces: Arc<[NamespaceBinding]> = match (prefix, namespace.as_deref()) {
        (Some(prefix), Some(namespace)) => Arc::from([NamespaceBinding {
            prefix: Some(prefix.to_owned()),
            namespace: namespace.to_owned(),
        }]),
        (None, Some(namespace)) => Arc::from([NamespaceBinding {
            prefix: None,
            namespace: namespace.to_owned(),
        }]),
        _ => Arc::from([]),
    };
    Ok((
        ExpandedName {
            namespace,
            local: local.to_owned(),
        },
        namespaces,
    ))
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
