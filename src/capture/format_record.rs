// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable application-owned capture.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
use super::LogLevel;
use super::LogRecord;
use super::LogValue;
use super::escape_terminal_controls;
use owo_colors::OwoColorize;
use owo_colors::Stream;
use owo_colors::Style;
use std::fmt::Write as _;

/// Render using stderr's color policy, including `NO_COLOR` and `FORCE_COLOR`.
/// The caller still chooses the destination; this never acquires stderr.
pub fn format_record(record: &LogRecord) -> String {
    format_record_with_style(record, true, |text, style| {
        text.if_supports_color(Stream::Stderr, |text| text.style(style))
            .to_string()
    })
}

fn format_record_with_style(
    record: &LogRecord,
    _retain_report_style: bool,
    styled: impl Fn(&str, Style) -> String,
) -> String {
    let mut event = format!(
        "  {} {}",
        format_level(record.level),
        escape_terminal_controls(&record.message)
    );
    for (name, value) in &record.fields {
        #[cfg(feature = "eyre")]
        let value = match value {
            LogValue::Text(value) => value,
            #[cfg(feature = "eyre")]
            LogValue::Report(_) => continue,
        };
        #[cfg(not(feature = "eyre"))]
        let LogValue::Text(value) = value;
        let _ = write!(
            event,
            ", {}: {}",
            escape_terminal_controls(name),
            escape_terminal_controls(value)
        );
    }

    let level_style = match record.level {
        LogLevel::Trace => Style::new().purple(),
        LogLevel::Debug => Style::new().blue(),
        LogLevel::Info => Style::new().green(),
        LogLevel::Warn => Style::new().yellow(),
        LogLevel::Error => Style::new().red(),
    };
    let mut output = styled(&event, level_style);
    if let Some(file) = &record.file {
        let _ = write!(
            output,
            "\n    {} {}",
            styled("at", Style::new().dimmed().italic()),
            escape_terminal_controls(file)
        );
        if let Some(line) = record.line {
            let _ = write!(output, ":{line}");
        }
    }

    for span in &record.spans {
        let _ = write!(
            output,
            "\n    {} {}",
            styled("in", Style::new().dimmed().italic()),
            styled(&escape_terminal_controls(&span.name), Style::new().bold())
        );
        if !span.fields.is_empty() {
            output.push(' ');
            output.push_str(&styled("with", Style::new().dimmed().italic()));
            output.push(' ');
            for (index, (name, value)) in span.fields.iter().enumerate() {
                if index != 0 {
                    output.push_str(", ");
                }
                let _ = write!(output, "{}: ", escape_terminal_controls(name));
                #[cfg(feature = "eyre")]
                match value {
                    LogValue::Text(text) => output.push_str(&escape_terminal_controls(text)),
                    #[cfg(feature = "eyre")]
                    LogValue::Report(report) => {
                        if _retain_report_style {
                            let _ = write!(output, "{}", report.terminal());
                        } else {
                            let _ = write!(output, "{report}");
                        }
                    }
                }
                #[cfg(not(feature = "eyre"))]
                {
                    let LogValue::Text(text) = value;
                    output.push_str(&escape_terminal_controls(text));
                }
            }
        }
    }
    #[cfg(feature = "eyre")]
    for (name, value) in &record.fields {
        if let LogValue::Report(report) = value {
            let _ = write!(
                output,
                "\n    {}:\n{}",
                escape_terminal_controls(name),
                if _retain_report_style {
                    report.terminal().to_string()
                } else {
                    report.to_string()
                }
            );
        }
    }
    output
}

/// Render without ANSI styling, retaining complete typed diagnostic details.
pub fn format_record_plain(record: &LogRecord) -> String {
    format_record_with_style(record, false, |text, _| text.to_owned())
}

