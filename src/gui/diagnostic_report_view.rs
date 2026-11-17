// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable egui diagnostic rendering.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
use super::diagnostic_text_style::DiagnosticTextStyle;
use crate::DiagnosticReport;
use egui::Color32;
use egui::Frame;
use egui::Label;
use egui::ScrollArea;
use egui::Ui;
use egui::text::LayoutJob;

/// Display a retained report's terminal styling as selectable GUI text.
///
/// The shared report renderer escapes unsupported controls before this view
/// translates supported ANSI colors and text styles. Copying uses its plain
/// representation, preserving all diagnostic sections without terminal escapes.
#[derive(Debug)]
pub struct DiagnosticReportView<'a> {
    pub report: &'a DiagnosticReport,
}

impl<'a> DiagnosticReportView<'a> {
    pub fn new(report: &'a DiagnosticReport) -> Self {
        Self { report }
    }

    pub fn show(&self, ui: &mut Ui) {
        if ui.button("Copy error").clicked() {
            ui.ctx().copy_text(self.report.to_string());
        }
        // color-eyre's default palette is designed for a dark background.
        Frame::new()
            .fill(Color32::from_gray(30))
            .inner_margin(10.0)
            .show(ui, |ui| {
                ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                    ui.add(Label::new(self.layout_job()).selectable(true).wrap());
                });
            });
    }

    /// Render the complete diagnostic as styled text for a custom egui layout.
    pub fn layout_job(&self) -> LayoutJob {
        let formatted = self.report.styled().to_string();
        let mut remaining = formatted.as_str();
        let mut style = DiagnosticTextStyle::default();
        let mut job = LayoutJob::default();
        while let Some(start) = remaining.find("\x1b[") {
            job.append(&remaining[..start], 0.0, style.text_format());
            remaining = &remaining[start..];
            if let Some(end) = remaining.find('m')
                && style.apply(&remaining[2..end]).is_some()
            {
                remaining = &remaining[end + 1..];
            } else {
                // The renderer normally escapes these. Keep an unexpected
                // escape visible if the shared style vocabulary ever changes.
                job.append("\\u{1b}", 0.0, style.text_format());
                remaining = &remaining[1..];
            }
        }
        job.append(remaining, 0.0, style.text_format());
        job
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn styled_details_and_plain_copy_retain_the_same_complete_diagnostic() {
        let report = DiagnosticReport::new(
            eyre::eyre!(
                "\x1b[91moriginal detail\x1b[0m\n\x1b[38;2;1;8;255;4mUnicode: café 資料\x1b[0m\n\x1b[2J\x1b]0;untrusted title\x07"
            )
            .wrap_err("Could not load synthetic resources"),
        );
        let job = DiagnosticReportView::new(&report).layout_job();
        let copied = report.to_string();

        assert_eq!(job.text, copied);
        assert!(copied.contains("Could not load synthetic resources"));
        assert!(copied.contains("original detail"));
        assert!(copied.contains("Unicode: café 資料"));
        assert!(copied.contains("\\u{1b}[2J\\u{1b}]0;untrusted title\\u{7}"));
        assert!(!copied.contains('\x1b'));
        let original = job
            .sections
            .iter()
            .find(|section| {
                &job.text[section.byte_range.start.0..section.byte_range.end.0] == "original detail"
            })
            .expect("colored source error");
        assert_eq!(original.format.color, Color32::from_rgb(241, 76, 76));
        let unicode = job
            .sections
            .iter()
            .find(|section| {
                &job.text[section.byte_range.start.0..section.byte_range.end.0]
                    == "Unicode: café 資料"
            })
            .expect("underlined source error");
        assert_eq!(unicode.format.color, Color32::from_rgb(1, 8, 255));
        assert_eq!(unicode.format.underline.width, 1.0);
    }

    #[test]
    fn resets_restore_default_styles_after_extended_color_and_emphasis() {
        let report =
            DiagnosticReport::new(eyre::eyre!("\x1b[3;9;48;5;236mstyled\x1b[23;29;49mplain"));
        let job = DiagnosticReportView::new(&report).layout_job();
        let styled = job
            .sections
            .iter()
            .find(|section| {
                &job.text[section.byte_range.start.0..section.byte_range.end.0] == "styled"
            })
            .expect("styled source error");
        assert!(styled.format.italics);
        assert_eq!(styled.format.strikethrough.width, 1.0);
        assert_eq!(styled.format.background, Color32::from_gray(48));
        let plain = job
            .sections
            .iter()
            .find(|section| {
                job.text[section.byte_range.start.0..section.byte_range.end.0].starts_with("plain")
            })
            .expect("plain source error");
        assert!(!plain.format.italics);
        assert_eq!(plain.format.strikethrough, egui::Stroke::NONE);
        assert_eq!(plain.format.background, Color32::TRANSPARENT);
    }
}
