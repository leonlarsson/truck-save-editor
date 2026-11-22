//! A career in numbers, read-only: every delivery in the delivery log, what each truck and hired
//! driver has earned, and the economy's running totals.
//!
//! A delivery is a `delivery_log_entry` with `params[]`, by position (checked against the game's
//! own delivery log screen): 0 the game minute it ended, 1 and 2 the source and target companies
//! (`company.volatile.<company>.<city>`), 3 the cargo (`cargo.<token>`), 4 the XP, 5 the income,
//! 6 the distance in km, 7 the cargo damage (0 to 1), 15 the minute it started, 16 the truck
//! (`vehicle.<brand>.<model>`) and 18 the kind of job (`quick`, `compn` from the freight market,
//! `on_compn` an external contract, `freerm` free roaming without cargo). Entries without cargo
//! (free roaming) aren't deliveries.
//!
//! Earnings come from `profit_log` units (a driver's `profit_log`, a truck's from the player's
//! `truck_profit_logs[]`), whose `stats_data[]` entries each have `revenue`, `wage`,
//! `maintenance`, `fuel` and `distance`.

use sii::{Document, value};

/// One delivery.
#[derive(Clone, Debug, PartialEq)]
pub struct Delivery {
    /// The game day it ended (0 is the career's first).
    pub day: i64,
    pub from_company: String,
    pub from_city: String,
    pub to_company: String,
    pub to_city: String,
    /// The cargo's token (`apples`).
    pub cargo: String,
    pub xp: i64,
    pub income: i64,
    pub distance_km: i64,
    /// How long it took, in game minutes.
    pub minutes: i64,
    /// 0 to 1.
    pub cargo_damage: f32,
    /// The truck, as the log names it (`volvo.fh16`).
    pub truck: String,
    pub kind: DeliveryKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DeliveryKind {
    QuickJob,
    FreightMarket,
    ExternalContract,
    Other,
}

fn company_and_city(text: &str) -> (String, String) {
    let rest = text.strip_prefix("company.volatile.").unwrap_or(text);
    match rest.split_once('.') {
        Some((company, city)) => (company.to_string(), city.to_string()),
        None => (rest.to_string(), String::new()),
    }
}

/// Every delivery in the log, oldest first.
pub fn deliveries(document: &Document) -> Vec<Delivery> {
    let Some(log) = document
        .first_of_kind("economy")
        .and_then(|economy| economy.single("delivery_log"))
        .and_then(|id| document.unit(id))
    else {
        return Vec::new();
    };
    let mut deliveries: Vec<Delivery> = log
        .items("entries")
        .iter()
        .filter_map(|id| document.unit(id))
        .filter_map(|entry| {
            let params: Vec<String> = entry
                .items("params")
                .iter()
                .map(|param| value::parse_string(param).into_owned())
                .collect();
            let text = |index: usize| params.get(index).map(String::as_str).unwrap_or("");
            let number = |index: usize| value::parse_i64(text(index)).unwrap_or(0);
            let cargo = text(3).strip_prefix("cargo.")?.to_string();
            let (from_company, from_city) = company_and_city(text(1));
            let (to_company, to_city) = company_and_city(text(2));
            let ended = number(0);
            let started = number(15);
            Some(Delivery {
                day: ended / crate::DAY_MINUTES,
                from_company,
                from_city,
                to_company,
                to_city,
                cargo,
                xp: number(4),
                income: number(5),
                distance_km: number(6),
                minutes: if started > 0 && started <= ended {
                    ended - started
                } else {
                    0
                },
                cargo_damage: value::parse_f32(text(7)).unwrap_or(0.0),
                truck: text(16).trim_start_matches("vehicle.").to_string(),
                kind: match text(18) {
                    "quick" => DeliveryKind::QuickJob,
                    "compn" => DeliveryKind::FreightMarket,
                    "on_compn" => DeliveryKind::ExternalContract,
                    _ => DeliveryKind::Other,
                },
            })
        })
        .collect();
    deliveries.sort_by_key(|delivery| delivery.day);
    deliveries
}

/// What one truck or driver has earned and cost (its profit log).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Earnings {
    pub revenue: i64,
    pub wage: i64,
    pub maintenance: i64,
    pub fuel: i64,
    pub distance_km: i64,
}

impl Earnings {
    pub fn profit(&self) -> i64 {
        self.revenue - self.wage - self.maintenance - self.fuel
    }
}

/// The earnings in a `profit_log` unit.
pub fn earnings(document: &Document, profit_log: &str) -> Earnings {
    let mut total = Earnings::default();
    let Some(log) = document.unit(profit_log) else {
        return total;
    };
    for entry in log
        .items("stats_data")
        .iter()
        .filter_map(|id| document.unit(id))
    {
        let number = |field: &str| entry.single(field).and_then(value::parse_i64).unwrap_or(0);
        total.revenue += number("revenue");
        total.wage += number("wage");
        total.maintenance += number("maintenance");
        total.fuel += number("fuel");
        total.distance_km += number("distance");
    }
    total
}

/// Each hired driver's earnings, by driver id.
pub fn driver_earnings(document: &Document) -> Vec<(String, Earnings)> {
    crate::drivers(document)
        .into_iter()
        .filter_map(|driver| {
            let log = document.unit(&driver.id)?.single("profit_log")?.to_string();
            Some((driver.id, earnings(document, &log)))
        })
        .collect()
}

