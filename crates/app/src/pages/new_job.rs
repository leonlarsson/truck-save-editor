//! Make a job: a freight market offer put on the market when saving, from a company that ships the
//! cargo, to one that takes it, on a trailer the game already carries that cargo on (copied from
//! an offer in the save, with its load and company truck), standard, important or urgent. The
//! route is the one an offer between the same cities has; otherwise it's estimated from the map,
//! scaled by how the save's own offers compare with the map.

use std::collections::{BTreeSet, HashMap};

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Selectable as _, Sizable as _, WindowExt as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
    h_flex,
    notification::{Notification, NotificationType},
    select::{SearchableVec, Select, SelectEvent, SelectState},
    v_flex,
};
use gpui_kit::{
    App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, SharedString,
    Styled as _, Subscription, Window, div, px,
};

use crate::game_data;
use crate::i18n::t;
use crate::session::{KEEP_OFFERS_MINUTES, Session};
use crate::ui::Choice;

pub struct NewJobForm {
    session: Entity<Session>,
    source: Entity<SelectState<SearchableVec<Choice>>>,
    cargo: Entity<SelectState<SearchableVec<Choice>>>,
    target: Entity<SelectState<SearchableVec<Choice>>>,
    trailer: Entity<SelectState<SearchableVec<Choice>>>,
    /// 0 standard, 1 important, 2 urgent.
    urgency: u32,
    /// The trailers offered for the chosen cargo, by their place in the list.
    trailers: Vec<save::OfferTrailer>,
    is_filled: bool,
    _subscriptions: Vec<Subscription>,
}

/// `company.volatile.<company>.<city>` → (`company`, `city`).
fn company_and_city(id: &str) -> Option<(&str, &str)> {
    id.strip_prefix("company.volatile.")?.split_once('.')
}

/// Every company in the save, as (company, city).
fn companies(document: &sii::Document) -> Vec<(String, String)> {
    document
        .units_of_kind("company")
        .filter_map(|unit| company_and_city(&unit.id))
        .map(|(company, city)| (company.to_string(), city.to_string()))
        .collect()
}

/// Where each city is on the map: the middle of its biggest area.
fn city_positions(places: &[gamedata::MapPlace]) -> HashMap<String, (f32, f32, f32)> {
    let mut positions: HashMap<String, (f32, f32, f32)> = HashMap::new();
    for place in places {
        if place.kind != gamedata::MapPlaceKind::CityArea {
            continue;
        }
        let size = place.width * place.depth;
        let entry = positions
            .entry(place.city.clone())
            .or_insert((place.x, place.z, 0.0));
        if size > entry.2 {
            *entry = (place.x, place.z, size);
        }
    }
    positions
}

/// A route between two cities no offer in the save has: the straight line on the map, scaled by
/// how the save's own offers' distances compare with their straight lines (the middle one of
/// those ratios). `None` without positions or offers to compare with.
fn estimated_route(
    document: &sii::Document,
    places: &[gamedata::MapPlace],
    from: &str,
    to: &str,
) -> Option<save::Route> {
    let positions = city_positions(places);
    let line = |a: &str, b: &str| -> Option<f32> {
        let (ax, az, _) = positions.get(a)?;
        let (bx, bz, _) = positions.get(b)?;
        let metres = ((ax - bx).powi(2) + (az - bz).powi(2)).sqrt();
        (metres > 1.0).then_some(metres)
    };
    let mut ratios: Vec<f32> = save::jobs(document)
        .into_iter()
        .filter(|job| job.distance_km > 0)
        .filter_map(|job| Some(job.distance_km as f32 / line(&job.from_city, &job.to_city)?))
        .collect();
    if ratios.is_empty() {
        return None;
    }
    ratios.sort_by(f32::total_cmp);
    let ratio = ratios[ratios.len() / 2];
    Some(save::Route {
        distance_km: (line(from, to)? * ratio).round() as i64,
        ferry_minutes: 0,
        ferry_price: 0,
    })
}

/// A trailer as the list shows it: its body and chassis from its definition
/// (`trailer_def.scs.box.single_3.curtain` → `Curtain, single 3`), and the load.
fn trailer_title(trailer: &save::OfferTrailer, cx: &App) -> SharedString {
    let parts: Vec<&str> = trailer.definition.split('.').collect();
    let words = |part: &str| part.replace('_', " ");
    let body = parts.last().map(|part| words(part)).unwrap_or_default();
    let chassis = parts
        .len()
        .checked_sub(2)
        .and_then(|at| parts.get(at))
        .map(|part| words(part))
        .unwrap_or_default();
    let body = capitalised(&body);
    t!(
        cx,
        "new-job-trailer",
        body = body,
        chassis = chassis,
        count = trailer.units
    )
}

