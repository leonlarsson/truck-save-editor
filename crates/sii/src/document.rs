//! A SII document: units in file order, each with its fields in file order.
//!
//! Values are kept in their text form (`2480350`, `"My save"`, `&3e98ae14`, `(1, 2, 3)`,
//! `_nameless.1ec.ab8c.2808`), exactly as text SII writes them. That keeps writing back exact
//! (nothing is reformatted or rounded), and binary saves decode into the same form. The `value`
//! module reads and writes the typed values inside.

use std::collections::HashMap;

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
