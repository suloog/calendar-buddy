//! How long a notice along the bottom of the window stays before it goes on its own: what could
//! not be saved or changed, an account connected, calendars gone (`DEC-159`).

use std::time::{Duration, Instant};

/// How long a notice stays, long enough to read a reason.
pub const SHOWN_FOR: Duration = Duration::from_secs(10);

/// When a notice of so many messages was first shown; one more message shows it for longer.
#[derive(Debug, Default)]
pub struct Shown {
    since: Option<(Instant, usize)>,
}

impl Shown {
    /// How long the notice of `count` messages stays from `now`; `None` once it has gone, and
    /// while it has no messages.
    pub fn left(&mut self, count: usize, now: Instant) -> Option<Duration> {
        if count == 0 {
            self.since = None;
            return None;
        }
        let since = match self.since {
            Some((since, shown)) if shown == count => since,
            _ => self.since.insert((now, count)).0,
        };
        SHOWN_FOR
            .checked_sub(now.saturating_duration_since(since))
            .filter(|left| !left.is_zero())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_notice_goes_after_ten_seconds_and_a_new_message_shows_it_again() {
        let start = Instant::now();
        let at = |seconds| start + Duration::from_secs(seconds);
        let mut shown = Shown::default();
        assert_eq!(shown.left(0, at(0)), None);
        assert_eq!(shown.left(1, at(0)), Some(SHOWN_FOR));
        assert_eq!(shown.left(1, at(4)), Some(Duration::from_secs(6)));
        // Another message: ten seconds from it.
        assert_eq!(shown.left(2, at(6)), Some(SHOWN_FOR));
        assert_eq!(shown.left(2, at(15)), Some(Duration::from_secs(1)));
        assert_eq!(shown.left(2, at(16)), None);
        // Closed, then a new message.
        assert_eq!(shown.left(0, at(17)), None);
        assert_eq!(shown.left(1, at(20)), Some(SHOWN_FOR));
    }
}
