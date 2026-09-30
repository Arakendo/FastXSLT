//! XML name validation shared by compiler and runtime `QName` boundaries.

/// Returns whether `value` is an XML 1.0 Fifth Edition `NCName`.
///
/// An `NCName` uses the XML `Name` character repertoire but excludes `:` so
/// `QName` splitting and namespace resolution remain the caller's concern.
pub(crate) fn is_ncname(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    is_name_start_character(first) && characters.all(is_name_character)
}

fn is_name_start_character(character: char) -> bool {
    matches!(
        character,
        'A'..='Z'
            | '_'
            | 'a'..='z'
            | '\u{00C0}'..='\u{00D6}'
            | '\u{00D8}'..='\u{00F6}'
            | '\u{00F8}'..='\u{02FF}'
            | '\u{0370}'..='\u{037D}'
            | '\u{037F}'..='\u{1FFF}'
            | '\u{200C}'..='\u{200D}'
            | '\u{2070}'..='\u{218F}'
            | '\u{2C00}'..='\u{2FEF}'
            | '\u{3001}'..='\u{D7FF}'
            | '\u{F900}'..='\u{FDCF}'
            | '\u{FDF0}'..='\u{FFFD}'
            | '\u{10000}'..='\u{EFFFF}'
    )
}

fn is_name_character(character: char) -> bool {
    is_name_start_character(character)
        || matches!(
            character,
            '-' | '.' | '0'..='9' | '\u{00B7}' | '\u{0300}'..='\u{036F}' | '\u{203F}'..='\u{2040}'
        )
}

#[cfg(test)]
mod tests {
    use super::is_ncname;

    #[test]
    fn admits_xml_10_non_ascii_names() {
        for name in ["日本語", "élément", "Δοκιμή", "名-1"] {
            assert!(is_ncname(name), "{name}");
        }
    }

    #[test]
    fn rejects_qnames_and_invalid_ncname_characters() {
        for name in ["", "1name", "a:b", "name value", "name/"] {
            assert!(!is_ncname(name), "{name}");
        }
    }
}
