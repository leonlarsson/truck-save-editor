//! Levels from experience points, the way the game counts them: each level needs a set amount of
//! XP more than the one before (`level_xp[]` in the game's `def/economy_data_xp.sii`, and
//! `def/bus_economy_data_xp.sii` for coaches). Past the last listed step, every level costs the
//! last step again.

/// The XP each level needs, from game 1.62 (`level_xp[0..30]`, the same for trucks and coaches).
/// Used when the game's own files can't be read; read them when they can (they may change).
pub const FALLBACK_STEPS: [i64; 30] = [
    200, 500, 700, 900, 1000, 1100, 1300, 1600, 1700, 2100, 2300, 2600, 2700, 2900, 3000, 3100,
    3400, 3700, 4000, 4300, 4600, 4700, 4900, 5200, 5700, 5900, 6000, 6200, 6600, 6800,
];

/// How much XP each level needs.
#[derive(Clone, Debug, PartialEq)]
pub struct LevelTable {
    /// `steps[n]` is the XP from level `n` to level `n + 1`.
    steps: Vec<i64>,
}

/// Where some amount of XP puts a driver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Level {
    pub number: i64,
    /// XP earned since this level started.
    pub progress: i64,
    /// XP this level needs in all, to reach the next one.
    pub step: i64,
}

impl Level {
    /// How far into the level, from 0.0 to 1.0.
    pub fn fraction(&self) -> f32 {
        if self.step <= 0 {
            0.0
        } else {
            (self.progress as f64 / self.step as f64) as f32
        }
    }
}

impl LevelTable {
    /// A table from its steps. Empty or non-positive steps fall back to the built-in table.
    pub fn new(steps: Vec<i64>) -> Self {
        if steps.is_empty() || steps.iter().any(|step| *step <= 0) {
            return Self::default();
        }
        Self { steps }
    }

    fn step(&self, level: i64) -> i64 {
        let index = usize::try_from(level.max(0)).unwrap_or(usize::MAX);
        self.steps
            .get(index)
            .or(self.steps.last())
            .copied()
            .unwrap_or(1)
    }

    /// The level a driver with this much XP is at.
    pub fn level(&self, experience: i64) -> Level {
        let mut remaining = experience.max(0);
        let mut number = 0;
        // Walk the listed steps, then jump over the repeating last step in one go.
        for (index, step) in self.steps.iter().enumerate() {
            if remaining < *step {
                return Level {
                    number: index as i64,
                    progress: remaining,
                    step: *step,
                };
            }
            remaining -= step;
            number = index as i64 + 1;
        }
        let last = self.step(number);
        Level {
            number: number + remaining / last,
            progress: remaining % last,
            step: last,
        }
    }

    /// The XP at which a level starts.
    pub fn experience_for(&self, level: i64) -> i64 {
        let level = level.max(0);
        let listed = usize::try_from(level)
            .unwrap_or(usize::MAX)
            .min(self.steps.len());
        let below: i64 = self.steps[..listed].iter().sum();
        let beyond = level - listed as i64;
        below.saturating_add(beyond.saturating_mul(self.step(listed as i64)))
    }
}

impl Default for LevelTable {
    fn default() -> Self {
        Self {
            steps: FALLBACK_STEPS.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_levels_like_the_game() {
        let table = LevelTable::default();
        assert_eq!(
            table.level(0),
            Level {
                number: 0,
                progress: 0,
                step: 200
            }
        );
        assert_eq!(table.level(199).number, 0);
        assert_eq!(
            table.level(200),
            Level {
                number: 1,
                progress: 0,
                step: 500
            }
        );
        assert_eq!(table.level(700).number, 2);
        // Level 30 starts at the sum of all listed steps; after that each level is 6,800.
        assert_eq!(table.experience_for(30), 99_700);
        assert_eq!(
            table.level(99_700),
            Level {
                number: 30,
                progress: 0,
                step: 6_800
            }
        );
        assert_eq!(table.level(99_700 + 6_800 * 10 + 5).number, 40);
        assert_eq!(table.level(-5).number, 0);
    }

    #[test]
    fn level_and_experience_agree() {
        let table = LevelTable::default();
        for level in [0, 1, 5, 29, 30, 31, 150, 500] {
            let start = table.experience_for(level);
            assert_eq!(table.level(start).number, level, "level {level}");
            assert_eq!(table.level(start).progress, 0);
            if level > 0 {
                assert_eq!(table.level(start - 1).number, level - 1);
            }
        }
    }

    #[test]
    fn falls_back_on_a_broken_table() {
        assert_eq!(LevelTable::new(vec![]), LevelTable::default());
        assert_eq!(LevelTable::new(vec![100, 0]), LevelTable::default());
        assert_eq!(LevelTable::new(vec![100]).level(250).number, 2);
    }
}
