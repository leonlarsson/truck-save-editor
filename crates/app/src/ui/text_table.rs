//! A plain table of text cells, for lists that only show text (All data's panes). Virtualized by
//! GPUI Kit's table, so thousands of rows stay smooth. Rows can be marked to stand out (an edited
//! value) or to recede (an empty one). A table whose rows open a panel gets a Details column, a
//! quiet button at the end of each row, as the other tables have. A table whose rows can be acted
//! on together gets a column of checkboxes at the start (`with_picks`; see `ui::picks`).

use std::rc::Rc;

use crate::ui::Tip as _;
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Icon, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    table::{Column, ColumnSort, TableDelegate, TableState},
};
use gpui_kit::{
    AnyElement, App, Context, InteractiveElement as _, IntoElement, ParentElement as _, Pixels,
    SharedString, Styled as _, Window, div,
};

/// The Details column's width.
const DETAILS_WIDTH: f32 = 44.;

/// The checkbox column's width, as on the tables with a delegate of their own (Fleet, Garages).
const PICK_WIDTH: f32 = 40.;

/// What a column of the table is, by its index: the checkboxes come first when rows can be
/// picked, then the text columns, then Details when rows open a panel.
#[derive(Clone, Copy, PartialEq)]
enum Slot {
    Pick,
    /// A text column, by its index among them.
    Text(usize),
    Details,
}

/// Opens a row's panel: the row's index, as the table shows it.
type OpenRow = Rc<dyn Fn(usize, &mut Window, &mut App)>;

pub struct TextColumn {
    pub key: &'static str,
    pub title: SharedString,
    pub width: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RowLook {
    #[default]
    Normal,
    /// Changed: stands out.
    Emphasized,
    /// Unimportant: recedes.
    Muted,
}

pub struct TextRow {
    pub cells: Vec<SharedString>,
    pub look: RowLook,
}

pub struct TextTable {
    pub columns: Vec<TextColumn>,
    pub rows: Vec<TextRow>,
    /// The table's measured width (see `ui::fill_table`): every column grows into a wider table,
    /// in proportion to its width.
    table_width: Option<Pixels>,
    /// The Details buttons' label, and what they do, when the rows open a panel.
    details: Option<(SharedString, OpenRow)>,
    /// Whether a click on a text column's heading sorts by it (see `with_sorting`).
    is_sortable: bool,
    /// The text column sorted by (its index among the text columns) and which way; `None` is
    /// the page's own order. The table only keeps this: the page, which knows what the cells
    /// mean (a number of visits isn't sorted as text), orders the rows it gives.
    pub sort: Option<(usize, ColumnSort)>,
    /// Whether rows can be picked (a checkbox column at the start).
    has_picks: bool,
    /// Each row's id for picking, alongside `rows`; `None` for a row that can't be picked (a
    /// photo trophy, which the editor only shows). Set together with `rows`.
    pub pick_ids: Vec<Option<String>>,
    /// The rows picked, by id: they stay picked through filtering and reloading.
    pub picks: super::Picks,
}

impl super::FillsWidth for TextTable {
    fn table_width(&self) -> Option<Pixels> {
        self.table_width
    }

    fn set_table_width(&mut self, width: Pixels) {
        self.table_width = Some(width);
    }
}

impl TextTable {
    pub fn new(columns: Vec<TextColumn>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            table_width: None,
            details: None,
            is_sortable: false,
            sort: None,
            has_picks: false,
            pick_ids: Vec::new(),
            picks: super::Picks::default(),
        }
    }

    /// A checkbox at the start of each row (and one in the heading for every row shown), to act
    /// on several rows together. The page gives each row its id in `pick_ids`, reads `picks`,
    /// and shows a `PickBar` while any are picked.
    pub fn with_picks(mut self) -> Self {
        self.has_picks = true;
        self
    }

    /// Headings that sort: ascending, then descending, then the page's own order again. The
    /// page calls `sort_by_header` when the table reports a heading clicked (`SelectColumn`),
    /// then orders its rows by `sort`.
    pub fn with_sorting(mut self) -> Self {
        self.is_sortable = true;
        self
    }

    /// A click on a column's heading: the next step of its sort. Columns that aren't text
    /// (checkboxes, Details) are left alone. Answers whether the sort changed.
    pub fn sort_by_header(&mut self, col_ix: usize) -> bool {
        let Slot::Text(column) = self.slot(col_ix) else {
            return false;
        };
        if !self.is_sortable {
            return false;
        }
        let current = self
            .sort
            .filter(|(sorted, _)| *sorted == column)
            .map(|(_, order)| order);
        let order = super::next_sort(current);
        self.sort = (order != ColumnSort::Default).then_some((column, order));
        true
    }

