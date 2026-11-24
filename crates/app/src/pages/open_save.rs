//! The start screen: every save on this computer, to pick one to edit. The player's own saves come
//! first; the game's autosaves follow. Finding them reads only the small `info.sii` of each slot, in
//! the background.
//!
//! It says what it shows and where from: the profile's saves and the folder they're in, and every
//! place it looked, with what each held. When saves are somewhere else (a backup, another drive),
//! a folder can be added, by choosing it or dropping it on the window; it's kept in the settings.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Selectable as _, Sizable as _, StyledExt as _,
    alert::Alert,
    button::{Button, ButtonGroup, ButtonVariants as _},
    h_flex,
    list::ListItem,
    spinner::Spinner,
    v_flex,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use gpui_kit::{
    AppContext as _, Context, EventEmitter, ExternalPaths, FocusHandle, InteractiveElement as _,
    IntoElement, KeyDownEvent, ObjectFit, ParentElement as _, PathPromptOptions, Render,
    RenderImage, SharedString, StatefulInteractiveElement as _, Styled as _, StyledImage as _,
    Task, Window, div, img, prelude::FluentBuilder as _, rems,
};
use save::{Game, Profile, SearchedFolder, Slot, Source};

use crate::APP_NAME;
use crate::format::thousands;
use crate::i18n::t;
use crate::labels;
use crate::ui::Tip as _;

/// How many of the game's own saves show before "Show more".
const GAME_SAVES_SHOWN: usize = 3;

/// Asks the workspace to open a slot.
#[derive(Clone)]
pub struct OpenSlot {
    pub profile: Profile,
    pub slot: Slot,
}

/// What the last look found, and where it looked.
struct Found {
    /// Each profile with its slots: the most recently saved first, ones without saves last.
    profiles: Vec<(Profile, Vec<Slot>)>,
    /// Every folder looked in.
    folders: Vec<SearchedFolder>,
    /// Whether Steam is on this PC (no Steam Cloud folders to look in otherwise).
    has_steam: bool,
}

pub struct OpenSavePage {
    found: Option<Found>,
    selected_profile: usize,
    /// A folder just added: the next look lands on the profile in it.
    select_after: Option<PathBuf>,
    /// Whether the folders looked in that aren't there are listed one by one (when saves were
    /// found, they're one line saying how many).
    shows_missing_folders: bool,
    /// Whether every save the game made is shown, rather than the newest few.
    shows_all_game_saves: bool,
    /// What happened when the editor last crashed, until it's reported or dismissed.
    crash: Option<String>,
    /// Why the last folder chosen or dropped wasn't added, if it wasn't.
    add_problem: Option<SharedString>,
    /// Each slot's preview picture, by slot folder, as they load.
    previews: HashMap<PathBuf, Arc<RenderImage>>,
    /// Each slot row's focus, by slot folder, so Tab reaches the rows and Enter opens one.
    row_focus: HashMap<PathBuf, FocusHandle>,
    /// The added folders the last look used, to look again when the settings change them (a
    /// reset in Settings).
    looked_with: Vec<PathBuf>,
    _search: Option<Task<()>>,
    _previews: Option<Task<()>>,
    _settings: Option<gpui_kit::Subscription>,
}

impl EventEmitter<OpenSlot> for OpenSavePage {}

