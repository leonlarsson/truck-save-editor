//! Live: the game as it runs, on the map. The map fills the page and follows the truck, with the
//! road driven this session behind it and a flag on the job's destination; over it sit the drive
//! in numbers (speed against the limit, gear, fuel, rest), the truck's wear, the job, and this
//! session (what was driven, earned and paid, and what just happened). Read-only: it shows what
//! the game says (see `crate::live`) and changes nothing.
//!
//! The game says it through the editor's plugin. The page's Plugin menu puts it into the game's
//! folder, updates it and takes it out again, at any time (also while the game runs). Until the
//! game is heard, the page says how to get there.

use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, Sizable as _, StyledExt as _,
    button::{Button, ButtonGroup, ButtonVariants as _},
    checkbox::Checkbox,
    h_flex,
    popover::Popover,
    progress::Progress,
    tag::Tag,
    v_flex,
};
use gpui_kit::{
    AnyElement, App, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, Subscription, WeakEntity, Window, div, px, rems,
};

use live::{Event, Snapshot};

use crate::format::{decimal, thousands};
use crate::game_data;
use crate::i18n::t;
use crate::labels;
use crate::live::PluginState;
use crate::pages::map_view::MapView;
use crate::session::Session;
use crate::ui::{Divided, MapCard, Meter, Section, Tip as _, page_body};
use crate::units;

/// Metres a second in a kilometre an hour, and in a mile an hour.
const KMH: f32 = 3.6;
const MPH: f32 = 2.236_936;
/// US gallons in a litre.
const GALLONS_PER_LITRE: f32 = 0.264_172;
/// Minutes in a game day.
const DAY: u32 = 1440;
/// How far over the limit counts as over it, in metres a second (about 2 km/h: the needle
/// wobbles).
const OVER_THE_LIMIT_BY: f32 = 0.6;
/// How many of the newest events the session card lists.
const SHOWN_EVENTS: usize = 5;

/// How often the game's folder is looked at for the plugin (see `LivePage::plugin_looked_at`).
const PLUGIN_LOOKED_AT_EVERY: std::time::Duration = std::time::Duration::from_secs(2);

/// How often the profile is looked at for a newer save (see `LivePage::coach`).
const SAVE_LOOKED_AT_EVERY: std::time::Duration = std::time::Duration::from_secs(5);

pub struct LivePage {
    session: Entity<Session>,
    map_view: Entity<MapView>,
    /// The coach service being driven, from the profile's newest save. The game's telemetry
    /// only knows cargo jobs, but the game saves when a service starts and at every stop, so the
    /// newest save knows the service. Read again whenever the game writes a newer save (only
    /// every so often while driving, so only what holds between saves is shown: not the
    /// passengers' satisfaction, which moves all the time).
    coach: Option<save::CurrentCoachService>,
    _watching_saves: Option<gpui_kit::Task<()>>,
    /// Why adding or removing the plugin didn't work, if it didn't.
    plugin_problem: Option<SharedString>,
    /// Whether the plugin is in the game's folder, and when that was last looked at: the page
    /// is drawn ten times a second while driving, and looking is reading the disk.
    plugin_looked_at: std::cell::Cell<Option<(std::time::Instant, PluginState)>>,
    _subscriptions: Vec<Subscription>,
}

/// What the Plugin menu shows, worked out before it's drawn (the menu's content is drawn by the
/// kit, which can't borrow the page).
#[derive(Clone)]
struct PluginInfo {
    page: WeakEntity<LivePage>,
    state: PluginState,
    is_heard: bool,
    is_older: bool,
    /// Why the plugin can't be added, when it can't.
    blocked: Option<SharedString>,
    problem: Option<SharedString>,
    /// The plugin's file in the game's folder (there or not).
    file: Option<std::path::PathBuf>,
}

impl LivePage {
    pub fn new(session: Entity<Session>, cx: &mut Context<Self>) -> Self {
        let map_view = cx.new(|cx| {
            let mut map = MapView::new(session.clone(), cx).bare();
            map.follow_live(true, cx);
            map
        });
        let _subscriptions = vec![
            // Something new from the game: drawn again, and the flag follows the job.
            crate::live::observe_with(cx, |this, cx| {
                this.flag_destination(cx);
                cx.notify();
            }),
            // Its Follow player box says whether the map still follows (dragging lets go).
            cx.observe(&map_view, |_, _, cx| cx.notify()),
        ];
        // Tests have no game writing saves to watch for.
        let _watching_saves = (!cfg!(test)).then(|| Self::watch_saves(cx));
        let mut page = Self {
            session,
            map_view,
            coach: None,
            _watching_saves,
            plugin_problem: None,
            plugin_looked_at: std::cell::Cell::new(None),
            _subscriptions,
        };
        page.flag_destination(cx);
        page
    }

