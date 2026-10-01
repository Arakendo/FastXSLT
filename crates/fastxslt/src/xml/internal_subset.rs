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
    attributes: HashMap<(String, String), DeclaredAttribute>,
    references: Cell<usize>,
    replacement_bytes: Cell<usize>,
    declaration_work: usize,
    limits: InternalSubsetLimits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DeclaredAttributeType {
    Cdata,
    Id,
    IdRef,
}

#[derive(Debug)]
struct DeclaredAttribute {
    declared_type: DeclaredAttributeType,
    default_value: Option<String>,
}

#[derive(Debug)]
pub(super) struct DefaultAttribute {
    pub(super) name: String,
    pub(super) value: String,
    pub(super) declared_type: DeclaredAttributeType,
}

type DeclaredAttributes = HashMap<(String, String), DeclaredAttribute>;
type ParsedAttributeDeclaration = (String, String, DeclaredAttribute);
type RawEntities = HashMap<String, String>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExternalSubsetFailureOrigin {
    Doctype,
    ExternalSubset,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ExternalSubsetFailure {
    pub(super) origin: ExternalSubsetFailureOrigin,
    pub(super) failure: InternalSubsetFailure,
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

    pub(super) fn attribute_type(
        &self,
        element_name: &str,
        attribute_name: &str,
    ) -> Option<DeclaredAttributeType> {
        self.attributes
            .get(&(element_name.to_owned(), attribute_name.to_owned()))
            .map(|attribute| attribute.declared_type)
    }

    pub(super) fn default_attributes(
        &self,
        element_name: &str,
        present_names: &std::collections::HashSet<String>,
    ) -> Result<Vec<DefaultAttribute>, InternalSubsetFailure> {
        let mut defaults = Vec::new();
        for ((element, name), declaration) in &self.attributes {
            if element != element_name || present_names.contains(name) {
                continue;
            }
            let Some(value) = &declaration.default_value else {
                continue;
            };
            let replacement_bytes = self.replacement_bytes.get().saturating_add(value.len());
            if replacement_bytes > self.limits.replacement_bytes {
                return Err(InternalSubsetFailure::Limit(format!(
                    "DTD replacement-byte limit is {}",
                    self.limits.replacement_bytes
                )));
            }
            self.replacement_bytes.set(replacement_bytes);
            defaults.push(DefaultAttribute {
                name: name.clone(),
                value: value.clone(),
                declared_type: declaration.declared_type,
            });
        }
        defaults.sort_unstable_by(|left, right| left.name.cmp(&right.name));
        Ok(defaults)
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

    let declarations = parse_declarations(subset, limits)?;
    build_entities(declarations, limits)
}

pub(super) fn parse_single_external_subset(
    doctype: &str,
    external_subset: &[u8],
    max_external_bytes: usize,
    limits: InternalSubsetLimits,
) -> Result<(String, InternalEntities), ExternalSubsetFailure> {
    let (head, internal) = split_doctype(doctype).map_err(|failure| ExternalSubsetFailure {
        origin: ExternalSubsetFailureOrigin::Doctype,
        failure,
    })?;
    let system_reference =
        parse_system_identifier(head).map_err(|failure| ExternalSubsetFailure {
            origin: ExternalSubsetFailureOrigin::Doctype,
            failure,
        })?;
    if external_subset.len() > max_external_bytes {
        return Err(ExternalSubsetFailure {
            origin: ExternalSubsetFailureOrigin::ExternalSubset,
            failure: InternalSubsetFailure::Limit(format!(
                "external DTD byte limit is {max_external_bytes}"
            )),
        });
    }
    let external = std::str::from_utf8(external_subset).map_err(|error| ExternalSubsetFailure {
        origin: ExternalSubsetFailureOrigin::ExternalSubset,
        failure: InternalSubsetFailure::Unsupported(format!(
            "external DTD encoding remains unsupported: {error}"
        )),
    })?;
    let mut declarations =
        parse_declarations(external, limits).map_err(|failure| ExternalSubsetFailure {
            origin: ExternalSubsetFailureOrigin::ExternalSubset,
            failure,
        })?;
    let remaining = limits
        .declarations
        .checked_sub(declarations.2)
        .ok_or_else(|| ExternalSubsetFailure {
            origin: ExternalSubsetFailureOrigin::ExternalSubset,
            failure: InternalSubsetFailure::Limit(format!(
                "DTD declaration limit is {}",
                limits.declarations
            )),
        })?;
    let internal_limits = InternalSubsetLimits {
        declarations: remaining,
        ..limits
    };
    let internal =
        parse_declarations(internal, internal_limits).map_err(|failure| ExternalSubsetFailure {
            origin: ExternalSubsetFailureOrigin::Doctype,
            failure,
        })?;
    merge_declarations(&mut declarations, internal);
    let entities =
        build_entities(declarations, limits).map_err(|failure| ExternalSubsetFailure {
            origin: ExternalSubsetFailureOrigin::ExternalSubset,
            failure,
        })?;
    Ok((system_reference, entities))
}

fn build_entities(
    declarations: (RawEntities, DeclaredAttributes, usize),
    limits: InternalSubsetLimits,
) -> Result<InternalEntities, InternalSubsetFailure> {
    let (raw, attributes, declaration_count) = declarations;
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
        attributes,
        references: Cell::new(declared_references),
        replacement_bytes: Cell::new(declared_replacement_bytes),
        declaration_work,
        limits,
    })
}