impl OpenSavePage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut page = Self {
            found: None,
            selected_profile: 0,
            select_after: None,
            shows_all_game_saves: false,
            shows_missing_folders: false,
            crash: crate::crash::last(),
            add_problem: None,
            previews: HashMap::new(),
            row_focus: HashMap::new(),
            looked_with: Vec::new(),
            _search: None,
            _previews: None,
            _settings: None,
        };
        page._settings = Some(cx.observe_global::<crate::settings::Settings>(|page, cx| {
            if crate::settings::get(cx).profile_dirs != page.looked_with {
                page.add_problem = None;
                page.look(cx);
            }
        }));
        page.search(cx);
        page
    }

    /// Looks for profiles and their slots again, in the background, showing that it's looking.
    pub fn search(&mut self, cx: &mut Context<Self>) {
        self.found = None;
        cx.notify();
        self.look(cx);
    }

    /// Looks again without clearing the list: what's shown stays until the new list arrives, on
    /// the same profile. For coming back to the window after saving in the game.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        if self.found.is_some() {
            self.look(cx);
        }
    }

    fn look(&mut self, cx: &mut Context<Self>) {
        // The profile shown now, to stay on it when the list comes back (or the one in a folder
        // just added).
        let shown = self
            .found
            .as_ref()
            .and_then(|found| found.profiles.get(self.selected_profile))
            .map(|(profile, _)| profile.dir.clone());
        let select_after = self.select_after.take();
        let added = crate::settings::get(cx).profile_dirs;
        self.looked_with = added.clone();
        self._search = Some(cx.spawn(async move |this, cx| {
            let found = cx
                .background_spawn(async move {
                    let mut profiles = Vec::new();
                    let mut folders = Vec::new();
                    let mut has_steam = false;
                    for game in Game::ALL {
                        let search = save::search(game, &added);
                        has_steam |= !search.steam_dirs.is_empty();
                        // Every game's: the folder list shows the picked game's (`render_folders`).
                        folders.extend(search.folders);
                        profiles.extend(search.profiles.into_iter().map(|profile| {
                            let slots = save::profile_slots(&profile.dir);
                            (profile, slots)
                        }));
                    }
                    // Both games together, most recently saved first; profiles without saves
                    // (never saved) come last.
                    profiles.sort_by_key(|(profile, _)| std::cmp::Reverse(profile.last_saved));
                    // Steam's, then Documents, then the folders added.
                    folders.sort_by_key(|folder| match folder.source {
                        Source::Steam => 0,
                        Source::Documents => 1,
                        Source::Added => 2,
                    });
                    Found {
                        profiles,
                        folders,
                        has_steam,
                    }
                })
                .await;
            let _ = this.update(cx, |page, cx| {
                page.row_focus = found
                    .profiles
                    .iter()
                    .flat_map(|(_, slots)| slots.iter())
                    .map(|slot| (slot.dir.clone(), cx.focus_handle().tab_stop(true)))
                    .collect();
                let position = |dir: &PathBuf| {
                    found
                        .profiles
                        .iter()
                        .position(|(profile, _)| &profile.dir == dir)
                };
                // A folder just added: the profile it is or holds (a save or profile chosen),
                // else the first profile in it.
                let in_added = select_after.and_then(|chosen| {
                    found
                        .profiles
                        .iter()
                        .position(|(profile, _)| chosen.starts_with(&profile.dir))
                        .or_else(|| {
                            found.profiles.iter().position(|(profile, _)| {
                                profile
                                    .dir
                                    .parent()
                                    .is_some_and(|root| root.starts_with(&chosen))
                            })
                        })
                });
                let selected = in_added
                    .or_else(|| shown.as_ref().and_then(position))
                    .unwrap_or(0);
                page.found = Some(found);
                page.select_profile(selected, cx);
                page.load_previews(cx);
                cx.notify();
            });
        }));
    }

    /// Asks for a folder with saves in it, and adds it.
    fn choose_folder(&mut self, cx: &mut Context<Self>) {
        let prompt = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(t!(cx, "start-folder-choose")),
        });
        cx.spawn(async move |this, cx| {
            let Ok(Ok(Some(paths))) = prompt.await else {
                return;
            };
            let _ = this.update(cx, |this, cx| this.add_folders(paths, cx));
        })
        .detach();
    }

    /// Adds the folders of profiles the chosen or dropped paths mean (a profiles folder, a
    /// profile, one save, or the game's folder in Documents), and looks again; or says why not.
    fn add_folders(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        let Some(chosen) = paths.into_iter().next() else {
            return;
        };
        cx.spawn(async move |this, cx| {
            let (chosen, roots, loose) = cx
                .background_spawn(async move {
                    // A dropped file (a `game.sii`, say): the folder it's in.
                    let dir = if chosen.is_file() {
                        chosen.parent().map(PathBuf::from).unwrap_or(chosen)
                    } else {
                        chosen
                    };
                    let roots = save::profile_folders_at(&dir);
                    let loose = save::is_loose_save(&dir)
                        .then(|| save::loose_save(&dir))
                        .flatten();
                    (dir, roots, loose)
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                // A save on its own: opened where it is (there are no folders to add).
                if let Some((profile, slot)) = loose {
                    this.add_problem = None;
                    cx.emit(OpenSlot { profile, slot });
                    return;
                }
                if roots.is_empty() {
                    this.add_problem = Some(t!(
                        cx,
                        "start-folder-not-saves",
                        path = chosen.to_string_lossy().into_owned()
                    ));
                    cx.notify();
                    return;
                }
                this.add_problem = None;
                // Only the ones not looked in already.
                let looked_in: Vec<PathBuf> = this
                    .found
                    .iter()
                    .flat_map(|found| found.folders.iter().map(|folder| folder.dir.clone()))
                    .collect();
                let new: Vec<PathBuf> = roots
                    .into_iter()
                    .filter(|root| !looked_in.contains(root))
                    .collect();
                if !new.is_empty() {
                    crate::settings::update(cx, |settings| {
                        settings.profile_dirs.extend(new);
                    });
                }
                this.select_after = Some(chosen);
                this.look(cx);
            });
        })
        .detach();
    }

    /// Stops looking in a folder that was added.
    fn remove_folder(&mut self, dir: PathBuf, cx: &mut Context<Self>) {
        crate::settings::update(cx, |settings| {
            settings.profile_dirs.retain(|added| added != &dir);
        });
        self.add_problem = None;
        self.look(cx);
    }

    /// Reads every slot's preview picture in the background; rows show them as they arrive.
    fn load_previews(&mut self, cx: &mut Context<Self>) {
        let dirs: Vec<PathBuf> = self
            .found
            .iter()
            .flat_map(|found| found.profiles.iter())
            .flat_map(|(_, slots)| slots.iter().map(|slot| slot.dir.clone()))
            .collect();
        self._previews = Some(cx.spawn(async move |this, cx| {
            let previews = cx
                .background_spawn(async move {
                    dirs.into_iter()
                        .filter_map(|dir| {
                            let bytes = std::fs::read(dir.join("preview.tga")).ok()?;
                            let picture = save::decode_tga(&bytes)?;
                            Some((dir, Arc::new(render_image(picture)?)))
                        })
                        .collect::<HashMap<_, _>>()
                })
                .await;
            let _ = this.update(cx, |page, cx| {
                page.previews = previews;
                cx.notify();
            });
        }));
    }

    /// Shows a profile's saves (by its place in the list of every profile found), and reads its
    /// game's data if that's another game: its saves' DLC check reads that game's install.
    fn select_profile(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected_profile = index;
        let game = self
            .found
            .as_ref()
            .and_then(|found| found.profiles.get(index))
            .map(|(profile, _)| profile.game);
        // Behind an open save (its search still finishing), the save's game stays.
        if let Some(game) = game
            && crate::workspace::shows_start_screen(cx)
        {
            crate::game_data::set_game(cx, game);
        }
        cx.notify();
    }

    fn render_profiles(
        &self,
        profiles: &[(Profile, Vec<Slot>)],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        // The folders added on this screen: a profile found in one is named by that folder (an
        // added backup of a profile would otherwise read the same as the profile itself).
        let added: Vec<PathBuf> = self
            .found
            .iter()
            .flat_map(|found| found.folders.iter())
            .filter(|folder| folder.source == Source::Added)
            .map(|folder| folder.dir.clone())
            .collect();
        let describe = |profile: &Profile| -> SharedString {
            let root = profile.dir.parent();
            let place = match root.filter(|root| added.iter().any(|dir| dir == root)) {
                Some(root) => added_folder_name(root).into(),
                None => labels::location(cx, profile.location),
            };
            t!(cx, "profile-place", name = &profile.name, place = place)
        };

        // The chosen game's profiles only (the game picker above chooses the game); each button
        // keeps its place in the whole list, which is what `selected_profile` counts in.
        let game = self.picked_game(profiles).unwrap_or(Game::Ets2);
        let shown: Vec<usize> = (0..profiles.len())
            .filter(|&index| profiles[index].0.game == game)
            .collect();
        let buttons = shown.iter().map(|&index| {
            Button::new(("profile", index))
                .label(describe(&profiles[index].0))
                .selected(index == self.selected_profile)
        });
        ButtonGroup::new("profiles")
            .outline()
            .small()
            .children(buttons)
            .on_click(cx.listener(move |this, clicked: &Vec<usize>, _, cx| {
                if let Some(&index) = clicked.first().and_then(|position| shown.get(*position)) {
                    this.select_profile(index, cx);
                }
            }))
    }

    /// The game whose profiles are shown: the selected profile's (`None` when nothing was found).
    fn picked_game(&self, profiles: &[(Profile, Vec<Slot>)]) -> Option<Game> {
        profiles
            .get(self.selected_profile)
            .map(|(profile, _)| profile.game)
    }

    /// The game picker: a card for each game, with how many profiles and saves were found for
    /// it; picking one shows its first profile. A game with nothing found is shown, off, saying so.
    fn render_games(
        &self,
        profiles: &[(Profile, Vec<Slot>)],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let chosen = self.picked_game(profiles).unwrap_or(Game::Ets2);
        let cards = Game::ALL.into_iter().map(|game| {
            let of_game: Vec<usize> = (0..profiles.len())
                .filter(|&index| profiles[index].0.game == game)
                .collect();
            let saves: usize = of_game.iter().map(|&index| profiles[index].1.len()).sum();
            let detail = if of_game.is_empty() {
                t!(cx, "start-game-none")
            } else {
                t!(
                    cx,
                    "start-game-detail",
                    profiles = of_game.len(),
                    saves = saves
                )
            };
            let is_chosen = game == chosen;
            let first = of_game.first().copied();
            // The game's own logo, from its install (none when it isn't installed).
            let logo = crate::game_data::game_logo(cx, game)
                .map(|logo| crate::ui::GameIconView::new(logo).size(2.5));
            Button::new(("game", game as usize))
                .outline()
                // Equal halves of the column, whatever each name's length.
                .flex_1()
                .min_w_0()
                .h(rems(4.))
                .selected(is_chosen)
                .disabled(first.is_none())
                .accessibility_label(SharedString::from(game.name()))
                .child(
                    h_flex()
                        .w_full()
                        .gap_3()
                        .items_center()
                        .children(logo)
                        .child(
                            v_flex()
                                .items_start()
                                .min_w_0()
                                .child(
                                    div()
                                        .text_sm()
                                        .truncate()
                                        // The chosen one says so in weight, not only its fill.
                                        .map(|name| {
                                            if is_chosen {
                                                name.font_semibold()
                                            } else {
                                                name.font_medium()
                                            }
                                        })
                                        .child(game.name()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(detail),
                                ),
                        ),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    if let Some(index) = first
                        && !is_chosen
                    {
                        this.select_profile(index, cx);
                    }
                }))
        });
        h_flex().gap_3().children(cards)
    }

    /// After a crash: says so, with a way to report it (an issue filled in with what happened)
    /// or put it away.
    fn render_crash(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let report = self.crash.clone()?;
        Some(
            v_flex()
                .gap_3()
                .child(
                    Alert::warning("crashed", t!(cx, "crash-body"))
                        .title(t!(cx, "crash-title"))
                        .text_color(cx.theme().foreground)
                        .icon(Icon::new(IconName::TriangleAlert).text_color(cx.theme().warning)),
                )
                .child(
                    h_flex()
                        .px_3()
                        .gap_2()
                        .child(
                            Button::new("crash-report")
                                .small()
                                .primary()
                                .icon(Icon::new(IconName::ExternalLink))
                                .label(t!(cx, "report-this"))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    crate::report::open_crash(cx, &report);
                                    crate::crash::forget();
                                    this.crash = None;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("crash-dismiss")
                                .small()
                                .ghost()
                                .label(t!(cx, "crash-dismiss"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    crate::crash::forget();
                                    this.crash = None;
                                    cx.notify();
                                })),
                        ),
                ),
        )
    }

    /// What's shown and where it's from: how many saves, the profile, where it's kept, and its
    /// folder.
    fn render_summary(
        &self,
        profile: &Profile,
        slots: &[Slot],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let text = match profile.location {
            save::Location::SteamCloud => t!(
                cx,
                "start-showing-cloud",
                count = slots.len(),
                name = &profile.name
            ),
            save::Location::Local | save::Location::Loose => t!(
                cx,
                "start-showing-local",
                count = slots.len(),
                name = &profile.name
            ),
        };
        let folder = profile.dir.join("save");
        v_flex()
            .gap_1()
            .items_start()
            .child(div().text_sm().child(text))
            // The folder itself, named in full in the tooltip and in the list of places below.
            .child(
                Button::new("show-profile-folder")
                    .ghost()
                    .xsmall()
                    .icon(Icon::new(IconName::FolderOpen))
                    .label(t!(cx, "start-folder-show"))
                    .tip(SharedString::from(folder.to_string_lossy().into_owned()))
                    .on_click(move |_, _, cx| cx.open_with_system(&folder)),
            )
    }

    /// A profile without saves: how to make one.
    fn render_empty_profile(&self, profile: &Profile, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_2()
            .child(div().font_medium().child(t!(
                cx,
                "start-profile-empty-title",
                name = &profile.name
            )))
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(t!(cx, "start-profile-empty-body")),
            )
    }

    /// Every folder looked in and what it held, a way to look again, and to add a folder.
    fn render_folders(&self, found: &Found, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        // Only the ones added here can be removed here (not ones from `TSE_PROFILE_DIRS`).
        let added = crate::settings::get(cx).profile_dirs;
        let row = |index: usize, folder: &SearchedFolder, cx: &mut Context<Self>| {
            let name = match folder.source {
                Source::Added => t!(cx, "start-folder-added"),
                Source::Steam | Source::Documents => labels::location(cx, folder.location),
            };
            let (status, has_profiles) = if !folder.is_there {
                (t!(cx, "start-folder-missing"), false)
            } else if folder.profiles == 0 {
                (t!(cx, "start-folder-empty"), false)
            } else {
                (
                    t!(cx, "start-folder-profiles", count = folder.profiles),
                    true,
                )
            };
            let dir = folder.dir.clone();
            folder_row(
                has_profiles,
                name,
                status,
                Some(path_text(&folder.dir, cx).into_any_element()),
                cx,
            )
            .when(folder.is_there, |row| {
                row.child(show_folder_button(("show-folder", index), dir.clone(), cx))
            })
            .when(added.contains(&folder.dir), |row| {
                let dir = dir.clone();
                row.child(
                    Button::new(("remove-folder", index))
                        .ghost()
                        .xsmall()
                        .icon(Icon::new(IconName::Close))
                        .tip(t!(cx, "start-folder-remove"))
                        .accessibility_label(t!(cx, "start-folder-remove"))
                        .on_click(
                            cx.listener(move |this, _, _, cx| this.remove_folder(dir.clone(), cx)),
                        ),
                )
            })
        };
        let mut rows: Vec<gpui_kit::AnyElement> = Vec::new();
        if !found.has_steam {
            rows.push(
                folder_row(
                    false,
                    t!(cx, "location-steam-cloud"),
                    t!(cx, "start-folder-no-steam"),
                    None,
                    cx,
                )
                .into_any_element(),
            );
        }
        // With saves found, folders that aren't there are only noise (another Steam account on
        // this PC, say), so they fold into one line; with none found, each one is part of the
        // answer.
        let folds_missing = !found.profiles.is_empty() && !self.shows_missing_folders;
        // The picked game's folders; with nothing found for either game, both games' (either
        // could be the answer), each folder once (an added folder is looked in for both).
        let picked = self.picked_game(&found.profiles);
        let mut listed: Vec<&std::path::Path> = Vec::new();
        let mut missing = 0;
        for (index, folder) in found.folders.iter().enumerate() {
            let is_picked_game = picked.is_none_or(|game| folder.game == game);
            if !is_picked_game || listed.contains(&folder.dir.as_path()) {
                continue;
            }
            listed.push(&folder.dir);
            if folds_missing && !folder.is_there && !added.contains(&folder.dir) {
                missing += 1;
                continue;
            }
            rows.push(row(index, folder, cx).into_any_element());
        }
        if missing > 0 || (self.shows_missing_folders && !found.profiles.is_empty()) {
            let label = if missing > 0 {
                t!(cx, "start-folders-missing", count = missing)
            } else {
                t!(cx, "start-folders-missing-hide")
            };
            rows.push(
                h_flex()
                    .px_1()
                    .child(
                        Button::new("missing-folders")
                            .ghost()
                            .xsmall()
                            .icon(Icon::new(if missing > 0 {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronUp
                            }))
                            .label(label)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.shows_missing_folders = !this.shows_missing_folders;
                                cx.notify();
                            })),
                    )
                    .into_any_element(),
            );
        }

        v_flex()
            .gap_3()
            .child(
                h_flex()
                    .px_3()
                    .gap_2()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_xs()
                            .font_medium()
                            .text_color(muted)
                            .child(t!(cx, "start-folders")),
                    )
                    .child(
                        Button::new("look-again")
                            .ghost()
                            .xsmall()
                            .icon(Icon::new(IconName::RefreshCw))
                            .label(t!(cx, "start-look-again"))
                            .on_click(cx.listener(|this, _, _, cx| this.search(cx))),
                    ),
            )
            .child(
                div()
                    .px_3()
                    .text_xs()
                    .text_color(muted)
                    .child(t!(cx, "start-folders-description")),
            )
            .child(v_flex().gap_1().children(rows))
            .child(
                h_flex()
                    .px_3()
                    .gap_3()
                    .flex_wrap()
                    .items_center()
                    .child(
                        Button::new("add-folder")
                            .outline()
                            .small()
                            .icon(Icon::new(IconName::FolderOpen))
                            .label(t!(cx, "start-folder-add"))
                            .on_click(cx.listener(|this, _, _, cx| this.choose_folder(cx))),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted)
                            .child(t!(cx, "start-folder-drop")),
                    ),
            )
            .children(self.add_problem.clone().map(|problem| {
                div()
                    .px_3()
                    .text_sm()
                    .text_color(cx.theme().danger)
                    .child(problem)
            }))
    }

    fn render_slots(
        &self,
        profile: &Profile,
        slots: &[Slot],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (own, mut game_made): (Vec<&Slot>, Vec<&Slot>) =
            slots.iter().partition(|slot| slot.kind.is_manual());
        // The game keeps dozens of autosaves; the newest few are the ones worth going back to,
        // and the rest would push everything after them far down.
        let hidden = if self.shows_all_game_saves {
            0
        } else {
            game_made.len().saturating_sub(GAME_SAVES_SHOWN)
        };
        game_made.truncate(game_made.len() - hidden);
        let can_fold = self.shows_all_game_saves && game_made.len() > GAME_SAVES_SHOWN;
        let currency = profile.game.currency_symbol();

        let group = |title: SharedString,
                     hint: Option<SharedString>,
                     slots: Vec<&Slot>,
                     cx: &mut Context<Self>| {
            v_flex()
                .gap_1()
                .child(
                    h_flex()
                        .px_3()
                        .pb_1()
                        .gap_2()
                        .flex_wrap()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(div().font_medium().child(title))
                        .children(hint),
                )
                .children(slots.into_iter().map(|slot| {
                    let detail = {
                        let mut parts = Vec::new();
                        if let save::SlotKind::Manual(number) = slot.kind {
                            parts.push(t!(cx, "start-slot-number", number = number).to_string());
                        }
                        if let Some(modified) = slot.modified {
                            parts.push(labels::ago(cx, modified).to_string());
                        }
                        parts.join(" · ")
                    };
                    // What the save needs that this PC doesn't have: the game won't load it.
                    // (Unknown until the game's data is read; nothing is said then.)
                    let missing = crate::game_data::installed(cx).map(|installed| {
                        slot.info
                            .requirements()
                            .iter()
                            .filter(|r| r.kind != save::RequirementKind::RemovableDlc)
                            .filter(|r| !installed.has(r))
                            .count()
                    });
                    let money = slot
                        .info
                        .money
                        .map(|money| format!("{currency} {}", thousands(money)))
                        .unwrap_or_default();
                    let open = OpenSlot {
                        profile: profile.clone(),
                        slot: slot.clone(),
                    };
                    let open_by_key = OpenSlot {
                        profile: profile.clone(),
                        slot: slot.clone(),
                    };
                    let ring = cx.theme().ring;
                    ListItem::new(SharedString::from(format!("slot-{}", slot.folder)))
                        .py_2()
                        .px_3()
                        .rounded(cx.theme().radius)
                        .accessibility_label(labels::slot_name(cx, slot))
                        // Tab reaches each row; Enter or Space opens it. The ring shows only for
                        // keyboard focus, not after a click.
                        .when_some(self.row_focus.get(&slot.dir), |row, focus| {
                            row.track_focus(focus)
                        })
                        // Always a border, clear until focused, so focus doesn't move anything.
                        .border_1()
                        .border_color(gpui_kit::transparent_black())
                        .focus_visible(|style| style.border_color(ring))
                        .on_key_down(cx.listener(move |_, event: &KeyDownEvent, _, cx| {
                            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                                cx.emit(open_by_key.clone());
                                cx.stop_propagation();
                            }
                        }))
                        .on_click(cx.listener(move |_, _, _, cx| {
                            cx.emit(open.clone());
                        }))
                        .child(
                            h_flex()
                                .w_full()
                                .justify_between()
                                .gap_4()
                                .child(
                                    h_flex()
                                        .gap_3()
                                        .min_w_0()
                                        .child(thumbnail(self.previews.get(&slot.dir).cloned(), cx))
                                        .child(
                                            v_flex()
                                                .min_w_0()
                                                .child(
                                                    div()
                                                        .text_sm()
                                                        .font_medium()
                                                        .child(labels::slot_name(cx, slot)),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(detail),
                                                )
                                                .children(missing.filter(|count| *count > 0).map(
                                                    |count| {
                                                        h_flex()
                                                            .gap_1()
                                                            .text_xs()
                                                            .text_color(cx.theme().danger)
                                                            .child(
                                                                Icon::new(IconName::TriangleAlert)
                                                                    .xsmall(),
                                                            )
                                                            .child(t!(
                                                                cx,
                                                                "start-slot-missing",
                                                                count = count
                                                            ))
                                                    },
                                                )),
                                        ),
                                )
                                .child(
                                    h_flex()
                                        .flex_shrink_0()
                                        .gap_2()
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(money),
                                        )
                                        .child(duplicate_button(profile, slot, cx)),
                                ),
                        )
                }))
        };

        v_flex()
            .gap_6()
            .when(!own.is_empty(), |this| {
                this.child(group(t!(cx, "start-your-saves"), None, own, cx))
            })
            .when(!game_made.is_empty(), |this| {
                this.child(group(
                    t!(cx, "start-game-saves"),
                    Some(t!(cx, "start-game-saves-hint")),
                    game_made,
                    cx,
                ))
            })
            .when(hidden > 0 || can_fold, |this| {
                let label = if hidden > 0 {
                    t!(cx, "start-game-saves-more", count = hidden)
                } else {
                    t!(cx, "start-game-saves-fewer")
                };
                this.child(
                    h_flex().px_1().mt(rems(-1.)).child(
                        Button::new("more-game-saves")
                            .ghost()
                            .small()
                            .icon(Icon::new(if hidden > 0 {
                                IconName::ChevronDown
                            } else {
                                IconName::ChevronUp
                            }))
                            .label(label)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.shows_all_game_saves = !this.shows_all_game_saves;
                                cx.notify();
                            })),
                    ),
                )
            })
    }
}

