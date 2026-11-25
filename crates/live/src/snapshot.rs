//! The game's state at one moment, and how the game's values fill it in.
//!
//! The game gives three kinds of things: **channels** (one value each, again whenever it changes:
//! the speed, the position), **configurations** (a named group of values that changes rarely: the
//! truck, the job) and **gameplay events** (something happened: delivered, fined).

use serde::{Deserialize, Serialize};

/// How many events a snapshot keeps: the newest.
const KEPT_EVENTS: usize = 20;

/// The plugin's version, sent with every snapshot. Goes up when the plugin learns to send
/// something new, so an editor can tell an older plugin is in the game.
pub const PLUGIN_VERSION: u32 = 2;

/// The most the odometer moves between two values the game sends, in kilometres: more is another
/// truck's odometer (the player switched vehicles), not driving.
const LARGEST_ODOMETER_STEP: f32 = 1.0;

/// The game as it is now. Distances and speeds are the game's own units (metres, metres a
/// second, kilometres where the name says so); the editor shows them in the player's.
///
/// `#[serde(default)]`: a field the other side doesn't know is left out or ignored, so a plugin
/// and an editor of different versions still understand each other.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Snapshot {
    /// Which version of the plugin sent this ([`PLUGIN_VERSION`]); 0 from the first plugin, which
    /// didn't say. The editor offers to update an older one.
    pub plugin_version: u32,
    /// The game's id: `eut2` or `ats`.
    pub game: String,
    /// In a menu, or not driving yet. The game sends no values while paused, so the rest is as
    /// it was when driving stopped.
    pub is_paused: bool,
    /// The game's clock: minutes since its first day began.
    pub game_minutes: u32,
    /// Minutes until the driver must rest.
    pub rest_stop_minutes: i32,

    /// Whether the game has said where the truck is yet.
    pub has_position: bool,
    /// The truck on the map, in the game's metres: `x` east, `y` up, `z` south.
    pub x: f64,
    pub y: f64,
    pub z: f64,
    /// Where it points, 0 to 1 counterclockwise from north (0.25 is west).
    pub heading: f32,

    /// Metres a second, as the speedometer shows (negative in reverse).
    pub speed: f32,
    pub rpm: f32,
    /// The gear the dashboard shows: above 0 forward, 0 neutral, below 0 reverse.
    pub gear: i32,
    /// The cruise control's speed in metres a second, 0 when off.
    pub cruise_control: f32,
    /// The road's speed limit in metres a second, 0 when the game knows none.
    pub speed_limit: f32,
    pub is_parking_brake_on: bool,

    /// Litres.
    pub fuel: f32,
    pub fuel_capacity: f32,
    pub fuel_range_km: f32,
    pub odometer_km: f32,

    /// What the navigation says is left: metres, and its estimate of the time in seconds.
    pub navigation_metres: f32,
    pub navigation_seconds: f32,

    /// Wear, 0 (new) to 1.
    pub wear_engine: f32,
    pub wear_transmission: f32,
    pub wear_cabin: f32,
    pub wear_chassis: f32,
    pub wear_wheels: f32,
    pub is_trailer_connected: bool,
    pub cargo_damage: f32,

    /// The truck's make and model, as the game names them (`Scania`, `S`); empty without a truck.
    pub truck_brand: String,
    pub truck_name: String,

    pub job: Option<Job>,
    /// What happened lately, oldest first.
    pub events: Vec<TimedEvent>,

    /// Since the game started (while the plugin has been loaded): how far the player drove, what
    /// deliveries paid and earned in XP, and what fines and travel (tolls, ferries, trains) cost.
    pub session_km: f32,
    pub session_deliveries: u32,
    pub session_earned: i64,
    pub session_xp: i64,
    pub session_fines: i64,
    pub session_travel_costs: i64,
}

