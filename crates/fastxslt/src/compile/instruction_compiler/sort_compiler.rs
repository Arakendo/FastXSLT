//! Private typed compilation of sort keys and static collation controls.

use super::{
    CompileFailure, Document, ExpandedName, LocationPath, NodeId, PathStep, SortCaseOrder,
    SortDataType, SortKey, SortLanguage, SortOrder, SortSelect, SourceLocation, Xslt10ConcatPart,
    boolean_expression_compiler, compile_sort_path, effective_xpath_default_namespace,
    ensure_no_meaningful_children, ensure_only_attributes, invalid, is_ascii_ncname,
    is_xslt_element, map_path_failure, meaningful_children, optional_attribute,
    parse_location_path, split_top_level_union, unsupported, uses_xslt10_compatibility,
    value_expression_compiler, xpath_string_literal,
};

pub(in crate::compile) fn compile_sort_keys(
    document: &Document,
    parent: NodeId,
) -> Result<(Vec<SortKey>, Vec<NodeId>), CompileFailure> {
    let xslt10_numeric_conversion = uses_xslt10_compatibility(document, parent);
    let children = meaningful_children(document, parent);
    let mut sorts = Vec::new();
    let mut sort_nodes = Vec::new();
    let mut saw_body = false;
    for child in children {
        if !is_xslt_element(document, child, "sort") {
            saw_body = true;
            continue;
        }
        if saw_body {
            return Err(invalid(
                "XTSE0010",
                "xsl:sort must precede every other sequence-constructor child",
                document.location(child),
            ));
        }
        ensure_only_attributes(
            document,
            child,
            &["select", "data-type", "order", "lang", "case-order"],
            "xsl:sort",
        )?;
        ensure_no_meaningful_children(document, child, "xsl:sort")?;
        let location = document.location(child).clone();
        let select = compile_sort_select(
            document,
            child,
            optional_attribute(document, child, None, "select").unwrap_or("."),
            xslt10_numeric_conversion,
            &location,
        )?;
        let data_type = compile_sort_data_type(
            optional_attribute(document, child, None, "data-type"),
            &location,
        )?;
        let (case_order, language) = compile_sort_collation_metadata(
            document,
            child,
            &data_type,
            xslt10_numeric_conversion,
            &location,
        )?;
        let order = compile_sort_order(
            optional_attribute(document, child, None, "order"),
            &location,
        )?;
        sorts.push(SortKey {
            select,
            data_type,
            xslt10_numeric_conversion,
            order,
            case_order,
            language,
            location,
        });
        sort_nodes.push(child);
    }
    Ok((sorts, sort_nodes))
}