/// `curtain` → `Curtain`.
fn capitalised(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn company_choice(company: &str, city: &str, cx: &App) -> Choice {
    Choice {
        value: format!("{company}.{city}").into(),
        name: t!(
            cx,
            "new-job-company",
            company = game_data::company_name(cx, company),
            city = game_data::city_name(cx, city)
        ),
    }
}

impl NewJobForm {
    fn new(session: Entity<Session>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let select = |window: &mut Window, cx: &mut Context<Self>| {
            cx.new(|cx| {
                SelectState::new(SearchableVec::new(Vec::new()), None, window, cx).searchable(true)
            })
        };
        let (source, cargo, target, trailer) = (
            select(window, cx),
            select(window, cx),
            select(window, cx),
            select(window, cx),
        );
        let _subscriptions = vec![
            cx.subscribe_in(
                &source,
                window,
                // Picking (the only event) means the lists after it start over.
                |this, _, _: &SelectEvent<SearchableVec<Choice>>, window, cx| {
                    this.fill_cargo(window, cx)
                },
            ),
            cx.subscribe_in(
                &cargo,
                window,
                |this, _, _: &SelectEvent<SearchableVec<Choice>>, window, cx| {
                    this.fill_target_and_trailer(window, cx)
                },
            ),
            cx.subscribe(
                &target,
                |_, _, _: &SelectEvent<SearchableVec<Choice>>, cx| cx.notify(),
            ),
            cx.subscribe(
                &trailer,
                |_, _, _: &SelectEvent<SearchableVec<Choice>>, cx| cx.notify(),
            ),
        ];
        Self {
            session,
            source,
            cargo,
            target,
            trailer,
            urgency: 0,
            trailers: Vec::new(),
            is_filled: false,
            _subscriptions,
        }
    }

    fn selected(select: &Entity<SelectState<SearchableVec<Choice>>>, cx: &App) -> Option<String> {
        select
            .read(cx)
            .selected_value()
            .map(|value| value.to_string())
    }

    /// The companies that ship something the save has a trailer for, and someone takes.
    fn fill_sources(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_filled {
            return;
        }
        let Some(cargo) = game_data::company_cargo(cx) else {
            return;
        };
        let document = self.session.read(cx).preview();
        let all = companies(&document);
        let taken: BTreeSet<&String> = all
            .iter()
            .filter_map(|(company, _)| cargo.takes.get(company))
            .flatten()
            .collect();
        let carried: BTreeSet<String> = save::jobs(&document)
            .into_iter()
            .map(|job| job.cargo)
            .collect();
        let mut choices: Vec<Choice> = all
            .iter()
            .filter(|(company, _)| {
                cargo.ships.get(company).is_some_and(|ships| {
                    ships
                        .iter()
                        .any(|token| carried.contains(token) && taken.contains(token))
                })
            })
            .map(|(company, city)| company_choice(company, city, cx))
            .collect();
        choices.sort_by(|a, b| a.name.cmp(&b.name));
        self.source.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(choices), window, cx)
        });
        self.is_filled = true;
    }

    /// The cargo the chosen company ships, that the save has a trailer for and someone takes.
    fn fill_cargo(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(source), Some(company_cargo)) = (
            Self::selected(&self.source, cx),
            game_data::company_cargo(cx),
        ) else {
            return;
        };
        let Some((company, _)) = source.split_once('.') else {
            return;
        };
        let document = self.session.read(cx).preview();
        let all = companies(&document);
        let mut choices: Vec<Choice> = company_cargo
            .ships
            .get(company)
            .into_iter()
            .flatten()
            .filter(|token| !save::offer_trailers(&document, token).is_empty())
            .filter(|token| {
                all.iter()
                    .any(|(other, _)| company_cargo.takes(other, token) && other != company)
            })
            .map(|token| Choice {
                value: token.clone().into(),
                name: game_data::cargo_name(cx, token).into(),
            })
            .collect();
        // Some cargo comes twice under one name (loose and in containers): those say what they
        // travel on.
        let names: Vec<SharedString> = choices.iter().map(|choice| choice.name.clone()).collect();
        for choice in &mut choices {
            let is_shared = names.iter().filter(|name| **name == choice.name).count() > 1;
            let body = save::offer_trailers(&document, &choice.value)
                .first()
                .and_then(|trailer| trailer.definition.rsplit('.').next().map(str::to_string));
            if let (true, Some(body)) = (is_shared, body) {
                choice.name = t!(
                    cx,
                    "new-job-cargo-on",
                    cargo = choice.name.clone(),
                    body = capitalised(&body.replace('_', " "))
                );
            }
        }
        choices.sort_by(|a, b| a.name.cmp(&b.name));
        self.cargo.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(choices), window, cx);
            select.set_selected_index(None, window, cx);
        });
        self.clear(&self.target.clone(), window, cx);
        self.clear(&self.trailer.clone(), window, cx);
        self.trailers.clear();
        cx.notify();
    }

    fn clear(
        &self,
        select: &Entity<SelectState<SearchableVec<Choice>>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        select.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(Vec::new()), window, cx);
            select.set_selected_index(None, window, cx);
        });
    }

    /// Who takes the chosen cargo, and the trailers the game carries it on.
    fn fill_target_and_trailer(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(source), Some(token), Some(company_cargo)) = (
            Self::selected(&self.source, cx),
            Self::selected(&self.cargo, cx),
            game_data::company_cargo(cx),
        ) else {
            return;
        };
        let document = self.session.read(cx).preview();
        let mut targets: Vec<Choice> = companies(&document)
            .iter()
            .filter(|(company, city)| {
                company_cargo.takes(company, &token) && format!("{company}.{city}") != source
            })
            .map(|(company, city)| company_choice(company, city, cx))
            .collect();
        targets.sort_by(|a, b| a.name.cmp(&b.name));
        self.trailers = save::offer_trailers(&document, &token);
        let trailers: Vec<Choice> = self
            .trailers
            .iter()
            .enumerate()
            .map(|(index, trailer)| Choice {
                value: index.to_string().into(),
                name: trailer_title(trailer, cx),
            })
            .collect();
        let only_trailer = (trailers.len() == 1).then_some(0);
        self.target.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(targets), window, cx);
            select.set_selected_index(None, window, cx);
        });
        self.trailer.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(trailers), window, cx);
            // One trailer: nothing to choose.
            select.set_selected_index(
                only_trailer.map(gpui_kit::component::IndexPath::new),
                window,
                cx,
            );
        });
        cx.notify();
    }

    /// The job as chosen, ready to add, or why it isn't.
    fn job(&self, cx: &mut Context<Self>) -> Result<save::NewJob, SharedString> {
        let missing = || t!(cx, "new-job-incomplete");
        let source = Self::selected(&self.source, cx).ok_or_else(missing)?;
        let cargo = Self::selected(&self.cargo, cx).ok_or_else(missing)?;
        let target = Self::selected(&self.target, cx).ok_or_else(missing)?;
        let trailer = Self::selected(&self.trailer, cx)
            .and_then(|index| index.parse::<usize>().ok())
            .and_then(|index| self.trailers.get(index).cloned())
            .ok_or_else(missing)?;
        let (Some((from_company, from_city)), Some((to_company, to_city))) =
            (source.split_once('.'), target.split_once('.'))
        else {
            return Err(missing());
        };
        let document = self.session.read(cx).preview();
        let route = match save::known_route(&document, from_city, to_city) {
            Some(route) => route,
            None => game_data::map_places(cx)
                .and_then(|places| estimated_route(&document, &places, from_city, to_city))
                .ok_or_else(|| t!(cx, "new-job-no-route"))?,
        };
        let now = save::game_time(&document).unwrap_or(0);
        Ok(save::NewJob {
            from_company: from_company.to_string(),
            from_city: from_city.to_string(),
            to_company: to_company.to_string(),
            to_city: to_city.to_string(),
            cargo,
            urgency: self.urgency,
            trailer,
            route,
            expires_at: now + KEEP_OFFERS_MINUTES,
        })
    }

    fn add(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Ok(job) = self.job(cx) else {
            return;
        };
        self.session
            .update(cx, |session, cx| session.add_job(job, cx));
        window.close_sheet(cx);
        window.push_notification(
            Notification::new()
                .with_type(NotificationType::Success)
                .title(t!(cx, "new-job-added")),
            cx,
        );
    }
}

