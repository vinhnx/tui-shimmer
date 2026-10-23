// Portions of this file adapt the shimmer in OpenAI Codex's
// `codex-rs/tui/src/summary_shimmer.rs` (Apache-2.0).
// Copyright 2025 OpenAI. Modified for this crate's phase-driven API and color fallbacks.
// Source: https://github.com/openai/codex/blob/71b38795d83f230206098b265c67707e1fb28769/codex-rs/tui/src/summary_shimmer.rs

use std::sync::OnceLock;
use std::time::{Duration, Instant};

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const SHIMMER_PADDING: usize = 10;
const SHIMMER_START_DELAY_SECONDS: f64 = 0.6;
const SHIMMER_SWEEP_SECONDS: f64 = 1.0;
const SHIMMER_INTERVAL_SECONDS: f64 = 4.0;
const MIN_BAND_HALF_WIDTH: f64 = 3.0;

static PROCESS_START: OnceLock<Instant> = OnceLock::new();
static TRUECOLOR_CACHE: OnceLock<bool> = OnceLock::new();

fn elapsed_since_start() -> Duration {
    let start = PROCESS_START.get_or_init(Instant::now);
    start.elapsed()
}

fn shimmer_phase_from_elapsed() -> f32 {
    (elapsed_since_start()
        .as_secs_f64()
        .rem_euclid(SHIMMER_INTERVAL_SECONDS)
        / SHIMMER_INTERVAL_SECONDS) as f32
}

fn normalize_phase(phase: f32) -> f64 {
    if phase.is_finite() {
        f64::from(phase.rem_euclid(1.0))
    } else {
        0.0
    }
}

fn sweep_progress(phase: f32) -> f64 {
    let cycle_elapsed = normalize_phase(phase) * SHIMMER_INTERVAL_SECONDS;
    let sweep_elapsed =
        (cycle_elapsed - SHIMMER_START_DELAY_SECONDS).clamp(0.0, SHIMMER_SWEEP_SECONDS);
    sweep_elapsed / SHIMMER_SWEEP_SECONDS
}

/// Creates a shimmer text effect for terminal UIs.
///
/// This function takes a text string and a base style, then returns a vector of styled spans
/// that create a moving shimmer effect. The shimmer appears to sweep across the text from
/// left to right for one second every four seconds, after an initial 600 ms delay.
///
/// # Arguments
///
/// * `text` - The text to apply the shimmer effect to
/// * `base_style` - The base style to use for the text, the shimmer effect will blend from this
///
/// # Returns
///
/// A vector of `Span` elements with dynamically applied styles to create the shimmer effect
///
/// # Example
///
/// ```rust
/// use ratatui::style::Style;
/// use tui_shimmer::shimmer_spans_with_style;
///
/// let spans = shimmer_spans_with_style("Loading...", Style::default());
/// ```
pub fn shimmer_spans_with_style(text: &str, base_style: Style) -> Vec<Span<'static>> {
    shimmer_spans_with_style_at_phase(text, base_style, shimmer_phase_from_elapsed())
}

/// Creates a shimmer effect at a fixed phase (0.0..1.0).
///
/// The phase represents a four-second cycle. The shimmer waits 600 ms, sweeps
/// from left to right for one second, then remains off the text until the next
/// cycle. Non-finite phase values are treated as zero.
///
/// Use this when the host app already owns a frame clock. For example, divide
/// elapsed seconds by `4.0` and wrap the result with `rem_euclid(1.0)`.
pub fn shimmer_spans_with_style_at_phase(
    text: &str,
    base_style: Style,
    phase: f32,
) -> Vec<Span<'static>> {
    let grapheme_count = text.graphemes(true).count();
    if grapheme_count == 0 {
        return Vec::new();
    }

    let text_width = text.width();
    if text_width == 0 {
        return vec![Span::styled(text.to_owned(), base_style)];
    }

    let progress = sweep_progress(phase);
    let half_width = (text_width as f64 * 0.1).max(MIN_BAND_HALF_WIDTH);
    let padding = SHIMMER_PADDING as f64;
    let position = progress * (text_width as f64 + padding * 2.0) - padding;

    let base_rgb = base_style
        .fg
        .and_then(color_to_rgb)
        .unwrap_or((128, 128, 128));
    let highlight_rgb = (255, 255, 255);
    let has_true_color = supports_true_color();
    let mut spans = Vec::with_capacity(grapheme_count);
    let mut buffer = String::new();
    let mut current_style: Option<Style> = None;
    let mut column = 0.0;

    for grapheme in text.graphemes(true) {
        let grapheme_width = grapheme.width() as f64;
        let center = column + grapheme_width / 2.0;
        column += grapheme_width;

        let distance = ((center - position).abs() / half_width).min(1.0);
        let intensity = 0.5 * (1.0 + (std::f64::consts::PI * distance).cos());
        let intensity = intensity as f32;

        let style = if has_true_color {
            let highlight = intensity.clamp(0.0, 1.0) * 0.9;
            let (r, g, b) = blend_rgb(highlight_rgb, base_rgb, highlight);
            // Custom RGB is intentional for shimmer.
            #[allow(clippy::disallowed_methods)]
            {
                let mut style = base_style.fg(Color::Rgb(r, g, b));
                if intensity > 0.0 {
                    style = style.add_modifier(Modifier::BOLD);
                }
                style
            }
        } else {
            style_for_level(intensity, base_style)
        };

        let same_style = current_style
            .as_ref()
            .is_some_and(|current| current == &style);
        if !same_style {
            if let Some(prev_style) = current_style.take() {
                if !buffer.is_empty() {
                    spans.push(Span::styled(buffer, prev_style));
                    buffer = String::new();
                }
            }
            current_style = Some(style);
        }
        buffer.push_str(grapheme);
    }

    if let Some(final_style) = current_style {
        if !buffer.is_empty() {
            spans.push(Span::styled(buffer, final_style));
        }
    }

    spans
}

