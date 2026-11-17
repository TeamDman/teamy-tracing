// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable application-owned capture.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
use std::borrow::Cow;

/// Escape source-provided terminal controls while retaining newlines and Unicode.
pub fn escape_terminal_controls(value: &str) -> Cow<'_, str> {
    if !value.chars().any(|character| {
        (character.is_control() && character != '\n')
            || matches!(character, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
    }) {
        return Cow::Borrowed(value);
    }
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        if (character.is_control() && character != '\n')
            || matches!(character, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        {
            escaped.extend(character.escape_default());
        } else {
            escaped.push(character);
        }
    }
    Cow::Owned(escaped)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_is_preserved_but_bidirectional_controls_are_visible() {
        let plain = "café 資料\n";
        assert!(matches!(escape_terminal_controls(plain), Cow::Borrowed(_)));
        assert_eq!(escape_terminal_controls(plain), plain);
        let escaped = escape_terminal_controls("café \u{202e}hidden\u{2066}direction");
        assert_eq!(escaped, "café \\u{202e}hidden\\u{2066}direction");
    }
}
