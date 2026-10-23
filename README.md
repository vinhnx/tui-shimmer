# tui-shimmer

<div align="center">

**An animated, Unicode-aware shimmer effect for Ratatui terminal UIs.**

Add a moving highlight to loading and status text with an internal clock or your own frame clock.

[![Crates.io](https://img.shields.io/crates/v/tui-shimmer.svg?style=flat-square)](https://crates.io/crates/tui-shimmer)
[![Docs.rs](https://img.shields.io/badge/Docs.rs-documentation-informational?style=flat-square)](https://docs.rs/tui-shimmer/latest/tui_shimmer/)
[![License: MIT and Apache-2.0](https://img.shields.io/badge/License-MIT%20%2B%20Apache--2.0-blue.svg?style=flat-square)](#license-and-attribution)

<img src="./resources/vtcode.gif" alt="VT Code demo showing tui-shimmer in a terminal" width="70%" />
<br />
<em>Demo from the VT Code terminal application.</em>

</div>

<details>
<summary><strong>Contents</strong></summary>

- [Overview](#overview)
- [Features](#features)
- [Quick start](#quick-start)
  - [1. Install](#1-install)
  - [2. Render shimmer text](#2-render-shimmer-text)
- [Usage](#usage)
  - [Caller-controlled phase](#caller-controlled-phase)
  - [Compose with other content](#compose-with-other-content)
- [API](#api)
- [Terminal compatibility](#terminal-compatibility)
- [Integration](#integration)
- [Development](#development)
- [Contributing](#contributing)
- [License and attribution](#license-and-attribution)

</details>

## Overview

`tui-shimmer` returns styled Ratatui spans with a moving highlight band. Render them in a `Paragraph`, or compose
them with other `Line` and `Text` content. The crate uses Ratatui's style and text types without requiring a terminal
backend.

## Features

- Sweeps left to right for one second every four seconds, after a 600 ms start delay.
- Keeps extended grapheme clusters intact and positions the highlight by terminal display width.
- Blends colors in true-color terminals and uses a basic palette fallback elsewhere.
- Supports an internal monotonic clock or a caller-controlled animation phase.

## Quick start

### 1. Install

Add the crate to your application:

```toml
[dependencies]
tui-shimmer = "0.1"
```

Your application should also depend on Ratatui.

### 2. Render shimmer text

Call `shimmer_spans_with_style` from your draw callback:

```rust
use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;
use tui_shimmer::shimmer_spans_with_style;

let spans = shimmer_spans_with_style("Loading...", Style::default().fg(Color::Cyan));
frame.render_widget(Paragraph::new(spans), area);
```

The self-timed effect waits 600 ms, sweeps for one second, and repeats on a four-second cycle.

## Usage

### Caller-controlled phase

Use `shimmer_spans_with_style_at_phase` when your application already owns a frame clock. The phase is normalized to
a four-second cycle; values outside `0.0..1.0` wrap around, and non-finite values start at phase zero.

```rust
use std::time::Instant;

let start = Instant::now();
let phase = (start.elapsed().as_secs_f32() / 4.0).rem_euclid(1.0);
let spans = tui_shimmer::shimmer_spans_with_style_at_phase("Working...", base_style, phase);
```

### Compose with other content

Both functions return `Vec<Span<'static>>`, which can be rendered directly or composed into a line:

```rust
use ratatui::text::Line;

let shimmer = tui_shimmer::shimmer_spans_with_style("Working...", base_style);
let line = Line::from(shimmer);
```

## API

| Function | Use when |
| --- | --- |
| `shimmer_spans_with_style(text, base_style)` | The library should drive the animation from its monotonic clock. |
| `shimmer_spans_with_style_at_phase(text, base_style, phase)` | Your application already owns the animation clock. |

## Terminal compatibility

- True-color terminals get an RGB highlight blended with the base foreground color.
- Other terminals use a `DarkGray`/`Gray`/`White` fallback with dim and bold modifiers.
- `NO_COLOR` selects the fallback path. Otherwise, nonzero `CLICOLOR_FORCE` forces RGB blending, `CLICOLOR=0` selects
  the fallback, and `COLORTERM=truecolor` or `24bit` enables RGB blending.

## Integration

The demo above shows the library in [VT Code](https://github.com/vinhnx/VTCode), a terminal coding agent maintained by
the author of `tui-shimmer`.

## Development

Run the checks before opening a pull request:

```sh
cargo fmt --all -- --check
cargo check --locked
cargo test --locked
cargo clippy --locked -- -D warnings
```

## Contributing

Bug reports, documentation improvements, and pull requests are welcome. Keep changes focused and include tests for
behavior changes.

## License and attribution

This crate combines the original MIT-licensed implementation with portions adapted from OpenAI Codex under
Apache-2.0. The package metadata declares `MIT AND Apache-2.0`; see [LICENSE](LICENSE), [LICENSE-APACHE](LICENSE-APACHE),
and [NOTICE](NOTICE).

The grapheme-aware sweep adapts OpenAI Codex's
[`summary_shimmer.rs`](https://github.com/openai/codex/blob/71b38795d83f230206098b265c67707e1fb28769/codex-rs/tui/src/summary_shimmer.rs),
Copyright 2025 OpenAI. This crate modifies its timing and positioning for the phase-driven API and uses its own color
handling and fallback behavior. The OpenAI notice applies to the adapted portions.
