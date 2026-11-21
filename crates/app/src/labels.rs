//! What the game calls things, for showing them. One place, so every screen uses the same words.

/// The leveled skills in the order the game lists them (matching `save::Skills::levels`).
pub const SKILLS: [SkillLabel; 5] = [
    SkillLabel {
        name: "Long distance",
        description: "Unlocks jobs over longer routes",
    },
    SkillLabel {
        name: "High value cargo",
        description: "Valuable loads that pay more",
    },
    SkillLabel {
        name: "Fragile cargo",
        description: "Loads that break easily",
    },
    SkillLabel {
        name: "Just-in-time delivery",
        description: "Urgent jobs with tight deadlines",
    },
    SkillLabel {
        name: "Ecodriving",
        description: "Uses less fuel",
    },
];

/// The ADR classes, in bit order (bit 0 first). Check the order against the game before shipping.
pub const ADR_CLASSES: [&str; 6] = [
    "Explosives",
    "Gases",
    "Flammable liquids",
    "Flammable solids",
    "Toxic",
    "Corrosive",
];

pub struct SkillLabel {
    pub name: &'static str,
    pub description: &'static str,
}
