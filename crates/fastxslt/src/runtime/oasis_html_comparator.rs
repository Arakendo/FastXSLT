//! Bounded HTML lexical normalization for the local OASIS comparison harness.
//!
//! This is deliberately not an HTML parser and is not engine behavior. It only
//! converts the small serialization differences exercised by the archival
//! suite into XML-readable text before the existing expanded-name comparator
//! runs.

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "basefont", "br", "col", "frame", "hr", "img", "input", "isindex", "link",
    "meta", "param",
];

pub(super) fn normalize_for_xml_comparison(html: &str) -> Option<String> {
    let mut normalized = String::with_capacity(html.len());
    let mut cursor = 0;

    while cursor < html.len() {
        let relative_open = html[cursor..].find('<');
        let Some(relative_open) = relative_open else {
            normalized.push_str(&html[cursor..]);
            break;
        };
        let open = cursor + relative_open;
        normalized.push_str(&html[cursor..open]);

        if html[open..].starts_with("<!--") {
            let relative_end = html[open + 4..].find("-->")?;
            let end = open + 4 + relative_end + 3;
            normalized.push_str(&html[open..end]);
            cursor = end;
            continue;
        }
        if html[open..].starts_with("<?") {
            let end = find_tag_end(html, open)?;
            normalized.push_str(&html[open..end]);
            if html.as_bytes().get(end.wrapping_sub(1)) != Some(&b'?') {
                normalized.push('?');
            }
            normalized.push('>');
            cursor = end + 1;
            continue;
        }
        if html[open..].starts_with("<!") {
            let end = find_tag_end(html, open)?;
            normalized.push_str(&html[open..=end]);
            cursor = end + 1;
            continue;
        }

        let end = find_tag_end(html, open)?;
        let tag = &html[open..=end];
        let tag_name = tag_name(tag)?;
        let closing = tag.as_bytes().get(1) == Some(&b'/');
        let normalized_tag = normalize_tag(tag, !closing && is_void_element(tag_name))?;
        normalized.push_str(&normalized_tag);
        cursor = end + 1;

        if !closing && matches_ignore_ascii_case(tag_name, "script", "style") {
            let closing_prefix = format!("</{tag_name}");
            let relative_close = find_ascii_case_insensitive(&html[cursor..], &closing_prefix)?;
            let close = cursor + relative_close;
            push_xml_escaped_raw_text(&mut normalized, &html[cursor..close]);
            cursor = close;
        }
    }

    Some(decode_bounded_named_references(&normalized))
}

fn decode_bounded_named_references(value: &str) -> String {
    value
        .replace("&nbsp;", "\u{00a0}")
        .replace("&copy;", "\u{00a9}")
        .replace("&Egrave;", "\u{00c8}")
        .replace("&oacute;", "\u{00f3}")
}

fn find_tag_end(value: &str, open: usize) -> Option<usize> {
    let mut quote = None;
    for (relative, character) in value[open + 1..].char_indices() {
        if let Some(delimiter) = quote {
            if character == delimiter {
                quote = None;
            }
            continue;
        }
        match character {
            '\'' | '"' => quote = Some(character),
            '>' => return Some(open + 1 + relative),
            _ => {}
        }
    }
    None
}

fn tag_name(tag: &str) -> Option<&str> {
    let content = tag.strip_prefix('<')?;
    let content = content.strip_prefix('/').unwrap_or(content).trim_start();
    let end = content
        .find(|character: char| character.is_whitespace() || matches!(character, '/' | '>'))
        .unwrap_or(content.len());
    (end > 0).then(|| &content[..end])
}

