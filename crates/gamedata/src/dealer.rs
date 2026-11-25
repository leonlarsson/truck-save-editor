//! The trucks the game's dealers sell ready-made (their stock offers): one file each under
//! `/def/vehicle/truck_dealer/<brand>/` (coaches: `/def/vehicle/bus_dealer/<brand>/`, Coaches
//! DLC) in the base game's definitions or a DLC's (ATS keeps each
//! truck brand's in its own archive). A file is a `vehicle` unit listing its accessories, then the
//! accessories, each pointing at a part's definition (`data_path`): written almost as a save keeps
//! a truck, so buying one copies its accessories into the save.
//!
//! Trailers likewise, under `/def/vehicle/trailer_dealer/<brand>/`: a file holds one `trailer`
//! unit, or a road train's chain of them (each pointing at the next as its `slave_trailer`), with
//! their accessories. Each trailer names its type (`trailer_definition:
//! trailer_def.scs.box.single_53_3.reefer`), defined in `/def/vehicle/trailer_defs/`.
//!
//! What a save adds to each accessory is `refund`, the part's price from its definition (`price`,
//! 0 where it has none): checked on a truck the game had just sold (ATS, 2026-11-25). The truck's
//! price is its parts' prices together.

use std::collections::HashMap;
use std::path::Path;

use crate::read_document;

/// One truck a dealer sells ready-made.
#[derive(Clone, Debug, PartialEq)]
pub struct DealerTruck {
    /// Its definition (`/def/vehicle/truck_dealer/daf/daf_1.sii`), which also serves as its id.
    pub path: String,
    /// The model (`daf.xf_euro6`), from its `data` accessory's folder.
    pub model: String,
    /// Whether it's a coach (a `bus_dealer` offer, Coaches DLC) rather than a truck.
    pub is_coach: bool,
    /// Its chassis's and engine's names, as text keys or plain text (`4x2`, `@@engine_mx13@@`).
    pub chassis: String,
    pub engine: String,
    /// The accessories as a save keeps them: the definition's units, each with `refund` added.
    /// Their ids are the definition's own (`.daf1.engine`); a save gives them new ones.
    pub accessories: Vec<sii::Unit>,
    /// What it costs: its parts' prices together.
    pub price: i64,
    /// The DLC it comes with, `None` for the base game's (see `crate::archive_dlc`).
    pub dlc: Option<String>,
}

/// One trailer (or road train) a dealer sells ready-made.
#[derive(Clone, Debug, PartialEq)]
pub struct DealerTrailer {
    /// Its definition (`/def/vehicle/trailer_dealer/scs/scs_reefer_53.sii`), also its id.
    pub path: String,
    /// The model (`scs.box`), from the front trailer's `data` accessory's folder.
    pub model: String,
    /// The front trailer's body (`refrigerated`), from its type, and its chassis's name.
    pub body_type: String,
    pub chassis: String,
    /// Every unit of the file in its order: the trailers, the front one first, and their
    /// accessories, each accessory with `refund` added. Ids are the file's own.
    pub units: Vec<sii::Unit>,
    /// The front trailer's type as the game defines it (a `trailer_def` unit, by its name:
    /// `trailer_def.scs.box.single_53_3.reefer`). The trailers behind it have none in a save.
    pub definitions: Vec<sii::Unit>,
    /// How many trailers it is: 1, or a road train's (a dolly, with no body, doesn't count).
    pub trailers: usize,
    pub price: i64,
    pub dlc: Option<String>,
}

/// A part's definition, as far as a dealer truck needs it.
#[derive(Clone, Default)]
struct PartInfo {
    price: i64,
    name: String,
}

/// Every truck the dealers sell, in the base game and its DLCs, by model and price. Read when asked
/// (a few hundred small files, and each part's definition for its price).
pub fn dealer_trucks(install_dir: &Path) -> Vec<DealerTruck> {
    // Parts' definitions can live in another archive than the dealer file (a DLC truck on the base
    // game's wheels), so every archive is opened once and searched in order.
    let mut archives = open_archives(install_dir);
    let mut parts: HashMap<String, PartInfo> = HashMap::new();
    let mut trucks: Vec<DealerTruck> = Vec::new();
    for index in 0..archives.len() {
        // Trucks, and the coach dealer's coaches (Coaches DLC), written the same way.
        let mut files = dealer_files(&mut archives[index].0, "/def/vehicle/truck_dealer");
        files.extend(dealer_files(
            &mut archives[index].0,
            "/def/vehicle/bus_dealer",
        ));
        for file in files {
            if trucks.iter().any(|truck| truck.path == file) {
                continue;
            }
            let Ok(document) = read_document(&mut archives[index].0, &file) else {
                continue;
            };
            let dlc = archives[index].1.clone();
            if let Some(truck) = dealer_truck(&document, file, dlc, &mut archives, &mut parts) {
                trucks.push(truck);
            }
        }
    }
    trucks.sort_by(|a, b| {
        a.model
            .cmp(&b.model)
            .then(a.price.cmp(&b.price))
            .then(a.path.cmp(&b.path))
    });
    trucks
}

