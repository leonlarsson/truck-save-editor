//! A SII document: units in file order, each with its fields in file order.
//!
//! Values are kept in their text form (`2480350`, `"My save"`, `&3e98ae14`, `(1, 2, 3)`,
//! `_nameless.1ec.ab8c.2808`), exactly as text SII writes them. That keeps writing back exact
//! (nothing is reformatted or rounded), and binary saves decode into the same form. The `value`
//! module reads and writes the typed values inside.

use std::collections::{HashMap, HashSet};

/// A whole SII file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Document {
    units: Vec<Unit>,
    /// Unit id → position in `units`.
    index: HashMap<String, usize>,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a unit at the end.
    pub fn push(&mut self, unit: Unit) {
        // `null` and anonymous ids aren't references, so they aren't indexed.
        if unit.id != "null" {
            self.index.insert(unit.id.clone(), self.units.len());
        }
        self.units.push(unit);
    }

    pub fn units(&self) -> &[Unit] {
        &self.units
    }

    pub fn len(&self) -> usize {
        self.units.len()
    }

    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    /// The unit with this id (`economy.bank` → the bank).
    pub fn unit(&self, id: &str) -> Option<&Unit> {
        self.index
            .get(id)
            .and_then(|&position| self.units.get(position))
    }

    /// The unit with this id, to change its fields. (Its id can't be changed, which keeps the
    /// index right.)
    pub fn unit_mut(&mut self, id: &str) -> Option<UnitMut<'_>> {
        let position = *self.index.get(id)?;
        self.units.get_mut(position).map(UnitMut)
    }

    /// Every unit of a kind (`vehicle`, `garage` …), in file order.
    pub fn units_of_kind<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a Unit> + 'a {
        self.units.iter().filter(move |unit| unit.kind == kind)
    }

    /// The first unit of a kind, for kinds there's only one of (`economy`, `bank`).
    pub fn first_of_kind(&self, kind: &str) -> Option<&Unit> {
        self.units.iter().find(|unit| unit.kind == kind)
    }

    /// The ids of every unit reachable from `root` (itself included) by following values that
    /// name a unit. The game reads a save as one tree from its root; a unit nothing points at
    /// is a second tree, and the game refuses the file ("There are multiple unit trees").
    pub fn reachable_from(&self, root: &str) -> HashSet<String> {
        let mut reached = HashSet::new();
        let mut waiting = vec![root.to_string()];
        while let Some(id) = waiting.pop() {
            let Some(unit) = self.unit(&id) else {
                continue;
            };
            if !reached.insert(id) {
                continue;
            }
            for field in &unit.fields {
                let values: &[String] = match &field.value {
                    Value::Single(text) => std::slice::from_ref(text),
                    Value::Array(items) => items,
                };
                for value in values {
                    if self.index.contains_key(value) && !reached.contains(value) {
                        waiting.push(value.clone());
                    }
                }
            }
        }
        reached
    }

    /// `count` anonymous ids (`_nameless.…`) that no unit has, for units an edit adds. They come
    /// after the highest anonymous id in the document, 8 apart as the game's are (they're its
    /// memory addresses, which it hands out again each time it saves).
    pub fn unused_nameless_ids(&self, count: usize) -> Vec<String> {
        let highest = self
            .units
            .iter()
            .filter_map(|unit| nameless_value(&unit.id))
            .max()
            .unwrap_or(0);
        // Rounded up to the next multiple of 8, then on from there.
        let first = (highest | 7).saturating_add(1);
        (0..count as u64)
            .map(|index| nameless_id(first.saturating_add(index * 8)))
            .collect()
    }

    /// Removes units by id, keeping the others in their order.
    pub fn remove_units(&mut self, ids: &HashSet<String>) {
        if ids.is_empty() {
            return;
        }
        self.units.retain(|unit| !ids.contains(&unit.id));
        self.index = self
            .units
            .iter()
            .enumerate()
            .filter(|(_, unit)| unit.id != "null")
            .map(|(position, unit)| (unit.id.clone(), position))
            .collect();
    }
}

