# English text for Truck Save Editor.
#
# Fluent format: https://projectfluent.org/fluent/guide/
# One idea, one name: if a word is used for something here, use the same word everywhere.
# Numbers passed as $count pick the plural form; amounts arrive already formatted.

## Start screen

start-choose = Choose a save to edit
start-looking = Looking for your saves…
start-none-title = No saves found
start-none-body = Saves are looked for in Steam Cloud and in Documents\Euro Truck Simulator 2\profiles. Save once in the game, then look again.
start-look-again = Look again
start-your-saves = Your saves
start-game-saves = Saved by the game
start-slot-number = Slot { $number }
profile-place = { $name } · { $place }
profile-game-place = { $name } · { $game } · { $place }
location-steam-cloud = Steam Cloud
location-this-pc = This PC

## Save slot names (for saves without a name of their own)

slot-unnamed = Unnamed save
slot-autosave = Autosave
slot-autosave-drive = Autosave while driving
slot-autosave-job = Autosave at a job
slot-quicksave = Quicksave
slot-multiplayer-backup = Before convoy

## How long ago

time-just-now = just now
time-minutes-ago = { $count ->
    [one] 1 minute ago
   *[other] { $count } minutes ago
}
time-hours-ago = { $count ->
    [one] 1 hour ago
   *[other] { $count } hours ago
}
time-days-ago = { $count ->
    [one] 1 day ago
   *[other] { $count } days ago
}
time-weeks-ago = { $count ->
    [one] 1 week ago
   *[other] { $count } weeks ago
}
time-months-ago = { $count ->
    [one] 1 month ago
   *[other] { $count } months ago
}
time-years-ago = { $count ->
    [one] 1 year ago
   *[other] { $count } years ago
}

## Sidebar and pages

sidebar-career = Career
sidebar-company = Company
sidebar-advanced = Advanced
page-overview = Overview
page-fleet = Fleet
page-garages = Garages
headquarters = Headquarters
headquarters-pick = Pick a city
page-drivers = Drivers
page-map = Map
page-all-data = All data
switch-save = Switch save
theme-light = Light mode
theme-dark = Dark mode

## Header

header-subtitle = { $game } · { $place }
changes-count = { $count ->
    [0] No changes
    [one] 1 change
   *[other] { $count } changes
}
discard = Discard
save = Save
saving = Saving…

## Opening, closing and saving

opening = Opening “{ $name }”…
open-failed = Couldn't open “{ $name }”
close-unsaved-title = Close “{ $name }” without saving?
close-unsaved-body = { $count ->
    [one] Your change will be lost.
   *[other] Your { $count } changes will be lost.
}
close-unsaved-confirm = Close without saving
save-confirm-title = Save changes to “{ $name }”?
cancel = Cancel
save-as-new-slot = Save as new slot
save-as-new-slot-tooltip = Write the changes into a new slot and keep this one as it is.
new-slot-name = { $name } (edited)
saved-as-new-title = Saved and checked as “{ $name }”
saved-as-new-body = “{ $original }” is unchanged. You're now editing the new slot.
save-confirm-note = A backup of the slot is made first, and the save is read back and checked after writing. Close the game before saving: it overwrites saves when it exits.
save-confirm-cloud = This profile is in Steam Cloud. If Steam asks which copy to keep the next time the game starts, choose the one on this PC.
save-prepare-failed = Couldn't prepare the save
save-no-backup-folder-title = Couldn't save
save-no-backup-folder = There's no folder to keep backups in, so nothing was written.
saved-title = Saved and checked “{ $name }”
saved-backup = The old version is backed up in { $path }
save-failed = Couldn't save “{ $name }”
save-profile-failed = The save was written, but the company name couldn't be: it's still waiting to be saved

## Warnings

newer-game-warning = This save is from a newer version of the game than this editor has been tested with. Everything the editor doesn't understand is kept as it is, but check your changes in the game, and keep the backup.

## Undo and redo

undo = Undo
redo = Redo
undo-tooltip = Undo (Ctrl+Z)
redo-tooltip = Redo (Ctrl+Shift+Z)

## Jobs

