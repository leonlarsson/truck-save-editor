//! The plugin the game loads to tell the editor what's happening while it runs.
//!
//! The game (ETS2 and ATS alike) loads every library in its `bin\win_x64\plugins` folder and
//! calls `scs_telemetry_init` in it, handing over functions to ask for values with. This plugin
//! asks for the ones in `live::Channel::ALL`, the truck's and the job's configuration and the
//! gameplay events, puts them all into one `live::Snapshot`, and lets `live::serve` send that to
//! the editor. It reads only: the telemetry SDK has no way to change the game, and nothing the
//! editor sends is looked at.
//!
//! **This is the one crate with `unsafe`**: the SDK is a C interface (pointers to C strings and
//! C structures, functions called from C). Each `unsafe` block says why it holds. The types below
//! are written from the SDK's headers (version 1.14, `scssdk_*.h`), whose sizes are checked at
//! compile time the way the headers check them.
//!
//! Nothing here may panic: a panic can't cross into the game's C++ code, so Rust would end the
//! whole game instead. Every failure is skipped quietly.

use std::ffi::{CStr, CString, c_char, c_void};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};

use live::{Attribute, Channel, Kind, Snapshot, Value};

// --- The SDK's types (scssdk.h, scssdk_value.h, scssdk_telemetry*.h), for 64-bit builds ---

/// `scs_result_t`.
const RESULT_OK: i32 = 0;
const RESULT_UNSUPPORTED: i32 = -1;

/// `SCS_TELEMETRY_VERSION_1_00` and `_1_01` (`major << 16 | minor`). 1.01 added gameplay events.
const TELEMETRY_VERSION_1_00: u32 = 1 << 16;
const TELEMETRY_VERSION_1_01: u32 = (1 << 16) | 1;

/// `SCS_U32_NIL`: the index of a channel that isn't a list.
const NO_INDEX: u32 = u32::MAX;

/// `scs_event_t`.
const EVENT_PAUSED: u32 = 3;
const EVENT_STARTED: u32 = 4;
const EVENT_CONFIGURATION: u32 = 5;
const EVENT_GAMEPLAY: u32 = 6;

/// `scs_value_type_t`.
const TYPE_BOOL: u32 = 1;
const TYPE_S32: u32 = 2;
const TYPE_U32: u32 = 3;
const TYPE_U64: u32 = 4;
const TYPE_FLOAT: u32 = 5;
const TYPE_DOUBLE: u32 = 6;
const TYPE_DPLACEMENT: u32 = 11;
const TYPE_STRING: u32 = 12;
const TYPE_S64: u32 = 13;

/// `SCS_LOG_TYPE_message`.
const LOG_MESSAGE: i32 = 0;

/// `scs_value_dplacement_t`: a position (x east, y up, z south) and a direction (heading, pitch,
/// roll).
#[repr(C)]
#[derive(Clone, Copy)]
struct ScsPlacement {
    position: [f64; 3],
    orientation: [f32; 3],
    _padding: u32,
}

/// The storage of `scs_value_t`: which field holds the value is said by its `kind`.
#[repr(C)]
#[derive(Clone, Copy)]
union ScsValueData {
    bool: u8,
    s32: i32,
    u32: u32,
    u64: u64,
    s64: i64,
    float: f32,
    double: f64,
    placement: ScsPlacement,
    string: *const c_char,
}

/// `scs_value_t`.
#[repr(C)]
struct ScsValue {
    kind: u32,
    _padding: u32,
    data: ScsValueData,
}

/// `scs_named_value_t`: one attribute of a configuration or an event.
#[repr(C)]
struct ScsNamedValue {
    name: *const c_char,
    index: u32,
    _padding: u32,
    value: ScsValue,
}

/// `scs_telemetry_configuration_t` and `scs_telemetry_gameplay_event_t` (the same shape): an id
/// and attributes, the last one marked by a null name.
#[repr(C)]
struct ScsNamedGroup {
    id: *const c_char,
    attributes: *const ScsNamedValue,
}

