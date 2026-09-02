//! Private template match-pattern normalization and priority compilation.

use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xpath::path_experiment::{PathStep, parse_location_path};
use crate::xslt::golden_semantics_experiment::{
    ChildPresenceTest, MatchPattern, NamedSiblingBoundary, TemplatePriority,
};

use super::variable_filtered_path_compiler::parse as parse_variable_filtered_path;
use super::{
    CompileFailure, effective_xpath_default_namespace, invalid, is_ascii_ncname, map_path_failure,
    optional_attribute, unsupported,
};

pub(super) fn compile_match_pattern(
    document: &Document,
    element: NodeId,
    lexical_pattern: &str,
) -> Result<(MatchPattern, TemplatePriority), CompileFailure> {
    let pattern = match lexical_pattern {
        "/" | "document-node()" => MatchPattern::Document,
        "/*" => MatchPattern::DocumentElement(None),
        "comment()" => MatchPattern::Comment,
        "text()" => MatchPattern::Text,
        "processing-instruction()" => MatchPattern::ProcessingInstruction,
        "node()" => MatchPattern::AnyNode,
        "//*" => MatchPattern::DescendantAnyElement,
        lexical if parse_document_element_test(lexical).is_some() => {
            compile_document_element_pattern(document, element, lexical)
        }
        predicate if parse_element_attribute_value_predicate(predicate).is_some() => {
            compile_element_attribute_value_pattern(predicate)
        }
        predicate if parse_element_child_presence_predicate(predicate).is_some() => {
            compile_element_child_presence_pattern(predicate)
        }
        predicate if parse_any_element_attribute_variable_predicate(predicate).is_some() => {
            compile_any_element_attribute_variable_pattern(predicate)
        }
        alternatives if is_homogeneous_qualified_path_union(alternatives) => {
            MatchPattern::QualifiedElementPathAlternatives(
                alternatives
                    .split('|')
                    .map(|path| compile_qualified_element_path(document, element, path.trim()))
                    .collect::<Result<Vec<_>, _>>()?,
            )
        }
        path if parse_variable_filtered_path(path).is_some() => {
            MatchPattern::VariableFilteredElementPath(
                parse_variable_filtered_path(path)
                    .expect("variable-filtered path shape was checked"),
            )
        }
        "*[*[name()=name(current())]]" | "*[some $x in child::* satisfies name($x) = name(.)]" => {
            MatchPattern::ElementWithSameNamedChild
        }
        "*[name()=name(current())]/*" => MatchPattern::ElementWithSameNamedParent,
        "*[name()=name(current())][2]/*" => MatchPattern::ElementWithSameNamedParentAtPosition(2),
        positional if parse_named_sibling_boundary(positional).is_some() => {
            let (element_name, boundary) =
                parse_named_sibling_boundary(positional).expect("positional shape was checked");
            MatchPattern::ElementAtNamedSiblingBoundary {
                element: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: effective_xpath_default_namespace(document, element)
                        .map(str::to_owned),
                    local: element_name.to_owned(),
                },
                boundary,
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
        "@*" | "attribute()" => MatchPattern::AnyAttribute,
        attribute if attribute.starts_with('@') && is_ascii_ncname(&attribute[1..]) => {
            MatchPattern::Attribute(crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: attribute[1..].to_owned(),
            })
        }
        "*" | "element()" => MatchPattern::AnyElement,
        name if is_ascii_ncname(name) => {
            MatchPattern::Element(crate::xml::quick_xml_experiment::ExpandedName {
                namespace: effective_xpath_default_namespace(document, element).map(str::to_owned),
                local: name.to_owned(),
            })
        }
        local_wildcard if parse_local_name_wildcard(local_wildcard).is_some() => {
            MatchPattern::ElementLocal(
                parse_local_name_wildcard(local_wildcard)
                    .expect("local-name wildcard shape was checked")
                    .to_owned(),
            )
        }
        qualified if parse_qualified_element_test(qualified).is_some() => {
            let (prefix, local) =
                parse_qualified_element_test(qualified).expect("qualified shape was checked");
            let namespace = namespace_for_prefix(document, element, prefix).ok_or_else(|| {
                invalid(
                    "FXST0031",
                    format!("unbound prefix in template match pattern: {prefix}"),
                    document.location(element),
                )
            })?;
            if local == "*" {
                MatchPattern::ElementNamespace(namespace.to_owned())
            } else {
                MatchPattern::Element(crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: Some(namespace.to_owned()),
                    local: local.to_owned(),
                })
            }
        }
        path if path.contains('/')
            && !path.starts_with("//")
            && effective_xpath_default_namespace(document, element).is_some() =>
        {
            return Err(unsupported(
                "FXST1027",
                "xpath-default-namespace on multi-step match paths is outside the private expanded-name path slice",
                document.location(element),
            ));
        }
        path if path.contains('/') && !path.starts_with("//") => MatchPattern::Path(
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

pub(super) fn is_homogeneous_qualified_path_union(pattern: &str) -> bool {
    let lengths = pattern
        .split('|')
        .map(str::trim)
        .map(|path| {
            let steps = path.split('/').map(str::trim).collect::<Vec<_>>();
            steps
                .iter()
                .all(|step| {
                    is_ascii_ncname(step)
                        || parse_qualified_element_test(step).is_some_and(|(_, local)| local != "*")
                })
                .then_some(steps.len())
        })
        .collect::<Option<Vec<_>>>();
    let Some(lengths) = lengths else {
        return false;
    };
    lengths.len() > 1
        && (lengths.iter().all(|length| *length == 1) || lengths.iter().all(|length| *length > 1))
}

#[derive(Debug, PartialEq, Eq)]
enum UnionMatchDomain {
    Element(Option<String>, String),
    Text,
}

pub(super) fn alternatives_are_pairwise_disjoint(
    patterns: &[(MatchPattern, TemplatePriority)],
) -> bool {
    let mut domains = Vec::with_capacity(patterns.len());
    for (pattern, _) in patterns {
        let domain = match pattern {
            MatchPattern::Element(name) => {
                UnionMatchDomain::Element(name.namespace.clone(), name.local.clone())
            }
            MatchPattern::ElementWithChild { element, .. } => {
                UnionMatchDomain::Element(element.namespace.clone(), element.local.clone())
            }
            MatchPattern::Text => UnionMatchDomain::Text,
            MatchPattern::Path(path) => match path.steps.last() {
                Some(PathStep::ChildNamed(local)) => UnionMatchDomain::Element(None, local.clone()),
                Some(PathStep::ChildText) => UnionMatchDomain::Text,
                _ => return false,
            },
            _ => return false,
        };
        if domains.contains(&domain) {
            return false;
        }
        domains.push(domain);
    }
    true
}

fn compile_qualified_element_path(
    document: &Document,
    element: NodeId,
    path: &str,
) -> Result<Vec<crate::xml::quick_xml_experiment::ExpandedName>, CompileFailure> {
    path.split('/')
        .map(str::trim)
        .map(|step| {
            if is_ascii_ncname(step) {
                return Ok(crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: effective_xpath_default_namespace(document, element)
                        .map(str::to_owned),
                    local: step.to_owned(),
                });
            }
            let (prefix, local) = parse_qualified_element_test(step)
                .filter(|(_, local)| *local != "*")
                .expect("homogeneous qualified path shape was checked");
            let namespace = namespace_for_prefix(document, element, prefix).ok_or_else(|| {
                invalid(
                    "FXST0031",
                    format!("unbound prefix in union match pattern: {prefix}"),
                    document.location(element),
                )
            })?;
            Ok(crate::xml::quick_xml_experiment::ExpandedName {
                namespace: Some(namespace.to_owned()),
                local: local.to_owned(),
            })
        })
        .collect()
}

fn parse_local_name_wildcard(pattern: &str) -> Option<&str> {
    let local = pattern.strip_prefix("*:")?;
    is_ascii_ncname(local).then_some(local)
}

fn parse_named_sibling_boundary(pattern: &str) -> Option<(&str, NamedSiblingBoundary)> {
    let (element, predicate) = pattern.split_once('[')?;
    let predicate = predicate.strip_suffix(']')?.trim();
    if !is_ascii_ncname(element) {
        return None;
    }
    let boundary = match predicate {
        "position()=last()" => NamedSiblingBoundary::Last,
        "position()<last()" => NamedSiblingBoundary::BeforeLast,
        _ => return None,
    };
    Some((element, boundary))
}

fn parse_document_element_test(pattern: &str) -> Option<&str> {
    let element = pattern
        .strip_prefix("document-node(element(")?
        .strip_suffix("))")?;
    (element == "*" || is_ascii_ncname(element)).then_some(element)
}

fn compile_document_element_pattern(
    document: &Document,
    element: NodeId,
    pattern: &str,
) -> MatchPattern {
    let element_test =
        parse_document_element_test(pattern).expect("document element-test shape was checked");
    MatchPattern::DocumentElement((element_test != "*").then(|| {
        crate::xml::quick_xml_experiment::ExpandedName {
            namespace: effective_xpath_default_namespace(document, element).map(str::to_owned),
            local: element_test.to_owned(),
        }
    }))
}

fn compile_element_attribute_value_pattern(pattern: &str) -> MatchPattern {
    let (element, attribute, value) = parse_element_attribute_value_predicate(pattern)
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

fn compile_element_child_presence_pattern(pattern: &str) -> MatchPattern {
    let (element, child) = parse_element_child_presence_predicate(pattern)
        .expect("child-presence predicate shape was checked");
    MatchPattern::ElementWithChild {
        element: crate::xml::quick_xml_experiment::ExpandedName {
            namespace: None,
            local: element.to_owned(),
        },
        child: match child {
            "text()" => ChildPresenceTest::Text,
            name => ChildPresenceTest::Element(crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: name.to_owned(),
            }),
        },
    }
}

fn parse_qualified_element_test(pattern: &str) -> Option<(&str, &str)> {
    let (prefix, local) = pattern.split_once(':')?;
    (is_ascii_ncname(prefix) && (local == "*" || is_ascii_ncname(local))).then_some((prefix, local))
}

fn namespace_for_prefix<'a>(
    document: &'a Document,
    element: NodeId,
    prefix: &str,
) -> Option<&'a str> {
    let mut current = Some(element);
    while let Some(node) = current {
        if let Some(binding) = document
            .namespace_declarations(node)
            .iter()
            .find(|binding| binding.prefix.as_deref() == Some(prefix))
        {
            return Some(binding.namespace.as_str());
        }
        current = document.parent(node);
    }
    None
}

