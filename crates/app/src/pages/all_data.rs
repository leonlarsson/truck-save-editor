//! All data: every unit in the save and every value in it, editable. For anything the other pages
//! don't cover. Pick a kind of unit, then a unit, then a value; edit it below the table. Or search
//! every value at once and pick from what matches.
//!
//! Edits are checked against the value's shape: a number stays a number, text stays quoted text,
//! so a typo can't turn into something the game can't read.

use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Icon, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    table::{DataTable, TableEvent, TableState},
    v_flex,
};
use gpui_kit::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, prelude::FluentBuilder as _, rems,
};
use sii::inventory::{Shape, shape_of};

use crate::format::thousands;
use crate::i18n::t;
use crate::session::{FieldPath, Session};
use crate::ui::{RowLook, TextColumn, TextRow, TextTable, fold};

pub struct AllDataPage {
    session: Entity<Session>,
    filter: Entity<InputState>,
    /// Search across every value; while it holds text, its results replace the three panes.
    search: Entity<InputState>,
    results: Entity<TableState<TextTable>>,
    /// The values the results table shows, in its order.
    result_paths: Vec<FieldPath>,
    /// Whether there were more matches than shown.
    has_more_results: bool,
    /// Counts searches, so a slow, stale one doesn't overwrite a newer one's results.
    search_generation: usize,
    kinds: Entity<TableState<TextTable>>,
    units: Entity<TableState<TextTable>>,
    fields: Entity<TableState<TextTable>>,
    editor: Entity<InputState>,
    /// Every unit kind with its count, sorted by name.
    all_kinds: Vec<(String, usize)>,
    /// The kinds the table shows (after the filter), in table order.
    shown_kinds: Vec<String>,
    /// The chosen kind's unit ids, in table order.
    unit_ids: Vec<String>,
    /// The chosen unit's rows, in table order.
    field_rows: Vec<FieldRow>,
    /// The chosen unit's kind, for labels.
    unit_kind: String,
    selected: Option<FieldPath>,
    /// Why the last edit couldn't be applied.
    problem: Option<SharedString>,
    _subscriptions: Vec<Subscription>,
}

impl AllDataPage {
    pub fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut counts = std::collections::BTreeMap::<String, usize>::new();
        for unit in session.read(cx).preview().units() {
            *counts.entry(unit.kind.clone()).or_default() += 1;
        }
        let all_kinds: Vec<(String, usize)> = counts.into_iter().collect();

        let table = |columns: Vec<TextColumn>, window: &mut Window, cx: &mut Context<Self>| {
            cx.new(|cx| TableState::new(TextTable::new(columns), window, cx))
        };
        let kinds = table(
            vec![
                TextColumn {
                    key: "kind",
                    title: t!(cx, "all-data-kind"),
                    width: 160.,
                },
                TextColumn {
                    key: "count",
                    title: t!(cx, "all-data-count"),
                    width: 66.,
                },
            ],
            window,
            cx,
        );
        let units = table(
            vec![TextColumn {
                key: "unit",
                title: t!(cx, "all-data-unit"),
                width: 226.,
            }],
            window,
            cx,
        );
        let fields = table(
            vec![
                TextColumn {
                    key: "field",
                    title: t!(cx, "all-data-field"),
                    width: 220.,
                },
                TextColumn {
                    key: "value",
                    title: t!(cx, "all-data-value"),
                    width: 520.,
                },
            ],
            window,
            cx,
        );
        let filter =
            cx.new(|cx| InputState::new(window, cx).placeholder(t!(cx, "all-data-filter")));
        let search =
            cx.new(|cx| InputState::new(window, cx).placeholder(t!(cx, "all-data-search")));
        let results = table(
            vec![
                TextColumn {
                    key: "kind",
                    title: t!(cx, "all-data-kind"),
                    width: 170.,
                },
                TextColumn {
                    key: "unit",
                    title: t!(cx, "all-data-unit"),
                    width: 230.,
                },
                TextColumn {
                    key: "field",
                    title: t!(cx, "all-data-field"),
                    width: 200.,
                },
                TextColumn {
                    key: "value",
                    title: t!(cx, "all-data-value"),
                    width: 420.,
                },
            ],
            window,
            cx,
        );
        let editor = cx.new(|cx| InputState::new(window, cx));

