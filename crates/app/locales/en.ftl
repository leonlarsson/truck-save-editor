# English text for Truck Save Editor.
#
# Fluent format: https://projectfluent.org/fluent/guide/
# One idea, one name: if a word is used for something here, use the same word everywhere.
# Numbers passed as $count pick the plural form; amounts arrive already formatted.

## Start screen

start-choose = Choose a save to edit
start-looking = Looking for your saves…
start-none-title = No saves found
start-none-body = None of the places below has a profile of the game. If you play on this PC, start the game and save once: the list updates when you come back to this window. If your saves are somewhere else, such as a backup or another drive, add that folder.
start-look-again = Look again
start-your-saves = Your saves
start-game-saves = Saved by the game
start-game-saves-hint = Autosaves and quicksaves. The game replaces them as you play.
start-game-saves-more = { $count ->
    [one] Show one more
   *[other] Show { $count } more
}
start-game-saves-fewer = Show fewer
start-showing-cloud = { $count ->
    [one] One save in “{ $name }”, kept in Steam Cloud, so it's the same on every PC you play on.
   *[other] { $count } saves in “{ $name }”, kept in Steam Cloud, so they're the same on every PC you play on.
}
start-showing-local = { $count ->
    [one] One save in “{ $name }”, kept on this PC only.
   *[other] { $count } saves in “{ $name }”, kept on this PC only.
}
start-profile-empty-title = No saves in “{ $name }” yet
start-profile-empty-body = Save once in the game and it shows here. The list updates when you come back to this window.
start-folders = Where saves are looked for
start-folders-description = The game keeps each profile in Steam Cloud or on this PC, whichever its profile settings in the game say.
start-folders-missing = { $count ->
    [one] Also looked in one folder that isn't on this PC
   *[other] Also looked in { $count } folders that aren't on this PC
}
start-folders-missing-hide = Hide the folders that aren't there
start-folder-added = Added by you
start-folder-profiles = { $count ->
    [one] One profile
   *[other] { $count } profiles
}
start-folder-empty = No profiles
start-folder-missing = Not there
start-folder-no-steam = Steam wasn't found on this PC
start-folder-show = Show the folder
start-folder-remove = Stop looking here
start-folder-add = Add a folder…
start-folder-drop = Or drop a save or profile folder on this window.
start-folder-choose = Choose a folder with saves
start-folder-not-saves = No saves or profiles in { $path }. Choose the game's profiles folder, one profile's folder, or one save's.
start-slot-missing = { $count ->
    [one] Needs a DLC or mod that isn't installed
   *[other] Needs { $count } DLCs or mods that aren't installed
}
start-slot-number = Slot { $number }
profile-place = { $name } · { $place }
profile-game-place = { $name } · { $game } · { $place }
start-game-detail = { $profiles ->
    [one] One profile
   *[other] { $profiles } profiles
} · { $saves ->
    [one] one save
   *[other] { $saves } saves
}
start-game-none = Not found on this PC
location-steam-cloud = Steam Cloud
location-this-pc = This PC
location-loose = A save on its own

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
sidebar-map = Map
sidebar-advanced = Advanced
page-overview = Overview
page-fleet = Fleet
page-garages = Garages
headquarters = Headquarters
headquarters-pick = Pick a city
page-drivers = Drivers
page-map = Map
page-cities = Cities
page-world-map = World map
page-gallery = Gallery
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
save-open = Save…
details-open = Details…
settings-open = Settings…
game-settings-open = Game settings…
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
quit-unsaved-title = Quit without saving “{ $name }”?
quit-unsaved-confirm = Quit without saving
save-confirm-title = Save changes to “{ $name }”?
cancel = Cancel
save-as-new-slot = Save as new slot
save-as-new-slot-tooltip = Write the changes into a new slot and keep this one as it is.
save-as-new-slot-loose = This save is on its own, outside a profile, so there's no profile to add a slot to.
new-slot-name = { $name } (edited)
saved-as-new-title = Saved and checked as “{ $name }”
saved-as-new-body = “{ $original }” is unchanged. You're now editing the new slot.
save-confirm-note = A backup of the slot is made first, and the save is read back and checked after writing. Close the game before saving: it overwrites saves when it exits.
save-confirm-cloud = This profile is in Steam Cloud: if Steam asks which copy to keep, choose the one on this PC. Save as new slot is off, because the game only lists saves it made itself; to keep this save as it is, save over another one instead.
save-prepare-failed = Couldn't prepare the save
save-no-backup-folder-title = Couldn't save
save-no-backup-folder = There's no folder to keep backups in, so nothing was written.
save-over-other = Save over another…
save-over-other-tooltip = Write the changes over another save of this profile and keep this one as it is. Works with Steam Cloud.
save-over-pick-title = Save over which save?
save-over-pick-body = The one you pick is backed up, then replaced by “{ $original }” with your changes. It keeps its name. “{ $original }” stays as it is.
save-over-confirm-title = Replace “{ $name }”?
save-over-confirm-body = “{ $name }” is backed up first, then replaced by “{ $original }” with your changes, under its own name. “{ $original }” stays as it is, and you carry on editing “{ $name }”.
save-over-confirm = Replace it
save-over-no-backups = There's nowhere to keep the backup, so nothing was written.
saved-over-title = Saved over “{ $name }” and checked
saved-over-body = “{ $original }” is unchanged. You're now editing the other save.
duplicate-tooltip = Duplicate…
duplicate-cloud = On Steam Cloud the game only lists saves it wrote itself, so a copy made here wouldn't show in the game. Save to a new slot in the game, or copy the whole profile as a local one (Profile tools).
duplicate-loose = A save on its own has no profile to put a copy in.
duplicate-title = Duplicate “{ $name }”?
duplicate-body = A copy, “{ $copy }”, goes in a new slot of the profile. “{ $name }” stays as it is.
duplicate-confirm = Duplicate
duplicate-name = { $name } (copy)
duplicated-title = Duplicated as “{ $name }”
duplicated-body = “{ $original }” is unchanged.
duplicate-failed = Couldn't duplicate “{ $name }”
saved-title = Saved and checked “{ $name }”
saved-backup = The previous version is in Backups.
save-failed = Couldn't save “{ $name }”
save-profile-failed = The save was written, but the company name couldn't be: it's still waiting to be saved