fn parse_element_attribute_predicate(pattern: &str) -> Option<(&str, &str)> {
    let (element, attribute) = pattern.split_once("[@")?;
    let attribute = attribute.strip_suffix(']')?;
    (is_ascii_ncname(element) && is_ascii_ncname(attribute)).then_some((element, attribute))
}

fn parse_element_child_presence_predicate(pattern: &str) -> Option<(&str, &str)> {
    let (element, child) = pattern.split_once('[')?;
    let child = child.strip_suffix(']')?.trim();
    (is_ascii_ncname(element.trim()) && (child == "text()" || is_ascii_ncname(child)))
        .then_some((element.trim(), child))
}

fn parse_element_attribute_value_predicate(pattern: &str) -> Option<(&str, &str, &str)> {
    let (element, predicate) = pattern.split_once("[@")?;
    let predicate = predicate.strip_suffix(']')?;
    let (attribute, literal) = predicate.split_once('=')?;
    let value = literal.strip_prefix('\'')?.strip_suffix('\'')?;
    (is_ascii_ncname(element) && is_ascii_ncname(attribute) && !value.contains('\''))
        .then_some((element, attribute, value))
}

fn parse_any_element_attribute_variable_predicate(pattern: &str) -> Option<(&str, &str)> {
    let predicate = pattern.strip_prefix("*[@")?.strip_suffix(']')?;
    let (attribute, variable) = predicate.split_once("=$")?;
    (is_ascii_ncname(attribute) && is_ascii_ncname(variable)).then_some((attribute, variable))
}

