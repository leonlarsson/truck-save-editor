//! Paint: a truck's or trailer's paint job and its colours, and the colours the player saved in
//! the game's paint screen.
//!
//! A vehicle's paint is one of its accessories, a `vehicle_paint_job_accessory`: `data_path`
//! names the paint job (`/def/vehicle/truck/<model>/paint_job/<name>.sii`, see the `gamedata`
//! crate), and `base_color`, `mask_r_color`, `mask_g_color` and `mask_b_color` are its colours
//! (plus `flake_color` and `flip_color` for metallic ones). Colours are three numbers from 0 to 1,
//! in linear light: `(0.5, 0.2, 0.1)`.
//!
//! The saved colours are `economy.user_colors[]`, 32 slots, each a 32-bit number `0xAABBGGRR`
//! in display (sRGB) values; 0 is an empty slot. (Worked out from a save whose saved colours match
//! colours on its vehicles exactly once converted.)

use sii::{Document, Value, value};

/// A colour as the save keeps paint: red, green and blue from 0 to 1, in linear light.
pub type LinearRgb = [f32; 3];

/// A colour as shown on screen and in the saved colours: red, green and blue bytes (sRGB).
pub type Rgb8 = [u8; 3];

/// How many saved colours the game keeps.
pub const SAVED_COLOUR_SLOTS: usize = 32;

/// Which of a paint job's colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PaintColour {
    Base,
    /// The paint job's first, second and third colour (the red, green and blue of its mask).
    MaskR,
    MaskG,
    MaskB,
}

impl PaintColour {
    pub const ALL: [PaintColour; 4] = [
        PaintColour::Base,
        PaintColour::MaskR,
        PaintColour::MaskG,
        PaintColour::MaskB,
    ];

    pub fn field(self) -> &'static str {
        match self {
            PaintColour::Base => "base_color",
            PaintColour::MaskR => "mask_r_color",
            PaintColour::MaskG => "mask_g_color",
            PaintColour::MaskB => "mask_b_color",
        }
    }
}

/// A vehicle's paint as the save has it.
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    /// The paint accessory's unit id.
    pub accessory: String,
    /// The paint job it is (`/def/vehicle/truck/volvo.fh16/paint_job/color0.sii`).
    pub data_path: String,
    pub base: Option<LinearRgb>,
    /// The first, second and third colour.
    pub masks: [Option<LinearRgb>; 3],
}

impl Paint {
    pub fn colour(&self, which: PaintColour) -> Option<LinearRgb> {
        match which {
            PaintColour::Base => self.base,
            PaintColour::MaskR => self.masks[0],
            PaintColour::MaskG => self.masks[1],
            PaintColour::MaskB => self.masks[2],
        }
    }
}

/// Reads `(r, g, b)`; numbers may be the game's `&hex` floats.
pub fn parse_colour(text: &str) -> Option<LinearRgb> {
    let inside = text.trim().strip_prefix('(')?.strip_suffix(')')?;
    let mut numbers = inside.split(',').map(value::parse_f32);
    let colour = [numbers.next()??, numbers.next()??, numbers.next()??];
    numbers.next().is_none().then_some(colour)
}

/// Writes a colour as the save does: `(r, g, b)`.
pub fn format_colour(colour: LinearRgb) -> String {
    let [r, g, b] = colour.map(|part| value::format_f32(part.clamp(0.0, 1.0)));
    format!("({r}, {g}, {b})")
}

/// A linear colour as shown on screen (sRGB bytes).
pub fn to_display(colour: LinearRgb) -> Rgb8 {
    colour.map(|part| {
        let part = part.clamp(0.0, 1.0);
        let shown = if part <= 0.003_130_8 {
            part * 12.92
        } else {
            1.055 * part.powf(1.0 / 2.4) - 0.055
        };
        (shown * 255.0).round() as u8
    })
}

/// A colour picked on screen (sRGB bytes) as the save keeps paint.
pub fn to_linear(colour: Rgb8) -> LinearRgb {
    colour.map(|byte| {
        let shown = f32::from(byte) / 255.0;
        if shown <= 0.040_45 {
            shown / 12.92
        } else {
            ((shown + 0.055) / 1.055).powf(2.4)
        }
    })
}

