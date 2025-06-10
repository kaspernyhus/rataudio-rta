use std::iter::zip;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::BlockExt,
    style::Color,
    widgets::{Block, Borders, Paragraph, Widget},
};

pub const MIN_DB: f64 = -65.0;

/// A widget to display an RTA audio meter.
///
/// A `RTA` renders a number of bars filled according to the value given to each `Band` in the `bands` vector.
#[derive(Debug, Clone)]
pub struct RTA<'a> {
    pub(crate) block: Option<Block<'a>>,
    pub(crate) bands: Vec<Band>,
    pub(crate) show_labels: bool,
}

/// A struct representing a single frequency band in the RTA meter.
#[derive(Debug, Clone)]
pub struct Band {
    /// The normalized value of the band, where the maximum is 1.0.
    pub value: f64,
    /// The color of the band.
    pub color: Color,
    /// Frequency band label, if any. Used for rendering frequency labels.
    pub frequency: Option<u16>,
}

impl<'a> Widget for RTA<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(block) = self.block.as_ref() {
            block.render(area, buf);
        }

        let mut rta_area = self.block.inner_if_some(area);
        if rta_area.is_empty() {
            return;
        }

        // Render labels at the top of the RTA meter.
        if self.show_labels {
            let [top_area, rest] =
                Layout::vertical([Constraint::Length(2), Constraint::Fill(0)]).areas(rta_area);
            self.render_peak_labels(top_area, buf);
            rta_area = rest;
        }

        // left_area is the dB axis, right_area holds the RTA area and the frequency axis.
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Length(3), Constraint::Fill(0)]).areas(rta_area);

        // db axis must start one block above the bottom to align with frequency axis.
        let [db_axis, _] =
            Layout::vertical([Constraint::Fill(0), Constraint::Length(1)]).areas(left_area);

        let [rta_area, freq_axis] =
            Layout::vertical([Constraint::Fill(0), Constraint::Length(1)]).areas(right_area);

        let num_bands = self.bands.len();
        if num_bands == 0 {
            panic!("No bands configured — cannot continue");
        }

        // The min bar_width is 1
        let bar_width = ((rta_area.width - 1) / num_bands as u16).clamp(1, rta_area.width);

        let axis = Block::default()
            .borders(Borders::LEFT | Borders::BOTTOM)
            .border_style(Color::White);

        let bands_area = axis.inner(rta_area);
        let bands_area_width = bar_width * num_bands as u16;

        axis.render(
            Rect::new(
                rta_area.x,
                rta_area.y,
                bands_area_width + 1,
                rta_area.height,
            ),
            buf,
        );

        let rta_bands =
            Layout::horizontal(vec![Constraint::Length(bar_width); num_bands]).split(bands_area);

        // Render dB scale
        self.render_db_scale(db_axis, buf);

        // Render frequency scale
        self.render_freq_scale(
            Rect::new(freq_axis.x, freq_axis.y, bands_area_width, freq_axis.height),
            buf,
        );

        for (band, area) in zip(self.bands, rta_bands.iter()) {
            band.render(*area, bar_width, buf);
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

    /// Sets the value of the band as a ratio between 0.0 and 1.0.
    pub fn set_ratio(&mut self, value: f64) {
        self.value = value;
    }

    /// Set the value of the band in decibels.
    pub fn set_db(&mut self, db: f64) {
        if db <= MIN_DB {
            self.value = 0.0;
            return;
        }
        if db >= 0.0 {
            self.value = 1.0;
            return;
        }
        let db = db.clamp(MIN_DB, 0.0);
        let db_ratio = 10_f64.powf(db / 20.0);
        let min_db_ratio = 10_f64.powf(MIN_DB / 20.0);
        let linear_ratio = (db_ratio.log10() - min_db_ratio.log10()) / (0.0 - min_db_ratio.log10());
        self.value = linear_ratio;
    }

    /// Get the value of the band in decibels.
    pub fn get_db(&self) -> f64 {
        let min_db_ratio = 10_f64.powf(MIN_DB / 20.0);
        let db_ratio =
            10_f64.powf(self.value * (0.0 - min_db_ratio.log10()) + min_db_ratio.log10());
        20.0 * db_ratio.log10()
    }

    fn render(self, area: Rect, width: u16, buf: &mut Buffer) {
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
            for x in 0..width {
                buf[(area.left() + x, area.bottom().saturating_sub(i + 1))]
                    .set_fg(self.color)
                    .set_symbol(ratatui::symbols::bar::FULL);
            }
        }
        if !partial_block.is_empty() {
            let partial_y = area.bottom().saturating_sub(full_blocks + 1);
            for x in 0..width {
                buf[(area.left() + x, partial_y)]
                    .set_fg(self.color)
                    .set_symbol(partial_block);
            }
        }
    }
}