/// The job being driven.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Job {
    /// Ids are the game's tokens (`hamburg`, `posped`), as saves use them; names are in the
    /// game's language.
    pub cargo_id: String,
    pub cargo: String,
    pub source_city_id: String,
    pub source_city: String,
    pub source_company_id: String,
    pub source_company: String,
    pub destination_city_id: String,
    pub destination_city: String,
    pub destination_company_id: String,
    pub destination_company: String,
    /// What it pays without penalties.
    pub income: u64,
    /// When it's late: the game's clock then (see `Snapshot::game_minutes`).
    pub delivery_minutes: u32,
    pub planned_distance_km: u32,
    pub cargo_mass_kg: f32,
    pub is_cargo_loaded: bool,
}

/// Something that happened, and the game's clock when it did.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TimedEvent {
    pub game_minutes: u32,
    pub event: Event,
}

/// Something that happened while playing.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Event {
    Delivered {
        revenue: i64,
        earned_xp: i64,
        distance_km: f64,
        cargo_damage: f64,
    },
    Cancelled {
        penalty: i64,
    },
    Fined {
        /// The game's name for what was done (`speeding`, `red_signal`).
        offence: String,
        amount: i64,
    },
    Toll {
        amount: i64,
    },
    Ferry {
        from: String,
        to: String,
        amount: i64,
    },
    Train {
        from: String,
        to: String,
        amount: i64,
    },
}

/// One value from the game, whatever its type there.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Bool(bool),
    Integer(i64),
    Number(f64),
    Text(String),
    Placement {
        x: f64,
        y: f64,
        z: f64,
        heading: f32,
    },
    /// A type the snapshot has no use for.
    Other,
}

impl Value {
    /// The value as a number. One that isn't a number at all (not finite) is no value: it
    /// couldn't be sent on (JSON has no way to write it) and means nothing on screen.
    fn number(&self) -> Option<f64> {
        match self {
            Value::Number(number) if number.is_finite() => Some(*number),
            Value::Integer(integer) => Some(*integer as f64),
            _ => None,
        }
    }

    fn integer(&self) -> Option<i64> {
        match self {
            Value::Integer(integer) => Some(*integer),
            Value::Number(number) => Some(*number as i64),
            _ => None,
        }
    }

    fn bool(&self) -> Option<bool> {
        match self {
            Value::Bool(value) => Some(*value),
            _ => None,
        }
    }

    fn text(&self) -> Option<&str> {
        match self {
            Value::Text(text) => Some(text),
            _ => None,
        }
    }
}

/// One named value of a configuration or an event.
#[derive(Clone, Debug, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: Value,
}

/// The type a channel is asked for in (the game converts where it can).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Bool,
    S32,
    U32,
    Float,
    /// A position and a direction, in double precision.
    Placement,
}

/// The channels the snapshot is filled from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    GameTime,
    RestStop,
    Placement,
    Speed,
    Rpm,
    Gear,
    CruiseControl,
    SpeedLimit,
    ParkingBrake,
    Fuel,
    FuelRange,
    Odometer,
    NavigationDistance,
    NavigationTime,
    WearEngine,
    WearTransmission,
    WearCabin,
    WearChassis,
    WearWheels,
    TrailerConnected,
    CargoDamage,
}

impl Channel {
    pub const ALL: [Channel; 21] = [
        Channel::GameTime,
        Channel::RestStop,
        Channel::Placement,
        Channel::Speed,
        Channel::Rpm,
        Channel::Gear,
        Channel::CruiseControl,
        Channel::SpeedLimit,
        Channel::ParkingBrake,
        Channel::Fuel,
        Channel::FuelRange,
        Channel::Odometer,
        Channel::NavigationDistance,
        Channel::NavigationTime,
        Channel::WearEngine,
        Channel::WearTransmission,
        Channel::WearCabin,
        Channel::WearChassis,
        Channel::WearWheels,
        Channel::TrailerConnected,
        Channel::CargoDamage,
    ];

