//! The running game, live: where the truck is, how fast it goes, the job, and what just happened
//! (a delivery, a fine, a toll).
//!
//! The game offers this through its own telemetry SDK to plugins it loads. The editor's plugin
//! (`crates/live-plugin`) is a thin shell: it hands every value the game gives it to a
//! [`Snapshot`] here, and [`serve`] sends the snapshot to whoever connects on this computer, one
//! line of JSON ten times a second. The editor reads it with [`listen`]. Everything that can be
//! tested without the game lives in this crate; nothing here is `unsafe`.
//!
//! It only ever reads. Nothing a client sends is looked at, and the port is open to this
//! computer alone (`127.0.0.1`).

mod snapshot;
mod wire;

pub use snapshot::{
    Attribute, Channel, Event, Job, Kind, PLUGIN_VERSION, Snapshot, TimedEvent, Value,
};
pub use wire::{PORT, Server, decode, encode, listen, serve, serve_on};
