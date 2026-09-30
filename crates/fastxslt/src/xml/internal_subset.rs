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

    let (raw, declaration_count) = parse_declarations(subset, limits)?;
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
    let declaration_work = declaration_count.saturating_add(declared_references);
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

fn parse_declarations(
    subset: &str,
    limits: InternalSubsetLimits,
) -> Result<(HashMap<String, String>, usize), InternalSubsetFailure> {
    let mut declarations = HashMap::new();
    let mut declaration_count = 0_usize;
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
        if declaration_count >= limits.declarations {
            return Err(InternalSubsetFailure::Limit(format!(
                "DTD declaration limit is {}",
                limits.declarations
            )));
        }
        declaration_count = declaration_count.saturating_add(1);
        if let Some(declaration) = remaining.strip_prefix("<!ENTITY") {
            if !declaration.starts_with(is_xml_space) {
                return Err(InternalSubsetFailure::Malformed(
                    "ENTITY declaration requires whitespace".to_owned(),
                ));
            }
            let (name, value, rest) = parse_entity_declaration(declaration)?;
            if resolve_predefined_entity(&name).is_some()
                || declarations.insert(name, value).is_some()
            {
                return Err(InternalSubsetFailure::Malformed(
                    "DTD entity name is duplicate or reserved".to_owned(),
                ));
            }
            remaining = rest;
        } else if let Some(declaration) = remaining.strip_prefix("<!ELEMENT") {
            remaining = parse_element_declaration(declaration, limits.nesting_depth)?;
        } else {
            return Err(InternalSubsetFailure::Unsupported(
                "the reference DTD profile currently admits only ELEMENT and general internal ENTITY declarations"
                    .to_owned(),
            ));
        }
    }
    Ok((declarations, declaration_count))
}

fn parse_element_declaration(
    declaration: &str,
    max_nesting_depth: usize,
) -> Result<&str, InternalSubsetFailure> {
    if !declaration.starts_with(is_xml_space) {
        return Err(InternalSubsetFailure::Malformed(
            "ELEMENT declaration requires whitespace".to_owned(),
        ));
    }
    let declaration = declaration.trim_start_matches(is_xml_space);
    let name_end = declaration.find(is_xml_space).ok_or_else(|| {
        InternalSubsetFailure::Malformed("ELEMENT name or content model is incomplete".to_owned())
    })?;
    if !is_xml_name(&declaration[..name_end]) {
        return Err(InternalSubsetFailure::Malformed(
            "ELEMENT name is malformed".to_owned(),
        ));
    }
    let content = declaration[name_end..].trim_start_matches(is_xml_space);
    let mut parser = ContentSpecParser::new(content, max_nesting_depth);
    parser.parse()?;
    parser.skip_space();
    parser.expect('>', "ELEMENT declaration is not closed")?;
    Ok(&content[parser.position..])
}

struct ContentSpecParser<'a> {
    input: &'a str,
    position: usize,
    max_nesting_depth: usize,
}

impl<'a> ContentSpecParser<'a> {
    const fn new(input: &'a str, max_nesting_depth: usize) -> Self {
        Self {
            input,
            position: 0,
            max_nesting_depth,
        }
    }

    fn parse(&mut self) -> Result<(), InternalSubsetFailure> {
        if self.consume_word("EMPTY") || self.consume_word("ANY") {
            return Ok(());
        }
        self.parse_group(1)
    }

    fn parse_group(&mut self, depth: usize) -> Result<(), InternalSubsetFailure> {
        if depth > self.max_nesting_depth {
            return Err(InternalSubsetFailure::Limit(format!(
                "DTD content-model nesting limit is {}",
                self.max_nesting_depth
            )));
        }
        self.expect('(', "ELEMENT content model must start with '('")?;
        self.skip_space();
        if self.consume_literal("#PCDATA") {
            return self.parse_mixed_content();
        }

        self.parse_content_particle(depth)?;
        self.skip_space();
        if let Some(separator @ ('|' | ',')) = self.peek() {
            while self.peek() == Some(separator) {
                self.position += separator.len_utf8();
                self.skip_space();
                self.parse_content_particle(depth)?;
                self.skip_space();
                if matches!(self.peek(), Some('|' | ',')) && self.peek() != Some(separator) {
                    return Err(InternalSubsetFailure::Malformed(
                        "ELEMENT content model cannot mix choice and sequence separators in one group"
                            .to_owned(),
                    ));
                }
            }
        }
        self.expect(')', "ELEMENT child content model is not closed")?;
        self.consume_quantifier();
        Ok(())
    }