type EventCallback =
    unsafe extern "system" fn(event: u32, info: *const c_void, context: *mut c_void);
type ChannelCallback = unsafe extern "system" fn(
    name: *const c_char,
    index: u32,
    value: *const ScsValue,
    context: *mut c_void,
);

/// `scs_telemetry_init_params_v100_t` (and v101, which is the same). The functions are `Option`s
/// so a null one is seen rather than called.
#[repr(C)]
struct InitParams {
    game_name: *const c_char,
    game_id: *const c_char,
    game_version: u32,
    _padding: u32,
    log: Option<unsafe extern "system" fn(kind: i32, message: *const c_char)>,
    register_for_event: Option<
        unsafe extern "system" fn(event: u32, callback: EventCallback, context: *mut c_void) -> i32,
    >,
    unregister_from_event: Option<unsafe extern "system" fn(event: u32) -> i32>,
    register_for_channel: Option<
        unsafe extern "system" fn(
            name: *const c_char,
            index: u32,
            kind: u32,
            flags: u32,
            callback: ChannelCallback,
            context: *mut c_void,
        ) -> i32,
    >,
    unregister_from_channel:
        Option<unsafe extern "system" fn(name: *const c_char, index: u32, kind: u32) -> i32>,
}

// The sizes the headers check (`scs_check_size`), for 64-bit: a mismatch here means the types
// above are wrong, and must not build.
#[cfg(target_pointer_width = "64")]
const _: () = {
    assert!(size_of::<ScsPlacement>() == 40);
    assert!(size_of::<ScsValue>() == 48);
    assert!(size_of::<ScsNamedValue>() == 64);
    assert!(size_of::<ScsNamedGroup>() == 16);
    assert!(size_of::<InitParams>() == 64);
};
#[cfg(not(target_pointer_width = "64"))]
compile_error!("The plugin is written for the 64-bit game only.");

// --- The plugin ---

/// The game as last heard, shared with the thread that sends it. Made on first use.
static SNAPSHOT: LazyLock<Arc<Mutex<Snapshot>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Snapshot::default())));

/// The sending thread, while the plugin is loaded.
static SERVER: Mutex<Option<live::Server>> = Mutex::new(None);

/// Changes the snapshot. A lock poisoned by a panic elsewhere still holds a usable snapshot.
fn update(change: impl FnOnce(&mut Snapshot)) {
    let mut snapshot = SNAPSHOT.lock().unwrap_or_else(PoisonError::into_inner);
    change(&mut snapshot);
}

/// A C string from the game as text; empty for a null pointer.
///
/// # Safety
/// `pointer` must be null or point to a NUL-terminated string that stays valid during the call.
unsafe fn text(pointer: *const c_char) -> String {
    if pointer.is_null() {
        return String::new();
    }
    // SAFETY: not null, and the caller promises a NUL-terminated string that's valid for now.
    // The SDK says its strings are UTF-8; anything that isn't is replaced, not trusted.
    unsafe { CStr::from_ptr(pointer) }
        .to_string_lossy()
        .into_owned()
}

/// One of the game's values as the snapshot's kind of value.
///
/// # Safety
/// `value` must be a `scs_value_t` the game filled in: its `kind` says which field of the data
/// is the one that was written (and for a string, points to a valid C string).
unsafe fn convert(value: &ScsValue) -> Value {
    // SAFETY: in every arm, the field read is the one `kind` names, which is the one the game
    // wrote; all of them are plain numbers, valid whatever the bytes, but for the string, which
    // `text` checks for null.
    unsafe {
        match value.kind {
            TYPE_BOOL => Value::Bool(value.data.bool != 0),
            TYPE_S32 => Value::Integer(i64::from(value.data.s32)),
            TYPE_U32 => Value::Integer(i64::from(value.data.u32)),
            TYPE_S64 => Value::Integer(value.data.s64),
            // Amounts of money and the like: far below where this would wrap.
            TYPE_U64 => Value::Integer(value.data.u64 as i64),
            TYPE_FLOAT => Value::Number(f64::from(value.data.float)),
            TYPE_DOUBLE => Value::Number(value.data.double),
            TYPE_STRING => Value::Text(text(value.data.string)),
            TYPE_DPLACEMENT => {
                let placement = value.data.placement;
                Value::Placement {
                    x: placement.position[0],
                    y: placement.position[1],
                    z: placement.position[2],
                    heading: placement.orientation[0],
                }
            }
            _ => Value::Other,
        }
    }
}

