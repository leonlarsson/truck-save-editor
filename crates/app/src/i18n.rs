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
//! sentences out of pieces. Adding a language is adding a file and a `Language`; anything a
//! translation lacks shows in English.

use std::borrow::Cow;

use fluent_bundle::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use gpui_kit::{App, Global, SharedString};
use unic_langid::LanguageIdentifier;

/// The messages, compiled in.
const ENGLISH: &str = include_str!("../locales/en.ftl");
const SWEDISH: &str = include_str!("../locales/sv.ftl");

/// A language the app speaks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Language {
    #[default]
    English,
    Swedish,
}

impl Language {
    pub const ALL: [Language; 2] = [Language::English, Language::Swedish];

    /// The code in the settings file and for Fluent (`en`, `sv`).
    pub fn code(self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Swedish => "sv",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Language::ALL
            .into_iter()
            .find(|language| language.code() == code)
    }

    /// The game's name for the language, for its own text (`en_us`, `sv_se`).
    pub fn game_code(self) -> &'static str {
        match self {
            Language::English => "en_us",
            Language::Swedish => "sv_se",
        }
    }

    /// The language's name in itself, as a language picker shows it.
    pub fn native_name(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Swedish => "Svenska",
        }
    }

    /// How the language writes numbers.
    pub fn number_style(self) -> crate::format::NumberStyle {
        match self {
            Language::English => crate::format::NumberStyle::Comma,
            Language::Swedish => crate::format::NumberStyle::Space,
        }
    }

    fn messages(self) -> &'static str {
        match self {
            Language::English => ENGLISH,
            Language::Swedish => SWEDISH,
        }
    }
}

/// The messages of the language in use, with English beneath for anything it lacks.
pub struct Strings {
    bundle: FluentBundle<FluentResource>,
    english: Option<FluentBundle<FluentResource>>,
}

impl Global for Strings {}

fn bundle(language: Language) -> FluentBundle<FluentResource> {
    let id: LanguageIdentifier = language.code().parse().unwrap_or_default();
    let mut bundle = FluentBundle::new(vec![id]);
    // Fluent wraps arguments in invisible direction marks by default (for mixing right-to-left
    // text); GPUI would draw them, so they're off until a right-to-left language arrives.
    bundle.set_use_isolating(false);
    let resource = match FluentResource::try_new(language.messages().to_string()) {
        Ok(resource) => resource,
        // A broken file still gives every message it could read; a test catches the error.
        Err((resource, _)) => resource,
    };
    let _ = bundle.add_resource(resource);
    bundle
}

impl Strings {
    pub fn new(language: Language) -> Self {
        Self {
            bundle: bundle(language),
            english: (language != Language::English).then(|| bundle(Language::English)),
        }
    }

    /// The text for a key. A missing key shows as the key itself, so it's visible, not blank.
    pub fn text(&self, key: &str, args: &[(&str, Arg)]) -> SharedString {
        let found = [Some(&self.bundle), self.english.as_ref()]
            .into_iter()
            .flatten()
            .find_map(|bundle| {
                let pattern = bundle.get_message(key)?.value()?;
                Some((bundle, pattern))
            });
        let Some((bundle, pattern)) = found else {
            return SharedString::from(key.to_string());
        };
        let mut fluent_args = FluentArgs::new();
        for (name, value) in args {
            fluent_args.set(*name, value.to_fluent());
        }
        let mut errors = Vec::new();
        let text = bundle.format_pattern(pattern, Some(&fluent_args), &mut errors);
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

/// Loads the language chosen in the settings.
pub fn init(cx: &mut App) {
    let language = crate::settings::get(cx).language;
    crate::format::set_style(language.number_style());
    cx.set_global(Strings::new(language));
}

/// Switches the app to another language: its own text, and the game's (city names and so on).
pub fn set_language(language: Language, cx: &mut App) {
    crate::settings::update(cx, |settings| settings.language = language);
    crate::format::set_style(language.number_style());
    cx.set_global(Strings::new(language));
    crate::game_data::reload(cx);
    cx.refresh_windows();
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
    fn the_swedish_file_parses() {
        if let Err((_, errors)) = FluentResource::try_new(SWEDISH.to_string()) {
            panic!("locales/sv.ftl has errors: {errors:?}");
        }
    }

    /// Message keys and the `$variables` each uses, from a Fluent file.
    fn messages(
        text: &str,
    ) -> std::collections::BTreeMap<String, std::collections::BTreeSet<String>> {
        let mut messages = std::collections::BTreeMap::new();
        let mut current: Option<String> = None;
        for line in text.lines() {
            let starts_message =
                line.chars().next().is_some_and(|c| c.is_ascii_lowercase()) && line.contains(" = ");
            if starts_message {
                let key = line
                    .split(" = ")
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                messages.insert(key.clone(), std::collections::BTreeSet::new());
                current = Some(key);
            } else if line.trim().is_empty() || line.starts_with('#') {
                current = None;
            }
            if let Some(key) = &current {
                let mut rest = line;
                while let Some(at) = rest.find('$') {
                    let name: String = rest[at + 1..]
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                        .collect();
                    if let Some(variables) = messages.get_mut(key) {
                        variables.insert(name);
                    }
                    rest = &rest[at + 1..];
                }
            }
        }
        messages
    }

    /// Swedish has every message English has, using the same variables: nothing falls back to
    /// English, and no number or name goes missing in translation.
    #[test]
    fn swedish_is_complete() {
        let english = messages(ENGLISH);
        let swedish = messages(SWEDISH);
        let missing: Vec<&String> = english
            .keys()
            .filter(|key| !swedish.contains_key(*key))
            .collect();
        assert!(missing.is_empty(), "missing in sv.ftl: {missing:?}");
        let extra: Vec<&String> = swedish
            .keys()
            .filter(|key| !english.contains_key(*key))
            .collect();
        assert!(extra.is_empty(), "in sv.ftl but not en.ftl: {extra:?}");
        let different: Vec<&String> = english
            .iter()
            .filter(|(key, variables)| swedish.get(*key) != Some(variables))
            .map(|(key, _)| key)
            .collect();
        assert!(
            different.is_empty(),
            "different variables in sv.ftl: {different:?}"
        );
    }

    #[test]
    fn swedish_picks_its_plural_forms() {
        let strings = Strings::new(Language::Swedish);
        assert_eq!(
            strings.text("changes-count", &[("count", Arg::from(1))]),
            "1 ändring"
        );
        assert_eq!(
            strings.text("changes-count", &[("count", Arg::from(3))]),
            "3 ändringar"
        );
    }

    #[test]
    fn picks_plural_forms() {
        let strings = Strings::new(Language::English);
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
        assert_eq!(
            Strings::new(Language::English).text("no-such-key", &[]),
            "no-such-key"
        );
    }

    /// Every `t!(cx, "key"…)` in the app's code has a message in the English file.
    #[test]
    fn every_key_in_the_code_has_english_text() {
        let strings = Strings::new(Language::English);
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
