# tui-shimmer

[![Lib.rs](https://img.shields.io/badge/Lib.rs-crate-informational?logo=rust)](https://lib.rs/crates/tui-shimmer)
[![Docs.rs](https://img.shields.io/badge/Docs.rs-documentation-informational?logo=docsdotrs)](https://docs.rs/tui-shimmer/latest/tui_shimmer/)
[![Crates.io](https://img.shields.io/crates/v/tui-shimmer.svg)](https://crates.io/crates/tui-shimmer)

Shimmer text effect for terminal UIs for [Ratatui](https://ratatui.rs/).

![gif](https://raw.githubusercontent.com/vinhnx/vtcode/main/resources/gif/vtcode.gif)

Demo usage from my [VT Code](https://github.com/vinhnx/vtcode) coding agent.

## Features

- Smooth shimmer animation effect
- True color support with fallbacks
- Lightweight and efficient
- Seamless integration with Ratatui

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tui-shimmer = "0.1"
```

## Usage

### Basic Usage

```rust
use ratatui::style::Style;
use tui_shimmer::shimmer_spans_with_style;

// Create shimmer effect with default style
let spans = shimmer_spans_with_style("Loading...", Style::default());

// Or with custom style
use ratatui::style::Color;
let custom_style = Style::default().fg(Color::Blue);
let spans = shimmer_spans_with_style("Processing...", custom_style);
```

### Advanced Usage

For more control over the animation timing, you can use `shimmer_spans_with_style_at_phase`:

```rust
use ratatui::style::Style;
use tui_shimmer::shimmer_spans_with_style_at_phase;

// Control the animation phase manually (0.0 to 1.0)
let phase = 0.5; // Middle of the animation cycle
let spans = shimmer_spans_with_style_at_phase("Custom Phase...", Style::default(), phase);
```

### Complete Example

Here's a complete example showing how to integrate tui-shimmer into a Ratatui application:

```rust
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Paragraph},
    Frame, Terminal,
};
use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};
use tui_shimmer::shimmer_spans_with_style;

struct App {
    start_time: Instant,
}

impl App {
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    fn get_animation_phase(&self) -> f32 {
        let elapsed = self.start_time.elapsed().as_secs_f32();
        // Cycle every 2 seconds
        (elapsed / 2.0) % 1.0
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.code == event::KeyCode::Char('q') || key.code == event::KeyCode::Esc {
                    return Ok(());
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    // Create shimmer text with custom style
    let shimmer_text = "Welcome to tui-shimmer demo!";
    let shimmer_spans = shimmer_spans_with_style(shimmer_text, Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(shimmer_spans)
        .block(Block::bordered().title("Shimmer Effect Demo"))
        .centered();

    f.render_widget(paragraph, chunks[0]);
}
```

## API Stability

The public API is experimental until 1.0. Expect occasional breaking changes
in minor releases.