fn merge_declarations(
    target: &mut (RawEntities, DeclaredAttributes, usize),
    additional: (RawEntities, DeclaredAttributes, usize),
) {
    let (entities, attributes, count) = additional;
    for (name, value) in entities {
        target.0.insert(name, value);
    }
    for (name, declaration) in attributes {
        target.1.insert(name, declaration);
    }
    target.2 = target.2.saturating_add(count);
}

fn parse_system_identifier(head: &str) -> Result<String, InternalSubsetFailure> {
    let head = trim_xml_space(head);
    let root_end = head.find(is_xml_space).unwrap_or(head.len());
    let root = &head[..root_end];
    if !is_xml_name(root) {
        return Err(InternalSubsetFailure::Malformed(
            "DOCTYPE root name is malformed".to_owned(),
        ));
    }
    let remainder = head[root_end..].trim_start_matches(is_xml_space);
    let Some(remainder) = remainder.strip_prefix("SYSTEM") else {
        return Err(InternalSubsetFailure::Unsupported(
            "only one SYSTEM external subset is admitted by this experiment".to_owned(),
        ));
    };
    if !remainder.starts_with(is_xml_space) {
        return Err(InternalSubsetFailure::Malformed(
            "DOCTYPE SYSTEM identifier requires whitespace".to_owned(),
        ));
    }
    let remainder = remainder.trim_start_matches(is_xml_space);
    let Some(quote @ ('\'' | '"')) = remainder.chars().next() else {
        return Err(InternalSubsetFailure::Malformed(
            "DOCTYPE SYSTEM identifier must be quoted".to_owned(),
        ));
    };
    let quoted = &remainder[quote.len_utf8()..];
    let Some(end) = quoted.find(quote) else {
        return Err(InternalSubsetFailure::Malformed(
            "DOCTYPE SYSTEM identifier is not closed".to_owned(),
        ));
    };
    if !trim_xml_space(&quoted[end + quote.len_utf8()..]).is_empty() {
        return Err(InternalSubsetFailure::Unsupported(
            "content after the SYSTEM identifier remains unsupported".to_owned(),
        ));
    }
    Ok(quoted[..end].to_owned())
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
) -> Result<(HashMap<String, String>, DeclaredAttributes, usize), InternalSubsetFailure> {
    let mut declarations = HashMap::new();
    let mut attributes = HashMap::new();
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
        if let Some(instruction) = remaining.strip_prefix("<?") {
            let end = instruction.find("?>").ok_or_else(|| {
                InternalSubsetFailure::Malformed(
                    "DTD processing instruction is not closed".to_owned(),
                )
            })?;
            remaining = &instruction[end + 2..];
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
        } else if let Some(declaration) = remaining.strip_prefix("<!ATTLIST") {
            let (types, rest) = parse_attribute_list_declaration(declaration)?;
            for (element, attribute, declaration) in types {
                attributes
                    .entry((element, attribute))
                    .or_insert(declaration);
            }
            remaining = rest;
        } else {
            return Err(InternalSubsetFailure::Unsupported(
                "the reference DTD profile currently admits only ELEMENT and general internal ENTITY declarations"
                    .to_owned(),
            ));
        }
    }
    Ok((declarations, attributes, declaration_count))
}

