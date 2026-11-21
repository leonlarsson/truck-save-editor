//! Every piece of text the app shows comes from a translation file (`locales/<language>.ftl`, in
//! Mozilla's Fluent format), looked up by key with `t!`:
//!
//! ```ignore
//! t!(cx, "save")                                // "Save"
//! t!(cx, "changes-count", count = 3)            // "3 changes"
//! t!(cx, "saved-title", name = slot_name)       // "Saved “Before the long haul”"
//! ```
//!
//! Fluent picks the right plural form and word order for each language, so code never builds
//! sentences out of pieces. English is the only language so far; adding one is adding a file.

use std::borrow::Cow;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use gpui_kit::{App, Global, SharedString};
use unic_langid::LanguageIdentifier;

/// The English messages, compiled in.
const ENGLISH: &str = include_str!("../locales/en.ftl");

/// The messages of the language in use.
pub struct Strings {
    bundle: FluentBundle<FluentResource>,
}

impl Global for Strings {}

impl Strings {
    fn english() -> Self {
        let language: LanguageIdentifier = "en".parse().unwrap_or_default();
        let mut bundle = FluentBundle::new(vec![language]);
        // Fluent wraps arguments in invisible direction marks by default (for mixing
        // right-to-left text); GPUI would draw them, so they're off until a right-to-left
        // language arrives.
        bundle.set_use_isolating(false);
        let resource = match FluentResource::try_new(ENGLISH.to_string()) {
            Ok(resource) => resource,
            // A broken file still gives every message it could read; a test catches the error.
            Err((resource, _)) => resource,
        };
        let _ = bundle.add_resource(resource);
        Self { bundle }
    }

    /// The text for a key. A missing key shows as the key itself, so it's visible, not blank.
    pub fn text(&self, key: &str, args: &[(&str, Arg)]) -> SharedString {
        let Some(pattern) = self.bundle.get_message(key).and_then(|m| m.value()) else {
            return SharedString::from(key.to_string());
        };
        let mut fluent_args = FluentArgs::new();
        for (name, value) in args {
            fluent_args.set(*name, value.to_fluent());
        }
        let mut errors = Vec::new();
        let text = self
            .bundle
            .format_pattern(pattern, Some(&fluent_args), &mut errors);
        SharedString::from(text.into_owned())
    }

    #[cfg(test)]
    pub fn has(&self, key: &str) -> bool {
        self.bundle.has_message(key)
    }
}

/// An argument to a message: a number (which also picks the plural form) or text.
pub enum Arg {
    Number(i64),
    Text(String),
}

impl Arg {
    fn to_fluent(&self) -> FluentValue<'_> {
        match self {
            Arg::Number(number) => FluentValue::from(*number),
            Arg::Text(text) => FluentValue::String(Cow::Borrowed(text)),
        }
    }
}

macro_rules! impl_number_arg {
    ($($t:ty),*) => {$(
        impl From<$t> for Arg {
            fn from(value: $t) -> Self {
                Arg::Number(value as i64)
            }
        }
    )*};
}
impl_number_arg!(i64, i32, u32, u8, usize);

impl From<String> for Arg {
    fn from(value: String) -> Self {
        Arg::Text(value)
    }
}

impl From<&str> for Arg {
    fn from(value: &str) -> Self {
        Arg::Text(value.to_string())
    }
}

impl From<&String> for Arg {
    fn from(value: &String) -> Self {
        Arg::Text(value.clone())
    }
}

impl From<SharedString> for Arg {
    fn from(value: SharedString) -> Self {
        Arg::Text(value.to_string())
    }
}

impl From<&SharedString> for Arg {
    fn from(value: &SharedString) -> Self {
        Arg::Text(value.to_string())
    }
}

/// Loads the language. English only, for now.
pub fn init(cx: &mut App) {
    cx.set_global(Strings::english());
}

/// The text for a key (see the module docs). Prefer the `t!` macro.
pub fn text(cx: &App, key: &str, args: &[(&str, Arg)]) -> SharedString {
    match cx.try_global::<Strings>() {
        Some(strings) => strings.text(key, args),
        None => SharedString::from(key.to_string()),
    }
}

/// `t!(cx, "key")` or `t!(cx, "key", name = value, …)`: the text for a key in the current language.
macro_rules! t {
    ($cx:expr, $key:literal) => {
        $crate::i18n::text($cx, $key, &[])
    };
    ($cx:expr, $key:literal, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::i18n::text(
            $cx,
            $key,
            &[$((stringify!($name), $crate::i18n::Arg::from($value))),+],
        )
    };
}
pub(crate) use t;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_english_file_parses() {
        if let Err((_, errors)) = FluentResource::try_new(ENGLISH.to_string()) {
            panic!("locales/en.ftl has errors: {errors:?}");
        }
    }

    #[test]
    fn picks_plural_forms() {
        let strings = Strings::english();
        assert_eq!(
            strings.text("changes-count", &[("count", Arg::from(0))]),
            "No changes"
        );
        assert_eq!(
            strings.text("changes-count", &[("count", Arg::from(1))]),
            "1 change"
        );
        assert_eq!(
            strings.text("changes-count", &[("count", Arg::from(5))]),
            "5 changes"
        );
    }

    #[test]
    fn a_missing_key_shows_itself() {
        assert_eq!(Strings::english().text("no-such-key", &[]), "no-such-key");
    }

    /// Every `t!(cx, "key"…)` in the app's code has a message in the English file.
    #[test]
    fn every_key_in_the_code_has_english_text() {
        let strings = Strings::english();
        let mut missing = Vec::new();
        let mut sources = Vec::new();
        collect_sources(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .as_path(),
            &mut sources,
        );
        for path in sources {
            let code = std::fs::read_to_string(&path).unwrap_or_default();
            for (index, _) in code.match_indices("t!(") {
                let rest = &code[index..];
                let Some(start) = rest.find('"') else {
                    continue;
                };
                let Some(length) = rest[start + 1..].find('"') else {
                    continue;
                };
                let key = &rest[start + 1..start + 1 + length];
                // Only literal keys straight after `cx, ` count (skips this file's examples).
                let before_key = &rest[3..start];
                if before_key.contains(',') && !before_key.contains('(') && !strings.has(key) {
                    missing.push(format!("{}: {key}", path.display()));
                }
            }
        }
        assert!(
            missing.is_empty(),
            "keys with no English text:\n{}",
            missing.join("\n")
        );
    }

    fn collect_sources(dir: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_sources(&path, found);
            } else if path.extension().is_some_and(|e| e == "rs")
                // This file's docs show `t!` with made-up keys.
                && path.file_name().is_some_and(|n| n != "i18n.rs")
            {
                found.push(path);
            }
        }
    }
}
