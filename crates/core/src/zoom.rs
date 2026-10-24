//! The height of the hours in the day, 3-day and week views (`MOD-03-F27`, `MOD-03-F29`,
//! `MOD-03-U08`, `DEC-166`): a factor on their usual height, from 50 % to 300 %.

/// Shortest the hours get.
pub const MIN: f32 = 0.5;
/// Tallest the hours get.
pub const MAX: f32 = 3.0;
/// How far a key moves them.
pub const STEP: f32 = 0.25;
/// How near a step, in steps, a height counts as being at it.
const NEAR: f32 = 0.001;

/// How tall the hours are, as a factor on their usual height; the usual height by default.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HourZoom(f32);

impl Default for HourZoom {
    fn default() -> Self {
        HourZoom(1.0)
    }
}

impl HourZoom {
    /// `factor`, kept between 50 % and 300 %.
    pub fn new(factor: f32) -> Self {
        if factor.is_nan() {
            return HourZoom::default();
        }
        HourZoom(factor.clamp(MIN, MAX))
    }

    pub fn factor(self) -> f32 {
        self.0
    }

    /// Zoomed by `delta`, as a wheel or a pinch does.
    pub fn times(self, delta: f32) -> Self {
        HourZoom::new(self.0 * delta)
    }

    /// One step of the keys taller: to the next multiple of 25 % (`DEC-168`).
    pub fn larger(self) -> Self {
        HourZoom::new(((self.0 / STEP + NEAR).floor() + 1.0) * STEP)
    }

    /// One step of the keys shorter: to the multiple of 25 % below.
    pub fn smaller(self) -> Self {
        HourZoom::new(((self.0 / STEP - NEAR).ceil() - 1.0) * STEP)
    }

    /// How it is kept in the settings file; the usual height is not kept.
    pub fn key(self) -> Option<String> {
        (self != HourZoom::default()).then(|| self.0.to_string())
    }

    pub fn from_key(key: &str) -> Self {
        key.parse()
            .map_or_else(|_| HourZoom::default(), HourZoom::new)
    }
}

/// How far down the hours are scrolled for the time `anchor` points down a view `view` points
/// high to stay where it is, when the hours, scrolled by `offset`, go from `from` to `to` points
/// an hour; kept within the day.
pub fn anchored(offset: f32, anchor: f32, from: f32, to: f32, view: f32) -> f32 {
    let bottom = (24.0 * to - view).max(0.0);
    ((offset + anchor) * to / from - anchor).clamp(0.0, bottom)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The minute of the day `y` points down a view scrolled by `offset`, at `hour` points an
    /// hour.
    fn minute_at(y: f32, offset: f32, hour: f32) -> f32 {
        (offset + y) / hour * 60.0
    }

    #[test]
    fn tc_03_13_the_time_under_the_pointer_stays_there() {
        // 14:00 under the pointer, 300 points down a view 500 high, at 44 points an hour.
        let (hour, view, anchor) = (44.0, 500.0, 300.0);
        let offset = 14.0 * hour - anchor;
        for to in [hour * 1.1, hour * 2.0, hour * 0.8] {
            let scrolled = anchored(offset, anchor, hour, to, view);
            assert!((minute_at(anchor, scrolled, to) - 14.0 * 60.0).abs() < 0.01);
        }
    }

    #[test]
    fn tc_03_13_the_view_stays_within_the_day() {
        let (hour, view) = (44.0, 300.0);
        // Near midnight at the top: the hours cannot scroll above 0:00.
        assert_eq!(anchored(10.0, 0.0, hour, hour * 0.5, view), 5.0);
        assert_eq!(anchored(10.0, 150.0, hour, hour * 0.5, view), 0.0);
        // Nor past 24:00 at the bottom.
        let bottom = 24.0 * hour * 2.0 - view;
        assert_eq!(
            anchored(bottom, 150.0, hour * 2.0, hour, view),
            24.0 * hour - view
        );
        // A day shorter than the view is not scrolled at all.
        assert_eq!(anchored(100.0, 450.0, hour, hour * 0.5, 900.0), 0.0);
    }

    #[test]
    fn tc_03_13_the_keys_step_by_25_percent_between_50_and_300() {
        let mut zoom = HourZoom::default();
        assert_eq!(zoom.factor(), 1.0);
        zoom = zoom.larger().larger();
        assert_eq!(zoom.factor(), 1.5);
        for _ in 0..10 {
            zoom = zoom.larger();
        }
        assert_eq!(zoom.factor(), 3.0);
        for _ in 0..20 {
            zoom = zoom.smaller();
        }
        assert_eq!(zoom.factor(), 0.5);
        // From a height between the steps, to the step next to it.
        assert_eq!(HourZoom::new(1.37).larger().factor(), 1.5);
        assert_eq!(HourZoom::new(1.37).smaller().factor(), 1.25);
        assert_eq!(HourZoom::new(1.75).smaller().factor(), 1.5);
        // Not to the step it is already about at.
        assert_eq!(HourZoom::new(1.250_001).larger().factor(), 1.5);
        assert_eq!(HourZoom::new(1.249_999).smaller().factor(), 1.0);
    }

    #[test]
    fn tc_03_13_the_wheel_and_a_pinch_zoom_smoothly_between_50_and_300_percent() {
        assert_eq!(HourZoom::default().times(1.1).factor(), 1.1);
        assert_eq!(HourZoom::new(2.8).times(1.5).factor(), 3.0);
        assert_eq!(HourZoom::new(0.6).times(0.5).factor(), 0.5);
        assert_eq!(HourZoom::new(7.0).factor(), 3.0);
        assert_eq!(HourZoom::new(f32::NAN), HourZoom::default());
    }

    #[test]
    fn tc_03_14_the_height_is_kept_as_it_is() {
        assert_eq!(HourZoom::default().key(), None);
        for factor in [0.5, 1.37, 1.5, 3.0] {
            let zoom = HourZoom::new(factor);
            assert_eq!(HourZoom::from_key(&zoom.key().unwrap()), zoom);
        }
        assert_eq!(HourZoom::new(1.5).key().as_deref(), Some("1.5"));
        assert_eq!(HourZoom::from_key("tall"), HourZoom::default());
        assert_eq!(HourZoom::from_key("9"), HourZoom::new(3.0));
    }
}
