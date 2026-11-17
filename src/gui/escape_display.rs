/// Show control and bidirectional formatting characters visibly.
pub fn escape_display(input: &str) -> String {
    let mut result = String::new();
    for ch in input.chars() {
        if (ch.is_control() && ch != '\n' && ch != '\t')
            || matches!(ch, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
        {
            result.extend(ch.escape_unicode());
        } else {
            result.push(ch);
        }
    }
    result
}