fn compile_any_element_attribute_variable_pattern(pattern: &str) -> MatchPattern {
    let (attribute, variable) = parse_any_element_attribute_variable_predicate(pattern)
        .expect("variable predicate shape was checked");
    MatchPattern::AnyElementWithAttributeVariable {
        attribute: crate::xml::quick_xml_experiment::ExpandedName {
            namespace: None,
            local: attribute.to_owned(),
        },
        variable: variable.to_owned(),
    }
}

fn compile_template_priority(
    document: &Document,
    element: NodeId,
    pattern: &MatchPattern,
) -> Result<TemplatePriority, CompileFailure> {
    let Some(lexical) = optional_attribute(document, element, None, "priority") else {
        return Ok(match pattern {
            MatchPattern::QualifiedElementPathAlternatives(alternatives)
                if alternatives.iter().all(|path| path.len() == 1) =>
            {
                TemplatePriority::EXACT_NAME_DEFAULT
            }
            MatchPattern::Path(_)
            | MatchPattern::QualifiedElementPathAlternatives(_)
            | MatchPattern::DescendantAnyElement
            | MatchPattern::ElementWithAttribute { .. }
            | MatchPattern::ElementWithAttributeValue { .. }
            | MatchPattern::ElementWithChild { .. }
            | MatchPattern::AnyElementWithAttributeVariable { .. }
            | MatchPattern::VariableFilteredElementPath(_)
            | MatchPattern::ElementWithSameNamedChild
            | MatchPattern::ElementWithSameNamedParent
            | MatchPattern::ElementWithSameNamedParentAtPosition(_)
            | MatchPattern::ElementAtNamedSiblingBoundary { .. }
            | MatchPattern::UnionAlternatives(_) => TemplatePriority::PATH_DEFAULT,
            MatchPattern::Document | MatchPattern::DocumentElement(None) => {
                TemplatePriority::ROOT_DEFAULT
            }
            MatchPattern::DocumentElement(Some(_))
            | MatchPattern::Element(_)
            | MatchPattern::Attribute(_) => TemplatePriority::EXACT_NAME_DEFAULT,
            MatchPattern::ElementLocal(_) | MatchPattern::ElementNamespace(_) => {
                TemplatePriority::NAMESPACE_WILDCARD_DEFAULT
            }
            MatchPattern::Comment
            | MatchPattern::Text
            | MatchPattern::ProcessingInstruction
            | MatchPattern::AnyNode
            | MatchPattern::AnyElement
            | MatchPattern::AnyAttribute => TemplatePriority::NODE_TEST_DEFAULT,
        });
    };
    let lexical = lexical.trim();
    if let Ok(value) = lexical.parse::<i32>() {
        return Ok(TemplatePriority::explicit_integer(value));
    }
    if is_decimal_lexical(lexical) {
        return parse_bounded_decimal_millionths(lexical)
            .map(TemplatePriority::explicit_millionths)
            .ok_or_else(|| {
                unsupported(
                    "FXST1025",
                    "explicit template priority exceeds the private six-place fixed-point domain",
                    document.location(element),
                )
            });
    }
    Err(invalid(
        "FXST0030",
        format!("invalid template priority: {lexical}"),
        document.location(element),
    ))
}

fn parse_bounded_decimal_millionths(value: &str) -> Option<i64> {
    let (negative, unsigned) = if let Some(unsigned) = value.strip_prefix('-') {
        (true, unsigned)
    } else {
        (false, value.strip_prefix('+').unwrap_or(value))
    };
    let (whole, fractional) = unsigned.split_once('.')?;
    if fractional.len() > 6 {
        return None;
    }
    let whole = if whole.is_empty() {
        0
    } else {
        whole.parse::<i64>().ok()?
    };
    let mut fraction = if fractional.is_empty() {
        0
    } else {
        fractional.parse::<i64>().ok()?
    };
    for _ in fractional.len()..6 {
        fraction = fraction.checked_mul(10)?;
    }
    let magnitude = whole.checked_mul(1_000_000)?.checked_add(fraction)?;
    if negative {
        magnitude.checked_neg()
    } else {
        Some(magnitude)
    }
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
