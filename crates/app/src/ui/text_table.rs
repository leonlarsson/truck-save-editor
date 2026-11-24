//! A plain table of text cells, for lists that only show text (All data's panes). Virtualized by
//! GPUI Kit's table, so thousands of rows stay smooth. Rows can be marked to stand out (an edited
//! value) or to recede (an empty one). A table whose rows open a panel gets a Details column, a
//! quiet button at the end of each row, as the other tables have.

use std::rc::Rc;

use crate::ui::Tip as _;
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    ActiveTheme as _, Icon, Sizable as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
    table::{Column, TableDelegate, TableState},
};
use gpui_kit::{
    AnyElement, App, Context, IntoElement, ParentElement as _, Pixels, SharedString, Styled as _,
    Window, div,
};

/// The Details column's width.
const DETAILS_WIDTH: f32 = 44.;

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
        }
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

    /// Whether this column is the Details one (after the text columns).
    fn is_details(&self, col_ix: usize) -> bool {
        self.details.is_some() && col_ix == self.columns.len()
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

    fn cell(&self, row: usize, column: usize) -> SharedString {
        self.rows
            .get(row)
            .and_then(|row| row.cells.get(column))
            .cloned()
            .unwrap_or_default()
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
        self.columns.len() + usize::from(self.details.is_some())
    }

    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> Column {
        let mut widths: Vec<(f32, bool)> = self
            .columns
            .iter()
            .map(|column| (column.width, true))
            .collect();
        // The Details column keeps its width; the text columns grow.
        if self.details.is_some() {
            widths.push((DETAILS_WIDTH, false));
        }
        let widths = super::fill_widths(&widths, self.table_width);
        if self.is_details(col_ix) {
            return Column::new("details", "")
                .width(widths.get(col_ix).copied().unwrap_or(DETAILS_WIDTH));
        }
        match self.columns.get(col_ix) {
            Some(column) => Column::new(column.key, column.title.clone())
                .width(widths.get(col_ix).copied().unwrap_or(column.width)),
            None => Column::new("missing", ""),
        }
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        if self.is_details(col_ix) {
            return self.details_button(row_ix);
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
        assert!(!plain.is_details(2));

        let with_details = TextTable::new(columns()).with_details("Details".into(), |_, _, _| {});
        assert!(!with_details.is_details(1));
        assert!(with_details.is_details(2));
    }
}