fn normalize_tag(tag: &str, make_empty: bool) -> Option<String> {
    let inner = tag.strip_prefix('<')?.strip_suffix('>')?;
    let closing = inner.starts_with('/');
    let inner = inner.strip_prefix('/').unwrap_or(inner).trim();
    let explicit_empty = inner.ends_with('/');
    let inner = inner.strip_suffix('/').unwrap_or(inner).trim_end();
    let name_end = inner.find(char::is_whitespace).unwrap_or(inner.len());
    let name = inner.get(..name_end)?;
    if name.is_empty() {
        return None;
    }
    if closing {
        let normalized_name = if name.contains(':') {
            name.to_owned()
        } else {
            name.to_ascii_lowercase()
        };
        return Some(format!("</{normalized_name}>"));
    }

    let mut attributes = Vec::new();
    let mut remainder = inner.get(name_end..)?;
    loop {
        remainder = remainder.trim_start();
        if remainder.is_empty() {
            break;
        }
        let attribute_end = remainder
            .find(|character: char| character.is_whitespace() || character == '=')
            .unwrap_or(remainder.len());
        let attribute = remainder.get(..attribute_end)?;
        if attribute.is_empty() {
            return None;
        }
        let attribute = attribute.to_ascii_lowercase();
        remainder = remainder.get(attribute_end..)?.trim_start();
        if let Some(after_equals) = remainder.strip_prefix('=') {
            remainder = after_equals.trim_start();
            let (value, rest, delimiter) = take_attribute_value(remainder)?;
            if !attributes
                .iter()
                .any(|(existing, _, _)| existing == &attribute)
            {
                attributes.push((attribute, value.to_owned(), delimiter));
            }
            remainder = rest;
        } else if !attributes
            .iter()
            .any(|(existing, _, _)| existing == &attribute)
        {
            attributes.push((attribute.clone(), attribute, '"'));
        }
    }
    let html_element = !name.contains(':')
        && !attributes
            .iter()
            .any(|(name, value, _)| name == "xmlns" && !value.is_empty());
    let normalized_name = if html_element {
        name.to_ascii_lowercase()
    } else {
        name.to_owned()
    };
    if normalized_name == "meta"
        && attributes.iter().any(|(name, value, _)| {
            name == "http-equiv" && value.eq_ignore_ascii_case("content-type")
        })
    {
        for (name, value, _) in &mut attributes {
            if name == "content" {
                normalize_charset_token(value);
            }
        }
    }

    let mut normalized = format!("<{normalized_name}");
    for (attribute, value, delimiter) in attributes {
        normalized.push(' ');
        normalized.push_str(&attribute);
        normalized.push('=');
        normalized.push(delimiter);
        push_xml_escaped_attribute_value(&mut normalized, &value);
        normalized.push(delimiter);
    }
    if (make_empty && html_element) || explicit_empty {
        normalized.push('/');
    }
    normalized.push('>');
    Some(normalized)
}

fn normalize_charset_token(value: &mut str) {
    let Some(charset) = find_ascii_case_insensitive(value, "charset=") else {
        return;
    };
    let start = charset + "charset=".len();
    let end = value[start..]
        .find(|character: char| character.is_ascii_whitespace() || character == ';')
        .map_or(value.len(), |end| start + end);
    value[start..end].make_ascii_lowercase();
}

fn take_attribute_value(value: &str) -> Option<(&str, &str, char)> {
    if let Some(delimiter) = value
        .chars()
        .next()
        .filter(|value| matches!(value, '\'' | '"'))
    {
        let content = value.get(delimiter.len_utf8()..)?;
        let end = content.find(delimiter)?;
        return Some((
            content.get(..end)?,
            content.get(end + delimiter.len_utf8()..)?,
            delimiter,
        ));
    }
    let end = value.find(char::is_whitespace).unwrap_or(value.len());
    (end > 0).then(|| (&value[..end], &value[end..], '"'))
}

fn is_void_element(name: &str) -> bool {
    VOID_ELEMENTS
        .iter()
        .any(|candidate| name.eq_ignore_ascii_case(candidate))
}

fn matches_ignore_ascii_case(value: &str, first: &str, second: &str) -> bool {
    value.eq_ignore_ascii_case(first) || value.eq_ignore_ascii_case(second)
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .char_indices()
        .map(|(offset, _)| offset)
        .find(|offset| {
            haystack[*offset..]
                .get(..needle.len())
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case(needle))
        })
}

fn push_xml_escaped_raw_text(target: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '&' => target.push_str("&amp;"),
            '<' => target.push_str("&lt;"),
            character => target.push(character),
        }
    }
}

fn push_xml_escaped_attribute_value(target: &mut String, value: &str) {
    let mut remainder = value;
    while let Some(character) = remainder.chars().next() {
        if character == '&' {
            if let Some(reference_length) = xml_reference_length(remainder) {
                target.push_str(&remainder[..reference_length]);
                remainder = &remainder[reference_length..];
            } else {
                target.push_str("&amp;");
                remainder = &remainder[character.len_utf8()..];
            }
        } else {
            if character == '<' {
                target.push_str("&lt;");
            } else {
                target.push(character);
            }
            remainder = &remainder[character.len_utf8()..];
        }
    }
}