/// Every trailer the dealers sell, in the base game and its DLCs, by model and price.
pub fn dealer_trailers(install_dir: &Path) -> Vec<DealerTrailer> {
    let mut archives = open_archives(install_dir);
    let mut parts: HashMap<String, PartInfo> = HashMap::new();
    let mut trailers: Vec<DealerTrailer> = Vec::new();
    for index in 0..archives.len() {
        for file in dealer_files(&mut archives[index].0, "/def/vehicle/trailer_dealer") {
            if trailers.iter().any(|trailer| trailer.path == file) {
                continue;
            }
            let Ok(document) = read_document(&mut archives[index].0, &file) else {
                continue;
            };
            let dlc = archives[index].1.clone();
            if let Some(trailer) = dealer_trailer(&document, file, dlc, &mut archives, &mut parts) {
                trailers.push(trailer);
            }
        }
    }
    trailers.sort_by(|a, b| {
        a.body_type
            .cmp(&b.body_type)
            .then(a.price.cmp(&b.price))
            .then(a.path.cmp(&b.path))
    });
    trailers
}

/// Every archive with definitions, opened once, with the DLC each comes with.
fn open_archives(install_dir: &Path) -> Vec<(hashfs::Archive, Option<String>)> {
    crate::def_archives(install_dir)
        .iter()
        .filter_map(|path| {
            let mut archive = hashfs::Archive::open(path).ok()?;
            let dlc = crate::archive_dlc(&mut archive, path);
            Some((archive, dlc))
        })
        .collect()
}

/// One dealer file's trailer (or road train), its parts priced and its types read. `None` if it
/// isn't one the editor understands.
fn dealer_trailer(
    document: &sii::Document,
    path: String,
    dlc: Option<String>,
    archives: &mut [(hashfs::Archive, Option<String>)],
    parts: &mut HashMap<String, PartInfo>,
) -> Option<DealerTrailer> {
    // The front trailer: the one no other trailer pulls.
    let pulled: Vec<&str> = document
        .units_of_kind("trailer")
        .filter_map(|unit| unit.single("slave_trailer"))
        .collect();
    let front = document
        .units_of_kind("trailer")
        .find(|unit| !pulled.contains(&unit.id.as_str()))?
        .id
        .clone();
    let mut units: Vec<sii::Unit> = Vec::new();
    let mut definitions: Vec<sii::Unit> = Vec::new();
    let (mut model, mut chassis, mut price) = (None, String::new(), 0);
    for unit in document.units() {
        let mut unit = unit.clone();
        if unit.kind == "trailer" {
            // Only the front trailer's type counts: a save writes `null` for the trailers behind
            // it, whatever the dealer file says (the front one's type describes the whole train).
            if unit.id == front
                && let Some(name) = unit.single("trailer_definition").map(str::to_string)
                && name.starts_with("trailer_def.")
            {
                definitions.push(trailer_definition(&name, archives)?);
            }
        } else if let Some(data_path) = unit
            .single("data_path")
            .map(|text| sii::value::parse_string(text).to_string())
        {
            let part = part_info(&data_path, archives, parts);
            if model.is_none()
                && let Some(folder) = data_path
                    .strip_prefix("/def/vehicle/trailer_owned/")
                    .and_then(|rest| rest.strip_suffix("/data.sii"))
            {
                model = Some(folder.to_string());
            }
            if chassis.is_empty() && data_path.contains("/chassis/") {
                chassis = part.name.clone();
            }
            price += part.price;
            unit.fields.push(sii::Field {
                name: "refund".to_string(),
                value: sii::Value::single(part.price.to_string()),
            });
        }
        units.push(unit);
    }
    // The front trailer first, so whoever reads this knows which one the company owns.
    if let Some(position) = units.iter().position(|unit| unit.id == front) {
        let unit = units.remove(position);
        units.insert(0, unit);
    }
    // Trailers with a body: a dolly between two has none.
    let trailers = units
        .iter()
        .filter(|unit| unit.kind == "trailer")
        .filter(|trailer| {
            trailer.items("accessories").iter().any(|id| {
                units.iter().any(|unit| {
                    unit.id == *id
                        && unit
                            .single("data_path")
                            .is_some_and(|path| path.contains("/body/"))
                })
            })
        })
        .count()
        .max(1);
    let body_type = units
        .first()
        .and_then(|unit| unit.single("trailer_definition"))
        .and_then(|name| definitions.iter().find(|definition| definition.id == name))
        .and_then(|definition| definition.single("body_type"))
        .unwrap_or_default()
        .to_string();
    Some(DealerTrailer {
        path,
        model: model?,
        body_type,
        chassis,
        units,
        definitions,
        trailers,
        price,
        dlc,
    })
}

