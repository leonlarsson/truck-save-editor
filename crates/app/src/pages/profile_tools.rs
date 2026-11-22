//! One profile's tools, in a side panel from the start screen: back it all up, make a copy under
//! a new name, and copy its settings (graphics, controls, gameplay options) to another profile.
//! They work on whole folders, so they run in the background and say how it went.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    select::{SearchableVec, Select, SelectItem, SelectState},
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Task, WeakEntity, Window, div, px,
};
use save::{CopyProblem, Profile};

use crate::i18n::t;
use crate::pages::open_save::OpenSavePage;
use crate::ui::Section;
use crate::workspace::backups_root;

/// How one of the tools went: what to say, and whether it's a problem.
#[derive(Clone)]
struct Outcome {
    text: SharedString,
    is_problem: bool,
    /// A folder to open (a backup's).
    folder: Option<std::path::PathBuf>,
}

pub struct ProfileTools {
    profile: Profile,
    /// The start screen, to look for profiles again after a copy.
    start: WeakEntity<OpenSavePage>,
    copy_name: Entity<InputState>,
    /// Profiles of the same game with settings of their own, to copy the settings to.
    targets: Entity<SelectState<SearchableVec<TargetProfile>>>,
    target_dirs: Vec<Profile>,
    backup: Option<Outcome>,
    copy: Option<Outcome>,
    settings: Option<Outcome>,
    is_busy: bool,
    _task: Option<Task<()>>,
}

impl ProfileTools {
    fn new(
        profile: Profile,
        others: Vec<Profile>,
        start: WeakEntity<OpenSavePage>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let suggested = t!(cx, "profile-tools-copy-suggestion", name = &profile.name);
        let copy_name = cx.new(|cx| InputState::new(window, cx).default_value(suggested));
        let target_dirs: Vec<Profile> = others
            .into_iter()
            .filter(|other| {
                other.game == profile.game
                    && other.dir != profile.dir
                    && other.dir.join("config.cfg").is_file()
            })
            .collect();
        let items: Vec<TargetProfile> = target_dirs
            .iter()
            .enumerate()
            .map(|(index, other)| TargetProfile {
                index: index.to_string().into(),
                name: crate::labels::profile_place(cx, other),
            })
            .collect();
        let targets = cx.new(|cx| SelectState::new(SearchableVec::new(items), None, window, cx));
        Self {
            profile,
            start,
            copy_name,
            targets,
            target_dirs,
            backup: None,
            copy: None,
            settings: None,
            is_busy: false,
            _task: None,
        }
    }

    /// What a failed tool says: the game running gets its own words.
    fn problem(error: &save::Error, cx: &Context<Self>) -> Outcome {
        let text = match error {
            save::Error::GameRunning => t!(cx, "profile-tools-running"),
            error => t!(cx, "profile-tools-failed", error = error.to_string()),
        };
        Outcome {
            text,
            is_problem: true,
            folder: None,
        }
    }

    fn back_up(&mut self, cx: &mut Context<Self>) {
        let Some(backups) = backups_root() else {
            return;
        };
        let (dir, name) = (self.profile.dir.clone(), self.profile.name.clone());
        self.is_busy = true;
        cx.notify();
        self._task = Some(cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { save::back_up_profile(&dir, &backups, &name) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.is_busy = false;
                this.backup = Some(match result {
                    Ok(backup) => Outcome {
                        text: t!(
                            cx,
                            "profile-tools-backed-up",
                            size = crate::format::decimal(backup.bytes as f64 / 1_000_000.0, 1)
                        ),
                        is_problem: false,
                        folder: Some(backup.dir),
                    },
                    Err(error) => Self::problem(&error, cx),
                });
                cx.notify();
            });
        }));
    }

    fn make_copy(&mut self, cx: &mut Context<Self>) {
        let name = self.copy_name.read(cx).value().trim().to_string();
        let destination = match save::copy_destination(self.profile.game, &name) {
            Ok(destination) => destination,
            Err(problem) => {
                self.copy = Some(Outcome {
                    text: match problem {
                        CopyProblem::EmptyName => t!(cx, "profile-tools-copy-name-empty"),
                        CopyProblem::Taken => t!(cx, "profile-tools-copy-name-taken"),
                        CopyProblem::NoDocuments => t!(cx, "profile-tools-copy-no-documents"),
                    },
                    is_problem: true,
                    folder: None,
                });
                cx.notify();
                return;
            }
        };
        let (game, dir) = (self.profile.game, self.profile.dir.clone());
        self.is_busy = true;
        cx.notify();
        self._task = Some(cx.spawn(async move |this, cx| {
            let copy_name = name.clone();
            let result = cx
                .background_spawn(async move {
                    save::copy_profile(game, &dir, &destination, &copy_name)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.is_busy = false;
                this.copy = Some(match result {
                    Ok(()) => Outcome {
                        text: t!(cx, "profile-tools-copied", name = &name),
                        is_problem: false,
                        folder: None,
                    },
                    Err(error) => Self::problem(&error, cx),
                });
                // The new profile shows on the start screen.
                let _ = this.start.update(cx, |page, cx| page.refresh(cx));
                cx.notify();
            });
        }));
    }

    fn copy_settings(&mut self, cx: &mut Context<Self>) {
        let Some(backups) = backups_root() else {
            return;
        };
        let Some(target) = self
            .targets
            .read(cx)
            .selected_value()
            .and_then(|index| index.parse::<usize>().ok())
            .and_then(|index| self.target_dirs.get(index).cloned())
        else {
            return;
        };
        let (game, from) = (self.profile.game, self.profile.dir.clone());
        self.is_busy = true;
        cx.notify();
        self._task = Some(cx.spawn(async move |this, cx| {
            let target_name = target.name.clone();
            let result = cx
                .background_spawn(async move {
                    save::copy_profile_settings(game, &from, &target.dir, &target.name, &backups)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.is_busy = false;
                this.settings = Some(match result {
                    Ok(()) => Outcome {
                        text: t!(cx, "profile-tools-settings-copied", name = &target_name),
                        is_problem: false,
                        folder: None,
                    },
                    Err(error) => Self::problem(&error, cx),
                });
                cx.notify();
            });
        }));
    }

    fn outcome(outcome: &Option<Outcome>, cx: &Context<Self>) -> Option<impl IntoElement> {
        let outcome = outcome.clone()?;
        let colour = if outcome.is_problem {
            cx.theme().danger
        } else {
            cx.theme().muted_foreground
        };
        Some(
            h_flex()
                .gap_2()
                .flex_wrap()
                .items_center()
                .child(div().text_sm().text_color(colour).child(outcome.text))
                .children(outcome.folder.map(|folder| {
                    Button::new("open-profile-backup")
                        .xsmall()
                        .ghost()
                        .label(t!(cx, "settings-backups-open"))
                        .on_click(move |_, _, cx| cx.reveal_path(&folder))
                })),
        )
    }
}

