use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, layout::Rect, style::Color};

use rataudio_rta::{Band, RTA};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut last_time = std::time::Instant::now();

    // Update the bands every 500ms
    const UPDATE_INTERVAL: Duration = Duration::from_millis(1000);

    let mut bands = vec![];
    for i in 0..100 {
        let value = (i as f64 / 100.0).clamp(0.0, 1.0);
        bands.push(Band {
            value,
            color: Color::Yellow,
        });
    }

    loop {
        if last_time.elapsed() >= UPDATE_INTERVAL {
            last_time = std::time::Instant::now();
        }

        terminal.draw(|frame| draw(frame, &bands))?;
        if handle_input()? == Command::Quit {
            break Ok(());
        }
    }
}

fn draw(frame: &mut Frame, bands: &[Band]) {
    let size = Rect::new(0, 0, 180, 24);
    frame.render_widget(
        RTA {
            bands: bands.to_vec(),
        },
        size,
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Noop,
    Quit,
}

fn handle_input() -> Result<Command> {
    if !event::poll(Duration::from_secs_f64(1.0 / 60.0))? {
        return Ok(Command::Noop);
    }
    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
            KeyCode::Char('q') => Ok(Command::Quit),
            _ => Ok(Command::Noop),
        },
        _ => Ok(Command::Noop),
    }
}
