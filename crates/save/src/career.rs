//! The player's career: money, experience, skills and HQ. Read from a save, changed in the app,
//! and written back field by field (nothing else in the save is touched).

use sii::{Document, Value, value};

use crate::error::{Error, Result};

/// The highest level of each skill (ADR is a set of classes instead).
pub const MAX_SKILL_LEVEL: u8 = 6;

/// All six ADR classes.
pub const ADR_ALL: u8 = 0b11_1111;

#[derive(Clone, Debug, PartialEq)]
pub struct Career {
    /// Bank balance.
    pub money: i64,
    /// Truck experience points.
    pub experience: i64,
    /// Bus experience points (only in saves from versions with buses).
    pub bus_experience: Option<i64>,
    pub skills: Skills,
    /// The HQ city's token (`berlin`).
    pub hq_city: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Skills {
    /// One bit per ADR class.
    pub adr: u8,
    pub long_distance: u8,
    pub high_value_cargo: u8,
    pub fragile_cargo: u8,
    pub just_in_time: u8,
    pub ecodriving: u8,
}

impl Skills {
    /// The leveled skills with their save field names, in the order the game lists them.
    pub fn levels(&self) -> [(&'static str, u8); 5] {
        [
            ("long_dist", self.long_distance),
            ("heavy", self.high_value_cargo),
            ("fragile", self.fragile_cargo),
            ("urgent", self.just_in_time),
            ("mechanical", self.ecodriving),
        ]
    }

    pub fn set_level(&mut self, index: usize, level: u8) {
        let level = level.min(MAX_SKILL_LEVEL);
        match index {
            0 => self.long_distance = level,
            1 => self.high_value_cargo = level,
            2 => self.fragile_cargo = level,
            3 => self.just_in_time = level,
            4 => self.ecodriving = level,
            _ => {}
        }
    }

    pub fn maxed() -> Self {
        Self {
            adr: ADR_ALL,
            long_distance: MAX_SKILL_LEVEL,
            high_value_cargo: MAX_SKILL_LEVEL,
            fragile_cargo: MAX_SKILL_LEVEL,
            just_in_time: MAX_SKILL_LEVEL,
            ecodriving: MAX_SKILL_LEVEL,
        }
    }
}

impl Career {
    pub fn read(document: &Document) -> Result<Self> {
        let economy = document
            .first_of_kind("economy")
            .ok_or(Error::MissingUnit("economy"))?;
        let bank_id = required(economy, "bank")?;
        let bank = document.unit(bank_id).ok_or(Error::MissingUnit("bank"))?;
        let player_id = required(economy, "player")?;
        let player = document
            .unit(player_id)
            .ok_or(Error::MissingUnit("player"))?;

        let skill = |field: &'static str| -> Result<u8> {
            let level = integer(economy, field)?;
            Ok(level.clamp(0, i64::from(u8::MAX)) as u8)
        };

        Ok(Self {
            money: integer(bank, "money_account")?,
            experience: integer(economy, "experience_points")?,
            bus_experience: economy
                .single("bus_experience_points")
                .and_then(value::parse_i64),
            skills: Skills {
                adr: skill("adr")?,
                long_distance: skill("long_dist")?,
                high_value_cargo: skill("heavy")?,
                fragile_cargo: skill("fragile")?,
                just_in_time: skill("urgent")?,
                ecodriving: skill("mechanical")?,
            },
            hq_city: value::parse_string(required(player, "hq_city")?).into_owned(),
        })
    }

    /// Writes the career into the save. Only fields that differ from what's there are touched.
    pub fn write(&self, document: &mut Document) -> Result<()> {
        let economy = document
            .first_of_kind("economy")
            .ok_or(Error::MissingUnit("economy"))?;
        let economy_id = economy.id.clone();
        let bank_id = required(economy, "bank")?.to_string();
        let player_id = required(economy, "player")?.to_string();

        let mut bank = document
            .unit_mut(&bank_id)
            .ok_or(Error::MissingUnit("bank"))?;
        bank.set("money_account", Value::single(self.money.to_string()));

        let mut economy = document
            .unit_mut(&economy_id)
            .ok_or(Error::MissingUnit("economy"))?;
        economy.set(
            "experience_points",
            Value::single(self.experience.to_string()),
        );
        if let Some(bus) = self.bus_experience {
            economy.set("bus_experience_points", Value::single(bus.to_string()));
        }
        economy.set("adr", Value::single(self.skills.adr.to_string()));
        for (field, level) in self.skills.levels() {
            economy.set(field, Value::single(level.to_string()));
        }

        let mut player = document
            .unit_mut(&player_id)
            .ok_or(Error::MissingUnit("player"))?;
        let hq = if value::is_plain_token(&self.hq_city) {
            self.hq_city.clone()
        } else {
            value::format_string(&self.hq_city)
        };
        player.set("hq_city", Value::single(hq));
        Ok(())
    }

