//! Tables that use the window's width. The kit's columns have fixed widths, so in a large window
//! a table would end in a strip of empty space. Instead each table is measured as it's drawn, and
//! the room beyond its columns' base widths goes to the columns that hold names (truck, city,
//! cargo), which are the ones that get cut off with "…".

use gpui_kit::component::table::{TableDelegate, TableState};
use gpui_kit::{
    AnyElement, App, Entity, IntoElement, ParentElement as _, Pixels, Styled as _, canvas, div, px,
};

/// A table delegate whose columns fill the table's width.
pub trait FillsWidth {
    /// The table's inner width, once it has been drawn.
    fn table_width(&self) -> Option<Pixels>;
    fn set_table_width(&mut self, width: Pixels);
}

/// Each column's width: its base width, plus a share of the table's spare room if it grows,
/// in proportion to its base width. `columns` is (base width, grows) per column.
///
/// Whole pixels, rounded down, so the columns never add up to more than the table and make it
/// scroll sideways.
pub fn fill_widths(columns: &[(f32, bool)], table_width: Option<Pixels>) -> Vec<f32> {
    let total: f32 = columns.iter().map(|(width, _)| width).sum();
    let growing: f32 = columns
        .iter()
        .filter(|(_, grows)| *grows)
        .map(|(width, _)| width)
        .sum();
    let spare = table_width.map(f32::from).unwrap_or(0.) - total;
    let mut widths: Vec<f32> = columns
        .iter()
        .map(|&(width, grows)| {
            if grows && spare > 0. && growing > 0. {
                (width + spare * width / growing).floor()
            } else {
                width
            }
        })
        .collect();
    // The pixels rounding left over go to the last growing column, so the columns end exactly at
    // the table's edge (a sliver left over showed as an extra, empty column).
    if spare > 0.
        && let Some(table_width) = table_width
        && let Some(last) = columns.iter().rposition(|(_, grows)| *grows)
    {
        let left_over = (f32::from(table_width) - widths.iter().sum::<f32>()).floor();
        if left_over > 0. {
            widths[last] += left_over;
        }
    }
    widths
}

/// The table, measured: when its width changes, its delegate hears the new width and its
/// columns are laid out again.
pub fn fill_table<D: TableDelegate + FillsWidth>(
    state: &Entity<TableState<D>>,
    table: AnyElement,
) -> AnyElement {
    let state = state.clone();
    // An invisible line across the top of the table, measured each time it's laid out. It's
    // absolutely placed, so it takes no room and no clicks.
    let probe = canvas(
        move |bounds, _, cx: &mut App| {
            // The table's border is one pixel on each side.
            let width = bounds.size.width - px(2.);
            if state.read(cx).delegate().table_width() != Some(width) {
                // Not now, in the middle of drawing: the change lays the table out again.
                let state = state.clone();
                cx.defer(move |cx| {
                    state.update(cx, |state, cx| {
                        state.delegate_mut().set_table_width(width);
                        state.refresh(cx);
                        cx.notify();
                    });
                });
            }
        },
        |_, _, _, _| {},
    )
    .absolute()
    .top_0()
    .left_0()
    .w_full()
    .h(px(0.));
    div()
        .relative()
        .size_full()
        .child(table)
        .child(probe)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growing_columns_share_the_spare_room_by_their_width() {
        let columns = [(200., true), (100., false), (100., true)];
        assert_eq!(
            fill_widths(&columns, Some(px(700.))),
            vec![400., 100., 200.]
        );
    }

    #[test]
    fn a_narrow_or_unmeasured_table_keeps_the_base_widths() {
        let columns = [(200., true), (100., false)];
        assert_eq!(fill_widths(&columns, Some(px(250.))), vec![200., 100.]);
        assert_eq!(fill_widths(&columns, None), vec![200., 100.]);
    }

    #[test]
    fn a_wider_table_is_filled_to_the_pixel() {
        let columns = [(130., true), (70., true), (45., false)];
        let widths = fill_widths(&columns, Some(px(1001.)));
        assert_eq!(widths.iter().sum::<f32>(), 1001.);
        assert_eq!(widths[2], 45.);
    }

    #[test]
    fn the_columns_never_add_up_to_more_than_the_table() {
        let columns = [(130., true), (70., true), (45., true)];
        let widths = fill_widths(&columns, Some(px(999.)));
        assert!(widths.iter().sum::<f32>() <= 999.);
    }
}