    /// The channel's name in the game's SDK, and the type to ask for.
    pub fn name_and_kind(self) -> (&'static str, Kind) {
        match self {
            Channel::GameTime => ("game.time", Kind::U32),
            Channel::RestStop => ("rest.stop", Kind::S32),
            Channel::Placement => ("truck.world.placement", Kind::Placement),
            Channel::Speed => ("truck.speed", Kind::Float),
            Channel::Rpm => ("truck.engine.rpm", Kind::Float),
            Channel::Gear => ("truck.displayed.gear", Kind::S32),
            Channel::CruiseControl => ("truck.cruise_control", Kind::Float),
            Channel::SpeedLimit => ("truck.navigation.speed.limit", Kind::Float),
            Channel::ParkingBrake => ("truck.brake.parking", Kind::Bool),
            Channel::Fuel => ("truck.fuel.amount", Kind::Float),
            Channel::FuelRange => ("truck.fuel.range", Kind::Float),
            Channel::Odometer => ("truck.odometer", Kind::Float),
            Channel::NavigationDistance => ("truck.navigation.distance", Kind::Float),
            Channel::NavigationTime => ("truck.navigation.time", Kind::Float),
            Channel::WearEngine => ("truck.wear.engine", Kind::Float),
            Channel::WearTransmission => ("truck.wear.transmission", Kind::Float),
            Channel::WearCabin => ("truck.wear.cabin", Kind::Float),
            Channel::WearChassis => ("truck.wear.chassis", Kind::Float),
            Channel::WearWheels => ("truck.wear.wheels", Kind::Float),
            Channel::TrailerConnected => ("trailer.connected", Kind::Bool),
            Channel::CargoDamage => ("job.cargo.damage", Kind::Float),
        }
    }
}

impl Snapshot {
    /// Takes one channel's new value. A value of an unexpected type is left alone.
    pub fn set(&mut self, channel: Channel, value: &Value) {
        if channel == Channel::Placement {
            if let Value::Placement { x, y, z, heading } = value
                && x.is_finite()
                && y.is_finite()
                && z.is_finite()
                && heading.is_finite()
            {
                self.has_position = true;
                self.x = *x;
                self.y = *y;
                self.z = *z;
                self.heading = *heading;
            }
            return;
        }
        if let Some(is_on) = value.bool() {
            match channel {
                Channel::ParkingBrake => self.is_parking_brake_on = is_on,
                Channel::TrailerConnected => self.is_trailer_connected = is_on,
                _ => {}
            }
            return;
        }
        let Some(number) = value.number() else {
            return;
        };
        let float = number as f32;
        match channel {
            Channel::GameTime => self.game_minutes = number.max(0.0) as u32,
            Channel::RestStop => self.rest_stop_minutes = number as i32,
            Channel::Speed => self.speed = float,
            Channel::Rpm => self.rpm = float,
            Channel::Gear => self.gear = number as i32,
            Channel::CruiseControl => self.cruise_control = float,
            Channel::SpeedLimit => self.speed_limit = float,
            Channel::Fuel => self.fuel = float,
            Channel::FuelRange => self.fuel_range_km = float,
            Channel::Odometer => {
                // Driven this session: what the odometer adds, in small steps. The first value
                // (from 0) and a jump to another truck's odometer aren't driving.
                let step = float - self.odometer_km;
                if self.odometer_km > 0.0 && step > 0.0 && step < LARGEST_ODOMETER_STEP {
                    self.session_km += step;
                }
                self.odometer_km = float;
            }
            Channel::NavigationDistance => self.navigation_metres = float,
            Channel::NavigationTime => self.navigation_seconds = float,
            Channel::WearEngine => self.wear_engine = float,
            Channel::WearTransmission => self.wear_transmission = float,
            Channel::WearCabin => self.wear_cabin = float,
            Channel::WearChassis => self.wear_chassis = float,
            Channel::WearWheels => self.wear_wheels = float,
            Channel::CargoDamage => self.cargo_damage = float,
            Channel::Placement | Channel::ParkingBrake | Channel::TrailerConnected => {}
        }
    }