## Warnings

newer-game-warning = This save is from a newer version of the game than this editor has been tested with. Everything the editor doesn't understand is kept as it is, but check your changes in the game, and keep the backup.
ats-warning = American Truck Simulator support is new. Its saves open and save safely, with a backup. Check your changes in the game.

## Undo and redo

undo = Undo
redo = Redo
save-tooltip = Review the changes and save (Ctrl+S)
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
new-job-open = Make a job…
new-job-title = Make a job
new-job-description = Puts a job on the freight market when you save: from a company that ships the cargo to one that takes it, on a trailer the game already carries that cargo on.
new-job-from = From
new-job-cargo = Cargo
new-job-to = To
new-job-trailer-label = Trailer
new-job-pick = Choose…
new-job-company = { $company }, { $city }
new-job-cargo-on = { $cargo } · { $body }
new-job-trailer = { $body }, { $chassis } · { $count ->
    [one] 1 unit
   *[other] { $count } units
}
new-job-add = Add the job
new-job-added = The job goes on the market when you save
new-job-incomplete = Choose where from, the cargo, where to and the trailer.
new-job-no-route = There's no offer between these cities to take the distance from, and the map isn't read yet.
new-job = New job
new-job-before = None
new-job-after = { $cargo }, { $from } → { $to }
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

