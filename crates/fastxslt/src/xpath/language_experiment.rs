//! Private context-language matching for `fn:lang`.

use crate::execution_control_experiment::{ControlFailure, InvocationControl, WorkDomain};
use crate::xdm::owned_tree_experiment::{Document, NodeId};

const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";

pub(crate) fn parse_literal(expression: &str) -> Option<String> {
    let expression = expression.trim();
    let argument = ["lang", "fn:lang"].iter().find_map(|name| {
        expression
            .strip_prefix(name)
            .and_then(|tail| tail.strip_prefix('('))
            .and_then(|tail| tail.strip_suffix(')'))
    })?;
    for quote in ['\'', '"'] {
        if let Some(value) = argument
            .trim()
            .strip_prefix(quote)
            .and_then(|value| value.strip_suffix(quote))
            .filter(|value| !value.contains(quote))
        {
            return Some(value.to_owned());
        }
    }
    None
}

pub(crate) fn evaluate(
    document: &Document,
    context: NodeId,
    requested: &str,
    control: &mut InvocationControl,
) -> Result<bool, ControlFailure> {
    let mut current = Some(context);
    while let Some(node) = current {
        control.charge(WorkDomain::XPathNodeVisit, 1)?;
        for attribute in document.attributes(node) {
            control.charge(WorkDomain::XPathNodeVisit, 1)?;
            if document.name(*attribute).is_some_and(|name| {
                name.namespace.as_deref() == Some(XML_NAMESPACE) && name.local == "lang"
            }) {
                return Ok(document
                    .value(*attribute)
                    .is_some_and(|actual| language_matches(actual, requested)));
            }
        }
        current = document.parent(node);
    }
    Ok(false)
}

fn language_matches(actual: &str, requested: &str) -> bool {
    actual.eq_ignore_ascii_case(requested)
        || (actual
            .get(..requested.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(requested))
            && actual.as_bytes().get(requested.len()) == Some(&b'-'))
}

#[cfg(test)]
mod tests {
    use super::{evaluate, language_matches, parse_literal};
    use crate::execution_control_experiment::InvocationControl;
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    #[test]
    fn parses_only_one_literal_argument() {
        assert_eq!(parse_literal("lang('en')").as_deref(), Some("en"));
        assert_eq!(parse_literal("fn:lang(\"fr\")").as_deref(), Some("fr"));
        assert_eq!(parse_literal("lang($requested)"), None);
        assert_eq!(parse_literal("lang('en', 'fr')"), None);
    }

    #[test]
    fn matches_the_nearest_inherited_xml_language_case_insensitively() {
        let parsed = parse_document(
            "memory:lang.xml",
            br#"<doc xml:lang="EN-us"><p><q xml:lang="fr"/></p></doc>"#,
            ParseLimits {
                max_events: 64,
                max_depth: 16,
            },
        )
        .expect("parse language source");
        let document = Document::from_parsed(parsed).expect("build language source");
        let root = document.children(document.document_node())[0];
        let p = document.children(root)[0];
        let q = document.children(p)[0];
        assert!(evaluate(&document, p, "en", &mut InvocationControl::unbounded()).unwrap());
        assert!(!evaluate(&document, q, "en", &mut InvocationControl::unbounded()).unwrap());
        assert!(language_matches("en-US", "EN"));
        assert!(!language_matches("english", "en"));
    }
}
