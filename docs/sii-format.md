# SCS save file formats

**SII** is SCS Software's own definition format (used by both Euro Truck Simulator 2 and American Truck Simulator). A SII file is a list of **units**: typed records with an id and named fields. The game keeps its config, its data and your saves in it. Saves get extra layers on top (encryption, a binary form), described below.

Confirmed on real saves from game version 1.62.x (save `version: 105`, binary version 3) and from 2020 (binary version 2). What changes between game versions, and how this editor copes, is in `compatibility.md`.

## Where saves are

- **Steam Cloud on:** `…\Steam\userdata\<steam id>\227300\remote\profiles\<profile>\` (227300 is ETS2's Steam app id).
- **Steam Cloud off:** `%USERPROFILE%\Documents\Euro Truck Simulator 2\profiles\<profile>\`. Folders named `steam_profiles(<version>).bak` there are the game's own backups.
- The Steam folder can be found through the registry or `libraryfolders.vdf`.
- The profile folder name is the profile name in hex (`4A6F686E` = "John").
- Each profile has `profile.sii` (profile metadata; **it holds the World of Trucks login and an encrypted password, so treat it as secret**) and `save/<slot>/` folders.
- Slots: `1`, `2`, …, `autosave`, `autosave_drive[_N]`, `autosave_job[_N]`, `quicksave`, `multiplayer_backup`.
- A slot holds `game.sii` (the save itself), `info.sii` (the summary the load menu shows) and preview images.

## Layers

A `.sii` file comes in one of three forms. Tell them apart by the first 4 bytes:

| Magic | Meaning |
|---|---|
| `ScsC` | Encrypted and compressed wrapper; unwraps to one of the two below |
| `BSII` | Binary SII (`game.sii` unwraps to this, version 3) |
| `SiiN` (`SiiNunit`) | Plain-text SII (`info.sii` and `profile.sii` unwrap to this) |

## `ScsC` wrapper

```
offset size
0      4    "ScsC"
4      32   HMAC-SHA256 (not checked by the game as far as we know; ignored)
36     16   AES IV
52     4    u32 LE: size after decompressing
56     …    AES-256-CBC ciphertext (PKCS7 padding) → zlib stream (with header) → payload
```

The AES-256 key is public and widely documented (for example in the SII_Decrypt project):
`2a5fcb1791d22fb60245b3d8369ed0b2c27371563fbf1f3c9edf6b11825a5d0a`

Rust crates: `aes` + `cbc`, `flate2` (with its pure-Rust backend). All build for wasm32.

## Binary SII (`BSII`)

All values are little-endian.

```
"BSII" u32 version
loop:
  u32 block_type
  if block_type == 0:                       structure definition
      u8 valid; if valid == 0 → end of file
      u32 struct_id; string name
      loop: u32 value_type; if 0 → break
            string field_name
            if value_type == 0x37: u32 n; n × (u32 index, string)    lookup table for this field
  else:                                     data block for structure `block_type`
      id unit_id; then one value per field, in the order they were defined