detail-truck = Truck
detail-coach = Coach
detail-trailer = Trailer
detail-driver = Driver
detail-garage-in = Garage in { $city }
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
fleet-filter-yours = Yours
fleet-yours = Yours
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
page-game-settings = Game settings
settings-language = Language
settings-language-description = The app's language. The game's own names (cities, cargo, ranks) follow it when the game has them in that language.
settings-appearance = Appearance
settings-appearance-system = Same as the system
settings-appearance-light = Light
settings-appearance-dark = Dark
settings-game = Game folder
settings-game-description = Where { $game } is installed. The editor reads level tables, names and icons from it, and never changes anything there.
settings-game-found = Found automatically
settings-game-picked = Chosen by you
settings-game-not-found = Not found. Choose the folder the game is installed in.
settings-game-not-game = That folder doesn't hold the game: it has no def.scs.
settings-game-choose = Choose folder…
settings-game-automatic = Find it automatically
settings-backups = Backups
settings-backups-description = A backup of a slot is made every time you save it, kept here, outside the game's folders.
settings-backups-open = Open folder
game-settings = In the game
game-settings-description = Settings of the game itself, in its config.cfg. The file is backed up before each change.
game-settings-console = Developer console (open it in the game with ~)
game-settings-convoy = Convoys of up to { $count } trucks
game-settings-running = Close the game to change these: it writes its settings when it exits.
game-settings-none = The game's settings file wasn't found. Start the game once, then come back.
game-settings-failed = Couldn't change the game's settings: { $error }
settings-updates = Updates
settings-updates-description = The editor can ask GitHub whether a newer version is out. That's the only time it uses the internet, and nothing about you or your saves is sent. Updating is downloading the new version and replacing the .exe; your settings and backups stay.
updates-not-checked = Not checked yet.
updates-checking = Checking…
updates-up-to-date = You have the latest version ({ $version }).
updates-available = Version { $version } is available.
updates-download = Download…
updates-failed = Couldn't reach GitHub. Check the connection and try again.
updates-check-now = Check now
updates-check-on-start = Check when the editor starts
updates-banner = Version { $version } is available
settings-reset = Reset the editor
settings-reset-description = Puts the editor's settings back as they were the first time: language, appearance, the game folder, folders added on the start screen, and checking for updates. Your saves and backups stay as they are.
settings-reset-button = Reset settings…
settings-reset-confirm-title = Reset the editor's settings?
settings-reset-confirm = Reset
settings-about = About
settings-version = Version { $version }
settings-source = Source code on GitHub
licences-open = Open-source licences
licences-title = Open-source licences
licences-description = The editor is built with code from other projects, under their licences. First the code adapted for it, then every library inside the editor, with its licence (open one to read it).
settings-not-affiliated = Not affiliated with or endorsed by SCS Software.

## Command palette

palette-open = Commands…
palette-shortcut = Ctrl+K
settings-shortcut = Ctrl+,
palette-placeholder = Type a command or a page…
palette-group-pages = Go to
palette-group-actions = Actions
palette-group-app = Save and app
palette-go = { $page }
palette-max-truck-skills = Max all truck skills
palette-max-coach-skills = Max all coach skills
palette-all-adr = { $game ->
    [ats] All hazard classes
   *[other] All ADR classes
}
palette-repair-trucks = Repair all trucks
palette-repair-coaches = Repair all coaches
palette-repair-trailers = Repair all trailers
palette-refuel-all = Fill every tank
palette-save = Save…
palette-buy-all-garages = Buy every garage, all large
palette-upgrade-owned-garages = Upgrade owned garages to large
palette-discard = Discard changes
palette-theme = Switch light or dark mode
report-problem = Report a problem…
report-this = Report this problem
report-description = Opens a new issue on GitHub in your browser, with the editor's version filled in. Nothing is sent until you post it.
crash-title = The editor closed unexpectedly last time
crash-body = Something went wrong and it had to stop. Your saves weren't touched: nothing is written without a backup. Reporting it opens a GitHub issue with what happened, for you to check and post.
crash-dismiss = Dismiss
palette-report = Report a problem on GitHub

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
company-section = Your company
profile-name = Profile name
company-name = Company name
company-name-description = Your company's name and your profile's, for the whole profile: every save in it. They're kept in the profile file, which is backed up before it's written.
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
coach-services = Coach services
coach-services-description = Your coach career as the game's statistics show it. The game works these out from the services it logged, so passengers are spread over them.
coach-service-count = Services
coach-passengers = Passengers transported
coach-average-satisfaction = Average satisfaction
coach-best-satisfaction = Best satisfaction
coach-distance = Coach distance
coach-stops = Bus stops found
coach-stops-found = { $found } of { $count }
coach-perfect-satisfaction = Perfect satisfaction
coach-perfect-satisfaction-tooltip = Every logged service's passengers leave fully satisfied, so the average and best become 100 %.
coach-discover-stops = Discover every bus stop
coach-discover-stops-tooltip = Marks every bus stop on the map as found, as if you'd been to each.
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
adr = { $game ->
    [ats] Hazardous cargo
   *[other] ADR
}
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
map-agencies-reading = Which cities have a recruitment agency is read from the game's map, a moment after the rest of its data.
column-dealer = Truck dealer
column-recruitment = Recruitment agency
map-place-unlocked = Unlocked
map-place-locked = Not found yet
city-place-missing = None in this city
city-place-unknown = Not known
city-visit = Mark as visited
city-visit-tooltip = Mark the city as visited, as if you'd driven through it.
city-unlock = Unlock
city-unlock-tooltip = Unlock it, as if you'd driven past it.
city-unvisit = Mark as not visited
city-unvisit-tooltip = Mark the city as never visited, as on a new profile. The game counts it again the next time you drive through.
city-lock = Lock
city-lock-tooltip = Lock it again, as if you'd never driven past it.
city-open-garage = Open
city-show-on-map = Show on map

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
drivers-max-all-tooltip = { $game ->
    [ats] Every skill at its highest level and all hazard classes, for every hired driver.
   *[other] Every skill at its highest level and all ADR classes, for every hired driver.
}
drivers-maxed = { $count ->
    [one] 1 driver maxed out
   *[other] { $count } drivers maxed out
}
column-hometown = Hometown
column-location = Now in
column-experience = Experience
column-skills = Skills
column-adr = { $game ->
    [ats] Hazardous cargo
   *[other] ADR
}
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
fleet-none = No trucks of your own yet
fleet-none-body = Buy a truck at a dealer in the game, and it shows up here.
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
percent = { $amount }%
amount-with-unit = { $amount } { $unit }
unit-km = km
unit-mi = mi
unit-kg = kg
unit-lb = lb
unit-t = t
trucks-table = Vehicles
unknown-truck = Unknown model

