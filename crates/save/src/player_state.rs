//! The player's day: how tired they are, their record with the police, the cargo's damage on the
//! job in progress, the time of day, and pulling one of the company's trailers.
//!
//! - Tiredness: the player's `driving_time` and `time_since_mandatory_break` (minutes); a sleep in
//!   the game brings both back near zero, which is what resting does.
//! - The record: the economy's `police_offence_log` keeps every offence
//!   (`detailed_history_entries[]`, each with its fine) and totals per kind
//!   (`offence_total_counts[]`, `offence_total_fines[]`). Fines are paid when they're given; this
//!   clears the record, not the bank.
//! - Cargo damage: `cargo_damage` (0 to 1) on the job's trailer and the ones it pulls.
//! - The time: the economy's `game_time` in minutes; a day is 1,440 of them, the week starts on
//!   Monday. The clock only moves forward, as it does in the game.

use sii::{Document, Value, value};

/// Minutes in a day.
pub const DAY_MINUTES: i64 = 1440;

/// One change to the player's day.
#[derive(Clone, Debug, PartialEq)]
pub enum PlayerEdit {
    /// Rested, as after a sleep.
    Rest,
    /// No offences on record.
    ClearOffences,
    /// No damage to the cargo of the job in progress.
    RepairCargo,
    /// The clock forward to the next time it shows this hour (0 to 23), on the hour.
    SkipTo { hour: i64 },
    /// One of the company's trailers for the player to pull (not hitched).
    TakeTrailer { trailer: String },
}

