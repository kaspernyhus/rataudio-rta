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

        let db_axis_width = if self.min_db > -100.0 { 3 } else { 4 };

        // left_area is the dB axis, right_area holds the RTA area and the frequency axis.
        let [left_area, right_area] =
            Layout::horizontal([Constraint::Length(db_axis_width), Constraint::Fill(0)])
                .areas(rta_area);

        // db axis must start one block above the bottom to align with frequency axis.
        let [db_axis, _] =
            Layout::vertical([Constraint::Fill(0), Constraint::Length(2)]).areas(left_area);

        let [rta_area, freq_axis] =
            Layout::vertical([Constraint::Fill(0), Constraint::Length(1)]).areas(right_area);

        let num_bands = self.bands.len() as u16;
        if num_bands == 0 {
            panic!("No bands configured — cannot continue");
        }

        // The min bar_width is 1
        let bar_width = ((rta_area.width - 1) / num_bands).clamp(1, rta_area.width);

        let axis = Block::default()
            .borders(Borders::LEFT | Borders::BOTTOM)
            .border_style(Color::White);

        let bands_area = axis.inner(rta_area);
        let bands_area_width = bar_width * num_bands;

        // Render the x-axis and frequency labels only as wide as the bars area
        axis.render(
            Rect {
                width: bands_area_width + 1,
                ..rta_area
            },
            buf,
        );

        let rta_bands = Layout::horizontal(vec![Constraint::Length(bar_width); num_bands as usize])
            .split(bands_area);

        self.render_db_scale(db_axis, buf);

        let freq_axis = Rect {
            width: bands_area_width + 1,
            ..freq_axis
        };
        self.render_freq_scale(freq_axis, bar_width, buf);

        for (band, area) in zip(self.bands, rta_bands.iter()) {
            band.render(*area, bar_width, buf);
        }
    }
}

impl RTA<'_> {
    fn render_db_scale(&self, area: Rect, buf: &mut Buffer) {
        // Render a label for each 3rd line
        let num_labels = (area.height as u32) / 3;

        let layout = Layout::vertical(vec![
            Constraint::Ratio(1, num_labels);
            num_labels.try_into().unwrap()
        ]);
        let label_areas = layout.split(area);

        let label_value_delta = -self.min_db / num_labels as f32;

        for (i, label_area) in label_areas.iter().enumerate() {
            let db_value = 0.0 - (label_value_delta * i as f32);
            let label_text = format!("{:.0}", db_value);
            Paragraph::new(label_text)
                .alignment(Alignment::Right)
                .render(*label_area, buf);
        }
    }

    fn format_frequency_label(freq: u16) -> String {
        if freq >= 10000 {
            let label = format!("{:.0}", freq as f64 / 1000.0);
            format!("{}k", label)
        } else if freq >= 1000 {
            let label = format!("{:.1}", freq as f64 / 1000.0);
            if label.ends_with(".0") {
                format!("{}k", label.trim_end_matches(".0"))
            } else {
                format!("{}k", label)
            }
        } else {
            format!("{}", freq)
        }
    }

    fn render_freq_scale(&self, area: Rect, bar_width: u16, buf: &mut Buffer) {
        // skip the first char position where the dB axis starts
        let [_, label_area] =
            Layout::horizontal([Constraint::Length(1), Constraint::Fill(0)]).areas(area);

        // Decide the spacing between labels based on the bar width.
        let label_spacing_bars = if bar_width > 3 {
            2
        } else if bar_width > 2 {
            4
        } else {
            6
        };

        let label_width = label_spacing_bars * bar_width;
        let num_labels = (label_area.width - (label_spacing_bars * bar_width).max(9)) / label_width;

        let mut constraints = vec![Constraint::Length(label_width); num_labels as usize];
        constraints.push(Constraint::Fill(0));

        let labels_area = Layout::horizontal(constraints).split(label_area);

        for (i, label_area) in labels_area.iter().enumerate() {
            let band_index = i * label_spacing_bars as usize;
            let freq = self.bands[band_index].frequency.unwrap_or(0);
            Paragraph::new(Self::format_frequency_label(freq))
                .alignment(Alignment::Left)
                .render(*label_area, buf);
        }

        // Render the last label on the right side of the last area.
        let freq = self.bands[self.bands.len() - 1].frequency.unwrap_or(0);
        Paragraph::new(Self::format_frequency_label(freq))
            .alignment(Alignment::Right)
            .render(labels_area[labels_area.len() - 1], buf);
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

    fn render_peak_labels(&self, area: Rect, buf: &mut Buffer) {
        let peak_band = self.get_peak_band().unwrap_or(Band::new(-60.0, 20));
        let peak_db_value = peak_band.get_db(self.min_db);

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