/// A trailer type as the game defines it (`trailer_def.scs.box.single_53_3.reefer`, in
/// `/def/vehicle/trailer_defs/scs.box.single_53_3.reefer.sii`).
fn trailer_definition(
    name: &str,
    archives: &mut [(hashfs::Archive, Option<String>)],
) -> Option<sii::Unit> {
    let file = format!(
        "/def/vehicle/trailer_defs/{}.sii",
        name.strip_prefix("trailer_def.")?
    );
    archives
        .iter_mut()
        .find_map(|(archive, _)| read_document(archive, &file).ok())?
        .unit(name)
        .cloned()
}

/// The dealer files under `root` (`/def/vehicle/truck_dealer`) in one archive:
/// `<root>/<brand>/<offer>.sii`. (ETS2's `truck_dealer_uk`, the right-hand-drive versions, are
/// another root, left out: the same trucks again.)
fn dealer_files(archive: &mut hashfs::Archive, root: &str) -> Vec<String> {
    let Ok(Some(brands)) = archive.list(root) else {
        return Vec::new();
    };
    let mut files = Vec::new();
    for brand in brands.directories {
        let folder = format!("{root}/{brand}");
        if let Ok(Some(listing)) = archive.list(&folder) {
            files.extend(
                listing
                    .files
                    .iter()
                    .filter(|name| name.ends_with(".sii"))
                    .map(|name| format!("{folder}/{name}")),
            );
        }
    }
    files
}

/// One dealer file's truck, its parts priced. `None` if it isn't a truck the editor understands.
fn dealer_truck(
    document: &sii::Document,
    path: String,
    dlc: Option<String>,
    archives: &mut [(hashfs::Archive, Option<String>)],
    parts: &mut HashMap<String, PartInfo>,
) -> Option<DealerTruck> {
    let vehicle = document.first_of_kind("vehicle")?;
    let mut accessories = Vec::new();
    let (mut model, mut chassis, mut engine) = (None, String::new(), String::new());
    let mut price = 0;
    for id in vehicle.items("accessories") {
        let mut unit = document.unit(id)?.clone();
        let data_path = unit
            .single("data_path")
            .map(|text| sii::value::parse_string(text).to_string())?;
        let part = part_info(&data_path, archives, parts);
        // The model's folder, from `/def/vehicle/truck/<model>/data.sii` (`…/bus/…` for a
        // coach).
        if let Some(folder) = data_path
            .strip_prefix("/def/vehicle/truck/")
            .or_else(|| data_path.strip_prefix("/def/vehicle/bus/"))
            .and_then(|rest| rest.strip_suffix("/data.sii"))
        {
            model = Some(folder.to_string());
        }
        if data_path.contains("/chassis/") {
            chassis = part.name.clone();
        }
        if data_path.contains("/engine/") {
            engine = part.name.clone();
        }
        price += part.price;
        // A plain `Unit`, so `set` from `UnitMut` isn't there; the field goes at the end, as the
        // game writes it.
        unit.fields.push(sii::Field {
            name: "refund".to_string(),
            value: sii::Value::single(part.price.to_string()),
        });
        accessories.push(unit);
    }
    Some(DealerTruck {
        is_coach: path.starts_with("/def/vehicle/bus_dealer/"),
        path,
        model: model?,
        chassis,
        engine,
        accessories,
        price,
        dlc,
    })
}

/// A part's price and name, from its definition in the first archive that has it (remembered).
fn part_info(
    path: &str,
    archives: &mut [(hashfs::Archive, Option<String>)],
    parts: &mut HashMap<String, PartInfo>,
) -> PartInfo {
    if let Some(part) = parts.get(path) {
        return part.clone();
    }
    let part = archives
        .iter_mut()
        .find_map(|(archive, _)| read_document(archive, path).ok())
        .and_then(|document| document.units().first().cloned())
        .map(|unit| PartInfo {
            price: unit
                .single("price")
                .and_then(sii::value::parse_i64)
                .unwrap_or(0),
            name: unit
                .single("name")
                .map(|name| sii::value::parse_string(name).to_string())
                .unwrap_or_default(),
        })
        .unwrap_or_default();
    parts.insert(path.to_string(), part.clone());
    part
}