/// A vehicle's paint, if it has a paint accessory.
pub fn vehicle_paint(document: &Document, vehicle: &str) -> Option<Paint> {
    let unit = document.unit(vehicle)?;
    unit.items("accessories").iter().find_map(|id| {
        let accessory = document.unit(id)?;
        if accessory.kind != "vehicle_paint_job_accessory" {
            return None;
        }
        let colour = |field: &str| accessory.single(field).and_then(parse_colour);
        Some(Paint {
            accessory: id.clone(),
            data_path: accessory
                .single("data_path")
                .map(|path| value::parse_string(path).to_string())
                .unwrap_or_default(),
            base: colour("base_color"),
            masks: [
                colour("mask_r_color"),
                colour("mask_g_color"),
                colour("mask_b_color"),
            ],
        })
    })
}

/// The model folder a paint job is in (`/def/vehicle/truck/volvo.fh16/paint_job/style1.sii` →
/// `volvo.fh16`): where the vehicle's other paint jobs are.
pub fn paint_model(data_path: &str) -> Option<&str> {
    let rest = data_path.strip_prefix("/def/vehicle/")?;
    let mut parts = rest.split('/');
    let _kind = parts.next()?;
    let model = parts.next()?;
    (parts.next()? == "paint_job").then_some(model)
}

/// A vehicle's parts as paint jobs name the ones they fit (`suitable_for`): a part defined in
/// `/def/vehicle/truck/volvo.fh16/cabin/l2h2.sii` is `l2h2.volvo.fh16.cabin`.
pub fn vehicle_part_ids(document: &Document, vehicle: &str) -> Vec<String> {
    let Some(unit) = document.unit(vehicle) else {
        return Vec::new();
    };
    unit.items("accessories")
        .iter()
        .filter_map(|id| {
            let path = document.unit(id)?.single("data_path")?;
            let path = value::parse_string(path);
            let mut parts = path.strip_prefix("/def/vehicle/")?.split('/');
            let (_kind, model, folder, file) =
                (parts.next()?, parts.next()?, parts.next()?, parts.next()?);
            let stem = file.strip_suffix(".sii")?;
            Some(format!("{stem}.{model}.{folder}"))
        })
        .collect()
}

/// A paint job to put on a vehicle, with the colours it comes in (each `None` keeps the colour
/// the vehicle has).
#[derive(Clone, Debug, PartialEq)]
pub struct NewPaintJob {
    pub data_path: String,
    /// What it cost, refunded when the vehicle is sold (as the game keeps it).
    pub price: i64,
    pub base: Option<LinearRgb>,
    pub masks: [Option<LinearRgb>; 3],
    pub flake: Option<LinearRgb>,
    pub flip: Option<LinearRgb>,
}

/// Sets one of a vehicle's paint colours. False without a paint accessory.
pub(crate) fn set_paint_colour(
    document: &mut Document,
    vehicle: &str,
    which: PaintColour,
    colour: LinearRgb,
) -> bool {
    let Some(paint) = vehicle_paint(document, vehicle) else {
        return false;
    };
    let Some(mut accessory) = document.unit_mut(&paint.accessory) else {
        return false;
    };
    accessory.set(which.field(), Value::single(format_colour(colour)));
    true
}

/// Puts another paint job on a vehicle, in its own colours. False without a paint accessory.
pub(crate) fn set_paint_job(document: &mut Document, vehicle: &str, job: &NewPaintJob) -> bool {
    let Some(paint) = vehicle_paint(document, vehicle) else {
        return false;
    };
    let Some(mut accessory) = document.unit_mut(&paint.accessory) else {
        return false;
    };
    let clean: String = job.data_path.chars().filter(|c| *c != '"').collect();
    accessory.set("data_path", Value::single(format!("\"{clean}\"")));
    accessory.set("refund", Value::single(job.price.max(0).to_string()));
    let colours = [
        ("base_color", job.base),
        ("mask_r_color", job.masks[0]),
        ("mask_g_color", job.masks[1]),
        ("mask_b_color", job.masks[2]),
        ("flake_color", job.flake),
        ("flip_color", job.flip),
    ];
    for (field, colour) in colours {
        if let Some(colour) = colour {
            accessory.set(field, Value::single(format_colour(colour)));
        }
    }
    true
}