/// Each truck's earnings, by vehicle id (the player's `trucks[]` and `truck_profit_logs[]` run
/// alongside each other).
pub fn truck_earnings(document: &Document) -> Vec<(String, Earnings)> {
    let Some(player) = document
        .first_of_kind("economy")
        .and_then(|economy| economy.single("player"))
        .and_then(|id| document.unit(id))
    else {
        return Vec::new();
    };
    player
        .items("trucks")
        .iter()
        .zip(player.items("truck_profit_logs"))
        .map(|(truck, log)| (truck.clone(), earnings(document, log)))
        .collect()
}

/// The economy's running totals.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Totals {
    pub distance_km: i64,
    pub fuel_litres: i64,
    pub fuel_price: i64,
    pub cancelled_jobs: i64,
    pub game_days: i64,
}

pub fn totals(document: &Document) -> Totals {
    let Some(economy) = document.first_of_kind("economy") else {
        return Totals::default();
    };
    let number = |field: &str| {
        economy
            .single(field)
            .and_then(value::parse_i64)
            .unwrap_or(0)
    };
    Totals {
        // Newer saves keep `total_distance` at 0 and count by mode instead (driving, and the
        // ones after it); older ones only have the total.
        distance_km: number("total_distance").max(
            economy
                .items("total_distances_by_mode")
                .iter()
                .filter_map(|distance| value::parse_i64(distance))
                .sum(),
        ),
        fuel_litres: number("total_fuel_litres"),
        fuel_price: number("total_fuel_price"),
        cancelled_jobs: number("cancelled_job_count"),
        game_days: number("game_time") / crate::DAY_MINUTES,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n player: p\r\n delivery_log: log\r\n total_distance: 120000\r\n total_fuel_litres: 30000\r\n cancelled_job_count: 2\r\n game_time: 2880\r\n}\r\n\r\nplayer : p {\r\n trucks: 1\r\n trucks[0]: truck.1\r\n truck_profit_logs: 1\r\n truck_profit_logs[0]: plog.1\r\n drivers: 0\r\n}\r\n\r\ndelivery_log : log {\r\n entries: 2\r\n entries[0]: d.1\r\n entries[1]: d.2\r\n}\r\n\r\ndelivery_log_entry : d.1 {\r\n params: 19\r\n params[0]: \"2000\"\r\n params[1]: \"company.volatile.acme.lyon\"\r\n params[2]: \"company.volatile.bolt.rome\"\r\n params[3]: \"cargo.apples\"\r\n params[4]: \"500\"\r\n params[5]: \"9000\"\r\n params[6]: \"800\"\r\n params[7]: \"0.050\"\r\n params[8]: \"1\"\r\n params[9]: \"0\"\r\n params[10]: \"0\"\r\n params[11]: \"1\"\r\n params[12]: \"0\"\r\n params[13]: \"9000\"\r\n params[14]: \"0\"\r\n params[15]: \"1500\"\r\n params[16]: \"vehicle.volvo.fh16\"\r\n params[17]: \"790\"\r\n params[18]: \"compn\"\r\n}\r\n\r\ndelivery_log_entry : d.2 {\r\n params: 19\r\n params[0]: \"2500\"\r\n params[1]: \"\"\r\n params[2]: \"\"\r\n params[3]: \"\"\r\n params[4]: \"50\"\r\n params[5]: \"0\"\r\n params[6]: \"100\"\r\n params[7]: \"0.100\"\r\n params[8]: \"0\"\r\n params[9]: \"0\"\r\n params[10]: \"0\"\r\n params[11]: \"1\"\r\n params[12]: \"0\"\r\n params[13]: \"0\"\r\n params[14]: \"0\"\r\n params[15]: \"0\"\r\n params[16]: \"\"\r\n params[17]: \"0\"\r\n params[18]: \"freerm\"\r\n}\r\n\r\nprofit_log : plog.1 {\r\n stats_data: 1\r\n stats_data[0]: pe.1\r\n}\r\n\r\nprofit_log_entry : pe.1 {\r\n revenue: 9000\r\n wage: 0\r\n maintenance: 500\r\n fuel: 700\r\n distance: 800\r\n}\r\n\r\n}\r\n";

    #[test]
    fn reads_deliveries_but_not_free_roaming() {
        let document = sii::parse_text(SAVE).expect("parses");
        let deliveries = deliveries(&document);
        assert_eq!(deliveries.len(), 1);
        let delivery = &deliveries[0];
        assert_eq!(
            (delivery.cargo.as_str(), delivery.from_city.as_str()),
            ("apples", "lyon")
        );
        assert_eq!(
            (delivery.xp, delivery.income, delivery.distance_km),
            (500, 9000, 800)
        );
        assert_eq!((delivery.day, delivery.minutes), (1, 500));
        assert_eq!(delivery.kind, DeliveryKind::FreightMarket);
        assert_eq!(delivery.truck, "volvo.fh16");
    }

    #[test]
    fn reads_earnings_and_totals() {
        let document = sii::parse_text(SAVE).expect("parses");
        let trucks = truck_earnings(&document);
        assert_eq!(trucks.len(), 1);
        assert_eq!(trucks[0].1.profit(), 7800);
        let totals = totals(&document);
        assert_eq!(
            (totals.distance_km, totals.cancelled_jobs, totals.game_days),
            (120000, 2, 2)
        );
    }
}