impl Render for NewJobForm {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.fill_sources(window, cx);
        let job = self.job(cx);
        let why_not = job.as_ref().err().cloned();
        let field = |label: SharedString,
                     select: &Entity<SelectState<SearchableVec<Choice>>>,
                     placeholder: SharedString| {
            v_flex()
                .gap_1p5()
                .child(div().text_sm().child(label))
                .child(Select::new(select).small().placeholder(placeholder))
        };
        let loading = !self.is_filled;
        let urgency = self.urgency;
        v_flex()
            .pt_2()
            .gap_4()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(t!(cx, "new-job-description")),
            )
            .child(field(
                t!(cx, "new-job-from"),
                &self.source,
                if loading {
                    t!(cx, "truck-part-loading")
                } else {
                    t!(cx, "new-job-pick")
                },
            ))
            .child(field(
                t!(cx, "new-job-cargo"),
                &self.cargo,
                t!(cx, "new-job-pick"),
            ))
            .child(field(
                t!(cx, "new-job-to"),
                &self.target,
                t!(cx, "new-job-pick"),
            ))
            .child(field(
                t!(cx, "new-job-trailer-label"),
                &self.trailer,
                t!(cx, "new-job-pick"),
            ))
            .child(
                v_flex()
                    .gap_1p5()
                    .child(div().text_sm().child(t!(cx, "column-urgency")))
                    .child(
                        h_flex().child(
                            ButtonGroup::new("new-job-urgency")
                                .outline()
                                .small()
                                .child(
                                    Button::new("standard")
                                        .label(t!(cx, "job-urgency-standard"))
                                        .selected(urgency == 0),
                                )
                                .child(
                                    Button::new("important")
                                        .label(t!(cx, "job-urgency-important"))
                                        .selected(urgency == 1),
                                )
                                .child(
                                    Button::new("urgent")
                                        .label(t!(cx, "job-urgency-urgent"))
                                        .selected(urgency == 2),
                                )
                                .on_click(cx.listener(|this, clicked: &Vec<usize>, _, cx| {
                                    if let Some(&index) = clicked.first() {
                                        this.urgency = index as u32;
                                        cx.notify();
                                    }
                                })),
                        ),
                    ),
            )
            .child(
                v_flex()
                    .gap_2()
                    .items_start()
                    .child(
                        Button::new("add-job")
                            .small()
                            .primary()
                            .label(t!(cx, "new-job-add"))
                            .disabled(job.is_err())
                            .on_click(cx.listener(|this, _, window, cx| this.add(window, cx))),
                    )
                    .children(why_not.map(|why| {
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(why)
                    })),
            )
    }
}

