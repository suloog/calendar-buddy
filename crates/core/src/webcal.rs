//! Calendar subscription links (`US-07-06`, `DEC-46`): a `webcal:` link opened anywhere in the
//! system launches the app with it, and the app opens Google Calendar's page for adding a
//! calendar from that address; the subscription comes back through synchronization.

use std::path::Path;

use crate::background::exec_argument;

/// The `webcal:` link the app was launched with, if any.
pub fn link(args: &[String]) -> Option<&str> {
    args.iter().skip(1).map(String::as_str).find(|arg| {
        arg.get(..7)
            .is_some_and(|scheme| scheme.eq_ignore_ascii_case("webcal:"))
    })
}

/// The desktop entry that hands `webcal:` links to the app at `exe` (`x-scheme-handler/webcal`),
/// kept out of the desktop's menus.
pub fn handler_entry(exe: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName=Calendar Buddy\nIcon=x-office-calendar\n\
         Exec={} %u\nMimeType=x-scheme-handler/webcal;\nNoDisplay=true\n",
        exec_argument(&exe.to_string_lossy())
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string()).collect()
    }

    #[test]
    fn the_app_launched_with_a_webcal_link_takes_it() {
        let launched = args(&["calendar-buddy", "webcal://example.com/holidays.ics"]);
        assert_eq!(link(&launched), Some("webcal://example.com/holidays.ics"));
        let upper = args(&["calendar-buddy", "WEBCAL://example.com/a.ics"]);
        assert_eq!(link(&upper), Some("WEBCAL://example.com/a.ics"));
    }

    #[test]
    fn the_app_launched_without_a_webcal_link_takes_none() {
        assert_eq!(link(&args(&["calendar-buddy"])), None);
        assert_eq!(link(&args(&["calendar-buddy", "--in-tray"])), None);
        assert_eq!(
            link(&args(&["calendar-buddy", "https://example.com/a.ics"])),
            None
        );
        assert_eq!(link(&args(&["webcal://example.com/a.ics"])), None);
    }

    #[test]
    fn the_handler_entry_hands_webcal_links_to_the_app() {
        let entry = handler_entry(Path::new("/usr/bin/calendar-buddy"));
        assert_eq!(
            entry,
            "[Desktop Entry]\nType=Application\nName=Calendar Buddy\nIcon=x-office-calendar\n\
             Exec=\"/usr/bin/calendar-buddy\" %u\nMimeType=x-scheme-handler/webcal;\n\
             NoDisplay=true\n"
        );
    }
}
