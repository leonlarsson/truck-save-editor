//! The Gallery: every viewpoint and photo trophy of the game (its photo album, see
//! `gamedata::PhotoAlbum`), and the player's progress. Viewpoints are in the save (visited, found
//! but not watched, or not found yet); photo trophies are captured per profile, in its album
//! (read only: a capture is a photo the player took, which the editor doesn't make up).
//!
//! A table lists them all, each opening its panel (double-click, Enter, or its Details button):
//! there a viewpoint is marked visited or not found, and either is shown on World map, which
//! marks them too.

use std::collections::HashMap;

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, WindowExt as _,
    button::Button,
    h_flex,
    table::{ColumnSort, DataTable, TableEvent, TableState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Subscription, Window, div, px,
};

use crate::game_data;
use crate::i18n::t;
use crate::session::Session;
use crate::ui::Tip as _;
use crate::ui::{FilterBar, FilterChanged, RowLook, TextColumn, TextRow, TextTable};
use crate::workspace::OpenDetails;

/// A viewpoint or a photo trophy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GalleryKind {
    Viewpoint,
    Landmark,
}

/// Where one stands: a viewpoint's state, or whether a photo trophy is captured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Progress {
    Visited,
    /// A viewpoint found on the map, not watched.
    Found,
    NotFound,
    Captured,
    NotCaptured,
}

impl Progress {
    pub fn is_done(self) -> bool {
        matches!(self, Progress::Visited | Progress::Captured)
    }
}

/// One viewpoint or photo trophy, as the Gallery and the map show it.
#[derive(Clone, Debug)]
pub struct GalleryEntry {
    pub kind: GalleryKind,
    pub item: gamedata::AlbumItem,
    pub progress: Progress,
    /// Where it is on the map (game metres), once the map is read.
    pub position: Option<(f32, f32)>,
}

impl GalleryEntry {
    /// A viewpoint's cutscene item: what the save lists.
    pub fn viewpoint_item(&self) -> Option<u64> {
        (self.kind == GalleryKind::Viewpoint)
            .then(|| self.item.map_items.first().copied())
            .flatten()
    }

    /// The same entry's key across redraws (kind and id).
    pub fn key(&self) -> (GalleryKind, String) {
        (self.kind, self.item.id.clone())
    }
}

/// Every viewpoint and photo trophy the save may have (the base game's and its map DLCs'), with
/// the player's progress and, once the map is read, where each is. Empty without the game.
pub fn entries(session: &Session, cx: &App) -> Vec<GalleryEntry> {
    let Some(album) = game_data::photo_album(cx) else {
        return Vec::new();
    };
    let states = session.viewpoint_states();
    let captured = session.captured_landmarks();
    let mut viewpoint_at: HashMap<u64, (f32, f32)> = HashMap::new();
    let mut landmark_at: HashMap<String, (f32, f32)> = HashMap::new();
    for marker in game_data::map_markers(cx)
        .iter()
        .flat_map(|markers| markers.iter())
    {
        match marker {
            gamedata::MapMarker::Viewpoint { item, x, z } => {
                viewpoint_at.insert(*item, (*x, *z));
            }
            gamedata::MapMarker::Landmark { id, x, z } => {
                landmark_at.insert(id.clone(), (*x, *z));
            }
        }
    }
    let viewpoints = album
        .viewpoints
        .into_iter()
        .filter(|item| session.has_dlc(item.dlc.as_deref()))
        .map(|item| {
            let first = item.map_items.first().copied();
            let progress = match first.and_then(|id| states.get(&id)) {
                Some(save::ViewpointState::Visited) => Progress::Visited,
                Some(save::ViewpointState::Found) => Progress::Found,
                None => Progress::NotFound,
            };
            GalleryEntry {
                kind: GalleryKind::Viewpoint,
                position: first.and_then(|id| viewpoint_at.get(&id).copied()),
                progress,
                item,
            }
        });
    let landmarks = album
        .landmarks
        .into_iter()
        .filter(|item| session.has_dlc(item.dlc.as_deref()))
        .map(|item| GalleryEntry {
            kind: GalleryKind::Landmark,
            position: landmark_at.get(&item.id).copied(),
            progress: if captured.contains(&item.id) {
                Progress::Captured
            } else {
                Progress::NotCaptured
            },
            item,
        });
    viewpoints.chain(landmarks).collect()
}

