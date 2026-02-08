# tui-shimmer

Shimmer text effect for terminal UIs for [Ratatui](https://ratatui.rs/).

[Libs.rs](https://lib.rs/crates/tui-shimmer) | [Docs.rs](https://docs.rs/tui-shimmer/latest/tui_shimmer/) | [Crates.io](https://crates.io/crates/tui-shimmer)

![gif](https://raw.githubusercontent.com/vinhnx/vtcode/main/resources/gif/vtcode.gif)

Demo usage from my [VT Code](https://github.com/vinhnx/vtcode) coding agent.

## Usage

```rust
use ratatui::style::Style;
use tui_shimmer::shimmer_spans_with_style;

let spans = shimmer_spans_with_style("Loading...", Style::default());
```

## API stability

The public API is experimental until 1.0. Expect occasional breaking changes
in minor releases.