page-jobs = Jobs
jobs-summary = { $count ->
    [one] 1 job offer
   *[other] { $amount } job offers
}
jobs-offers = Job offers
jobs-keep = Keep every offer a week
jobs-keep-tooltip = Make every job offer last at least a week of game time, so the one you want is still there.
jobs-kept-week = Kept for at least a week
jobs-table = Job offers
jobs-search = Search by city, company or cargo
jobs-filter-my-garages = From my garages
jobs-filter-urgent = Urgent
jobs-filter-no-ferry = No ferry
column-from = From
column-to = To
column-cargo = Cargo
column-distance = Distance
column-expires = Expires
column-urgency = Urgency
job-place = { $company }, { $city }
job-urgency-standard = Standard
job-urgency-important = Important
job-urgency-urgent = Urgent
job-expires-in = { $minutes ->
    [one] in 1 minute
   *[other] in { $minutes } minutes
}
job-expires-in-hours = { $hours ->
    [one] in 1 hour
   *[other] in { $hours } hours
}
job-expires-in-days = { $days ->
    [one] in 1 day
   *[other] in { $days } days
}
job-expired = Expired
job-ferry = { $distance } · ferry

## Detail panels

details = Details
detail-truck = Truck
detail-coach = Coach
detail-trailer = Trailer
detail-driver = Driver
detail-garage = Garage
detail-about = About
detail-gone = This is no longer in the save.
part-engine = Engine
part-transmission = Transmission
part-cabin = Cabin
part-chassis = Chassis
part-body = Body
part-wheels = Wheels
repair-all-parts = Repair everything
fill-tank = Fill the tank
driver-from = From { $city }
driver-now-in = Now in { $city }
driver-max = Max this driver
garage-empty = None
garage-make-large = Make it large
garage-buy-large = Buy it, large

## Filters

filter-count = { $shown } of { $total }
filter-no-matches = Nothing matches
filter-no-matches-body = Try a different search, or clear the filters.
drivers-none = No drivers hired yet
drivers-none-body = Hire drivers at a recruitment agency in the game, and they show up here.
jobs-none = No job offers
jobs-none-body = The game fills the job market as you drive; this save has none yet.
filter-clear = Clear filters
fleet-search = Search by model, plate or garage
fleet-filter-worn = Needs repair
fleet-filter-no-garage = No garage
garages-search = Search by city
garages-filter-owned = Owned
garages-filter-not-large = Not large
drivers-search = Search by city
drivers-filter-not-maxed = Not maxed
map-search = Search by city
map-filter-not-visited = Not visited
map-filter-dealer-locked = Dealer not found yet
map-filter-recruitment-locked = Agency not found yet

## Quick start

quick-start = Quick start
quick-start-open = Quick start…
quick-start-description = The usual changes for a fresh career, in one go. Start a new profile in the game, open its save here, and pick what you want. One Undo takes it all back, and nothing is written until you save.
quick-start-rich = Rich start
quick-start-everything = Everything unlocked
quick-start-money = Set money
quick-start-hq-garage = A large garage in { $city }
quick-start-apply = Apply
quick-start-no-coaches = This save has no coach career (Coaches DLC).
quick-start-no-hq = This save has no headquarters city yet.
quick-start-hq-large = Your headquarters garage is already large.
quick-start-needs-game = Needs the game installed on this PC: its data says which cities have them.

## Settings

page-settings = Settings
settings-language = Language
settings-language-description = The app's language. The game's own names (cities, cargo, ranks) follow it when the game has them in that language.
settings-appearance = Appearance
settings-appearance-system = Same as the system
settings-appearance-light = Light
settings-appearance-dark = Dark
settings-game = Game folder
settings-game-description = Where Euro Truck Simulator 2 is installed. The editor reads level tables, names and icons from it, and never changes anything there.
settings-game-found = Found automatically
settings-game-picked = Chosen by you
settings-game-not-found = Not found. Choose the folder the game is installed in.
settings-game-not-game = That folder doesn't hold the game: it has no def.scs.
settings-game-choose = Choose folder…
settings-game-automatic = Find it automatically
settings-backups = Backups
settings-backups-description = A backup of a slot is made every time you save it, kept here, outside the game's folders.
settings-backups-open = Open folder
settings-about = About
settings-version = Version { $version }
settings-source = Source code on GitHub
settings-not-affiliated = Not affiliated with or endorsed by SCS Software.

## Command palette

palette-open = Commands
palette-shortcut = Ctrl+K
palette-placeholder = Type a command or a page…
palette-group-pages = Go to
palette-group-actions = Actions
palette-group-app = Save and app
palette-go = { $page }
palette-max-truck-skills = Max all truck skills
palette-max-coach-skills = Max all coach skills
palette-all-adr = All ADR classes
palette-repair-trucks = Repair all trucks
palette-repair-coaches = Repair all coaches
palette-repair-trailers = Repair all trailers
palette-refuel-all = Fill every tank
palette-save = Save…
palette-buy-all-garages = Buy every garage, all large
palette-upgrade-owned-garages = Upgrade owned garages to large
palette-discard = Discard changes
palette-theme = Switch light or dark mode

