//! Private AR-0025 reference parser for a deliberately narrow internal DTD subset.

use std::cell::Cell;
use std::collections::HashMap;

use quick_xml::escape::resolve_predefined_entity;

#[derive(Debug, Clone, Copy)]
pub(crate) struct InternalSubsetLimits {
    pub(crate) declarations: usize,
    pub(crate) nesting_depth: usize,
    pub(crate) references: usize,
    pub(crate) replacement_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum InternalSubsetFailure {
    Malformed(String),
    Unsupported(String),
    Limit(String),
    UnknownEntity(String),
}

#[derive(Debug)]
struct ExpandedEntity {
    value: String,
    nested_references: usize,
    nesting_depth: usize,
}

#[derive(Debug)]
pub(super) struct InternalEntities {
    values: HashMap<String, ExpandedEntity>,
    references: Cell<usize>,
    replacement_bytes: Cell<usize>,
    declaration_work: usize,
    limits: InternalSubsetLimits,
}

impl InternalEntities {
    pub(super) fn resolve(&self, name: &str) -> Result<&str, InternalSubsetFailure> {
        if let Some(value) = resolve_predefined_entity(name) {
            return Ok(value);
        }
        let entity = self
            .values
            .get(name)
            .ok_or_else(|| InternalSubsetFailure::UnknownEntity(name.to_owned()))?;
        let references = self
            .references
            .get()
            .saturating_add(1)
            .saturating_add(entity.nested_references);
        if references > self.limits.references {
            return Err(InternalSubsetFailure::Limit(format!(
                "DTD entity-reference limit is {}",
                self.limits.references
            )));
        }
        let replacement_bytes = self
            .replacement_bytes
            .get()
            .saturating_add(entity.value.len());
        if replacement_bytes > self.limits.replacement_bytes {
            return Err(InternalSubsetFailure::Limit(format!(
                "DTD replacement-byte limit is {}",
                self.limits.replacement_bytes
            )));
        }
        self.references.set(references);
        self.replacement_bytes.set(replacement_bytes);
        Ok(&entity.value)
    }

    pub(super) fn reference_count(&self) -> usize {
        self.references.get()
    }

