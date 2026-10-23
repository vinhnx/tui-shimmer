# AGENTS.md

Guidance for AI agents working on or integrating `tui-shimmer`.

---

## What This Crate Does

`tui-shimmer` is a single-file Rust library (`src/lib.rs`) that produces a
moving highlight effect over text in [Ratatui](https://ratatui.rs/) terminal
UIs. It is used for animated loading/processing indicators.

## Crate Surface

The entire public API is two functions:

```rust
pub fn shimmer_spans_with_style(text: &str, base_style: Style) -> Vec<Span<'static>>
pub fn shimmer_spans_with_style_at_phase(text: &str, base_style: Style, phase: f32) -> Vec<Span<'static>>
```

- `shimmer_spans_with_style` -- drives phase from an internal monotonic clock
  (`Instant::now` captured at first call via `OnceLock`). Call once per frame.
- `shimmer_spans_with_style_at_phase` -- caller supplies `phase` as `0.0..1.0`
  across the four-second shimmer cycle. Use this when the host app already owns
  a frame clock.

Both return `Vec<Span<'static>>`. Drop the result into a `Paragraph`, `Line`,
or `Text`.

## Key Constants (internal, not public)

| Constant | Value | Purpose |
|---|---|---|
| `SHIMMER_PADDING` | 10 columns | Extra space before/after text for smooth band entry/exit |
| `SHIMMER_START_DELAY_SECONDS` | 0.6 | Delay before the first sweep in each cycle |
| `SHIMMER_SWEEP_SECONDS` | 1.0 | Active left-to-right sweep duration |
| `SHIMMER_INTERVAL_SECONDS` | 4.0 | Time between sweep starts |
| `MIN_BAND_HALF_WIDTH` | 3 columns | Minimum half-width; longer text scales the band with its width |

## Architecture

All logic lives in `src/lib.rs`. There are no modules, traits, or type
aliases. The file is ~330 lines and contains:

1. **Static state** -- `OnceLock` for `PROCESS_START` and `TRUECOLOR_CACHE`.
2. **Core renderer** -- iterates Unicode grapheme clusters, measures terminal
   columns, applies a cosine falloff, and batches adjacent equal styles into
   `Span`s without splitting graphemes.
3. **Color path selection** -- `supports_true_color()` checks env vars
   (`NO_COLOR`, `CLICOLOR_FORCE`, `CLICOLOR`, `COLORTERM`). True color path
   blends white toward base fg via `blend_rgb`. Fallback maps intensity to
   `DarkGray`/`Gray`/`White` with `DIM`/`BOLD` modifiers.
4. **Color conversion helpers** -- `color_to_rgb` and `indexed_to_rgb` cover
   all `ratatui::style::Color` variants including the 256-color indexed cube.
5. **Regression tests** -- cover sweep cadence, phase normalization, and
   grapheme preservation.

## Integration Patterns

### Minimal (Ratatui app)

```rust
use tui_shimmer::shimmer_spans_with_style;
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;

// Inside your draw callback:
let spans = shimmer_spans_with_style("Loading...", Style::default().fg(Color::Cyan));
f.render_widget(Paragraph::new(spans), area);
```

### External clock (game loop, async runtime)

```rust
use tui_shimmer::shimmer_spans_with_style_at_phase;

// You already have an Instant or frame counter. Phase covers the four-second cycle:
let phase = (start.elapsed().as_secs_f32() / 4.0).rem_euclid(1.0);
let spans = shimmer_spans_with_style_at_phase("Working...", base_style, phase);
```

### Composing with other content

```rust
use ratatui::text::Line;

let shimmer = shimmer_spans_with_style("header text", Style::default());
let line = Line::from(shimmer);
// append static spans, or mix into a Text block
```

## Dependencies

- `ratatui = "0.30"` (with `default-features = false`). Only needs the
  `style` and `text` features; no full backend required.
- `unicode-segmentation` and `unicode-width` for grapheme-safe terminal-column
  positioning.
- `std` only otherwise (no `tokio`, `serde`, etc.).

## Conventions

- **No `unsafe` code.**
- **No public types beyond the two functions.** Do not add public structs,
  traits, or enums without strong justification.
- **Single-file layout.** Keep everything in `src/lib.rs` unless the file
  exceeds ~500 lines.
- **`OnceLock` for all statics.** No `lazy_static` or `once_cell` dep.
- **Keep upstream attribution intact.** The shimmer adaptation from OpenAI Codex
  is Apache-2.0 licensed; retain its source notice, `LICENSE-APACHE`, and `NOTICE`.
- **`clippy::disallowed_methods`** is intentionally allowed around
  `Color::Rgb` construction (custom RGB needed for shimmer blending).
- Edition 2021. Minimum Rust version not pinned in `Cargo.toml`.

## Common Agent Tasks

### "Add a new shimmer color mode"
Edit the `supports_true_color` / color path logic in `src/lib.rs`. The
`blend_rgb` function handles true-color; `style_for_level` handles fallback.
Add a new `Color` variant mapping in `color_to_rgb` if needed.

### "Change the sweep speed"
Modify `SHIMMER_SWEEP_SECONDS` and `SHIMMER_INTERVAL_SECONDS` together. This is
not a public parameter today; if the caller needs control, expose it through a
new function signature or builder.

### "Add a feature flag"
Add to `[features]` in `Cargo.toml`. The only dependency that makes sense to
gate is `ratatui` itself (e.g., for backend selection).

### "Run tests / verify"
```sh
cargo check
cargo clippy -- -D warnings
cargo test
cargo doc --no-deps
```

## Versioning

The crate is pre-1.0 (current: 0.1.5). Breaking changes may land in minor
releases. The `CHANGELOG.md` tracks notable changes.
