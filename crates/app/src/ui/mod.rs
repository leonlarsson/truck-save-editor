//! The app's UI kit: our own components, built on GPUI Kit's. Screens are built from these, so the
//! whole app shares one look. A component a screen is missing goes here, not into the screen.

mod amount_field;
mod change_list;
mod colour;
mod empty_state;
mod fill_table;
mod filter_bar;
mod game_icon;
mod level_picker;
mod map_card;
mod meter;
mod page;
mod path_text;
mod picks;
mod section;
mod stat;
mod step_buttons;
mod text_table;
mod tooltip;

pub use amount_field::{AmountChanged, AmountField};
pub use change_list::ChangeList;
pub use colour::{to_hsla, to_rgb8};
pub use empty_state::EmptyState;
pub use fill_table::{FillsWidth, fill_table, fill_widths};
pub use filter_bar::{FilterBar, FilterChanged, fold, table_or_empty};
pub use game_icon::GameIconView;
pub use level_picker::LevelPicker;
pub use map_card::MapCard;
pub use meter::Meter;
pub use page::{PageHeader, page_body};
pub use path_text::path_text;
pub use picks::{
    Choice, DetailChoice, PickBar, Picks, keep_clicks, owned_garages, pick_box, picks_done,
};
pub use section::Section;
pub use stat::Stat;
pub use step_buttons::StepButtons;
pub use text_table::{RowLook, TextColumn, TextRow, TextTable};
pub use tooltip::Tip;

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