    /// What the column at this index is.
    fn slot(&self, col_ix: usize) -> Slot {
        let first_text = usize::from(self.has_picks);
        if self.has_picks && col_ix == 0 {
            Slot::Pick
        } else if self.details.is_some() && col_ix == first_text + self.columns.len() {
            Slot::Details
        } else {
            Slot::Text(col_ix.saturating_sub(first_text))
        }
    }

    /// The ids of the rows shown that can be picked.
    fn shown_ids(&self) -> Vec<String> {
        self.pick_ids.iter().flatten().cloned().collect()
    }

    /// A Details button at the end of each row, opening its panel (`label` names it for screen
    /// readers and its tooltip).
    pub fn with_details(
        mut self,
        label: SharedString,
        open: impl Fn(usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.details = Some((label, Rc::new(open)));
        self
    }

    fn details_button(&self, row_ix: usize) -> AnyElement {
        let Some((label, open)) = self.details.clone() else {
            return div().into_any_element();
        };
        h_flex()
            .w_full()
            .h_full()
            .items_center()
            .justify_end()
            .child(
                Button::new(("details", row_ix))
                    .xsmall()
                    .ghost()
                    // Quiet: every row has one, and rows also open by double-click and Enter.
                    .icon(Icon::new(IconName::ChevronRight))
                    .accessibility_label(label.clone())
                    .tip(label)
                    .on_click(move |_, window, cx| open(row_ix, window, cx)),
            )
            .into_any_element()
    }

    /// The text in a row's column, by the column's index in the table (checkboxes and Details
    /// have none).
    fn cell(&self, row: usize, col_ix: usize) -> SharedString {
        let Slot::Text(column) = self.slot(col_ix) else {
            return SharedString::default();
        };
        self.rows
            .get(row)
            .and_then(|row| row.cells.get(column))
            .cloned()
            .unwrap_or_default()
    }

    /// A row's checkbox, named by the row's first text (its city, its viewpoint).
    fn pick_cell(&self, row_ix: usize, cx: &mut Context<TableState<Self>>) -> AnyElement {
        let cell = h_flex().w_full().h_full().items_center().justify_center();
        let Some(Some(id)) = self.pick_ids.get(row_ix).cloned() else {
            // A row that can't be picked keeps the column's room, empty.
            return cell.into_any_element();
        };
        let name = self
            .rows
            .get(row_ix)
            .and_then(|row| row.cells.first())
            .cloned()
            .unwrap_or_default();
        let label = crate::i18n::t!(cx, "picks-row", name = name);
        cell.on_mouse_down(gpui_kit::MouseButton::Left, super::keep_clicks)
            .child(
                super::pick_box(("pick", row_ix), self.picks.contains(&id), label).on_click(
                    cx.listener(move |table, _: &bool, _, cx| {
                        table.delegate_mut().picks.toggle(&id);
                        cx.notify();
                    }),
                ),
            )
            .into_any_element()
    }
}

impl TableDelegate for TextTable {
    /// No empty column after the last one: the columns fill the table (`ui::fill_table`), and the
    /// kit's spare column showed as an odd extra heading.
    fn render_last_empty_col(
        &mut self,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        gpui_kit::div()
    }

    fn columns_count(&self, _: &App) -> usize {
        usize::from(self.has_picks) + self.columns.len() + usize::from(self.details.is_some())
    }

    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> Column {
        // The checkbox and Details columns keep their widths; the text columns grow.
        let mut widths: Vec<(f32, bool)> = Vec::new();
        if self.has_picks {
            widths.push((PICK_WIDTH, false));
        }
        widths.extend(self.columns.iter().map(|column| (column.width, true)));
        if self.details.is_some() {
            widths.push((DETAILS_WIDTH, false));
        }
        let widths = super::fill_widths(&widths, self.table_width);
        let width = |fallback: f32| widths.get(col_ix).copied().unwrap_or(fallback);
        match self.slot(col_ix) {
            Slot::Pick => Column::new("pick", "")
                .width(width(PICK_WIDTH))
                .resizable(false)
                .movable(false),
            Slot::Details => Column::new("details", "").width(width(DETAILS_WIDTH)),
            Slot::Text(index) => match self.columns.get(index) {
                Some(column) => {
                    let column =
                        Column::new(column.key, column.title.clone()).width(width(column.width));
                    // The sorted column shows its arrow; the others show they can be sorted.
                    match (self.is_sortable, self.sort) {
                        (false, _) => column,
                        (true, Some((sorted, order))) if sorted == index => column.sort(order),
                        (true, _) => column.sortable(),
                    }
                }
                None => Column::new("missing", ""),
            },
        }
    }

