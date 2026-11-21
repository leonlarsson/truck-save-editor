//! What differs between Euro Truck Simulator 2 and American Truck Simulator. Everything that
//! depends on the game asks a `Game`, so supporting ATS stays a matter of filling this in.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Game {
    Ets2,
    Ats,
}

impl Game {
    pub const ALL: [Game; 2] = [Game::Ets2, Game::Ats];

    pub fn name(self) -> &'static str {
        match self {
            Game::Ets2 => "Euro Truck Simulator 2",
            Game::Ats => "American Truck Simulator",
        }
    }

    /// The folder under Documents (and the game's name in paths).
    pub fn documents_folder(self) -> &'static str {
        self.name()
    }

    pub fn steam_app_id(self) -> u32 {
        match self {
            Game::Ets2 => 227_300,
            Game::Ats => 270_880,
        }
    }

    /// The game's process, to refuse writing while it runs.
    pub fn process_names(self) -> &'static [&'static str] {
        match self {
            Game::Ets2 => &["eurotrucks2.exe", "eurotrucks2"],
            Game::Ats => &["amtrucks.exe", "amtrucks"],
        }
    }

    pub fn currency_symbol(self) -> &'static str {
        match self {
            Game::Ets2 => "€",
            Game::Ats => "$",
        }
    }

    pub fn uses_miles(self) -> bool {
        matches!(self, Game::Ats)
    }
}