impl Render for ProfileTools {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_busy = self.is_busy;
        let has_target = self.targets.read(cx).selected_value().is_some();
        v_flex()
            .pt_2()
            .gap_4()
            .child(
                Section::new(t!(cx, "profile-tools-backup"))
                    .description(t!(cx, "profile-tools-backup-description"))
                    .child(
                        v_flex()
                            .gap_2()
                            .child(
                                h_flex().child(
                                    Button::new("back-up-profile")
                                        .small()
                                        .outline()
                                        .label(t!(cx, "profile-tools-backup-now"))
                                        .disabled(is_busy)
                                        .on_click(cx.listener(|this, _, _, cx| this.back_up(cx))),
                                ),
                            )
                            .children(Self::outcome(&self.backup, cx)),
                    ),
            )
            .child(
                Section::new(t!(cx, "profile-tools-copy"))
                    .description(t!(cx, "profile-tools-copy-description"))
                    .child(
                        v_flex()
                            .gap_2()
                            .child(Input::new(&self.copy_name).small())
                            .child(
                                h_flex().child(
                                    Button::new("copy-profile")
                                        .small()
                                        .outline()
                                        .label(t!(cx, "profile-tools-copy-now"))
                                        .disabled(is_busy)
                                        .on_click(cx.listener(|this, _, _, cx| this.make_copy(cx))),
                                ),
                            )
                            .children(Self::outcome(&self.copy, cx)),
                    ),
            )
            .child(
                Section::new(t!(cx, "profile-tools-settings"))
                    .description(t!(cx, "profile-tools-settings-description"))
                    .child(if self.target_dirs.is_empty() {
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(t!(cx, "profile-tools-settings-none"))
                            .into_any_element()
                    } else {
                        v_flex()
                            .gap_2()
                            .child(
                                Select::new(&self.targets)
                                    .small()
                                    .placeholder(t!(cx, "profile-tools-settings-pick")),
                            )
                            .child(
                                h_flex().child(
                                    Button::new("copy-settings")
                                        .small()
                                        .outline()
                                        .label(t!(cx, "profile-tools-settings-now"))
                                        .disabled(is_busy || !has_target)
                                        .on_click(
                                            cx.listener(|this, _, _, cx| this.copy_settings(cx)),
                                        ),
                                ),
                            )
                            .children(Self::outcome(&self.settings, cx))
                            .into_any_element()
                    }),
            )
    }
}

/// A profile to copy settings to: shown by its name and place, chosen by its place in the list.
#[derive(Clone)]
struct TargetProfile {
    index: SharedString,
    name: SharedString,
}

impl SelectItem for TargetProfile {
    type Value = SharedString;

    fn title(&self) -> SharedString {
        self.name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.index
    }
}

/// Opens a profile's tools in a side panel.
pub fn open(
    profile: Profile,
    others: Vec<Profile>,
    start: WeakEntity<OpenSavePage>,
    window: &mut Window,
    cx: &mut gpui_kit::App,
) {
    let title = t!(cx, "profile-tools-title", name = &profile.name);
    let view = cx.new(|cx| ProfileTools::new(profile, others, start, window, cx));
    window.open_sheet(cx, move |sheet, _, _| {
        sheet
            .title(title.clone())
            .size(px(480.))
            .child(view.clone())
    });
}