    fn parse_mixed_content(&mut self) -> Result<(), InternalSubsetFailure> {
        self.skip_space();
        let mut named = false;
        while self.peek() == Some('|') {
            named = true;
            self.position += 1;
            self.skip_space();
            self.parse_name()?;
            self.skip_space();
        }
        self.expect(')', "ELEMENT mixed content model is not closed")?;
        if named {
            self.expect('*', "ELEMENT mixed content with names requires '*'")?;
        } else if self.peek() == Some('*') {
            self.position += 1;
        }
        Ok(())
    }

    fn parse_content_particle(&mut self, depth: usize) -> Result<(), InternalSubsetFailure> {
        if self.peek() == Some('(') {
            self.parse_group(depth.saturating_add(1))?;
        } else {
            self.parse_name()?;
            self.consume_quantifier();
        }
        Ok(())
    }

    fn parse_name(&mut self) -> Result<(), InternalSubsetFailure> {
        let start = self.position;
        while let Some(character) = self.peek() {
            if matches!(
                character,
                ' ' | '\t' | '\r' | '\n' | '|' | ',' | ')' | '?' | '*' | '+'
            ) {
                break;
            }
            self.position += character.len_utf8();
        }
        if start == self.position || !is_xml_name(&self.input[start..self.position]) {
            return Err(InternalSubsetFailure::Malformed(
                "ELEMENT content model contains a malformed name".to_owned(),
            ));
        }
        Ok(())
    }

    fn consume_quantifier(&mut self) {
        if matches!(self.peek(), Some('?' | '*' | '+')) {
            self.position += 1;
        }
    }

    fn consume_word(&mut self, word: &str) -> bool {
        let start = self.position;
        if !self.consume_literal(word) {
            return false;
        }
        let has_name_continuation = self.peek().is_some_and(|character| {
            character == '_'
                || character == ':'
                || character == '-'
                || character == '.'
                || character.is_alphanumeric()
        });
        if has_name_continuation {
            self.position = start;
            false
        } else {
            true
        }
    }

    fn consume_literal(&mut self, literal: &str) -> bool {
        if self.input[self.position..].starts_with(literal) {
            self.position += literal.len();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: char, message: &str) -> Result<(), InternalSubsetFailure> {
        if self.peek() == Some(expected) {
            self.position += expected.len_utf8();
            Ok(())
        } else {
            Err(InternalSubsetFailure::Malformed(message.to_owned()))
        }
    }

    fn skip_space(&mut self) {
        while self.peek().is_some_and(is_xml_space) {
            self.position += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }
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
    fn admits_well_formed_non_validating_element_declarations() {
        for content_model in [
            "EMPTY",
            "ANY",
            "(#PCDATA)",
            "(#PCDATA | child | other)*",
            "(child, other?)",
            "((left | right)+, tail*)",
        ] {
            parse_internal_subset(&format!("root [<!ELEMENT root {content_model}>]"), LIMITS)
                .expect("well-formed ELEMENT declarations should be syntactically admitted");
        }
    }

    #[test]
    fn rejects_malformed_or_over_nested_element_declarations() {
        for doctype in [
            "root [<!ELEMENT root>]",
            "root [<!ELEMENT root (#PCDATA | child)>]",
            "root [<!ELEMENT root (child other)>]",
            "root [<!ELEMENT root (child | other, tail)>]",
        ] {
            assert!(matches!(
                parse_internal_subset(doctype, LIMITS),
                Err(InternalSubsetFailure::Malformed(_))
            ));
        }
        assert!(matches!(
            parse_internal_subset(
                "root [<!ELEMENT root (((((child, other), tail), last), end), done)>]",
                LIMITS
            ),
            Err(InternalSubsetFailure::Limit(_))
        ));
    }

    #[test]
    fn counts_element_and_entity_declarations_together() {
        assert!(matches!(
            parse_internal_subset(
                "root [<!ELEMENT root ANY><!ENTITY one '1'>]",
                InternalSubsetLimits {
                    declarations: 1,
                    ..LIMITS
                }
            ),
            Err(InternalSubsetFailure::Limit(_))
        ));

        let entities = parse_internal_subset("root [<!ELEMENT root ANY><!ENTITY one '1'>]", LIMITS)
            .expect("both bounded declarations should parse");
        assert_eq!(entities.declaration_work(), 2);
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
