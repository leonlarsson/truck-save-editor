# Svensk text för Truck Save Editor.
#
# Fluent-format: https://projectfluent.org/fluent/guide/
# Samma ord för samma sak överallt. Siffror i $count väljer pluralform; belopp kommer färdigformaterade.
# Varje meddelande i en.ftl finns här, med samma variabler (ett test kontrollerar det).

## Start screen

start-choose = Välj en sparfil att redigera
start-looking = Letar efter dina sparfiler…
start-none-title = Inga sparfiler hittades
start-none-body = Sparfiler letas efter i Steam Cloud och i Dokument\Euro Truck Simulator 2\profiles. Spara en gång i spelet och leta sedan igen.
start-look-again = Leta igen
start-your-saves = Dina sparfiler
start-game-saves = Sparade av spelet
start-slot-number = Plats { $number }
profile-place = { $name } · { $place }
profile-game-place = { $name } · { $game } · { $place }
location-steam-cloud = Steam Cloud
location-this-pc = Den här datorn

## Save slot names (for saves without a name of their own)

slot-unnamed = Namnlös sparfil
slot-autosave = Autosparning
slot-autosave-drive = Autosparning under körning
slot-autosave-job = Autosparning vid uppdrag
slot-quicksave = Snabbsparning
slot-multiplayer-backup = Före konvoj

## How long ago

time-just-now = nyss
time-minutes-ago = { $count ->
    [one] för 1 minut sedan
   *[other] för { $count } minuter sedan
}
time-hours-ago = { $count ->
    [one] för 1 timme sedan
   *[other] för { $count } timmar sedan
}
time-days-ago = { $count ->
    [one] för 1 dag sedan
   *[other] för { $count } dagar sedan
}
time-weeks-ago = { $count ->
    [one] för 1 vecka sedan
   *[other] för { $count } veckor sedan
}
time-months-ago = { $count ->
    [one] för 1 månad sedan
   *[other] för { $count } månader sedan
}
time-years-ago = { $count ->
    [one] för 1 år sedan
   *[other] för { $count } år sedan
}

## Sidebar and pages

sidebar-career = Karriär
sidebar-company = Företag
sidebar-advanced = Avancerat
page-overview = Översikt
page-fleet = Fordon
page-garages = Garage
headquarters = Huvudkontor
headquarters-pick = Välj en stad
page-drivers = Förare
page-map = Karta
page-all-data = All data
switch-save = Byt sparfil
theme-light = Ljust läge
theme-dark = Mörkt läge

## Header

header-subtitle = { $game } · { $place }
changes-count = { $count ->
    [0] Inga ändringar
    [one] 1 ändring
   *[other] { $count } ändringar
}
discard = Kasta bort
save = Spara
save-open = Spara…
details-open = Detaljer…
settings-open = Inställningar…
saving = Sparar…

## Opening, closing and saving

opening = Öppnar ”{ $name }”…
open-failed = Kunde inte öppna ”{ $name }”
close-unsaved-title = Stäng ”{ $name }” utan att spara?
close-unsaved-body = { $count ->
    [one] Din ändring går förlorad.
   *[other] Dina { $count } ändringar går förlorade.
}
close-unsaved-confirm = Stäng utan att spara
quit-unsaved-title = Avsluta utan att spara ”{ $name }”?
quit-unsaved-confirm = Avsluta utan att spara
save-confirm-title = Spara ändringarna i ”{ $name }”?
cancel = Avbryt
save-as-new-slot = Spara på ny plats
save-as-new-slot-tooltip = Skriv ändringarna till en ny plats och låt den här vara som den är.
new-slot-name = { $name } (redigerad)
saved-as-new-title = Sparad och kontrollerad som ”{ $name }”
saved-as-new-body = ”{ $original }” är oförändrad. Du redigerar nu den nya platsen.
save-confirm-note = Först görs en säkerhetskopia av platsen, och efter skrivningen läses sparfilen in igen och kontrolleras. Stäng spelet innan du sparar: det skriver över sparfiler när det avslutas.
save-confirm-cloud = Den här profilen ligger i Steam Cloud. Om Steam frågar vilken kopia som ska behållas nästa gång spelet startar, välj den på den här datorn. Spara som ny plats är avstängt: spelet visar bara sparfiler som Steam känner till, och det är de som spelet har gjort. För att behålla den här sparfilen som den är, gör en ny sparfil i spelet och redigera den.
save-prepare-failed = Kunde inte förbereda sparningen
save-no-backup-folder-title = Kunde inte spara
save-no-backup-folder = Det finns ingen mapp att lägga säkerhetskopior i, så ingenting skrevs.
saved-title = Sparad och kontrollerad ”{ $name }”
saved-backup = Den förra versionen finns under Säkerhetskopior.
save-failed = Kunde inte spara ”{ $name }”
save-profile-failed = Sparfilen skrevs, men inte företagsnamnet: det väntar fortfarande på att sparas