impl Render for OpenSavePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = h_flex()
            .gap_3()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size_10()
                    .flex_shrink_0()
                    .rounded(cx.theme().radius_lg)
                    .bg(cx.theme().primary)
                    .text_color(cx.theme().primary_foreground)
                    .child(Icon::new(IconName::Truck)),
            )
            .child(
                v_flex()
                    .flex_1()
                    .child(div().text_lg().font_semibold().child(APP_NAME))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(t!(cx, "start-choose")),
                    ),
            )
            // Both settings before a save is open too: the editor's (its language), and the
            // game's (its folder, if it wasn't found).
            .child(
                Button::new("start-game-settings")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::Gamepad2))
                    .label(t!(cx, "game-settings-open"))
                    .on_click(|_, window, cx| crate::pages::game_settings::open_sheet(window, cx)),
            )
            .child(
                Button::new("start-settings")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::Settings))
                    .label(t!(cx, "settings-open"))
                    .on_click(|_, window, cx| crate::pages::settings::open_sheet(window, cx)),
            );

        let body = match &self.found {
            None => h_flex()
                .px_3()
                .gap_2()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(Spinner::new().small())
                .child(t!(cx, "start-looking"))
                .into_any_element(),
            Some(found) if found.profiles.is_empty() => v_flex()
                .gap_8()
                .child(
                    v_flex()
                        .px_3()
                        .gap_2()
                        .child(div().font_medium().child(t!(cx, "start-none-title")))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(t!(cx, "start-none-body")),
                        )
                        // Still stuck: a way to say so.
                        .child(
                            h_flex().child(
                                Button::new("start-report")
                                    .ghost()
                                    .small()
                                    .icon(Icon::new(IconName::ExternalLink))
                                    .label(t!(cx, "report-problem"))
                                    .on_click(|_, _, cx| crate::report::open(cx, None)),
                            ),
                        ),
                )
                .child(self.render_folders(found, cx))
                .into_any_element(),
            Some(found) => {
                let profiles = &found.profiles;
                let selected = self.selected_profile.min(profiles.len() - 1);
                let (profile, slots) = &profiles[selected];
                let tools_profile = profile.clone();
                let all_profiles: Vec<Profile> = profiles
                    .iter()
                    .map(|(profile, _)| profile.clone())
                    .collect();
                let start = cx.entity().downgrade();
                let tools = Button::new("profile-tools")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::User))
                    .label(t!(cx, "profile-tools-open"))
                    .on_click(move |_, window, cx| {
                        crate::pages::profile_tools::open(
                            tools_profile.clone(),
                            all_profiles.clone(),
                            start.clone(),
                            window,
                            cx,
                        )
                    });
                // What's shown (or that there's nothing yet), with the tools for that profile.
                let about = if slots.is_empty() {
                    self.render_empty_profile(profile, cx).into_any_element()
                } else {
                    self.render_summary(profile, slots, cx).into_any_element()
                };
                // More than one profile of the chosen game: which one.
                let profiles_of_game = profiles
                    .iter()
                    .filter(|(other, _)| other.game == profile.game)
                    .count();
                v_flex()
                    .gap_6()
                    // Across the column's full width, like the save rows and the warning below.
                    .child(self.render_games(profiles, cx))
                    .when(profiles_of_game > 1, |this| {
                        this.child(div().px_3().child(self.render_profiles(profiles, cx)))
                    })
                    // Before opening one: what isn't supported for this game yet.
                    .when(profile.game != Game::Ets2, |this| {
                        this.child(
                            Alert::warning("unsupported-game", t!(cx, "ats-warning"))
                                .text_color(cx.theme().foreground)
                                .icon(
                                    Icon::new(IconName::TriangleAlert)
                                        .text_color(cx.theme().warning),
                                ),
                        )
                    })
                    .child(
                        h_flex()
                            .px_3()
                            .gap_4()
                            .justify_between()
                            .items_start()
                            .child(div().flex_1().min_w_0().child(about))
                            .child(tools),
                    )
                    .when(!slots.is_empty(), |this| {
                        this.child(self.render_slots(profile, slots, cx))
                    })
                    .child(div().mx_3().h_px().bg(cx.theme().border))
                    .child(self.render_folders(found, cx))
                    .into_any_element()
            }
        };

        div()
            .id("open-save-scroll")
            .size_full()
            // A folder with saves dropped anywhere on the window is added.
            .drag_over::<ExternalPaths>(|style, _, _, cx| style.bg(cx.theme().secondary))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, _, cx| {
                this.add_folders(paths.paths().to_vec(), cx)
            }))
            .overflow_y_scroll()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                v_flex()
                    .w_full()
                    .max_w(rems(40.))
                    .mx_auto()
                    .px_8()
                    .py_12()
                    .gap_8()
                    // Everything on the slot rows' text edge: the rows pad their content so their
                    // hover fill reaches past it; the header and the rest line up with that text.
                    .child(div().px_3().child(header))
                    .children(self.render_crash(cx))
                    .children(crate::updates::available(cx).map(|(version, url)| {
                        // A newer version: one quiet line with the way to get it.
                        h_flex()
                            .px_3()
                            .gap_3()
                            .flex_wrap()
                            .items_center()
                            .text_sm()
                            .child(t!(cx, "updates-available", version = version))
                            .child(
                                Button::new("start-update")
                                    .small()
                                    .outline()
                                    .icon(Icon::new(IconName::ArrowDown))
                                    .label(t!(cx, "updates-download"))
                                    .on_click(move |_, _, cx| cx.open_url(&url)),
                            )
                    }))
                    .child(body),
            )
    }
}