pub fn kind_label(cx: &App, kind: GalleryKind) -> SharedString {
    match kind {
        GalleryKind::Viewpoint => t!(cx, "gallery-viewpoint"),
        GalleryKind::Landmark => t!(cx, "gallery-photo-trophy"),
    }
}

pub fn progress_label(cx: &App, progress: Progress) -> SharedString {
    match progress {
        Progress::Visited => t!(cx, "gallery-visited"),
        Progress::Found => t!(cx, "gallery-found"),
        Progress::NotFound => t!(cx, "gallery-not-found"),
        Progress::Captured => t!(cx, "gallery-captured"),
        Progress::NotCaptured => t!(cx, "gallery-not-captured"),
    }
}

/// The filter's toggles, in order.
const FILTER_VIEWPOINTS: usize = 0;
const FILTER_LANDMARKS: usize = 1;
const FILTER_TO_DO: usize = 2;

/// The Gallery page: the counts, Visit every viewpoint, and the table.
pub struct GalleryView {
    session: Entity<Session>,
    table: Entity<TableState<TextTable>>,
    filter: Entity<FilterBar>,
    /// The entries as the table shows them, row by row.
    shown: Vec<GalleryEntry>,
    _subscriptions: Vec<Subscription>,
}

impl GalleryView {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let columns = vec![
            TextColumn {
                key: "name",
                title: t!(cx, "gallery-column-name"),
                width: 240.,
            },
            TextColumn {
                key: "kind",
                title: t!(cx, "gallery-column-kind"),
                width: 130.,
            },
            TextColumn {
                key: "country",
                title: t!(
                    cx,
                    "gallery-column-country",
                    game = crate::labels::current_game_key(cx)
                ),
                width: 150.,
            },
            TextColumn {
                key: "progress",
                title: t!(cx, "gallery-column-progress"),
                width: 160.,
            },
        ];
        let page = cx.entity().downgrade();
        let details = t!(cx, "details-open");
        let table = cx.new(|cx| {
            let rows = TextTable::new(columns)
                .with_picks()
                .with_sorting()
                .with_details(details, move |row, window, cx| {
                    let _ = page.update(cx, |page, cx| page.open_row(row, window, cx));
                });
            TableState::new(rows, window, cx)
        });
        let toggles = vec![
            t!(cx, "gallery-filter-viewpoints"),
            t!(cx, "gallery-filter-photo-trophies"),
            t!(cx, "gallery-filter-to-do"),
        ];
        let placeholder = t!(
            cx,
            "gallery-search",
            game = crate::labels::current_game_key(cx)
        );
        let filter = cx.new(|cx| FilterBar::new(placeholder, toggles, window, cx));
        let _subscriptions = vec![
            cx.observe(&session, |this, _, cx| this.reload(cx)),
            game_data::observe(cx, |this, cx| this.reload(cx)),
            cx.subscribe(&filter, |this, _, _: &FilterChanged, cx| this.reload(cx)),
            // Picking a row shows the bar of what can be done to the picked viewpoints.
            cx.observe(&table, |_, _, cx| cx.notify()),
            cx.subscribe_in(
                &table,
                window,
                |this, table, event: &TableEvent, window, cx| {
                    match event {
                        TableEvent::DoubleClickedRow(row) => this.open_row(*row, window, cx),
                        // A click on a column's heading sorts by it (the table itself would select
                        // the column, which nothing here uses).
                        TableEvent::SelectColumn(col_ix) => {
                            let col_ix = *col_ix;
                            table.update(cx, |table, cx| {
                                table.delegate_mut().sort_by_header(col_ix);
                                table.clear_selection(cx);
                            });
                            this.reload(cx);
                        }
                        _ => {}
                    }
                },
            ),
        ];
        let mut view = Self {
            session,
            table,
            filter,
            shown: Vec::new(),
            _subscriptions,
        };
        view.reload(cx);
        view
    }

    /// Makes the rows again (the save, the filter or the game's data changed).
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        let all = entries(self.session.read(cx), cx);
        let total = all.len();
        let filter = self.filter.read(cx);
        let (viewpoints, landmarks, to_do) = (
            filter.is_on(FILTER_VIEWPOINTS),
            filter.is_on(FILTER_LANDMARKS),
            filter.is_on(FILTER_TO_DO),
        );
        let mut shown: Vec<(GalleryEntry, [SharedString; 4])> = all
            .into_iter()
            .filter(|entry| match entry.kind {
                // Neither toggle on: both kinds.
                GalleryKind::Viewpoint => viewpoints || !landmarks,
                GalleryKind::Landmark => landmarks || !viewpoints,
            })
            .filter(|entry| !to_do || !entry.progress.is_done())
            .map(|entry| {
                let cells: [SharedString; 4] = [
                    game_data::album_item_name(cx, &entry.item).into(),
                    kind_label(cx, entry.kind),
                    entry
                        .item
                        .group
                        .as_deref()
                        .map(|group| game_data::resolve(cx, group))
                        .unwrap_or_default()
                        .into(),
                    progress_label(cx, entry.progress),
                ];
                (entry, cells)
            })
            .filter(|(_, cells)| {
                let texts: Vec<&str> = cells.iter().map(|cell| cell.as_ref()).collect();
                filter.matches(&texts, cx)
            })
            .collect();
        // By country, then name; then by the column whose heading was clicked, if one was
        // (rows that are equal there keep that order).
        shown.sort_by(|a, b| a.1[2].cmp(&b.1[2]).then(a.1[0].cmp(&b.1[0])));
        if let Some((column, order)) = self.table.read(cx).delegate().sort {
            shown.sort_by(|a, b| {
                let ordering = a.1.get(column).cmp(&b.1.get(column));
                if order == ColumnSort::Descending {
                    ordering.reverse()
                } else {
                    ordering
                }
            });
        }
        let rows: Vec<TextRow> = shown
            .iter()
            .map(|(entry, cells)| TextRow {
                cells: cells.to_vec(),
                look: if entry.progress.is_done() {
                    RowLook::Normal
                } else {
                    RowLook::Muted
                },
            })
            .collect();
        let count = shown.len();
        self.shown = shown.into_iter().map(|(entry, _)| entry).collect();
        self.filter
            .update(cx, |filter, cx| filter.set_counts(count, total, cx));
        // Viewpoints can be picked (by their item in the save); photo trophies are the player's
        // own photos, which the editor only shows, so their rows have no checkbox.
        let pick_ids: Vec<Option<String>> = self
            .shown
            .iter()
            .map(|entry| entry.viewpoint_item().map(|item| item.to_string()))
            .collect();
        self.table.update(cx, |table, cx| {
            table.delegate_mut().pick_ids = pick_ids;
            table.delegate_mut().rows = rows;
            table.refresh(cx);
            cx.notify();
        });
        cx.notify();
    }

    /// Ticks the first rows' checkboxes (for `TSE_SCRIPT`'s `pick` step, to look at the bar).
    pub(crate) fn pick_first(&mut self, count: usize, cx: &mut Context<Self>) {
        self.table.update(cx, |table, cx| {
            let delegate = table.delegate_mut();
            let ids: Vec<String> = delegate
                .pick_ids
                .iter()
                .flatten()
                .take(count)
                .cloned()
                .collect();
            for id in &ids {
                if !delegate.picks.contains(id) {
                    delegate.picks.toggle(id);
                }
            }
            cx.notify();
        });
    }

    /// The bar above the table while viewpoints are picked: mark them all visited, or not
    /// found. Each is one change (one step for Undo).
    fn render_pick_bar(&self, cx: &mut Context<Self>) -> Option<crate::ui::PickBar> {
        let picks = self.table.read(cx).delegate().picks.clone();
        if picks.is_empty() {
            return None;
        }
        // The picks are the viewpoints' items, written as text.
        let items: Vec<u64> = picks
            .ids()
            .iter()
            .filter_map(|id| id.parse().ok())
            .collect();
        let table = self.table.clone();
        let (visit, forget) = (items.clone(), items);
        let bar = crate::ui::PickBar::new(picks.len(), move |_, cx| {
            table.update(cx, |table, cx| {
                table.delegate_mut().picks.clear();
                cx.notify();
            });
        })
        .action(
            Button::new("picks-visit")
                .small()
                .outline()
                .label(t!(cx, "gallery-visit"))
                .tip(t!(cx, "gallery-visit-tooltip"))
                .on_click(cx.listener(move |this, _, window, cx| {
                    let changed = this
                        .session
                        .update(cx, |session, cx| session.visit_viewpoints(&visit, cx));
                    crate::ui::picks_changed(changed, window, cx);
                })),
        )
        .action(
            Button::new("picks-forget")
                .small()
                .outline()
                .label(t!(cx, "gallery-forget"))
                .tip(t!(cx, "gallery-forget-tooltip"))
                .on_click(cx.listener(move |this, _, window, cx| {
                    let changed = this
                        .session
                        .update(cx, |session, cx| session.forget_viewpoints(&forget, cx));
                    crate::ui::picks_changed(changed, window, cx);
                })),
        );
        Some(bar)
    }

    fn open_row(&mut self, row: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(entry) = self.shown.get(row) {
            open(self.session.clone(), entry.key(), window, cx);
        }
    }
}

