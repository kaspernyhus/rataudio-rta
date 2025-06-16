use std::iter::zip;

use ratatui::{
    layout::{Alignment, Constraint, Layout},
    prelude::{BlockExt, Buffer, Color, Rect, Widget},
    widgets::{Block, Borders, Paragraph},
};

use crate::rta::{Band, RTA};

impl Band {
    fn render(self, area: Rect, width: u16, buf: &mut Buffer) {
        let value = self.value.clamp(0.0, 1.0);

        let scaled = value * area.height as f32;
        let full_blocks = scaled.floor() as u16;
        let fraction = scaled - full_blocks as f32;

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

impl<'a> Widget for RTA<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(block) = self.block.as_ref() {
            block.render(area, buf);
        }

        let mut rta_area = self.block.inner_if_some(area);
        if rta_area.is_empty() {
            return;
        }

        if self.show_peak_labels {
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

impl RTA<'_> {
    fn render_db_scale(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Constraint::Ratio(1, 7); 7]);
        let labels = layout.split(area);

        Paragraph::new("0")
            .alignment(Alignment::Right)
            .render(labels[0], buf);

        Paragraph::new("-20")
            .alignment(Alignment::Right)
            .render(labels[1], buf);

        Paragraph::new("-40")
            .alignment(Alignment::Right)
            .render(labels[2], buf);

        Paragraph::new("-60")
            .alignment(Alignment::Right)
            .render(labels[3], buf);

        Paragraph::new("-80")
            .alignment(Alignment::Right)
            .render(labels[4], buf);

        Paragraph::new("-100")
            .alignment(Alignment::Right)
            .render(labels[5], buf);

        Paragraph::new("-120")
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
    fn ratio_to_db(&self, ratio: f32) -> f32 {
        let min_db_ratio = 10_f32.powf(self.min_db / 20.0);
        let db_ratio = 10_f32.powf(ratio * (0.0 - min_db_ratio.log10()) + min_db_ratio.log10());
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