    /// Takes a configuration: `truck` (the make, the model, the tank) or `job`. The game sends
    /// one without attributes when there's no truck, or no job, any more.
    pub fn apply_configuration(&mut self, id: &str, attributes: &[Attribute]) {
        let text = |name: &str| -> String {
            find(attributes, name)
                .and_then(Value::text)
                .unwrap_or_default()
                .to_string()
        };
        let number = |name: &str| find(attributes, name).and_then(Value::number);
        match id {
            "truck" => {
                self.truck_brand = text("brand");
                self.truck_name = text("name");
                self.fuel_capacity = number("fuel.capacity").unwrap_or(0.0) as f32;
            }
            "job" => {
                if attributes.is_empty() {
                    self.job = None;
                    self.cargo_damage = 0.0;
                    return;
                }
                self.job = Some(Job {
                    cargo_id: text("cargo.id"),
                    cargo: text("cargo"),
                    source_city_id: text("source.city.id"),
                    source_city: text("source.city"),
                    source_company_id: text("source.company.id"),
                    source_company: text("source.company"),
                    destination_city_id: text("destination.city.id"),
                    destination_city: text("destination.city"),
                    destination_company_id: text("destination.company.id"),
                    destination_company: text("destination.company"),
                    income: number("income").unwrap_or(0.0).max(0.0) as u64,
                    delivery_minutes: number("delivery.time").unwrap_or(0.0).max(0.0) as u32,
                    planned_distance_km: number("planned_distance.km").unwrap_or(0.0).max(0.0)
                        as u32,
                    cargo_mass_kg: number("cargo.mass").unwrap_or(0.0) as f32,
                    is_cargo_loaded: find(attributes, "cargo.loaded")
                        .and_then(Value::bool)
                        .unwrap_or(false),
                });
            }
            _ => {}
        }
    }

    /// Takes a gameplay event and keeps it among the newest. One the snapshot doesn't know is
    /// left out.
    pub fn apply_event(&mut self, id: &str, attributes: &[Attribute]) {
        let text = |name: &str| -> String {
            find(attributes, name)
                .and_then(Value::text)
                .unwrap_or_default()
                .to_string()
        };
        let integer = |name: &str| find(attributes, name).and_then(Value::integer).unwrap_or(0);
        let number = |name: &str| {
            find(attributes, name)
                .and_then(Value::number)
                .unwrap_or(0.0)
        };
        let event = match id {
            "job.delivered" => Event::Delivered {
                revenue: integer("revenue"),
                earned_xp: integer("earned.xp"),
                distance_km: number("distance.km"),
                cargo_damage: number("cargo.damage"),
            },
            "job.cancelled" => Event::Cancelled {
                penalty: integer("cancel.penalty"),
            },
            "player.fined" => Event::Fined {
                offence: text("fine.offence"),
                amount: integer("fine.amount"),
            },
            "player.tollgate.paid" => Event::Toll {
                amount: integer("pay.amount"),
            },
            "player.use.ferry" => Event::Ferry {
                from: text("source.name"),
                to: text("target.name"),
                amount: integer("pay.amount"),
            },
            "player.use.train" => Event::Train {
                from: text("source.name"),
                to: text("target.name"),
                amount: integer("pay.amount"),
            },
            _ => return,
        };
        // The session's tally.
        match &event {
            Event::Delivered {
                revenue, earned_xp, ..
            } => {
                // Saturating: an absurd amount must never overflow (which would end the game in
                // a build that checks for it).
                self.session_deliveries = self.session_deliveries.saturating_add(1);
                self.session_earned = self.session_earned.saturating_add(*revenue);
                self.session_xp = self.session_xp.saturating_add(*earned_xp);
            }
            Event::Cancelled { penalty } => {
                self.session_fines = self.session_fines.saturating_add(*penalty)
            }
            Event::Fined { amount, .. } => {
                self.session_fines = self.session_fines.saturating_add(*amount)
            }
            Event::Toll { amount } | Event::Ferry { amount, .. } | Event::Train { amount, .. } => {
                self.session_travel_costs = self.session_travel_costs.saturating_add(*amount)
            }
        }
        self.events.push(TimedEvent {
            game_minutes: self.game_minutes,
            event,
        });
        if self.events.len() > KEPT_EVENTS {
            let extra = self.events.len() - KEPT_EVENTS;
            self.events.drain(..extra);
        }
    }
}

/// The longest a name in a snapshot gets, in characters, and the farthest a position is from the
/// map's middle, in metres: what `Snapshot::made_safe` holds a snapshot to.
const LONGEST_TEXT: usize = 200;
const FARTHEST: f64 = 2_000_000.0;