    pub(super) fn declaration_work(&self) -> usize {
        self.declaration_work
    }
}

pub(super) fn parse_internal_subset(
    doctype: &str,
    limits: InternalSubsetLimits,
) -> Result<InternalEntities, InternalSubsetFailure> {
    let (head, subset) = split_doctype(doctype)?;
    let head = trim_xml_space(head);
    let root_end = head.find(is_xml_space).unwrap_or(head.len());
    let root = head.get(..root_end).ok_or_else(|| {
        InternalSubsetFailure::Malformed("DOCTYPE is missing its root name".to_owned())
    })?;
    if !is_xml_name(root) {
        return Err(InternalSubsetFailure::Malformed(
            "DOCTYPE root name is malformed".to_owned(),
        ));
    }
    if !trim_xml_space(&head[root_end..]).is_empty() {
        return Err(InternalSubsetFailure::Unsupported(
            "external DTD identifiers remain denied".to_owned(),
        ));
    }

    let raw = parse_entity_declarations(subset, limits.declarations)?;
    let mut expanded = HashMap::with_capacity(raw.len());
    for name in raw.keys() {
        let mut active = Vec::new();
        expand_entity(name, &raw, &mut expanded, &mut active, limits)?;
    }
    let declared_references = raw.values().map(|value| count_references(value)).sum();
    if declared_references > limits.references {
        return Err(InternalSubsetFailure::Limit(format!(
            "DTD entity-reference limit is {}",
            limits.references
        )));
    }
    let declared_replacement_bytes = expanded.values().map(|entity| entity.value.len()).sum();
    if declared_replacement_bytes > limits.replacement_bytes {
        return Err(InternalSubsetFailure::Limit(format!(
            "DTD replacement-byte limit is {}",
            limits.replacement_bytes
        )));
    }
    let declaration_work = raw.len().saturating_add(declared_references);
    Ok(InternalEntities {
        values: expanded,
        references: Cell::new(declared_references),
        replacement_bytes: Cell::new(declared_replacement_bytes),
        declaration_work,
        limits,
    })
}

fn count_references(value: &str) -> usize {
    let mut remaining = value;
    let mut count = 0_usize;
    while let Some(start) = remaining.find('&') {
        count = count.saturating_add(1);
        let Some(end) = remaining[start + 1..].find(';') else {
            break;
        };
        remaining = &remaining[start + 1 + end + 1..];
    }
    count
}

fn split_doctype(doctype: &str) -> Result<(&str, &str), InternalSubsetFailure> {
    let trimmed = trim_xml_space(doctype);
    let Some(open) = trimmed.find('[') else {
        return Ok((trimmed, ""));
    };
    let close = trimmed.rfind(']').ok_or_else(|| {
        InternalSubsetFailure::Malformed("DOCTYPE internal subset is not closed".to_owned())
    })?;
    if close < open || !trim_xml_space(&trimmed[close + 1..]).is_empty() {
        return Err(InternalSubsetFailure::Malformed(
            "DOCTYPE has content after its internal subset".to_owned(),
        ));
    }
    Ok((trim_xml_space(&trimmed[..open]), &trimmed[open + 1..close]))
}

fn parse_entity_declarations(
    subset: &str,
    max_declarations: usize,
) -> Result<HashMap<String, String>, InternalSubsetFailure> {
    let mut declarations = HashMap::new();
    let mut remaining = subset;
    loop {
        remaining = remaining.trim_start_matches(is_xml_space);
        if remaining.is_empty() {
            break;
        }
        if let Some(comment) = remaining.strip_prefix("<!--") {
            let end = comment.find("-->").ok_or_else(|| {
                InternalSubsetFailure::Malformed("DTD comment is not closed".to_owned())
            })?;
            remaining = &comment[end + 3..];
            continue;
        }
        let declaration = remaining.strip_prefix("<!ENTITY").ok_or_else(|| {
            InternalSubsetFailure::Unsupported(
                "the reference DTD profile currently admits only general internal entities"
                    .to_owned(),
            )
        })?;
        if !declaration.starts_with(is_xml_space) {
            return Err(InternalSubsetFailure::Malformed(
                "ENTITY declaration requires whitespace".to_owned(),
            ));
        }
        if declarations.len() >= max_declarations {
            return Err(InternalSubsetFailure::Limit(format!(
                "DTD declaration limit is {max_declarations}"
            )));
        }
        let (name, value, rest) = parse_entity_declaration(declaration)?;
        if resolve_predefined_entity(&name).is_some() || declarations.insert(name, value).is_some()
        {
            return Err(InternalSubsetFailure::Malformed(
                "DTD entity name is duplicate or reserved".to_owned(),
            ));
        }
        remaining = rest;
    }
    Ok(declarations)
}

fn parse_entity_declaration(
    declaration: &str,
) -> Result<(String, String, &str), InternalSubsetFailure> {
    let declaration = declaration.trim_start_matches(is_xml_space);
    if declaration.starts_with('%') {
        return Err(InternalSubsetFailure::Unsupported(
            "parameter entities remain denied".to_owned(),
        ));
    }
    let name_end = declaration
        .find(is_xml_space)
        .ok_or_else(|| InternalSubsetFailure::Malformed("ENTITY name is incomplete".to_owned()))?;
    let name = &declaration[..name_end];
    if !is_xml_name(name) {
        return Err(InternalSubsetFailure::Malformed(
            "ENTITY name is malformed".to_owned(),
        ));
    }
    let value = declaration[name_end..].trim_start_matches(is_xml_space);
    let quote = value
        .chars()
        .next()
        .ok_or_else(|| InternalSubsetFailure::Malformed("ENTITY value is missing".to_owned()))?;
    if !matches!(quote, '\'' | '"') {
        return Err(InternalSubsetFailure::Unsupported(
            "external and unquoted entity declarations remain denied".to_owned(),
        ));
    }
    let quoted = &value[quote.len_utf8()..];
    let value_end = quoted
        .find(quote)
        .ok_or_else(|| InternalSubsetFailure::Malformed("ENTITY value is not closed".to_owned()))?;
    let raw_value = &quoted[..value_end];
    if raw_value.contains('<') || raw_value.contains('%') {
        return Err(InternalSubsetFailure::Unsupported(
            "markup and parameter references in entity values remain denied".to_owned(),
        ));
    }
    let after_value = quoted[value_end + quote.len_utf8()..].trim_start_matches(is_xml_space);
    let rest = after_value.strip_prefix('>').ok_or_else(|| {
        InternalSubsetFailure::Malformed("ENTITY declaration is not closed".to_owned())
    })?;
    Ok((name.to_owned(), raw_value.to_owned(), rest))
}

fn expand_entity(
    name: &str,
    raw: &HashMap<String, String>,
    expanded: &mut HashMap<String, ExpandedEntity>,
    active: &mut Vec<String>,
    limits: InternalSubsetLimits,
) -> Result<(), InternalSubsetFailure> {
    if let Some(entity) = expanded.get(name) {
        if active.len().saturating_add(entity.nesting_depth) > limits.nesting_depth {
            return Err(InternalSubsetFailure::Limit(format!(
                "DTD entity-nesting limit is {}",
                limits.nesting_depth
            )));
        }
        return Ok(());
    }
    if active.iter().any(|active_name| active_name == name) {
        return Err(InternalSubsetFailure::Malformed(format!(
            "cyclic DTD entity reference involving {name}"
        )));
    }
    if active.len() >= limits.nesting_depth {
        return Err(InternalSubsetFailure::Limit(format!(
            "DTD entity-nesting limit is {}",
            limits.nesting_depth
        )));
    }
    let value = raw
        .get(name)
        .ok_or_else(|| InternalSubsetFailure::UnknownEntity(name.to_owned()))?;
    active.push(name.to_owned());
    let mut output = String::with_capacity(value.len());
    let mut references = 0_usize;
    let mut nesting_depth = 1_usize;
    let mut cursor = 0_usize;
    while let Some(relative_start) = value[cursor..].find('&') {
        let start = cursor + relative_start;
        output.push_str(&value[cursor..start]);
        let end = value[start + 1..]
            .find(';')
            .map(|end| start + 1 + end)
            .ok_or_else(|| {
                InternalSubsetFailure::Malformed(format!(
                    "unterminated reference in DTD entity {name}"
                ))
            })?;
        let reference = &value[start + 1..end];
        references = references.saturating_add(1);
        if references > limits.references {
            return Err(InternalSubsetFailure::Limit(format!(
                "DTD entity-reference limit is {}",
                limits.references
            )));
        }
        if reference.starts_with('#') || resolve_predefined_entity(reference).is_some() {
            let escaped = &value[start..=end];
            let replacement = quick_xml::escape::unescape(escaped).map_err(|error| {
                InternalSubsetFailure::Malformed(format!(
                    "invalid character reference in DTD entity {name}: {error}"
                ))
            })?;
            output.push_str(&replacement);
        } else {
            expand_entity(reference, raw, expanded, active, limits)?;
            let nested = expanded
                .get(reference)
                .expect("successful expansion records the referenced entity");
            references = references.saturating_add(nested.nested_references);
            nesting_depth = nesting_depth.max(1_usize.saturating_add(nested.nesting_depth));
            output.push_str(&nested.value);
        }
        if output.len() > limits.replacement_bytes {
            return Err(InternalSubsetFailure::Limit(format!(
                "DTD replacement-byte limit is {}",
                limits.replacement_bytes
            )));
        }
        cursor = end + 1;
    }
    output.push_str(&value[cursor..]);
    if output.len() > limits.replacement_bytes {
        return Err(InternalSubsetFailure::Limit(format!(
            "DTD replacement-byte limit is {}",
            limits.replacement_bytes
        )));
    }
    active.pop();
    expanded.insert(
        name.to_owned(),
        ExpandedEntity {
            value: output,
            nested_references: references,
            nesting_depth,
        },
    );
    Ok(())
}

fn is_xml_name(name: &str) -> bool {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    (first == '_' || first == ':' || first.is_alphabetic())
        && characters.all(|character| {
            character == '_'
                || character == ':'
                || character == '-'
                || character == '.'
                || character.is_alphanumeric()
        })
}

const fn is_xml_space(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\r' | '\n')
}

fn trim_xml_space(value: &str) -> &str {
    value.trim_matches(is_xml_space)
}

#[cfg(test)]
mod tests {
    use super::{InternalSubsetFailure, InternalSubsetLimits, parse_internal_subset};