/// Opens Make a job in a side panel.
pub fn open(session: Entity<Session>, window: &mut Window, cx: &mut App) {
    let view = cx.new(|cx| NewJobForm::new(session, window, cx));
    window.open_sheet(cx, move |sheet, _, cx| {
        sheet
            .title(t!(cx, "new-job-title"))
            .size(px(crate::pages::details::SHEET_WIDTH))
            .child(view.clone())
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_company_and_city_of_a_company_unit() {
        assert_eq!(
            company_and_city("company.volatile.acme.lyon"),
            Some(("acme", "lyon"))
        );
        assert_eq!(company_and_city("garage.lyon"), None);
    }

    #[test]
    fn estimates_a_route_from_the_saves_own_offers() {
        let place = |city: &str, x: f32| gamedata::MapPlace {
            kind: gamedata::MapPlaceKind::CityArea,
            city: city.into(),
            x,
            z: 0.0,
            width: 100.0,
            depth: 100.0,
        };
        let places = [
            place("lyon", 0.0),
            place("berlin", 1000.0),
            place("paris", 500.0),
        ];
        // An offer of 950 km over 1,000 m of map: 0.95 km a metre.
        let save = "SiiNunit\r\n{\r\ncompany : company.volatile.acme.lyon {\r\n job_offer: 1\r\n job_offer[0]: j1\r\n}\r\n\r\njob_offer_data : j1 {\r\n target: \"bolt.berlin\"\r\n cargo: cargo.bricks\r\n shortest_distance_km: 950\r\n}\r\n\r\n}\r\n";
        let document = sii::parse_text(save).expect("parses");
        let route = estimated_route(&document, &places, "lyon", "paris").expect("a route");
        assert_eq!(route.distance_km, 475);
        assert_eq!(estimated_route(&document, &places, "lyon", "rome"), None);
    }
}