impl Render for GalleryView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let session = self.session.read(cx);
        let all = entries(session, cx);
        let count = |kind: GalleryKind| all.iter().filter(|e| e.kind == kind).count();
        let done = |kind: GalleryKind| {
            all.iter()
                .filter(|e| e.kind == kind && e.progress.is_done())
                .count()
        };
        let (viewpoints, visited) = (count(GalleryKind::Viewpoint), done(GalleryKind::Viewpoint));
        let (landmarks, captured) = (count(GalleryKind::Landmark), done(GalleryKind::Landmark));
        let has_game = game_data::photo_album(cx).is_some();
        let pick_bar = self.render_pick_bar(cx);
        v_flex()
            .size_full()
            .gap_4()
            .px_8()
            .py_6()
            // While viewpoints are picked, the bar of what can be done to them takes the
            // toolbar's place.
            .child(match pick_bar {
                Some(bar) => bar.into_any_element(),
                None => h_flex()
                    .justify_between()
                    .items_center()
                    .gap_4()
                    .flex_wrap()
                    .child(
                        h_flex()
                            .gap_x_4()
                            .flex_wrap()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(t!(
                                cx,
                                "gallery-viewpoints-summary",
                                visited = visited,
                                count = viewpoints
                            ))
                            .child(t!(
                                cx,
                                "gallery-photo-trophies-summary",
                                captured = captured,
                                count = landmarks
                            )),
                    )
                    .child(
                        Button::new("visit-every-viewpoint")
                            .small()
                            .outline()
                            .label(t!(cx, "gallery-visit-all"))
                            .tip(t!(cx, "gallery-visit-all-tooltip"))
                            .disabled(visited >= viewpoints)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.session
                                    .update(cx, |session, cx| session.visit_every_viewpoint(cx));
                            })),
                    )
                    .into_any_element(),
            })
            // Where each is kept, so a change here isn't expected in the profile's other saves.
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(t!(cx, "gallery-where-kept")),
            )
            .child(self.filter.clone())
            // Not hidden without the game: say why.
            .children((!has_game).then(|| {
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(t!(cx, "gallery-needs-game"))
            }))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    // Enter on the selected row opens it, like a double-click.
                    .on_action(cx.listener(|this, _: &OpenDetails, window, cx| {
                        if let Some(row) = this.table.read(cx).selected_row() {
                            this.open_row(row, window, cx);
                        }
                    }))
                    .child(crate::ui::table_or_empty(
                        &self.filter,
                        crate::ui::fill_table(
                            &self.table,
                            DataTable::new(&self.table)
                                .stripe(true)
                                .bordered(true)
                                .accessibility_label(t!(cx, "gallery-table"))
                                .into_any_element(),
                        ),
                        None,
                        cx,
                    )),
            )
    }
}

