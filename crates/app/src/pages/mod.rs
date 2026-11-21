//! The app's pages, one per sidebar entry.

pub mod fleet;
pub mod open_save;
pub mod overview;

use gpui_kit::SharedString;

use gpui_kit::assets::IconName;

/// Every page the sidebar lists, in order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Overview,
    Fleet,
    Garages,
    Drivers,
    Map,
    AllData,
}

impl Page {
    pub const ALL: [Page; 6] = [
        Page::Overview,
        Page::Fleet,
        Page::Garages,
        Page::Drivers,
        Page::Map,
        Page::AllData,
    ];
    pub const CAREER: [Page; 1] = [Page::Overview];
    pub const COMPANY: [Page; 4] = [Page::Fleet, Page::Garages, Page::Drivers, Page::Map];
    pub const ADVANCED: [Page; 1] = [Page::AllData];

    pub fn title(self) -> SharedString {
        match self {
            Page::Overview => "Overview",
            Page::Fleet => "Fleet",
            Page::Garages => "Garages",
            Page::Drivers => "Drivers",
            Page::Map => "Map",
            Page::AllData => "All data",
        }
        .into()
    }

    pub fn icon(self) -> IconName {
        match self {
            Page::Overview => IconName::Gauge,
            Page::Fleet => IconName::Truck,
            Page::Garages => IconName::Warehouse,
            Page::Drivers => IconName::Users,
            Page::Map => IconName::Map,
            Page::AllData => IconName::Database,
        }
    }

    /// Whether the page exists yet. Pages that don't are listed but disabled, so people can see
    /// what's coming instead of wondering where it is.
    pub fn is_built(self) -> bool {
        matches!(self, Page::Overview | Page::Fleet)
    }
}