impl<'a> RTA<'a> {
    /// Creates a new `RTA` widget with the given bands.
    pub fn new(bands: Vec<Band>) -> Self {
        RTA {
            block: None,
            bands,
            show_labels: false,
        }
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

    /// Sets whether to show the peak labels at the top of the meter.
    pub fn show_labels(mut self, show: bool) -> Self {
        self.show_labels = show;
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
    fn render_db_scale(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Constraint::Ratio(1, 7); 7]);
        let labels = layout.split(area);

        Paragraph::new("0")
            .alignment(Alignment::Right)
            .render(labels[0], buf);

        Paragraph::new("-10")
            .alignment(Alignment::Right)
            .render(labels[1], buf);

        Paragraph::new("-20")
            .alignment(Alignment::Right)
            .render(labels[2], buf);

        Paragraph::new("-30")
            .alignment(Alignment::Right)
            .render(labels[3], buf);

        Paragraph::new("-40")
            .alignment(Alignment::Right)
            .render(labels[4], buf);

        Paragraph::new("-50")
            .alignment(Alignment::Right)
            .render(labels[5], buf);

        Paragraph::new("-60")
            .alignment(Alignment::Right)
            .render(labels[6], buf);
    }

    fn format_frequency_label(freq: u16) -> String {
        if freq >= 1000 {
            format!("{:.0}k", freq as f64 / 1000.0)
        } else {
            format!("{:.0}", freq)
        }
    }

    fn render_freq_scale(&self, area: Rect, buf: &mut Buffer) {
        let [_, label_area] =
            Layout::horizontal([Constraint::Length(1), Constraint::Fill(0)]).areas(area);
        let layout = Layout::horizontal([Constraint::Ratio(1, 5); 5]);
        let areas = layout.split(label_area);

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

    /// Get a clone of the band with the highest value.
    fn get_peak_band(&self) -> Option<Band> {
        self.bands
            .iter()
            .max_by(|a, b| {
                a.value
                    .partial_cmp(&b.value)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
    }

    /// Convert a ratio to a decibel value
    fn ratio_to_db(&self, ratio: f64) -> f64 {
        let min_db_ratio = 10_f64.powf(MIN_DB / 20.0);
        let db_ratio = 10_f64.powf(ratio * (0.0 - min_db_ratio.log10()) + min_db_ratio.log10());
        20.0 * db_ratio.log10()
    }

    fn render_peak_labels(&self, area: Rect, buf: &mut Buffer) {
        let peak_band = self.get_peak_band().unwrap_or(Band::new(-60.0, 20));
        let peak_db_value = self.ratio_to_db(peak_band.value);

        let [db_label_area, band_label_area] =
            Layout::vertical([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]).areas(area);

        let peak_db_label =
            Paragraph::new(format!("Peak: {:.2}dB", peak_db_value)).alignment(Alignment::Center);
        let peak_band_label =
            Paragraph::new(format!("Band: {}Hz", peak_band.frequency.unwrap_or(20)))
                .alignment(Alignment::Center);
        peak_db_label.render(db_label_area, buf);
        peak_band_label.render(band_label_area, buf);
    }
}
