//! Settings: the app's language, light or dark, where the game is, where backups go, and what
//! this app is. Changes apply at once and are kept for next time.

use gpui_kit::component::{
    ActiveTheme as _, Selectable as _, Sizable as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    AppContext as _, Context, IntoElement, ParentElement as _, PathPromptOptions, Render,
    SharedString, Styled as _, Window, div, rems,
};

use crate::game_data;
use crate::i18n::{Language, set_language, t};
use crate::settings::{self, Appearance};
use crate::theme;
use crate::ui::{Section, page_body};
use crate::workspace::backups_root;

pub struct SettingsPage {
    /// Why the last folder picked wasn't taken, if it wasn't.
    folder_problem: Option<SharedString>,
}

impl SettingsPage {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self {
            folder_problem: None,
        }
    }

    /// Asks for the game's folder, and takes it if the game is there.
    fn choose_game_folder(&mut self, cx: &mut Context<Self>) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(t!(cx, "settings-game-choose")),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = prompt.await else {
                return;
            };
            let Some(dir) = paths.into_iter().next() else {
                return;
            };
            let _ = this.update(cx, |this, cx| {
                if dir.join("def.scs").is_file() {
                    this.folder_problem = None;
                    settings::update(cx, |settings| settings.game_dir = Some(dir));
                    game_data::reload(cx);
                } else {
                    this.folder_problem = Some(t!(cx, "settings-game-not-game"));
                }
                cx.notify();
            });
        })
        .detach();
    }
}

impl Render for SettingsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current = settings::get(cx);
        let muted = cx.theme().muted_foreground;

        let languages = ButtonGroup::new("language")
            .outline()
            .small()
            .children(Language::ALL.iter().map(|language| {
                Button::new(language.code())
                    .label(language.native_name())
                    .selected(current.language == *language)
            }))
            .on_click(cx.listener(|_, clicked: &Vec<usize>, _, cx| {
                if let Some(language) = clicked.first().and_then(|index| Language::ALL.get(*index))
                {
                    set_language(*language, cx);
                }
            }));

        let appearances = [Appearance::System, Appearance::Light, Appearance::Dark];
        let appearance = ButtonGroup::new("appearance")
            .outline()
            .small()
            .child(
                Button::new("system")
                    .label(t!(cx, "settings-appearance-system"))
                    .selected(current.appearance == Appearance::System),
            )
            .child(
                Button::new("light")
                    .label(t!(cx, "settings-appearance-light"))
                    .selected(current.appearance == Appearance::Light),
            )
            .child(
                Button::new("dark")
                    .label(t!(cx, "settings-appearance-dark"))
                    .selected(current.appearance == Appearance::Dark),
            )
            .on_click(cx.listener(move |_, clicked: &Vec<usize>, window, cx| {
                if let Some(appearance) = clicked.first().and_then(|index| appearances.get(*index))
                {
                    theme::set_appearance(*appearance, window, cx);
                }
            }));

        let game_dir = game_data::install_dir(cx);
        let game_status = match (&game_dir, current.game_dir.is_some()) {
            (Some(_), true) => t!(cx, "settings-game-picked"),
            (Some(_), false) => t!(cx, "settings-game-found"),
            (None, _) => t!(cx, "settings-game-not-found"),
        };
        let path_text = |path: Option<std::path::PathBuf>| {
            div()
                .text_sm()
                .font_family(cx.theme().mono_font_family.clone())
                .text_color(muted)
                .child(
                    path.map(|path| path.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                )
        };

        let backups = backups_root();
        page_body().child(
            v_flex()
                .gap_6()
                .max_w(rems(46.))
                .child(
                    Section::new(t!(cx, "settings-language"))
                        .description(t!(cx, "settings-language-description"))
                        .child(h_flex().child(languages)),
                )
                .child(
                    Section::new(t!(cx, "settings-appearance")).child(h_flex().child(appearance)),
                )
                .child(
                    Section::new(t!(cx, "settings-game"))
                        .description(t!(cx, "settings-game-description"))
                        .child(
                            v_flex()
                                .gap_2()
                                .child(div().text_sm().child(game_status))
                                .child(path_text(game_dir))
                                .children(self.folder_problem.clone().map(|problem| {
                                    div().text_sm().text_color(cx.theme().danger).child(problem)
                                }))
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .child(
                                            Button::new("choose-game")
                                                .small()
                                                .outline()
                                                .label(t!(cx, "settings-game-choose"))
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.choose_game_folder(cx)
                                                })),
                                        )
                                        .children(current.game_dir.is_some().then(|| {
                                            Button::new("find-game")
                                                .small()
                                                .ghost()
                                                .label(t!(cx, "settings-game-automatic"))
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.folder_problem = None;
                                                    settings::update(cx, |settings| {
                                                        settings.game_dir = None
                                                    });
                                                    game_data::reload(cx);
                                                    cx.notify();
                                                }))
                                        })),
                                ),
                        ),
                )
                .child(
                    Section::new(t!(cx, "settings-backups"))
                        .description(t!(cx, "settings-backups-description"))
                        .child(
                            v_flex().gap_2().child(path_text(backups.clone())).child(
                                h_flex().child(
                                    Button::new("open-backups")
                                        .small()
                                        .outline()
                                        .label(t!(cx, "settings-backups-open"))
                                        .on_click(move |_, _, cx| {
                                            if let Some(dir) = &backups {
                                                // Made on the first save; make it now so there's
                                                // a folder to open.
                                                let _ = std::fs::create_dir_all(dir);
                                                cx.reveal_path(dir);
                                            }
                                        }),
                                ),
                            ),
                        ),
                )
                .child(
                    Section::new(t!(cx, "settings-about")).child(
                        v_flex()
                            .gap_2()
                            .child(div().text_sm().child(format!(
                                "{} · {}",
                                crate::APP_NAME,
                                t!(cx, "settings-version", version = env!("CARGO_PKG_VERSION"))
                            )))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(muted)
                                    .child(t!(cx, "settings-not-affiliated")),
                            )
                            .child(
                                h_flex().child(
                                    Button::new("source")
                                        .small()
                                        .ghost()
                                        .label(t!(cx, "settings-source"))
                                        .on_click(|_, _, cx| {
                                            cx.open_url(env!("CARGO_PKG_REPOSITORY"))
                                        }),
                                ),
                            ),
                    ),
                ),
        )
    }
}

/// Opens the settings in a side panel (from the start screen, before a save is open).
pub fn open_sheet(window: &mut Window, cx: &mut gpui_kit::App) {
    use gpui_kit::component::WindowExt as _;
    let view = cx.new(|cx| SettingsPage::new(window, cx));
    window.open_sheet(cx, move |sheet, _, cx| {
        sheet
            .title(t!(cx, "page-settings"))
            .size(gpui_kit::px(520.))
            .child(view.clone())
    });
}