/// An anonymous id's number: `_nameless.246.5b82.fe8` → 0x0246_5b82_0fe8 (each dot-separated
/// part is 16 bits, leading zero parts left out). `None` for any other id.
pub fn nameless_value(id: &str) -> Option<u64> {
    let parts = id.strip_prefix("_nameless.")?;
    let mut value: u64 = 0;
    let mut count = 0;
    for part in parts.split('.') {
        let group = u64::from_str_radix(part, 16)
            .ok()
            .filter(|group| *group <= 0xFFFF)?;
        value = value.checked_shl(16)? | group;
        count += 1;
    }
    (1..=4).contains(&count).then_some(value)
}

/// An anonymous id for a number, written as the game writes it (see `nameless_value`).
pub fn nameless_id(value: u64) -> String {
    let groups = [48, 32, 16, 0].map(|shift| (value >> shift) & 0xFFFF);
    let first = groups.iter().position(|&group| group != 0).unwrap_or(3);
    let mut id = String::from("_nameless");
    for group in &groups[first..] {
        id.push_str(&format!(".{group:x}"));
    }
    id
}

/// One unit: `kind : id { fields }`.
#[derive(Clone, Debug, PartialEq)]
pub struct Unit {
    pub kind: String,
    pub id: String,
    pub fields: Vec<Field>,
}

impl Unit {
    pub fn new(kind: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            id: id.into(),
            fields: Vec::new(),
        }
    }

    /// The value of a field. If a field appears more than once, the last one wins, as in the game.
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.fields
            .iter()
            .rev()
            .find(|field| field.name == name)
            .map(|field| &field.value)
    }

    /// A single value's text (`None` for missing fields and arrays).
    pub fn single(&self, name: &str) -> Option<&str> {
        self.get(name).and_then(Value::as_single)
    }

    /// An array's items. A missing field, or an array written as just its count (`trucks: 0`),
    /// gives no items.
    pub fn items(&self, name: &str) -> &[String] {
        self.get(name).map(Value::items).unwrap_or(&[])
    }
}

/// A unit borrowed for changes. It can change fields, not the unit's id.
pub struct UnitMut<'a>(&'a mut Unit);

impl UnitMut<'_> {
    pub fn kind(&self) -> &str {
        &self.0.kind
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.0.get(name)
    }

    /// Sets a field's value, in place if it exists (keeping its position), else at the end.
    pub fn set(&mut self, name: &str, value: Value) {
        match self
            .0
            .fields
            .iter_mut()
            .rev()
            .find(|field| field.name == name)
        {
            Some(field) => field.value = value,
            None => self.0.fields.push(Field {
                name: name.to_string(),
                value,
            }),
        }
    }
}

/// One `name: value` line, or an array of them.
#[derive(Clone, Debug, PartialEq)]
pub struct Field {
    pub name: String,
    pub value: Value,
}

/// A field's value, in its text form.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Single(String),
    /// Written as `name: count` and then `name[0]: …`, `name[1]: …`.
    Array(Vec<String>),
}

impl Value {
    pub fn single(text: impl Into<String>) -> Self {
        Value::Single(text.into())
    }

    pub fn as_single(&self) -> Option<&str> {
        match self {
            Value::Single(text) => Some(text),
            Value::Array(_) => None,
        }
    }

