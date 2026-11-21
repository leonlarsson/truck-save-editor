//! What a document is made of: every unit kind, its fields, and what shape of value each holds.
//! Comparing two saves' inventories shows what a game update changed: new units and fields are
//! harmless (they're kept as they are), but a field the editor changes going missing, or holding a
//! different shape of value, needs work. See `docs/compatibility.md`.

use std::collections::BTreeMap;
use std::fmt;

use crate::document::{Document, Value};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Inventory {
    pub kinds: BTreeMap<String, KindInventory>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct KindInventory {
    /// How many units of this kind there are.
    pub count: usize,
    pub fields: BTreeMap<String, FieldShape>,
}

/// What a field holds, as far as its text shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Shape {
    Bool,
    Integer,
    Float,
    Text,
    /// Ids and tokens: `economy`, `_nameless.1.2`, `company.volatile.x`, `null`.
    Id,
    /// `(1, 2, 3)` and placements.
    Vector,
    /// Seen with no value to tell (empty arrays).
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldShape {
    pub is_array: bool,
    pub shape: Shape,
}

impl fmt::Display for FieldShape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_array {
            write!(f, "array of {:?}", self.shape)
        } else {
            write!(f, "{:?}", self.shape)
        }
    }
}

/// Lists a document's units, fields and value shapes.
pub fn inventory(document: &Document) -> Inventory {
    let mut inventory = Inventory::default();
    for unit in document.units() {
        let kind = inventory.kinds.entry(unit.kind.clone()).or_default();
        kind.count += 1;
        for field in &unit.fields {
            let (is_array, shape) = match &field.value {
                Value::Single(text) => (false, shape_of(text)),
                Value::Array(items) => (
                    true,
                    items
                        .first()
                        .map(|item| shape_of(item))
                        .unwrap_or(Shape::Unknown),
                ),
            };
            let entry = kind
                .fields
                .entry(field.name.clone())
                .or_insert(FieldShape { is_array, shape });
            // An array is an array if any unit has items in it; a known shape beats unknown.
            entry.is_array |= is_array;
            if entry.shape == Shape::Unknown {
                entry.shape = shape;
            }
        }
    }
    inventory
}

fn shape_of(text: &str) -> Shape {
    let text = text.trim();
    if text == "true" || text == "false" {
        Shape::Bool
    } else if text.parse::<i64>().is_ok() || text.parse::<u64>().is_ok() {
        Shape::Integer
    } else if text.starts_with('&') || text.parse::<f64>().is_ok() {
        Shape::Float
    } else if text.starts_with('"') {
        Shape::Text
    } else if text.starts_with('(') {
        Shape::Vector
    } else if text.is_empty() {
        Shape::Unknown
    } else {
        Shape::Id
    }
}

/// What changed from one inventory to another.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InventoryDiff {
    pub added_kinds: Vec<String>,
    pub removed_kinds: Vec<String>,
    /// (kind, field, shape)
    pub added_fields: Vec<(String, String, FieldShape)>,
    pub removed_fields: Vec<(String, String, FieldShape)>,
    /// (kind, field, before, after)
    pub changed_fields: Vec<(String, String, FieldShape, FieldShape)>,
}

impl InventoryDiff {
    pub fn is_empty(&self) -> bool {
        self.added_kinds.is_empty()
            && self.removed_kinds.is_empty()
            && self.added_fields.is_empty()
            && self.removed_fields.is_empty()
            && self.changed_fields.is_empty()
    }
}

/// Compares two inventories. Shapes that only differ by one side being unknown (an empty array,
/// a field that's always 0) don't count as changes.
pub fn diff(before: &Inventory, after: &Inventory) -> InventoryDiff {
    let mut diff = InventoryDiff::default();
    for (kind, old) in &before.kinds {
        let Some(new) = after.kinds.get(kind) else {
            diff.removed_kinds.push(kind.clone());
            continue;
        };
        for (field, old_shape) in &old.fields {
            match new.fields.get(field) {
                None => diff
                    .removed_fields
                    .push((kind.clone(), field.clone(), *old_shape)),
                Some(new_shape) if shapes_differ(old_shape, new_shape) => diff
                    .changed_fields
                    .push((kind.clone(), field.clone(), *old_shape, *new_shape)),
                Some(_) => {}
            }
        }
        for (field, new_shape) in &new.fields {
            if !old.fields.contains_key(field) {
                diff.added_fields
                    .push((kind.clone(), field.clone(), *new_shape));
            }
        }
    }
    for kind in after.kinds.keys() {
        if !before.kinds.contains_key(kind) {
            diff.added_kinds.push(kind.clone());
        }
    }
    diff
}

fn shapes_differ(a: &FieldShape, b: &FieldShape) -> bool {
    let unknown = |s: Shape| s == Shape::Unknown;
    if unknown(a.shape) || unknown(b.shape) {
        return false;
    }
    // Floats written as whole numbers look like integers; that isn't a change of type.
    let numeric = |s: Shape| matches!(s, Shape::Integer | Shape::Float);
    if numeric(a.shape) && numeric(b.shape) {
        return a.is_array != b.is_array;
    }
    a != b
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(text: &str) -> Document {
        crate::parse_text(text).expect("parses")
    }

    #[test]
    fn lists_kinds_fields_and_shapes() {
        let document = parse(
            "SiiNunit\n{\nbank : b {\n money: 5\n rate: &3f800000\n name: \"x\"\n on: true\n list: 2\n list[0]: a.b\n list[1]: null\n pos: (1, 2, 3)\n}\nbank : c {\n money: 7\n}\n}\n",
        );
        let inventory = inventory(&document);
        let bank = &inventory.kinds["bank"];
        assert_eq!(bank.count, 2);
        let shape = |field: &str| bank.fields[field];
        assert_eq!(shape("money").shape, Shape::Integer);
        assert_eq!(shape("rate").shape, Shape::Float);
        assert_eq!(shape("name").shape, Shape::Text);
        assert_eq!(shape("on").shape, Shape::Bool);
        assert_eq!(shape("pos").shape, Shape::Vector);
        assert!(shape("list").is_array);
        assert_eq!(shape("list").shape, Shape::Id);
    }

    #[test]
    fn diffs_what_an_update_changed() {
        let before = inventory(&parse(
            "SiiNunit\n{\nbank : b {\n money: 5\n old: 1\n flag: true\n}\nlegacy : l {\n}\n}\n",
        ));
        let after = inventory(&parse(
            "SiiNunit\n{\nbank : b {\n money: &40a00000\n new: 1\n flag: \"yes\"\n}\ncoach : c {\n}\n}\n",
        ));
        let diff = diff(&before, &after);
        assert_eq!(diff.added_kinds, ["coach"]);
        assert_eq!(diff.removed_kinds, ["legacy"]);
        assert_eq!(diff.added_fields.len(), 1);
        assert_eq!(diff.removed_fields.len(), 1);
        // money went from 5 to a float: not a change. flag went from bool to text: a change.
        assert_eq!(diff.changed_fields.len(), 1);
        assert_eq!(diff.changed_fields[0].1, "flag");
        assert!(!diff.is_empty());
    }
}