/// A viewpoint's or photo trophy's panel.
pub struct GalleryItemPanel {
    session: Entity<Session>,
    key: (GalleryKind, String),
    _subscriptions: Vec<Subscription>,
}

impl GalleryItemPanel {
    fn entry(&self, cx: &App) -> Option<GalleryEntry> {
        entries(self.session.read(cx), cx)
            .into_iter()
            .find(|entry| entry.key() == self.key)
    }
}

impl Render for GalleryItemPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(entry) = self.entry(cx) else {
            return div().child(t!(cx, "detail-gone")).into_any_element();
        };
        let fact = crate::pages::details::fact;
        let description = entry
            .item
            .description
            .as_deref()
            .map(|description| game_data::resolve(cx, description))
            .filter(|description| !description.is_empty());
        let country = entry
            .item
            .group
            .as_deref()
            .map(|group| game_data::resolve(cx, group))
            .unwrap_or_default();
        let country: SharedString = country.into();
        let viewpoint = entry.viewpoint_item();
        let action = viewpoint.map(|id| {
            if entry.progress == Progress::Visited || entry.progress == Progress::Found {
                Button::new("forget-viewpoint")
                    .small()
                    .outline()
                    .label(t!(cx, "gallery-forget"))
                    .tip(t!(cx, "gallery-forget-tooltip"))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.session
                            .update(cx, |session, cx| session.forget_viewpoint(id, cx));
                    }))
            } else {
                Button::new("visit-viewpoint")
                    .small()
                    .outline()
                    .label(t!(cx, "gallery-visit"))
                    .tip(t!(cx, "gallery-visit-tooltip"))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.session
                            .update(cx, |session, cx| session.visit_viewpoints(&[id], cx));
                    }))
            }
        });
        // A found viewpoint can also be marked watched.
        let watch = (viewpoint.is_some() && entry.progress == Progress::Found).then(|| {
            let id = viewpoint.unwrap_or_default();
            Button::new("watch-viewpoint")
                .small()
                .outline()
                .label(t!(cx, "gallery-visit"))
                .tip(t!(cx, "gallery-visit-tooltip"))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.session
                        .update(cx, |session, cx| session.visit_viewpoints(&[id], cx));
                }))
        });
        let show = match entry.position {
            Some((x, z)) => Button::new("show-on-map")
                .small()
                .outline()
                .label(t!(cx, "city-show-on-map"))
                .on_click(move |_, window, cx| {
                    window.close_sheet(cx);
                    crate::workspace::show_on_world_map(
                        crate::pages::world_map::MapSpot::Point(x, z),
                        cx,
                    );
                }),
            // Not hidden while the map is read (a few seconds after startup): say why.
            None => Button::new("show-on-map")
                .small()
                .outline()
                .label(t!(cx, "city-show-on-map"))
                .tip(t!(cx, "gallery-map-not-read"))
                .disabled(true),
        };
        v_flex()
            .pt_2()
            .gap_4()
            .children(description.map(|description| {
                v_flex()
                    .gap_2()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .children(paragraphs(&description))
            }))
            .child(
                v_flex()
                    .gap_1()
                    .child(fact(
                        t!(cx, "gallery-column-kind"),
                        kind_label(cx, entry.kind),
                        cx,
                    ))
                    .child(fact(
                        t!(
                            cx,
                            "gallery-column-country",
                            game = crate::labels::current_game_key(cx)
                        ),
                        country,
                        cx,
                    ))
                    .child(fact(
                        t!(cx, "gallery-column-progress"),
                        progress_label(cx, entry.progress),
                        cx,
                    )),
            )
            .child(
                h_flex()
                    .gap_2()
                    .flex_wrap()
                    .children(watch)
                    .children(action)
                    .child(show),
            )
            // Photo trophies are the player's own photos: shown, never made up.
            .children((entry.kind == GalleryKind::Landmark).then(|| {
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(t!(cx, "gallery-photo-trophy-note"))
            }))
            .into_any_element()
    }
}

