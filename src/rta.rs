use std::iter::zip;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::BlockExt,
    style::Color,
    widgets::{Block, Borders, Paragraph, Widget},
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
pub struct RTA<'a> {
    pub(crate) block: Option<Block<'a>>,
    pub(crate) bands: Vec<Band>,
}

/// A struct representing a single frequency band in the equalizer.
#[derive(Debug, Clone)]
pub struct Band {
    /// The normalized value of the band, where the maximum is 1.0.
    pub value: f64,
    /// The color of the band.
    pub color: Color,
    /// Frequency band label, if any.
    pub frequency: Option<u16>,
}

impl<'a> Widget for RTA<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(block) = self.block.as_ref() {
            block.render(area, buf);
        }

        let rta_area = self.block.inner_if_some(area);
        if rta_area.is_empty() {
            return;
        }

        let [left_area, right_area] =
            Layout::horizontal([Constraint::Length(3), Constraint::Fill(0)]).areas(rta_area);

        let [db_axis, _] =
            Layout::vertical([Constraint::Fill(0), Constraint::Length(1)]).areas(left_area);

        let [rta_area, freq_axis] =
            Layout::vertical([Constraint::Fill(0), Constraint::Length(1)]).areas(right_area);

        let borders = Block::new().borders(Borders::LEFT | Borders::BOTTOM);
        let bars_area = borders.inner(rta_area);
        borders.render(rta_area, buf);

        let rta_bands =
            Layout::horizontal(vec![Constraint::Length(1); self.bands.len()]).split(bars_area);

        // Render dB labels
        self.render_db_labels(db_axis, buf);

        // Render frequency axis labels
        self.render_freq_labels(freq_axis, buf);

        for (band, area) in zip(self.bands, rta_bands.iter()) {
            band.render(*area, buf);
        }
    }
}

impl Band {
    pub fn new(value: f64, frequency: u16) -> Self {
        Band {
            value,
            color: Color::Yellow,
            frequency: Some(frequency),
        }
    }

    pub fn set_value(&mut self, value: f64) {
        self.value = value;
    }

    fn render(self, area: Rect, buf: &mut Buffer) {
        let value = self.value.clamp(0.0, 1.0);

        let scaled = value * area.height as f64;
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
            buf[(area.left(), area.bottom().saturating_sub(i + 1))]
                .set_fg(self.color)
                .set_symbol(ratatui::symbols::bar::FULL);
        }
        if !partial_block.is_empty() {
            let partial_y = area.bottom().saturating_sub(full_blocks + 1);
            buf[(area.left(), partial_y)]
                .set_fg(self.color)
                .set_symbol(partial_block);
        }
    }
}

impl<'a> RTA<'a> {
    /// Creates a new `RTA` widget with the given bands.
    pub fn new(bands: Vec<Band>) -> Self {
        RTA { block: None, bands }
    }

    /// Highlights the band with the maximum value by changing its color to red.
    pub fn highlight_peak_band(mut self) -> Self {
        if let Some((max_index, _)) = self.bands.iter().enumerate().max_by(|(_, a), (_, b)| {
            a.value
                .partial_cmp(&b.value)
                .unwrap_or(std::cmp::Ordering::Equal)
        }) {
            self.bands[max_index].color = Color::Red;
        }
        self
    }

    /// Surrounds the `RTA` widget with a [`Block`].
    ///
    /// The meter is rendered in the inner portion of the block once space for borders and padding
    /// is reserved. Styles set on the block do **not** affect the meter itself.
    #[must_use = "method moves the value of self and returns the modified value"]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl RTA<'_> {
    fn render_db_labels(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Constraint::Ratio(1, 7); 7]);
        let areas = layout.split(area);

        Paragraph::new("0")
            .alignment(Alignment::Right)
            .render(areas[0], buf);

        Paragraph::new("-10")
            .alignment(Alignment::Right)
            .render(areas[1], buf);

        Paragraph::new("-20")
            .alignment(Alignment::Right)
            .render(areas[2], buf);

        Paragraph::new("-30")
            .alignment(Alignment::Right)
            .render(areas[3], buf);

        Paragraph::new("-40")
            .alignment(Alignment::Right)
            .render(areas[4], buf);

        Paragraph::new("-50")
            .alignment(Alignment::Right)
            .render(areas[5], buf);

        Paragraph::new("-60")
            .alignment(Alignment::Right)
            .render(areas[6], buf);
    }

    fn format_frequency_label(freq: u16) -> String {
        if freq >= 1000 {
            format!("{:.0}k", freq as f64 / 1000.0)
        } else {
            format!("{:.0}", freq)
        }
    }

    fn render_freq_labels(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::horizontal([Constraint::Ratio(1, 5); 5]);
        let areas = layout.split(area);

        let n_bands = self.bands.len();
        let n_labels = areas.len();
        let n_bands_per_label = n_bands / n_labels;
        let label_positions = Vec::new()
            .into_iter()
            .chain((0..n_labels).map(|i| i * n_bands_per_label))
            .collect::<Vec<_>>();

        for (i, area) in areas.iter().enumerate() {
            let band_index = label_positions[i];
            let freq = self.bands[band_index].frequency.unwrap_or(0);

            Paragraph::new(Self::format_frequency_label(freq))
                .alignment(Alignment::Left)
                .render(*area, buf);
        }

        let freq = self.bands[self.bands.len() - 1].frequency.unwrap_or(0);
        Paragraph::new(Self::format_frequency_label(freq))
            .alignment(Alignment::Right)
            .render(areas[areas.len() - 1], buf);
    }
}