# Moving trucks and drivers; the current job
move-garage-full = { $city } · full
move-what-truck = { $model } · { $plate }
move-what-driver = Driver from { $city }
driver-truck = Truck
driver-truck-none = No truck
driver-truck-open = Open the truck
garage-driver-open = Open the driver
move-hint = Pick another garage to move it there.
move-with-driver = Its driver moves with it.
move-with-truck = Their truck moves with them.
move-no-room = No other garage has a free slot. Buy or upgrade one under Garages.
move-is-players = You drive this one: it goes where you take it.
current-job = Current job
current-job-summary = { $cargo } to { $city }
current-job-no-deadline = No deadline
current-job-cancelled = Cancelled
current-job-deadline = Deadline
current-job-late = Late
current-job-remove-deadline = Remove the deadline
current-job-cancel = Cancel job
current-job-cancelled-note = Cancelled when you save, with no cancellation fee. Undo brings it back.
current-job-freight-market = From the freight market, in your truck with the company's trailer. Cancelling here costs no fee.
current-job-external = An external contract, in your truck with the company's trailer. Cancelling here costs no fee.
current-job-quick-job = A quick job in the company's truck. The editor can't cancel those yet: cancel it in the game.
current-job-own-trailer = A cargo market job in your own trailer. The editor can't cancel those yet: cancel it in the game.
current-job-special = A special transport. The editor can't cancel those: cancel it in the game.

# Removing from the company; roads
remove-truck = Remove truck
remove-trailer = Remove trailer
remove-driver = Remove driver
remove-garage = Remove garage
remove-tooltip = Takes it out of your company when you save. Unlike selling in the game, you get no money for it. Undo brings it back.
remove-why-yours = You drive this one: it stays.
remove-why-your-trailer = You're pulling this trailer: it stays.
remove-why-has-driver = A driver drives this truck. Remove the driver, or move them, first.
remove-why-in-use = This trailer is on the job in progress.
remove-why-headquarters = This is your headquarters. Make another garage the headquarters first.
remove-why-not-empty = Move or remove its trucks, drivers and trailers first.
remove-why-unknown = This save's lists aren't in a shape the editor knows (a newer game version?), so it won't guess.
removal-before = In your company
removal-after = Removed
change-career = Career
change-you = You
change-cities = Cities
change-wear = { $part } wear
change-on = On
changes-none = Nothing differs from the save on disk.
roads-what = Visited roads
roads-count = { $count ->
    [one] One road piece
   *[other] { $amount } road pieces
}
roads-none = None
map-forget-roads = Forget visited roads
map-forget-roads-tooltip = The map's discovered roads start over, as on a new profile. Cities you've visited stay visited.
map-discover-roads = Discover every road
map-discover-roads-tooltip = Marks every road and junction on the map as driven, yours and your map DLCs', read from the game's own map. Cities you haven't been to stay unvisited.
map-discover-roads-needs-game = Needs the game installed on this PC: its map says where the roads are.
map-discover-roads-confirm-title = Discover every road?
map-discover-roads-confirm-body = Every road and junction on the map, yours and your map DLCs', is marked as driven. The game may count these roads toward its achievements, such as Pathfinder (40,000 km of discovered roads), but whether it does, or when, isn't known. Undo takes it back until you save, and the save is backed up first.
map-view-driven = Driven
map-view-not-driven = Not driven yet
map-view-truck = Your truck
map-view-garages = Your garages
map-view-names = City names
map-view-hide = Click to hide
map-view-show = Click to show
map-view-drawing = Drawing the map…
map-view-needs-game = Needs the game installed on this PC: the map comes from its files.
map-view-hint = Scroll to zoom, drag to move, click to open
map-view-zoom-in = Zoom in
map-view-zoom-out = Zoom out
map-view-fit = Whole map
map-view-find = Find a city…

