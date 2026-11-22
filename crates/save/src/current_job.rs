//! The job the player is driving now: `player.current_job` points to a `player_job` unit with the
//! cargo, the source and target companies (`company.volatile.<company>.<city>`), the planned
//! distance, the urgency and the deadline (`time_upper_limit`, game minutes; `4294967295` for
//! none, as external contracts have).
//!
//! **Cancelling** handles the one shape real saves show: the player's own truck pulling a company
//! trailer (`company_truck: null`, `company_trailer` set), from the freight market or an external
//! contract. It clears `player.current_job` and unhitches the company trailer (from
//! `player.assigned_trailer` in older saves, from a `player_vehicles` unit's `trailer` in newer
//! ones), then removes the job and trailer units (and the trailer's parts): with nothing pointing
//! at them they'd be a second unit tree, which the game refuses ("There are multiple unit trees";
//! found in the in-game check). That's what the game does when a job ends.
//! Quick jobs (a company truck), jobs in the player's own trailer and special transports are
//! left alone: no save to learn them from yet.
//!
//! **Finishing** isn't offered: the pay isn't in the save (the game works it out on delivery,
//! along with the delivery log, XP and statistics). Removing the deadline is.

use sii::{Document, Value, value};

/// The deadline value meaning "none" (`u32::MAX`), as external contracts have.
const NO_DEADLINE: &str = "4294967295";

/// What kind of job it is, which decides whether it can be cancelled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JobKind {
    /// Own truck, company trailer, from the freight market.
    FreightMarket,
    /// Own truck, company trailer, from World of Trucks (an online job id).
    ExternalContract,
    /// A company truck too.
    QuickJob,
    /// The player's own trailer (the cargo market).
    OwnTrailer,
    /// A special transport (oversize cargo with an escort).
    SpecialTransport,
}

