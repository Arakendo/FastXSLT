//! Named/matched declaration routing and eligibility for the direct root fast path.

use super::{
    CompileFailure, Document, MatchPattern, MatchedTemplate, NamedTemplate, NodeId, Template,
    TemplatePriority, compile_matched_template, compile_matched_templates, compile_named_template,
    compile_template, effective_default_mode, invalid, is_xslt_element,
    normalize_named_template_name, optional_attribute, required_attribute, unsupported,
};

pub(super) fn compile_top_level_template(
    document: &Document,
    element: NodeId,
    root_template: &mut Option<Template>,
    root_template_modes: &mut Vec<String>,
    matched_templates: &mut Vec<MatchedTemplate>,
    named_templates: &mut Vec<NamedTemplate>,
) -> Result<(), CompileFailure> {
    if let Some(name) = optional_attribute(document, element, None, "name") {
        if optional_attribute(document, element, None, "match").is_none()
            && optional_attribute(document, element, None, "mode").is_some()
        {
            return Err(invalid(
                "XTSE0500",
                "xsl:template mode is permitted only when match is present",
                document.location(element),
            ));
        }
        let name = normalize_named_template_name(document, element, name)?;
        if named_templates.iter().any(|template| template.name == name) {
            return Err(invalid(
                "FXST0010",
                format!("duplicate named template: {name}"),
                document.location(element),
            ));
        }
        named_templates.push(compile_named_template(document, element, &name)?);
        if optional_attribute(document, element, None, "match").is_none() {
            return Ok(());
        }
    }

    let pattern = required_attribute(document, element, None, "match")?;
    if pattern == "/" {
        let has_priority = optional_attribute(document, element, None, "priority").is_some();
        let has_mode = optional_attribute(document, element, None, "mode").is_some()
            || effective_default_mode(document, element).is_some_and(|mode| mode != "#unnamed");
        let has_competing_default = matched_templates.iter().any(|existing| {
            existing.pattern == MatchPattern::Document && existing.modes.is_empty()
        });
        if has_priority
            || has_mode
            || root_template.is_some()
            || has_competing_default
            || requires_current_rule(document, element)
        {
            let matched_template = compile_matched_template(document, element, pattern)?;
            let competes_with_default = matched_template.modes.is_empty()
                || matched_template
                    .modes
                    .iter()
                    .any(|mode| matches!(mode.as_str(), "#default" | "#unnamed" | "#all"));
            if competes_with_default {
                if let Some(previous) = root_template.take() {
                    matched_templates.push(MatchedTemplate {
                        pattern: MatchPattern::Document,
                        import_precedence: 0,
                        apply_imports_min_precedence: 0,
                        priority: TemplatePriority::ROOT_DEFAULT,
                        modes: Vec::new(),
                        template: previous,
                    });
                    root_template_modes.clear();
                }
                matched_templates.push(matched_template);
                return Ok(());
            }
            if matched_templates.iter().any(|existing| {
                existing.pattern == matched_template.pattern
                    && existing.modes == matched_template.modes
            }) {
                return Err(unsupported(
                    "FXST1008",
                    "template priority for duplicate mode-qualified root pattern is outside the private slice",
                    document.location(element),
                ));
            }
            matched_templates.push(matched_template);
            return Ok(());
        }
        root_template_modes.clear();
        *root_template = Some(compile_template(document, element)?);
        return Ok(());
    }

    matched_templates.extend(compile_matched_templates(document, element, pattern)?);
    Ok(())
}

fn requires_current_rule(document: &Document, root_rule: NodeId) -> bool {
    // Named calls preserve the current rule. Follow only calls reachable from
    // this root, not unrelated matched rules elsewhere in the module.
    let mut pending = vec![root_rule];
    let mut followed = vec![root_rule];
    while let Some(node) = pending.pop() {
        if is_xslt_element(document, node, "apply-imports")
            || is_xslt_element(document, node, "next-match")
        {
            return true;
        }
        if is_xslt_element(document, node, "call-template") {
            let target = optional_attribute(document, node, None, "name")
                .and_then(|name| normalize_named_template_name(document, node, name).ok());
            let declaration = document
                .children(document.document_node())
                .iter()
                .flat_map(|stylesheet| document.children(*stylesheet))
                .copied()
                .find(|candidate| {
                    is_xslt_element(document, *candidate, "template")
                        && optional_attribute(document, *candidate, None, "name")
                            .and_then(|name| {
                                normalize_named_template_name(document, *candidate, name).ok()
                            })
                            .is_some_and(|name| target.as_ref() == Some(&name))
                });
            // A call into another admitted module cannot prove fast-path
            // eligibility here; the module linker resolves its target later.
            let Some(declaration) = declaration else {
                return true;
            };
            if !followed.contains(&declaration) {
                followed.push(declaration);
                pending.push(declaration);
            }
        }
        pending.extend_from_slice(document.children(node));
    }
    false
}