    const LIMITS: InternalSubsetLimits = InternalSubsetLimits {
        declarations: 8,
        nesting_depth: 4,
        references: 16,
        replacement_bytes: 128,
    };

    #[test]
    fn expands_forward_and_nested_general_entities() {
        let entities = parse_internal_subset(
            r#"root [<!ENTITY outer "left &inner; right"><!ENTITY inner "&lt;middle&gt;">]"#,
            LIMITS,
        )
        .expect("bounded internal entities should parse");

        assert_eq!(entities.resolve("outer"), Ok("left <middle> right"));
        assert_eq!(entities.reference_count(), 7);
    }

    #[test]
    fn denies_external_parameter_and_non_entity_declarations() {
        for doctype in [
            "root SYSTEM 'file:///secret'",
            "root [<!ENTITY % shared 'value'>]",
            "root [<!ATTLIST root value CDATA 'default'>]",
        ] {
            assert!(matches!(
                parse_internal_subset(doctype, LIMITS),
                Err(InternalSubsetFailure::Unsupported(_))
            ));
        }
    }

    #[test]
    fn bounds_cycles_depth_references_and_replacement_bytes() {
        assert!(matches!(
            parse_internal_subset("root [<!ENTITY a '&b;'><!ENTITY b '&a;'>]", LIMITS),
            Err(InternalSubsetFailure::Malformed(_))
        ));
        assert!(matches!(
            parse_internal_subset(
                "root [<!ENTITY a '&b;'><!ENTITY b '&c;'><!ENTITY c '&d;'><!ENTITY d '&e;'><!ENTITY e 'x'>]",
                LIMITS
            ),
            Err(InternalSubsetFailure::Limit(_))
        ));
        assert!(matches!(
            parse_internal_subset(
                "root [<!ENTITY a '0123456789'>]",
                InternalSubsetLimits {
                    replacement_bytes: 4,
                    ..LIMITS
                }
            ),
            Err(InternalSubsetFailure::Limit(_))
        ));
    }
}