/// The attributes of a configuration or an event. Values in a list (a wheel's, a trailer's) are
/// left out: the snapshot uses none.
///
/// # Safety
/// `attributes` must be null or point to `scs_named_value_t`s ending with one whose name is
/// null, all valid during the call.
unsafe fn attributes(attributes: *const ScsNamedValue) -> Vec<Attribute> {
    let mut found = Vec::new();
    if attributes.is_null() {
        return found;
    }
    let mut current = attributes;
    // A limit no real list comes near, in case the end marker is ever missing.
    for _ in 0..4096 {
        // SAFETY: `current` is inside the list: it started at its first entry and only moves
        // one entry at a time, stopping at the end marker (a null name) below.
        let entry = unsafe { &*current };
        if entry.name.is_null() {
            break;
        }
        if entry.index == NO_INDEX {
            found.push(Attribute {
                // SAFETY: not null (checked), and the SDK gives valid C strings.
                name: unsafe { text(entry.name) },
                // SAFETY: a value the game filled in.
                value: unsafe { convert(&entry.value) },
            });
        }
        // SAFETY: this entry wasn't the end marker, so the next one exists.
        current = unsafe { current.add(1) };
    }
    found
}

/// Called by the game with one channel's new value. `context` is the channel's place in
/// `Channel::ALL`, as given when registering.
unsafe extern "system" fn on_channel(
    _name: *const c_char,
    _index: u32,
    value: *const ScsValue,
    context: *mut c_void,
) {
    let Some(channel) = Channel::ALL.get(context as usize).copied() else {
        return;
    };
    if value.is_null() {
        return;
    }
    // SAFETY: not null, and the game passes a `scs_value_t` it filled in, valid during the call.
    let value = unsafe { convert(&*value) };
    update(|snapshot| snapshot.set(channel, &value));
}

/// Called by the game for the events registered in `scs_telemetry_init`.
unsafe extern "system" fn on_event(event: u32, info: *const c_void, _context: *mut c_void) {
    match event {
        EVENT_PAUSED => update(|snapshot| snapshot.is_paused = true),
        EVENT_STARTED => update(|snapshot| snapshot.is_paused = false),
        EVENT_CONFIGURATION | EVENT_GAMEPLAY => {
            if info.is_null() {
                return;
            }
            // SAFETY: for these two events the SDK passes a `scs_telemetry_configuration_t` or a
            // `scs_telemetry_gameplay_event_t`, which have the same shape (`ScsNamedGroup`),
            // valid during the call.
            let group = unsafe { &*info.cast::<ScsNamedGroup>() };
            // SAFETY: the SDK gives a valid C string and a list ending in a null name.
            let (id, attributes) = unsafe { (text(group.id), attributes(group.attributes)) };
            update(|snapshot| {
                if event == EVENT_CONFIGURATION {
                    snapshot.apply_configuration(&id, &attributes);
                } else {
                    snapshot.apply_event(&id, &attributes);
                }
            });
        }
        _ => {}
    }
}

