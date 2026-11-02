//! Reusable egui view of an application-owned store; no window or listener.
use super::RecordStore;
use super::TelemetryRecord;
use super::TelemetryValue;

/// Case-insensitive filter over event name, source/session, call ID and attributes.
#[derive(Debug, Default)]
pub struct RecordView {
    pub filter: String,
}

impl RecordView {
    pub fn show(&mut self, ui: &mut egui::Ui, store: &RecordStore) {
        ui.horizontal(|ui| {
            ui.label("Filter");
            ui.text_edit_singleline(&mut self.filter);
        });
        let stats = store.stats();
        ui.label(format!(
            "{} retained · {} evicted · {} rejected · {} upstream truncated",
            stats.records, stats.evicted, stats.rejected_oversize, stats.upstream_truncated
        ));
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (index, record) in store.iter().enumerate() {
                if !matches_filter(record, &self.filter) {
                    continue;
                }
                let event = record.attribute("event.name");
                let name = if record.event_name.is_empty() {
                    match event {
                        Some(TelemetryValue::String(name)) => name.as_str(),
                        _ => "(unknown event)",
                    }
                } else {
                    &record.event_name
                };
                let call = match record.attribute("call_id") {
                    Some(TelemetryValue::String(call)) => call.as_str(),
                    _ => "unknown call",
                };
                egui::CollapsingHeader::new(format!(
                    "{} · {}",
                    escape_display(name),
                    escape_display(call)
                ))
                .id_salt(index)
                .show(ui, |ui| {
                    ui.monospace(escape_display(&format!("{record:#?}")));
                });
            }
        });
    }
}

/// Filter without changing or executing captured data.
pub fn matches_filter(record: &TelemetryRecord, filter: &str) -> bool {
    let filter = filter.to_lowercase();
    filter.is_empty() || format!("{record:?}").to_lowercase().contains(&filter)
}

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