fn supports_true_color() -> bool {
    *TRUECOLOR_CACHE.get_or_init(|| {
        if std::env::var_os("NO_COLOR").is_some() {
            return false;
        }

        if std::env::var("CLICOLOR_FORCE")
            .ok()
            .as_deref()
            .is_some_and(|value| value != "0")
        {
            return true;
        }

        if std::env::var("CLICOLOR")
            .ok()
            .as_deref()
            .is_some_and(|value| value == "0")
        {
            return false;
        }

        std::env::var("COLORTERM")
            .ok()
            .map(|val| {
                let lower = val.to_lowercase();
                lower.contains("truecolor") || lower.contains("24bit")
            })
            .unwrap_or(false)
    })
}

fn style_for_level(intensity: f32, base_style: Style) -> Style {
    let mut style = base_style;
    let color = if intensity < 0.2 {
        Color::DarkGray
    } else if intensity < 0.6 {
        Color::Gray
    } else {
        Color::White
    };
    style = style.fg(color);
    if intensity < 0.2 {
        style.add_modifier(Modifier::DIM)
    } else if intensity < 0.6 {
        style
    } else {
        style.add_modifier(Modifier::BOLD)
    }
}

fn blend_rgb(highlight: (u8, u8, u8), base: (u8, u8, u8), amount: f32) -> (u8, u8, u8) {
    let amount = amount.clamp(0.0, 1.0);
    let blend = |from: u8, to: u8| -> u8 {
        let from = from as f32;
        let to = to as f32;
        (from + (to - from) * amount).round().clamp(0.0, 255.0) as u8
    };

    (
        blend(base.0, highlight.0),
        blend(base.1, highlight.1),
        blend(base.2, highlight.2),
    )
}

fn color_to_rgb(color: Color) -> Option<(u8, u8, u8)> {
    match color {
        Color::Rgb(r, g, b) => Some((r, g, b)),
        Color::Black => Some((0, 0, 0)),
        Color::Red => Some((170, 0, 0)),
        Color::Green => Some((0, 170, 0)),
        Color::Yellow => Some((170, 85, 0)),
        Color::Blue => Some((0, 0, 170)),
        Color::Magenta => Some((170, 0, 170)),
        Color::Cyan => Some((0, 170, 170)),
        Color::Gray => Some((170, 170, 170)),
        Color::DarkGray => Some((85, 85, 85)),
        Color::LightRed => Some((255, 85, 85)),
        Color::LightGreen => Some((85, 255, 85)),
        Color::LightYellow => Some((255, 255, 85)),
        Color::LightBlue => Some((85, 85, 255)),
        Color::LightMagenta => Some((255, 85, 255)),
        Color::LightCyan => Some((85, 255, 255)),
        Color::White => Some((255, 255, 255)),
        Color::Indexed(code) => Some(indexed_to_rgb(code)),
        Color::Reset => None,
    }
}

fn indexed_to_rgb(code: u8) -> (u8, u8, u8) {
    match code {
        0 => (0, 0, 0),
        1 => (170, 0, 0),
        2 => (0, 170, 0),
        3 => (170, 85, 0),
        4 => (0, 0, 170),
        5 => (170, 0, 170),
        6 => (0, 170, 170),
        7 => (170, 170, 170),
        8 => (85, 85, 85),
        9 => (255, 85, 85),
        10 => (85, 255, 85),
        11 => (255, 255, 85),
        12 => (85, 85, 255),
        13 => (255, 85, 255),
        14 => (85, 255, 255),
        15 => (255, 255, 255),
        n if (16..=231).contains(&n) => {
            let adjusted = n - 16;
            let r = adjusted / 36;
            let g = (adjusted % 36) / 6;
            let b = adjusted % 6;
            let scale = |value: u8| if value == 0 { 0 } else { 55 + value * 40 };
            (scale(r), scale(g), scale(b))
        }
        n if n >= 232 => {
            let gray = 8 + (n - 232) * 10;
            (gray, gray, gray)
        }
        _ => (128, 128, 128),
    }
}

#[cfg(test)]
mod tests {
    use super::{shimmer_spans_with_style_at_phase, sweep_progress};
    use ratatui::style::Style;
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn sweep_uses_codex_cadence() {
        assert_eq!(sweep_progress(0.125), 0.0);
        assert!((sweep_progress(0.25) - 0.4).abs() < 1e-6);
        assert_eq!(sweep_progress(0.5), 1.0);
        assert_eq!(sweep_progress(0.875), 1.0);
    }

    #[test]
    fn phase_wraps_and_non_finite_values_start_at_zero() {
        let style = Style::default();
        assert_eq!(
            shimmer_spans_with_style_at_phase("hello", style, 0.25),
            shimmer_spans_with_style_at_phase("hello", style, 1.25)
        );
        assert_eq!(
            shimmer_spans_with_style_at_phase("hello", style, f32::NAN),
            shimmer_spans_with_style_at_phase("hello", style, 0.0)
        );
    }

    #[test]
    fn shimmer_keeps_extended_graphemes_intact() {
        let text = "e\u{301} 界 👩‍💻";
        let expected = text.graphemes(true).collect::<Vec<_>>();

        for step in 0..32 {
            let phase = step as f32 / 32.0;
            let spans = shimmer_spans_with_style_at_phase(text, Style::default(), phase);
            let actual = spans
                .iter()
                .flat_map(|span| span.content.graphemes(true))
                .collect::<Vec<_>>();
            assert_eq!(actual, expected);
        }
    }
}