fn format_level(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Trace => "TRACE",
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => " INFO",
        LogLevel::Warn => " WARN",
        LogLevel::Error => "ERROR",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capture::LogSpan;
    use std::sync::Arc;
    use std::time::SystemTime;
    #[test]
    fn pretty_format_includes_fields_and_source_location() {
        let record = LogRecord {
            timestamp: SystemTime::now(),
            level: LogLevel::Info,
            target: Arc::from("cloud_terrastodon_entrypoint"),
            message: Arc::from("hello world"),
            fields: vec![("answer".to_string(), "42".into())],
            spans: Vec::new(),
            file: Some(Arc::from("crates/entrypoint/src/echo.rs")),
            line: Some(12),
        };

        assert_eq!(
            format_record_with_style(&record, false, |text, _| text.to_owned()),
            "   INFO hello world, answer: 42\n    at crates/entrypoint/src/echo.rs:12"
        );
    }

    #[test]
    fn pretty_format_includes_empty_span_context() {
        let record = LogRecord {
            timestamp: SystemTime::now(),
            level: LogLevel::Info,
            target: Arc::from("cloud_terrastodon_entrypoint"),
            message: Arc::from("hello world"),
            fields: Vec::new(),
            spans: vec![LogSpan {
                name: Arc::from("cli_invocation"),
                fields: Vec::new(),
            }],
            file: None,
            line: None,
        };

        assert_eq!(
            format_record_with_style(&record, false, |text, _| text.to_owned()),
            "   INFO hello world\n    in cli_invocation"
        );
    }

    #[test]
    fn terminal_format_colours_the_level_without_hyperlinking() {
        let record = LogRecord {
            timestamp: SystemTime::now(),
            level: LogLevel::Info,
            target: Arc::from("cloud_terrastodon_entrypoint"),
            message: Arc::from("hello world"),
            fields: Vec::new(),
            spans: Vec::new(),
            file: Some(Arc::from("crates/entrypoint/src/echo.rs")),
            line: Some(12),
        };

        let output =
            format_record_with_style(&record, false, |text, style| text.style(style).to_string());

        assert!(output.starts_with("\x1b[32m   INFO hello world\x1b[0m"));
        assert!(output.contains("\n    \x1b[2;3mat\x1b[0m crates/entrypoint/src/echo.rs:12"));
        assert!(!output.contains("\x1b]8;;"));
    }

    #[test]
    fn terminal_controls_in_record_content_are_escaped_before_styling() {
        let record = LogRecord {
            timestamp: SystemTime::UNIX_EPOCH,
            level: LogLevel::Warn,
            target: Arc::from("test"),
            message: Arc::from("unsafe\x1b[2J\nsecond line"),
            fields: vec![("key\r".to_owned(), "value\x07".into())],
            spans: vec![LogSpan {
                name: Arc::from("span\x1b[2J"),
                fields: vec![("scope".to_owned(), "value\x08".into())],
            }],
            file: Some(Arc::from("file\x1b[2J.rs")),
            line: Some(42),
        };
        let plain = format_record_with_style(&record, false, |text, _| text.to_owned());
        assert!(
            !plain
                .chars()
                .any(|character| character.is_control() && character != '\n')
        );
        assert!(plain.contains("unsafe\\u{1b}[2J\nsecond line"));
        assert!(plain.contains("file\\u{1b}[2J.rs:42"));
        let styled =
            format_record_with_style(&record, false, |text, style| text.style(style).to_string());
        assert!(styled.contains("\x1b[33m"));
        assert!(!styled.contains("\x1b[2J"));
    }
    #[test]
    fn text_span_fields_are_rendered_once_with_or_without_report_support() {
        let record = LogRecord {
            timestamp: SystemTime::UNIX_EPOCH,
            level: LogLevel::Info,
            target: Arc::from("synthetic"),
            message: Arc::from("observed"),
            fields: Vec::new(),
            spans: vec![LogSpan {
                name: Arc::from("context"),
                fields: vec![("phase".to_owned(), "unique value".into())],
            }],
            file: None,
            line: None,
        };
        let rendered = format_record_plain(&record);
        assert_eq!(rendered.matches("unique value").count(), 1);
        assert!(rendered.contains("phase: unique value"));
    }
}