impl Snapshot {
    /// The snapshot with anything that can't be real taken out: what the editor does to every
    /// snapshot it reads, since any program on the computer could be the one sending (not only
    /// the plugin). Numbers that aren't finite become 0, a position off the map is no position,
    /// names are cut to a sane length and the events to the newest few.
    pub fn made_safe(mut self) -> Self {
        fn text(value: &mut String) {
            if let Some((cut, _)) = value.char_indices().nth(LONGEST_TEXT) {
                value.truncate(cut);
            }
        }
        for number in [
            &mut self.heading,
            &mut self.speed,
            &mut self.rpm,
            &mut self.cruise_control,
            &mut self.speed_limit,
            &mut self.fuel,
            &mut self.fuel_capacity,
            &mut self.fuel_range_km,
            &mut self.odometer_km,
            &mut self.navigation_metres,
            &mut self.navigation_seconds,
            &mut self.wear_engine,
            &mut self.wear_transmission,
            &mut self.wear_cabin,
            &mut self.wear_chassis,
            &mut self.wear_wheels,
            &mut self.cargo_damage,
            &mut self.session_km,
        ] {
            if !number.is_finite() {
                *number = 0.0;
            }
        }
        let is_on_the_map = [self.x, self.y, self.z]
            .iter()
            .all(|at| at.is_finite() && at.abs() < FARTHEST);
        if !is_on_the_map {
            self.has_position = false;
            (self.x, self.y, self.z) = (0.0, 0.0, 0.0);
        }
        text(&mut self.game);
        text(&mut self.truck_brand);
        text(&mut self.truck_name);
        if let Some(job) = &mut self.job {
            if !job.cargo_mass_kg.is_finite() {
                job.cargo_mass_kg = 0.0;
            }
            for value in [
                &mut job.cargo_id,
                &mut job.cargo,
                &mut job.source_city_id,
                &mut job.source_city,
                &mut job.source_company_id,
                &mut job.source_company,
                &mut job.destination_city_id,
                &mut job.destination_city,
                &mut job.destination_company_id,
                &mut job.destination_company,
            ] {
                text(value);
            }
        }
        if self.events.len() > KEPT_EVENTS {
            let extra = self.events.len() - KEPT_EVENTS;
            self.events.drain(..extra);
        }
        for timed in &mut self.events {
            match &mut timed.event {
                Event::Delivered {
                    distance_km,
                    cargo_damage,
                    ..
                } => {
                    for number in [distance_km, cargo_damage] {
                        if !number.is_finite() {
                            *number = 0.0;
                        }
                    }
                }
                Event::Fined { offence, .. } => text(offence),
                Event::Ferry { from, to, .. } | Event::Train { from, to, .. } => {
                    text(from);
                    text(to);
                }
                Event::Cancelled { .. } | Event::Toll { .. } => {}
            }
        }
        self
    }
}