fn compile_sort_select(
    document: &Document,
    sort: NodeId,
    select: &str,
    xslt10_compatibility: bool,
    location: &SourceLocation,
) -> Result<SortSelect, CompileFailure> {
    if xslt10_compatibility
        && let Some(expression) =
            value_expression_compiler::compile_xslt10_concat(document, sort, select, location)?
    {
        return Ok(SortSelect::Xslt10Concat(Box::new(expression)));
    }
    if xslt10_compatibility && select.trim_start().starts_with("key(") {
        return Ok(SortSelect::Xslt10KeyLookup(Box::new(
            value_expression_compiler::compile_xslt10_literal_key_lookup(
                document, sort, select, location,
            )?,
        )));
    }
    if xslt10_compatibility
        && let Some((path, delimiter)) =
            compile_xslt10_path_substring_after(document, sort, select, location)?
    {
        return Ok(SortSelect::Xslt10PathSubstringAfter { path, delimiter });
    }
    if xslt10_compatibility
        && let Some(expression) = value_expression_compiler::compile_xslt10_path_substring(
            document, sort, select, location,
        )?
    {
        return Ok(SortSelect::Xslt10PathSubstring(Box::new(expression)));
    }
    if xslt10_compatibility
        && let Some(expression) = value_expression_compiler::compile_xslt10_binary_numeric(
            document, sort, select, location,
        )
    {
        return Ok(SortSelect::Xslt10BinaryNumeric(Box::new(expression)));
    }
    if let Some(alternatives) = split_top_level_union(select) {
        let alternatives = alternatives
            .into_iter()
            .map(str::trim)
            .map(|alternative| {
                if alternative.is_empty() {
                    return Err(invalid(
                        "XPST0003",
                        "xsl:sort path union contains an empty alternative",
                        location,
                    ));
                }
                compile_sort_path(document, sort, alternative, location)
            })
            .collect::<Result<Vec<_>, CompileFailure>>()?;
        return Ok(SortSelect::PathUnion(alternatives));
    }
    if let Some(value) = xpath_string_literal(select.trim()) {
        return Ok(SortSelect::Literal(value.to_owned()));
    }
    if let Some(variable) = select.trim().strip_prefix('$')
        && is_ascii_ncname(variable)
    {
        return Ok(SortSelect::Variable(variable.to_owned()));
    }
    if xslt10_compatibility
        && let Some((path, variable, explicit_position_comparison)) =
            value_expression_compiler::compile_xslt10_variable_position_path(
                document, sort, select, location,
            )?
    {
        return Ok(SortSelect::Xslt10VariablePositionPath {
            path,
            variable,
            explicit_position_comparison,
        });
    }
    if xslt10_compatibility && let Some(variable) = parse_xslt10_child_name_variable_sort(select) {
        return Ok(SortSelect::Xslt10ChildNameEqualsVariable { variable });
    }
    if xslt10_compatibility
        && let Some((path, literal, equal)) =
            boolean_expression_compiler::parse_xslt10_source_path_comparison(select)
        && let Some(literal) = xpath_string_literal(literal)
    {
        return Ok(SortSelect::Xslt10PathStringLiteralComparison {
            path: compile_sort_path(document, sort, path, location)?,
            literal: literal.to_owned(),
            equal,
        });
    }
    compile_sort_scalar_or_path(document, sort, select, location)
}

fn compile_sort_scalar_or_path(
    document: &Document,
    sort: NodeId,
    select: &str,
    location: &SourceLocation,
) -> Result<SortSelect, CompileFailure> {
    match select.trim() {
        "position()" => Ok(SortSelect::ContextPosition),
        "last()" => Ok(SortSelect::ContextSize),
        "name()" | "name(.)" => Ok(SortSelect::ContextNodeName),
        "local-name()" | "local-name(.)" => Ok(SortSelect::ContextNodeLocalName),
        "string-length()" | "string-length(.)" => Ok(SortSelect::ContextStringLength),
        _ => {
            if let Some(path) =
                compile_sort_function_path(document, sort, select, "count", location)?
            {
                return Ok(SortSelect::CountPath(path));
            }
            if let Some(path) =
                compile_sort_function_path(document, sort, select, "number", location)?
            {
                return Ok(SortSelect::NumberPath(path));
            }
            Ok(SortSelect::LocationPath(compile_sort_path(
                document, sort, select, location,
            )?))
        }
    }
}

fn compile_xslt10_path_substring_after(
    document: &Document,
    context: NodeId,
    expression: &str,
    location: &SourceLocation,
) -> Result<Option<(LocationPath, String)>, CompileFailure> {
    let expression = expression.trim();
    let Some(arguments) = expression
        .strip_prefix("substring-after(")
        .and_then(|arguments| arguments.strip_suffix(')'))
    else {
        return Ok(None);
    };
    let Some(arguments) = crate::xpath::static_string_experiment::split_arguments(arguments, 2)
    else {
        return Ok(None);
    };
    let [path, delimiter] = arguments.as_slice() else {
        return Ok(None);
    };
    let Some(delimiter) = xpath_string_literal(delimiter.trim()) else {
        return Ok(None);
    };
    Ok(Some((
        compile_sort_path(document, context, path.trim(), location)?,
        delimiter.to_owned(),
    )))
}

