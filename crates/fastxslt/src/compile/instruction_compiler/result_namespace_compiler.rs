//! Static literal-result namespace selection: declaration-scoped exclusions and
//! effective bindings. No runtime namespace fixup, document acquisition or cache.
use super::{
    Document, NamespaceBinding, NodeId, XSLT_NAMESPACE, is_declared_extension_element,
    namespace_for_prefix, optional_attribute,
};

pub(super) fn literal_result_namespaces(
    document: &Document,
    element: NodeId,
) -> Vec<NamespaceBinding> {
    let mut namespaces = Vec::new();
    let (excluded_namespaces, exclude_all) = exclusions(document, element);
    let mut seen_prefixes = Vec::new();
    let mut current = Some(element);
    while let Some(node) = current {
        for binding in document.namespace_declarations(node) {
            let prefix = binding.prefix.as_deref();
            // Shadowing applies before exclusion, including undeclarations.
            // Filtering a nearer binding must not resurrect an ancestor's URI.
            if seen_prefixes.contains(&prefix) {
                continue;
            }
            seen_prefixes.push(prefix);
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
            let excluded = excluded_namespaces.contains(&binding.namespace.as_str());
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

fn exclusions(document: &Document, element: NodeId) -> (Vec<&str>, bool) {
    let mut excluded_namespaces = Vec::new();
    let mut exclude_all = false;
    let mut current = Some(element);
    while let Some(node) = current {
        let is_stylesheet_root = document.name(node).is_some_and(|name| {
            name.namespace.as_deref() == Some(XSLT_NAMESPACE)
                && matches!(name.local.as_str(), "stylesheet" | "transform")
        });
        let is_literal_result = document.name(node).is_some_and(|name| {
            name.namespace.as_deref() != Some(XSLT_NAMESPACE)
                && !is_declared_extension_element(document, node)
        });
        let exclusions = if is_stylesheet_root {
            optional_attribute(document, node, None, "exclude-result-prefixes")
        } else if is_literal_result {
            optional_attribute(
                document,
                node,
                Some(XSLT_NAMESPACE),
                "exclude-result-prefixes",
            )
        } else {
            None
        };
        if let Some(exclusions) = exclusions {
            for prefix in exclusions.split_whitespace() {
                if prefix == "#all" {
                    exclude_all = true;
                } else if let Some(namespace) = namespace_for_prefix(
                    document,
                    node,
                    if prefix == "#default" { "" } else { prefix },
                ) && !excluded_namespaces.contains(&namespace)
                {
                    excluded_namespaces.push(namespace);
                }
            }
        }
        let extensions = if is_stylesheet_root {
            optional_attribute(document, node, None, "extension-element-prefixes")
        } else if is_literal_result {
            optional_attribute(
                document,
                node,
                Some(XSLT_NAMESPACE),
                "extension-element-prefixes",
            )
        } else {
            None
        };
        if let Some(extensions) = extensions {
            for prefix in extensions.split_whitespace() {
                if let Some(namespace) = namespace_for_prefix(
                    document,
                    node,
                    if prefix == "#default" { "" } else { prefix },
                ) && !excluded_namespaces.contains(&namespace)
                {
                    excluded_namespaces.push(namespace);
                }
            }
        }
        current = document.parent(node);
    }
    (excluded_namespaces, exclude_all)
}
