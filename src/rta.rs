use std::iter::zip;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Color,
    widgets::Widget,
};

/// A widget to display an RTA audio meter.
///
/// A `RTA` renders a bar filled according to the value given to [`RTA::db`], [`RTA::sample_amplitude`] or
/// [`RTA::ratio`]. The bar width and height are defined by the [`Rect`] it is
/// [rendered](Widget::render) in.
///
/// [`RTA`] is also a [`StatefulWidget`], which means you can use it with [`RTAState`] to allow
/// the meter to hold its peak value for a certain amount of time.
#[derive(Debug, Clone)]
pub struct RTA {
    pub bands: Vec<Band>,
}

/// A struct representing a single frequency band in the equalizer.
#[derive(Debug, Clone)]
pub struct Band {
    /// The normalized value of the band, where the maximum is 1.0.
    pub value: f64,
    pub color: Color,
}

impl Widget for RTA {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let areas = Layout::horizontal(vec![Constraint::Length(1); self.bands.len()]).split(area);
        for (band, area) in zip(self.bands, areas.iter()) {
            band.render(*area, buf);
        }
    }
}

impl Band {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let value = self.value.clamp(0.0, 1.0);

        let scaled = value * (area.height - 4) as f64;
        let full_blocks = scaled.floor() as u16;
        let fraction = scaled - full_blocks as f64;

        let partial_block = match fraction {
            f if f >= 7.0 / 8.0 => ratatui::symbols::bar::SEVEN_EIGHTHS,
            f if f >= 3.0 / 4.0 => ratatui::symbols::bar::THREE_QUARTERS,
            f if f >= 5.0 / 8.0 => ratatui::symbols::bar::FIVE_EIGHTHS,
            f if f >= 1.0 / 2.0 => ratatui::symbols::bar::HALF,
            f if f >= 3.0 / 8.0 => ratatui::symbols::bar::THREE_EIGHTHS,
            f if f >= 1.0 / 4.0 => ratatui::symbols::bar::ONE_QUARTER,
            f if f >= 1.0 / 8.0 => ratatui::symbols::bar::ONE_EIGHTH,
            _ => "",
        };

        for i in 0..full_blocks {
            buf[(area.left(), 4 + area.bottom().saturating_sub(i + 1))]
                .set_fg(self.color)
                .set_symbol(ratatui::symbols::bar::FULL);
        }
        if !partial_block.is_empty() {
            let partial_y = (area.bottom() + 3).saturating_sub(full_blocks);
            buf[(area.left(), partial_y)]
                .set_fg(self.color)
                .set_symbol(partial_block);
        }
    }
}