## Warnings

newer-game-warning = Den här sparfilen kommer från en nyare version av spelet än redigeraren har testats med. Allt som redigeraren inte förstår lämnas orört, men kontrollera dina ändringar i spelet och behåll säkerhetskopian.

## Undo and redo

undo = Ångra
redo = Gör om
save-tooltip = Granska ändringarna och spara (Ctrl+S)
undo-tooltip = Ångra (Ctrl+Z)
redo-tooltip = Gör om (Ctrl+Skift+Z)

## Jobs

page-jobs = Uppdrag
jobs-summary = { $count ->
    [one] 1 uppdrag
   *[other] { $amount } uppdrag
}
jobs-offers = Uppdrag
jobs-keep = Behåll alla uppdrag en vecka
jobs-keep-tooltip = Låt varje uppdrag finnas kvar minst en vecka speltid, så att det du vill ha fortfarande finns där.
jobs-kept-week = Kvar i minst en vecka
jobs-table = Uppdrag
jobs-search = Sök på stad, företag eller last
jobs-filter-my-garages = Från mina garage
jobs-filter-urgent = Brådskande
jobs-filter-no-ferry = Utan färja
column-from = Från
column-to = Till
column-cargo = Last
column-distance = Sträcka
column-expires = Går ut
column-urgency = Brådska
job-place = { $company }, { $city }
job-urgency-standard = Normal
job-urgency-important = Viktig
job-urgency-urgent = Brådskande
job-expires-in = { $minutes ->
    [one] om 1 minut
   *[other] om { $minutes } minuter
}
job-expires-in-hours = { $hours ->
    [one] om 1 timme
   *[other] om { $hours } timmar
}
job-expires-in-days = { $days ->
    [one] om 1 dag
   *[other] om { $days } dagar
}
job-expired = Utgånget
job-ferry = { $distance } · färja

## Detail panels

detail-truck = Lastbil
detail-coach = Buss
detail-trailer = Släp
detail-driver = Förare
detail-garage-in = Garage i { $city }
detail-about = Om
detail-gone = Det här finns inte längre i sparfilen.
part-engine = Motor
part-transmission = Växellåda
part-cabin = Hytt
part-chassis = Chassi
part-body = Skåp
part-wheels = Hjul
repair-all-parts = Reparera allt
fill-tank = Fyll tanken
driver-from = Från { $city }
driver-now-in = Nu i { $city }
driver-max = Maxa föraren
garage-empty = Inga
garage-make-large = Gör det stort
garage-buy-large = Köp det, stort

## Filters

