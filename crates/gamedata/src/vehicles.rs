//! The vehicle models in the installed game (its own and every truck, coach or trailer DLC's): each
//! model's brand and series, as text keys (`brand_volvo`, `series_volvo_fh16_2012`), from
//! `def/vehicle/{truck,bus,trailer_owned}/<brand.model>/data.sii`. Trailers have no brand text
//! (an empty key), only a series.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::read_document;

/// Brand and series text keys, by model id (`volvo.fh16_2012`).
pub(crate) fn read_models(archives: &[PathBuf]) -> HashMap<String, (String, String)> {
    let mut models = HashMap::new();
    for path in archives {
        let Ok(mut archive) = hashfs::Archive::open(path) else {
            continue;
        };
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
                let Ok(document) = read_document(&mut archive, &data_path) else {
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
    models
}
