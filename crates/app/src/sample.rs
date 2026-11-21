//! Made-up data for the design preview, until real saves load. Nothing here comes from a real save.

/// The skills a driver levels up, in the order the game lists them. ADR is separate (`ADR_CLASSES`).
pub const SKILLS: [SkillInfo; 5] = [
    SkillInfo {
        name: "Long distance",
        description: "Unlocks jobs over longer routes",
    },
    SkillInfo {
        name: "High value cargo",
        description: "Valuable loads that pay more",
    },
    SkillInfo {
        name: "Fragile cargo",
        description: "Loads that break easily",
    },
    SkillInfo {
        name: "Just-in-time delivery",
        description: "Urgent jobs with tight deadlines",
    },
    SkillInfo {
        name: "Ecodriving",
        description: "Uses less fuel",
    },
];

/// The highest level of each skill.
pub const MAX_SKILL_LEVEL: u8 = 6;

/// The ADR classes, in bit order (bit 0 first). Check the order against the game before shipping.
pub const ADR_CLASSES: [&str; 6] = [
    "Explosives",
    "Gases",
    "Flammable liquids",
    "Flammable solids",
    "Toxic",
    "Corrosive",
];

/// All ADR classes at once.
pub const ADR_ALL: u8 = 0b11_1111;

pub struct SkillInfo {
    pub name: &'static str,
    pub description: &'static str,
}

/// The player's career: what the Overview page edits.
#[derive(Clone, PartialEq, Debug)]
pub struct Career {
    pub money: i64,
    pub experience: i64,
    pub skills: [u8; 5],
    pub adr: u8,
}

impl Career {
    pub fn sample() -> Self {
        Self {
            money: 2_480_350,
            experience: 812_400,
            skills: [3, 2, 4, 1, 2],
            adr: 0b00_0111,
        }
    }

    /// How many fields differ from `other`: the "N changes" count.
    pub fn changes_from(&self, other: &Career) -> usize {
        let skill_changes = self
            .skills
            .iter()
            .zip(other.skills.iter())
            .filter(|(a, b)| a != b)
            .count();
        usize::from(self.money != other.money)
            + usize::from(self.experience != other.experience)
            + usize::from(self.adr != other.adr)
            + skill_changes
    }
}

/// A level and how far into it the player is (0.0–1.0).
///
/// PLACEHOLDER curve for the preview. The game's real level table belongs in the `save` crate.
pub fn level_for(experience: i64) -> (i64, f32) {
    let per_level_squared = 400.0;
    let exact = (experience.max(0) as f64 / per_level_squared).sqrt();
    let level = exact.floor();
    (level as i64, (exact - level) as f32)
}

/// One truck in the player's fleet.
#[derive(Clone)]
pub struct Truck {
    /// Stable id (in a real save, the unit id), used for row identity, never the row index.
    pub id: u32,
    pub name: String,
    pub plate: String,
    pub driver: Option<String>,
    pub garage: &'static str,
    pub odometer_km: i64,
    /// 0 = like new, 100 = wrecked.
    pub wear_percent: u8,
}

const MAKES: [&str; 7] = [
    "DAF XG+",
    "Iveco S-Way",
    "MAN TGX",
    "Mercedes-Benz Actros",
    "Renault T",
    "Scania S",
    "Volvo FH16",
];
const CITIES: [&str; 12] = [
    "Amsterdam",
    "Berlin",
    "Bratislava",
    "Brussels",
    "Calais",
    "Kraków",
    "Lyon",
    "Milano",
    "Praha",
    "Stockholm",
    "Wien",
    "Zürich",
];
const FIRST_NAMES: [&str; 10] = [
    "Anna", "Bram", "Clara", "Dmitri", "Elif", "Finn", "Greta", "Hugo", "Ines", "Jonas",
];
const LAST_NAMES: [&str; 10] = [
    "Berg", "Costa", "Dvořák", "Eriksen", "Fischer", "Horvat", "Janssen", "Kowalski", "Moreau",
    "Novak",
];

/// `count` made-up trucks, the same every run.
pub fn trucks(count: u32) -> Vec<Truck> {
    let mut random = Lcg(0x5eed);
    (0..count)
        .map(|id| {
            let has_driver = random.below(10) < 8;
            Truck {
                id,
                name: MAKES[random.below(MAKES.len())].to_string(),
                plate: format!(
                    "{}{} {:03}",
                    letter(random.below(26)),
                    letter(random.below(26)),
                    random.below(1000)
                ),
                driver: has_driver.then(|| {
                    format!(
                        "{} {}",
                        FIRST_NAMES[random.below(FIRST_NAMES.len())],
                        LAST_NAMES[random.below(LAST_NAMES.len())]
                    )
                }),
                garage: CITIES[random.below(CITIES.len())],
                odometer_km: random.below(1_200_000) as i64,
                wear_percent: random.below(101) as u8,
            }
        })
        .collect()
}

fn letter(index: usize) -> char {
    char::from(b'A' + (index % 26) as u8)
}

/// A tiny repeatable random number generator, so the sample looks the same every run.
struct Lcg(u64);

impl Lcg {
    fn below(&mut self, limit: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((self.0 >> 33) as usize) % limit.max(1)
    }
}