    /// Flags where the job goes on the map: the cargo job's city, else the coach service's next
    /// stop when that's a city's.
    fn flag_destination(&mut self, cx: &mut Context<Self>) {
        let snapshot = crate::live::snapshot(cx);
        let cargo_city = snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.job.as_ref())
            .map(|job| job.destination_city_id.clone())
            .filter(|city| !city.is_empty());
        let coach_city = self
            .coach
            .as_ref()
            .filter(|_| snapshot.is_some())
            .and_then(|coach| save::CurrentCoachService::stop_city(&coach.to_stop))
            .map(str::to_string);
        let city = cargo_city.or(coach_city);
        self.map_view
            .update(cx, |map, cx| map.set_destination(city, cx));
    }

    /// Keeps `coach` current: while the game is heard without a cargo job, looks for a newer
    /// save in the profile every few seconds and reads the coach service from it, in the
    /// background (a large save takes a moment to decode).
    fn watch_saves(cx: &mut Context<Self>) -> gpui_kit::Task<()> {
        cx.spawn(async move |this, cx| {
            // The save read last: its file and when it was written.
            let mut read: Option<(std::path::PathBuf, std::time::SystemTime)> = None;
            loop {
                // The profile to look in, when there's something to look for. The page is gone
                // when this fails: stop.
                let Ok(profile_dir) = this.update(cx, |this, cx| {
                    let is_wanted =
                        crate::live::snapshot(cx).is_some_and(|snapshot| snapshot.job.is_none());
                    is_wanted.then(|| this.session.read(cx).profile().dir.clone())
                }) else {
                    return;
                };
                if let Some(profile_dir) = profile_dir {
                    let already_read = read.clone();
                    let newer = cx
                        .background_spawn(async move {
                            let newest = newest_save(&profile_dir)?;
                            if Some(&newest) == already_read.as_ref() {
                                return None;
                            }
                            // A save the game is still writing doesn't decode: tried again on
                            // the next look.
                            let bytes = std::fs::read(&newest.0).ok()?;
                            let document = sii::read(&bytes).ok()?;
                            Some((newest, save::current_coach_service(&document)))
                        })
                        .await;
                    if let Some((newest, coach)) = newer {
                        read = Some(newest);
                        let _ = this.update(cx, |this, cx| {
                            this.coach = coach;
                            this.flag_destination(cx);
                            cx.notify();
                        });
                    }
                }
                cx.background_executor().timer(SAVE_LOOKED_AT_EVERY).await;
            }
        })
    }

    /// Puts the plugin into the game's folder (also over an older one), or takes it out.
    fn change_plugin(&mut self, is_adding: bool, cx: &mut Context<Self>) {
        let Some(install_dir) = game_data::install_dir(cx) else {
            return;
        };
        let result = if is_adding {
            crate::live::install_plugin(&install_dir)
        } else {
            crate::live::remove_plugin(&install_dir)
        };
        self.plugin_problem = result
            .err()
            .map(|error| t!(cx, "live-plugin-failed", error = error.to_string()));
        // Looked at again right away.
        self.plugin_looked_at.set(None);
        cx.notify();
    }

    /// Whether the plugin is in the game's folder, looked at again at most every other second.
    fn plugin_state(&self, install_dir: Option<&std::path::Path>) -> PluginState {
        let Some(install_dir) = install_dir else {
            return PluginState::NotAdded;
        };
        if let Some((when, state)) = self.plugin_looked_at.get()
            && when.elapsed() < PLUGIN_LOOKED_AT_EVERY
        {
            return state;
        }
        let state = crate::live::plugin_state(install_dir);
        self.plugin_looked_at
            .set(Some((std::time::Instant::now(), state)));
        state
    }

    /// What the Plugin menu and the steps say about the plugin now.
    fn plugin_info(&self, cx: &mut Context<Self>) -> PluginInfo {
        let install_dir = game_data::install_dir(cx);
        let snapshot = crate::live::snapshot(cx);
        let blocked = if install_dir.is_none() {
            Some(t!(cx, "live-needs-game-folder"))
        } else if !crate::live::has_plugin() {
            Some(t!(cx, "live-needs-plugin-file"))
        } else {
            None
        };
        PluginInfo {
            page: cx.entity().downgrade(),
            state: self.plugin_state(install_dir.as_deref()),
            is_heard: snapshot.is_some(),
            is_older: snapshot
                .as_deref()
                .is_some_and(crate::live::is_plugin_older),
            blocked,
            problem: self.plugin_problem.clone(),
            file: install_dir.as_deref().map(crate::live::plugin_path),
        }
    }

    /// The Plugin button and its menu: where the plugin is, and adding, updating and removing
    /// it. On the page whatever the game is doing, so the plugin can be removed at any time.
    fn render_plugin_menu(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let info = self.plugin_info(cx);
        Popover::new("plugin-menu")
            .anchor(gpui_kit::Anchor::TopRight)
            .trigger(
                Button::new("plugin")
                    .small()
                    .outline()
                    .label(t!(cx, "live-plugin")),
            )
            .content(move |_, _, cx| plugin_panel(&info, cx))
    }

    /// What the page shows until the game is heard: what Live is, and the way there.
    fn render_setup(&self, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let muted = cx.theme().muted_foreground;
        let info = self.plugin_info(cx);
        let is_added = info.state == PluginState::Added;

        // One step of the way: its number (ticked once done), what to do, a note, an action.
        let step = |number: usize,
                    is_done: bool,
                    title: SharedString,
                    note: SharedString,
                    action: Option<AnyElement>,
                    cx: &App| {
            let theme = cx.theme();
            // No box of its own: the steps are rows of one list, with a line between each two.
            h_flex()
                .gap_4()
                .items_center()
                .py_1()
                .child(
                    div()
                        .size(rems(1.75))
                        .flex_shrink_0()
                        .rounded_full()
                        .border_1()
                        .border_color(theme.border)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_sm()
                        .font_semibold()
                        // Done: a tick in place of the number, not only another colour.
                        .child(if is_done {
                            "✓".to_string()
                        } else {
                            number.to_string()
                        }),
                )
                .child(
                    v_flex()
                        .flex_1()
                        .min_w_0()
                        .gap_0p5()
                        .child(div().font_medium().child(title))
                        .child(
                            div()
                                .text_sm()
                                .text_color(theme.muted_foreground)
                                .child(note),
                        ),
                )
                .children(action)
        };
        let add = (!is_added).then(|| {
            Button::new("add-plugin")
                .primary()
                .label(t!(cx, "live-plugin-add"))
                .tip(
                    info.blocked
                        .clone()
                        .unwrap_or_else(|| t!(cx, "live-plugin-add-tooltip")),
                )
                .disabled(info.blocked.is_some())
                .on_click(cx.listener(|this, _, _, cx| this.change_plugin(true, cx)))
                .into_any_element()
        });
        let plugin_note = match (is_added, &info.blocked) {
            (true, _) => t!(cx, "live-plugin-added"),
            (false, Some(reason)) => reason.clone(),
            (false, None) => t!(cx, "live-plugin-not-added"),
        };
        let fact = |title: SharedString, note: SharedString| {
            v_flex()
                .gap_1()
                .child(div().child(title))
                .child(div().text_sm().text_color(muted).child(note))
        };

        page_body().child(
            v_flex()
                .gap_6()
                .child(
                    h_flex()
                        .justify_between()
                        .items_start()
                        .gap_4()
                        .child(
                            v_flex()
                                .gap_2()
                                .max_w(rems(40.))
                                .child(h_flex().gap_3().items_center().child(
                                    div().text_2xl().font_semibold().child(t!(cx, "live-setup")),
                                ))
                                .child(
                                    div()
                                        .text_color(muted)
                                        .child(t!(cx, "live-setup-description")),
                                ),
                        )
                        .child(self.render_plugin_menu(cx)),
                )
                .child(
                    h_flex()
                        .gap_6()
                        .items_start()
                        .flex_wrap()
                        .child(
                            Divided::rows()
                                .flex_1()
                                .min_w(rems(24.))
                                .child(step(
                                    1,
                                    is_added,
                                    t!(cx, "live-plugin-add"),
                                    plugin_note,
                                    add,
                                    cx,
                                ))
                                .child(step(
                                    2,
                                    false,
                                    t!(cx, "live-step-start"),
                                    t!(cx, "live-step-start-note"),
                                    None,
                                    cx,
                                ))
                                .child(step(
                                    3,
                                    false,
                                    t!(cx, "live-step-drive"),
                                    t!(cx, "live-step-drive-note"),
                                    None,
                                    cx,
                                ))
                                .children(self.plugin_problem.clone().map(|problem| {
                                    div().text_sm().text_color(cx.theme().danger).child(problem)
                                })),
                        )
                        .child(
                            Section::new(t!(cx, "live-facts"))
                                .panel()
                                .w(rems(24.))
                                .child(fact(
                                    t!(cx, "live-fact-reads"),
                                    t!(cx, "live-fact-reads-note"),
                                ))
                                .child(fact(
                                    t!(cx, "live-fact-local"),
                                    t!(cx, "live-fact-local-note"),
                                ))
                                .child(fact(
                                    t!(cx, "live-fact-remove"),
                                    t!(cx, "live-fact-remove-note"),
                                )),
                        ),
                ),
        )
    }

    /// A speed as the game shows it: `79 km/h`, `49 mph`, as the number and its unit.
    fn speed(snapshot: &Snapshot, metres_a_second: f32, cx: &App) -> (SharedString, SharedString) {
        let (per, unit) = if crate::live::game_of(snapshot).uses_miles() {
            (MPH, t!(cx, "unit-mph"))
        } else {
            (KMH, t!(cx, "unit-kmh"))
        };
        (
            thousands((metres_a_second.abs() * per).round() as i64).into(),
            unit,
        )
    }

    /// The speed, big, against the limit as a road sign, with the gear and the cruise control.
    fn render_speed(
        &self,
        snapshot: &Snapshot,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let theme = cx.theme().clone();
        let (speed, unit) = Self::speed(snapshot, snapshot.speed, cx);
        let has_limit = snapshot.speed_limit > 0.0;
        let is_over = has_limit && snapshot.speed.abs() > snapshot.speed_limit + OVER_THE_LIMIT_BY;
        let gear: SharedString = match snapshot.gear {
            0 => t!(cx, "live-gear-neutral"),
            gear if gear < 0 => t!(cx, "live-gear-reverse", gear = gear.unsigned_abs()),
            gear => gear.to_string().into(),
        };
        // A road sign is white with dark digits in either theme: the lighter of the theme's two
        // base colours behind, the darker for the number.
        let (sign, digits) = if theme.is_dark() {
            (theme.foreground, theme.background)
        } else {
            (theme.background, theme.foreground)
        };
        let small = |name: SharedString, value: SharedString| {
            h_flex()
                .gap_1p5()
                .items_baseline()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(name)
                .child(
                    div()
                        .text_sm()
                        .font_semibold()
                        .text_color(theme.foreground)
                        .child(value),
                )
        };
        MapCard::new().child(
            h_flex()
                .gap_5()
                .items_center()
                .child(
                    v_flex()
                        .child(
                            h_flex()
                                .gap_1p5()
                                .items_baseline()
                                .child(
                                    div()
                                        .text_size(rems(2.5))
                                        .line_height(rems(2.5))
                                        .font_bold()
                                        .child(speed),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .child(unit),
                                ),
                        )
                        // Said in words, not only by the sign beside it.
                        .children(is_over.then(|| {
                            div()
                                .text_xs()
                                .font_medium()
                                .text_color(theme.danger)
                                .child(t!(cx, "live-over-limit"))
                        })),
                )
                .children(has_limit.then(|| {
                    let (limit, _) = Self::speed(snapshot, snapshot.speed_limit, cx);
                    div()
                        .size(rems(3.))
                        .flex_shrink_0()
                        .rounded_full()
                        .border_4()
                        .border_color(theme.danger)
                        .bg(sign)
                        .text_color(digits)
                        .flex()
                        .items_center()
                        .justify_center()
                        .font_bold()
                        .child(limit)
                }))
                .child(
                    v_flex()
                        .gap_1()
                        .child(small(t!(cx, "live-gear"), gear))
                        .children((snapshot.cruise_control > 0.0).then(|| {
                            let (cruise, _) = Self::speed(snapshot, snapshot.cruise_control, cx);
                            small(t!(cx, "live-cruise"), cruise)
                        })),
                ),
        )
    }

    /// The tank as a bar with what's in it and how far it goes, the rest that's due, the clock.
    fn render_fuel(&self, snapshot: &Snapshot, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let game = crate::live::game_of(snapshot);
        let muted = cx.theme().muted_foreground;
        let amount = if game.uses_miles() {
            t!(
                cx,
                "amount-with-unit",
                amount = thousands((snapshot.fuel * GALLONS_PER_LITRE).round() as i64),
                unit = t!(cx, "unit-gal")
            )
        } else {
            t!(
                cx,
                "amount-with-unit",
                amount = thousands(snapshot.fuel.round() as i64),
                unit = t!(cx, "unit-l")
            )
        };
        let range = units::distance(cx, game, snapshot.fuel_range_km.round() as i64);
        let full = if snapshot.fuel_capacity > 0.0 {
            snapshot.fuel / snapshot.fuel_capacity
        } else {
            0.0
        };
        let line = |name: SharedString, value: SharedString| {
            h_flex()
                .justify_between()
                .gap_3()
                .text_xs()
                .child(div().text_color(muted).child(name))
                .child(value)
        };
        MapCard::new()
            .w(rems(15.))
            .child(Meter::new(
                "fuel",
                t!(cx, "live-fuel"),
                t!(cx, "live-fuel-value", amount = amount, range = range),
                full,
            ))
            .children((snapshot.rest_stop_minutes > 0).then(|| {
                line(
                    t!(cx, "live-rest"),
                    labels::duration(cx, i64::from(snapshot.rest_stop_minutes)),
                )
            }))
            .child(line(
                t!(cx, "day-time"),
                labels::day_time(
                    cx,
                    i64::from(snapshot.game_minutes / DAY),
                    i64::from(snapshot.game_minutes % DAY),
                ),
            ))
    }

    /// The truck: what it is, how far it has gone, and its wear, a bar a part.
    fn render_truck(
        &self,
        snapshot: &Snapshot,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let game = crate::live::game_of(snapshot);
        let name = format!("{} {}", snapshot.truck_brand, snapshot.truck_name);
        let title: SharedString = if name.trim().is_empty() {
            t!(cx, "live-vehicle")
        } else {
            name.trim().to_string().into()
        };
        let part = |id: &'static str, name: SharedString, wear: f32, cx: &App| {
            Meter::new(id, name, percent(cx, wear), wear).on_one_line()
        };
        MapCard::new()
            .w(rems(16.))
            .child(
                h_flex()
                    .justify_between()
                    .gap_3()
                    .child(div().font_semibold().child(title))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(units::distance(
                                cx,
                                game,
                                snapshot.odometer_km.round() as i64,
                            )),
                    ),
            )
            .child(part(
                "wear-engine",
                t!(cx, "part-engine"),
                snapshot.wear_engine,
                cx,
            ))
            .child(part(
                "wear-transmission",
                t!(cx, "part-transmission"),
                snapshot.wear_transmission,
                cx,
            ))
            .child(part(
                "wear-cabin",
                t!(cx, "part-cabin"),
                snapshot.wear_cabin,
                cx,
            ))
            .child(part(
                "wear-chassis",
                t!(cx, "part-chassis"),
                snapshot.wear_chassis,
                cx,
            ))
            .child(part(
                "wear-wheels",
                t!(cx, "part-wheels"),
                snapshot.wear_wheels,
                cx,
            ))
    }

    /// This session: what was driven, earned and paid since the game started, and the newest of
    /// what happened.
    fn render_session(
        &self,
        snapshot: &Snapshot,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let game = crate::live::game_of(snapshot);
        let theme = cx.theme().clone();
        let figure = |name: SharedString, value: SharedString| {
            v_flex()
                .gap_0p5()
                .child(
                    div()
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(name),
                )
                .child(div().text_lg().font_semibold().child(value))
        };
        let tally = if crate::live::is_plugin_older(snapshot) {
            // The first plugin didn't count: say how to get the numbers, don't show zeros.
            div()
                .text_xs()
                .text_color(theme.muted_foreground)
                .child(t!(cx, "live-session-needs-update"))
                .into_any_element()
        } else {
            div()
                .grid()
                .grid_cols(2)
                .gap_3()
                .child(figure(
                    t!(cx, "live-session-driven"),
                    units::distance(cx, game, snapshot.session_km.round() as i64),
                ))
                .child(figure(
                    t!(cx, "live-session-earned"),
                    money(game, snapshot.session_earned),
                ))
                .child(figure(
                    t!(cx, "live-session-xp"),
                    thousands(snapshot.session_xp).into(),
                ))
                .child(figure(
                    t!(cx, "live-session-costs"),
                    money(
                        game,
                        snapshot
                            .session_fines
                            .saturating_add(snapshot.session_travel_costs),
                    ),
                ))
                .into_any_element()
        };
        let events: Vec<AnyElement> = snapshot
            .events
            .iter()
            .rev()
            .take(SHOWN_EVENTS)
            .map(|timed| {
                let when = format!(
                    "{:02}:{:02}",
                    timed.game_minutes % DAY / 60,
                    timed.game_minutes % 60
                );
                let what = match &timed.event {
                    Event::Delivered {
                        revenue, earned_xp, ..
                    } => t!(
                        cx,
                        "live-event-delivered",
                        amount = money(game, *revenue),
                        xp = thousands(*earned_xp)
                    ),
                    Event::Cancelled { penalty } => {
                        t!(cx, "live-event-cancelled", amount = money(game, *penalty))
                    }
                    Event::Fined { amount, .. } => {
                        t!(cx, "live-event-fined", amount = money(game, *amount))
                    }
                    Event::Toll { amount } => {
                        t!(cx, "live-event-toll", amount = money(game, *amount))
                    }
                    Event::Ferry { from, to, amount } => t!(
                        cx,
                        "live-event-ferry",
                        from = from.as_str(),
                        to = to.as_str(),
                        amount = money(game, *amount)
                    ),
                    Event::Train { from, to, amount } => t!(
                        cx,
                        "live-event-train",
                        from = from.as_str(),
                        to = to.as_str(),
                        amount = money(game, *amount)
                    ),
                };
                h_flex()
                    .justify_between()
                    .items_start()
                    .gap_3()
                    .text_xs()
                    .child(div().min_w_0().child(what))
                    .child(
                        div()
                            .flex_shrink_0()
                            .text_color(theme.muted_foreground)
                            .child(when),
                    )
                    .into_any_element()
            })
            .collect();
        MapCard::new()
            .w(rems(18.))
            .gap_3()
            .child(div().font_semibold().child(t!(cx, "live-session")))
            .child(tally)
            .child(
                v_flex()
                    .gap_2()
                    .pt_3()
                    .border_t_1()
                    .border_color(theme.border)
                    .children(if events.is_empty() {
                        vec![
                            div()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(t!(cx, "live-events-none"))
                                .into_any_element(),
                        ]
                    } else {
                        events
                    }),
            )
    }

    /// The job along the foot of the map: the cargo, the way from where it was loaded to where
    /// it goes as a bar, and what's left. A coach service, from the newest save, in its place;
    /// nothing without either.
    fn render_job(&self, snapshot: &Snapshot, cx: &mut Context<Self>) -> Option<AnyElement> {
        let game = crate::live::game_of(snapshot);
        let muted = cx.theme().muted_foreground;
        let primary = cx.theme().primary;
        let figure = |name: SharedString, value: SharedString| {
            v_flex()
                .flex_shrink_0()
                .gap_0p5()
                .child(div().text_xs().text_color(muted).child(name))
                .child(div().font_semibold().child(value))
        };
        let left_km = (snapshot.navigation_metres / 1000.0).round() as i64;
        // The way as a bar: from one place to the other, as far along as the distance says.
        let way = |from: SharedString, to: SharedString, planned_km: i64| {
            let done = if planned_km > 0 {
                (1.0 - left_km as f32 / planned_km as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_1p5()
                .child(
                    h_flex()
                        .justify_between()
                        .gap_4()
                        .text_sm()
                        .child(div().min_w_0().child(from))
                        .child(div().min_w_0().text_right().child(to)),
                )
                .child(
                    Progress::new("job-way")
                        .value(done * 100.0)
                        .color(primary)
                        .accessibility_label(t!(cx, "live-job-distance-left")),
                )
        };
        let due = |deadline: i64, cx: &App| {
            let left = deadline - i64::from(snapshot.game_minutes);
            if left >= 0 {
                labels::duration(cx, left)
            } else {
                t!(cx, "live-job-late")
            }
        };
        let left = figure(
            t!(cx, "live-job-distance-left"),
            units::distance(cx, game, left_km),
        );

        if let Some(job) = &snapshot.job {
            let place = |company: &str, city: &str| -> SharedString {
                if company.is_empty() {
                    city.to_string().into()
                } else {
                    t!(cx, "job-place", company = company, city = city)
                }
            };
            let cargo: SharedString = if job.cargo_mass_kg > 0.0 {
                t!(
                    cx,
                    "live-job-cargo",
                    cargo = job.cargo.as_str(),
                    weight = units::weight(cx, game, f64::from(job.cargo_mass_kg))
                )
            } else {
                job.cargo.clone().into()
            };
            return Some(
                MapCard::new()
                    .child(
                        h_flex()
                            .gap_6()
                            .items_center()
                            .child(figure(t!(cx, "column-cargo"), cargo))
                            .child(way(
                                place(&job.source_company, &job.source_city),
                                place(&job.destination_company, &job.destination_city),
                                i64::from(job.planned_distance_km),
                            ))
                            .child(left)
                            .child(figure(
                                t!(cx, "live-job-time-left"),
                                due(i64::from(job.delivery_minutes), cx),
                            ))
                            .child(figure(
                                t!(cx, "day-cargo-damage"),
                                percent(cx, snapshot.cargo_damage),
                            ))
                            .child(figure(
                                t!(cx, "live-job-pays"),
                                money(game, i64::try_from(job.income).unwrap_or(i64::MAX)),
                            )),
                    )
                    .into_any_element(),
            );
        }

        let coach = self.coach.as_ref()?;
        // A city's stop by the city's name; the others are at viewpoints.
        let stop = |stop: &str| -> SharedString {
            match save::CurrentCoachService::stop_city(stop) {
                Some(city) => game_data::city_name(cx, city).into(),
                None => t!(cx, "gallery-viewpoint"),
            }
        };
        Some(
            MapCard::new()
                .child(
                    h_flex()
                        .gap_6()
                        .items_center()
                        .child(figure(
                            t!(cx, "live-coach"),
                            t!(
                                cx,
                                "live-coach-stage-value",
                                stage = coach.stage,
                                stages = coach.stages
                            ),
                        ))
                        .child(way(
                            stop(&coach.from_stop),
                            stop(&coach.to_stop),
                            coach.planned_distance_km,
                        ))
                        .child(left)
                        .children(
                            coach.planned_arrival.map(|arrival| {
                                figure(t!(cx, "live-job-time-left"), due(arrival, cx))
                            }),
                        )
                        .child(figure(
                            t!(cx, "column-passengers"),
                            thousands(coach.passengers as i64).into(),
                        )),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(muted)
                        .child(t!(cx, "live-coach-note")),
                )
                .into_any_element(),
        )
    }

    /// The buttons over the map's corner: the game's state, zooming, following, the plugin.
    fn render_controls(
        &self,
        snapshot: &Snapshot,
        cx: &mut Context<Self>,
    ) -> impl IntoElement + use<> {
        let follows = self.map_view.read(cx).follows_live();
        MapCard::new().px_3().py_2().child(
            h_flex()
                .gap_3()
                .items_center()
                // In a menu the game sends nothing: the numbers are from when driving stopped.
                .children(
                    snapshot
                        .is_paused
                        .then(|| Tag::secondary().small().child(t!(cx, "live-state-paused"))),
                )
                .child(
                    ButtonGroup::new("live-zoom")
                        .outline()
                        .small()
                        .child(
                            Button::new("zoom-in")
                                .label("+")
                                .tip(t!(cx, "map-view-zoom-in"))
                                .accessibility_label(t!(cx, "map-view-zoom-in")),
                        )
                        .child(
                            Button::new("zoom-out")
                                .label("−")
                                .tip(t!(cx, "map-view-zoom-out"))
                                .accessibility_label(t!(cx, "map-view-zoom-out")),
                        )
                        .child(Button::new("zoom-fit").label(t!(cx, "map-view-fit")))
                        .on_click(cx.listener(|this, clicked: &Vec<usize>, _, cx| {
                            let clicked = clicked.first().copied();
                            this.map_view.update(cx, |map, cx| match clicked {
                                Some(0) => map.zoom_in(cx),
                                Some(1) => map.zoom_out(cx),
                                Some(2) => {
                                    // The whole map: no longer about the truck.
                                    map.follow_live(false, cx);
                                    map.fit(cx);
                                }
                                _ => {}
                            });
                        })),
                )
                // Dragging the map lets go of the truck, and unticks this.
                .child(
                    Checkbox::new("follow-truck")
                        .label(t!(cx, "live-follow"))
                        .checked(follows)
                        .disabled(!snapshot.has_position)
                        .on_change(cx.listener(|this, checked: &bool, _, cx| {
                            let follows = *checked;
                            this.map_view
                                .update(cx, |map, cx| map.follow_live(follows, cx));
                        })),
                )
                .child(self.render_plugin_menu(cx)),
        )
    }
}

/// The Plugin menu's content: where the plugin is, its file, and what can be done with it.
fn plugin_panel(info: &PluginInfo, cx: &App) -> impl IntoElement + use<> {
    let theme = cx.theme();
    let muted = theme.muted_foreground;
    let state = match (info.state, info.is_heard) {
        (PluginState::Added, true) => t!(cx, "live-plugin-state-in-use"),
        (PluginState::Added, false) => t!(cx, "live-plugin-added"),
        (PluginState::RemovedButInUse, _) => t!(cx, "live-plugin-state-removed-in-use"),
        // Heard without its file: removed since, and the game hasn't closed yet.
        (PluginState::NotAdded, true) => t!(cx, "live-plugin-state-removed-in-use"),
        (PluginState::NotAdded, false) => t!(cx, "live-plugin-not-added"),
    };
    let change = |is_adding: bool| {
        let page = info.page.clone();
        move |_: &gpui_kit::ClickEvent, _: &mut Window, cx: &mut App| {
            let _ = page.update(cx, |page, cx| page.change_plugin(is_adding, cx));
        }
    };
    let is_added = info.state == PluginState::Added;
    let folder = info
        .file
        .as_ref()
        .and_then(|file| file.parent().map(std::path::Path::to_path_buf));
    v_flex()
        .w(rems(22.))
        .gap_3()
        .child(
            v_flex()
                .gap_1()
                .child(div().font_semibold().child(t!(cx, "live-plugin-title")))
                .child(div().text_sm().text_color(muted).child(state)),
        )
        .children(info.file.as_ref().map(|file| {
            div()
                .text_xs()
                .text_color(muted)
                .child(crate::ui::path_text(Some(file.as_path()), cx))
        }))
        // A newer plugin comes with this editor than the one the game has.
        .children(
            (info.is_older && is_added).then(|| div().text_sm().child(t!(cx, "live-plugin-older"))),
        )
        .child(
            h_flex()
                .gap_2()
                .flex_wrap()
                .children(folder.filter(|folder| folder.is_dir()).map(|folder| {
                    Button::new("open-plugin-folder")
                        .small()
                        .outline()
                        .label(t!(cx, "live-plugin-open-folder"))
                        .on_click(move |_, _, cx| cx.reveal_path(&folder))
                }))
                .children((info.is_older && is_added).then(|| {
                    Button::new("update-plugin")
                        .small()
                        .primary()
                        .label(t!(cx, "live-plugin-update"))
                        .disabled(info.blocked.is_some())
                        .on_click(change(true))
                }))
                .children(is_added.then(|| {
                    Button::new("remove-plugin")
                        .small()
                        .danger()
                        .outline()
                        .label(t!(cx, "live-plugin-remove"))
                        .on_click(change(false))
                }))
                .children((!is_added).then(|| {
                    Button::new("add-plugin-menu")
                        .small()
                        .primary()
                        .label(t!(cx, "live-plugin-add"))
                        .disabled(info.blocked.is_some())
                        .on_click(change(true))
                })),
        )
        .children(
            info.blocked
                .clone()
                .filter(|_| !is_added)
                .map(|reason| div().text_xs().text_color(muted).child(reason)),
        )
        .children(
            info.problem
                .clone()
                .map(|problem| div().text_xs().text_color(theme.danger).child(problem)),
        )
        .child(
            div()
                .text_xs()
                .text_color(muted)
                .child(t!(cx, "live-plugin-in-use-note")),
        )
}

/// The profile's save written last: its `game.sii` and when it was written.
fn newest_save(
    profile_dir: &std::path::Path,
) -> Option<(std::path::PathBuf, std::time::SystemTime)> {
    std::fs::read_dir(profile_dir.join("save"))
        .ok()?
        .flatten()
        .filter_map(|slot| {
            let file = slot.path().join("game.sii");
            let written = std::fs::metadata(&file).ok()?.modified().ok()?;
            Some((file, written))
        })
        .max_by_key(|(_, written)| *written)
}

/// An amount in the game's currency.
fn money(game: save::Game, amount: i64) -> SharedString {
    format!("{} {}", game.currency_symbol(), thousands(amount)).into()
}

/// A share (wear, damage) from 0 to 1, as a percentage with one decimal.
fn percent(cx: &gpui_kit::App, share: f32) -> SharedString {
    t!(
        cx,
        "live-percent",
        percent = decimal(f64::from(share) * 100.0, 1)
    )
}

impl Render for LivePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let Some(snapshot) = crate::live::snapshot(cx) else {
            return self.render_setup(cx).into_any_element();
        };
        // The other game is the one running: its numbers are right, but the map is this save's
        // game's, so the map shows no position. Said on the page, with what to do.
        let running = crate::live::game_of(&snapshot);
        let open = self.session.read(cx).game();
        let other_game = (running != open).then(|| {
            MapCard::new()
                .max_w(rems(26.))
                .child(div().text_sm().child(t!(
                    cx,
                    "live-other-game",
                    running = running.name(),
                    open = open.name()
                )))
        });
        // The cards' distance from the map's edges.
        let edge = px(16.);
        div()
            .size_full()
            .relative()
            .overflow_hidden()
            .child(div().absolute().inset_0().child(self.map_view.clone()))
            // Top, leading: the drive.
            .child(
                h_flex()
                    .absolute()
                    .top(edge)
                    .left(edge)
                    .gap_3()
                    .items_start()
                    .child(self.render_speed(&snapshot, cx))
                    .child(self.render_fuel(&snapshot, cx)),
            )
            .children(other_game.map(|notice| {
                // Under the drive's cards, where the eye already is.
                div().absolute().top(px(140.)).left(edge).child(notice)
            }))
            // Top, trailing: the buttons, and this session under them.
            .child(
                v_flex()
                    .absolute()
                    .top(edge)
                    .right(edge)
                    .gap_3()
                    .items_end()
                    .child(self.render_controls(&snapshot, cx))
                    .child(self.render_session(&snapshot, cx)),
            )
            // The foot: the truck, and the job across the width under it.
            .child(
                v_flex()
                    .absolute()
                    .left(edge)
                    .right(edge)
                    .bottom(edge)
                    .gap_3()
                    .items_start()
                    .child(self.render_truck(&snapshot, cx))
                    .children(
                        self.render_job(&snapshot, cx)
                            .map(|job| div().w_full().child(job)),
                    ),
            )
            .into_any_element()
    }
}
