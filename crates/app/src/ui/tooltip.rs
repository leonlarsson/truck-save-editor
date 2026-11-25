//! Tooltips that wrap. The kit draws a tooltip as wide as its text, and its components build
//! their tooltip themselves (no way to give it a width from outside, in the pinned version), so a
//! long explanation ran off the screen as one line. `.tip(text)` breaks the text into lines first,
//! at spaces, then sets the component's tooltip. Every tooltip in the app goes through it.

use gpui_kit::component::button::Button;
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::{Div, SharedString, Stateful, StatefulInteractiveElement as _};

/// About 60 characters a line: some 360 pixels in the tooltip's small type, comfortable to read.
const LINE_CHARACTERS: usize = 60;

/// The text in lines of at most `LINE_CHARACTERS` characters, broken at spaces. A word longer
/// than a line keeps a line of its own; line breaks already in the text stay.
pub fn wrap_tooltip(text: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split(' ').filter(|word| !word.is_empty()) {
            let fits = line.chars().count() + 1 + word.chars().count() <= LINE_CHARACTERS;
            if line.is_empty() {
                line.push_str(word);
            } else if fits {
                line.push(' ');
                line.push_str(word);
            } else {
                lines.push(std::mem::take(&mut line));
                line.push_str(word);
            }
        }
        lines.push(line);
    }
    lines.join("\n")
}

/// `.tip(text)`: the component's tooltip, wrapped (see the module's comment).
pub trait Tip: Sized {
    fn tip(self, text: impl Into<SharedString>) -> Self;
}

impl Tip for Button {
    fn tip(self, text: impl Into<SharedString>) -> Self {
        self.tooltip(wrap_tooltip(&text.into()))
    }
}

impl Tip for Checkbox {
    fn tip(self, text: impl Into<SharedString>) -> Self {
        self.tooltip(wrap_tooltip(&text.into()))
    }
}

/// Plain elements with an id (`div().id(…)`): text that explains itself on hover, like a save's
/// exact time under its "2 hours ago".
impl Tip for Stateful<Div> {
    fn tip(self, text: impl Into<SharedString>) -> Self {
        let text: SharedString = wrap_tooltip(&text.into()).into();
        self.tooltip(move |window, cx| Tooltip::new(text.clone()).build(window, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaks_long_text_at_spaces() {
        let text = "Marks every road on the map as driven, as if you'd been everywhere. It may count toward the game's achievements.";
        let wrapped = wrap_tooltip(text);
        assert!(wrapped.lines().count() > 1);
        assert!(
            wrapped
                .lines()
                .all(|line| line.chars().count() <= LINE_CHARACTERS)
        );
        // Nothing lost or added but the breaks.
        assert_eq!(wrapped.replace('\n', " "), text);
    }

    #[test]
    fn leaves_short_text_and_long_words_alone() {
        assert_eq!(wrap_tooltip("Undo"), "Undo");
        let word = "x".repeat(LINE_CHARACTERS + 5);
        assert_eq!(
            wrap_tooltip(&format!("a {word} b")),
            format!("a\n{word}\nb")
        );
        assert_eq!(wrap_tooltip("one\ntwo"), "one\ntwo");
    }
}
