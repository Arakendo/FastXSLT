//! Private validation for the admitted `xsl:mode` declaration slice.

use std::collections::BTreeMap;

use crate::xdm::owned_tree_experiment::{Document, NodeId};
use crate::xslt::golden_semantics_experiment::{
    ModeOnNoMatch, OnNoMatchPolicy, TypedModeRequirement,
};

use super::{
    CompileFailure, ensure_no_meaningful_children, ensure_only_attributes, invalid,
    optional_attribute, parse_template_modes, unsupported,
};

pub(super) fn validate_same_precedence_mode_declaration_conflicts(
    document: &Document,
    top_level_children: &[NodeId],
) -> Result<(), CompileFailure> {
    let mut on_no_match_by_mode = BTreeMap::<String, (&str, NodeId)>::new();
    let mut visibility_by_mode = BTreeMap::<String, &str>::new();
    for element in top_level_children.iter().copied().filter(|element| {
        document.name(*element).is_some_and(|name| {
            name.namespace.as_deref() == Some(super::XSLT_NAMESPACE) && name.local == "mode"
        })
    }) {
        if !super::meaningful_children(document, element).is_empty() {
            return Err(invalid(
                "XTSE0260",
                "xsl:mode must be empty",
                document.location(element),
            ));
        }
        let Some(lexical_name) = optional_attribute(document, element, None, "name") else {
            continue;
        };
        let names = parse_template_modes(document, element, lexical_name)?;
        if names.len() != 1 || names[0].starts_with('#') {
            continue;
        }
        if let Some(visibility) = optional_attribute(document, element, None, "visibility") {
            if let Some(existing) = visibility_by_mode.get(&names[0]) {
                if *existing != visibility {
                    return Err(invalid(
                        "XTSE0545",
                        "same-precedence xsl:mode declarations specify conflicting visibility values",
                        document.location(element),
                    ));
                }
            } else {
                visibility_by_mode.insert(names[0].clone(), visibility);
            }
        }
        let Some(on_no_match) = optional_attribute(document, element, None, "on-no-match") else {
            continue;
        };
        validate_on_no_match(on_no_match, document, element)?;
        if let Some((existing, _)) = on_no_match_by_mode.get(&names[0]) {
            if *existing != on_no_match {
                return Err(invalid(
                    "XTSE0545",
                    "same-precedence xsl:mode declarations specify conflicting on-no-match values",
                    document.location(element),
                ));
            }
        } else {
            on_no_match_by_mode.insert(names[0].clone(), (on_no_match, element));
        }
    }
    Ok(())
}

fn validate_on_no_match(
    value: &str,
    document: &Document,
    element: NodeId,
) -> Result<(), CompileFailure> {
    if matches!(
        value,
        "deep-copy" | "shallow-copy" | "deep-skip" | "shallow-skip" | "text-only-copy" | "fail"
    ) {
        Ok(())
    } else {
        Err(invalid(
            "XTSE0020",
            "xsl:mode on-no-match has an invalid value",
            document.location(element),
        ))
    }
}

pub(super) struct CompiledModeDeclaration {
    pub(super) typed_requirement: Option<TypedModeRequirement>,
    pub(super) on_no_match: Option<ModeOnNoMatch>,
}