    /// A click on a heading's sort button is also a click on the heading, which the page sorts
    /// by (`sort_by_header`); sorting here too would make the two undo each other.
    fn perform_sort(
        &mut self,
        _: usize,
        _: ColumnSort,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) {
    }

    /// Headings as the kit draws them, but the checkbox column's: a checkbox for every row shown.
    fn render_th(
        &mut self,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        if self.slot(col_ix) != Slot::Pick {
            // One line, ending in "…" when a long heading doesn't fit.
            return div()
                .w_full()
                .truncate()
                .child(self.column(col_ix, cx).name)
                .into_any_element();
        }
        let shown = self.shown_ids();
        let is_all = self.picks.all_of(shown.iter().map(String::as_str));
        div()
            .w_full()
            .flex()
            .justify_center()
            .on_mouse_down(gpui_kit::MouseButton::Left, super::keep_clicks)
            .child(
                super::pick_box("pick-all", is_all, crate::i18n::t!(cx, "picks-all")).on_click(
                    cx.listener(move |table, _: &bool, _, cx| {
                        table
                            .delegate_mut()
                            .picks
                            .toggle_all(shown.iter().map(String::as_str));
                        cx.notify();
                    }),
                ),
            )
            .into_any_element()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        match self.slot(col_ix) {
            Slot::Details => return self.details_button(row_ix),
            Slot::Pick => return self.pick_cell(row_ix, cx),
            Slot::Text(_) => {}
        }
        let text = self.cell(row_ix, col_ix);
        let look = self
            .rows
            .get(row_ix)
            .map(|row| row.look)
            .unwrap_or_default();
        let cell = div().truncate().child(text);
        match look {
            RowLook::Normal => cell,
            RowLook::Emphasized => cell.font_semibold().text_color(cx.theme().primary_active),
            RowLook::Muted => cell.text_color(cx.theme().muted_foreground),
        }
        .into_any_element()
    }

    fn cell_label(&self, row_ix: usize, col_ix: usize, _: &App) -> Option<SharedString> {
        Some(self.cell(row_ix, col_ix))
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, _: &App) -> String {
        self.cell(row_ix, col_ix).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn columns() -> Vec<TextColumn> {
        ["city", "visits"]
            .into_iter()
            .map(|key| TextColumn {
                key,
                title: SharedString::from(key),
                width: 100.,
            })
            .collect()
    }

    #[test]
    fn a_details_column_comes_after_the_text_columns() {
        let plain = TextTable::new(columns());
        assert!(plain.slot(0) == Slot::Text(0));
        assert!(plain.slot(2) != Slot::Details);

        let with_details = TextTable::new(columns()).with_details("Details".into(), |_, _, _| {});
        assert!(with_details.slot(1) == Slot::Text(1));
        assert!(with_details.slot(2) == Slot::Details);
    }

    #[test]
    fn checkboxes_come_first_and_move_the_text_along() {
        let table = TextTable::new(columns())
            .with_picks()
            .with_details("Details".into(), |_, _, _| {});
        assert!(table.slot(0) == Slot::Pick);
        assert!(table.slot(1) == Slot::Text(0));
        assert!(table.slot(2) == Slot::Text(1));
        assert!(table.slot(3) == Slot::Details);
    }

    #[test]
    fn a_heading_sorts_up_then_down_then_not_at_all() {
        let mut table = TextTable::new(columns()).with_picks().with_sorting();
        // The checkbox column doesn't sort.
        assert!(!table.sort_by_header(0));
        assert!(table.sort_by_header(2));
        assert!(table.sort == Some((1, ColumnSort::Ascending)));
        assert!(table.sort_by_header(2));
        assert!(table.sort == Some((1, ColumnSort::Descending)));
        assert!(table.sort_by_header(2));
        assert!(table.sort.is_none());
        // Another column starts from ascending.
        table.sort_by_header(2);
        table.sort_by_header(1);
        assert!(table.sort == Some((0, ColumnSort::Ascending)));

        let mut plain = TextTable::new(columns());
        assert!(!plain.sort_by_header(0));
    }

    #[test]
    fn only_rows_with_an_id_can_be_picked() {
        let mut table = TextTable::new(columns()).with_picks();
        table.pick_ids = vec![Some("a".into()), None, Some("c".into())];
        assert_eq!(table.shown_ids(), ["a", "c"]);
    }
}
