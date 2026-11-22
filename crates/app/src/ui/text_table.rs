//! A plain table of text cells, for lists that only show text (All data's panes). Virtualized by
//! GPUI Kit's table, so thousands of rows stay smooth. Rows can be marked to stand out (an edited
//! value) or to recede (an empty one).

use gpui_kit::component::{
    ActiveTheme as _, StyledExt as _,
    table::{Column, TableDelegate, TableState},
};
use gpui_kit::{
    App, Context, IntoElement, ParentElement as _, Pixels, SharedString, Styled as _, Window, div,
};

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
        }
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
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> Column {
        let widths: Vec<(f32, bool)> = self
            .columns
            .iter()
            .map(|column| (column.width, true))
            .collect();
        let widths = super::fill_widths(&widths, self.table_width);
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
    }

    fn cell_label(&self, row_ix: usize, col_ix: usize, _: &App) -> Option<SharedString> {
        Some(self.cell(row_ix, col_ix))
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, _: &App) -> String {
        self.cell(row_ix, col_ix).to_string()
    }
}