    /// An array's items; a single value (like an array written as only its count) has none.
    pub fn items(&self) -> &[String] {
        match self {
            Value::Single(_) => &[],
            Value::Array(items) => items,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_and_writes_anonymous_ids() {
        assert_eq!(
            nameless_value("_nameless.246.5b82.fe8"),
            Some(0x0246_5b82_0fe8)
        );
        assert_eq!(nameless_id(0x0246_5b82_0fe8), "_nameless.246.5b82.fe8");
        assert_eq!(nameless_id(0x1_0002), "_nameless.1.2");
        assert_eq!(nameless_value("garage.lyon"), None);
        assert_eq!(nameless_value("_nameless.12345"), None);
        let mut document = Document::new();
        document.push(Unit::new("bank", "_nameless.1.2"));
        document.push(Unit::new("garage", "garage.lyon"));
        assert_eq!(
            document.unused_nameless_ids(2),
            ["_nameless.1.8", "_nameless.1.10"]
        );
    }

    fn tree() -> Document {
        let mut document = Document::new();
        let unit = |kind: &str, id: &str, fields: &[(&str, Value)]| Unit {
            kind: kind.to_string(),
            id: id.to_string(),
            fields: fields
                .iter()
                .map(|(name, value)| Field {
                    name: name.to_string(),
                    value: value.clone(),
                })
                .collect(),
        };
        document.push(unit(
            "economy",
            "root",
            &[("bank", Value::single("bank.1"))],
        ));
        document.push(unit(
            "bank",
            "bank.1",
            &[("loans", Value::Array(vec!["loan.1".into(), "null".into()]))],
        ));
        document.push(unit("loan", "loan.1", &[]));
        document.push(unit(
            "job",
            "job.1",
            &[("trailer", Value::single("trailer.1"))],
        ));
        document.push(unit("trailer", "trailer.1", &[]));
        document
    }

    #[test]
    fn follows_references_from_the_root() {
        let document = tree();
        let reached = document.reachable_from("root");
        let expected: HashSet<String> = ["root", "bank.1", "loan.1"].map(String::from).into();
        assert_eq!(reached, expected);
    }

    #[test]
    fn removes_units_and_keeps_the_rest_in_order() {
        let mut document = tree();
        let gone: HashSet<String> = ["job.1", "trailer.1"].map(String::from).into();
        document.remove_units(&gone);
        let ids: Vec<&str> = document.units().iter().map(|u| u.id.as_str()).collect();
        assert_eq!(ids, ["root", "bank.1", "loan.1"]);
        assert!(document.unit("loan.1").is_some() && document.unit("job.1").is_none());
    }

    fn bank() -> Unit {
        let mut unit = Unit::new("bank", "_nameless.1.2");
        unit.fields.push(Field {
            name: "money_account".into(),
            value: Value::single("100"),
        });
        unit.fields.push(Field {
            name: "loans".into(),
            value: Value::single("0"),
        });
        unit
    }

    #[test]
    fn finds_units_by_id_and_kind() {
        let mut document = Document::new();
        document.push(Unit::new("economy", "economy"));
        document.push(bank());
        assert_eq!(
            document.unit("_nameless.1.2").map(|u| u.kind.as_str()),
            Some("bank")
        );
        assert_eq!(
            document.first_of_kind("economy").map(|u| u.id.as_str()),
            Some("economy")
        );
        assert!(document.unit("nope").is_none());
    }

    #[test]
    fn sets_fields_in_place() {
        let mut document = Document::new();
        document.push(bank());
        let mut unit = document.unit_mut("_nameless.1.2").expect("bank");
        unit.set("money_account", Value::single("5000"));
        unit.set("overdraft", Value::single("false"));
        let bank = document.unit("_nameless.1.2").expect("bank");
        assert_eq!(bank.fields[0].value, Value::single("5000"));
        assert_eq!(
            bank.fields.last().map(|f| f.name.as_str()),
            Some("overdraft")
        );
    }

    #[test]
    fn arrays_written_as_a_count_have_no_items() {
        let mut unit = Unit::new("player", "player");
        unit.fields.push(Field {
            name: "trucks".into(),
            value: Value::single("0"),
        });
        assert!(unit.items("trucks").is_empty());
        assert!(unit.items("missing").is_empty());
    }
}
