//! The cargo market: the job offers each company has, and how long they last.
//!
//! Companies are `company : company.volatile.<company>.<city>` units with `job_offer[]`, each a
//! `job_offer_data`: `target` (`"<company>.<city>"`), `cargo` (`cargo.<token>`, `null` for an
//! empty slot), `shortest_distance_km`, `urgency` (0 standard, 1 important, 2 urgent),
//! `ferry_time`, `units_count`, and `expiration_time` in game minutes, against the economy's
//! `game_time`.

use sii::{Document, Value, value};

/// One job offer.
#[derive(Clone, Debug, PartialEq)]
pub struct Job {
    /// The `job_offer_data` unit's id.
    pub id: String,
    pub from_company: String,
    pub from_city: String,
    pub to_company: String,
    pub to_city: String,
    /// The cargo's token (`battery`).
    pub cargo: String,
    pub distance_km: i64,
    /// 0 standard, 1 important, 2 urgent.
    pub urgency: u32,
    pub has_ferry: bool,
    /// How many units of cargo (pallets, cars …).
    pub units: i64,
    /// When the offer goes away, in game minutes.
    pub expires_at: i64,
}

/// The game's clock, in minutes since the career began.
pub fn game_time(document: &Document) -> Option<i64> {
    document
        .first_of_kind("economy")?
        .single("game_time")
        .and_then(value::parse_i64)
}

/// Every offer with cargo, company by company in the save's order.
pub fn jobs(document: &Document) -> Vec<Job> {
    let mut jobs = Vec::new();
    for company in document.units_of_kind("company") {
        let Some((from_company, from_city)) = company
            .id
            .strip_prefix("company.volatile.")
            .and_then(|rest| rest.split_once('.'))
        else {
            continue;
        };
        for offer in company
            .items("job_offer")
            .iter()
            .filter_map(|id| document.unit(id))
        {
            let Some(cargo) = offer
                .single("cargo")
                .and_then(|cargo| cargo.strip_prefix("cargo."))
            else {
                continue;
            };
            let target = offer
                .single("target")
                .map(value::parse_string)
                .unwrap_or_default();
            let Some((to_company, to_city)) = target.split_once('.') else {
                continue;
            };
            let number = |field: &str| offer.single(field).and_then(value::parse_i64).unwrap_or(0);
            jobs.push(Job {
                id: offer.id.clone(),
                from_company: from_company.to_string(),
                from_city: from_city.to_string(),
                to_company: to_company.to_string(),
                to_city: to_city.to_string(),
                cargo: cargo.to_string(),
                distance_km: number("shortest_distance_km"),
                urgency: u32::try_from(number("urgency")).unwrap_or(0),
                has_ferry: number("ferry_time") > 0,
                units: number("units_count"),
                expires_at: number("expiration_time"),
            });
        }
    }
    jobs
}

/// Makes every offer with cargo last until at least `until` (game minutes). Returns how many
/// were extended.
pub fn extend_offers(document: &mut Document, until: i64) -> usize {
    let ids: Vec<String> = jobs(document)
        .into_iter()
        .filter(|job| job.expires_at < until)
        .map(|job| job.id)
        .collect();
    for id in &ids {
        if let Some(mut offer) = document.unit_mut(id) {
            offer.set("expiration_time", Value::single(until.to_string()));
        }
    }
    ids.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n game_time: 1000\r\n}\r\n\r\ncompany : company.volatile.acme.lyon {\r\n job_offer: 2\r\n job_offer[0]: j1\r\n job_offer[1]: j2\r\n}\r\n\r\njob_offer_data : j1 {\r\n target: \"bolt.berlin\"\r\n expiration_time: 1100\r\n urgency: 2\r\n shortest_distance_km: 950\r\n ferry_time: 0\r\n cargo: cargo.bricks\r\n units_count: 12\r\n}\r\n\r\njob_offer_data : j2 {\r\n target: \"\"\r\n expiration_time: 4294967295\r\n cargo: null\r\n}\r\n\r\n}\r\n";

    #[test]
    fn lists_offers_with_cargo() {
        let document = sii::parse_text(SAVE).expect("parses");
        assert_eq!(game_time(&document), Some(1000));
        let jobs = jobs(&document);
        assert_eq!(jobs.len(), 1);
        let job = &jobs[0];
        assert_eq!(
            (job.from_company.as_str(), job.from_city.as_str()),
            ("acme", "lyon")
        );
        assert_eq!(
            (job.to_company.as_str(), job.to_city.as_str()),
            ("bolt", "berlin")
        );
        assert_eq!(job.cargo, "bricks");
        assert_eq!((job.distance_km, job.urgency, job.units), (950, 2, 12));
        assert!(!job.has_ferry);
    }

    #[test]
    fn extends_offers_that_would_expire_sooner() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        assert_eq!(extend_offers(&mut document, 5000), 1);
        assert_eq!(jobs(&document)[0].expires_at, 5000);
        assert_eq!(extend_offers(&mut document, 4000), 0);
    }
}