impl PlayerEdit {
    /// Whether it changes the same thing as `other` (a later one replaces it).
    pub fn same_kind(&self, other: &PlayerEdit) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

fn economy(document: &Document) -> Option<&sii::Unit> {
    document.first_of_kind("economy")
}

fn player(document: &Document) -> Option<String> {
    economy(document)?.single("player").map(str::to_string)
}

fn number(document: &Document, unit: &str, field: &str) -> Option<i64> {
    document
        .unit(unit)?
        .single(field)
        .and_then(value::parse_i64)
}

fn set_if_present(document: &mut Document, unit: &str, field: &str, value: Value) {
    if let Some(mut unit) = document.unit_mut(unit)
        && unit.get(field).is_some()
    {
        unit.set(field, value);
    }
}

/// Minutes since the player's last break, if the save says.
pub fn minutes_since_break(document: &Document) -> Option<i64> {
    number(document, &player(document)?, "time_since_mandatory_break")
}

/// The offences on record and their fines together.
pub fn offences(document: &Document) -> (usize, i64) {
    let Some(log) = economy(document)
        .and_then(|economy| economy.single("police_offence_log"))
        .and_then(|id| document.unit(id))
    else {
        return (0, 0);
    };
    let fines = log
        .items("offence_total_fines")
        .iter()
        .filter_map(|fine| value::parse_i64(fine))
        .sum();
    let count = log
        .items("offence_total_counts")
        .iter()
        .filter_map(|count| value::parse_i64(count))
        .sum::<i64>();
    (usize::try_from(count).unwrap_or(0), fines)
}

/// The job in progress's trailers (the first and the ones it pulls).
fn job_trailers(document: &Document) -> Vec<String> {
    let trailer = player(document)
        .and_then(|id| document.unit(&id))
        .and_then(|player| player.single("current_job"))
        .and_then(|job| document.unit(job))
        .and_then(|job| job.single("company_trailer"))
        .filter(|trailer| *trailer != "null")
        .map(str::to_string);
    trailer
        .map(|trailer| crate::trailer_chain(document, &trailer))
        .unwrap_or_default()
}

/// The damage to the cargo of the job in progress (0 to 1, the worst of its trailers).
pub fn cargo_damage(document: &Document) -> Option<f32> {
    let trailers = job_trailers(document);
    if trailers.is_empty() {
        return None;
    }
    Some(
        trailers
            .iter()
            .filter_map(|id| document.unit(id)?.single("cargo_damage"))
            .filter_map(value::parse_f32)
            .fold(0.0, f32::max),
    )
}

/// The game's clock: minutes since the career began.
fn clock(document: &Document) -> Option<i64> {
    economy(document)?
        .single("game_time")
        .and_then(value::parse_i64)
}

/// The day of the week (0 is Monday) and the time of day in minutes.
pub fn time_of_day(document: &Document) -> Option<(i64, i64)> {
    let now = clock(document)?;
    Some(((now / DAY_MINUTES) % 7, now % DAY_MINUTES))
}

/// Whether the player is on a job (moving the clock would make it late).
pub fn is_on_job(document: &Document) -> bool {
    player(document)
        .and_then(|id| document.unit(&id))
        .and_then(|player| player.single("current_job"))
        .is_some_and(|job| job != "null")
}

/// Makes one change (see `PlayerEdit`). `false` if the save has nothing to change.
pub fn apply_player_edit(document: &mut Document, edit: &PlayerEdit) -> bool {
    match edit {
        PlayerEdit::Rest => {
            let Some(player) = player(document) else {
                return false;
            };
            for field in ["driving_time", "time_since_mandatory_break"] {
                set_if_present(document, &player, field, Value::single("0"));
            }
            true
        }
        PlayerEdit::ClearOffences => {
            let Some(log) = economy(document)
                .and_then(|economy| economy.single("police_offence_log"))
                .map(str::to_string)
            else {
                return false;
            };
            set_if_present(
                document,
                &log,
                "detailed_history_entries",
                Value::Array(Vec::new()),
            );
            for field in ["offence_total_counts", "offence_total_fines"] {
                let zeros = document
                    .unit(&log)
                    .map(|unit| vec!["0".to_string(); unit.items(field).len()])
                    .unwrap_or_default();
                set_if_present(document, &log, field, Value::Array(zeros));
            }
            // The entries now hang from nothing.
            crate::drop_unreachable(document);
            true
        }
        PlayerEdit::RepairCargo => {
            let trailers = job_trailers(document);
            for trailer in &trailers {
                set_if_present(document, trailer, "cargo_damage", Value::single("0"));
            }
            !trailers.is_empty()
        }
        PlayerEdit::SkipTo { hour } => {
            let (Some(now), Some(economy)) = (
                clock(document),
                economy(document).map(|unit| unit.id.clone()),
            ) else {
                return false;
            };
            let hour = (*hour).clamp(0, 23);
            let mut then = now - now % DAY_MINUTES + hour * 60;
            if then <= now {
                then += DAY_MINUTES;
            }
            set_if_present(
                document,
                &economy,
                "game_time",
                Value::single(then.to_string()),
            );
            true
        }
        PlayerEdit::TakeTrailer { trailer } => {
            crate::player_vehicle::take_trailer(document, trailer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n game_time: 16606526\r\n police_offence_log: log\r\n}\r\n\r\nplayer : p {\r\n driving_time: 300\r\n time_since_mandatory_break: 480\r\n current_job: job.1\r\n}\r\n\r\nplayer_job : job.1 {\r\n company_trailer: trailer.a\r\n}\r\n\r\ntrailer : trailer.a {\r\n cargo_damage: &3d4ccccd\r\n slave_trailer: null\r\n}\r\n\r\npolice_offence_log : log {\r\n detailed_history_entries: 2\r\n detailed_history_entries[0]: off.1\r\n detailed_history_entries[1]: off.2\r\n offence_total_counts: 2\r\n offence_total_counts[0]: 1\r\n offence_total_counts[1]: 1\r\n offence_total_fines: 2\r\n offence_total_fines[0]: 100\r\n offence_total_fines[1]: 250\r\n}\r\n\r\npolice_offence_log_entry : off.1 {\r\n fine: 100\r\n}\r\n\r\npolice_offence_log_entry : off.2 {\r\n fine: 250\r\n}\r\n\r\n}\r\n";

    fn document() -> Document {
        sii::parse_text(SAVE).expect("parses")
    }

    #[test]
    fn rests_and_clears_the_record() {
        let mut document = document();
        assert_eq!(minutes_since_break(&document), Some(480));
        assert!(apply_player_edit(&mut document, &PlayerEdit::Rest));
        assert_eq!(minutes_since_break(&document), Some(0));
        assert_eq!(offences(&document), (2, 350));
        assert!(apply_player_edit(&mut document, &PlayerEdit::ClearOffences));
        assert_eq!(offences(&document), (0, 0));
        assert!(document.unit("off.1").is_none(), "the entries are gone");
        let log = document.unit("log").expect("the log stays");
        assert_eq!(log.items("offence_total_counts"), ["0", "0"]);
    }

    #[test]
    fn repairs_the_cargo() {
        let mut document = document();
        assert!(cargo_damage(&document).is_some_and(|damage| damage > 0.04));
        assert!(apply_player_edit(&mut document, &PlayerEdit::RepairCargo));
        assert_eq!(cargo_damage(&document), Some(0.0));
    }

    #[test]
    fn moves_the_clock_forward_only() {
        let mut document = document();
        // 16,606,526 is a Thursday, 7:26.
        assert_eq!(time_of_day(&document), Some((3, 7 * 60 + 26)));
        assert!(apply_player_edit(
            &mut document,
            &PlayerEdit::SkipTo { hour: 8 }
        ));
        assert_eq!(time_of_day(&document), Some((3, 8 * 60)));
        // An hour already past today is tomorrow's.
        assert!(apply_player_edit(
            &mut document,
            &PlayerEdit::SkipTo { hour: 6 }
        ));
        assert_eq!(time_of_day(&document), Some((4, 6 * 60)));
    }
}