filter-count = { $shown } av { $total }
filter-no-matches = Inget matchar
filter-no-matches-body = Prova en annan sökning, eller rensa filtren.
drivers-none = Inga förare anställda än
drivers-none-body = Anställ förare på en rekryteringsbyrå i spelet, så syns de här.
jobs-none = Inga uppdrag
jobs-none-body = Spelet fyller uppdragsmarknaden medan du kör; den här sparfilen har inga än.
filter-clear = Rensa filter
fleet-search = Sök på modell, registreringsnummer eller garage
fleet-filter-worn = Behöver reparation
fleet-filter-no-garage = Utan garage
garages-search = Sök på stad
garages-filter-owned = Egna
garages-filter-not-large = Inte stora
drivers-search = Sök på stad
drivers-filter-not-maxed = Inte maxade
map-search = Sök på stad
map-filter-not-visited = Inte besökta
map-filter-dealer-locked = Handlare inte hittad
map-filter-recruitment-locked = Byrå inte hittad

## Quick start

quick-start = Snabbstart
quick-start-open = Snabbstart…
quick-start-description = De vanliga ändringarna för en ny karriär, på en gång. Starta en ny profil i spelet, öppna dess sparfil här och välj vad du vill ha. En Ångra tar tillbaka allt, och ingenting skrivs förrän du sparar.
quick-start-rich = Rik start
quick-start-everything = Allt upplåst
quick-start-money = Sätt pengar
quick-start-hq-garage = Ett stort garage i { $city }
quick-start-apply = Verkställ
quick-start-no-coaches = Den här sparfilen har ingen busskarriär (Coaches-DLC).
quick-start-no-hq = Den här sparfilen har ingen huvudkontorsstad än.
quick-start-hq-large = Garaget vid ditt huvudkontor är redan stort.
quick-start-needs-game = Kräver att spelet är installerat på den här datorn: dess data berättar vilka städer som har dem.

## Settings

page-settings = Inställningar
settings-language = Språk
settings-language-description = Appens språk. Spelets egna namn (städer, last, grader) följer med när spelet har dem på det språket.
settings-appearance = Utseende
settings-appearance-system = Samma som systemet
settings-appearance-light = Ljust
settings-appearance-dark = Mörkt
settings-game = Spelets mapp
settings-game-description = Där Euro Truck Simulator 2 är installerat. Redigeraren läser nivåtabeller, namn och ikoner därifrån och ändrar aldrig något där.
settings-game-found = Hittades automatiskt
settings-game-picked = Vald av dig
settings-game-not-found = Hittades inte. Välj mappen där spelet är installerat.
settings-game-not-game = Den mappen innehåller inte spelet: den har ingen def.scs.
settings-game-choose = Välj mapp…
settings-game-automatic = Hitta den automatiskt
settings-backups = Säkerhetskopior
settings-backups-description = En säkerhetskopia av en plats görs varje gång du sparar den, och läggs här, utanför spelets mappar.
settings-backups-open = Öppna mappen
settings-about = Om
settings-version = Version { $version }
settings-source = Källkod på GitHub
settings-not-affiliated = Inte knuten till eller godkänd av SCS Software.

## Command palette

palette-open = Kommandon…
palette-shortcut = Ctrl+K
palette-placeholder = Skriv ett kommando eller en sida…
palette-group-pages = Gå till
palette-group-actions = Åtgärder
palette-group-app = Spara och appen
palette-go = { $page }
palette-max-truck-skills = Maxa alla lastbilsfärdigheter
palette-max-coach-skills = Maxa alla bussfärdigheter
palette-all-adr = Alla ADR-klasser
palette-repair-trucks = Reparera alla lastbilar
palette-repair-coaches = Reparera alla bussar
palette-repair-trailers = Reparera alla släp
palette-refuel-all = Fyll alla tankar
palette-save = Spara…
palette-buy-all-garages = Köp alla garage, alla stora
palette-upgrade-owned-garages = Gör egna garage stora
palette-discard = Kasta bort ändringarna
palette-theme = Byt mellan ljust och mörkt läge

## Backups

