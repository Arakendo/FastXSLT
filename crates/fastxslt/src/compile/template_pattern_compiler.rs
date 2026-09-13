//! Private template match-pattern normalization and priority compilation.

use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xpath::path_experiment::{PathStep, parse_location_path};
use crate::xslt::golden_semantics_experiment::{
    ChildPresenceTest, MatchNodeTest, MatchPattern, MatchStringPredicate, NamedSiblingBoundary,
    TemplatePriority,
};

use super::match_sequence_predicate_compiler::parse as parse_match_sequence_predicates;
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
        atomic if parse_atomic_integer_threshold(atomic).is_some() => {
            MatchPattern::AtomicIntegerGreaterOrEqual(
                parse_atomic_integer_threshold(atomic).expect("atomic threshold shape was checked"),
            )
        }
        "/" | "document-node()" => MatchPattern::Document,
        "/*" => MatchPattern::DocumentElement(None),
        "comment()" => MatchPattern::Comment,
        "text()" => MatchPattern::Text,
        "processing-instruction()" => MatchPattern::ProcessingInstruction,
        named if parse_named_processing_instruction_pattern(named).is_some() => {
            MatchPattern::ProcessingInstructionNamed(
                parse_named_processing_instruction_pattern(named)
                    .expect("named processing-instruction pattern shape was checked")
                    .to_owned(),
            )
        }
        "node()" => MatchPattern::AnyNode,
        "//*" => MatchPattern::DescendantAnyElement,
        lexical if parse_document_element_test(lexical).is_some() => {
            compile_document_element_pattern(document, element, lexical)
        }
        predicate if parse_element_two_attribute_values(predicate).is_some() => {
            compile_element_two_attribute_values_pattern(predicate)
        }
        predicate if parse_qualified_element_attribute_value(predicate).is_some() => {
            compile_qualified_element_attribute_value(document, element, predicate)?
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
        predicate if parse_any_element_attribute_value_predicate(predicate).is_some() => {
            compile_any_element_attribute_value_pattern(predicate)
        }
        predicate if parse_any_element_attribute_number_predicate(predicate).is_some() => {
            compile_any_element_attribute_number_pattern(predicate)
        }
        predicate if parse_any_element_number_predicate(predicate).is_some() => {
            MatchPattern::AnyElementNumberEquals(
                parse_any_element_number_predicate(predicate)
                    .expect("wildcard element numeric predicate shape was checked"),
            )
        }
        predicate if parse_node_string_predicate(predicate).is_some() => {
            compile_node_string_value_pattern(document, element, predicate)
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
        positional if parse_named_sibling_attribute_value(positional).is_some() => {
            let (element_name, position, attribute, value, attribute_filters_position) =
                parse_named_sibling_attribute_value(positional)
                    .expect("positional attribute pattern shape was checked");
            MatchPattern::ElementAtNamedSiblingWithAttributeValue {
                element: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: effective_xpath_default_namespace(document, element)
                        .map(str::to_owned),
                    local: element_name.to_owned(),
                },
                position,
                attribute: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: None,
                    local: attribute.to_owned(),
                },
                value: value.to_owned(),
                attribute_filters_position,
            }
        }
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
        sequential if parse_match_sequence_predicates(sequential).is_some() => {
            let (element_name, predicates) = parse_match_sequence_predicates(sequential)
                .expect("sequential match-predicate shape was checked");
            MatchPattern::ElementWithSequentialPredicates {
                element: crate::xml::quick_xml_experiment::ExpandedName {
                    namespace: effective_xpath_default_namespace(document, element)
                        .map(str::to_owned),
                    local: element_name.to_owned(),
                },
                predicates,
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
        predicate if parse_any_element_attribute_predicate(predicate).is_some() => {
            MatchPattern::AnyElementWithAttribute(crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: parse_any_element_attribute_predicate(predicate)
                    .expect("wildcard attribute predicate shape was checked")
                    .to_owned(),
            })
        }
        predicate
            if predicate.contains('[')
                && !predicate.contains('/')
                && effective_xpath_default_namespace(document, element).is_none()
                && parse_location_path(predicate, document.location(element).clone())
                    .is_ok_and(|path| path.has_positional_child_string_predicate()) =>
        {
            MatchPattern::Path(
                parse_location_path(predicate, document.location(element).clone())
                    .expect("single-step predicate path shape was checked"),
            )
        }
        predicate if parse_any_node_attribute_predicate(predicate).is_some() => {
            MatchPattern::AnyElementWithAttribute(crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: parse_any_node_attribute_predicate(predicate)
                    .expect("node-test attribute predicate shape was checked")
                    .to_owned(),
            })
        }
        predicate if parse_any_element_qualified_attribute_predicate(predicate).is_some() => {
            compile_any_element_qualified_attribute(document, element, predicate)?
        }
        predicate if parse_attribute_name_predicate(predicate).is_some() => {
            MatchPattern::AttributeNameEquals(
                parse_attribute_name_predicate(predicate)
                    .expect("attribute name predicate shape was checked"),
            )
        }
        "@*" | "attribute()" | "attribute::*" | "attribute::node()" => MatchPattern::AnyAttribute,
        attribute if parse_attribute_namespace_wildcard(attribute).is_some() => {
            let prefix = parse_attribute_namespace_wildcard(attribute)
                .expect("attribute namespace wildcard shape was checked");
            MatchPattern::AttributeNamespace(
                namespace_for_prefix(document, element, prefix)
                    .ok_or_else(|| {
                        invalid(
                            "FXST0031",
                            format!("unbound prefix in attribute match pattern: {prefix}"),
                            document.location(element),
                        )
                    })?
                    .to_owned(),
            )
        }
        attribute if parse_qualified_attribute_test(attribute).is_some() => {
            let (prefix, local) = parse_qualified_attribute_test(attribute)
                .expect("qualified attribute test shape was checked");
            let namespace = namespace_for_prefix(document, element, prefix).ok_or_else(|| {
                invalid(
                    "FXST0031",
                    format!("unbound prefix in attribute match pattern: {prefix}"),
                    document.location(element),
                )
            })?;
            MatchPattern::Attribute(crate::xml::quick_xml_experiment::ExpandedName {
                namespace: Some(namespace.to_owned()),
                local: local.to_owned(),
            })
        }
        attribute if attribute.starts_with('@') && is_ascii_ncname(&attribute[1..]) => {
            MatchPattern::Attribute(crate::xml::quick_xml_experiment::ExpandedName {
                namespace: None,
                local: attribute[1..].to_owned(),
            })
        }
        "*" | "element()" | "child::*" => MatchPattern::AnyElement,
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
        path if parse_leading_descendant_static_true_pattern(path).is_some()
            && effective_xpath_default_namespace(document, element).is_none() =>
        {
            MatchPattern::Path(
                parse_location_path(
                    parse_leading_descendant_static_true_pattern(path)
                        .expect("static-true descendant pattern shape was checked"),
                    document.location(element).clone(),
                )
                .expect("normalized static-true descendant name is a location path"),
            )
        }
        path if path.starts_with("//")
            && effective_xpath_default_namespace(document, element).is_none()
            && parse_location_path(path, document.location(element).clone())
                .is_ok_and(|path| path.is_bounded_descendant_named_match_path()) =>
        {
            MatchPattern::Path(
                parse_location_path(path, document.location(element).clone())
                    .expect("bounded descendant named match path shape was checked"),
            )
        }
        path if path.contains('/') && !path.starts_with("//") => {
            let path = parse_location_path(path, document.location(element).clone())
                .map_err(map_path_failure)?;
            if path.has_non_simple_position_predicate() {
                return Err(unsupported(
                    "FXST1005",
                    "non-simple position predicates in multi-step match patterns are outside the private pattern slice",
                    document.location(element),
                ));
            }
            MatchPattern::Path(path)
        }
        invalid_pattern if invalid_match_pattern_reason(invalid_pattern).is_some() => {
            return Err(invalid(
                "FXST1005",
                invalid_match_pattern_reason(invalid_pattern)
                    .expect("invalid match-pattern reason was checked"),
                document.location(element),
            ));
        }
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

