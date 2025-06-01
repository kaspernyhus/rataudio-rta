use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, layout::Rect};

use rand::{Rng, rng};
use rataudio_rta::{Band, RTA};

use simplelog::*;
use std::fs::File;

fn init_logging() {
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("app.log").unwrap(),
    )
    .unwrap();
}

fn main() -> Result<()> {
    color_eyre::install()?;
    init_logging();
    log::debug!("HELLO");
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    let mut last_time = std::time::Instant::now();

    const UPDATE_INTERVAL: Duration = Duration::from_millis(100);

    let f_min: f64 = 20.0;
    let f_max: f64 = 20000.0;
    let n_bands = 100;

    let freqs: Vec<f64> = (0..n_bands)
        .map(|i| {
            let ratio = i as f64 / (n_bands - 1) as f64;
            f_min * (f_max / f_min).powf(ratio)
        })
        .collect();

    let mut bands = vec![];
    for i in 0..n_bands {
        bands.push(Band::new(0.0, freqs[i] as u16));
    }

    loop {
        if last_time.elapsed() >= UPDATE_INTERVAL {
            last_time = std::time::Instant::now();
            for i in 0..bands.len() {
                let current = bands[i].value;

                let new_val = (current + rng().random_range(-0.3..0.2)).clamp(0.0, 1.0);

                bands[i].set_value(new_val);
            }
        }

        terminal.draw(|frame| draw(frame, &bands))?;
        if handle_input()? == Command::Quit {
            break Ok(());
        }
    }
}

fn draw(frame: &mut Frame, bands: &[Band]) {
    let rta_area = Rect::new(0, 0, 105, 28);

    let rta = RTA::new(bands.to_vec());

    frame.render_widget(rta, rta_area);
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