backups = Säkerhetskopior
backups-intro = En säkerhetskopia av platsen görs varje gång du sparar, och läggs utanför spelets mappar. Att återställa en säkerhetskopierar först det som finns där nu, så det går att ångra också.
backups-none = Inga säkerhetskopior än
backups-none-body = En görs första gången du sparar den här platsen.
backup-made-at = Gjord { $time } (UTC)
backup-show = Visa i mappen
backup-restore = Återställ
restore-title = Återställa ”{ $name }” från den här säkerhetskopian?
restore-body = { $changes ->
    [0] Det som finns på platsen nu säkerhetskopieras först.
    [one] Det som finns på platsen nu säkerhetskopieras först. Din osparade ändring går förlorad.
   *[other] Det som finns på platsen nu säkerhetskopieras först. Dina { $changes } osparade ändringar går förlorade.
}
restored-title = Återställde ”{ $name }”
restored-body = Platsen är öppen igen som den var i säkerhetskopian.
restore-failed = Kunde inte återställa ”{ $name }”

## Overview

amount-hint = Prova 2,5m, 10k, +1m eller -500k.
company-section = Ditt företag
company-name = Företagsnamn
company-name-description = Ditt företags namn, för hela profilen: varje sparfil i den. Det ligger i profilfilen, som säkerhetskopieras innan den skrivs.
money = Pengar
money-description = Företagets saldo på banken.
money-label = Banksaldo
experience = Lastbilserfarenhet
experience-description = Din nivå som lastbilschaufför.
experience-label = Erfarenhetspoäng som lastbilschaufför
coach-experience = Busserfarenhet
coach-experience-description = Bussarna är en egen karriär.
coach-experience-label = Erfarenhetspoäng som busschaufför
experience-unit = XP
experience-amount = { $amount } XP
skills = Lastbilsfärdigheter
level-number = Nivå { $level }
level-progress = { $percent } % till nivå { $next }
level-progress-label = Framsteg mot nästa nivå
level-step = { $count ->
    [one] 1 nivå
   *[other] { $count } nivåer
}
coach-skills = Bussfärdigheter
coach-skills-description = Varje bussfärdighet går upp till nivå 3.
coach-skill-longer-duties = Längre turer
coach-skill-longer-duties-description = Fler hållplatser per tur
coach-skill-demanding-timetables = Krävande tidtabeller
coach-skill-demanding-timetables-description = Snävare tidtabeller som ger mer XP
coach-skill-premium-service = Premiumservice
coach-skill-premium-service-description = Passagerare som betalar mer och förlåter mindre
coach-skill-cushioned-journey = Mjuk resa
coach-skill-cushioned-journey-description = Mjuk körning ger mer betalt och mer XP
coach-skill-passenger-favourite = Passagerarnas favorit
coach-skill-passenger-favourite-description = Felfri körning höjer nöjdheten
coach-skill-ecodriving = Sparsam körning
coach-skill-ecodriving-description = Drar mindre bränsle
skills-description = Klicka på en nivå för att välja den. Klicka på den högsta ifyllda nivån igen för att sänka den.
skills-max = Maxa alla färdigheter
skill-level = Nivå { $level }
skill-level-of = { $skill }: nivå { $level } av { $max }
skill-long-distance = Långdistans
skill-long-distance-description = Låser upp uppdrag på längre sträckor
skill-high-value-cargo = Värdefull last
skill-high-value-cargo-description = Dyrbara laster som betalar mer
skill-fragile-cargo = Ömtålig last
skill-fragile-cargo-description = Laster som lätt går sönder
skill-just-in-time = Just-in-time-leveranser
skill-just-in-time-description = Brådskande uppdrag med snäva tidsgränser
skill-ecodriving = Sparsam körning
skill-ecodriving-description = Drar mindre bränsle
adr = ADR
adr-description = Farligt gods din förare får köra.
adr-all = Alla klasser
adr-none = Inga
adr-some = { $count } av { $total } klasser
adr-explosives = Explosiva ämnen
adr-gases = Gaser
adr-flammable-liquids = Brandfarliga vätskor
adr-flammable-solids = Brandfarliga fasta ämnen
adr-toxic = Giftiga ämnen
adr-corrosive = Frätande ämnen
hq = Huvudkontor

## All data