fn invalid_match_pattern_reason(pattern: &str) -> Option<String> {
    if pattern.starts_with('$') {
        return Some(
            "a variable reference cannot replace a match-pattern location path".to_owned(),
        );
    }
    let arguments = pattern.strip_prefix("key(")?.strip_suffix(')')?;
    let (name, value) = arguments.split_once(',')?;
    (!is_xpath_string_literal(name.trim()) || !is_xpath_string_literal(value.trim())).then(|| {
        "xsl:key match-pattern arguments must be string literals in this pattern grammar".to_owned()
    })
}

fn is_xpath_string_literal(value: &str) -> bool {
    ['\'', '"'].into_iter().any(|delimiter| {
        value
            .strip_prefix(delimiter)
            .and_then(|inner| inner.strip_suffix(delimiter))
            .is_some_and(|inner| !inner.contains(delimiter))
    })
}

fn parse_leading_descendant_static_true_pattern(pattern: &str) -> Option<&str> {
    let path = pattern.strip_suffix("[true()]")?;
    let name = path.strip_prefix("//")?;
    is_ascii_ncname(name).then_some(path)
}

fn parse_named_processing_instruction_pattern(pattern: &str) -> Option<&str> {
    let argument = pattern
        .strip_prefix("processing-instruction(")?
        .strip_suffix(')')?;
    for delimiter in ['\'', '"'] {
        let target = argument
            .strip_prefix(delimiter)
            .and_then(|value| value.strip_suffix(delimiter));
        if let Some(target) = target.filter(|value| *value == "*" || is_ascii_ncname(value)) {
            return Some(target);
        }
    }
    None
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
        _ => parse_static_named_sibling_position(predicate)?,
    };
    Some((element, boundary))
}