        let _subscriptions = vec![
            cx.subscribe_in(&filter, window, |this, _, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    this.show_kinds(cx);
                }
            }),
            cx.subscribe_in(&search, window, |this, _, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    this.run_search(cx);
                }
            }),
            cx.subscribe_in(
                &results,
                window,
                |this, _, event: &TableEvent, window, cx| {
                    if let TableEvent::SelectRow(row) = event
                        && let Some(path) = this.result_paths.get(*row).cloned()
                    {
                        this.select_path(path, window, cx);
                    }
                },
            ),
            cx.subscribe_in(&kinds, window, |this, _, event: &TableEvent, _, cx| {
                if let TableEvent::SelectRow(row) = event
                    && let Some(kind) = this.shown_kinds.get(*row).cloned()
                {
                    this.show_units(&kind, cx);
                }
            }),
            cx.subscribe_in(&units, window, |this, _, event: &TableEvent, _, cx| {
                if let TableEvent::SelectRow(row) = event
                    && let Some(id) = this.unit_ids.get(*row).cloned()
                {
                    this.show_fields(&id, cx);
                }
            }),
            cx.subscribe_in(
                &fields,
                window,
                |this, _, event: &TableEvent, window, cx| {
                    if let TableEvent::SelectRow(row) = event {
                        this.select_field(*row, window, cx);
                    }
                },
            ),
            cx.subscribe_in(&editor, window, |this, _, event: &InputEvent, _, cx| {
                if let InputEvent::PressEnter { .. } = event {
                    this.apply(cx);
                }
            }),
            // Edits (here or on other pages, Discard, saving) show in the values table and the
            // search results.
            cx.observe(&session, |this, _, cx| {
                this.refresh_fields(cx);
                if this.is_searching(cx) {
                    this.run_search(cx);
                }
            }),
        ];

        let mut page = Self {
            session,
            filter,
            search,
            results,
            result_paths: Vec::new(),
            has_more_results: false,
            search_generation: 0,
            kinds,
            units,
            fields,
            editor,
            all_kinds,
            shown_kinds: Vec::new(),
            unit_ids: Vec::new(),
            field_rows: Vec::new(),
            unit_kind: String::new(),
            selected: None,
            problem: None,
            _subscriptions,
        };
        page.show_kinds(cx);
        page
    }

    fn is_searching(&self, cx: &gpui_kit::App) -> bool {
        self.search.read(cx).value().trim().chars().count() >= 2
    }

    /// Searches every unit id, field name and value for the search text, in the background (a
    /// big save holds a million values), and shows the first matches.
    fn run_search(&mut self, cx: &mut Context<Self>) {
        self.search_generation += 1;
        let generation = self.search_generation;
        if !self.is_searching(cx) {
            self.result_paths.clear();
            set_rows(&self.results, Vec::new(), cx);
            cx.notify();
            return;
        }
        let query = fold(self.search.read(cx).value().trim());
        let document = self.session.read(cx).preview();
        cx.spawn(async move |this, cx| {
            let (matches, has_more) = cx
                .background_spawn(async move { search(&document, &query) })
                .await;
            let _ = this.update(cx, |this, cx| {
                if generation != this.search_generation {
                    return;
                }
                let rows = matches
                    .iter()
                    .map(|found| TextRow {
                        cells: vec![
                            found.kind.clone().into(),
                            found.path.unit.clone().into(),
                            found.path.field_name().into(),
                            found.value.clone().into(),
                        ],
                        look: RowLook::Normal,
                    })
                    .collect();
                this.result_paths = matches.into_iter().map(|found| found.path).collect();
                this.has_more_results = has_more;
                set_rows(&this.results, rows, cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Chooses a value from the search results for the editor.
    fn select_path(&mut self, path: FieldPath, window: &mut Window, cx: &mut Context<Self>) {
        self.problem = None;
        self.unit_kind = self
            .session
            .read(cx)
            .preview()
            .unit(&path.unit)
            .map(|unit| unit.kind.clone())
            .unwrap_or_default();
        let text = self.session.read(cx).raw_value(&path).unwrap_or_default();
        self.selected = Some(path);
        self.editor
            .update(cx, |editor, cx| editor.set_value(text, window, cx));
        cx.notify();
    }

    fn show_kinds(&mut self, cx: &mut Context<Self>) {
        let query = self.filter.read(cx).value().to_lowercase();
        let shown: Vec<&(String, usize)> = self
            .all_kinds
            .iter()
            .filter(|(kind, _)| query.is_empty() || kind.contains(query.trim()))
            .collect();
        self.shown_kinds = shown.iter().map(|(kind, _)| kind.clone()).collect();
        let rows = shown
            .iter()
            .map(|(kind, count)| TextRow {
                cells: vec![kind.clone().into(), thousands(*count as i64).into()],
                look: RowLook::Normal,
            })
            .collect();
        set_rows(&self.kinds, rows, cx);
    }

    fn show_units(&mut self, kind: &str, cx: &mut Context<Self>) {
        self.unit_ids = self
            .session
            .read(cx)
            .preview()
            .units_of_kind(kind)
            .map(|unit| unit.id.clone())
            .collect();
        let rows = self
            .unit_ids
            .iter()
            .map(|id| TextRow {
                cells: vec![id.clone().into()],
                look: RowLook::Normal,
            })
            .collect();
        set_rows(&self.units, rows, cx);
        self.field_rows.clear();
        self.selected = None;
        set_rows(&self.fields, Vec::new(), cx);
        cx.notify();
    }

    fn show_fields(&mut self, id: &str, cx: &mut Context<Self>) {
        self.field_rows.clear();
        self.selected = None;
        self.problem = None;
        if let Some(unit) = self.session.read(cx).preview().unit(id) {
            self.unit_kind = unit.kind.clone();
            for field in &unit.fields {
                let path = |index| FieldPath {
                    unit: id.to_string(),
                    field: field.name.clone(),
                    index,
                };
                match &field.value {
                    sii::Value::Single(_) => self.field_rows.push(FieldRow::Value(path(None))),
                    sii::Value::Array(items) => {
                        self.field_rows.push(FieldRow::Array {
                            field: field.name.clone(),
                            count: items.len(),
                        });
                        for index in 0..items.len() {
                            self.field_rows.push(FieldRow::Value(path(Some(index))));
                        }
                    }
                }
            }
        }
        self.refresh_fields(cx);
    }

    /// Fills the values table from the session (showing edits).
    fn refresh_fields(&mut self, cx: &mut Context<Self>) {
        let session = self.session.read(cx);
        let rows = self
            .field_rows
            .iter()
            .map(|row| match row {
                FieldRow::Value(path) => TextRow {
                    cells: vec![
                        path.field_name().into(),
                        session.raw_value(path).unwrap_or_default().into(),
                    ],
                    look: if session.is_edited(path) {
                        RowLook::Emphasized
                    } else {
                        RowLook::Normal
                    },
                },
                FieldRow::Array { field, count } => TextRow {
                    cells: vec![
                        field.clone().into(),
                        t!(cx, "all-data-items", count = *count),
                    ],
                    look: RowLook::Muted,
                },
            })
            .collect();
        set_rows(&self.fields, rows, cx);
        cx.notify();
    }

    fn select_field(&mut self, row: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.problem = None;
        self.selected = match self.field_rows.get(row) {
            Some(FieldRow::Value(path)) => Some(path.clone()),
            _ => None,
        };
        let text = self
            .selected
            .as_ref()
            .and_then(|path| self.session.read(cx).raw_value(path))
            .unwrap_or_default();
        self.editor
            .update(cx, |editor, cx| editor.set_value(text, window, cx));
        cx.notify();
    }

    /// Applies the editor's text to the chosen value, if it has the same shape.
    fn apply(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.selected.clone() else {
            return;
        };
        let text = self.editor.read(cx).value().trim().to_string();
        let original = self.original_value(&path, cx);
        match check_shape(&original, &text) {
            Ok(()) => {
                self.problem = None;
                self.session
                    .update(cx, |session, cx| session.set_raw_value(path, text, cx));
            }
            Err(expected) => {
                self.problem = Some(t!(
                    cx,
                    "all-data-wrong-shape",
                    expected = shape_name(cx, expected)
                ));
                cx.notify();
            }
        }
    }

    /// Puts the chosen value back to what's in the save.
    fn revert(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(path) = self.selected.clone() else {
            return;
        };
        let original = self.original_value(&path, cx);
        self.problem = None;
        self.editor.update(cx, |editor, cx| {
            editor.set_value(original.clone(), window, cx)
        });
        self.session
            .update(cx, |session, cx| session.set_raw_value(path, original, cx));
    }

    /// The value as it is in the save on disk: what Revert goes back to, and the shape an edit
    /// must keep.
    fn original_value(&self, path: &FieldPath, cx: &Context<Self>) -> String {
        self.session.read(cx).saved_value(path).unwrap_or_default()
    }
}

/// How many search results show at most (the rest are counted as "more").
const MAX_RESULTS: usize = 1_000;

struct Found {
    kind: String,
    path: FieldPath,
    value: String,
}

/// Every value whose unit id, field name or text contains `query` (already folded), up to
/// `MAX_RESULTS`; and whether there were more.
fn search(document: &sii::Document, query: &str) -> (Vec<Found>, bool) {
    let mut found = Vec::new();
    for unit in document.units() {
        let unit_matches = fold(&unit.id).contains(query);
        for field in &unit.fields {
            let field_matches = unit_matches || fold(&field.name).contains(query);
            let values: Vec<(Option<usize>, &String)> = match &field.value {
                sii::Value::Single(text) => vec![(None, text)],
                sii::Value::Array(items) => items
                    .iter()
                    .enumerate()
                    .map(|(i, t)| (Some(i), t))
                    .collect(),
            };
            for (index, text) in values {
                if field_matches || fold(text).contains(query) {
                    if found.len() == MAX_RESULTS {
                        return (found, true);
                    }
                    found.push(Found {
                        kind: unit.kind.clone(),
                        path: FieldPath {
                            unit: unit.id.clone(),
                            field: field.name.clone(),
                            index,
                        },
                        value: text.clone(),
                    });
                }
            }
        }
    }
    (found, false)
}

/// A row of the values table: a value, or the heading of an array (its name and item count).
enum FieldRow {
    Value(FieldPath),
    Array { field: String, count: usize },
}

fn set_rows(table: &Entity<TableState<TextTable>>, rows: Vec<TextRow>, cx: &mut gpui_kit::App) {
    table.update(cx, |table, cx| {
        table.delegate_mut().rows = rows;
        table.refresh(cx);
        cx.notify();
    });
}

/// `Ok` when `text` holds the same shape of value as `original` (whole numbers and decimals
/// count as one shape: the game writes floats as whole numbers when they are).
fn check_shape(original: &str, text: &str) -> Result<(), Shape> {
    let expected = shape_of(original);
    let actual = shape_of(text);
    let numeric = |shape: Shape| matches!(shape, Shape::Integer | Shape::Float);
    let same =
        expected == Shape::Unknown || actual == expected || (numeric(expected) && numeric(actual));
    let quotes_closed = actual != Shape::Text || (text.len() >= 2 && text.ends_with('"'));
    // A vector keeps its punctuation: `(1, 2, 3)` stays three numbers, and a rotation keeps the
    // semicolon the game needs after its first number (`(1; 0, 0, 0)`).
    let same_punctuation = expected != Shape::Vector || punctuation(original) == punctuation(text);
    if same && quotes_closed && same_punctuation && !text.is_empty() {
        Ok(())
    } else {
        Err(expected)
    }
}

/// A vector's brackets, commas and semicolons, without the numbers or spaces.
fn punctuation(text: &str) -> String {
    text.chars()
        .filter(|c| matches!(c, '(' | ')' | ',' | ';'))
        .collect()
}

fn shape_name(cx: &gpui_kit::App, shape: Shape) -> SharedString {
    match shape {
        Shape::Bool => t!(cx, "shape-bool"),
        Shape::Integer | Shape::Float => t!(cx, "shape-number"),
        Shape::Text => t!(cx, "shape-text"),
        Shape::Id => t!(cx, "shape-id"),
        Shape::Vector => t!(cx, "shape-vector"),
        Shape::Unknown => t!(cx, "shape-value"),
    }
}

impl Render for AllDataPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_selection = self.selected.is_some();
        let is_searching = self.is_searching(cx);
        let pane = |title: SharedString, body: gpui_kit::AnyElement, width: Option<f32>| {
            v_flex()
                .gap_2()
                .h_full()
                .min_h_0()
                // The lists start at their width and give some up in a small window; Values
                // takes the rest. None gets narrower than its minimum.
                .when_some(width, |this, width| {
                    this.flex_basis(rems(width))
                        .flex_shrink(1.)
                        .min_w(rems(10.))
                })
                .when(width.is_none(), |this| this.flex_1().min_w(rems(12.)))
                .child(div().text_xs().font_medium().child(title))
                .child(div().flex_1().min_h_0().child(body))
        };

        v_flex()
            .size_full()
            .gap_4()
            .px_8()
            .py_6()
            .child(
                h_flex()
                    // The intro sits beside the search, or below it when the window is narrow.
                    .flex_wrap()
                    .gap_x_4()
                    .gap_y_2()
                    .items_center()
                    .child(
                        div().w(rems(22.)).flex_shrink_0().child(
                            Input::new(&self.search)
                                .small()
                                .cleanable(true)
                                .prefix(Icon::new(IconName::Search).small()),
                        ),
                    )
                    .child(
                        // Takes what's left of the row and wraps there, rather than running off
                        // the window (longer in some languages).
                        div()
                            .flex_1()
                            .min_w(rems(20.))
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(if is_searching {
                                let count = thousands(self.result_paths.len() as i64);
                                if self.has_more_results {
                                    t!(cx, "all-data-search-more", count = count)
                                } else {
                                    t!(
                                        cx,
                                        "all-data-search-count",
                                        count = self.result_paths.len(),
                                        amount = count
                                    )
                                }
                            } else {
                                t!(cx, "all-data-intro")
                            }),
                    ),
            )
            .when(is_searching, |this| {
                this.child(
                    div().flex_1().min_h_0().child(crate::ui::fill_table(
                        &self.results,
                        DataTable::new(&self.results)
                            .stripe(true)
                            .bordered(true)
                            .accessibility_label(t!(cx, "all-data-results"))
                            .into_any_element(),
                    )),
                )
            })
            .when(!is_searching, |this| {
                this.child(
                    h_flex()
                        .flex_1()
                        .min_h_0()
                        .gap_4()
                        .child(pane(
                            t!(cx, "all-data-kinds"),
                            v_flex()
                                .size_full()
                                .gap_2()
                                .child(Input::new(&self.filter).small())
                                .child(
                                    div().flex_1().min_h_0().child(
                                        DataTable::new(&self.kinds).stripe(true).bordered(true),
                                    ),
                                )
                                .into_any_element(),
                            Some(16.),
                        ))
                        .child(pane(
                            t!(cx, "all-data-units"),
                            DataTable::new(&self.units)
                                .stripe(true)
                                .bordered(true)
                                .into_any_element(),
                            Some(16.),
                        ))
                        .child(pane(
                            t!(cx, "all-data-values"),
                            crate::ui::fill_table(
                                &self.fields,
                                DataTable::new(&self.fields)
                                    .stripe(true)
                                    .bordered(true)
                                    .into_any_element(),
                            ),
                            None,
                        )),
                )
            })
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(div().text_sm().font_medium().min_w(rems(12.)).child(
                                match &self.selected {
                                    Some(path) => SharedString::from(format!(
                                        "{} › {}",
                                        self.unit_kind,
                                        path.field_name()
                                    )),
                                    None => t!(cx, "all-data-pick-value"),
                                },
                            ))
                            .child(
                                div()
                                    .flex_1()
                                    .child(Input::new(&self.editor).disabled(!has_selection)),
                            )
                            .child(
                                Button::new("revert-value")
                                    .ghost()
                                    .label(t!(cx, "all-data-revert"))
                                    .disabled(!has_selection)
                                    .on_click(
                                        cx.listener(|this, _, window, cx| this.revert(window, cx)),
                                    ),
                            )
                            .child(
                                Button::new("apply-value")
                                    .primary()
                                    .label(t!(cx, "all-data-apply"))
                                    .disabled(!has_selection)
                                    .on_click(cx.listener(|this, _, _, cx| this.apply(cx))),
                            ),
                    )
                    .when_some(self.problem.clone(), |this, problem| {
                        this.child(div().text_sm().text_color(cx.theme().danger).child(problem))
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_values_in_their_shape() {
        assert!(check_shape("5", "500").is_ok());
        assert!(check_shape("&3f800000", "2").is_ok());
        assert!(check_shape("5", "five").is_err());
        assert!(check_shape("\"Name\"", "\"Other\"").is_ok());
        assert!(check_shape("\"Name\"", "\"Unclosed").is_err());
        assert!(check_shape("\"Name\"", "Bare").is_err());
        assert!(check_shape("true", "false").is_ok());
        assert!(check_shape("true", "1").is_err());
        // Vectors keep their punctuation: the count of numbers, and a rotation's semicolon.
        assert!(check_shape("(1, 2, 3)", "(4, 5, 6)").is_ok());
        assert!(check_shape("(1, 2, 3)", "(4, 5)").is_err());
        assert!(check_shape("(0, 0, 0) (1; 0, 0, 0)", "(5, 0, 5) (1; 0, 0, 0)").is_ok());
        assert!(check_shape("(0, 0, 0) (1; 0, 0, 0)", "(5, 0, 5) (1, 0, 0, 0)").is_err());
        assert!(check_shape("(1, 2, 3)", "(4, 5, 6)").is_ok());
        assert!(check_shape("null", "_nameless.1.2").is_ok());
        assert!(check_shape("5", "").is_err());
    }
}
