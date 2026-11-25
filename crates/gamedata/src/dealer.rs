//! The trucks the game's dealers sell ready-made (their stock offers): one file each under
//! `/def/vehicle/truck_dealer/<brand>/` in the base game's definitions or a DLC's (ATS keeps each
//! truck brand's in its own archive). A file is a `vehicle` unit listing its accessories, then the
//! accessories, each pointing at a part's definition (`data_path`): written almost as a save keeps
//! a truck, so buying one copies its accessories into the save.
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
    let mut archives: Vec<(hashfs::Archive, Option<String>)> = crate::def_archives(install_dir)
        .iter()
        .filter_map(|path| {
            let mut archive = hashfs::Archive::open(path).ok()?;
            let dlc = crate::archive_dlc(&mut archive, path);
            Some((archive, dlc))
        })
        .collect();
    let mut parts: HashMap<String, PartInfo> = HashMap::new();
    let mut trucks: Vec<DealerTruck> = Vec::new();
    for index in 0..archives.len() {
        for file in dealer_files(&mut archives[index].0) {
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

/// The dealer files in one archive: `/def/vehicle/truck_dealer/<brand>/<offer>.sii`. (ETS2's
/// `truck_dealer_uk`, the right-hand-drive versions, are left out: the same trucks again.)
fn dealer_files(archive: &mut hashfs::Archive) -> Vec<String> {
    let root = "/def/vehicle/truck_dealer";
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
        // The model's folder, from `/def/vehicle/truck/<model>/data.sii`.
        if let Some(folder) = data_path
            .strip_prefix("/def/vehicle/truck/")
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