fn parse_attribute_list_declaration(
    declaration: &str,
) -> Result<(Vec<ParsedAttributeDeclaration>, &str), InternalSubsetFailure> {
    if !declaration.starts_with(is_xml_space) {
        return Err(InternalSubsetFailure::Malformed(
            "ATTLIST declaration requires whitespace".to_owned(),
        ));
    }
    let mut parser = AttributeListParser::new(declaration);
    parser.skip_space();
    let element = parser.take_name("ATTLIST element name is malformed")?;
    let mut declarations = Vec::new();
    loop {
        let had_space = parser.skip_required_space();
        if parser.peek() == Some('>') {
            parser.position += 1;
            return Ok((declarations, &declaration[parser.position..]));
        }
        if !had_space {
            return Err(InternalSubsetFailure::Malformed(
                "ATTLIST attribute declaration requires whitespace".to_owned(),
            ));
        }
        let attribute = parser.take_name("ATTLIST attribute name is malformed")?;
        if !parser.skip_required_space() {
            return Err(InternalSubsetFailure::Malformed(
                "ATTLIST attribute type is missing".to_owned(),
            ));
        }
        let declared_type = parser.take_attribute_type()?;
        if !parser.skip_required_space() {
            return Err(InternalSubsetFailure::Malformed(
                "ATTLIST default declaration is missing".to_owned(),
            ));
        }
        let default_value = parser
            .take_default_declaration()?
            .map(|value| normalize_declared_attribute_value(&value, declared_type));
        if declared_type == DeclaredAttributeType::Id && default_value.is_some() {
            return Err(InternalSubsetFailure::Unsupported(
                "defaulted ID attributes remain outside the bounded profile".to_owned(),
            ));
        }
        declarations.push((
            element.clone(),
            attribute,
            DeclaredAttribute {
                declared_type,
                default_value,
            },
        ));
    }
}

