//! A description written in Google Calendar's web app, which Google keeps as HTML: read into
//! lines of text with their bold, italics, underlines and links, to be shown formatted, or as
//! plain text to be edited (`MOD-05-F30`, `MOD-05-F31`).

/// A run of text and how it is written.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    /// The web page it opens in the browser.
    pub link: Option<String>,
}

/// What a tag does, among those Google's editor writes.
enum Tag {
    Bold,
    Italic,
    Underline,
    Link,
    /// Starts a new line: a line break, a paragraph, a block, a list.
    Break,
    /// A list item: a new line with a bullet.
    Item,
}

fn tag(name: &str) -> Option<Tag> {
    Some(match name {
        "b" | "strong" => Tag::Bold,
        "i" | "em" => Tag::Italic,
        "u" => Tag::Underline,
        "a" => Tag::Link,
        "br" | "p" | "div" | "ul" | "ol" => Tag::Break,
        "li" => Tag::Item,
        _ => return None,
    })
}

/// The tags of `text`, in order: where each starts and ends, its name, whether it closes, and
/// its `href`.
fn tags(text: &str) -> Vec<(usize, usize, String, bool, Option<String>)> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(open) = text[from..].find('<').map(|at| from + at) {
        let Some(close) = text[open..].find('>').map(|at| open + at) else {
            break;
        };
        let inside = &text[open + 1..close];
        let closing = inside.starts_with('/');
        let name = (inside.trim_start_matches('/').split([' ', '/', '\n']))
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        if tag(&name).is_some() {
            let href = inside.split_once("href=").and_then(|(_, rest)| {
                let quote = rest.chars().next().filter(|c| *c == '"' || *c == '\'')?;
                rest[1..].split(quote).next().map(unescape)
            });
            found.push((open, close + 1, name, closing, href));
        }
        from = close + 1;
    }
    found
}

/// Whether `text` was written with formatting: it holds a tag Google's editor writes.
pub fn is_formatted(text: &str) -> bool {
    !tags(text).is_empty()
}

fn unescape(text: &str) -> String {
    [
        ("&nbsp;", " "),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#39;", "'"),
    ]
    .into_iter()
    .fold(text.to_string(), |text, (entity, character)| {
        text.replace(entity, character)
    })
    .replace("&amp;", "&")
}

/// `text` as lines of spans; text without formatting is its lines as they are. Empty lines at
/// the start and the end are left out, and so are runs of more than one empty line.
pub fn lines(text: &str) -> Vec<Vec<Span>> {
    let formatted = is_formatted(text);
    let mut lines: Vec<Vec<Span>> = vec![Vec::new()];
    let mut style = Span::default();
    let (mut bold, mut italic, mut underline) = (0, 0, 0);
    let add = |lines: &mut Vec<Vec<Span>>, style: &Span, piece: &str| {
        let piece = if formatted {
            unescape(piece)
        } else {
            piece.to_string()
        };
        for (i, part) in piece.split('\n').enumerate() {
            if i > 0 {
                lines.push(Vec::new());
            }
            if !part.is_empty() {
                let span = Span {
                    text: part.to_string(),
                    ..style.clone()
                };
                lines.last_mut().expect("a line").push(span);
            }
        }
    };
    let mut at = 0;
    let found = if formatted { tags(text) } else { Vec::new() };
    for (start, end, name, closing, href) in found {
        // Google's HTML breaks lines with tags; its own new lines are only layout.
        add(&mut lines, &style, &text[at..start].replace('\n', ""));
        at = end;
        let step = |count: &mut i32| *count = (*count + if closing { -1 } else { 1 }).max(0);
        match tag(&name).expect("a known tag") {
            Tag::Bold => step(&mut bold),
            Tag::Italic => step(&mut italic),
            Tag::Underline => step(&mut underline),
            Tag::Link => style.link = if closing { None } else { href },
            Tag::Break => {
                if name == "br" || !lines.last().expect("a line").is_empty() {
                    lines.push(Vec::new());
                }
            }
            Tag::Item if !closing => {
                if !lines.last().expect("a line").is_empty() {
                    lines.push(Vec::new());
                }
                add(&mut lines, &Span::default(), "• ");
            }
            Tag::Item => {}
        }
        (style.bold, style.italic, style.underline) = (bold > 0, italic > 0, underline > 0);
    }
    let rest = &text[at..];
    add(
        &mut lines,
        &style,
        &if formatted {
            rest.replace('\n', "")
        } else {
            rest.to_string()
        },
    );
    // No empty lines around it, and never two in a row.
    let mut kept: Vec<Vec<Span>> = Vec::new();
    for line in lines {
        let blank = line.iter().all(|span| span.text.trim().is_empty());
        let after_blank = kept.last().is_none_or(|last| last.is_empty());
        if blank && after_blank {
            continue;
        }
        kept.push(if blank { Vec::new() } else { line });
    }
    while kept.last().is_some_and(Vec::is_empty) {
        kept.pop();
    }
    kept
}

/// `text` without its formatting: its lines as plain text, links as their text.
pub fn plain(text: &str) -> String {
    (lines(text).iter())
        .map(|line| {
            line.iter()
                .map(|span| span.text.as_str())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(text: &str) -> Span {
        Span {
            text: text.into(),
            ..Span::default()
        }
    }

    #[test]
    fn text_without_formatting_is_its_lines_as_they_are() {
        let text = "Agenda:\n1. Numbers <30 min> & plan\n2. Plan";
        assert!(!is_formatted(text));
        assert_eq!(
            lines(text),
            [
                vec![span("Agenda:")],
                vec![span("1. Numbers <30 min> & plan")],
                vec![span("2. Plan")]
            ]
        );
        assert_eq!(plain(text), text);
    }

    #[test]
    fn googles_html_keeps_its_bold_italics_underlines_and_links() {
        let text = "<b>Agenda</b> for <i>Q4</i><br>See <a href=\"https://docs.google.com/d/1\">\
                    the <u>notes</u></a> &amp; more";
        assert!(is_formatted(text));
        let link = Some("https://docs.google.com/d/1".to_string());
        assert_eq!(
            lines(text),
            [
                vec![
                    Span {
                        bold: true,
                        ..span("Agenda")
                    },
                    span(" for "),
                    Span {
                        italic: true,
                        ..span("Q4")
                    },
                ],
                vec![
                    span("See "),
                    Span {
                        link: link.clone(),
                        ..span("the ")
                    },
                    Span {
                        link,
                        underline: true,
                        ..span("notes")
                    },
                    span(" & more"),
                ],
            ]
        );
        assert_eq!(plain(text), "Agenda for Q4\nSee the notes & more");
    }

    #[test]
    fn paragraphs_and_list_items_start_new_lines() {
        let text = "<p>Bring&nbsp;laptops</p><ul><li>Demo</li><li>Plan</li></ul><br><br><br>End";
        assert_eq!(plain(text), "Bring laptops\n• Demo\n• Plan\n\nEnd");
    }

    #[test]
    fn a_description_of_only_tags_has_no_lines() {
        assert_eq!(lines("<br><br>"), Vec::<Vec<Span>>::new());
        assert_eq!(lines(""), Vec::<Vec<Span>>::new());
    }
}
