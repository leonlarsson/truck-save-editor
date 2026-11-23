//! What each company ships and takes: `def/company/<company>/out/<file>.sii` for a cargo it ships
//! and `in/<file>.sii` for one it takes, each a `cargo_def` naming the cargo (`cargo.apples`).
//! From the base game and every DLC.

use std::collections::{BTreeMap, BTreeSet};

use crate::read_document;

/// Cargo tokens (`apples`) by company token (`agronord`).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CompanyCargo {
    pub ships: BTreeMap<String, BTreeSet<String>>,
    pub takes: BTreeMap<String, BTreeSet<String>>,
}

impl CompanyCargo {
    /// Whether `company` ships `cargo`.
    pub fn ships(&self, company: &str, cargo: &str) -> bool {
        self.ships
            .get(company)
            .is_some_and(|cargo_set| cargo_set.contains(cargo))
    }

    /// Whether `company` takes `cargo`.
    pub fn takes(&self, company: &str, cargo: &str) -> bool {
        self.takes
            .get(company)
            .is_some_and(|cargo_set| cargo_set.contains(cargo))
    }
}

/// Reads what every company ships and takes. A few thousand small files: read in the background.
pub fn company_cargo(install_dir: &std::path::Path) -> CompanyCargo {
    let mut found = CompanyCargo::default();
    for path in crate::def_archives(install_dir) {
        let Ok(mut archive) = hashfs::Archive::open(&path) else {
            continue;
        };
        let Ok(Some(companies)) = archive.list("/def/company") else {
            continue;
        };
        for company in companies.directories {
            let company = company.trim_end_matches('/').to_string();
            for (direction, into) in [("out", &mut found.ships), ("in", &mut found.takes)] {
                let folder = format!("/def/company/{company}/{direction}");
                let Ok(Some(listing)) = archive.list(&folder) else {
                    continue;
                };
                for file in listing.files.iter().filter(|name| name.ends_with(".sii")) {
                    let Ok(document) = read_document(&mut archive, &format!("{folder}/{file}"))
                    else {
                        continue;
                    };
                    let cargo = document
                        .first_of_kind("cargo_def")
                        .and_then(|unit| unit.single("cargo"))
                        .map(|cargo| sii::value::parse_string(cargo).to_string());
                    if let Some(token) = cargo.as_deref().and_then(|c| c.strip_prefix("cargo.")) {
                        into.entry(company.clone())
                            .or_default()
                            .insert(token.to_string());
                    }
                }
            }
        }
    }
    found
}
