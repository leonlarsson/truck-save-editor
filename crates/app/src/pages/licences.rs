//! The licences of what's built into the editor, shown from Settings → About: code adapted from
//! other projects (`THIRD_PARTY_NOTICES.md`), then every crate's licence, as `cargo-about` lists
//! them (`assets/licenses.txt`, made by `mise run licenses` from `about.toml` and `about.hbs`).
//! They're inside the .exe, so they go wherever it's copied.
//!
//! Each licence is a row with the crates under it; its text shows when the row is opened (one at
//! a time: laying out all of them at once would be a third of a megabyte of text).

use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    AppContext as _, Context, InteractiveElement as _, IntoElement, ParentElement as _, Render,
    SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px, rems,
};

use crate::i18n::t;

/// The adapted code's notices, then the crates' licences.
const NOTICES: &str = include_str!("../../../../THIRD_PARTY_NOTICES.md");
const LICENSES: &str = include_str!("../../assets/licenses.txt");
/// What `about.hbs` puts between licences.
const SEPARATOR: &str =
    "--------------------------------------------------------------------------------";

/// One licence: its name, what uses it, its text.
#[derive(Clone, Debug, PartialEq)]
struct Entry {
    name: SharedString,
    used_by: SharedString,
    text: SharedString,
}

/// The licences in `assets/licenses.txt` (`about.hbs`: the name, `Used by: …`, a blank line, the
/// text, then the separator).
fn entries_from(listed: &str) -> Vec<Entry> {
    listed
        .split(SEPARATOR)
        .filter_map(|block| {
            let block = block.trim_matches('\n');
            let mut lines = block.lines();
            let name = lines.next()?.trim().to_string();
            let used_by = lines.next()?.trim().strip_prefix("Used by: ")?.to_string();
            let text = lines.collect::<Vec<_>>().join("\n").trim().to_string();
            (!name.is_empty()).then(|| Entry {
                name: name.into(),
                used_by: used_by.into(),
                text: text.into(),
            })
        })
        .collect()
}

pub struct Licences {
    entries: Vec<Entry>,
    /// The licence whose text shows.
    open: Option<usize>,
}

/// The adapted code's notice as a licence like the others: the licence text is the notice's
/// fenced block, and what it covers is the line before it.
fn adapted(notices: &str) -> Option<Entry> {
    let (before, rest) = notices.split_once("```text")?;
    let (text, _) = rest.split_once("```")?;
    let used_by = before
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())?
        // Plain text: no Markdown links or code marks.
        .replace(['`', '[', ']'], "");
    let used_by = match used_by.split_once("(https://") {
        Some((start, end)) => format!(
            "{start}{}",
            end.split_once(')').map(|(_, after)| after).unwrap_or("")
        ),
        None => used_by,
    };
    Some(Entry {
        name: "Code adapted from scs_tools (MIT)".into(),
        used_by: used_by.into(),
        text: text.trim().to_string().into(),
    })
}

impl Licences {
    fn new() -> Self {
        let mut entries: Vec<Entry> = adapted(NOTICES).into_iter().collect();
        entries.extend(entries_from(LICENSES));
        Self {
            entries,
            open: None,
        }
    }
}

impl Render for Licences {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let mono = theme.mono_font_family.clone();
        let rows: Vec<_> = self
            .entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let is_open = self.open == Some(index);
                v_flex()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        // As tall as its two lines (a button is one line tall otherwise).
                        Button::new(("licence", index))
                            .ghost()
                            .small()
                            .w_full()
                            .h_auto()
                            .py_1p5()
                            .child(
                                h_flex()
                                    .w_full()
                                    .gap_2()
                                    .items_start()
                                    .child(Icon::new(if is_open {
                                        IconName::ChevronDown
                                    } else {
                                        IconName::ChevronRight
                                    }))
                                    .child(
                                        v_flex()
                                            .min_w_0()
                                            .flex_1()
                                            .items_start()
                                            .child(div().text_sm().child(entry.name.clone()))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme.muted_foreground)
                                                    .whitespace_normal()
                                                    .child(entry.used_by.clone()),
                                            ),
                                    ),
                            )
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.open = if is_open { None } else { Some(index) };
                                cx.notify();
                            })),
                    )
                    .children(is_open.then(|| {
                        div()
                            .px_3()
                            .pb_3()
                            .text_xs()
                            .font_family(mono.clone())
                            .text_color(theme.muted_foreground)
                            .whitespace_normal()
                            .child(entry.text.clone())
                    }))
            })
            .collect();
        v_flex()
            .id("licences")
            .max_h(rems(32.))
            .overflow_y_scroll()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child(t!(cx, "licences-description")),
            )
            .child(v_flex().children(rows))
    }
}

/// Opens the licences, over whatever is open.
pub fn open(window: &mut Window, cx: &mut gpui_kit::App) {
    let view = cx.new(|_| Licences::new());
    window.open_dialog(cx, move |dialog, _, cx| {
        dialog
            .title(t!(cx, "licences-title"))
            .width(px(640.))
            .child(view.clone())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_every_licence_listed() {
        let listed = format!(
            "\nMIT License\nUsed by: a 1.0, b 2.0\n\nCopyright (c) A\n\nPermission …\n{SEPARATOR}\n\nApache License 2.0\nUsed by: c 0.1\n\nText\n{SEPARATOR}\n"
        );
        let found = entries_from(&listed);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].used_by.as_ref(), "a 1.0, b 2.0");
        assert_eq!(found[0].text.as_ref(), "Copyright (c) A\n\nPermission …");
        // The list built into the app reads too, every entry.
        let built_in = entries_from(LICENSES);
        assert!(built_in.len() > 100, "{}", built_in.len());
        assert!(built_in.iter().all(|entry| !entry.text.is_empty()));
        // The adapted code's notice reads as one too, with its licence's text.
        let notice = adapted(NOTICES).expect("the notice");
        assert!(notice.text.starts_with("MIT License"), "{}", notice.text);
        assert!(!notice.used_by.contains('`'), "{}", notice.used_by);
    }
}