/// The game's text as paragraphs: it breaks lines with `<br>` and may mark words up with other
/// tags (`<b>`), which are left out.
fn paragraphs(text: &str) -> Vec<String> {
    text.split("<br>")
        .map(|part| {
            let mut plain = String::new();
            let mut is_in_tag = false;
            for character in part.chars() {
                match character {
                    '<' => is_in_tag = true,
                    '>' if is_in_tag => is_in_tag = false,
                    _ if !is_in_tag => plain.push(character),
                    _ => {}
                }
            }
            plain.trim().to_string()
        })
        .filter(|part| !part.is_empty())
        .collect()
}

/// Opens a viewpoint's or photo trophy's panel, titled with its name.
pub fn open(
    session: Entity<Session>,
    key: (GalleryKind, String),
    window: &mut Window,
    cx: &mut App,
) {
    let title: SharedString = entries(session.read(cx), cx)
        .into_iter()
        .find(|entry| entry.key() == key)
        .map(|entry| game_data::album_item_name(cx, &entry.item))
        .unwrap_or_default()
        .into();
    let view = cx.new(|cx| {
        let _subscriptions = vec![cx.observe(&session, |_, _, cx| cx.notify())];
        GalleryItemPanel {
            session,
            key,
            _subscriptions,
        }
    });
    window.open_sheet(cx, move |sheet, _, _| {
        sheet
            .title(title.clone())
            .size(px(crate::pages::details::SHEET_WIDTH))
            .child(view.clone())
    });
}

#[cfg(test)]
mod tests {
    use super::paragraphs;

    #[test]
    fn splits_the_games_text_into_paragraphs() {
        assert_eq!(
            paragraphs("A <b>high</b> pass.<br><br>Snow in May."),
            ["A high pass.", "Snow in May."]
        );
    }
}