pub(super) fn validate_mode_declaration(
    document: &Document,
    element: NodeId,
    declared_version: &str,
) -> Result<CompiledModeDeclaration, CompileFailure> {
    ensure_only_attributes(
        document,
        element,
        &[
            "name",
            "on-no-match",
            "typed",
            "visibility",
            "warning-on-multiple-match",
            "warning-on-no-match",
        ],
        "xsl:mode",
    )?;
    ensure_no_meaningful_children(document, element, "xsl:mode")?;

    let lexical_name = optional_attribute(document, element, None, "name");
    let names = lexical_name
        .map(|name| parse_template_modes(document, element, name))
        .transpose()?;
    validate_visibility(
        lexical_name.is_some(),
        optional_attribute(document, element, None, "visibility"),
        document,
        element,
    )?;
    if names
        .as_ref()
        .is_some_and(|names| names.len() != 1 || names[0].starts_with('#'))
    {
        return Err(unsupported(
            "FXST1037",
            "the private xsl:mode declaration slice requires one named mode",
            document.location(element),
        ));
    }

    let warning_on_multiple_match = parse_optional_boolean(
        document,
        element,
        declared_version,
        "warning-on-multiple-match",
    )?;
    let warning_on_no_match =
        parse_optional_boolean(document, element, declared_version, "warning-on-no-match")?;
    let typed = parse_optional_boolean(document, element, declared_version, "typed")?;

    if warning_on_multiple_match == Some(true) {
        return Err(unsupported(
            "FXST1038",
            "warning-on-multiple-match requires an owned warning delivery channel",
            document.location(element),
        ));
    }
    if warning_on_no_match == Some(true) {
        return Err(unsupported(
            "FXST1038",
            "warning-on-no-match requires an owned warning delivery channel",
            document.location(element),
        ));
    }
    compile_mode_semantics(
        document,
        element,
        names.and_then(|names| names.into_iter().next()),
        typed,
    )
}

fn compile_mode_semantics(
    document: &Document,
    element: NodeId,
    mode_name: Option<String>,
    typed: Option<bool>,
) -> Result<CompiledModeDeclaration, CompileFailure> {
    if typed == Some(true) {
        let Some(name) = mode_name else {
            return Err(unsupported(
                "FXST1037",
                "typed semantics for the unnamed mode are outside the private declaration slice",
                document.location(element),
            ));
        };
        let on_no_match = compile_on_no_match(document, element, Some(name.clone()))?;
        let typed_requirement = Some(TypedModeRequirement {
            name,
            location: document.location(element).clone(),
        });
        return Ok(CompiledModeDeclaration {
            typed_requirement,
            on_no_match,
        });
    }
    let on_no_match = compile_on_no_match(document, element, mode_name.clone())?;
    if mode_name.is_none() && on_no_match.is_none() {
        return Err(unsupported(
            "FXST1037",
            "the private xsl:mode declaration slice requires a named mode or an admitted on-no-match policy",
            document.location(element),
        ));
    }
    Ok(CompiledModeDeclaration {
        typed_requirement: None,
        on_no_match,
    })
}

fn compile_on_no_match(
    document: &Document,
    element: NodeId,
    mode_name: Option<String>,
) -> Result<Option<ModeOnNoMatch>, CompileFailure> {
    let Some(value) = optional_attribute(document, element, None, "on-no-match") else {
        return Ok(None);
    };
    let policy = match value {
        "fail" => OnNoMatchPolicy::Fail,
        "shallow-copy" => OnNoMatchPolicy::ShallowCopy,
        "text-only-copy" => OnNoMatchPolicy::TextOnlyCopy,
        _ => {
            return Err(unsupported(
                "FXST1041",
                "this explicit on-no-match policy is outside the private mode declaration slice",
                document.location(element),
            ));
        }
    };
    Ok(Some(ModeOnNoMatch {
        name: mode_name,
        policy,
        location: document.location(element).clone(),
    }))
}

fn validate_visibility(
    is_named: bool,
    visibility: Option<&str>,
    document: &Document,
    element: NodeId,
) -> Result<(), CompileFailure> {
    let Some(visibility) = visibility else {
        return Ok(());
    };
    if !matches!(visibility, "public" | "private" | "final" | "abstract") {
        return Err(invalid(
            "XTSE0020",
            "xsl:mode visibility has an invalid value",
            document.location(element),
        ));
    }
    if (!is_named && matches!(visibility, "public" | "final"))
        || (is_named && visibility == "abstract")
    {
        return Err(invalid(
            "XTSE0020",
            "xsl:mode visibility is incompatible with its mode name",
            document.location(element),
        ));
    }
    Err(unsupported(
        "FXST1042",
        "mode visibility semantics are outside the private declaration slice",
        document.location(element),
    ))
}

fn parse_optional_boolean(
    document: &Document,
    element: NodeId,
    declared_version: &str,
    attribute: &str,
) -> Result<Option<bool>, CompileFailure> {
    optional_attribute(document, element, None, attribute)
        .map(|value| parse_boolean(value, declared_version, attribute, document, element))
        .transpose()
}