impl JobKind {
    /// Whether the editor knows how to cancel this kind of job.
    pub fn can_cancel(self) -> bool {
        matches!(self, JobKind::FreightMarket | JobKind::ExternalContract)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurrentJob {
    /// The `player_job` unit's id.
    pub id: String,
    pub kind: JobKind,
    /// The cargo's token (`apples`).
    pub cargo: String,
    pub from_company: String,
    pub from_city: String,
    pub to_company: String,
    pub to_city: String,
    pub planned_distance_km: i64,
    /// 0 standard, 1 important, 2 urgent.
    pub urgency: u32,
    /// When it must be delivered by, in game minutes; `None` without a deadline.
    pub deadline: Option<i64>,
    /// How many units of cargo.
    pub units: i64,
}

/// The player unit's id, through the economy.
fn player_id(document: &Document) -> Option<String> {
    document
        .first_of_kind("economy")?
        .single("player")
        .map(str::to_string)
}

/// `company.volatile.<company>.<city>` as (company, city).
fn company_and_city(id: &str) -> (String, String) {
    let rest = id.strip_prefix("company.volatile.").unwrap_or(id);
    match rest.split_once('.') {
        Some((company, city)) => (company.to_string(), city.to_string()),
        None => (rest.to_string(), String::new()),
    }
}

/// The job the player is driving, if any.
pub fn current_job(document: &Document) -> Option<CurrentJob> {
    let player = document.unit(&player_id(document)?)?;
    let job = document
        .unit(player.single("current_job")?)
        .filter(|unit| unit.kind == "player_job")?;
    let text = |field: &str| job.single(field).unwrap_or("null");
    let number = |field: &str| job.single(field).and_then(value::parse_i64).unwrap_or(0);
    let online_id = job.single("online_job_id").and_then(value::parse_u64);
    let kind = if text("special") != "null" {
        JobKind::SpecialTransport
    } else if text("company_truck") != "null" {
        JobKind::QuickJob
    } else if text("company_trailer") == "null" {
        JobKind::OwnTrailer
    } else if online_id.is_some_and(|id| id != u64::MAX) {
        JobKind::ExternalContract
    } else {
        JobKind::FreightMarket
    };
    let (from_company, from_city) = company_and_city(text("source_company"));
    let (to_company, to_city) = company_and_city(text("target_company"));
    let deadline = (text("time_upper_limit") != NO_DEADLINE)
        .then(|| number("time_upper_limit"))
        .filter(|deadline| *deadline > 0);
    Some(CurrentJob {
        id: job.id.clone(),
        kind,
        cargo: text("cargo")
            .strip_prefix("cargo.")
            .unwrap_or_default()
            .to_string(),
        from_company,
        from_city,
        to_company,
        to_city,
        planned_distance_km: number("planned_distance_km"),
        urgency: u32::try_from(number("urgency")).unwrap_or(0),
        deadline,
        units: number("units_count"),
    })
}

/// Sets a field only if the unit has it: the save's version decides which fields exist, and a
/// field it doesn't know would be an error for the game.
fn set_existing(document: &mut Document, unit: &str, field: &str, value: Value) {
    if let Some(mut unit) = document.unit_mut(unit)
        && unit.get(field).is_some()
    {
        unit.set(field, value);
    }
}

/// Cancels the current job (see the module docs for which kinds). `false` when there's no job
/// or it's a kind the editor doesn't cancel.
pub fn cancel_job(document: &mut Document) -> bool {
    let Some(job) = current_job(document) else {
        return false;
    };
    if !job.kind.can_cancel() {
        return false;
    }
    let Some(player) = player_id(document) else {
        return false;
    };
    let trailer = document
        .unit(&job.id)
        .and_then(|unit| unit.single("company_trailer"))
        .unwrap_or("null")
        .to_string();

    set_existing(document, &player, "current_job", Value::single("null"));
    // Older saves: the hitched trailer is the player's `assigned_trailer`.
    let is_assigned = document
        .unit(&player)
        .and_then(|unit| unit.single("assigned_trailer"))
        == Some(trailer.as_str());
    if is_assigned {
        set_existing(document, &player, "assigned_trailer", Value::single("null"));
        set_existing(
            document,
            &player,
            "assigned_trailer_connected",
            Value::single("false"),
        );
    }
    // Newer saves: a `player_vehicles` unit (one per kind of vehicle) holds it.
    let holders: Vec<String> = document
        .units_of_kind("player_vehicles")
        .filter(|unit| unit.single("trailer") == Some(trailer.as_str()))
        .map(|unit| unit.id.clone())
        .collect();
    for holder in &holders {
        let holder = holder.as_str();
        set_existing(document, holder, "trailer", Value::single("null"));
        set_existing(
            document,
            holder,
            "stored_trailer_attached",
            Value::single("false"),
        );
        set_existing(
            document,
            holder,
            "stored_trailer_placements",
            Value::Array(Vec::new()),
        );
    }
    // The job and its trailer (with its parts) now hang from nothing.
    crate::drop_unreachable(document);
    true
}

/// Removes the current job's deadline, as external contracts have none. `false` without a job.
pub fn remove_job_deadline(document: &mut Document) -> bool {
    let Some(job) = current_job(document) else {
        return false;
    };
    set_existing(
        document,
        &job.id,
        "time_upper_limit",
        Value::single(NO_DEADLINE),
    );
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A newer save: the hitched trailer is in `player_vehicles`.
    const NEWER: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n game_time: 1000\r\n}\r\n\r\nplayer : p {\r\n assigned_vehicles: pv.truck\r\n current_job: job.1\r\n assigned_trailer: null\r\n}\r\n\r\nplayer_vehicles : pv.truck {\r\n vehicle: truck.1\r\n trailer: trailer.c\r\n stored_trailer_placements: 1\r\n stored_trailer_placements[0]: (0, 0, 0) (1, 0, 0, 0)\r\n stored_trailer_attached: true\r\n}\r\n\r\nplayer_job : job.1 {\r\n company_truck: null\r\n company_trailer: trailer.c\r\n time_upper_limit: 1500\r\n online_job_id: 18446744073709551615\r\n cargo: cargo.apples\r\n source_company: company.volatile.fishco.bergen\r\n target_company: company.volatile.market.oslo\r\n planned_distance_km: 480\r\n urgency: 2\r\n special: null\r\n units_count: 30\r\n}\r\n\r\ntrailer : trailer.c {\r\n odometer: 5\r\n}\r\n\r\n}\r\n";

