//! Private semantic result-tree representation and literal-attribute materialization.

use crate::execution_control_experiment::{InvocationControl, WorkDomain};
use crate::xml::quick_xml_experiment::{ExpandedName, NamespaceBinding};
use crate::xslt::golden_semantics_experiment::{LiteralAttribute, LiteralAttributeValue};

use super::runtime_context::RuntimeVariables;
use super::{ExecutionFailure, FailureCategory, control_failure, failure_at};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ResultNode {
    Element {
        name: ExpandedName,
        namespaces: Vec<NamespaceBinding>,
        attributes: Vec<ResultAttribute>,
        children: Vec<ResultNode>,
    },
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResultAttribute {
    pub(super) name: ExpandedName,
    pub(super) value: String,
}

pub(super) fn materialize_literal_attributes(
    attributes: &[LiteralAttribute],
    variables: &RuntimeVariables,
    request_id: &str,
    control: &mut InvocationControl,
) -> Result<Vec<ResultAttribute>, ExecutionFailure> {
    attributes
        .iter()
        .map(|attribute| {
            control
                .charge(WorkDomain::ResultNode, 1)
                .map_err(|failure| control_failure(failure, request_id))?;
            let value = match &attribute.value {
                LiteralAttributeValue::Text(value) => value.clone(),
                LiteralAttributeValue::Variable(name) => variables
                    .atomics
                    .get(name)
                    .ok_or_else(|| {
                        failure_at(
                            "FXRT0002",
                            FailureCategory::Invalid,
                            Some(request_id),
                            attribute.location.clone(),
                            format!("unbound variable in attribute value template: ${name}"),
                        )
                    })?
                    .lexical()
                    .to_owned(),
            };
            Ok(ResultAttribute {
                name: attribute.name.clone(),
                value,
            })
        })
        .collect()
}