/// The saved colours, slot by slot (`None` for an empty one). Always `SAVED_COLOUR_SLOTS` long.
pub fn saved_colours(document: &Document) -> Vec<Option<Rgb8>> {
    let items = document
        .first_of_kind("economy")
        .map(|economy| economy.items("user_colors").to_vec())
        .unwrap_or_default();
    (0..SAVED_COLOUR_SLOTS)
        .map(|slot| {
            let packed: u32 = items.get(slot)?.parse().ok()?;
            // An empty slot is 0: no alpha.
            (packed >> 24 != 0).then_some([
                (packed & 0xff) as u8,
                ((packed >> 8) & 0xff) as u8,
                ((packed >> 16) & 0xff) as u8,
            ])
        })
        .collect()
}

/// Sets the saved colours, slot by slot (more than `SAVED_COLOUR_SLOTS` are left out). False when
/// the save has no economy.
pub fn set_saved_colours(document: &mut Document, colours: &[Option<Rgb8>]) -> bool {
    let Some(id) = document
        .first_of_kind("economy")
        .map(|economy| economy.id.clone())
    else {
        return false;
    };
    let Some(mut economy) = document.unit_mut(&id) else {
        return false;
    };
    let items: Vec<String> = (0..SAVED_COLOUR_SLOTS)
        .map(|slot| match colours.get(slot).copied().flatten() {
            Some([r, g, b]) => {
                let packed =
                    0xff00_0000 | (u32::from(b) << 16) | (u32::from(g) << 8) | u32::from(r);
                packed.to_string()
            }
            None => "0".to_string(),
        })
        .collect();
    economy.set("user_colors", Value::Array(items));
    true
}

