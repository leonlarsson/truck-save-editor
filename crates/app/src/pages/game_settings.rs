//! Game settings: where the game is installed (the editor reads its data from there, and never
//! changes anything in it), and the game's own settings in its `config.cfg` that the editor can
//! change. About the installed game, not a save: the current game's (the open save's, or the one
//! picked on the start screen), whichever of its saves is open, or none.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex, v_flex,
};
use gpui_kit::{
    AppContext as _, Context, IntoElement, ParentElement as _, PathPromptOptions, Render,
    SharedString, Styled as _, Window, div, rems,
};

use crate::game_data;
use crate::i18n::t;
use crate::settings;
use crate::ui::{Divided, Section, page_body, path_text};
use crate::workspace::backups_root;

pub struct GameSettingsPage {
    /// The game the page is about (the current game, followed when it changes).
    game: save::Game,
    /// Why the last folder picked wasn't taken, if it wasn't.
    folder_problem: Option<SharedString>,
    /// The game's `config.cfg` (where it is, what it says), read when the page opens and after
    /// each change.
    game_config: Option<(std::path::PathBuf, String)>,
    /// Why the last change to the game's settings didn't happen, if it didn't.
    game_config_problem: Option<SharedString>,
    /// Whether the game was running when last looked (looking lists processes, so not on every
    /// redraw: when the page opens and when the window comes back to the front).
    is_game_running: bool,
    _subscriptions: Vec<gpui_kit::Subscription>,
}

impl GameSettingsPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let game = game_data::current_game(cx);
        let _subscriptions = vec![
            cx.observe_window_activation(window, |this, window, cx| {
                if window.is_window_active() {
                    this.look_again(cx);
                }
            }),
            // Another game picked (on the start screen, or by opening one of its saves).
            game_data::observe_current_game_in(window, cx, |this, _, cx| {
                this.game = game_data::current_game(cx);
                this.folder_problem = None;
                this.game_config_problem = None;
                this.look_again(cx);
            }),
        ];
        Self {
            game,
            folder_problem: None,
            game_config: read_game_config(game),
            game_config_problem: None,
            is_game_running: save::is_game_running(game),
            _subscriptions,
        }
    }

    /// Reads the game's settings and whether it runs again.
    fn look_again(&mut self, cx: &mut Context<Self>) {
        self.game_config = read_game_config(self.game);
        self.is_game_running = save::is_game_running(self.game);
        cx.notify();
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
                    let game = this.game;
                    settings::update(cx, |settings| settings.set_game_dir(game, Some(dir)));
                    game_data::reload(cx);
                } else {
                    this.folder_problem = Some(t!(cx, "settings-game-not-game"));
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Writes a change to the game's own settings (backed up first), or says why it can't.
    fn change_game_config(&mut self, change: impl FnOnce(&str) -> String, cx: &mut Context<Self>) {
        let Some((path, text)) = read_game_config(self.game) else {
            return;
        };
        let changed = change(&text);
        let Some(backups) = backups_root() else {
            return;
        };
        self.game_config_problem =
            match save::write_game_config(self.game, &path, &changed, &backups) {
                Ok(()) => None,
                Err(save::Error::GameRunning) => Some(t!(cx, "game-settings-running")),
                Err(error) => Some(t!(cx, "game-settings-failed", error = error.to_string())),
            };
        self.look_again(cx);
    }

    /// The game's own settings the editor can change: the developer console and convoy size.
    fn render_game_settings(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let body = match &self.game_config {
            None => div()
                .text_sm()
                .text_color(muted)
                .child(t!(cx, "game-settings-none"))
                .into_any_element(),
            Some((_, text)) => {
                let settings = save::game_settings(text);
                let is_running = self.is_game_running;
                v_flex()
                    .gap_2()
                    .child(
                        Checkbox::new("developer-console")
                            .label(t!(cx, "game-settings-console"))
                            .checked(settings.developer_console)
                            .disabled(is_running)
                            .on_change(cx.listener(|this, checked: &bool, _, cx| {
                                let checked = *checked;
                                this.change_game_config(
                                    |text| save::with_developer_console(text, checked),
                                    cx,
                                );
                            })),
                    )
                    .child(
                        Checkbox::new("big-convoys")
                            .label(t!(
                                cx,
                                "game-settings-convoy",
                                count = save::MAX_CONVOY_SIZE
                            ))
                            .checked(settings.convoy_size >= save::MAX_CONVOY_SIZE)
                            .disabled(is_running)
                            .on_change(cx.listener(|this, checked: &bool, _, cx| {
                                let size = if *checked {
                                    save::MAX_CONVOY_SIZE
                                } else {
                                    save::DEFAULT_CONVOY_SIZE
                                };
                                this.change_game_config(
                                    |text| save::with_convoy_size(text, size),
                                    cx,
                                );
                            })),
                    )
                    .children(is_running.then(|| {
                        div()
                            .text_xs()
                            .text_color(muted)
                            .child(t!(cx, "game-settings-running"))
                    }))
                    .children(self.game_config_problem.clone().map(|problem| {
                        div().text_xs().text_color(cx.theme().danger).child(problem)
                    }))
                    .into_any_element()
            }
        };
        Section::new(t!(cx, "game-settings"))
            .aside()
            .description(t!(cx, "game-settings-description"))
            .child(body)
    }
}

impl Render for GameSettingsPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let game = self.game;
        let is_picked = settings::get(cx).game_dir(game).is_some();
        let game_dir = game_data::install_dir_of(cx, game);
        let game_status = match (&game_dir, is_picked) {
            (Some(_), true) => t!(cx, "settings-game-picked"),
            (Some(_), false) => t!(cx, "settings-game-found"),
            (None, _) => t!(cx, "settings-game-not-found"),
        };

        page_body().child(
            Divided::sections()
                .max_w(rems(60.))
                .child(
                    Section::new(t!(cx, "settings-game"))
                        .aside()
                        .description(t!(cx, "settings-game-description", game = game.name()))
                        .child(
                            v_flex()
                                .gap_2()
                                .child(div().text_sm().child(game_status))
                                .child(path_text(game_dir.as_deref(), cx))
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
                                        .children(is_picked.then(|| {
                                            Button::new("find-game")
                                                .small()
                                                .ghost()
                                                .label(t!(cx, "settings-game-automatic"))
                                                .on_click(cx.listener(|this, _, _, cx| {
                                                    this.folder_problem = None;
                                                    let game = this.game;
                                                    settings::update(cx, |settings| {
                                                        settings.set_game_dir(game, None)
                                                    });
                                                    game_data::reload(cx);
                                                    cx.notify();
                                                }))
                                        })),
                                ),
                        ),
                )
                .child(self.render_game_settings(cx)),
        )
    }
}

/// A game's `config.cfg`, if it has one.
fn read_game_config(game: save::Game) -> Option<(std::path::PathBuf, String)> {
    let path = save::game_config_path(game)?;
    let text = std::fs::read_to_string(&path).ok()?;
    Some((path, text))
}

/// Opens the game settings in a side panel (from the start screen, before a save is open).
pub fn open_sheet(window: &mut Window, cx: &mut gpui_kit::App) {
    use gpui_kit::component::WindowExt as _;
    let view = cx.new(|cx| GameSettingsPage::new(window, cx));
    window.open_sheet(cx, move |sheet, _, cx| {
        sheet
            .title(t!(cx, "page-game-settings"))
            .size(gpui_kit::px(520.))
            .child(view.clone())
    });
}