fn parse_xslt10_child_name_variable_sort(expression: &str) -> Option<String> {
    let (path, predicate) = expression.trim().split_once('[')?;
    if !matches!(path.trim(), "./*" | "*") {
        return None;
    }
    let predicate = predicate.strip_suffix(']')?;
    let (left, right) = predicate.split_once('=')?;
    if right.contains('=') {
        return None;
    }
    let (left, right) = (left.trim(), right.trim());
    let variable = if left == "name(.)" {
        right.strip_prefix('$')?
    } else if right == "name(.)" {
        left.strip_prefix('$')?
    } else {
        return None;
    };
    is_ascii_ncname(variable).then(|| variable.to_owned())
}

fn compile_sort_data_type(
    value: Option<&str>,
    location: &SourceLocation,
) -> Result<SortDataType, CompileFailure> {
    match value.map(fold_static_sort_control_avt) {
        None | Some(Some("text")) => Ok(SortDataType::Text),
        Some(Some("number")) => Ok(SortDataType::Number),
        Some(Some(value)) if is_prefixed_qname(value) => Err(unsupported(
            "FXST1044",
            format!("extension xsl:sort data-type is outside the admitted slice: {value}"),
            location,
        )),
        Some(Some(value)) => Err(invalid(
            "XTDE0030",
            format!("invalid xsl:sort data-type: {value}"),
            location,
        )),
        Some(None) => compile_sort_control_variable(value.expect("dynamic sort control exists"))
            .map(SortDataType::Variable)
            .ok_or_else(|| {
                unsupported(
                    "FXST1044",
                    "dynamic xsl:sort data-type is outside the admitted variable-only slice",
                    location,
                )
            }),
    }
}

fn is_prefixed_qname(value: &str) -> bool {
    let Some((prefix, local)) = value.split_once(':') else {
        return false;
    };
    !local.contains(':') && is_ascii_ncname(prefix) && is_ascii_ncname(local)
}

fn compile_sort_order(
    value: Option<&str>,
    location: &SourceLocation,
) -> Result<SortOrder, CompileFailure> {
    match value.map(fold_static_sort_control_avt) {
        None | Some(Some("ascending")) => Ok(SortOrder::Ascending),
        Some(Some("descending")) => Ok(SortOrder::Descending),
        Some(Some(value)) => Err(invalid(
            "XTDE0030",
            format!("invalid xsl:sort order: {value}"),
            location,
        )),
        Some(None) => compile_sort_control_variable(value.expect("dynamic sort control exists"))
            .map(SortOrder::Variable)
            .ok_or_else(|| {
                unsupported(
                    "FXST1044",
                    "dynamic xsl:sort order is outside the admitted variable-only slice",
                    location,
                )
            }),
    }
}

fn fold_static_sort_control_avt(value: &str) -> Option<&str> {
    if !value.contains(['{', '}']) {
        return Some(value);
    }
    let expression = value.strip_prefix('{')?.strip_suffix('}')?.trim();
    xpath_string_literal(expression)
}

fn compile_sort_control_variable(value: &str) -> Option<String> {
    let expression = value.strip_prefix('{')?.strip_suffix('}')?.trim();
    let variable = expression.strip_prefix('$')?;
    is_ascii_ncname(variable).then(|| variable.to_owned())
}

fn compile_sort_collation_metadata(
    document: &Document,
    sort: NodeId,
    data_type: &SortDataType,
    xslt10_compatibility: bool,
    location: &SourceLocation,
) -> Result<(SortCaseOrder, SortLanguage), CompileFailure> {
    let lang = optional_attribute(document, sort, None, "lang");
    let case_order = optional_attribute(document, sort, None, "case-order");
    if !xslt10_compatibility
        && matches!(data_type, SortDataType::Text | SortDataType::Variable(_))
        && (lang.is_some() || case_order.is_some())
    {
        return Err(unsupported(
            "FXST1063",
            "language-sensitive text collation is outside the admitted xsl:sort slice",
            location,
        ));
    }
    if matches!(data_type, SortDataType::Number)
        && lang
            .into_iter()
            .chain(case_order)
            .any(|value| value.contains(['{', '}']))
    {
        return Err(unsupported(
            "FXST1064",
            "dynamic ignored xsl:sort collation attributes are outside the admitted numeric slice",
            location,
        ));
    }
    if matches!(data_type, SortDataType::Number) {
        return Ok((SortCaseOrder::Default, SortLanguage::Codepoint));
    }
    let language = compile_sort_language(document, sort, lang, xslt10_compatibility, location)?;
    let case_order = compile_sort_case_order(case_order, xslt10_compatibility, location)?;
    Ok((case_order, language))
}