fn parse_named_sibling_attribute_value(pattern: &str) -> Option<(&str, usize, &str, &str, bool)> {
    let (element, predicates) = pattern.split_once('[')?;
    if !is_ascii_ncname(element) {
        return None;
    }

    if let Some((position, attribute)) = predicates.split_once("][@") {
        let position = parse_exact_position(position)?;
        let (attribute, value) = parse_attribute_literal(attribute.strip_suffix(']')?)?;
        return Some((element, position, attribute, value, false));
    }
    if let Some((position, attribute)) = predicates.strip_suffix(']')?.split_once(" and @") {
        let position = parse_exact_position(position)?;
        let (attribute, value) = parse_attribute_literal(attribute)?;
        return Some((element, position, attribute, value, false));
    }
    if let Some(predicates) = predicates.strip_prefix('@')
        && let Some((attribute, position)) = predicates.split_once("][")
    {
        let (attribute, value) = parse_attribute_literal(attribute)?;
        let position = parse_exact_position(position.strip_suffix(']')?)?;
        return Some((element, position, attribute, value, true));
    }
    None
}

fn parse_exact_position(predicate: &str) -> Option<usize> {
    let lexical = predicate
        .trim()
        .strip_prefix("position()=")
        .unwrap_or(predicate.trim());
    lexical.parse().ok().filter(|position| *position > 0)
}