fn parse_boolean(
    value: &str,
    declared_version: &str,
    attribute: &str,
    document: &Document,
    element: NodeId,
) -> Result<bool, CompileFailure> {
    match value {
        "yes" => Ok(true),
        "no" => Ok(false),
        _ if declared_version == "3.0" => match value.trim() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(invalid(
                "XTSE0020",
                format!("{attribute} has an invalid XSLT 3.0 boolean value"),
                document.location(element),
            )),
        },
        _ => Err(invalid(
            "XTSE0020",
            format!("{attribute} must be 'yes' or 'no'"),
            document.location(element),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::super::{CompileCategory, compile_stylesheet};
    use crate::xdm::owned_tree_experiment::Document;
    use crate::xml::quick_xml_experiment::{ParseLimits, parse_document};
    use crate::xslt::golden_semantics_experiment::OnNoMatchPolicy;

    fn compile(
        attribute: &str,
        value: &str,
    ) -> Result<crate::xslt::golden_semantics_experiment::StylesheetProgram, super::CompileFailure>
    {
        let xml = format!(
            r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:mode name="m" {attribute}="{value}"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#
        );
        let parsed = parse_document(
            "memory:mode-declaration.xsl",
            xml.as_bytes(),
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .expect("parse mode declaration fixture");
        let document = Document::from_parsed(parsed).expect("build mode declaration fixture");
        compile_stylesheet(&document)
    }

    #[test]
    fn admits_only_warning_disabled_lexicals_without_a_warning_channel() {
        for value in ["no", "false", " 0 "] {
            compile("warning-on-multiple-match", value)
                .expect("warning-disabled declaration should be inert");
        }

        for value in ["yes", "true", " 1 "] {
            let failure = compile("warning-on-multiple-match", value)
                .expect_err("warning delivery is not yet owned");
            assert_eq!(failure.code, "FXST1038");
            assert_eq!(failure.category, CompileCategory::Unsupported);
        }

        let failure = compile("warning-on-multiple-match", "Yes")
            .expect_err("boolean lexicals are case-sensitive");
        assert_eq!(failure.code, "XTSE0020");
        assert_eq!(failure.category, CompileCategory::Invalid);
    }

    #[test]
    fn validates_all_mode_boolean_attributes_before_unsupported_semantics() {
        for attribute in ["warning-on-no-match", "typed"] {
            let failure = compile(attribute, "No")
                .expect_err("mixed-case boolean must fail before runtime semantics");
            assert_eq!(failure.code, "XTSE0020");
            assert_eq!(failure.category, CompileCategory::Invalid);
            assert!(failure.detail.contains(attribute));
        }

        compile("typed", "false").expect("typed=false is semantically inert");
        let program = compile("typed", "true")
            .expect("typed=true should retain an invocation requirement for untyped sources");
        assert_eq!(program.typed_mode_requirements.len(), 1);
        assert_eq!(program.typed_mode_requirements[0].name, "m");
        assert_eq!(
            program.typed_mode_requirements[0].location.resource,
            "memory:mode-declaration.xsl"
        );
    }

    #[test]
    fn retains_named_fail_on_no_match_policy() {
        let program = compile("on-no-match", "fail")
            .expect("named fail-on-no-match declaration should compile");
        assert_eq!(program.mode_on_no_match.len(), 1);
        assert_eq!(program.mode_on_no_match[0].name.as_deref(), Some("m"));
        assert_eq!(program.mode_on_no_match[0].policy, OnNoMatchPolicy::Fail);
    }

    #[test]
    fn retains_named_and_unnamed_text_only_copy_policies() {
        let named = compile("on-no-match", "text-only-copy")
            .expect("named text-only-copy declaration should compile");
        assert_eq!(
            named.mode_on_no_match[0].policy,
            OnNoMatchPolicy::TextOnlyCopy
        );

        let parsed = parse_document(
            "memory:unnamed-text-only-copy.xsl",
            br#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:mode on-no-match="text-only-copy"/></xsl:stylesheet>"#,
            ParseLimits {
                max_events: 32,
                max_depth: 8,
            },
        )
        .expect("parse unnamed text-only-copy fixture");
        let document = Document::from_parsed(parsed).expect("build fixture document");
        let unnamed = compile_stylesheet(&document)
            .expect("unnamed text-only-copy declaration should compile");
        assert_eq!(unnamed.mode_on_no_match[0].name, None);
        assert_eq!(
            unnamed.mode_on_no_match[0].policy,
            OnNoMatchPolicy::TextOnlyCopy
        );
    }

    #[test]
    fn rejects_invalid_named_and_unnamed_mode_visibility_combinations() {
        for (name, visibility) in [(None, "public"), (None, "final"), (Some("m"), "abstract")] {
            let name = name.map_or(String::new(), |name| format!(r#" name="{name}""#));
            let xml = format!(
                r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:mode{name} visibility="{visibility}"/><xsl:template match="/"><out/></xsl:template></xsl:stylesheet>"#
            );
            let parsed = parse_document(
                "memory:mode-visibility.xsl",
                xml.as_bytes(),
                ParseLimits {
                    max_events: 64,
                    max_depth: 8,
                },
            )
            .expect("parse mode visibility fixture");
            let document = Document::from_parsed(parsed).expect("build mode visibility fixture");
            let failure = compile_stylesheet(&document)
                .expect_err("invalid mode visibility combination should fail");
            assert_eq!(failure.code, "XTSE0020");
            assert_eq!(failure.category, CompileCategory::Invalid);
        }
    }

    #[test]
    fn rejects_conflicting_on_no_match_at_one_import_precedence() {
        let xml = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:mode name="m" on-no-match="shallow-copy"/>
            <xsl:mode name="m" on-no-match="text-only-copy"/>
            <xsl:template name="main"><out/></xsl:template>
        </xsl:stylesheet>"#;
        let parsed = parse_document(
            "memory:mode-conflict.xsl",
            xml.as_bytes(),
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .expect("parse mode conflict fixture");
        let document = Document::from_parsed(parsed).expect("build mode conflict fixture");
        let failure = compile_stylesheet(&document)
            .expect_err("same-precedence mode property conflict should fail");
        assert_eq!(failure.code, "XTSE0545");
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.location.resource, "memory:mode-conflict.xsl");
    }

    #[test]
    fn rejects_nonempty_mode_before_unrelated_top_level_features() {
        let xml = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:accumulator name="unsupported" initial-value="0"/>
            <xsl:mode name="m"><xsl:apply-templates/></xsl:mode>
        </xsl:stylesheet>"#;
        let parsed = parse_document(
            "memory:nonempty-mode.xsl",
            xml.as_bytes(),
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .expect("parse nonempty mode fixture");
        let document = Document::from_parsed(parsed).expect("build nonempty mode fixture");
        let failure = compile_stylesheet(&document)
            .expect_err("nonempty mode should fail before unrelated unsupported features");
        assert_eq!(failure.code, "XTSE0260");
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(failure.location.resource, "memory:nonempty-mode.xsl");
    }

    #[test]
    fn rejects_conflicting_visibility_at_one_import_precedence() {
        let xml = r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0">
            <xsl:mode name="m" visibility="final"/>
            <xsl:mode name="m" visibility="private"/>
            <xsl:template match="/" mode="m"><out/></xsl:template>
        </xsl:stylesheet>"#;
        let parsed = parse_document(
            "memory:mode-visibility-conflict.xsl",
            xml.as_bytes(),
            ParseLimits {
                max_events: 64,
                max_depth: 8,
            },
        )
        .expect("parse mode visibility conflict fixture");
        let document =
            Document::from_parsed(parsed).expect("build mode visibility conflict fixture");
        let failure = compile_stylesheet(&document)
            .expect_err("same-precedence mode visibility conflict should fail");
        assert_eq!(failure.code, "XTSE0545");
        assert_eq!(failure.category, CompileCategory::Invalid);
        assert_eq!(
            failure.location.resource,
            "memory:mode-visibility-conflict.xsl"
        );
    }
}