fn compile_sort_language(
    document: &Document,
    sort: NodeId,
    lang: Option<&str>,
    xslt10_compatibility: bool,
    location: &SourceLocation,
) -> Result<SortLanguage, CompileFailure> {
    let language = if let Some(lang) = lang {
        if let Some(lang) = fold_static_sort_control_avt(lang) {
            if !is_xslt10_language_tag(lang) {
                return Err(invalid(
                    "XTDE0030",
                    format!("invalid xsl:sort language hint: {lang}"),
                    location,
                ));
            }
            let primary = lang.split('-').next().unwrap_or_default();
            match primary.to_ascii_lowercase().as_str() {
                "en" => SortLanguage::English,
                "es" => SortLanguage::Spanish,
                "fr" => SortLanguage::French,
                "sv" => SortLanguage::Swedish,
                "tr" => SortLanguage::Turkish,
                _ => {
                    return Err(unsupported(
                        "FXST1063",
                        format!("unsupported xsl:sort language hint: {lang}"),
                        location,
                    ));
                }
            }
        } else {
            let expression = lang
                .strip_prefix('{')
                .and_then(|value| value.strip_suffix('}'))
                .map(str::trim);
            let expression = match expression {
                Some(expression) => value_expression_compiler::compile_xslt10_concat(
                    document, sort, expression, location,
                )?,
                None => None,
            };
            let expression = expression.filter(|expression| {
                expression.parts.iter().all(|part| {
                    matches!(
                        part,
                        Xslt10ConcatPart::Literal(_) | Xslt10ConcatPart::Variable(_)
                    )
                })
            });
            SortLanguage::Xslt10Concat(Box::new(expression.ok_or_else(|| {
                unsupported(
                    "FXST1063",
                    "dynamic xsl:sort lang is outside the admitted compatibility concat slice",
                    location,
                )
            })?))
        }
    } else if xslt10_compatibility {
        SortLanguage::English
    } else {
        SortLanguage::Codepoint
    };
    Ok(language)
}

fn compile_sort_case_order(
    case_order: Option<&str>,
    xslt10_compatibility: bool,
    location: &SourceLocation,
) -> Result<SortCaseOrder, CompileFailure> {
    match case_order.map(fold_static_sort_control_avt) {
        // The XSLT 1.0 compatibility collation used by the admitted OASIS
        // processors orders lowercase before uppercase when primary and
        // secondary weights are equal. Keep the modern no-hint path on the
        // codepoint/default behavior rather than leaking this compatibility
        // choice into later language versions.
        None if xslt10_compatibility => Ok(SortCaseOrder::LowerFirst),
        None => Ok(SortCaseOrder::Default),
        Some(Some("upper-first")) => Ok(SortCaseOrder::UpperFirst),
        Some(Some("lower-first")) => Ok(SortCaseOrder::LowerFirst),
        Some(Some(value)) => Err(invalid(
            "XTDE0030",
            format!("invalid xsl:sort case-order: {value}"),
            location,
        )),
        Some(None) => {
            compile_sort_control_variable(case_order.expect("dynamic case-order control exists"))
                .map(SortCaseOrder::Variable)
                .ok_or_else(|| {
                    unsupported(
                        "FXST1063",
                        "dynamic xsl:sort case-order is outside the admitted variable-only slice",
                        location,
                    )
                })
        }
    }
}

fn is_xslt10_language_tag(value: &str) -> bool {
    let mut parts = value.split('-');
    let Some(primary) = parts.next() else {
        return false;
    };
    !primary.is_empty()
        && primary.len() <= 8
        && primary.bytes().all(|byte| byte.is_ascii_alphabetic())
        && parts.all(|part| {
            !part.is_empty()
                && part.len() <= 8
                && part.bytes().all(|byte| byte.is_ascii_alphanumeric())
        })
}

