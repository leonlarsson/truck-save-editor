//! The vehicle models in the installed game (its own and every truck, coach or trailer DLC's): each
//! model's brand and series, as text keys (`brand_volvo`, `series_volvo_fh16_2012`), from
//! `def/vehicle/{truck,bus,trailer_owned}/<brand.model>/data.sii`. Trailers have no brand text
//! (an empty key), only a series.

use std::collections::HashMap;

use crate::read_document;

/// Adds the models an archive defines: brand and series text keys, by model id
/// (`volvo.fh16_2012`). Models already known are kept.
pub(crate) fn add_models(
    archive: &mut hashfs::Archive,
    models: &mut HashMap<String, (String, String)>,
) {
    {
        for kind in ["truck", "bus", "trailer_owned"] {
            let folder = format!("/def/vehicle/{kind}");
            let Ok(Some(listing)) = archive.list(&folder) else {
                continue;
            };
            for model in listing.directories {
                let model = model.trim_end_matches('/').to_string();
                if models.contains_key(&model) {
                    continue;
                }
                // Paint and accessory DLCs list every model folder too, without its data.
                let data_path = format!("{folder}/{model}/data.sii");
                if !archive.contains(&data_path) {
                    continue;
                }
                let Ok(document) = read_document(archive, &data_path) else {
                    continue;
                };
                let info = document
                    .first_of_kind("accessory_truck_data")
                    .or_else(|| document.first_of_kind("accessory_bus_data"))
                    .or_else(|| document.first_of_kind("accessory_trailer_data"))
                    .map(|unit| unit.items("info").to_vec())
                    .unwrap_or_default();
                let key = |index: usize| {
                    info.get(index)
                        .map(|text| sii::value::parse_string(text).trim_matches('@').to_string())
                };
                if let (Some(brand), Some(series)) = (key(0), key(1)) {
                    models.insert(model, (brand, series));
                }
            }
        }
    }
}

/// An engine or a transmission a truck model can have.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PartKind {
    Engine,
    Transmission,
}

impl PartKind {
    fn folder(self) -> &'static str {
        match self {
            PartKind::Engine => "engine",
            PartKind::Transmission => "transmission",
        }
    }

    fn unit_kind(self) -> &'static str {
        match self {
            PartKind::Engine => "accessory_engine_data",
            PartKind::Transmission => "accessory_transmission_data",
        }
    }
}

/// One engine or transmission, as the game defines it (texts still as the game's keys).
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    /// Its definition (`/def/vehicle/truck/volvo.fh16_2012/engine/d16g750.sii`): what a truck's
    /// accessory points at.
    pub path: String,
    /// Its name, as a text key or plain text (`@@engine_d16g750@@`).
    pub name: String,
    /// Lines describing it (`750 @@hp@@ (551@@kw@@)`).
    pub info: Vec<String>,
    pub price: i64,
    /// The level it unlocks at.
    pub unlock: i64,
}

/// The engines or transmissions a truck (or coach) model can have, from the base game and every
/// DLC, by its model folder (`volvo.fh16_2012`). Read when asked: a model has a few dozen.
pub fn model_parts(install_dir: &std::path::Path, model: &str, kind: PartKind) -> Vec<Part> {
    let mut parts: Vec<Part> = Vec::new();
    for path in crate::def_archives(install_dir) {
        let Ok(mut archive) = hashfs::Archive::open(&path) else {
            continue;
        };
        for vehicle in ["truck", "bus"] {
            let folder = format!("/def/vehicle/{vehicle}/{model}/{}", kind.folder());
            let Ok(Some(listing)) = archive.list(&folder) else {
                continue;
            };
            for file in listing.files.iter().filter(|name| name.ends_with(".sii")) {
                let path = format!("{folder}/{file}");
                if parts.iter().any(|part| part.path == path) {
                    continue;
                }
                let Ok(document) = read_document(&mut archive, &path) else {
                    continue;
                };
                let Some(unit) = document.first_of_kind(kind.unit_kind()) else {
                    continue;
                };
                let number = |field: &str| {
                    unit.single(field)
                        .and_then(sii::value::parse_i64)
                        .unwrap_or(0)
                };
                parts.push(Part {
                    path,
                    name: unit
                        .single("name")
                        .map(|name| sii::value::parse_string(name).to_string())
                        .unwrap_or_default(),
                    info: unit
                        .items("info")
                        .iter()
                        .map(|line| sii::value::parse_string(line).to_string())
                        .collect(),
                    price: number("price"),
                    unlock: number("unlock"),
                });
            }
        }
    }
    parts.sort_by(|a, b| a.price.cmp(&b.price).then(a.path.cmp(&b.path)));
    parts
}