fn find<'a>(attributes: &'a [Attribute], name: &str) -> Option<&'a Value> {
    attributes
        .iter()
        .find(|attribute| attribute.name == name)
        .map(|attribute| &attribute.value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attribute(name: &str, value: Value) -> Attribute {
        Attribute {
            name: name.to_string(),
            value,
        }
    }

    #[test]
    fn channels_fill_the_snapshot() {
        let mut snapshot = Snapshot::default();
        snapshot.set(Channel::Speed, &Value::Number(22.5));
        snapshot.set(Channel::Gear, &Value::Integer(-1));
        snapshot.set(Channel::ParkingBrake, &Value::Bool(true));
        snapshot.set(
            Channel::Placement,
            &Value::Placement {
                x: 1200.0,
                y: 40.0,
                z: -3400.0,
                heading: 0.25,
            },
        );
        // The wrong type changes nothing.
        snapshot.set(Channel::Rpm, &Value::Text("fast".to_string()));
        assert_eq!(snapshot.speed, 22.5);
        assert_eq!(snapshot.gear, -1);
        assert!(snapshot.is_parking_brake_on);
        assert!(snapshot.has_position);
        assert_eq!((snapshot.x, snapshot.z), (1200.0, -3400.0));
        assert_eq!(snapshot.rpm, 0.0);
    }

    #[test]
    fn every_channel_has_its_own_name() {
        let mut names: Vec<&str> = Channel::ALL
            .iter()
            .map(|channel| channel.name_and_kind().0)
            .collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), Channel::ALL.len());
    }

    #[test]
    fn a_job_comes_and_goes() {
        let mut snapshot = Snapshot::default();
        snapshot.apply_configuration(
            "job",
            &[
                attribute("cargo", Value::Text("Apples".to_string())),
                attribute("destination.city.id", Value::Text("oslo".to_string())),
                attribute("income", Value::Integer(18_400)),
                attribute("planned_distance.km", Value::Integer(480)),
                attribute("cargo.loaded", Value::Bool(true)),
            ],
        );
        let job = snapshot.job.clone().unwrap_or_default();
        assert_eq!(job.cargo, "Apples");
        assert_eq!(job.destination_city_id, "oslo");
        assert_eq!(job.income, 18_400);
        assert_eq!(job.planned_distance_km, 480);
        assert!(job.is_cargo_loaded);

        snapshot.apply_configuration("job", &[]);
        assert_eq!(snapshot.job, None);
    }

    #[test]
    fn tallies_the_session() {
        let mut snapshot = Snapshot::default();
        // The first odometer value isn't driving; small steps after it are; a jump (another
        // truck) isn't.
        for km in [48_000.0, 48_000.4, 48_000.9, 120_500.0, 120_500.5] {
            snapshot.set(Channel::Odometer, &Value::Number(km));
        }
        assert!((snapshot.session_km - 1.4).abs() < 0.01);

        snapshot.apply_event(
            "job.delivered",
            &[
                attribute("revenue", Value::Integer(21_300)),
                attribute("earned.xp", Value::Integer(640)),
            ],
        );
        snapshot.apply_event(
            "player.fined",
            &[attribute("fine.amount", Value::Integer(550))],
        );
        snapshot.apply_event(
            "player.tollgate.paid",
            &[attribute("pay.amount", Value::Integer(35))],
        );
        assert_eq!(snapshot.session_deliveries, 1);
        assert_eq!(snapshot.session_earned, 21_300);
        assert_eq!(snapshot.session_xp, 640);
        assert_eq!(snapshot.session_fines, 550);
        assert_eq!(snapshot.session_travel_costs, 35);
    }

    #[test]
    fn values_that_arent_numbers_are_left_out_and_a_read_snapshot_is_made_safe() {
        let mut snapshot = Snapshot::default();
        snapshot.set(Channel::Speed, &Value::Number(20.0));
        snapshot.set(Channel::Speed, &Value::Number(f64::NAN));
        snapshot.set(
            Channel::Placement,
            &Value::Placement {
                x: f64::INFINITY,
                y: 0.0,
                z: 0.0,
                heading: 0.0,
            },
        );
        assert_eq!(snapshot.speed, 20.0);
        assert!(!snapshot.has_position);

        // As another program might send it.
        let hostile = Snapshot {
            has_position: true,
            x: 1e12,
            speed: f32::INFINITY,
            fuel: f32::NAN,
            truck_name: "é".repeat(5000),
            events: vec![
                TimedEvent {
                    game_minutes: 0,
                    event: Event::Toll { amount: 1 },
                };
                500
            ],
            ..Snapshot::default()
        }
        .made_safe();
        assert!(!hostile.has_position);
        assert_eq!((hostile.speed, hostile.fuel), (0.0, 0.0));
        assert_eq!(hostile.truck_name.chars().count(), LONGEST_TEXT);
        assert_eq!(hostile.events.len(), KEPT_EVENTS);
    }

    #[test]
    fn keeps_the_newest_events() {
        let mut snapshot = Snapshot::default();
        for amount in 0..30 {
            snapshot.game_minutes = amount as u32;
            snapshot.apply_event(
                "player.tollgate.paid",
                &[attribute("pay.amount", Value::Integer(amount))],
            );
        }
        snapshot.apply_event("something.new", &[]);
        assert_eq!(snapshot.events.len(), KEPT_EVENTS);
        assert_eq!(
            snapshot.events.last(),
            Some(&TimedEvent {
                game_minutes: 29,
                event: Event::Toll { amount: 29 }
            })
        );
    }
}