# Truck and trailer details
plate-colours = Plate colours
plate-colours-pick = Choose…
plate-world-of-trucks = Your truck always carries your World of Trucks plate: the game puts it on whichever truck is yours each time a save loads. Change it on World of Trucks: My page → Profile customization (Game default gives the plate back to the game).
plate-plain = The game's own plate
plate-black-on-yellow = Black on yellow
plate-black-on-white = Black on white
plate-white-on-black = White on black
plate-white-on-blue = White on blue
plate-white-on-red = White on red
plate-white-on-green = White on green
plate-gold-on-black = Gold on black
plate-custom = Other colours
truck-engine = Engine
truck-transmission = Transmission
truck-part-pick = Choose…
truck-part-loading = Reading the game…
paint = Paint
paint-job = Paint job
paint-job-pick = Choose a paint job
paint-base = Base colour
paint-colour-1 = First colour
paint-colour-2 = Second colour
paint-colour-3 = Third colour
paint-locked = This paint job's colours are set by the game.
saved-colours = Saved colours
saved-colours-count = { $count ->
    [0] None
    [one] 1 colour
   *[other] { $count } colours
}
saved-colours-open = Saved colours…
saved-colours-description = The colours saved in the game's paint screen, for the whole company: 8, each up to four colours (the game saves a paint job's colours together; a single colour uses the first). They show as quick picks in every vehicle's paint.
saved-colours-slot = Saved colour { $slot }
saved-colours-colour = Saved colour { $slot }, colour { $colour }
saved-colours-copy = Copy as text
saved-colours-copy-tooltip = Copies the colours as text to share: #RRGGBB for each saved colour, a set's colours joined by /.
saved-colours-copied = Colours copied
saved-colours-paste = Paste text
saved-colours-paste-tooltip = Takes colours shared as text (#RRGGBB, a set joined by /, - for an empty one) from the clipboard, from the first saved colour on.
saved-colours-not-colours = The clipboard doesn't hold colours as text: #RRGGBB for each saved colour, separated by spaces.
truck-badge = Side badge
truck-remove-badge = Remove it
truck-remove-badge-tooltip = Takes the engine badge off the sides of the cab (the badge at the front stays).
trailer-load = Load
trailer-cargo = Cargo
trailer-empty-weight = Empty weight
trailer-countries = { $game ->
    [ats] States
   *[other] Countries
}
trailer-everywhere = Everywhere
trailer-allow-everywhere = Allow everywhere
edit-restricted = { $game ->
    [ats] Some states only
   *[other] Some countries only
}
edit-unrestricted = Everywhere
edit-fuel = { $percent } % fuel
edit-badge = With the side badge
edit-no-badge = No side badge
edit-plate = { $plate } · { $colours }
edit-wear = { $part } { $percent } % worn

# Profile tools
profile-tools-open = Profile tools…
profile-tools-title = Profile “{ $name }”
profile-tools-backup = Back up the profile
profile-tools-backup-description = A copy of the whole profile (every save, its photos and settings), kept with the other backups.
profile-tools-backup-now = Back up now
profile-tools-backed-up = Backed up ({ $size } MB).
profile-tools-copy = Make a copy
profile-tools-copy-description = A second profile with everything in this one, under a new name. It's made on this PC (Steam Cloud only shows profiles the game made), and without your World of Trucks login: link it in the game if you want to.
profile-tools-copy-suggestion = { $name } copy
profile-tools-copy-now = Make the copy
profile-tools-copied = Made “{ $name }”. It's in the game's profile list.
profile-tools-copy-name-empty = Give the copy a name.
profile-tools-copy-name-taken = There's a profile with that name already.
profile-tools-copy-no-documents = The game's folder in Documents wasn't found. Start the game once, then try again.
profile-tools-settings = Copy the settings
profile-tools-settings-description = This profile's graphics, controls and gameplay options, over another profile's. The other profile's own settings are backed up first.
profile-tools-settings-pick = Choose a profile…
profile-tools-settings-now = Copy the settings
profile-tools-settings-copied = Copied to “{ $name }”.
profile-tools-settings-none = There's no other profile with settings to copy to.
profile-tools-running = Close the game first: it writes profiles when it exits.
profile-tools-failed = That didn't work: { $error }

# The player's truck and position
player-truck = Your truck
player-truck-yours = You drive this one
player-drive = Drive it yourself
player-drive-tooltip = Makes it your truck: whoever drove it gets yours, and its garage becomes your headquarters. It's at your position when you load the save.
player-why-on-job = Finish or cancel the job in progress first.
player-why-not-in-garage = Only trucks in a garage can be yours.
player-position = Your position
player-position-somewhere = Somewhere on the map
player-position-from = { $place }, as in “{ $save }”
player-position-now = Your truck is in { $place }.
player-quick-travel = Quick travel when the save loads
player-quick-travel-tooltip = When the save loads, the game says “Game change detected” and lets you pick any city you've visited for your truck (and coach), for free. It's off during a job, so nothing is cancelled.
player-quick-travel-asked = When the save loads, the game says “Game change detected” and lets you pick a city for your truck (and coach), for free. Undo takes it back.
player-quick-travel-after = Quick travel when the save loads
player-quick-travel-unavailable = This save can't ask the game for a quick travel (an older game version).
player-why-no-truck = You don't have a truck of your own yet: quick travel moves your own vehicles.
player-move = Where another save is…
player-move-pick = Choose a save…
player-move-description = Your truck moves to where it stands in another save of this profile.
player-move-reading = Reading “{ $save }”…
player-move-no-position = “{ $save }” doesn't say where your truck is.
player-move-hitched = Unhitch your trailer in the game first: it would need a place too.

# Your day
day-section = Your day
day-description = How long you've driven, your record with the police, and the clock. Changes here are written when you save, like the rest.
day-since-break = Since your last break
day-rest = Rest
day-rest-tooltip = As if you'd slept: no tiredness and no break due.
day-rest-what = Tiredness
day-rested = Rested
day-record = Police record
day-record-summary = { $count ->
    [0] Clean
    [one] One offence, { $amount } in fines
   *[other] { $count } offences, { $amount } in fines
}
day-record-clear-button = Clear the record
day-record-tooltip = Removes every offence from the record. Fines already paid stay paid.
day-record-clear = Clean
day-cargo-damage = Cargo damage
day-repair-cargo = Repair the cargo
day-time = Time
day-time-on-job = Moving the clock would make your job late.
day-skip-morning = Morning (08:00)
day-skip-noon = Noon (12:00)
day-skip-evening = Evening (18:00)
day-skip-night = Night (22:00)
day-and-time = { $day } { $time }
day-monday = Monday
day-tuesday = Tuesday
day-wednesday = Wednesday
day-thursday = Thursday
day-friday = Friday
day-saturday = Saturday
day-sunday = Sunday
duration-minutes = { $minutes } min
duration-hours = { $hours } h { $minutes } min
player-trailer = Your trailer
player-trailer-none = None
player-trailer-yours = You pull this one
player-pull = Pull it yourself
player-pull-tooltip = Makes it the trailer you pull. It isn't hitched: hitch it in the game.

# Picking rows
picks-count = { $count ->
    [one] One picked
   *[other] { $count } picked
}
picks-max = Max their skills
picks-large = Make them large
picks-move = Move to garage…
picks-refuel = Fill the tanks
picks-remove = Remove
picks-moved = Moved { $moved } of { $count }: the others didn't fit there.
picks-removed = Removed { $removed } of { $count }: the rest can't be (open one to see why).
picks-moved-all = { $count ->
    [one] Moved.
   *[other] Moved all { $count }.
}
picks-removed-all = { $count ->
    [one] Removed.
   *[other] Removed all { $count }.
}
picks-clear = Clear
picks-done-repair = { $count ->
    [one] Repaired 1 vehicle
   *[other] Repaired { $count } vehicles
}
picks-done-refuel = { $count ->
    [one] Filled 1 tank
   *[other] Filled { $count } tanks
}
picks-done-max = { $count ->
    [one] Maxed 1 driver's skills
   *[other] Maxed { $count } drivers' skills
}
picks-done-large = { $count ->
    [one] Made 1 garage large
   *[other] Made { $count } garages large
}
picks-done-body = Undo takes it back. Nothing is written until you save.
picks-all = Pick every row shown
picks-row = Pick { $name }

## Statistics
page-statistics = Statistics
page-coach-career = Coach career
stats-deliveries = Deliveries
stats-income = Earned from deliveries
stats-xp = XP from deliveries
stats-driven = Driven in all
stats-days = Days on the road
stats-average = Average per delivery
stats-favourite = Hauled most
stats-favourite-value = { $cargo } ({ $count })
stats-cancelled = Jobs cancelled
stats-note = From the game's delivery log. Read-only: the game keeps these numbers itself.
stats-search = Search by cargo or city
stats-filter-damaged = Damaged cargo
stats-table = Deliveries
stats-none = No deliveries yet
stats-none-body = Each cargo you deliver shows here, newest first.
stats-when = { $days ->
    [0] Today
    [1] Yesterday
   *[other] { $days } days ago
}
column-when = When
column-income = Income
column-xp = XP
column-damage = Damage
column-passengers = Passengers
column-satisfaction = Satisfaction
coach-earned = Earned from services
coach-log-note = From the game's coach job log. Read-only: the game keeps these numbers itself.
coach-log-search = Search by city
coach-log-table = Coach services
coach-log-none = No coach services yet
coach-log-none-body = Each coach service you drive shows here, newest first.

## What the save needs
needs = DLC and mods
needs-description = { $mods ->
    [0] This save was made with { $dlcs } DLCs. The game loads it only with the ones it needs installed.
   *[other] This save was made with { $dlcs } DLCs and { $mods } mods. The game loads it only with the ones it needs installed.
}
needs-unknown = Can't check without the game. Pick its folder in Settings.
needs-all-here = Everything it was made with is installed.
needs-missing = { $count ->
    [one] One missing that the save needs. Install it, or the game won't load this save:
   *[other] { $count } missing that the save needs. Install them, or the game won't load this save:
}
needs-missing-extras = { $count ->
    [one] One extra missing (the save loads without it):
   *[other] { $count } extras missing (the save loads without them):
}
needs-mods = Mods
needs-installed = Installed
needs-not-installed = Not installed

## Gallery: viewpoints and photo trophies (the game's words)

gallery-viewpoint = Viewpoint
gallery-viewpoints = Viewpoints
gallery-photo-trophy = Photo trophy
gallery-photo-trophies = Photo trophies
gallery-visited = Visited
gallery-found = Found, not watched
gallery-not-found = Not found yet
gallery-captured = Captured
gallery-not-captured = Not captured yet
gallery-visited-of = { $visited } of { $count } visited
gallery-viewpoints-summary = { $visited } of { $count } viewpoints visited
gallery-photo-trophies-summary = { $captured } of { $count } photo trophies captured
gallery-where-kept = Viewpoints are kept in this save, so each save has its own. Photo trophies are kept in the profile, shared by all its saves.
gallery-column-name = Name
gallery-column-kind = Kind
gallery-column-country = { $game ->
    [ats] State
   *[other] Country
}
gallery-column-progress = Progress
gallery-filter-viewpoints = Viewpoints
gallery-filter-photo-trophies = Photo trophies
gallery-filter-to-do = Not done yet
gallery-search = { $game ->
    [ats] Search by name or state
   *[other] Search by name or country
}
gallery-table = Viewpoints and photo trophies
gallery-visit-all = Visit every viewpoint
gallery-visit-all-tooltip = Mark every viewpoint as visited, as if you'd watched each. The Gallery can play them all then.
gallery-visit = Mark as visited
gallery-visit-tooltip = As if you'd watched it: it counts as visited, and the Gallery can play it.
gallery-forget = Mark as not found
gallery-forget-tooltip = As if you'd never been there: the game finds it again when you drive past.
gallery-needs-game = The Gallery needs the game installed on this PC: its data lists the viewpoints and photo trophies.
gallery-photo-trophy-note = Photo trophies are your own photos, kept with the profile, so the editor only shows them: capture one in game by taking a photo of it.
gallery-map-not-read = The game's map is still being read, or wasn't found.
gallery-map-hover = { $name }: { $progress }
