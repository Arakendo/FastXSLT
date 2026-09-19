//! Runtime `QName` resolution for the bounded path-valued `xsl:attribute` name slice.

use crate::execution_control_experiment::InvocationControl;
use crate::xdm::owned_tree_experiment::SourceLocation;
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xpath::path_experiment::{LocationPath, evaluate_location_path_controlled};
use crate::xslt::golden_semantics_experiment::DynamicAttributeName;

use super::runtime_context::SequenceInputs;
use super::{
    ExecutionFailure, FailureCategory, control_failure, failure_at, required_source_context,
};

pub(super) fn resolve(
    inputs: &SequenceInputs<'_>,
    context: Option<crate::xdm::owned_tree_experiment::NodeId>,
    name: &DynamicAttributeName,
    location: &SourceLocation,
    control: &mut InvocationControl,
) -> Result<ExpandedName, ExecutionFailure> {
    let DynamicAttributeName::Path {
        path,
        namespace_override,
        static_namespaces,
    } = name;
    let lexical = path_name_value(inputs, context, path, control)?;
    resolve_lexical_name(
        &lexical,
        namespace_override.as_deref(),
        static_namespaces,
        location,
        inputs.request_id,
    )
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
