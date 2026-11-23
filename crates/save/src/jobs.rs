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

/// A trailer an offer comes on, as an offer in the save has it: what a new job copies, so the
/// trailer, its load and the company's truck are ones the game itself chose for the cargo.
#[derive(Clone, Debug, PartialEq)]
pub struct OfferTrailer {
    /// `trailer.scs_curt_a`.
    pub variant: String,
    /// `trailer_def.scs.box.single_3.curtain`.
    pub definition: String,
    pub units: i64,
    /// As the save writes it.
    pub fill_ratio: String,
    /// The truck a quick job comes with (`"renault_magnum_4x2_a"`, as the save writes it).
    pub company_truck: String,
}

/// The trailers the save's offers carry `cargo` on, each once (by variant and definition).
pub fn offer_trailers(document: &Document, cargo: &str) -> Vec<OfferTrailer> {
    let wanted = format!("cargo.{cargo}");
    let mut found: Vec<OfferTrailer> = Vec::new();
    for offer in document.units_of_kind("job_offer_data") {
        if offer.single("cargo") != Some(wanted.as_str()) {
            continue;
        }
        let text = |field: &str| offer.single(field).unwrap_or_default().to_string();
        let trailer = OfferTrailer {
            variant: text("trailer_variant"),
            definition: text("trailer_definition"),
            units: offer
                .single("units_count")
                .and_then(value::parse_i64)
                .unwrap_or(1),
            fill_ratio: offer.single("fill_ratio").unwrap_or("1").to_string(),
            company_truck: offer.single("company_truck").unwrap_or("\"\"").to_string(),
        };
        let is_known = found.iter().any(|other| {
            other.variant == trailer.variant && other.definition == trailer.definition
        });
        if !trailer.variant.is_empty() && !trailer.definition.is_empty() && !is_known {
            found.push(trailer);
        }
    }
    found
}

/// How far a job goes, and its ferry if it takes one.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Route {
    pub distance_km: i64,
    pub ferry_minutes: i64,
    pub ferry_price: i64,
}

/// The route between two cities as an offer in the save has it, if one does.
pub fn known_route(document: &Document, from_city: &str, to_city: &str) -> Option<Route> {
    jobs(document)
        .into_iter()
        .find(|job| job.from_city == from_city && job.to_city == to_city)
        .and_then(|job| {
            let offer = document.unit(&job.id)?;
            let number = |field: &str| offer.single(field).and_then(value::parse_i64).unwrap_or(0);
            Some(Route {
                distance_km: job.distance_km,
                ferry_minutes: number("ferry_time"),
                ferry_price: number("ferry_price"),
            })
        })
}

/// A job to put on the market.
#[derive(Clone, Debug, PartialEq)]
pub struct NewJob {
    pub from_company: String,
    pub from_city: String,
    pub to_company: String,
    pub to_city: String,
    /// The cargo's token (`apples`).
    pub cargo: String,
    /// 0 standard, 1 important, 2 urgent.
    pub urgency: u32,
    pub trailer: OfferTrailer,
    pub route: Route,
    /// When it goes away, in game minutes.
    pub expires_at: i64,
}