all-data-intro = Varje enhet i sparfilen och varje värde i den. Välj en sort, sedan en enhet, sedan ett värde att ändra.
all-data-search = Sök i alla värden, fält och enheter
all-data-search-count = { $count ->
    [0] Inget matchar
    [one] 1 träff
   *[other] { $amount } träffar
}
all-data-search-more = De första { $count } träffarna
all-data-results = Sökresultat
all-data-kinds = Sorter
all-data-units = Enheter
all-data-values = Värden
all-data-filter = Filtrera sorter
all-data-kind = Sort
all-data-count = Enheter
all-data-unit = Enhet
all-data-field = Fält
all-data-value = Värde
all-data-items = { $count ->
    [one] 1 post
   *[other] { $count } poster
}
all-data-pick-value = Välj ett värde
all-data-apply = Verkställ
all-data-revert = Återställ
all-data-wrong-shape = Det här värdet rymmer { $expected }; skriv ett sådant (text behåller sina citattecken).
shape-bool = true eller false
shape-number = ett tal
shape-text = text inom citattecken
shape-id = ett id eller namn
shape-vector = tal inom parentes
shape-value = ett värde

## Map

map-summary = { $visited } av { $count } städer besökta
map-visit-all = Besök alla städer
map-visit-all-tooltip = Markera varje stad som besökt, som om du hade kört genom alla.
map-visited-all = { $count ->
    [one] 1 stad till besökt
   *[other] { $count } städer till besökta
}
map-visits = { $count ->
    [0] Inte besökt
    [one] En gång
   *[other] { $count } gånger
}
column-visits = Besökt
map-table = Städer
map-dealers = Lastbilshandlare
map-recruitments = Rekryteringsbyråer
map-dealers-summary = { $unlocked } av { $count ->
    [one] 1 lastbilshandlare
   *[other] { $count } lastbilshandlare
}
map-recruitments-summary = { $unlocked } av { $count ->
    [one] 1 rekryteringsbyrå
   *[other] { $count } rekryteringsbyråer
}
map-unlocked-count = { $count } upplåsta
map-unlocked-of = { $unlocked } av { $count } upplåsta
map-unlock-dealers = Lås upp alla handlare
map-unlock-dealers-tooltip = Lås upp lastbilshandlaren i varje stad som har en, som om du hade kört förbi alla.
map-unlock-recruitments = Lås upp alla byråer
map-unlock-recruitments-tooltip = Lås upp rekryteringsbyrån i varje stad som har en, som om du hade kört förbi alla.
map-needs-game = För att låsa upp handlare och byråer måste spelet vara installerat på den här datorn: dess data berättar vilka städer som har dem.
column-dealer = Lastbilshandlare
column-recruitment = Rekryteringsbyrå
map-place-unlocked = Upplåst
map-place-locked = Inte hittad än

## Garages

garages = Garage
garages-before = Som de var
garages-after = { $bought ->
    [0] { $upgraded ->
        [one] 1 gjort stort
       *[other] { $upgraded } gjorda stora
    }
   *[other] { $upgraded ->
        [0] { $bought } köpta, alla stora
       *[other] { $bought } köpta och { $upgraded } gjorda stora, alla stora
    }
}
garages-summary = { $count ->
    [one] 1 stad
   *[other] { $count } städer
} · { $owned } egna · { $large } stora
garages-upgrade-owned = Gör egna stora
garages-upgrade-owned-tooltip = Gör varje garage du äger stort: 5 lastbilar och 5 förare.
garages-buy-all = Köp alla
garages-buy-all-tooltip = Köp garaget i varje stad, alla stora.
garage-not-owned = Inte eget
garage-large = Stort
garage-slots = { $count ->
    [one] 1 plats
   *[other] { $count } platser
}
garage-filled = { $filled } av { $slots }
column-city = Stad
column-garage-size = Garage
column-trucks = Lastbilar
column-drivers = Förare
garages-table = Garage

## Drivers