/// The game calls this when it loads the plugin, with the newest SDK version it knows first.
///
/// # Safety
/// Called by the game only, as its SDK describes: `params` points to a
/// `scs_telemetry_init_params_v100_t` for versions 1.00 and 1.01.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn scs_telemetry_init(version: u32, params: *const c_void) -> i32 {
    if (version != TELEMETRY_VERSION_1_01 && version != TELEMETRY_VERSION_1_00) || params.is_null()
    {
        // The game then tries the next older version it knows.
        return RESULT_UNSUPPORTED;
    }
    // SAFETY: not null, and for these versions the SDK passes this structure (its parent type
    // has no fields, so it starts at the same address), valid during the call.
    let params = unsafe { &*params.cast::<InitParams>() };
    let (Some(register_for_event), Some(register_for_channel)) =
        (params.register_for_event, params.register_for_channel)
    else {
        return RESULT_UNSUPPORTED;
    };

    // SAFETY: the SDK says `game_id` is never null and a valid C string.
    let game = unsafe { text(params.game_id) };
    update(|snapshot| {
        *snapshot = Snapshot {
            plugin_version: live::PLUGIN_VERSION,
            game,
            // "At the time this function is called, the telemetry is in the paused state."
            is_paused: true,
            ..Snapshot::default()
        }
    });

    for event in [
        EVENT_PAUSED,
        EVENT_STARTED,
        EVENT_CONFIGURATION,
        EVENT_GAMEPLAY,
    ] {
        // SAFETY: the game's own function, called during init as the SDK allows, with a
        // callback of the type it expects. A refusal (an older game without gameplay events)
        // only means that event never comes.
        let _ = unsafe { register_for_event(event, on_event, std::ptr::null_mut()) };
    }
    for (place, channel) in Channel::ALL.iter().enumerate() {
        let (name, kind) = channel.name_and_kind();
        let kind = match kind {
            Kind::Bool => TYPE_BOOL,
            Kind::S32 => TYPE_S32,
            Kind::U32 => TYPE_U32,
            Kind::Float => TYPE_FLOAT,
            Kind::Placement => TYPE_DPLACEMENT,
        };
        // Our own names: none holds a NUL byte, so this never skips one.
        let Ok(name) = CString::new(name) else {
            continue;
        };
        // SAFETY: the game's own function, called during init as the SDK allows; `name` is a
        // valid C string for the call (the game keeps its own copy). The context isn't a real
        // pointer, only the channel's place in `Channel::ALL`, read back in `on_channel`. A
        // channel this game doesn't have is refused, and then just never comes.
        let _ = unsafe {
            register_for_channel(
                name.as_ptr(),
                NO_INDEX,
                kind,
                0,
                on_channel,
                place as *mut c_void,
            )
        };
    }

    // The thread that sends the snapshot to the editor. If the port is taken (the other game is
    // running with the plugin too), this game simply isn't heard.
    // An earlier server first, should the game ever initialise twice without shutting down in
    // between: it holds the port the new one needs.
    let earlier = SERVER.lock().unwrap_or_else(PoisonError::into_inner).take();
    drop(earlier);
    let server = live::serve(SNAPSHOT.clone());
    let message = match &server {
        Ok(server) => format!(
            "[tse_live] Sending live data to the editor on 127.0.0.1:{} (this computer only, read-only).",
            server.port()
        ),
        Err(error) => format!("[tse_live] Could not start sending live data: {error}"),
    };
    *SERVER.lock().unwrap_or_else(PoisonError::into_inner) = server.ok();
    if let (Some(log), Ok(message)) = (params.log, CString::new(message)) {
        // SAFETY: the game's own log function, with a valid C string.
        unsafe { log(LOG_MESSAGE, message.as_ptr()) };
    }
    RESULT_OK
}

/// The game calls this before it unloads the plugin.
///
/// # Safety
/// Called by the game only, after a successful `scs_telemetry_init`.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn scs_telemetry_shutdown() {
    // Dropping the server stops its thread and waits for it, so no code of ours is running when
    // the game unloads the library.
    let server = SERVER.lock().unwrap_or_else(PoisonError::into_inner).take();
    drop(server);
    update(|snapshot| *snapshot = Snapshot::default());
}