    /// How many values differ from `other`: the "N changes" count.
    pub fn changes_from(&self, other: &Career) -> usize {
        let skills = self
            .skills
            .levels()
            .iter()
            .zip(other.skills.levels().iter())
            .filter(|(a, b)| a != b)
            .count();
        usize::from(self.money != other.money)
            + usize::from(self.experience != other.experience)
            + usize::from(self.bus_experience != other.bus_experience)
            + usize::from(self.skills.adr != other.skills.adr)
            + usize::from(self.hq_city != other.hq_city)
            + skills
    }
}

pub(crate) fn required<'a>(unit: &'a sii::Unit, field: &'static str) -> Result<&'a str> {
    unit.single(field).ok_or_else(|| Error::MissingField {
        unit: unit.kind.clone(),
        field,
    })
}

pub(crate) fn integer(unit: &sii::Unit, field: &'static str) -> Result<i64> {
    let text = required(unit, field)?;
    value::parse_i64(text).ok_or_else(|| Error::BadValue {
        unit: unit.kind.clone(),
        field,
        value: text.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) const SAVE: &str = "SiiNunit\r\n{\r\neconomy : _nameless.1 {\r\n bank: _nameless.2\r\n player: _nameless.3\r\n experience_points: 812400\r\n adr: 7\r\n long_dist: 6\r\n heavy: 5\r\n fragile: 4\r\n urgent: 3\r\n mechanical: 2\r\n bus_experience_points: 1200\r\n}\r\n\r\nbank : _nameless.2 {\r\n money_account: 2480350\r\n loans: 0\r\n}\r\n\r\nplayer : _nameless.3 {\r\n hq_city: berlin\r\n}\r\n\r\n}\r\n";

    #[test]
    fn reads_the_career() {
        let document = sii::parse_text(SAVE).expect("parses");
        let career = Career::read(&document).expect("reads");
        assert_eq!(career.money, 2_480_350);
        assert_eq!(career.experience, 812_400);
        assert_eq!(career.bus_experience, Some(1200));
        assert_eq!(career.skills.adr, 7);
        assert_eq!(career.skills.levels().map(|(_, l)| l), [6, 5, 4, 3, 2]);
        assert_eq!(career.hq_city, "berlin");
    }

    #[test]
    fn writes_only_what_changed() {
        let mut document = sii::parse_text(SAVE).expect("parses");
        let mut career = Career::read(&document).expect("reads");
        career.money = 5_000;
        career.skills = Skills::maxed();
        career.write(&mut document).expect("writes");

        let text = sii::to_text(&document);
        let expected = SAVE
            .replace("money_account: 2480350", "money_account: 5000")
            .replace("adr: 7", "adr: 63")
            .replace("heavy: 5", "heavy: 6")
            .replace("fragile: 4", "fragile: 6")
            .replace("urgent: 3", "urgent: 6")
            .replace("mechanical: 2", "mechanical: 6");
        assert_eq!(text, expected);
        assert_eq!(Career::read(&document).expect("reads again"), career);
    }

    #[test]
    fn counts_changes() {
        let document = sii::parse_text(SAVE).expect("parses");
        let original = Career::read(&document).expect("reads");
        let mut changed = original.clone();
        assert_eq!(changed.changes_from(&original), 0);
        changed.money += 1;
        changed.skills.set_level(0, 1);
        changed.skills.set_level(4, 6);
        assert_eq!(changed.changes_from(&original), 3);
    }

    #[test]
    fn says_what_is_missing() {
        let document = sii::parse_text("SiiNunit\r\n{\r\n}\r\n").expect("parses");
        assert!(matches!(
            Career::read(&document),
            Err(Error::MissingUnit("economy"))
        ));
    }
}