/// A preview picture as GPUI draws it (the decoder already gives blue-green-red-alpha order).
/// A save row's Duplicate button: off, with the reason, where a copy can't be made.
fn duplicate_button(
    profile: &Profile,
    slot: &Slot,
    cx: &mut Context<OpenSavePage>,
) -> impl IntoElement {
    let why_not = crate::duplicate::why_not(profile, cx);
    let label = t!(cx, "duplicate-tooltip");
    let (profile, slot) = (profile.clone(), slot.clone());
    // When a copy is written, the list shows it.
    let page = cx.entity().downgrade();
    let on_done: crate::duplicate::OnDone = std::rc::Rc::new(move |_, cx| {
        let _ = page.update(cx, |page, cx| page.refresh(cx));
    });
    Button::new(SharedString::from(format!("duplicate-{}", slot.folder)))
        .ghost()
        .xsmall()
        .icon(Icon::new(IconName::Copy))
        .accessibility_label(label.clone())
        .tip(why_not.clone().unwrap_or(label))
        .disabled(why_not.is_some())
        .on_click(move |_, window, cx| {
            // The row opens the save when clicked; this button doesn't.
            cx.stop_propagation();
            crate::duplicate::open(profile.clone(), slot.clone(), on_done.clone(), window, cx);
        })
}

