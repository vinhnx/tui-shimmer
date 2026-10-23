# tui-shimmer

[![Crates.io](https://img.shields.io/crates/v/tui-shimmer.svg)](https://crates.io/crates/tui-shimmer)
[![Docs.rs](https://img.shields.io/badge/Docs.rs-documentation-informational)](https://docs.rs/tui-shimmer/latest/tui_shimmer/)
[![License: MIT and Apache-2.0](https://img.shields.io/badge/License-MIT%20%2B%20Apache--2.0-blue.svg)](#license-and-attribution)

A shimmer text effect for [Ratatui](https://ratatui.rs/) terminal UIs.

![VT Code terminal demo featuring tui-shimmer](resources/vtcode.gif)

`tui-shimmer` is integrated into [VT Code](https://github.com/vinhnx/VTCode),
a terminal coding agent maintained by the author of this crate. The demo above
shows the library in a working application.

Part of the [Ratatui](https://ratatui.rs/) ecosystem -- see
[awesome-ratatui](https://github.com/ratatui/awesome-ratatui) for more
community widgets and tools.

---

## Quick Start

```toml
[dependencies]
tui-shimmer = "0.1"
```

**Minimal integration** -- call `shimmer_spans_with_style` inside your draw loop:

```rust
use ratatui::style::{Color, Style};
use tui_shimmer::shimmer_spans_with_style;

fn draw_loading(f: &mut ratatui::Frame, area: ratatui::layout::Rect) {
    let spans = shimmer_spans_with_style("Loading...", Style::default().fg(Color::Cyan));
    let paragraph = ratatui::widgets::Paragraph::new(spans);
    f.render_widget(paragraph, area);
}
```

The shimmer phase is driven by an internal monotonic clock. It waits 600 ms,
sweeps left-to-right for one second, then rests until the next four-second cycle.

---

## API

| Function                                                     | Use when |
| ------------------------------------------------------------ | --- |
| `shimmer_spans_with_style(text, base_style)`                 | Default. Phase derived from elapsed time. |
| `shimmer_spans_with_style_at_phase(text, base_style, phase)` | You control timing externally (game loop, manual tick, etc.). `phase` is the normalized position in a four-second cycle (`0.0..1.0`). |

Both return `Vec<Span<'static>>` -- render it directly in a `Paragraph` or
compose with other `Line`/`Text` content.

### Choosing a phase source

```rust
// -- self-timed, zero setup
shimmer_spans_with_style

// -- use when your app already has a frame clock
// (avoids double-elapsed-time drift under load)
shimmer_spans_with_style_at_phase
```

---

## Full Example

```rust
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Paragraph},
    Terminal,
};
use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};
use tui_shimmer::shimmer_spans_with_style_at_phase;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let start = Instant::now();
    loop {
        let phase = (start.elapsed().as_secs_f32() / 4.0).rem_euclid(1.0);
        terminal.draw(|f| {
            let area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)])
                .split(f.size())[0];

            let spans = shimmer_spans_with_style_at_phase(
                "Welcome to tui-shimmer!",
                Style::default().fg(Color::Cyan),
                phase,
            );
            let paragraph = Paragraph::new(spans)
                .block(Block::bordered().title("Demo"))
                .centered();
            f.render_widget(paragraph, area);
        })?;

        if event::poll(Duration::from_millis(16))?
            && matches!(event::read()?, Event::Key(k) if matches!(k.code, KeyCode::Char('q') | KeyCode::Esc))
        {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
```

---

## Terminal Compatibility

- **True-color terminals** (most modern terminals): full RGB shimmer blend.
- **256-color / 16-color terminals**: automatic fallback to bold/grey ramp.
- Respects the [NO_COLOR](https://no-color.org/) and `CLICOLOR`/`CLICOLOR_FORCE` environment variables.

---

## License and attribution

This crate combines the existing MIT-licensed implementation with portions
adapted from OpenAI Codex under Apache-2.0. The package metadata declares
`MIT AND Apache-2.0`; see [LICENSE](LICENSE), [LICENSE-APACHE](LICENSE-APACHE),
and [NOTICE](NOTICE).

The grapheme-aware sweep adapts OpenAI Codex's
[`summary_shimmer.rs`](https://github.com/openai/codex/blob/71b38795d83f230206098b265c67707e1fb28769/codex-rs/tui/src/summary_shimmer.rs),
Copyright 2025 OpenAI. The adaptation is marked in the source and changes the
timing and positioning to fit this crate's phase-driven API, color handling, and
fallback behavior. The OpenAI notice applies to those adapted portions.

## Contributing

Issues and pull requests are welcome. Before submitting, run:

```sh
cargo fmt --all -- --check
cargo check --locked
cargo test --locked
cargo clippy --locked -- -D warnings
```
