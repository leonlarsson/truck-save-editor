//! Settings: the editor's own. Its language, light or dark, where backups go, whether a new version
//! is out, and what this app is. Changes apply at once and are kept for next time. The game's
//! folder and its own settings are on Game settings (`game_settings.rs`).

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Selectable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    spinner::Spinner,
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, IntoElement, ParentElement as _, Render, Styled as _, Window, div,
    rems,
};

use crate::i18n::{Language, set_language, t};
use crate::settings::{self, Appearance};
use crate::theme;
use crate::ui::{Divided, Section, page_body, path_text};
use crate::updates::{self, UpdateState};
use crate::workspace::backups_root;

pub struct SettingsPage;

impl SettingsPage {
    pub fn new(_: &mut Window, _: &mut Context<Self>) -> Self {
        Self
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

        let backups = backups_root();
        page_body().child(
            Divided::sections()
                .max_w(rems(60.))
                .child(
                    Section::new(t!(cx, "settings-language"))
                        .aside()
                        .description(t!(cx, "settings-language-description"))
                        .child(h_flex().child(languages)),
                )
                .child(
                    Section::new(t!(cx, "settings-appearance"))
                        .aside()
                        .child(h_flex().child(appearance)),
                )
                .child(
                    Section::new(t!(cx, "settings-backups"))
                        .aside()
                        .description(t!(cx, "settings-backups-description"))
                        .child(
                            v_flex()
                                .gap_2()
                                .child(path_text(backups.as_deref(), cx))
                                .child(
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
                .child(self.render_updates(cx))
                .child(
                    Section::new(t!(cx, "settings-reset"))
                        .aside()
                        .description(t!(cx, "settings-reset-description"))
                        .child(
                            h_flex().child(
                                Button::new("reset-settings")
                                    .small()
                                    .outline()
                                    .label(t!(cx, "settings-reset-button"))
                                    .on_click(|_, window, cx| confirm_reset(window, cx)),
                            ),
                        ),
                )
                .child(
                    Section::new(t!(cx, "settings-about")).aside().child(
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
                                h_flex()
                                    .gap_2()
                                    .flex_wrap()
                                    .child(
                                        Button::new("report")
                                            .small()
                                            .outline()
                                            .label(t!(cx, "report-problem"))
                                            .on_click(|_, _, cx| crate::report::open(cx, None)),
                                    )
                                    .child(
                                        Button::new("licences")
                                            .small()
                                            .ghost()
                                            .label(t!(cx, "licences-open"))
                                            .on_click(|_, window, cx| {
                                                crate::pages::licences::open(window, cx)
                                            }),
                                    )
                                    .child(
                                        Button::new("source")
                                            .small()
                                            .ghost()
                                            .label(t!(cx, "settings-source"))
                                            .on_click(|_, _, cx| {
                                                cx.open_url(env!("CARGO_PKG_REPOSITORY"))
                                            }),
                                    ),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted)
                                    .child(t!(cx, "report-description")),
                            ),
                    ),
                ),
        )
    }
}

/// Asks before putting every setting back as on the first start.
fn confirm_reset(window: &mut Window, cx: &mut gpui_kit::App) {
    window.open_alert_dialog(cx, |alert, _, cx| {
        alert
            .title(t!(cx, "settings-reset-confirm-title"))
            .description(t!(cx, "settings-reset-description"))
            .confirm()
            // The kit's own Cancel is English; ours follows the app's language.
            .cancel_text(t!(cx, "cancel"))
            .ok_text(t!(cx, "settings-reset-confirm"))
            .on_ok(|_, window, cx| {
                settings::reset(window, cx);
                true
            })
    });
}

impl SettingsPage {
    /// Whether a newer version is out, checking again, and turning the check off.
    fn render_updates(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let state = updates::state(cx);
        let is_on = settings::get(cx).check_for_updates;
        let status = match &state {
            UpdateState::NotChecked => div()
                .text_color(muted)
                .child(t!(cx, "updates-not-checked"))
                .into_any_element(),
            UpdateState::Checking => h_flex()
                .gap_2()
                .text_color(muted)
                .child(Spinner::new().small())
                .child(t!(cx, "updates-checking"))
                .into_any_element(),
            UpdateState::UpToDate => div()
                .child(t!(
                    cx,
                    "updates-up-to-date",
                    version = updates::CURRENT_VERSION
                ))
                .into_any_element(),
            UpdateState::Available { version, url } => {
                let url = url.clone();
                h_flex()
                    .gap_3()
                    .flex_wrap()
                    .child(t!(cx, "updates-available", version = version.clone()))
                    .child(
                        Button::new("download-update")
                            .small()
                            .primary()
                            .label(t!(cx, "updates-download"))
                            .on_click(move |_, _, cx| cx.open_url(&url)),
                    )
                    .into_any_element()
            }
            UpdateState::Failed => div()
                .text_color(muted)
                .child(t!(cx, "updates-failed"))
                .into_any_element(),
        };
        Section::new(t!(cx, "settings-updates"))
            .aside()
            .description(t!(cx, "settings-updates-description"))
            .child(
                v_flex().gap_3().child(div().text_sm().child(status)).child(
                    h_flex()
                        .gap_4()
                        .flex_wrap()
                        .child(
                            Button::new("check-updates")
                                .small()
                                .outline()
                                .label(t!(cx, "updates-check-now"))
                                .disabled(state == UpdateState::Checking)
                                .on_click(|_, _, cx| updates::check(cx)),
                        )
                        .child(
                            Checkbox::new("check-on-start")
                                .label(t!(cx, "updates-check-on-start"))
                                .checked(is_on)
                                .on_change(|checked: &bool, _, cx| {
                                    let checked = *checked;
                                    settings::update(cx, |settings| {
                                        settings.check_for_updates = checked
                                    });
                                    cx.refresh_windows();
                                }),
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
