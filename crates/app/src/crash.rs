//! When the editor crashes: what happened is written to a file (`crash.txt`, next to the
//! settings) before it closes, and on the next start the start screen says so and offers to report
//! it (a GitHub issue filled in with it, see `report.rs`) or to dismiss it.
//!
//! The report holds the editor's version, the system, the error and where in the code it
//! happened. Paths under the user's home folder are written from `~`, so the user's name doesn't
//! go in it; the issue still asks the reporter to check it before posting.

use std::path::{Path, PathBuf};

use crate::APP_NAME;

/// Where the last crash is written.
fn file() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join(APP_NAME).join("crash.txt"))
}

/// From now on, a crash is written down before the editor closes. The usual message (on the
/// console, in development) still follows.
pub fn install() {
    let usual = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let message = match info.payload().downcast_ref::<&str>() {
            Some(text) => (*text).to_string(),
            None => info
                .payload()
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_else(|| "(no message)".to_string()),
        };
        let place = info
            .location()
            .map(|place| format!("{}:{}", place.file(), place.line()))
            .unwrap_or_default();
        let backtrace = std::backtrace::Backtrace::force_capture().to_string();
        let text = describe(&message, &place, &backtrace, dirs::home_dir().as_deref());
        if let Some(file) = file() {
            // Nothing more can be done about a crash report that can't be written.
            let _ = file.parent().map(std::fs::create_dir_all);
            let _ = std::fs::write(&file, text);
        }
        usual(info);
    }));
}

/// The report: version, system, what went wrong and where, then the backtrace; paths under
/// `home` written from `~`.
fn describe(message: &str, place: &str, backtrace: &str, home: Option<&Path>) -> String {
    let text = format!(
        "{APP_NAME} {} on {} ({})\n{message}\nat {place}\n\n{backtrace}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
    );
    let Some(home) = home.map(|home| home.to_string_lossy().into_owned()) else {
        return text;
    };
    if home.is_empty() {
        return text;
    }
    // Both ways Windows writes a path.
    text.replace(&home, "~")
        .replace(&home.replace('\\', "/"), "~")
}

/// The last crash's report, if the editor crashed since it was last dismissed.
pub fn last() -> Option<String> {
    if cfg!(test) {
        return None;
    }
    std::fs::read_to_string(file()?).ok()
}

/// Forgets the last crash (once reported or dismissed).
pub fn forget() {
    if let Some(file) = file() {
        let _ = std::fs::remove_file(file);
    }
}

/// The report's first line that says what went wrong, for the issue's title.
pub fn summary(report: &str) -> String {
    report
        .lines()
        .nth(1)
        .unwrap_or("A crash")
        .chars()
        .take(80)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_paths_from_home_without_the_users_name() {
        let report = describe(
            "index out of bounds",
            "crates/app/src/x.rs:10",
            "at C:\\Users\\anna\\x.rs\nat C:/Users/anna/y.rs",
            Some(Path::new("C:\\Users\\anna")),
        );
        assert!(!report.contains("anna"), "{report}");
        assert!(report.contains("~\\x.rs") && report.contains("~/y.rs"));
        assert_eq!(summary(&report), "index out of bounds");
    }
}
