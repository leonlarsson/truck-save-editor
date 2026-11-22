//! Whether a newer version of the editor is out. When the editor starts (unless turned off in
//! Settings) it asks GitHub for the repository's latest release, and if that's newer than this
//! build, the start screen, the sidebar and Settings say so, with a link to download it.
//!
//! This is the only time the editor uses the network: one request to GitHub's API, which sees the
//! computer's address like any download does, and an agent string with the editor's name and
//! version. Nothing about the player or their saves is sent. Updating is still downloading the
//! new .exe and using it in place of the old one; settings and backups live elsewhere and stay.

use std::time::Duration;

use gpui_kit::{App, AppContext as _, Global, SharedString};

use crate::APP_NAME;

/// What's known about a newer version.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum UpdateState {
    /// Not asked yet (or checking is turned off).
    #[default]
    NotChecked,
    Checking,
    /// This is the latest version (or there's no release yet).
    UpToDate,
    /// A newer version is out: its number and its release page.
    Available {
        version: String,
        url: String,
    },
    /// GitHub couldn't be reached (offline, a firewall); try again from Settings.
    Failed,
}

struct Updates(UpdateState);

impl Global for Updates {}

/// This build's version.
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Checks for a newer version when the editor starts, if the player hasn't turned it off.
pub fn init(cx: &mut App) {
    cx.set_global(Updates(UpdateState::NotChecked));
    // Development builds: `TSE_FAKE_UPDATE=0.9.0` pretends that version is out, to see how it
    // shows without a release.
    #[cfg(debug_assertions)]
    if let Ok(version) = std::env::var("TSE_FAKE_UPDATE") {
        let url = format!("https://github.com/{}/releases", repository());
        cx.set_global(Updates(UpdateState::Available { version, url }));
        return;
    }
    if crate::settings::get(cx).check_for_updates {
        check(cx);
    }
}

/// What's known now.
pub fn state(cx: &App) -> UpdateState {
    cx.try_global::<Updates>()
        .map(|updates| updates.0.clone())
        .unwrap_or_default()
}

/// The newer version and its page, when there is one.
pub fn available(cx: &App) -> Option<(SharedString, String)> {
    match state(cx) {
        UpdateState::Available { version, url } => Some((version.into(), url)),
        _ => None,
    }
}

/// Asks GitHub for the latest release, in the background.
pub fn check(cx: &mut App) {
    if state(cx) == UpdateState::Checking {
        return;
    }
    cx.set_global(Updates(UpdateState::Checking));
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        let result = cx.background_spawn(async { latest_release() }).await;
        let state = match result {
            Ok(Some((version, url))) if is_newer(&version, CURRENT_VERSION) => {
                UpdateState::Available { version, url }
            }
            Ok(_) => UpdateState::UpToDate,
            Err(_) => UpdateState::Failed,
        };
        cx.update(|cx| {
            cx.set_global(Updates(state));
            cx.refresh_windows();
        })
    })
    .detach();
}

/// `owner/name` of the repository, from Cargo.toml's `repository`.
fn repository() -> &'static str {
    env!("CARGO_PKG_REPOSITORY")
        .trim_start_matches("https://github.com/")
        .trim_end_matches('/')
}

/// The latest published release's version (without the `v`) and page; `None` when there's no
/// release yet.
fn latest_release() -> Result<Option<(String, String)>, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        // A 404 (no release yet) is an answer, not an error.
        .http_status_as_error(false)
        .build()
        .into();
    let url = format!(
        "https://api.github.com/repos/{}/releases/latest",
        repository()
    );
    let user_agent = format!("{}/{CURRENT_VERSION}", APP_NAME.replace(' ', "-"));
    let mut response = agent
        .get(&url)
        .header("User-Agent", &user_agent)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|error| error.to_string())?;
    match response.status().as_u16() {
        200 => {}
        404 => return Ok(None),
        status => return Err(format!("GitHub answered {status}")),
    }
    let body = response
        .body_mut()
        .read_to_string()
        .map_err(|error| error.to_string())?;
    parse_release(&body)
}

/// The version and page from GitHub's release JSON.
fn parse_release(json: &str) -> Result<Option<(String, String)>, String> {
    let release: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let tag = release["tag_name"].as_str().unwrap_or_default();
    let url = release["html_url"].as_str().unwrap_or_default();
    if tag.is_empty() || url.is_empty() {
        return Ok(None);
    }
    Ok(Some((
        tag.trim_start_matches('v').to_string(),
        url.to_string(),
    )))
}

/// Whether `latest` is a newer version than `current` (`0.2.0` > `0.1.5`; a pre-release such as
/// `0.2.0-beta.1` comes before `0.2.0`).
pub fn is_newer(latest: &str, current: &str) -> bool {
    fn parts(version: &str) -> ([u64; 3], bool) {
        let version = version.trim_start_matches('v');
        let (numbers, pre_release) = match version.split_once('-') {
            Some((numbers, _)) => (numbers, true),
            None => (version, false),
        };
        let mut parsed = [0; 3];
        for (slot, part) in parsed.iter_mut().zip(numbers.split('.')) {
            *slot = part.parse().unwrap_or(0);
        }
        (parsed, pre_release)
    }
    let (latest_numbers, latest_is_pre) = parts(latest);
    let (current_numbers, current_is_pre) = parts(current);
    latest_numbers > current_numbers
        || (latest_numbers == current_numbers && current_is_pre && !latest_is_pre)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(is_newer("0.2.0", "0.1.9"));
        assert!(is_newer("v1.0.0", "0.9.0"));
        assert!(is_newer("0.10.0", "0.9.0"), "numbers, not text");
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
        assert!(
            is_newer("0.2.0", "0.2.0-beta.1"),
            "the release after its beta"
        );
        assert!(!is_newer("0.2.0-beta.1", "0.2.0"));
    }

    #[test]
    fn reads_the_release() {
        let json = r#"{"tag_name": "v0.2.0", "html_url": "https://github.com/o/r/releases/tag/v0.2.0", "draft": false}"#;
        assert_eq!(
            parse_release(json),
            Ok(Some((
                "0.2.0".to_string(),
                "https://github.com/o/r/releases/tag/v0.2.0".to_string()
            )))
        );
        assert_eq!(parse_release("{}"), Ok(None));
        assert!(parse_release("not json").is_err());
    }

    #[test]
    fn knows_the_repository() {
        assert!(repository().contains('/') && !repository().starts_with("http"));
    }
}
