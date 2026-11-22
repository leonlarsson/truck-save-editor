//! Reporting a problem: opens a new issue on the project's GitHub page in the browser, filled in
//! with what helps (the editor's version, the system, the language, and the error when there was
//! one). Nothing is sent from here: the player reads the issue, changes it, and posts it themselves.
//!
//! The issue text is in English and lives here rather than in the locale files: it's written for
//! the people fixing the problem, on GitHub, not shown in the editor. Nothing personal goes in it by
//! default (no paths, profile or save names); an error message can name a file, which is why the
//! text asks the reporter to check it before posting.

use gpui_kit::component::{
    Sizable as _,
    button::Button,
    notification::{Notification, NotificationType},
};
use gpui_kit::{App, SharedString};

use crate::i18n::t;

/// The GitHub page for a new issue, filled in. `error` is what went wrong, when an error message
/// offered the report: a short summary (the title) and the details.
pub fn issue_url(cx: &App, error: Option<(&str, &str)>) -> String {
    let title = error.map(|(summary, _)| summary).unwrap_or_default();
    let mut body = String::from(
        "**What happened**\n\n<!-- What did you do, and what went wrong? Screenshots help. -->\n\n\n\
         **What you expected**\n\n\n",
    );
    if let Some((_, details)) = error {
        body.push_str("**The error**\n\n```\n");
        body.push_str(&shortened(details, 1500));
        body.push_str("\n```\n\n");
    }
    body.push_str(&format!(
        "**About**\n\n- {} {}\n- {} ({})\n- Language: {}\n- Game data found: {}\n\n\
         <!-- Check the text above before posting: it shouldn't need anything personal. -->\n",
        crate::APP_NAME,
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        crate::settings::get(cx).language.code(),
        if crate::game_data::install_dir(cx).is_some() {
            "yes"
        } else {
            "no"
        },
    ));
    format!(
        "{}/issues/new?title={}&body={}",
        env!("CARGO_PKG_REPOSITORY"),
        encode(title),
        encode(&body)
    )
}

/// Opens a new issue about a crash, with its report (see `crash.rs`).
pub fn open_crash(cx: &mut App, report: &str) {
    let title = format!("The editor crashed: {}", crate::crash::summary(report));
    let url = issue_url(cx, Some((&title, report)));
    cx.open_url(&url);
}

/// Opens the new issue in the browser.
pub fn open(cx: &mut App, error: Option<(&str, &str)>) {
    let url = issue_url(cx, error);
    cx.open_url(&url);
}

/// An error message, with a button to report it unless it's one the player can fix (closing the
/// game first). `issue_title` names the problem in English for the issue, without the save's name.
pub fn error_notification(
    title: SharedString,
    error: &save::Error,
    issue_title: &'static str,
) -> Notification {
    let details = error.to_string();
    let notification = Notification::new()
        .with_type(NotificationType::Error)
        .title(title)
        .message(details.clone());
    if matches!(error, save::Error::GameRunning) {
        return notification;
    }
    notification.action(move |_, _, cx| {
        let details = details.clone();
        Button::new("report-problem")
            .small()
            .outline()
            .label(t!(cx, "report-this"))
            .on_click(move |_, _, cx| open(cx, Some((issue_title, &details))))
    })
}

/// At most `limit` characters (a link can only be so long), saying when there was more.
fn shortened(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut short: String = text.chars().take(limit).collect();
    short.push_str("\n…");
    short
}

/// Text made safe for a link: letters, digits and `-._~` as they are, every other byte as `%XX`.
fn encode(text: &str) -> String {
    let mut encoded = String::with_capacity(text.len() * 3);
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_text_for_a_link() {
        assert_eq!(encode("a b&c=ü\n"), "a%20b%26c%3D%C3%BC%0A");
        assert_eq!(encode("v0.1-x_y~"), "v0.1-x_y~");
    }

    #[test]
    fn shortens_long_details() {
        assert_eq!(shortened("abc", 5), "abc");
        assert_eq!(shortened("abcdef", 3), "abc\n…");
    }
}
