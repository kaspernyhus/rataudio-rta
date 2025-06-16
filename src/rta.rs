use ratatui::{style::Color, widgets::Block};

/// A widget to display an RTA audio meter.
///
/// A `RTA` renders a number of bars filled according to the value given to each `Band` in the `bands` vector.
#[derive(Debug, Clone)]
pub struct RTA<'a> {
    /// The block that surrounds the RTA widget, if any.
    pub(crate) block: Option<Block<'a>>,
    /// The frequency bands that make up the RTA meter.
    pub(crate) bands: Vec<Band>,
    /// Whether to show the peak labels at the top of the meter.
    pub(crate) show_peak_labels: bool,
    pub min_db: f32,
}

/// A struct representing a single frequency band in the RTA meter.
#[derive(Debug, Clone)]
pub struct Band {
    /// The normalized value of the band, where the maximum is 1.0.
    pub value: f32,
    /// The color of the band.
    pub color: Color,
    /// Frequency band label, if any. Used for rendering frequency labels.
    pub frequency: Option<u16>,
}

impl Band {
    pub fn new(value: f32, frequency: u16) -> Self {
        Band {
            value,
            color: Color::Yellow,
            frequency: Some(frequency),
        }
    }

    /// Sets the value of the band as a ratio between 0.0 and 1.0.
    pub fn set_ratio(&mut self, value: f32) {
        self.value = value;
    }

    /// Set the value of the band in decibels.
    pub fn set_db(&mut self, db: f32, min_db: f32) {
        if db <= min_db {
            self.value = 0.0;
            return;
        }
        if db >= 0.0 {
            self.value = 1.0;
            return;
        }
        let db = db.clamp(min_db, 0.0);
        let db_ratio = 10_f32.powf(db / 20.0);
        let min_db_ratio = 10_f32.powf(min_db / 20.0);
        let linear_ratio = (db_ratio.log10() - min_db_ratio.log10()) / (0.0 - min_db_ratio.log10());
        self.value = linear_ratio;
    }

    /// Get the value of the band in decibels.
    pub fn get_db(&self, min_db: f32) -> f32 {
        let min_db_ratio = 10_f32.powf(min_db / 20.0);
        let db_ratio =
            10_f32.powf(self.value * (0.0 - min_db_ratio.log10()) + min_db_ratio.log10());
        20.0 * db_ratio.log10()
    }
}

impl<'a> RTA<'a> {
    /// Creates a new `RTA` widget with the given bands.
    pub fn new(bands: Vec<Band>, min_db: f32) -> Self {
        RTA {
            block: None,
            bands,
            show_peak_labels: true,
            min_db,
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
    pub fn show_peak_labels(mut self, show: bool) -> Self {
        self.show_peak_labels = show;
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