/// The saved colours as text to share: `#RRGGBB` for each, `-` for an empty slot, separated by
/// spaces; empty slots at the end are left out.
pub fn colours_to_text(colours: &[Option<Rgb8>]) -> String {
    let last = colours
        .iter()
        .rposition(Option::is_some)
        .map_or(0, |at| at + 1);
    colours[..last]
        .iter()
        .map(|colour| match colour {
            Some([r, g, b]) => format!("#{r:02X}{g:02X}{b:02X}"),
            None => "-".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Reads shared colours: `#RRGGBB` (or `RRGGBB`) and `-` for empty, separated by spaces, commas
/// or new lines. `None` when anything else is in it, or there are more than the game keeps.
pub fn colours_from_text(text: &str) -> Option<Vec<Option<Rgb8>>> {
    let colours: Option<Vec<Option<Rgb8>>> = text
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|word| !word.is_empty())
        .map(|word| {
            if word == "-" {
                return Some(None);
            }
            let hex = word.strip_prefix('#').unwrap_or(word);
            if hex.len() != 6 {
                return None;
            }
            let byte = |at: usize| u8::from_str_radix(hex.get(at..at + 2)?, 16).ok();
            Some(Some([byte(0)?, byte(2)?, byte(4)?]))
        })
        .collect();
    colours.filter(|colours| !colours.is_empty() && colours.len() <= SAVED_COLOUR_SLOTS)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAVE: &str = "SiiNunit\r\n{\r\neconomy : e {\r\n user_colors: 3\r\n user_colors[0]: 4288651167\r\n user_colors[1]: 0\r\n user_colors[2]: 4278255584\r\n}\r\n\r\nvehicle : truck.1 {\r\n accessories: 2\r\n accessories[0]: acc.engine\r\n accessories[1]: acc.paint\r\n}\r\n\r\nvehicle_engine_accessory : acc.engine {\r\n data_path: \"/def/vehicle/truck/volvo.fh16/engine/d16.sii\"\r\n}\r\n\r\nvehicle_paint_job_accessory : acc.paint {\r\n mask_r_color: (1, 0, 0)\r\n mask_g_color: (0, 1, 0)\r\n mask_b_color: (0, 0, 1)\r\n flake_color: (1, 1, 1)\r\n flip_color: (1, 1, 1)\r\n base_color: (&3f000000, 0.25, 0)\r\n data_path: \"/def/vehicle/truck/volvo.fh16/paint_job/color0.sii\"\r\n refund: 0\r\n}\r\n\r\n}\r\n";

    #[test]
    fn reads_a_vehicles_paint() {
        let document = sii::parse_text(SAVE).expect("parses");
        let paint = vehicle_paint(&document, "truck.1").expect("paint");
        assert_eq!(paint.accessory, "acc.paint");
        assert_eq!(
            paint.data_path,
            "/def/vehicle/truck/volvo.fh16/paint_job/color0.sii"
        );
        assert_eq!(paint.base, Some([0.5, 0.25, 0.0]));
        assert_eq!(paint.colour(PaintColour::MaskB), Some([0.0, 0.0, 1.0]));
    }

    #[test]
    fn sets_a_colour_and_a_paint_job() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        assert!(set_paint_colour(
            &mut document,
            "truck.1",
            PaintColour::MaskG,
            [0.1, 0.2, 0.3]
        ));
        let paint = vehicle_paint(&document, "truck.1").expect("paint");
        assert_eq!(paint.masks[1], Some([0.1, 0.2, 0.3]));

        let job = NewPaintJob {
            data_path: "/def/vehicle/truck/volvo.fh16/paint_job/style1.sii".into(),
            price: 3000,
            base: Some([1.0, 1.0, 1.0]),
            masks: [None, None, None],
            flake: None,
            flip: None,
        };
        assert!(set_paint_job(&mut document, "truck.1", &job));
        let paint = vehicle_paint(&document, "truck.1").expect("paint");
        assert_eq!(paint.data_path, job.data_path);
        assert_eq!(paint.base, Some([1.0, 1.0, 1.0]));
        // Colours the paint job doesn't name are kept.
        assert_eq!(paint.masks[1], Some([0.1, 0.2, 0.3]));
        let accessory = document.unit("acc.paint").expect("accessory");
        assert_eq!(accessory.single("refund"), Some("3000"));
    }

    #[test]
    fn names_the_parts_as_paint_jobs_do() {
        let document = sii::parse_text(SAVE).expect("parses");
        assert_eq!(
            vehicle_part_ids(&document, "truck.1"),
            ["d16.volvo.fh16.engine", "color0.volvo.fh16.paint_job"]
        );
        assert_eq!(
            paint_model("/def/vehicle/trailer_owned/scs.box/paint_job/x.sii"),
            Some("scs.box")
        );
        assert_eq!(
            paint_model("/def/vehicle/truck/volvo.fh16/engine/d16.sii"),
            None
        );
    }

    #[test]
    fn display_colours_round_trip() {
        for byte in [0u8, 1, 10, 100, 128, 200, 255] {
            let shown = [byte, byte / 2, 255 - byte];
            assert_eq!(to_display(to_linear(shown)), shown);
        }
    }

    #[test]
    fn reads_and_writes_saved_colours() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        let colours = saved_colours(&document);
        assert_eq!(colours.len(), SAVED_COLOUR_SLOTS);
        // 0xFF9F9F9F, empty, 0xFF00FFE0 (blue 0x00, green 0xFF, red 0xE0).
        assert_eq!(
            &colours[..3],
            [Some([0x9f, 0x9f, 0x9f]), None, Some([0xe0, 0xff, 0x00])]
        );

        let mut changed = colours.clone();
        changed[1] = Some([0x12, 0x34, 0x56]);
        assert!(set_saved_colours(&mut document, &changed));
        assert_eq!(saved_colours(&document), changed);
        let economy = document.first_of_kind("economy").expect("economy");
        assert_eq!(economy.items("user_colors").len(), SAVED_COLOUR_SLOTS);
        assert_eq!(economy.items("user_colors")[2], "4278255584");
    }

    #[test]
    fn shares_colours_as_text() {
        let colours = vec![
            Some([0x9f, 0x9f, 0x9f]),
            None,
            Some([0xe0, 0xff, 0x00]),
            None,
        ];
        let text = colours_to_text(&colours);
        assert_eq!(text, "#9F9F9F - #E0FF00");
        assert_eq!(
            colours_from_text(&text),
            Some(vec![
                Some([0x9f, 0x9f, 0x9f]),
                None,
                Some([0xe0, 0xff, 0x00])
            ])
        );
        assert_eq!(colours_from_text("#12345, -"), None);
        assert_eq!(colours_from_text("nothing"), None);
        assert_eq!(colours_from_text(""), None);
    }
}