fn render_image(picture: save::Picture) -> Option<RenderImage> {
    let buffer = image::RgbaImage::from_raw(picture.width, picture.height, picture.bgra)?;
    Some(RenderImage::new(vec![image::Frame::new(buffer)]))
}

/// A slot's preview at the size of the game's own thumbnails, or a quiet placeholder.
fn thumbnail(preview: Option<Arc<RenderImage>>, cx: &Context<OpenSavePage>) -> impl IntoElement {
    let frame = div()
        .w(rems(6.))
        .h(rems(3.))
        .flex_shrink_0()
        .rounded(cx.theme().radius)
        .overflow_hidden()
        .bg(cx.theme().muted)
        .border_1()
        .border_color(cx.theme().border);
    match preview {
        Some(preview) => frame.child(img(preview).size_full().object_fit(ObjectFit::Cover)),
        None => frame
            .flex()
            .items_center()
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .child(Icon::new(IconName::Truck).small()),
    }
}

/// A folder added on the start screen, by name: its own, or the one around it when it's the
/// game's usual `profiles` folder (`ETS2 save backups\profiles` reads as `ETS2 save backups`).
fn added_folder_name(root: &std::path::Path) -> String {
    let name = |path: &std::path::Path| {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
    };
    match name(root) {
        Some(own)
            if own.eq_ignore_ascii_case("profiles")
                || own.eq_ignore_ascii_case("steam_profiles") =>
        {
            root.parent().and_then(name).unwrap_or(own)
        }
        Some(own) => own,
        None => root.to_string_lossy().into_owned(),
    }
}

