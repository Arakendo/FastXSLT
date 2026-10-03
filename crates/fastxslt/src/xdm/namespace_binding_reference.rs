//! Shared test-only binding and owner qualification; no runtime provider API.

use super::owned_tree_experiment::{Document, NodeId};
use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xml::quick_xml_experiment::NamespaceBinding;
use std::collections::BTreeMap;

pub(crate) const XML_URI: &str = "http://www.w3.org/XML/1998/namespace";

#[derive(Debug, Default)]
pub(crate) struct InvocationScope {
    // Distinct live non-zero-sized objects have distinct borrowed identities.
    _private: u8,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum NamespaceOwner<'a> {
    Source {
        origin: &'a Document,
        element: NodeId,
    },
    Temporary {
        invocation: &'a InvocationScope,
        tree: u64,
        element: usize,
    },
}

impl NamespaceOwner<'_> {
    pub(crate) fn same_namespace(self, prefix: &str, other: Self, other_prefix: &str) -> bool {
        if prefix != other_prefix {
            return false;
        }
        match (self, other) {
            (
                Self::Source { origin, element },
                Self::Source {
                    origin: other,
                    element: other_element,
                },
            ) => std::ptr::eq(origin, other) && element == other_element,
            (
                Self::Temporary {
                    invocation,
                    tree,
                    element,
                },
                Self::Temporary {
                    invocation: other,
                    tree: other_tree,
                    element: other_element,
                },
            ) => std::ptr::eq(invocation, other) && tree == other_tree && element == other_element,
            _ => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum BindingFailure {
    Capacity,
    Control(ControlFailure),
}

// Owners supply nearest-first declaration rows, not navigation or authority.
// P retains their honest provenance: source spans or explicit absence.
pub(crate) fn resolve<'a, P: Copy>(
    ancestors: impl Iterator<Item = (&'a [NamespaceBinding], P)>,
    xml_provenance: P,
    max_bindings: usize,
    control: &mut InvocationControl,
) -> Result<BTreeMap<&'a str, (&'a str, P)>, BindingFailure> {
    if max_bindings == 0 {
        return Err(BindingFailure::Capacity);
    }
    let mut bindings = BTreeMap::from([("xml", (XML_URI, xml_provenance))]);
    for (declarations, provenance) in ancestors {
        charge(control)?;
        for binding in declarations {
            charge(control)?;
            let prefix = binding.prefix.as_deref().unwrap_or("");
            if bindings.contains_key(prefix) {
                continue;
            }
            if bindings.len() == max_bindings {
                return Err(BindingFailure::Capacity);
            }
            bindings.insert(prefix, (binding.namespace.as_str(), provenance));
        }
    }
    // Keep undeclaration tombstones until consumers materialize occurrences.
    Ok(bindings)
}

fn charge(control: &mut InvocationControl) -> Result<(), BindingFailure> {
    control
        .charge(WorkDomain::XdmNode, 1)
        .map_err(BindingFailure::Control)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    #[test]
    fn namespace_owner_domains_do_not_alias_even_with_matching_local_numbers() {
        let source = Document::from_parsed(
            parse_document(
                "memory:owner.xml",
                b"<r/>",
                ParseLimits {
                    max_events: 10,
                    max_depth: 2,
                },
            )
            .unwrap(),
        )
        .unwrap();
        let element = source.children(source.document_node())[0];
        let invocation = InvocationScope::default();
        let owner = NamespaceOwner::Source {
            origin: &source,
            element,
        };
        let temporary = NamespaceOwner::Temporary {
            invocation: &invocation,
            tree: 0,
            element: element.index(),
        };
        assert!(!owner.same_namespace("p", temporary, "p"));
        assert!(!temporary.same_namespace("p", owner, "p"));
        assert!(temporary.same_namespace("p", temporary, "p"));
        assert!(!temporary.same_namespace("p", temporary, "q"));
    }
}
