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
page-drivers = Drivers
page-map = Map
page-all-data = All data
page-not-built = { $page } isn't built yet
page-not-built-body = It's on the way.
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
save-confirm-note = A backup of the slot is made first. Close the game before saving: it overwrites saves when it exits.
save-prepare-failed = Couldn't prepare the save
save-no-backup-folder-title = Couldn't save
save-no-backup-folder = There's no folder to keep backups in, so nothing was written.
saved-title = Saved “{ $name }”
saved-backup = The old version is backed up in { $path }
save-failed = Couldn't save “{ $name }”

## Overview

amount-hint = Try 2.5m, 10k, +1m or -500k.
money = Money
money-description = Your company's bank balance.
money-label = Bank balance
experience = Experience
experience-description = Experience points decide your level.
experience-label = Experience points
experience-unit = XP
experience-amount = { $amount } XP
bus-experience = Bus driving: { $amount } XP
skills = Skills
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
fleet-summary = { $trucks } · { $worn }
repair-all = Repair all
repairs = Repairs
repairs-before = Worn
repairs-after = { $count ->
    [one] 1 truck repaired
   *[other] { $count } trucks repaired
}
column-truck = Truck
column-plate = Plate
column-garage = Garage
column-odometer = Odometer
column-wear = Wear
no-garage = No garage
distance-km = { $amount } km
percent = { $amount }%
trucks-table = Trucks
unknown-truck = Unknown truck