    /// An older save: the hitched trailer is the player's `assigned_trailer`.
    const OLDER: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n}\r\n\r\nplayer : p {\r\n current_job: job.1\r\n assigned_trailer: trailer.c\r\n assigned_trailer_connected: true\r\n}\r\n\r\nplayer_job : job.1 {\r\n company_truck: null\r\n company_trailer: trailer.c\r\n time_upper_limit: 4294967295\r\n online_job_id: 1234\r\n cargo: cargo.bricks\r\n source_company: company.volatile.a.lyon\r\n target_company: company.volatile.b.rome\r\n special: null\r\n}\r\n\r\n}\r\n";

    #[test]
    fn reads_the_current_job() {
        let document = sii::parse_text(NEWER).expect("parses");
        let job = current_job(&document).expect("a job");
        assert_eq!(job.kind, JobKind::FreightMarket);
        assert_eq!(job.cargo, "apples");
        assert_eq!(
            (job.from_company.as_str(), job.from_city.as_str()),
            ("fishco", "bergen")
        );
        assert_eq!(
            (job.to_company.as_str(), job.to_city.as_str()),
            ("market", "oslo")
        );
        assert_eq!(
            (job.planned_distance_km, job.urgency, job.units),
            (480, 2, 30)
        );
        assert_eq!(job.deadline, Some(1500));

        let older = sii::parse_text(OLDER).expect("parses");
        let job = current_job(&older).expect("a job");
        assert_eq!(job.kind, JobKind::ExternalContract);
        assert_eq!(job.deadline, None);
    }

    #[test]
    fn cancels_and_unhitches_the_company_trailer_in_a_newer_save() {
        let mut document = sii::parse_text(NEWER).expect("parses");
        assert!(cancel_job(&mut document));
        assert_eq!(current_job(&document), None);
        let holder = document.unit("pv.truck").expect("player_vehicles");
        assert_eq!(holder.single("trailer"), Some("null"));
        assert_eq!(holder.single("stored_trailer_attached"), Some("false"));
        assert!(holder.items("stored_trailer_placements").is_empty());
        // The truck stays; the job and the company's trailer are gone from the file.
        assert_eq!(holder.single("vehicle"), Some("truck.1"));
        assert!(document.unit("job.1").is_none() && document.unit("trailer.c").is_none());
        // No field the save didn't have was added.
        let player = document.unit("p").expect("player");
        assert!(player.get("assigned_trailer_connected").is_none());
        assert!(!cancel_job(&mut document), "nothing left to cancel");
    }

    #[test]
    fn cancels_and_unhitches_the_company_trailer_in_an_older_save() {
        let mut document = sii::parse_text(OLDER).expect("parses");
        assert!(cancel_job(&mut document));
        let player = document.unit("p").expect("player");
        assert_eq!(player.single("current_job"), Some("null"));
        assert_eq!(player.single("assigned_trailer"), Some("null"));
        assert_eq!(player.single("assigned_trailer_connected"), Some("false"));
    }

    #[test]
    fn leaves_quick_jobs_alone() {
        let text = NEWER.replace("company_truck: null", "company_truck: truck.company");
        let mut document = sii::parse_text(&text).expect("parses");
        assert_eq!(
            current_job(&document).map(|job| job.kind),
            Some(JobKind::QuickJob)
        );
        assert!(!cancel_job(&mut document));
        assert!(current_job(&document).is_some());
    }

    #[test]
    fn removes_the_deadline() {
        let mut document = sii::parse_text(NEWER).expect("parses");
        assert!(remove_job_deadline(&mut document));
        assert_eq!(current_job(&document).and_then(|job| job.deadline), None);
    }
}