struct AttributeListParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> AttributeListParser<'a> {
    const fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn take_attribute_type(&mut self) -> Result<DeclaredAttributeType, InternalSubsetFailure> {
        let token = self.take_token();
        match token {
            "CDATA" => Ok(DeclaredAttributeType::Cdata),
            "ID" => Ok(DeclaredAttributeType::Id),
            "IDREF" => Ok(DeclaredAttributeType::IdRef),
            _ => Err(InternalSubsetFailure::Unsupported(format!(
                "DTD attribute type remains unsupported: {token}"
            ))),
        }
    }

    fn take_default_declaration(&mut self) -> Result<Option<String>, InternalSubsetFailure> {
        if self.input[self.position..].starts_with("#REQUIRED") {
            self.position += "#REQUIRED".len();
            return Ok(None);
        }
        if self.input[self.position..].starts_with("#IMPLIED") {
            self.position += "#IMPLIED".len();
            return Ok(None);
        }
        if self.input[self.position..].starts_with("#FIXED") {
            self.position += "#FIXED".len();
            if !self.skip_required_space() {
                return Err(InternalSubsetFailure::Malformed(
                    "#FIXED requires an attribute value".to_owned(),
                ));
            }
        }
        self.take_default_value().map(Some)
    }

    fn take_default_value(&mut self) -> Result<String, InternalSubsetFailure> {
        let Some(quote @ ('\'' | '"')) = self.peek() else {
            return Err(InternalSubsetFailure::Malformed(
                "ATTLIST default declaration is malformed".to_owned(),
            ));
        };
        self.position += quote.len_utf8();
        let start = self.position;
        while self.peek().is_some_and(|character| character != quote) {
            self.position += self.peek().expect("checked character").len_utf8();
        }
        if self.peek() != Some(quote) {
            return Err(InternalSubsetFailure::Malformed(
                "ATTLIST default value is not closed".to_owned(),
            ));
        }
        let raw = &self.input[start..self.position];
        self.position += quote.len_utf8();
        if raw.contains('<') || raw.contains('%') {
            return Err(InternalSubsetFailure::Unsupported(
                "markup and parameter references in attribute defaults remain denied".to_owned(),
            ));
        }
        quick_xml::escape::unescape(raw)
            .map(std::borrow::Cow::into_owned)
            .map_err(|error| {
                InternalSubsetFailure::Unsupported(format!(
                    "attribute-default entity semantics remain unsupported: {error}"
                ))
            })
    }

    fn take_name(&mut self, message: &str) -> Result<String, InternalSubsetFailure> {
        let token = self.take_token();
        if is_xml_name(token) {
            Ok(token.to_owned())
        } else {
            Err(InternalSubsetFailure::Malformed(message.to_owned()))
        }
    }

    fn take_token(&mut self) -> &str {
        let start = self.position;
        while let Some(character) = self.peek() {
            if is_xml_space(character) || character == '>' {
                break;
            }
            self.position += character.len_utf8();
        }
        &self.input[start..self.position]
    }

    fn skip_required_space(&mut self) -> bool {
        let start = self.position;
        self.skip_space();
        self.position > start
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

fn normalize_declared_attribute_value(value: &str, declared_type: DeclaredAttributeType) -> String {
    let spaces_normalized = value
        .chars()
        .map(|character| {
            if is_xml_space(character) {
                ' '
            } else {
                character
            }
        })
        .collect::<String>();
    if declared_type == DeclaredAttributeType::Cdata {
        spaces_normalized
    } else {
        spaces_normalized
            .split(' ')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
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
    use std::collections::HashSet;

    use super::{
        DeclaredAttributeType, ExternalSubsetFailureOrigin, InternalSubsetFailure,
        InternalSubsetLimits, parse_internal_subset, parse_single_external_subset,
    };

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
    fn parses_one_bounded_system_external_subset_without_acquiring_it() {
        let external = br"
            <!ELEMENT Plant-Sheet (Author | item)*>
            <!ELEMENT Author (#PCDATA)>
            <!ELEMENT item (#PCDATA)>
            <!ATTLIST Plant-Sheet xmlns:xsl CDATA #FIXED 'urn:legacy-xsl'>
        ";
        let (reference, declarations) = parse_single_external_subset(
            "Plant-Sheet SYSTEM 'plants.dtd' [<?fixture local?> <!-- local -->]",
            external,
            1_024,
            LIMITS,
        )
        .expect("one already supplied external subset should parse");

        assert_eq!(reference, "plants.dtd");
        let defaults = declarations
            .default_attributes("Plant-Sheet", &HashSet::new())
            .expect("external fixed attribute should be retained");
        assert_eq!(defaults.len(), 1);
        assert_eq!(defaults[0].name, "xmlns:xsl");
        assert_eq!(defaults[0].value, "urn:legacy-xsl");
    }

    #[test]
    fn first_repeated_attribute_declaration_remains_binding() {
        let entities = parse_internal_subset(
            "root [<!ATTLIST root value CDATA 'first'><!ATTLIST root value CDATA 'second'>]",
            LIMITS,
        )
        .expect("XML DTD first-declaration binding should remain deterministic");
        let defaults = entities
            .default_attributes("root", &HashSet::new())
            .expect("first default should expand");
        assert_eq!(defaults.len(), 1);
        assert_eq!(defaults[0].value, "first");
    }

    #[test]
    fn bounds_and_classifies_the_single_external_subset_surface() {
        let too_large = parse_single_external_subset(
            "root SYSTEM 'root.dtd'",
            b"<!ELEMENT root EMPTY>",
            4,
            LIMITS,
        )
        .expect_err("external bytes require an independent bound");
        assert_eq!(
            too_large.origin,
            ExternalSubsetFailureOrigin::ExternalSubset
        );
        assert_eq!(
            too_large.failure,
            InternalSubsetFailure::Limit("external DTD byte limit is 4".to_owned())
        );

        for declaration in [
            "root PUBLIC '-//EXAMPLE//DTD Root//EN' 'root.dtd'",
            "root SYSTEM 'root.dtd' extra",
        ] {
            let failure = parse_single_external_subset(declaration, b"", 4, LIMITS)
                .expect_err("broader external identifier forms remain unsupported");
            assert_eq!(failure.origin, ExternalSubsetFailureOrigin::Doctype);
        }

        let (_, duplicate) = parse_single_external_subset(
            "root SYSTEM 'root.dtd' [<!ATTLIST root value CDATA #IMPLIED>]",
            b"<!ATTLIST root value CDATA 'external'>",
            128,
            LIMITS,
        )
        .expect("internal declarations should take precedence over external declarations");
        assert!(
            duplicate
                .default_attributes("root", &HashSet::new())
                .expect("internal implied declaration has no default")
                .is_empty()
        );

        let cycle = parse_single_external_subset(
            "root SYSTEM 'root.dtd' [<!ENTITY internal '&external;'>]",
            b"<!ENTITY external '&internal;'>",
            128,
            LIMITS,
        )
        .expect_err("cycles spanning external and internal declarations must be rejected");
        assert_eq!(cycle.origin, ExternalSubsetFailureOrigin::ExternalSubset);
        assert!(matches!(cycle.failure, InternalSubsetFailure::Malformed(_)));

        let aggregate_limit = parse_single_external_subset(
            "root SYSTEM 'root.dtd' [<!ENTITY internal 'two'>]",
            b"<!ENTITY external 'one'>",
            128,
            InternalSubsetLimits {
                declarations: 1,
                ..LIMITS
            },
        )
        .expect_err("external and internal declarations share one aggregate ceiling");
        assert_eq!(aggregate_limit.origin, ExternalSubsetFailureOrigin::Doctype);
        assert!(matches!(
            aggregate_limit.failure,
            InternalSubsetFailure::Limit(_)
        ));
    }

    #[test]
    fn denies_external_parameter_and_non_entity_declarations() {
        for doctype in [
            "root SYSTEM 'file:///secret'",
            "root [<!ENTITY % shared 'value'>]",
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
    fn retains_only_non_defaulting_bounded_attribute_types() {
        let subset = parse_internal_subset(
            "root [<!ELEMENT root ANY><!ATTLIST root id ID #REQUIRED ref IDREF #IMPLIED note CDATA #IMPLIED>]",
            LIMITS,
        )
        .expect("bounded non-defaulting ATTLIST should parse");

        assert_eq!(
            subset.attribute_type("root", "id"),
            Some(DeclaredAttributeType::Id)
        );
        assert_eq!(
            subset.attribute_type("root", "ref"),
            Some(DeclaredAttributeType::IdRef)
        );
        assert_eq!(
            subset.attribute_type("root", "note"),
            Some(DeclaredAttributeType::Cdata)
        );
        assert_eq!(subset.declaration_work(), 2);
    }

    #[test]
    fn retains_bounded_literal_defaults_and_rejects_unadmitted_attribute_types() {
        let subset = parse_internal_subset(
            "root [<!ATTLIST root value CDATA '  default  ' fixed CDATA #FIXED 'yes'>]",
            LIMITS,
        )
        .expect("bounded literal defaults should parse");
        let defaults = subset
            .default_attributes("root", &HashSet::new())
            .expect("defaults should remain within replacement budget");
        assert_eq!(defaults.len(), 2);
        assert_eq!(defaults[0].name, "fixed");
        assert_eq!(defaults[0].value, "yes");
        assert_eq!(defaults[1].name, "value");
        assert_eq!(defaults[1].value, "  default  ");

        assert!(matches!(
            parse_internal_subset("root [<!ATTLIST root value NMTOKEN #IMPLIED>]", LIMITS),
            Err(InternalSubsetFailure::Unsupported(_))
        ));
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