## Backups

backups = Backups
backups-intro = A backup of the slot is made every time you save, kept outside the game's folders. Restoring one backs up what's there now first, so it can be undone too.
backups-none = No backups yet
backups-none-body = One is made the first time you save this slot.
backup-made-at = Made { $time } (UTC)
backup-show = Show in folder
backup-restore = Restore
restore-title = Restore “{ $name }” from this backup?
restore-body = { $changes ->
    [0] What's in the slot now is backed up first.
    [one] What's in the slot now is backed up first. Your unsaved change will be lost.
   *[other] What's in the slot now is backed up first. Your { $changes } unsaved changes will be lost.
}
restored-title = Restored “{ $name }”
restored-body = The slot is open again as it was in the backup.
restore-failed = Couldn't restore “{ $name }”

## Overview

amount-hint = Try 2.5m, 10k, +1m or -500k.
company-name = Company name
company-name-description = Your company's name, for the whole profile: every save in it. It's kept in the profile file, which is backed up before it's written.
money = Money
money-description = Your company's bank balance.
money-label = Bank balance
experience = Truck experience
experience-description = Your level as a truck driver.
experience-label = Truck experience points
coach-experience = Coach experience
coach-experience-description = Coaching is a career of its own.
coach-experience-label = Coach experience points
experience-unit = XP
experience-amount = { $amount } XP
skills = Truck skills
level-number = Level { $level }
level-progress = { $percent }% to level { $next }
level-progress-label = Progress to the next level
level-step = { $count ->
    [one] 1 level
   *[other] { $count } levels
}
coach-skills = Coach skills
coach-skills-description = Each coach skill goes up to level 3.
coach-skill-longer-duties = Longer duties
coach-skill-longer-duties-description = More stops per duty
coach-skill-demanding-timetables = Demanding timetables
coach-skill-demanding-timetables-description = Stricter schedules that pay more XP
coach-skill-premium-service = Premium service
coach-skill-premium-service-description = Passengers who pay more, and forgive less
coach-skill-cushioned-journey = Cushioned journey
coach-skill-cushioned-journey-description = Smooth driving pays and earns more
coach-skill-passenger-favourite = Passenger favourite
coach-skill-passenger-favourite-description = Flawless driving lifts satisfaction
coach-skill-ecodriving = Ecodriving
coach-skill-ecodriving-description = Uses less fuel
skills-description = Click a level to set it. Click the highest filled level again to lower it.
skills-max = Max all skills
skill-level = Level { $level }
skill-level-of = { $skill }: level { $level } of { $max }
skill-long-distance = Long distance
skill-long-distance-description = Unlocks jobs over longer routes
skill-high-value-cargo = High value cargo
skill-high-value-cargo-description = Valuable loads that pay more
skill-fragile-cargo = Fragile cargo
skill-fragile-cargo-description = Loads that break easily
skill-just-in-time = Just-in-time delivery
skill-just-in-time-description = Urgent jobs with tight deadlines
skill-ecodriving = Ecodriving
skill-ecodriving-description = Uses less fuel
adr = ADR
adr-description = Dangerous goods your driver may haul.
adr-all = All classes
adr-none = None
adr-some = { $count } of { $total } classes
adr-explosives = Explosives
adr-gases = Gases
adr-flammable-liquids = Flammable liquids
adr-flammable-solids = Flammable solids
adr-toxic = Toxic
adr-corrosive = Corrosive
hq = HQ

## All data

all-data-intro = Every unit in the save, and every value in it. Pick a kind, then a unit, then a value to change.
all-data-search = Search every value, field and unit
all-data-search-count = { $count ->
    [0] Nothing matches
    [one] 1 match
   *[other] { $amount } matches
}
all-data-search-more = The first { $count } matches
all-data-results = Search results
all-data-kinds = Kinds
all-data-units = Units
all-data-values = Values
all-data-filter = Filter kinds
all-data-kind = Kind
all-data-count = Units
all-data-unit = Unit
all-data-field = Field
all-data-value = Value
all-data-items = { $count ->
    [one] 1 item
   *[other] { $count } items
}
all-data-pick-value = Pick a value
all-data-apply = Apply
all-data-revert = Revert
all-data-wrong-shape = This value holds { $expected }; type one of those (text keeps its quotes).
shape-bool = true or false
shape-number = a number
shape-text = text in quotes
shape-id = an id or name
shape-vector = numbers in brackets
shape-value = a value

## Map

