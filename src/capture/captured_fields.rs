// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable application-owned capture.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
use super::LogValue;
#[cfg(feature = "eyre")]
use crate::DiagnosticReport;
use tracing::field::Field;
use tracing::field::Visit;

#[derive(Clone, Debug)]
#[cfg_attr(not(feature = "eyre"), derive(Default))]
pub(super) struct CapturedFields {
    pub(super) message: Option<String>,
    pub(super) fields: Vec<(String, LogValue)>,
    #[cfg(feature = "eyre")]
    retain_reports: bool,
}

#[cfg(feature = "eyre")]
impl Default for CapturedFields {
    fn default() -> Self {
        Self {
            message: None,
            fields: Vec::new(),
            #[cfg(feature = "eyre")]
            retain_reports: true,
        }
    }
}

impl CapturedFields {
    pub(super) fn for_span() -> Self {
        Self {
            // A report can capture this same span in its SpanTrace. Retaining
            // it in the span's extensions would create a strong ownership cycle.
            #[cfg(feature = "eyre")]
            retain_reports: false,
            ..Self::default()
        }
    }

    pub(super) fn merge(&mut self, other: &Self) -> Vec<LogValue> {
        let mut replaced = Vec::new();
        if other.message.is_some() {
            self.message = other.message.clone();
        }
        for (name, value) in &other.fields {
            if let Some(existing) = self.fields.iter_mut().find(|(key, _)| key == name) {
                replaced.push(std::mem::replace(&mut existing.1, value.clone()));
            } else {
                self.fields.push((name.clone(), value.clone()));
            }
        }
        replaced
    }

    fn record_value(&mut self, field: &Field, value: impl Into<LogValue>) {
        let value = value.into();
        if field.name() == "message" {
            self.message = Some(value.to_string());
        } else {
            self.fields.push((field.name().to_string(), value));
        }
    }
}

impl Visit for CapturedFields {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.record_value(field, format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.record_value(field, value.to_string());
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.record_value(field, value.to_string());
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.record_value(field, value.to_string());
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.record_value(field, value.to_string());
    }

    fn record_i128(&mut self, field: &Field, value: i128) {
        self.record_value(field, value.to_string());
    }

    fn record_u128(&mut self, field: &Field, value: u128) {
        self.record_value(field, value.to_string());
    }

    fn record_f64(&mut self, field: &Field, value: f64) {
        self.record_value(field, value.to_string());
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        #[cfg(feature = "eyre")]
        if self.retain_reports
            && let Some(report) = value.downcast_ref::<DiagnosticReport>()
        {
            self.record_value(field, LogValue::Report(report.clone()));
            return;
        }
        self.record_value(field, value.to_string());
    }
}
