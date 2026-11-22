//! The app's UI kit: our own components, built on GPUI Kit's. Screens are built from these, so the
//! whole app shares one look. A component a screen is missing goes here, not into the screen.

mod amount_field;
mod empty_state;
mod fill_table;
mod filter_bar;
mod game_icon;
mod level_picker;
mod page;
mod picks;
mod section;
mod stat;
mod step_buttons;
mod text_table;

pub use amount_field::{AmountChanged, AmountField};
pub use empty_state::EmptyState;
pub use fill_table::{FillsWidth, fill_table, fill_widths};
pub use filter_bar::{FilterBar, FilterChanged, fold, table_or_empty};
pub use game_icon::GameIconView;
pub use level_picker::LevelPicker;
pub use page::{PageHeader, page_body};
pub use picks::{Choice, PickBar, Picks, owned_garages, pick_box};
pub use section::Section;
pub use stat::Stat;
pub use step_buttons::StepButtons;
pub use text_table::{RowLook, TextColumn, TextRow, TextTable};

/// The order a click on a column heading sorts by: ascending first, then the other way.
pub fn next_sort(
    current: Option<gpui_kit::component::table::ColumnSort>,
) -> gpui_kit::component::table::ColumnSort {
    use gpui_kit::component::table::ColumnSort;
    match current {
        Some(ColumnSort::Ascending) => ColumnSort::Descending,
        _ => ColumnSort::Ascending,
    }
}

/// A table cell's text: one line, ending in "…" when it doesn't fit (the whole text stays in the
/// cell's label for screen readers and in the table's copy).
pub fn cell_text(text: impl Into<gpui_kit::SharedString>) -> gpui_kit::Div {
    use gpui_kit::{ParentElement as _, Styled as _};
    gpui_kit::div().min_w_0().truncate().child(text.into())
}