fn compile_sort_function_path(
    document: &Document,
    element: NodeId,
    expression: &str,
    function: &str,
    location: &SourceLocation,
) -> Result<Option<LocationPath>, CompileFailure> {
    let expression = expression.trim();
    let Some(argument) = expression
        .strip_prefix(function)
        .and_then(|value| value.strip_prefix('('))
        .and_then(|value| value.strip_suffix(')'))
        .map(str::trim)
    else {
        return Ok(None);
    };
    if argument.is_empty() {
        return Ok(None);
    }
    let mut path = parse_location_path(argument, location.clone()).map_err(map_path_failure)?;
    if let Some(namespace) = effective_xpath_default_namespace(document, element) {
        for step in &mut path.steps {
            if let PathStep::ChildNamed(local) = step {
                *step = PathStep::ChildExpandedName(ExpandedName {
                    namespace: Some(namespace.to_owned()),
                    local: local.clone(),
                });
            }
        }
    }
    Ok(Some(path))
}

#[cfg(test)]
mod tests {
    use super::super::super::{CompileCategory, compile_stylesheet};
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};

    fn compile(select: &str, version: &str) -> Result<(), super::CompileFailure> {
        let select = select.replace('"', "&quot;");
        let xml = format!(
            r#"<xsl:stylesheet version="{version}" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:p="urn:p"><xsl:template match="/"><xsl:for-each select="r/item"><xsl:sort select="{select}"/><xsl:value-of select="."/></xsl:for-each></xsl:template></xsl:stylesheet>"#
        );
        let source = Document::from_parsed(
            parse_document(
                "urn:sort-routing:style",
                xml.as_bytes(),
                ParseLimits {
                    max_events: 64,
                    max_depth: 8,
                },
            )
            .unwrap(),
        )
        .unwrap();
        compile_stylesheet(&source).map(|_| ())
    }

    #[test]
    fn sort_quoted_colons_do_not_fabricate_invalid_namespace_syntax() {
        for version in ["1.0", "3.0"] {
            for expression in [
                "concat('Re: ', .)",
                "concat(\"Re: \" , .)",
                "concat('a:b', 'c:d')",
            ] {
                if version == "1.0" {
                    compile(expression, version).unwrap();
                    continue;
                }
                let failure = compile(expression, version).unwrap_err();
                assert_eq!(
                    failure.category,
                    CompileCategory::Unsupported,
                    "{expression}: {failure:?}"
                );
                assert_eq!(failure.code, "FXXP1001");
                assert_eq!(failure.location.resource, "urn:sort-routing:style");
            }
            compile("p:item", version).unwrap();
            let unbound = compile("q:item", version).unwrap_err();
            assert_eq!(unbound.category, CompileCategory::Invalid);
            assert_eq!(unbound.code, "XPST0081");
            let malformed = compile("p:*:*", version).unwrap_err();
            assert_eq!(malformed.category, CompileCategory::Invalid);
            assert_eq!(malformed.code, "XPST0003");
        }
    }

    #[test]
    fn concat_substring_sort_retains_bounded_legacy_grammar() {
        compile("concat(substring(name,1.5,2.5),':')", "1.0").unwrap();
        compile("concat(substring(name,1,string-length(name)),':')", "1.0").unwrap();
        compile(
            "concat(substring(substring-after(name,':'),1,2),':')",
            "1.0",
        )
        .unwrap();
        for (expression, version) in [
            ("concat(substring(name,1,2),':')", "3.0"),
            ("concat(substring(name,1,$n),':')", "1.0"),
            (
                "concat(substring(substring-after(name,':'),1,unknown()),':')",
                "1.0",
            ),
        ] {
            let failure = compile(expression, version).unwrap_err();
            assert_eq!(failure.category, CompileCategory::Unsupported);
            assert_eq!(failure.location.resource, "urn:sort-routing:style");
        }
    }
}