fn parse_static_named_sibling_position(predicate: &str) -> Option<NamedSiblingBoundary> {
    let relation = predicate.strip_prefix("position()")?.trim_start();
    let (exact, operand) = if let Some(operand) = relation.strip_prefix('=') {
        (true, operand)
    } else if let Some(operand) = relation.strip_prefix('<') {
        (false, operand)
    } else {
        return None;
    };
    operand
        .trim()
        .parse()
        .ok()
        .filter(|position| *position > 0)
        .map(|position| {
            if exact {
                NamedSiblingBoundary::Exact(position)
            } else {
                NamedSiblingBoundary::Before(position)
            }
        })
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

fn parse_any_element_attribute_predicate(pattern: &str) -> Option<&str> {
    let attribute = pattern.strip_prefix("*[@")?.strip_suffix(']')?;
    is_ascii_ncname(attribute).then_some(attribute)
}

fn parse_any_node_attribute_predicate(pattern: &str) -> Option<&str> {
    let attribute = pattern.strip_prefix("node()[@")?.strip_suffix(']')?;
    is_ascii_ncname(attribute).then_some(attribute)
}

fn parse_attribute_namespace_wildcard(pattern: &str) -> Option<&str> {
    let prefix = pattern.strip_prefix('@')?.strip_suffix(":*")?;
    is_ascii_ncname(prefix).then_some(prefix)
}

fn parse_attribute_name_predicate(pattern: &str) -> Option<String> {
    let predicate = pattern.strip_prefix("@*[")?.strip_suffix(']')?;
    parse_context_name_literal(predicate)
}

fn parse_context_name_literal(predicate: &str) -> Option<String> {
    let (function, literal) = predicate.split_once('=')?;
    if !matches!(function.trim(), "name()" | "name(.)") {
        return None;
    }
    let literal = literal.trim();
    ['\'', '"'].into_iter().find_map(|delimiter| {
        literal
            .strip_prefix(delimiter)
            .and_then(|value| value.strip_suffix(delimiter))
            .filter(|value| !value.contains(delimiter) && is_ascii_ncname(value))
            .map(str::to_owned)
    })
}

fn parse_qualified_attribute_test(pattern: &str) -> Option<(&str, &str)> {
    let (prefix, local) = pattern.strip_prefix('@')?.split_once(':')?;
    (is_ascii_ncname(prefix) && is_ascii_ncname(local)).then_some((prefix, local))
}

fn parse_any_element_qualified_attribute_predicate(pattern: &str) -> Option<(&str, &str)> {
    let attribute = pattern.strip_prefix("*[@")?.strip_suffix(']')?;
    let (prefix, local) = attribute.split_once(':')?;
    (is_ascii_ncname(prefix) && is_ascii_ncname(local)).then_some((prefix, local))
}

fn compile_any_element_qualified_attribute(
    document: &Document,
    element: NodeId,
    pattern: &str,
) -> Result<MatchPattern, CompileFailure> {
    let (prefix, local) = parse_any_element_qualified_attribute_predicate(pattern)
        .expect("qualified attribute predicate shape was checked");
    let namespace = namespace_for_prefix(document, element, prefix).ok_or_else(|| {
        invalid(
            "FXST0031",
            format!("unbound prefix in attribute match predicate: {prefix}"),
            document.location(element),
        )
    })?;
    Ok(MatchPattern::AnyElementWithAttribute(
        crate::xml::quick_xml_experiment::ExpandedName {
            namespace: Some(namespace.to_owned()),
            local: local.to_owned(),
        },
    ))
}

fn parse_any_element_attribute_value_predicate(pattern: &str) -> Option<(&str, &str)> {
    let predicate = pattern.strip_prefix("*[@")?.strip_suffix(']')?;
    parse_attribute_literal(predicate)
}

fn compile_any_element_attribute_value_pattern(pattern: &str) -> MatchPattern {
    let (attribute, value) = parse_any_element_attribute_value_predicate(pattern)
        .expect("wildcard element attribute-value predicate shape was checked");
    MatchPattern::AnyElementWithAttributeValue {
        attribute: crate::xml::quick_xml_experiment::ExpandedName {
            namespace: None,
            local: attribute.to_owned(),
        },
        value: value.to_owned(),
    }
}

fn parse_any_element_number_predicate(pattern: &str) -> Option<i32> {
    pattern
        .strip_prefix("*[.=")?
        .strip_suffix(']')?
        .parse()
        .ok()
}

fn parse_any_element_attribute_number_predicate(pattern: &str) -> Option<(&str, i32)> {
    let predicate = pattern.strip_prefix("*[@")?.strip_suffix(']')?;
    let (attribute, value) = predicate.split_once('=')?;
    is_ascii_ncname(attribute).then_some(())?;
    Some((attribute, value.parse().ok()?))
}

fn compile_any_element_attribute_number_pattern(pattern: &str) -> MatchPattern {
    let (attribute, value) = parse_any_element_attribute_number_predicate(pattern)
        .expect("wildcard element attribute-number predicate shape was checked");
    MatchPattern::AnyElementWithAttributeNumberEquals {
        attribute: crate::xml::quick_xml_experiment::ExpandedName {
            namespace: None,
            local: attribute.to_owned(),
        },
        value,
    }
}

fn parse_node_string_predicate(pattern: &str) -> Option<(&str, MatchStringPredicate)> {
    let (node_test, predicate) = pattern.split_once('[')?;
    let predicate = predicate.strip_suffix(']')?.trim();
    let predicate = if let Some(value) = parse_context_contains_literal(predicate) {
        MatchStringPredicate::Contains(value)
    } else if let Some((left, right)) = predicate.split_once(" or ") {
        MatchStringPredicate::EqualsEither(
            parse_context_string_literal(left, "=")?,
            parse_context_string_literal(right, "=")?,
        )
    } else if let Some(operand) = predicate
        .strip_prefix("not(")
        .and_then(|value| value.strip_suffix(')'))
    {
        MatchStringPredicate::NotEquals(parse_context_string_literal(operand, "=")?)
    } else if predicate.starts_with(".!=") {
        MatchStringPredicate::NotEquals(parse_context_string_literal(predicate, "!=")?)
    } else {
        MatchStringPredicate::Equals(parse_context_string_literal(predicate, "=")?)
    };
    let supported_node_test = is_ascii_ncname(node_test)
        || matches!(
            node_test,
            "text()" | "comment()" | "processing-instruction()"
        )
        || parse_named_processing_instruction_pattern(node_test).is_some();
    supported_node_test.then_some((node_test, predicate))
}

fn parse_context_contains_literal(expression: &str) -> Option<String> {
    let arguments = expression.strip_prefix("contains(")?.strip_suffix(')')?;
    let (context, literal) = arguments.split_once(',')?;
    if context.trim() != "." {
        return None;
    }
    let literal = literal.trim();
    ['\'', '"'].into_iter().find_map(|delimiter| {
        literal
            .strip_prefix(delimiter)
            .and_then(|value| value.strip_suffix(delimiter))
            .filter(|value| !value.contains(delimiter))
            .map(str::to_owned)
    })
}

fn parse_context_string_literal(expression: &str, operator: &str) -> Option<String> {
    let literal = expression
        .trim()
        .strip_prefix('.')?
        .strip_prefix(operator)?
        .trim();
    ['\'', '"'].into_iter().find_map(|delimiter| {
        literal
            .strip_prefix(delimiter)
            .and_then(|value| value.strip_suffix(delimiter))
            .filter(|value| !value.contains(delimiter))
            .map(str::to_owned)
    })
}

fn compile_node_string_value_pattern(
    document: &Document,
    element: NodeId,
    pattern: &str,
) -> MatchPattern {
    let (node_test, predicate) = parse_node_string_predicate(pattern)
        .expect("node string-value predicate shape was checked");
    let node_test = match node_test {
        "text()" => MatchNodeTest::Text,
        "comment()" => MatchNodeTest::Comment,
        "processing-instruction()" => MatchNodeTest::ProcessingInstruction(None),
        named if parse_named_processing_instruction_pattern(named).is_some() => {
            MatchNodeTest::ProcessingInstruction(Some(
                parse_named_processing_instruction_pattern(named)
                    .expect("named processing-instruction shape was checked")
                    .to_owned(),
            ))
        }
        name => MatchNodeTest::Element(crate::xml::quick_xml_experiment::ExpandedName {
            namespace: effective_xpath_default_namespace(document, element).map(str::to_owned),
            local: name.to_owned(),
        }),
    };
    MatchPattern::NodeStringPredicate {
        node_test,
        predicate,
    }
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

fn parse_qualified_element_attribute_value(pattern: &str) -> Option<(&str, &str, &str, &str)> {
    let (element, predicate) = pattern.split_once("[@")?;
    let (prefix, local) = parse_qualified_element_test(element)?;
    let (attribute, value) = parse_attribute_literal(predicate.strip_suffix(']')?)?;
    is_ascii_ncname(attribute).then_some((prefix, local, attribute, value))
}

fn compile_qualified_element_attribute_value(
    document: &Document,
    element: NodeId,
    pattern: &str,
) -> Result<MatchPattern, CompileFailure> {
    let (prefix, local, attribute, value) = parse_qualified_element_attribute_value(pattern)
        .expect("qualified element attribute-value pattern shape was checked");
    let namespace = namespace_for_prefix(document, element, prefix).ok_or_else(|| {
        invalid(
            "FXST0031",
            format!("unbound prefix in template match pattern: {prefix}"),
            document.location(element),
        )
    })?;
    Ok(MatchPattern::ElementWithAttributeValue {
        element: crate::xml::quick_xml_experiment::ExpandedName {
            namespace: Some(namespace.to_owned()),
            local: local.to_owned(),
        },
        attribute: crate::xml::quick_xml_experiment::ExpandedName {
            namespace: None,
            local: attribute.to_owned(),
        },
        value: value.to_owned(),
    })
}

fn parse_element_two_attribute_values(pattern: &str) -> Option<(&str, &str, &str, &str, &str)> {
    let (element, predicates) = pattern.split_once("[@")?;
    if !is_ascii_ncname(element) {
        return None;
    }
    let (first, second) = if let Some((first, second)) = predicates.split_once("][@") {
        (first, second.strip_suffix(']')?)
    } else {
        predicates.strip_suffix(']')?.split_once(" and @")?
    };
    let (first_attribute, first_value) = parse_attribute_literal(first)?;
    let (second_attribute, second_value) = parse_attribute_literal(second)?;
    Some((
        element,
        first_attribute,
        first_value,
        second_attribute,
        second_value,
    ))
}

fn parse_attribute_literal(expression: &str) -> Option<(&str, &str)> {
    let (attribute, literal) = expression.split_once('=')?;
    let value = literal.strip_prefix('\'')?.strip_suffix('\'')?;
    (is_ascii_ncname(attribute) && !value.contains('\'')).then_some((attribute, value))
}

fn compile_element_two_attribute_values_pattern(pattern: &str) -> MatchPattern {
    let (element, first_attribute, first_value, second_attribute, second_value) =
        parse_element_two_attribute_values(pattern)
            .expect("two-attribute predicate shape was checked");
    let name = |local: &str| crate::xml::quick_xml_experiment::ExpandedName {
        namespace: None,
        local: local.to_owned(),
    };
    MatchPattern::ElementWithTwoAttributeValues {
        element: name(element),
        first_attribute: name(first_attribute),
        first_value: first_value.to_owned(),
        second_attribute: name(second_attribute),
        second_value: second_value.to_owned(),
    }
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
            | MatchPattern::AtomicIntegerGreaterOrEqual(_)
            | MatchPattern::QualifiedElementPathAlternatives(_)
            | MatchPattern::DescendantAnyElement
            | MatchPattern::ElementWithAttribute { .. }
            | MatchPattern::AnyElementWithAttribute(_)
            | MatchPattern::AnyElementWithAttributeValue { .. }
            | MatchPattern::AnyElementNumberEquals(_)
            | MatchPattern::AnyElementWithAttributeNumberEquals { .. }
            | MatchPattern::AttributeNameEquals(_)
            | MatchPattern::NodeStringPredicate { .. }
            | MatchPattern::ElementWithAttributeValue { .. }
            | MatchPattern::ElementWithTwoAttributeValues { .. }
            | MatchPattern::ElementWithChild { .. }
            | MatchPattern::AnyElementWithAttributeVariable { .. }
            | MatchPattern::VariableFilteredElementPath(_)
            | MatchPattern::ElementWithSameNamedChild
            | MatchPattern::ElementWithSameNamedParent
            | MatchPattern::ElementWithSameNamedParentAtPosition(_)
            | MatchPattern::ElementAtNamedSiblingBoundary { .. }
            | MatchPattern::ElementAtNamedSiblingWithAttributeValue { .. }
            | MatchPattern::ElementWithSequentialPredicates { .. }
            | MatchPattern::UnionAlternatives(_) => TemplatePriority::PATH_DEFAULT,
            MatchPattern::Document | MatchPattern::DocumentElement(None) => {
                TemplatePriority::ROOT_DEFAULT
            }
            MatchPattern::DocumentElement(Some(_))
            | MatchPattern::Element(_)
            | MatchPattern::Attribute(_)
            | MatchPattern::ProcessingInstructionNamed(_) => TemplatePriority::EXACT_NAME_DEFAULT,
            MatchPattern::AttributeNamespace(_) => TemplatePriority::NAMESPACE_WILDCARD_DEFAULT,
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

fn parse_atomic_integer_threshold(pattern: &str) -> Option<i64> {
    pattern
        .strip_prefix(".[. ge ")?
        .strip_suffix(']')?
        .trim()
        .parse()
        .ok()
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
