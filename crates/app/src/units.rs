//! The units a game shows distances and weights in. Saves store kilometres and kilograms in both
//! games; American Truck Simulator shows miles and pounds. Fields that edit a distance or a weight
//! show the game's unit and convert back when the value is stored.

use gpui_kit::{App, SharedString};
use save::Game;

use crate::i18n::t;

/// Miles in a kilometre, and pounds in a kilogram.
const MILES_PER_KM: f64 = 0.621_371;
const POUNDS_PER_KG: f64 = 2.204_623;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Kilometre,
    Mile,
    Kilogram,
    Pound,
}

impl Unit {
    /// The unit a game shows distances in.
    pub fn distance(game: Game) -> Unit {
        if game.uses_miles() {
            Unit::Mile
        } else {
            Unit::Kilometre
        }
    }

    /// The unit a game's fields edit weights in.
    pub fn weight(game: Game) -> Unit {
        if game.uses_miles() {
            Unit::Pound
        } else {
            Unit::Kilogram
        }
    }

    /// A stored value (kilometres, kilograms) in this unit, rounded.
    pub fn shown(self, stored: f64) -> i64 {
        (stored * self.per_stored()).round() as i64
    }

    /// A value typed in this unit, as stored (kilometres, kilograms).
    pub fn stored(self, shown: i64) -> f64 {
        shown as f64 / self.per_stored()
    }

    /// How many of this unit make one stored unit.
    fn per_stored(self) -> f64 {
        match self {
            Unit::Kilometre | Unit::Kilogram => 1.0,
            Unit::Mile => MILES_PER_KM,
            Unit::Pound => POUNDS_PER_KG,
        }
    }

    /// The unit's symbol, as fields show it after a number.
    pub fn symbol(self, cx: &App) -> SharedString {
        match self {
            Unit::Kilometre => t!(cx, "unit-km"),
            Unit::Mile => t!(cx, "unit-mi"),
            Unit::Kilogram => t!(cx, "unit-kg"),
            Unit::Pound => t!(cx, "unit-lb"),
        }
    }
}

/// A distance as the game shows it (`1,245 km`, `774 mi`), from kilometres.
pub fn distance(cx: &App, game: Game, km: i64) -> SharedString {
    let unit = Unit::distance(game);
    t!(
        cx,
        "amount-with-unit",
        amount = crate::format::thousands(unit.shown(km as f64)),
        unit = unit.symbol(cx)
    )
}

/// A weight as the game shows it: tonnes with one decimal in ETS2 (`12.5 t`), whole pounds in ATS
/// (`27,558 lb`), from kilograms.
pub fn weight(cx: &App, game: Game, kg: f64) -> SharedString {
    match Unit::weight(game) {
        Unit::Pound => t!(
            cx,
            "amount-with-unit",
            amount = crate::format::thousands(Unit::Pound.shown(kg)),
            unit = Unit::Pound.symbol(cx)
        ),
        _ => t!(
            cx,
            "amount-with-unit",
            amount = crate::format::decimal(kg / 1000.0, 1),
            unit = t!(cx, "unit-t")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_both_ways() {
        assert_eq!(Unit::Kilometre.shown(1000.0), 1000);
        assert_eq!(Unit::Mile.shown(1000.0), 621);
        assert_eq!(Unit::Pound.shown(1000.0), 2205);
        // What's typed comes back close enough to round to the same shown value.
        assert_eq!(Unit::Mile.shown(Unit::Mile.stored(621)), 621);
        assert_eq!(Unit::Pound.shown(Unit::Pound.stored(2205)), 2205);
        assert_eq!(Unit::distance(Game::Ats), Unit::Mile);
        assert_eq!(Unit::weight(Game::Ets2), Unit::Kilogram);
    }
}