drivers-summary = { $count ->
    [0] Inga förare anställda än
    [one] 1 förare
   *[other] { $count } förare
} · { $maxed ->
    [0] ingen fullt utbildad
   *[other] { $maxed } fullt utbildade
}
drivers-max-all = Maxa alla förare
drivers-max-all-tooltip = Varje färdighet på högsta nivån och alla ADR-klasser, för varje anställd förare.
drivers-maxed = { $count ->
    [one] 1 förare maxad
   *[other] { $count } förare maxade
}
column-hometown = Hemstad
column-location = Nu i
column-experience = Erfarenhet
column-skills = Färdigheter
column-adr = ADR
drivers-table = Förare
skills-short = { $long } · { $value } · { $fragile } · { $urgent } · { $eco }
adr-count = { $count } av { $total }

## Fleet

fleet-trucks = { $count ->
    [one] 1 lastbil
   *[other] { $amount } lastbilar
}
fleet-worn = { $count ->
    [0] alla i gott skick
    [one] 1 behöver reparation
   *[other] { $amount } behöver reparation
}
fleet-coaches = { $count ->
    [one] 1 buss
   *[other] { $amount } bussar
}
fleet-tab-trucks = Lastbilar · { $count }
fleet-tab-coaches = Bussar · { $count }
fleet-summary = { $vehicles } · { $worn }
repair-all = Reparera alla
repairs = Reparationer
repairs-before = Slitna
repairs-after = { $count ->
    [one] 1 fordon reparerat
   *[other] { $count } fordon reparerade
}
column-truck = Modell
column-plate = Reg.nr
column-garage = Garage
column-odometer = Mätarställning
column-wear = Slitage
repair = Reparera
refuel-all = Fyll alla tankar
refuel-all-tooltip = Fyll bränsletanken i varje lastbil eller buss i den här listan.
column-fuel = Bränsle
vehicle-in-use = Körs nu
vehicle-parked = Parkerad i { $city }
fuel = Bränsle
fuel-before = Som det var
fuel-after = { $count ->
    [one] 1 tank fylld
   *[other] { $count } tankar fyllda
}
fleet-trailers = { $count ->
    [one] 1 släp
   *[other] { $amount } släp
}
fleet-tab-trailers = Släp · { $count }
no-garage = Utan garage
distance-km = { $amount } km
percent = { $amount } %
trucks-table = Fordon
unknown-truck = Okänd modell

# Moving trucks and drivers; the current job
moves-what = Flyttar mellan garage
moves-after = { $count ->
    [one] En flytt
   *[other] { $count } flyttar
}
move-garage-full = { $city } · fullt
move-hint = Välj ett annat garage för att flytta dit.
move-with-driver = Föraren följer med.
move-with-truck = Lastbilen följer med.
move-no-room = Inget annat garage har en ledig plats. Köp eller bygg ut ett under Garage.
move-is-players = Det här är din egen: den står där du kör den.
current-job = Pågående uppdrag
current-job-summary = { $cargo } till { $city }
current-job-no-deadline = Ingen tidsgräns
current-job-cancelled = Avbrutet
current-job-deadline = Tidsgräns
current-job-late = Försenad
current-job-remove-deadline = Ta bort tidsgränsen
current-job-cancel = Avbryt uppdraget
current-job-cancelled-note = Avbryts när du sparar, utan avgift. Ångra tar tillbaka det.
current-job-freight-market = Från fraktmarknaden, i din lastbil med företagets trailer. Att avbryta här kostar ingen avgift.
current-job-external = Ett externt kontrakt, i din lastbil med företagets trailer. Att avbryta här kostar ingen avgift.
current-job-quick-job = Ett snabbuppdrag i företagets lastbil. Redigeraren kan inte avbryta sådana än: avbryt det i spelet.
current-job-own-trailer = Ett uppdrag från lastmarknaden i din egen trailer. Redigeraren kan inte avbryta sådana än: avbryt det i spelet.
current-job-special = En specialtransport. Redigeraren kan inte avbryta sådana: avbryt den i spelet.