map-summary = { $visited } of { $count } cities visited
map-visit-all = Visit every city
map-visit-all-tooltip = Mark every city as visited, as if you'd driven through each.
map-visited-all = { $count ->
    [one] 1 more city visited
   *[other] { $count } more cities visited
}
map-visits = { $count ->
    [0] Not visited
    [one] Once
   *[other] { $count } times
}
column-visits = Visited
map-table = Cities
map-dealers = Truck dealers
map-recruitments = Recruitment agencies
map-dealers-summary = { $unlocked } of { $count ->
    [one] 1 truck dealer
   *[other] { $count } truck dealers
}
map-recruitments-summary = { $unlocked } of { $count ->
    [one] 1 recruitment agency
   *[other] { $count } recruitment agencies
}
map-unlocked-count = { $count } unlocked
map-unlocked-of = { $unlocked } of { $count } unlocked
map-unlock-dealers = Unlock every dealer
map-unlock-dealers-tooltip = Unlock the truck dealer in every city that has one, as if you'd driven past each.
map-unlock-recruitments = Unlock every agency
map-unlock-recruitments-tooltip = Unlock the recruitment agency in every city that has one, as if you'd driven past each.
map-needs-game = Unlocking dealers and agencies needs the game installed on this PC: its data says which cities have them.
column-dealer = Truck dealer
column-recruitment = Recruitment agency
map-place-unlocked = Unlocked
map-place-locked = Not found yet

## Garages

garages = Garages
garages-before = As they were
garages-after = { $bought ->
    [0] { $upgraded ->
        [one] 1 upgraded to large
       *[other] { $upgraded } upgraded to large
    }
   *[other] { $upgraded ->
        [0] { $bought } bought, all large
       *[other] { $bought } bought and { $upgraded } upgraded, all large
    }
}
garages-summary = { $count ->
    [one] 1 city
   *[other] { $count } cities
} · { $owned } owned · { $large } large
garages-upgrade-owned = Upgrade owned
garages-upgrade-owned-tooltip = Make every garage you own large: 5 trucks and 5 drivers.
garages-buy-all = Buy all
garages-buy-all-tooltip = Buy every city's garage, all large.
garage-not-owned = Not owned
garage-large = Large
garage-slots = { $count ->
    [one] 1 slot
   *[other] { $count } slots
}
garage-filled = { $filled } of { $slots }
column-city = City
column-garage-size = Garage
column-trucks = Trucks
column-drivers = Drivers
garages-table = Garages

## Drivers

drivers-summary = { $count ->
    [0] No drivers hired yet
    [one] 1 driver
   *[other] { $count } drivers
} · { $maxed ->
    [0] none fully skilled
   *[other] { $maxed } fully skilled
}
drivers-max-all = Max all drivers
drivers-max-all-tooltip = Every skill at its highest level and all ADR classes, for every hired driver.
drivers-maxed = { $count ->
    [one] 1 driver maxed out
   *[other] { $count } drivers maxed out
}
column-hometown = Hometown
column-location = Now in
column-experience = Experience
column-skills = Skills
column-adr = ADR
drivers-table = Drivers
skills-short = { $long } · { $value } · { $fragile } · { $urgent } · { $eco }
adr-count = { $count } of { $total }

## Fleet

fleet-trucks = { $count ->
    [one] 1 truck
   *[other] { $amount } trucks
}
fleet-worn = { $count ->
    [0] all in good shape
    [one] 1 needs repairs
   *[other] { $amount } need repairs
}
fleet-coaches = { $count ->
    [one] 1 coach
   *[other] { $amount } coaches
}
fleet-tab-trucks = Trucks · { $count }
fleet-tab-coaches = Coaches · { $count }
fleet-summary = { $vehicles } · { $worn }
repair-all = Repair all
repairs = Repairs
repairs-before = Worn
repairs-after = { $count ->
    [one] 1 vehicle repaired
   *[other] { $count } vehicles repaired
}
column-truck = Model
column-plate = Plate
column-garage = Garage
column-odometer = Odometer
column-wear = Wear
repair = Repair
refuel-all = Fill every tank
refuel-all-tooltip = Fill the fuel tank of every truck or coach in this list.
column-fuel = Fuel
vehicle-in-use = Driving now
vehicle-parked = Parked in { $city }
fuel = Fuel
fuel-before = As it was
fuel-after = { $count ->
    [one] 1 tank filled
   *[other] { $count } tanks filled
}
fleet-trailers = { $count ->
    [one] 1 trailer
   *[other] { $amount } trailers
}
fleet-tab-trailers = Trailers · { $count }
no-garage = No garage
distance-km = { $amount } km
percent = { $amount }%
trucks-table = Vehicles
unknown-truck = Unknown model
