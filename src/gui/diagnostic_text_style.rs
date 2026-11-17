// SPDX-License-Identifier: LiLiQ-R-1.1
// Adapted from Cloud Terrastodon; modified for reusable egui diagnostic rendering.
// See THIRD_PARTY_NOTICES.md and LICENSES/Cloud-Terrastodon-LiLiQ-R-1.1*.txt.
use egui::Color32;
use egui::FontId;
use egui::Stroke;
use egui::TextFormat;
use std::str::Split;

/// The terminal text styles that can be represented in an egui diagnostic.
#[derive(Clone, Debug, Default)]
pub(crate) struct DiagnosticTextStyle {
    foreground: Option<Color32>,
    background: Option<Color32>,
    strong: bool,
    dim: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
}

impl DiagnosticTextStyle {
    /// Apply one complete SGR sequence, retaining the old style on malformed input.
    pub(crate) fn apply(&mut self, parameters: &str) -> Option<()> {
        let mut next = self.clone();
        let mut parameters = parameters.split(';');
        while let Some(parameter) = parameters.next() {
            let parameter = if parameter.is_empty() {
                0
            } else {
                parameter.parse::<u8>().ok()?
            };
            match parameter {
                0 => next = Self::default(),
                1 => next.strong = true,
                2 => next.dim = true,
                3 => next.italic = true,
                4 => next.underline = true,
                9 => next.strikethrough = true,
                22 => {
                    next.strong = false;
                    next.dim = false;
                }
                23 => next.italic = false,
                24 => next.underline = false,
                29 => next.strikethrough = false,
                30..=37 => next.foreground = Some(Self::indexed_color(parameter - 30)),
                38 => next.foreground = Some(Self::extended_color(&mut parameters)?),
                39 => next.foreground = None,
                40..=47 => next.background = Some(Self::indexed_color(parameter - 40)),
                48 => next.background = Some(Self::extended_color(&mut parameters)?),
                49 => next.background = None,
                90..=97 => next.foreground = Some(Self::indexed_color(parameter - 90 + 8)),
                100..=107 => next.background = Some(Self::indexed_color(parameter - 100 + 8)),
                _ => return None,
            }
        }
        *self = next;
        Some(())
    }

    pub(crate) fn text_format(&self) -> TextFormat {
        // Emphasis follows the terminal palette without requiring a custom bold face.
        let mut foreground = self.foreground.unwrap_or(if self.strong {
            Color32::WHITE
        } else {
            Color32::from_gray(220)
        });
        if self.dim {
            foreground = foreground.gamma_multiply(0.65);
        }
        TextFormat {
            font_id: FontId::monospace(12.0),
            color: foreground,
            background: self.background.unwrap_or(Color32::TRANSPARENT),
            italics: self.italic,
            underline: if self.underline {
                Stroke::new(1.0_f32, foreground)
            } else {
                Stroke::NONE
            },
            strikethrough: if self.strikethrough {
                Stroke::new(1.0_f32, foreground)
            } else {
                Stroke::NONE
            },
            ..Default::default()
        }
    }

    fn extended_color(parameters: &mut Split<'_, char>) -> Option<Color32> {
        match parameters.next()? {
            "5" => Some(Self::indexed_color(parameters.next()?.parse().ok()?)),
            "2" => Some(Color32::from_rgb(
                parameters.next()?.parse().ok()?,
                parameters.next()?.parse().ok()?,
                parameters.next()?.parse().ok()?,
            )),
            _ => None,
        }
    }

    fn indexed_color(index: u8) -> Color32 {
        const COLORS: [[u8; 3]; 16] = [
            [0, 0, 0],
            [205, 49, 49],
            [13, 188, 121],
            [229, 229, 16],
            [36, 114, 200],
            [188, 63, 188],
            [17, 168, 205],
            [229, 229, 229],
            [102, 102, 102],
            [241, 76, 76],
            [35, 209, 139],
            [245, 245, 67],
            [59, 142, 234],
            [214, 112, 214],
            [41, 184, 219],
            [255, 255, 255],
        ];
        match index {
            0..=15 => {
                let [red, green, blue] = COLORS[usize::from(index)];
                Color32::from_rgb(red, green, blue)
            }
            16..=231 => {
                const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
                let index = usize::from(index - 16);
                Color32::from_rgb(
                    LEVELS[index / 36],
                    LEVELS[(index / 6) % 6],
                    LEVELS[index % 6],
                )
            }
            232..=255 => Color32::from_gray(8 + (index - 232) * 10),
        }
    }
}
