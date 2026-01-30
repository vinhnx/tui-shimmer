use std::sync::OnceLock;
use std::time::{Duration, Instant};

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

const SHIMMER_PADDING: usize = 10;
const SHIMMER_SWEEP_SECONDS: f32 = 2.0;
const BAND_HALF_WIDTH: usize = 5;

static PROCESS_START: OnceLock<Instant> = OnceLock::new();
static TRUECOLOR_CACHE: OnceLock<bool> = OnceLock::new();
static INTENSITY_LUT: OnceLock<Vec<f32>> = OnceLock::new();

fn elapsed_since_start() -> Duration {
    let start = PROCESS_START.get_or_init(Instant::now);
    start.elapsed()
}

fn intensity_lut() -> &'static [f32] {
    INTENSITY_LUT.get_or_init(|| {
        let mut values = Vec::with_capacity(BAND_HALF_WIDTH + 1);
        let band_half_width = BAND_HALF_WIDTH as f32;
        for dist in 0..=BAND_HALF_WIDTH {
            let intensity = if band_half_width > 0.0 {
                let x = std::f32::consts::PI * (dist as f32 / band_half_width);
                0.5 * (1.0 + x.cos())
            } else {
                0.0
            };
            values.push(intensity);
        }
        values
    })
}

fn shimmer_phase_from_elapsed() -> f32 {
    if SHIMMER_SWEEP_SECONDS <= 0.0 {
        return 0.0;
    }
    let elapsed = elapsed_since_start().as_secs_f32() / SHIMMER_SWEEP_SECONDS;
    elapsed.rem_euclid(1.0)
}

/// Creates a shimmer text effect for terminal UIs.
///
/// This function takes a text string and a base style, then returns a vector of styled spans
/// that create a moving shimmer effect. The shimmer appears to sweep across the text from
/// left to right in a continuous loop.
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
/// This is useful for driving animation from an external frame/tick source to avoid
/// time-based jumps under heavy CPU load.
pub fn shimmer_spans_with_style_at_phase(
    text: &str,
    base_style: Style,
    phase: f32,
) -> Vec<Span<'static>> {
    let char_count = text.chars().count();
    if char_count == 0 {
        return Vec::new();
    }

    let phase = phase.rem_euclid(1.0);
    let period = char_count + SHIMMER_PADDING * 2;
    let pos = (phase * period as f32) as isize;

    let base_rgb = base_style
        .fg
        .and_then(color_to_rgb)
        .unwrap_or((128, 128, 128));
    let highlight_rgb = (255, 255, 255);
    let has_true_color = supports_true_color();
    let lut = intensity_lut();

    let mut spans = Vec::with_capacity(char_count);
    let mut buffer = String::new();
    let mut current_style: Option<Style> = None;

    for (index, ch) in text.chars().enumerate() {
        let i_pos = index as isize + SHIMMER_PADDING as isize;
        let dist = (i_pos - pos).abs() as usize;
        let intensity = if dist <= BAND_HALF_WIDTH { lut[dist] } else { 0.0 };

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
        buffer.push(ch);
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