fn xml_reference_length(value: &str) -> Option<usize> {
    for reference in [
        "&amp;", "&lt;", "&gt;", "&quot;", "&apos;", "&nbsp;", "&copy;", "&Egrave;", "&oacute;",
    ] {
        if value.starts_with(reference) {
            return Some(reference.len());
        }
    }
    let end = value.find(';')?;
    let body = value.get(1..end)?;
    let valid_numeric = body.strip_prefix("#x").is_some_and(|digits| {
        !digits.is_empty() && digits.chars().all(|digit| digit.is_ascii_hexdigit())
    }) || body.strip_prefix('#').is_some_and(|digits| {
        !digits.is_empty() && digits.chars().all(|digit| digit.is_ascii_digit())
    });
    valid_numeric.then_some(end + 1)
}

#[cfg(test)]
mod tests {
    use super::normalize_for_xml_comparison;

    #[test]
    fn normalizes_html_names_void_elements_and_raw_text_for_xml_comparison() {
        let input = r#"<HTML><META content="Text/HTML"><SCRIPT>if (a < b && c > d) x = "<P>";</SCRIPT></HTML>"#;

        assert_eq!(
            normalize_for_xml_comparison(input).as_deref(),
            Some(
                r#"<html><meta content="Text/HTML"/><script>if (a &lt; b &amp;&amp; c > d) x = "&lt;P>";</script></html>"#
            )
        );
    }

    #[test]
    fn normalizes_unquoted_and_boolean_html_attributes_for_xml_comparison() {
        assert_eq!(
            normalize_for_xml_comparison(r"<INPUT Type=text CHECKED>").as_deref(),
            Some(r#"<input type="text" checked="checked"/>"#)
        );
        assert_eq!(
            normalize_for_xml_comparison(r"<INPUT Checked CHECKED=other checkED>").as_deref(),
            Some(r#"<input checked="checked"/>"#)
        );
    }

    #[test]
    fn normalizes_html_processing_instruction_termination_for_xml_comparison() {
        assert_eq!(
            normalize_for_xml_comparison(r#"<HTML><?my-pi href="book.css"></HTML>"#).as_deref(),
            Some(r#"<html><?my-pi href="book.css"?></html>"#)
        );
        assert_eq!(
            normalize_for_xml_comparison("<html><?already?></html>").as_deref(),
            Some("<html><?already?></html>")
        );
    }

    #[test]
    fn decodes_only_the_bounded_archival_html_named_references() {
        assert_eq!(
            normalize_for_xml_comparison("<P>&nbsp;&copy;&Egrave;&oacute;&amp;&unknown;</P>")
                .as_deref(),
            Some("<p>\u{00a0}\u{00a9}\u{00c8}\u{00f3}&amp;&unknown;</p>")
        );
    }

    #[test]
    fn escapes_raw_html_attribute_characters_without_double_escaping_xml_references() {
        assert_eq!(
            normalize_for_xml_comparison(r#"<AREA href="&{rect};" title="&amp; &#160; &#xA0; <">"#)
                .as_deref(),
            Some(r#"<area href="&amp;{rect};" title="&amp; &#160; &#xA0; &lt;"/>"#)
        );
    }

    #[test]
    fn normalizes_content_type_meta_charset_case_for_html_comparison() {
        assert_eq!(
            normalize_for_xml_comparison(
                r#"<META content="text/html; charset=UTF-8" http-equiv="Content-Type">"#
            )
            .as_deref(),
            Some(r#"<meta content="text/html; charset=utf-8" http-equiv="Content-Type"/>"#)
        );
        assert_eq!(
            normalize_for_xml_comparison(r#"<meta content="example=UTF-8">"#).as_deref(),
            Some(r#"<meta content="example=UTF-8"/>"#)
        );
    }

    #[test]
    fn preserves_foreign_elements_whose_local_names_look_like_html_void_elements() {
        assert_eq!(
            normalize_for_xml_comparison(
                r#"<area xmlns="http://example.test/foreign"></area><foo:INPUT xmlns:foo="http://example.test/foreign"></foo:INPUT>"#
            )
            .as_deref(),
            Some(
                r#"<area xmlns="http://example.test/foreign"></area><foo:INPUT xmlns:foo="http://example.test/foreign"></foo:INPUT>"#
            )
        );
    }
}
