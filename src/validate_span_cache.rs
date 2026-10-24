use facet_format::FormatParser;
use facet_format::ParseEventKind;
use facet_json::JsonParser;
use std::fmt;

/// Validate the complete single-line object before any RawJson lexemes are reused.
/// Raw capture alone is not a strict grammar/EOF check in the pinned parser.
pub(crate) fn validate_span_cache(source: &str) -> fmt::Result {
    if source.len() > u32::MAX as usize || source.bytes().any(|b| b < b' ' && b != b'\t') {
        return Err(fmt::Error);
    }
    let bytes = source.as_bytes();
    // Public extensions can contain malformed escapes even when the full str is
    // UTF-8. Use the checked decoder before token-level escape validation.
    let mut parser = JsonParser::<false>::new(bytes);
    let first = parser
        .next_event()
        .map_err(|_| fmt::Error)?
        .ok_or(fmt::Error)?;
    if !matches!(first.kind, ParseEventKind::StructStart(_)) {
        return Err(fmt::Error);
    }
    let mut end = first.span.offset as usize + first.span.len as usize;
    while let Some(event) = parser.next_event().map_err(|_| fmt::Error)? {
        let start = event.span.offset as usize;
        end = start
            .checked_add(event.span.len as usize)
            .ok_or(fmt::Error)?;
        let token = bytes.get(start..end).ok_or(fmt::Error)?;
        match event.kind {
            ParseEventKind::StructEnd | ParseEventKind::SequenceEnd => {
                // The parser permits trailing commas; JSON/NDJSON does not.
                if bytes[..start]
                    .iter()
                    .rev()
                    .find(|b| !matches!(b, b' ' | b'\t'))
                    == Some(&b',')
                {
                    return Err(fmt::Error);
                }
            }
            ParseEventKind::FieldKey(_) | ParseEventKind::Scalar(_) => {
                if token.first() == Some(&b'"') {
                    // The pinned parser also permits some invalid escapes.
                    if !json_string(token) {
                        return Err(fmt::Error);
                    }
                } else if token != b"true"
                    && token != b"false"
                    && token != b"null"
                    && !json_number(token)
                {
                    return Err(fmt::Error);
                }
            }
            _ => {}
        }
    }
    // next_event returns None at root completion, not necessarily input exhaustion.
    if !bytes
        .get(end..)
        .ok_or(fmt::Error)?
        .iter()
        .all(|b| matches!(b, b' ' | b'\t'))
    {
        return Err(fmt::Error);
    }
    Ok(())
}

/// Syntax only: never round-trip the numeric token through a lossy numeric type.
fn json_number(mut bytes: &[u8]) -> bool {
    if bytes.first() == Some(&b'-') {
        bytes = &bytes[1..];
    }
    match bytes.first() {
        Some(b'0') => bytes = &bytes[1..],
        Some(b'1'..=b'9') => {
            bytes = &bytes[1..];
            while bytes.first().is_some_and(u8::is_ascii_digit) {
                bytes = &bytes[1..];
            }
        }
        _ => return false,
    }
    if bytes.first() == Some(&b'.') {
        bytes = &bytes[1..];
        if !bytes.first().is_some_and(u8::is_ascii_digit) {
            return false;
        }
        while bytes.first().is_some_and(u8::is_ascii_digit) {
            bytes = &bytes[1..];
        }
    }
    if bytes.first().is_some_and(|b| matches!(b, b'e' | b'E')) {
        bytes = &bytes[1..];
        if bytes.first().is_some_and(|b| matches!(b, b'+' | b'-')) {
            bytes = &bytes[1..];
        }
        if !bytes.first().is_some_and(u8::is_ascii_digit) {
            return false;
        }
        while bytes.first().is_some_and(u8::is_ascii_digit) {
            bytes = &bytes[1..];
        }
    }
    bytes.is_empty()
}

/// Check JSON escape syntax without decoding or rewriting the original token.
fn json_string(token: &[u8]) -> bool {
    if token.len() < 2 || token.first() != Some(&b'"') || token.last() != Some(&b'"') {
        return false;
    }
    let mut bytes = &token[1..token.len() - 1];
    while let Some(&byte) = bytes.first() {
        if byte == b'\\' {
            bytes = &bytes[1..];
            match bytes.first() {
                Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => bytes = &bytes[1..],
                Some(b'u')
                    if bytes
                        .get(1..5)
                        .is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit)) =>
                {
                    bytes = &bytes[5..]
                }
                _ => return false,
            }
        } else if byte < b' ' || byte == b'"' {
            return false;
        } else {
            bytes = &bytes[1..];
        }
    }
    true
}