/// Puts a job on the source company's market: in a free offer slot, or in place of the offer
/// that would go soonest. Returns the offer's id; `None` when the company isn't in the save or
/// has no offer slots.
pub fn add_job(document: &mut Document, job: &NewJob) -> Option<String> {
    let company = format!("company.volatile.{}.{}", job.from_company, job.from_city);
    let slots: Vec<String> = document.unit(&company)?.items("job_offer").to_vec();
    let slot = slots
        .iter()
        .find(|id| {
            document
                .unit(id)
                .is_some_and(|offer| offer.single("cargo") == Some("null"))
        })
        .or_else(|| {
            slots.iter().min_by_key(|id| {
                document
                    .unit(id)
                    .and_then(|offer| offer.single("expiration_time"))
                    .and_then(value::parse_i64)
                    .unwrap_or(i64::MAX)
            })
        })?
        .clone();
    let mut offer = document.unit_mut(&slot)?;
    let fields = [
        ("target", format!("\"{}.{}\"", job.to_company, job.to_city)),
        ("expiration_time", job.expires_at.max(0).to_string()),
        ("urgency", job.urgency.min(2).to_string()),
        (
            "shortest_distance_km",
            job.route.distance_km.max(1).to_string(),
        ),
        ("ferry_time", job.route.ferry_minutes.max(0).to_string()),
        ("ferry_price", job.route.ferry_price.max(0).to_string()),
        ("cargo", format!("cargo.{}", job.cargo)),
        ("company_truck", job.trailer.company_truck.clone()),
        ("trailer_variant", job.trailer.variant.clone()),
        ("trailer_definition", job.trailer.definition.clone()),
        ("units_count", job.trailer.units.max(1).to_string()),
        ("fill_ratio", job.trailer.fill_ratio.clone()),
        ("trailer_place", "0".to_string()),
    ];
    for (field, text) in fields {
        offer.set(field, Value::single(text));
    }
    Some(slot)
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

    fn new_job(trailer: OfferTrailer) -> NewJob {
        NewJob {
            from_company: "acme".into(),
            from_city: "lyon".into(),
            to_company: "zeta".into(),
            to_city: "paris".into(),
            cargo: "bricks".into(),
            urgency: 1,
            trailer,
            route: Route {
                distance_km: 460,
                ferry_minutes: 0,
                ferry_price: 0,
            },
            expires_at: 9000,
        }
    }

    #[test]
    fn makes_a_job_in_a_free_slot_from_a_known_trailer() {
        let mut document = sii::parse_text(&SAVE.replace(
            " units_count: 12\r\n",
            " units_count: 12\r\n trailer_variant: trailer.scs_curt_a\r\n trailer_definition: trailer_def.scs.box.single_3.curtain\r\n fill_ratio: 1\r\n company_truck: \"daf_xf\"\r\n",
        ))
        .expect("parses");
        let trailers = offer_trailers(&document, "bricks");
        assert_eq!(trailers.len(), 1);
        assert_eq!(trailers[0].variant, "trailer.scs_curt_a");
        assert_eq!(
            known_route(&document, "lyon", "berlin").map(|r| r.distance_km),
            Some(950)
        );
        assert_eq!(known_route(&document, "lyon", "paris"), None);

        let slot = add_job(&mut document, &new_job(trailers[0].clone())).expect("added");
        // The free slot, not the offer with cargo.
        assert_eq!(slot, "j2");
        let jobs = jobs(&document);
        assert_eq!(jobs.len(), 2);
        let made = jobs.iter().find(|job| job.id == "j2").expect("the new job");
        assert_eq!(
            (
                made.to_company.as_str(),
                made.to_city.as_str(),
                made.cargo.as_str()
            ),
            ("zeta", "paris", "bricks")
        );
        assert_eq!(
            (made.distance_km, made.urgency, made.units, made.expires_at),
            (460, 1, 12, 9000)
        );
        let offer = document.unit("j2").expect("offer");
        assert_eq!(
            offer.single("trailer_definition"),
            Some("trailer_def.scs.box.single_3.curtain")
        );
        assert_eq!(offer.single("company_truck"), Some("\"daf_xf\""));
    }

    #[test]
    fn without_a_free_slot_replaces_the_offer_that_goes_soonest() {
        let mut document =
            sii::parse_text(&SAVE.replace("cargo: null", "cargo: cargo.glass")).expect("parses");
        let trailer = OfferTrailer {
            variant: "trailer.a".into(),
            definition: "trailer_def.a".into(),
            units: 3,
            fill_ratio: "1".into(),
            company_truck: "\"\"".into(),
        };
        // j1 expires at 1100, j2 never: j1 goes.
        assert_eq!(
            add_job(&mut document, &new_job(trailer)).as_deref(),
            Some("j1")
        );
    }

    #[test]
    fn extends_offers_that_would_expire_sooner() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        assert_eq!(extend_offers(&mut document, 5000), 1);
        assert_eq!(jobs(&document)[0].expires_at, 5000);
        assert_eq!(extend_offers(&mut document, 4000), 0);
    }
}