/// A path as the start screen shows it: small, in the code font, wrapping where it must.
fn path_text(path: &std::path::Path, cx: &Context<OpenSavePage>) -> impl IntoElement {
    div()
        .min_w_0()
        .text_xs()
        .font_family(cx.theme().mono_font_family.clone())
        .text_color(cx.theme().muted_foreground)
        .child(path.to_string_lossy().into_owned())
}

/// Opens the folder itself in the system's file manager (revealing would open the one around
/// it, with this one selected).
fn show_folder_button(
    id: impl Into<gpui_kit::ElementId>,
    dir: PathBuf,
    cx: &Context<OpenSavePage>,
) -> Button {
    Button::new(id)
        .ghost()
        .xsmall()
        .icon(Icon::new(IconName::FolderOpen))
        .tip(t!(cx, "start-folder-show"))
        .accessibility_label(t!(cx, "start-folder-show"))
        .on_click(move |_, _, cx| cx.open_with_system(&dir))
}

/// One place looked in: whether it held profiles (a check, or a dash), its name, what was there,
/// and its path. Buttons go after, as children.
fn folder_row(
    has_profiles: bool,
    name: SharedString,
    status: SharedString,
    path: Option<gpui_kit::AnyElement>,
    cx: &Context<OpenSavePage>,
) -> gpui_kit::Div {
    let muted = cx.theme().muted_foreground;
    h_flex()
        .px_3()
        .py_1()
        .gap_3()
        .items_start()
        .child(
            div().pt(rems(0.125)).child(
                Icon::new(if has_profiles {
                    IconName::CircleCheck
                } else {
                    IconName::Dash
                })
                .small()
                .text_color(if has_profiles {
                    cx.theme().success
                } else {
                    muted
                }),
            ),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .child(
                    h_flex()
                        .gap_2()
                        .flex_wrap()
                        .text_sm()
                        .child(div().font_medium().child(name))
                        .child(div().text_color(muted).child(status)),
                )
                .children(path),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::TestAppContext;

    /// A made-up profile folder with one save, in the system's temporary folder.
    struct Scratch(PathBuf);

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn names_added_folders_by_what_they_hold() {
        assert_eq!(
            added_folder_name(std::path::Path::new("/docs/ETS2 save backups")),
            "ETS2 save backups"
        );
        assert_eq!(
            added_folder_name(std::path::Path::new("/d/Old PC/profiles")),
            "Old PC"
        );
    }

    #[gpui_kit::test]
    fn adds_the_folder_a_save_is_in_and_lands_on_its_profile(cx: &mut TestAppContext) {
        let scratch =
            Scratch(std::env::temp_dir().join(format!("tse-start-{}", std::process::id())));
        let root = scratch.0.join("profiles");
        let profile = root.join("416E6E61");
        let slot = profile.join("save").join("2");
        std::fs::create_dir_all(&slot).expect("makes the slot");
        std::fs::write(slot.join("game.sii"), "").expect("writes a save");

        cx.update(|cx| {
            gpui_kit::init(cx);
            crate::i18n::init(cx);
        });
        let page = cx.update(|cx| cx.new(OpenSavePage::new));
        cx.run_until_parked();

        // Something that isn't saves: said so, nothing added.
        let elsewhere = scratch.0.join("elsewhere");
        std::fs::create_dir_all(&elsewhere).expect("makes a folder");
        page.update(cx, |page, cx| page.add_folders(vec![elsewhere], cx));
        cx.run_until_parked();
        cx.update(|cx| {
            assert!(page.read(cx).add_problem.is_some());
            assert!(crate::settings::get(cx).profile_dirs.is_empty());
        });

        // A save on its own (copied to the desktop): opened where it is, nothing added (not the
        // folders around it).
        let opened = std::rc::Rc::new(std::cell::RefCell::new(None));
        cx.update(|cx| {
            let opened = opened.clone();
            cx.subscribe(&page, move |_, event: &OpenSlot, _| {
                *opened.borrow_mut() = Some((event.profile.location, event.slot.dir.clone()));
            })
            .detach();
        });
        let loose = scratch.0.join("desktop").join("13");
        std::fs::create_dir_all(&loose).expect("makes the loose save");
        std::fs::write(loose.join("game.sii"), "").expect("writes a save");
        page.update(cx, |page, cx| page.add_folders(vec![loose.clone()], cx));
        cx.run_until_parked();
        assert_eq!(*opened.borrow(), Some((save::Location::Loose, loose)));
        cx.update(|cx| {
            assert!(page.read(cx).add_problem.is_none());
            assert!(crate::settings::get(cx).profile_dirs.is_empty());
        });

        // One save, chosen: its profiles folder is added, and its profile shown.
        page.update(cx, |page, cx| page.add_folders(vec![slot.clone()], cx));
        cx.run_until_parked();
        cx.update(|cx| {
            assert_eq!(
                crate::settings::get(cx).profile_dirs,
                std::slice::from_ref(&root)
            );
            let page = page.read(cx);
            assert!(page.add_problem.is_none());
            let found = page.found.as_ref().expect("looked");
            let (shown, slots) = &found.profiles[page.selected_profile];
            assert_eq!(shown.name, "Anna");
            assert_eq!(slots.len(), 1);
            assert!(found.folders.iter().any(|folder| folder.dir == root
                && folder.source == Source::Added
                && folder.profiles == 1));
        });

        // Removed again: no longer looked in.
        page.update(cx, |page, cx| page.remove_folder(root.clone(), cx));
        cx.run_until_parked();
        cx.update(|cx| {
            assert!(crate::settings::get(cx).profile_dirs.is_empty());
            let found = page.read(cx).found.as_ref().expect("looked");
            assert!(!found.folders.iter().any(|folder| folder.dir == root));
        });
    }
}
