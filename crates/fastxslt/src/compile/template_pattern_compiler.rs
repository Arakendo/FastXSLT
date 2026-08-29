//! Private template match-pattern normalization and priority compilation.

use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xpath::path_experiment::parse_location_path;
use crate::xslt::golden_semantics_experiment::{MatchPattern, TemplatePriority};

use super::{
    CompileFailure, invalid, is_ascii_ncname, map_path_failure, optional_attribute, unsupported,
};

pub(super) fn compile_match_pattern(
    document: &Document,
    element: NodeId,
    lexical_pattern: &str,
) -> Result<(MatchPattern, TemplatePriority), CompileFailure> {
    let pattern = match lexical_pattern {
        "comment()" => MatchPattern::Comment,
        "processing-instruction()" => MatchPattern::ProcessingInstruction,
        "node()" => MatchPattern::AnyNode,
        "//*" => MatchPattern::DescendantAnyElement,
        predicate if parse_element_attribute_value_predicate(predicate).is_some() => {
            let (element, attribute, value) = parse_element_attribute_value_predicate(predicate)
                .expect("attribute-value predicate shape was checked");
            MatchPattern::ElementWithAttributeValue {
                element: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: element.to_owned(),
                },
                attribute: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: attribute.to_owned(),
                },
                value: value.to_owned(),
            }
        }
        predicate if parse_element_attribute_predicate(predicate).is_some() => {
            let (element, attribute) =
                parse_element_attribute_predicate(predicate).expect("predicate shape was checked");
            MatchPattern::ElementWithAttribute {
                element: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: element.to_owned(),
                },
                attribute: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: attribute.to_owned(),
                },
            }
        }
        attribute if attribute.starts_with('@') && is_ascii_ncname(&attribute[1..]) => {
            MatchPattern::Attribute(crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: attribute[1..].to_owned(),
            })
        }
        "*" => MatchPattern::AnyElement,
        name if is_ascii_ncname(name) => {
            MatchPattern::Element(crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: name.to_owned(),
            })
        }
        path if path.contains('/') && !path.starts_with('/') => MatchPattern::Path(
            parse_location_path(path, document.location(element).clone())
                .map_err(map_path_failure)?,
        ),
        _ => {
            return Err(unsupported(
                "FXST1005",
                format!("unsupported template match pattern: {lexical_pattern}"),
                document.location(element),
            ));
        }
    };
    let priority = compile_template_priority(document, element, &pattern)?;
    Ok((pattern, priority))
}

fn parse_element_attribute_predicate(pattern: &str) -> Option<(&str, &str)> {
    let (element, attribute) = pattern.split_once("[@")?;
    let attribute = attribute.strip_suffix(']')?;
    (is_ascii_ncname(element) && is_ascii_ncname(attribute)).then_some((element, attribute))
}

fn parse_element_attribute_value_predicate(pattern: &str) -> Option<(&str, &str, &str)> {
    let (element, predicate) = pattern.split_once("[@")?;
    let predicate = predicate.strip_suffix(']')?;
    let (attribute, literal) = predicate.split_once('=')?;
    let value = literal.strip_prefix('\'')?.strip_suffix('\'')?;
    (is_ascii_ncname(element) && is_ascii_ncname(attribute) && !value.contains('\''))
        .then_some((element, attribute, value))
}

fn compile_template_priority(
    document: &Document,
    element: NodeId,
    pattern: &MatchPattern,
) -> Result<TemplatePriority, CompileFailure> {
    let Some(lexical) = optional_attribute(document, element, None, "priority") else {
        return Ok(match pattern {
            MatchPattern::Path(_)
            | MatchPattern::DescendantAnyElement
            | MatchPattern::ElementWithAttribute { .. }
            | MatchPattern::ElementWithAttributeValue { .. } => TemplatePriority::PATH_DEFAULT,
            MatchPattern::Element(_) | MatchPattern::Attribute(_) => {
                TemplatePriority::EXACT_NAME_DEFAULT
            }
            MatchPattern::Comment
            | MatchPattern::ProcessingInstruction
            | MatchPattern::AnyNode
            | MatchPattern::AnyElement => TemplatePriority::NODE_TEST_DEFAULT,
        });
    };
    let lexical = lexical.trim();
    if let Ok(value) = lexical.parse::<i32>() {
        return Ok(TemplatePriority::explicit_integer(value));
    }
    if is_decimal_lexical(lexical) {
        return Err(unsupported(
            "FXST1025",
            "the private explicit template-priority slice permits bounded signed integers only",
            document.location(element),
        ));
    }
    Err(invalid(
        "FXST0030",
        format!("invalid template priority: {lexical}"),
        document.location(element),
    ))
}

fn is_decimal_lexical(value: &str) -> bool {
    let unsigned = value
        .strip_prefix('+')
        .or_else(|| value.strip_prefix('-'))
        .unwrap_or(value);
    let Some((whole, fractional)) = unsigned.split_once('.') else {
        return !unsigned.is_empty() && unsigned.bytes().all(|byte| byte.is_ascii_digit());
    };
    (!whole.is_empty() || !fractional.is_empty())
        && whole.bytes().all(|byte| byte.is_ascii_digit())
        && fractional.bytes().all(|byte| byte.is_ascii_digit())
}