string = u32 length + UTF-8 bytes
```

### Ids

`u8 n`, then:

- `n == 0xFF`: one `u64`, written as `_nameless.` plus 16-bit hex groups with leading zero groups dropped (for example `_nameless.1ec.ab8c.2808`).
- `n == 0`: `null`.
- otherwise: `n` encoded tokens (u64 each), joined with `.`.

### Encoded tokens (u64 → string)

Repeat `v, i = divmod(v, 38); char = CHARS[i - 1]` while `v > 0`, with `CHARS = "0123456789abcdefghijklmnopqrstuvwxyz_"` (at most 12 characters).

### Value types

Arrays are `u32 count` followed by the items. An array's type is the single type + 1.

| Type | Meaning | Type | Meaning |
|---|---|---|---|
| 0x01 / 0x02 | string | 0x25 / 0x26 | i32 |
| 0x03 / 0x04 | encoded token (u64) | 0x27 / 0x28 | u32 |
| 0x05 / 0x06 | f32 | 0x29 / 0x2A | i16 |
| 0x07 / 0x08 | vec2 f32 | 0x2B / 0x2C | u16 |
| 0x09 / 0x0A | vec3 f32 | 0x2F | u32 (another variant) |
| 0x11 / 0x12 | vec3 i32 | 0x31 / 0x32 | i64 |
| 0x17 / 0x18 | vec4 f32 (quaternion) | 0x33 / 0x34 | u64 |
| 0x19 / 0x1A | vec8 f32 (placement) | 0x35 / 0x36 | bool (u8) |
| 0x37 | lookup value: u32 index into the field's table | 0x39–0x3E | id (odd = single, even = array) |

**Placement (0x19):** in BSII version 1, 7 × f32 `[x, y, z, qw, qx, qy, qz]`. In version 2 and later, 8 × f32 `[x, y, z, bias, qw, qx, qy, qz]`, then
`b = bias as i32; x += ((b & 0xFFF) - 2048) << 9; z += (((b >> 12) & 0xFFF) - 2048) << 9`.
The neutral bias is `8390656` (2048 in both parts). The text form is `(x, y, z) (qw, qx, qy, qz)`; the bias isn't written.
(Confirmed on a real version 2 save, whose placements carry other biases.)

## Text SII

```
SiiNunit
{
unit_type : unit_id {
 field: value
 array_field: 3
 array_field[0]: value
 …
}
}
```

- Strings are written bare when they only contain token characters, otherwise in quotes.
- The game writes some floats as `&3e98ae14` (the raw IEEE-754 bits in hex). The parser must accept both that and decimals.

## Writing saves back

The plan is to write `game.sii` as **plain text SII**, with no re-encryption or binary encoding (see `decisions.md`).

- **Not yet checked:** that game 1.62 loads a plain-text `game.sii` in place of the encrypted binary one, and accepts decimal floats. It's widely reported to work (guides and tools from 2026), never officially confirmed. The game's own setting `g_save_format` (in `config.cfg`) chooses what it writes; sources disagree on the values (most say 2 = text). Check by decoding a save, writing it back unchanged into a backed-up slot and loading it in game.
- **Not yet checked:** whether `info.sii` needs updating to match, or whether the game recomputes it. Until known, update the fields it mirrors.

## What's in a save

A large save has tens of thousands of units. The biggest kinds: `economy_event`, `job_offer_data`, `vehicle_addon_accessory`, `line_offer_data`, `profit_log_entry`, `vehicle_wheel_accessory`, `company`, `vehicle_accessory`, `delivery_log_entry`, `garage`, `vehicle`, `driver_ai`, `bus_stop`.

The units to edit first (each points to others by id):

- **`economy`**: the root of the player's state.
  - `experience_points`, `bus_experience_points`
  - skills: `adr` (bitmask of 6 classes, 63 = all), `long_dist`, `heavy`, `fragile`, `urgent`, `mechanical` (each 0–6)
  - `visited_cities`, `unlocked_dealers`, `unlocked_recruitments`
  - `game_time` (in minutes)
  - `bank`, `player`: ids of those units
- **`bank`**: `money_account`, `loans`, `overdraft`.
- **`player`**: `hq_city`, `trucks[]`, `drivers[]`, `trailers[]`, `current_job`, `my_vehicles_mode` (truck/bus).
- **`garage`**: `status` (size: tiny/small/large …), `vehicles[]`, `drivers[]`.
- **`vehicle`**: `odometer`, wear and damage fields, `accessories[]` (the `vehicle_*_accessory` units), license plate.
- **`driver_ai`**: hired drivers' skills and XP.
- **`info.sii`**: `info_money_account`, `info_players_experience` (= economy XP + bus XP), and more.

### Skill names as the game shows them

| Field | In game |
|---|---|
| `adr` | ADR (6 classes: explosives, gases, flammable liquids, flammable solids, toxic, corrosive) |
| `long_dist` | Long distance |
| `heavy` | High value cargo |
| `fragile` | Fragile cargo |
| `urgent` | Just-in-time delivery |
| `mechanical` | Ecodriving |

Check these labels and the ADR bit order against the game before shipping them.

## Dealers and recruitment agencies

The save lists the cities where the player has found a truck dealer (`economy.unlocked_dealers[]`) and a recruitment agency (`economy.unlocked_recruitments[]`), by city token. Which cities have them isn't in the save, and the game has no plain table of either:

- **Dealers:** a city has one when its definition lists brands: `vehicle_brands[]` in `city_data : city.<token>`, in `def/city/<token>.sui`. `def/city.sii` (in `def.scs`) includes the base game's cities, and each map DLC adds a `def/city.<dlc>.sii` in its own archive. In 1.62 that's 146 cities, and it matches the dealer prefabs placed on the map (two cities list brands without a dealer on the map: `edirne`, `veli_tarnovo`).
- **Recruitment agencies:** only the map knows. Prefab models (`def/world/prefab*.sii`) point to `.ppd` descriptors whose spawn points have types; type 16 is a recruitment agency (type 7 a truck dealer). Finding where those prefabs sit in the map sectors (`map/europe/sec*.base`) and which city area each falls in gives 138 cities in 1.62. The map format changes between versions, so the editor ships that list (`crates/gamedata/src/recruitment.rs`) and keeps only the cities the installed game defines; it needs reading again when a map DLC adds cities.

Saves can hold tokens for cities that no longer have one (a leftover from an older map); the editor keeps them and doesn't count them.

## Trailers and fuel

- Owned trailers are `player.trailers[]`, each a `trailer` unit: `trailer_definition` (a `trailer_def` with `body_type`, `axles`, `chain_type`), `license_plate`, `odometer`, and wear in `trailer_body_wear`, `chassis_wear` and `wheels_wear` (each with an `_unfixable` part the garage can't repair). A double or B-double links its next trailer through `slave_trailer` (`null` at the end); `player.trailers[]` lists only the first. A garage lists the trailers kept there in `trailers[]`.
- The trailer's model is the accessory whose `data_path` is `/def/vehicle/trailer_owned/<brand.model>/data.sii`. SCS's own trailers have no brand text and a terse series (`STD`), so the editor names them by body type (the text key is the `body_type` token: `refrigerated` → "Refrigerated").
- A truck's or coach's fuel is `fuel_relative`, from 0 to 1 of its tank, whatever the tank's size.

## Where vehicles are, and drivers' faces

- Trucks and trailers live in garages (`garage.vehicles[]`, `garage.trailers[]`). Coaches have no garage. The player has one own vehicle slot per mode in `player.my_vehicles[]` (`player_vehicles` units: `vehicle`, `stored_vehicle_city`, where it's parked while another mode is in use), and `player.assigned_vehicles` is the one being driven now.
- Hired drivers are `driver_ai : driver.<n>` units with no name field. Their face is the game's `material/ui/driver/<n>.mat` (a plain 256×256 picture). The game reads names from `/locale/<lang>/driver_names.sii` when there is one; 1.62 ships none.

## Jobs

Each company in each city is `company : company.volatile.<company>.<city>`, with its offers in `job_offer[]`: `job_offer_data` units with `target` (`"<company>.<city>"`), `cargo` (`cargo.<token>`; `null` for an empty slot), `shortest_distance_km`, `urgency` (0 standard, 1 important, 2 urgent), `ferry_time` and `ferry_price`, `units_count`, `trailer_definition`, and `expiration_time` in game minutes, against `economy.game_time`. Names come from the game: `def/company/<token>.sui` (`company_permanent`, a plain `name`) and `def/cargo/<token>.sui` (`cargo_data`, `name` a text key), plus the `company.*.sii` and `cargo.*.sii` each DLC adds.
