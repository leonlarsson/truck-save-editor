# Svensk text för Truck Save Editor.
#
# Fluent-format: https://projectfluent.org/fluent/guide/
# Samma ord för samma sak överallt. Siffror i $count väljer pluralform; belopp kommer färdigformaterade.
# Varje meddelande i en.ftl finns här, med samma variabler (ett test kontrollerar det).

## Start screen

start-choose = Välj en sparfil att redigera
start-looking = Letar efter dina sparfiler…
start-none-title = Inga sparfiler hittades
start-none-body = Ingen av platserna nedan har en profil för spelet. Om du spelar på den här datorn: starta spelet och spara en gång, så uppdateras listan när du kommer tillbaka till fönstret. Om dina sparfiler ligger någon annanstans, till exempel i en säkerhetskopia eller på en annan disk, lägg till den mappen.
start-look-again = Leta igen
start-your-saves = Dina sparfiler
start-game-saves = Sparade av spelet
start-game-saves-hint = Autosparningar och snabbsparningar. Spelet ersätter dem medan du spelar.
start-game-saves-more = { $count ->
    [one] Visa en till
   *[other] Visa { $count } till
}
start-game-saves-fewer = Visa färre
start-showing-cloud = { $count ->
    [one] En sparfil i ”{ $name }”, i Steam Cloud, så den är likadan på alla datorer du spelar på.
   *[other] { $count } sparfiler i ”{ $name }”, i Steam Cloud, så de är likadana på alla datorer du spelar på.
}
start-showing-local = { $count ->
    [one] En sparfil i ”{ $name }”, bara på den här datorn.
   *[other] { $count } sparfiler i ”{ $name }”, bara på den här datorn.
}
start-profile-empty-title = Inga sparfiler i ”{ $name }” än
start-profile-empty-body = Spara en gång i spelet så syns det här. Listan uppdateras när du kommer tillbaka till det här fönstret.
start-folders = Var sparfiler letas efter
start-folders-description = Spelet lägger varje profil i Steam Cloud eller på den här datorn, enligt profilens inställningar i spelet.
start-folders-missing = { $count ->
    [one] Letade också i en mapp som inte finns på den här datorn
   *[other] Letade också i { $count } mappar som inte finns på den här datorn
}
start-folders-missing-hide = Dölj mapparna som inte finns
start-folder-added = Tillagd av dig
start-folder-profiles = { $count ->
    [one] En profil
   *[other] { $count } profiler
}
start-folder-empty = Inga profiler
start-folder-missing = Finns inte
start-folder-no-steam = Steam hittades inte på den här datorn
start-folder-show = Visa mappen
start-folder-remove = Sluta leta här
start-folder-add = Lägg till en mapp…
start-folder-drop = Eller släpp en sparfils- eller profilmapp på fönstret.
start-folder-choose = Välj en mapp med sparfiler
start-folder-not-saves = Inga sparfiler eller profiler i { $path }. Välj spelets profiles-mapp, en profils mapp eller en sparfils.
start-slot-missing = { $count ->
    [one] Behöver en DLC eller modd som inte är installerad
   *[other] Behöver { $count } DLC:er eller moddar som inte är installerade
}
start-slot-number = Plats { $number }
profile-place = { $name } · { $place }
profile-game-place = { $name } · { $game } · { $place }
start-game-detail = { $profiles ->
    [one] En profil
   *[other] { $profiles } profiler
} · { $saves ->
    [one] en sparfil
   *[other] { $saves } sparfiler
}
start-game-none = Hittades inte på den här datorn
location-steam-cloud = Steam Cloud
location-this-pc = Den här datorn
location-loose = En ensam sparfil

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
sidebar-map = Karta
sidebar-advanced = Avancerat
page-overview = Översikt
page-fleet = Fordon
page-garages = Garage
headquarters = Huvudkontor
headquarters-pick = Välj en stad
page-drivers = Förare
page-map = Karta
page-cities = Städer
page-world-map = Världskarta
page-gallery = Galleri
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
game-settings-open = Spelinställningar…
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
save-as-new-slot-loose = Den här sparfilen ligger ensam, utanför en profil, så det finns ingen profil att lägga till en plats i.
new-slot-name = { $name } (redigerad)
saved-as-new-title = Sparad och kontrollerad som ”{ $name }”
saved-as-new-body = ”{ $original }” är oförändrad. Du redigerar nu den nya platsen.
save-confirm-note = Först görs en säkerhetskopia av platsen, och efter skrivningen läses sparfilen in igen och kontrolleras. Stäng spelet innan du sparar: det skriver över sparfiler när det avslutas.
save-confirm-cloud = Den här profilen ligger i Steam Cloud: om Steam frågar vilken kopia som ska behållas, välj den på den här datorn. Spara som ny plats är avstängt, eftersom spelet bara visar sparfiler som det har gjort själv. Spara över ett annat sparat spel för att behålla det här som det är.
save-prepare-failed = Kunde inte förbereda sparningen
save-no-backup-folder-title = Kunde inte spara
save-no-backup-folder = Det finns ingen mapp att lägga säkerhetskopior i, så ingenting skrevs.
save-over-other = Spara över ett annat…
save-over-other-tooltip = Skriv ändringarna över ett annat sparat spel i den här profilen och behåll det här som det är. Fungerar med Steam Cloud.
save-over-pick-title = Spara över vilket sparat spel?
save-over-pick-body = Det du väljer säkerhetskopieras och ersätts sedan av ”{ $original }” med dina ändringar. Det behåller sitt namn. ”{ $original }” förblir som det är.
save-over-confirm-title = Ersätta ”{ $name }”?
save-over-confirm-body = ”{ $name }” säkerhetskopieras först och ersätts sedan av ”{ $original }” med dina ändringar, under sitt eget namn. ”{ $original }” förblir som det är, och du fortsätter redigera ”{ $name }”.
save-over-confirm = Ersätt det
save-over-no-backups = Det finns ingenstans att lägga säkerhetskopian, så inget skrevs.
saved-over-title = Sparade över ”{ $name }” och kontrollerade
saved-over-body = ”{ $original }” är oförändrat. Du redigerar nu det andra sparade spelet.
duplicate-tooltip = Duplicera…
duplicate-cloud = På Steam Cloud visar spelet bara sparfiler som det själv har skrivit, så en kopia som görs här skulle inte synas i spelet. Spara på en ny plats i spelet, eller kopiera hela profilen som en lokal profil (Profilverktyg).
duplicate-loose = En sparfil för sig själv har ingen profil att lägga en kopia i.
duplicate-title = Duplicera ”{ $name }”?
duplicate-body = En kopia, ”{ $copy }”, hamnar på en ny plats i profilen. ”{ $name }” förblir som det är.
duplicate-confirm = Duplicera
duplicate-name = { $name } (kopia)
duplicated-title = Duplicerad som ”{ $name }”
duplicated-body = ”{ $original }” är oförändrad.
duplicate-failed = Kunde inte duplicera ”{ $name }”
saved-title = Sparad och kontrollerad ”{ $name }”
saved-backup = Den förra versionen finns under Säkerhetskopior.
save-failed = Kunde inte spara ”{ $name }”
save-profile-failed = Sparfilen skrevs, men inte företagsnamnet: det väntar fortfarande på att sparas

## Warnings

newer-game-warning = Den här sparfilen kommer från en nyare version av spelet än redigeraren har testats med. Allt som redigeraren inte förstår lämnas orört, men kontrollera dina ändringar i spelet och behåll säkerhetskopian.
ats-warning = Stödet för American Truck Simulator är nytt. Dess sparfiler öppnas och sparas säkert, med en säkerhetskopia. Kontrollera dina ändringar i spelet.

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
new-job-open = Skapa ett uppdrag…
new-job-title = Skapa ett uppdrag
new-job-description = Lägger ut ett uppdrag på fraktmarknaden när du sparar: från ett företag som skickar lasten till ett som tar emot den, på en trailer spelet redan kör lasten på.
new-job-from = Från
new-job-cargo = Last
new-job-to = Till
new-job-trailer-label = Trailer
new-job-pick = Välj…
new-job-company = { $company }, { $city }
new-job-cargo-on = { $cargo } · { $body }
new-job-trailer = { $body }, { $chassis } · { $count ->
    [one] 1 enhet
   *[other] { $count } enheter
}
new-job-add = Lägg till uppdraget
new-job-added = Uppdraget läggs ut på marknaden när du sparar
new-job-incomplete = Välj varifrån, last, vart och trailer.
new-job-no-route = Det finns inget erbjudande mellan städerna att ta sträckan från, och kartan är inte inläst än.
new-job = Nytt uppdrag
new-job-before = Inget
new-job-after = { $cargo }, { $from } → { $to }
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
fleet-filter-yours = Dina
fleet-yours = Din
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
page-game-settings = Spelinställningar
settings-language = Språk
settings-language-description = Appens språk. Spelets egna namn (städer, last, grader) följer med när spelet har dem på det språket.
settings-appearance = Utseende
settings-appearance-system = Samma som systemet
settings-appearance-light = Ljust
settings-appearance-dark = Mörkt
settings-game = Spelets mapp
settings-game-description = Där { $game } är installerat. Redigeraren läser nivåtabeller, namn och ikoner därifrån och ändrar aldrig något där.
settings-game-found = Hittades automatiskt
settings-game-picked = Vald av dig
settings-game-not-found = Hittades inte. Välj mappen där spelet är installerat.
settings-game-not-game = Den mappen innehåller inte spelet: den har ingen def.scs.
settings-game-choose = Välj mapp…
settings-game-automatic = Hitta den automatiskt
settings-backups = Säkerhetskopior
settings-backups-description = En säkerhetskopia av en plats görs varje gång du sparar den, och läggs här, utanför spelets mappar.
settings-backups-open = Öppna mappen
game-settings = I spelet
game-settings-description = Inställningar för själva spelet, i dess config.cfg. Filen säkerhetskopieras före varje ändring.
game-settings-console = Utvecklarkonsol (öppna den i spelet med ~)
game-settings-convoy = Konvojer med upp till { $count } lastbilar
game-settings-running = Stäng spelet för att ändra dessa: det skriver sina inställningar när det avslutas.
game-settings-none = Spelets inställningsfil hittades inte. Starta spelet en gång och kom sedan tillbaka.
game-settings-failed = Kunde inte ändra spelets inställningar: { $error }
settings-updates = Uppdateringar
settings-updates-description = Redigeraren kan fråga GitHub om en nyare version finns. Det är enda gången den använder internet, och inget om dig eller dina sparfiler skickas. Att uppdatera är att ladda ner den nya versionen och byta ut .exe-filen; dina inställningar och säkerhetskopior finns kvar.
updates-not-checked = Inte kontrollerat än.
updates-checking = Kontrollerar…
updates-up-to-date = Du har den senaste versionen ({ $version }).
updates-available = Version { $version } finns.
updates-download = Ladda ner…
updates-failed = Kunde inte nå GitHub. Kontrollera anslutningen och försök igen.
updates-check-now = Kontrollera nu
updates-check-on-start = Kontrollera när redigeraren startar
updates-banner = Version { $version } finns
settings-reset = Återställ redigeraren
settings-reset-description = Sätter tillbaka redigerarens inställningar som de var första gången: språk, utseende, spelmappen, mappar som lagts till på startskärmen och sökning efter uppdateringar. Dina sparfiler och säkerhetskopior lämnas som de är.
settings-reset-button = Återställ inställningarna…
settings-reset-confirm-title = Återställa redigerarens inställningar?
settings-reset-confirm = Återställ
settings-about = Om
settings-version = Version { $version }
settings-source = Källkod på GitHub
licences-open = Licenser för öppen källkod
licences-title = Licenser för öppen källkod
licences-description = Redigeraren är byggd med kod från andra projekt, under deras licenser. Först koden som anpassats för den, sedan varje bibliotek i redigeraren med sin licens (öppna en för att läsa den).
settings-not-affiliated = Inte knuten till eller godkänd av SCS Software.

## Command palette

palette-open = Kommandon…
palette-shortcut = Ctrl+K
settings-shortcut = Ctrl+,
palette-placeholder = Skriv ett kommando eller en sida…
palette-group-pages = Gå till
palette-group-actions = Åtgärder
palette-group-app = Spara och appen
palette-go = { $page }
palette-max-truck-skills = Maxa alla lastbilsfärdigheter
palette-max-coach-skills = Maxa alla bussfärdigheter
palette-all-adr = { $game ->
    [ats] Alla faroklasser
   *[other] Alla ADR-klasser
}
palette-repair-trucks = Reparera alla lastbilar
palette-repair-coaches = Reparera alla bussar
palette-repair-trailers = Reparera alla släp
palette-refuel-all = Fyll alla tankar
palette-save = Spara…
palette-buy-all-garages = Köp alla garage, alla stora
palette-upgrade-owned-garages = Gör egna garage stora
palette-discard = Kasta bort ändringarna
palette-theme = Byt mellan ljust och mörkt läge
report-problem = Rapportera ett problem…
report-this = Rapportera problemet
report-description = Öppnar ett nytt ärende på GitHub i webbläsaren, med redigerarens version ifylld. Inget skickas förrän du publicerar det.
crash-title = Redigeraren stängdes oväntat förra gången
crash-body = Något gick fel och den var tvungen att avslutas. Dina sparfiler rördes inte: inget skrivs utan en säkerhetskopia. En rapport öppnar ett GitHub-ärende med vad som hände, som du kan granska och publicera.
crash-dismiss = Stäng
palette-report = Rapportera ett problem på GitHub

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
profile-name = Profilnamn
company-name = Företagsnamn
company-name-description = Ditt företags och din profils namn, för hela profilen: varje sparfil i den. De ligger i profilfilen, som säkerhetskopieras innan den skrivs.
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
coach-services = Busstjänster
coach-services-description = Din busskarriär som spelets statistik visar den. Spelet räknar fram den ur de tjänster det loggat, så passagerarna fördelas över dem.
coach-service-count = Tjänster
coach-passengers = Transporterade passagerare
coach-average-satisfaction = Genomsnittlig nöjdhet
coach-best-satisfaction = Bästa nöjdhet
coach-distance = Bussträcka
coach-stops = Hittade busshållplatser
coach-stops-found = { $found } av { $count }
coach-perfect-satisfaction = Perfekt nöjdhet
coach-perfect-satisfaction-tooltip = Passagerarna i varje loggad tjänst går helt nöjda, så genomsnittet och det bästa blir 100 %.
coach-discover-stops = Hitta varje busshållplats
coach-discover-stops-tooltip = Markerar varje busshållplats på kartan som hittad, som om du varit vid alla.
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
adr = { $game ->
    [ats] Farligt gods
   *[other] ADR
}
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
agency-unlock-off = Spelet måste hitta en rekryteringsbyrå själv: kör förbi den. En som låses upp i sparfilen förblir stängd i spelet (svart skärm när du går in).
agency-lock-tooltip = Lås den igen, som om du aldrig kört förbi den. Om dess skärm förblir svart i spelet: lås den här och kör förbi den, så hittar spelet den igen och den öppnas.
map-needs-game = För att låsa upp handlare och byråer måste spelet vara installerat på den här datorn: dess data berättar vilka städer som har dem.
map-agencies-reading = Vilka städer som har en rekryteringsbyrå läses från spelets karta, en stund efter resten av dess data.
column-dealer = Lastbilshandlare
column-recruitment = Rekryteringsbyrå
map-place-unlocked = Upplåst
map-place-locked = Inte hittad än
city-place-missing = Ingen i den här staden
city-place-unknown = Okänt
city-visit = Markera som besökt
city-visit-tooltip = Markera staden som besökt, som om du hade kört genom den.
city-unlock = Lås upp
city-unlock-tooltip = Lås upp, som om du hade kört förbi.
city-unvisit = Markera som inte besökt
city-unvisit-tooltip = Markera staden som aldrig besökt, som på en ny profil. Spelet räknar den igen nästa gång du kör genom den.
city-lock = Lås
city-lock-tooltip = Lås igen, som om du aldrig hade kört förbi.
city-open-garage = Öppna
city-show-on-map = Visa på kartan

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
drivers-max-all-tooltip = { $game ->
    [ats] Varje färdighet på högsta nivån och alla faroklasser, för varje anställd förare.
   *[other] Varje färdighet på högsta nivån och alla ADR-klasser, för varje anställd förare.
}
drivers-maxed = { $count ->
    [one] 1 förare maxad
   *[other] { $count } förare maxade
}
column-hometown = Hemstad
column-location = Nu i
column-experience = Erfarenhet
column-skills = Färdigheter
column-adr = { $game ->
    [ats] Farligt gods
   *[other] ADR
}
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
fleet-none = Inga egna lastbilar än
fleet-none-body = Köp en lastbil hos en återförsäljare i spelet, så syns den här.
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
percent = { $amount } %
amount-with-unit = { $amount } { $unit }
unit-km = km
unit-mi = mi
unit-kg = kg
unit-lb = lb
unit-t = t
trucks-table = Fordon
unknown-truck = Okänd modell

# Moving trucks and drivers; the current job
move-garage-full = { $city } · fullt
move-what-truck = { $model } · { $plate }
move-what-driver = Förare från { $city }
driver-truck = Lastbil
driver-truck-none = Ingen lastbil
driver-truck-open = Öppna lastbilen
garage-driver-open = Öppna föraren
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

# Removing from the company; roads
remove-truck = Ta bort lastbilen
remove-trailer = Ta bort trailern
remove-driver = Ta bort föraren
remove-garage = Ta bort garaget
remove-tooltip = Tas bort ur ditt företag när du sparar. Till skillnad från att sälja i spelet får du inga pengar för det. Ångra tar tillbaka det.
remove-why-yours = Den här kör du själv: den blir kvar.
remove-why-your-trailer = Du drar den här trailern: den blir kvar.
remove-why-has-driver = En förare kör den här lastbilen. Ta bort föraren, eller flytta hen, först.
remove-why-in-use = Den här trailern används i det pågående uppdraget.
remove-why-headquarters = Det här är ditt huvudkontor. Gör ett annat garage till huvudkontor först.
remove-why-not-empty = Flytta eller ta bort dess lastbilar, förare och trailrar först.
remove-why-unknown = Den här sparfilens listor har en form redigeraren inte känner till (en nyare version av spelet?), så den gissar inte.
removal-before = I ditt företag
removal-after = Borttagen
copy-to-garage = Kopiera till ett garage
copy-tooltip = Lägger till en kopia i ditt företag när du sparar, i garaget du väljer: samma delar, lack, registreringsskylt och slitage. Ångra tar tillbaka det.
copy-no-room = Inget av dina garage har plats för en till: köp eller uppgradera ett under Garage, eller frigör en plats.
copy-change = Ny kopia
copy-change-before = Ingen
buy-truck = Köp en lastbil…
buy-truck-title = Köp en lastbil
buy-truck-description = Lägger till en ny lastbil i ditt företag när du sparar, som spelets återförsäljare säljer den färdig, i ett garage med en ledig plats. Den är gratis: inga pengar dras. Ångra tar tillbaka det.
buy-truck-model = Lastbil
buy-truck-option = { $model } · { $chassis } · { $engine } · { $price }
buy-button = Köp
buy-truck-added = Lastbilen köpt: den ingår i ditt företag när du sparar.
buy-truck-choose = Välj en lastbil och ett garage.
buy-reading = Läser spelets återförsäljare…
buy-needs-game = Lastbilarna och släpen kommer från spelets egna återförsäljare, vilket kräver att spelet är installerat på den här datorn (se Spelinställningar).
buy-truck-no-room = Inget av dina garage har en ledig lastbilsplats: köp eller uppgradera ett under Garage, eller frigör en plats.
buy-change = Ny lastbil
buy-trailer = Köp ett släp…
buy-trailer-title = Köp ett släp
buy-trailer-description = Lägger till ett nytt släp i ditt företag när du sparar, som spelets återförsäljare säljer det färdigt, i ett garage med plats för det. Det är gratis: inga pengar dras. Ångra tar tillbaka det.
buy-trailer-model = Släp
buy-trailer-option = { $model } · { $chassis } · { $price }
buy-trailer-added = Släpet köpt: det ingår i ditt företag när du sparar.
buy-trailer-choose = Välj ett släp och ett garage.
buy-trailer-no-room = Inget av dina garage har plats för ett släp till (ett garage rymmer lika många som det har lastbilsplatser): köp eller uppgradera ett under Garage.
buy-trailer-change = Nytt släp
change-career = Karriär
change-you = Du
change-cities = Städer
change-wear = Slitage: { $part }
change-on = På
changes-none = Inget skiljer sig från sparfilen på disken.
roads-what = Besökta vägar
roads-count = { $count ->
    [one] En vägbit
   *[other] { $amount } vägbitar
}
roads-none = Inga
map-forget-roads = Glöm besökta vägar
map-forget-roads-tooltip = Kartans upptäckta vägar börjar om, som på en ny profil. Städer du har besökt förblir besökta.
map-discover-roads = Upptäck alla vägar
map-discover-roads-tooltip = Markerar varje väg och korsning på kartan som körd, även i dina kart-DLC, läst från spelets egen karta. Städer du inte varit i förblir obesökta.
map-discover-roads-needs-game = Kräver att spelet är installerat på den här datorn: dess karta säger var vägarna finns.
map-discover-roads-confirm-title = Upptäcka alla vägar?
map-discover-roads-confirm-body = Varje väg och korsning på kartan, även i dina kart-DLC, markeras som körd. Spelet kan räkna vägarna mot sina achievements{ $game ->
    [ats] {""}
   *[other] , som Pathfinder (40 000 km upptäckta vägar)
}, men om det gör det, eller när, vet man inte. Ångra tar tillbaka det tills du sparar, och sparfilen säkerhetskopieras först.
map-view-driven = Körda
map-view-not-driven = Inte körda än
map-view-truck = Din lastbil
map-view-garages = Dina garage
map-view-names = Stadsnamn
map-view-hide = Klicka för att dölja
map-view-show = Klicka för att visa
map-view-drawing = Ritar kartan…
map-view-needs-game = Kräver att spelet är installerat på den här datorn: kartan kommer från dess filer.
map-view-hint = Scrolla för att zooma, dra för att flytta, klicka för att öppna
map-view-zoom-in = Zooma in
map-view-zoom-out = Zooma ut
map-view-fit = Hela kartan
map-view-find = Hitta en stad…

# Truck and trailer details
plate-colours = Skyltens färger
plate-colours-pick = Välj…
plate-world-of-trucks = Din lastbil har alltid din World of Trucks-skylt: spelet sätter den på den lastbil som är din varje gång en sparfil laddas. Ändra den på World of Trucks: My page → Profile customization (Game default lämnar tillbaka skylten till spelet).
plate-plain = Spelets egen skylt
plate-black-on-yellow = Svart på gult
plate-black-on-white = Svart på vitt
plate-white-on-black = Vitt på svart
plate-white-on-blue = Vitt på blått
plate-white-on-red = Vitt på rött
plate-white-on-green = Vitt på grönt
plate-gold-on-black = Guld på svart
plate-custom = Andra färger
truck-engine = Motor
truck-transmission = Växellåda
truck-part-pick = Välj…
truck-part-loading = Läser spelet…
paint = Lack
paint-job = Lackering
paint-job-pick = Välj en lackering
paint-base = Grundfärg
paint-colour-1 = Första färgen
paint-colour-2 = Andra färgen
paint-colour-3 = Tredje färgen
paint-locked = Lackeringens färger bestäms av spelet.
saved-colours = Sparade färger
saved-colours-count = { $count ->
    [0] Inga
    [one] 1 färg
   *[other] { $count } färger
}
saved-colours-open = Sparade färger…
saved-colours-description = Färgerna som sparats i spelets lackskärm, för hela företaget: 8, var och en upp till fyra färger (spelet sparar en lackerings färger tillsammans; en enda färg använder den första). De visas som snabbval i varje fordons lack.
saved-colours-slot = Sparad färg { $slot }
saved-colours-colour = Sparad färg { $slot }, färg { $colour }
saved-colours-copy = Kopiera som text
saved-colours-copy-tooltip = Kopierar färgerna som text att dela: #RRGGBB för varje sparad färg, en uppsättnings färger sammanfogade med /.
saved-colours-copied = Färgerna kopierade
saved-colours-paste = Klistra in text
saved-colours-paste-tooltip = Tar färger som delats som text (#RRGGBB, en uppsättning sammanfogad med /, - för en tom) från urklipp, från första sparade färgen.
saved-colours-not-colours = Urklipp innehåller inga färger som text: #RRGGBB för varje sparad färg, åtskilda med mellanslag.
truck-badge = Sidoemblem
truck-remove-badge = Ta bort det
truck-remove-badge-tooltip = Tar bort motoremblemet från hyttens sidor (emblemet fram sitter kvar).
trailer-load = Last
trailer-cargo = Last
trailer-empty-weight = Tomvikt
trailer-countries = { $game ->
    [ats] Delstater
   *[other] Länder
}
trailer-everywhere = Överallt
trailer-allow-everywhere = Tillåt överallt
edit-restricted = { $game ->
    [ats] Bara vissa delstater
   *[other] Bara vissa länder
}
edit-unrestricted = Överallt
edit-fuel = { $percent } % bränsle
edit-badge = Med sidoemblem
edit-no-badge = Utan sidoemblem
edit-plate = { $plate } · { $colours }
edit-wear = { $part } { $percent } % slitage

# Profile tools
profile-tools-open = Profilverktyg…
profile-tools-title = Profilen ”{ $name }”
profile-tools-backup = Säkerhetskopiera profilen
profile-tools-backup-description = En kopia av hela profilen (varje sparfil, dess bilder och inställningar), sparad med de andra säkerhetskopiorna.
profile-tools-backup-now = Säkerhetskopiera nu
profile-tools-backed-up = Säkerhetskopierad ({ $size } MB).
profile-tools-copy = Gör en kopia
profile-tools-copy-description = En andra profil med allt i den här, med ett nytt namn. Den görs på den här datorn (Steam Cloud visar bara profiler som spelet har gjort), och utan din World of Trucks-inloggning: koppla den i spelet om du vill.
profile-tools-copy-suggestion = { $name } kopia
profile-tools-copy-now = Gör kopian
profile-tools-copied = ”{ $name }” är gjord. Den finns i spelets profillista.
profile-tools-copy-name-empty = Ge kopian ett namn.
profile-tools-copy-name-taken = Det finns redan en profil med det namnet.
profile-tools-copy-no-documents = Spelets mapp under Dokument hittades inte. Starta spelet en gång och försök igen.
profile-tools-settings = Kopiera inställningarna
profile-tools-settings-description = Den här profilens grafik, kontroller och spelalternativ, över en annan profils. Den andra profilens egna inställningar säkerhetskopieras först.
profile-tools-settings-pick = Välj en profil…
profile-tools-settings-now = Kopiera inställningarna
profile-tools-settings-copied = Kopierade till ”{ $name }”.
profile-tools-settings-none = Det finns ingen annan profil med inställningar att kopiera till.
profile-tools-running = Stäng spelet först: det skriver profiler när det avslutas.
profile-tools-failed = Det gick inte: { $error }

# The player's truck and position
player-truck = Din lastbil
player-truck-yours = Den här kör du
player-drive = Kör den själv
player-drive-tooltip = Gör den till din lastbil: den som körde den får din, och dess garage blir ditt huvudkontor. Den står där du är när du laddar sparfilen.
player-why-on-job = Slutför eller avbryt det pågående uppdraget först.
player-why-not-in-garage = Bara lastbilar i ett garage kan bli dina.
player-position = Din position
player-position-somewhere = Någonstans på kartan
player-position-from = { $place }, som i ”{ $save }”
player-position-now = Din lastbil är i { $place }.
player-position-away = Din lastbil är ute på vägen, utanför städerna.
player-position-no-truck = Du har ingen egen lastbil än.
player-position-unknown = Sparfilen säger inte var din lastbil är.
player-quick-travel = Snabbresa när sparfilen laddas
player-quick-travel-tooltip = När sparfilen laddas säger spelet ”Game change detected” och låter dig välja vilken stad du har besökt som helst för din lastbil (och buss), gratis. Det går inte under ett uppdrag, så inget avbryts.
player-quick-travel-asked = När sparfilen laddas säger spelet ”Game change detected” och låter dig välja en stad för din lastbil (och buss), gratis. Ångra tar tillbaka det.
player-quick-travel-after = Snabbresa när sparfilen laddas
player-quick-travel-unavailable = Den här sparfilen kan inte be spelet om en snabbresa (en äldre version av spelet).
player-why-no-truck = Snabbresa flyttar dina egna fordon, så den kräver en egen lastbil.
player-move = Där en annan sparfil är…
player-move-pick = Välj en sparfil…
player-move-description = Din lastbil flyttas dit den står i en annan sparfil i den här profilen.
player-move-reading = Läser ”{ $save }”…
player-move-no-position = ”{ $save }” säger inte var din lastbil är.
player-move-hitched = Koppla loss trailern i spelet först: den skulle också behöva en plats.

# Your day
day-section = Din dag
day-description = Hur länge du har kört, ditt register hos polisen och klockan. Ändringar här skrivs när du sparar, som resten.
day-since-break = Sedan din senaste rast
day-rest = Vila
day-rest-tooltip = Som om du hade sovit: ingen trötthet och ingen rast som väntar.
day-rest-what = Trötthet
day-rested = Utvilad
day-record = Polisregister
day-record-summary = { $count ->
    [0] Rent
    [one] En förseelse, { $amount } i böter
   *[other] { $count } förseelser, { $amount } i böter
}
day-record-clear-button = Rensa registret
day-record-tooltip = Tar bort varje förseelse ur registret. Böter som redan är betalda förblir betalda.
day-record-clear = Rent
day-cargo-damage = Lastskada
day-repair-cargo = Laga lasten
day-time = Tid
day-time-on-job = Att flytta klockan skulle göra ditt uppdrag försenat.
day-skip-morning = Morgon (08:00)
day-skip-noon = Mitt på dagen (12:00)
day-skip-evening = Kväll (18:00)
day-skip-night = Natt (22:00)
day-and-time = { $day } { $time }
day-monday = måndag
day-tuesday = tisdag
day-wednesday = onsdag
day-thursday = torsdag
day-friday = fredag
day-saturday = lördag
day-sunday = söndag
duration-minutes = { $minutes } min
duration-hours = { $hours } h { $minutes } min
player-trailer = Din trailer
player-trailer-none = Ingen
player-trailer-yours = Den här drar du
player-pull = Dra den själv
player-pull-tooltip = Gör den till trailern du drar. Den är inte påkopplad: koppla på den i spelet.

# Picking rows
picks-count = { $count ->
    [one] En vald
   *[other] { $count } valda
}
picks-max = Maxa deras färdigheter
picks-large = Gör dem stora
picks-move = Flytta till garage…
picks-refuel = Fyll tankarna
picks-remove = Ta bort
picks-moved = Flyttade { $moved } av { $count }: de andra fick inte plats där.
picks-removed = Tog bort { $removed } av { $count }: resten kan inte tas bort (öppna en för att se varför).
picks-moved-all = { $count ->
    [one] Flyttad.
   *[other] Flyttade alla { $count }.
}
picks-removed-all = { $count ->
    [one] Borttagen.
   *[other] Tog bort alla { $count }.
}
picks-clear = Rensa
picks-done-repair = { $count ->
    [one] Lagade 1 fordon
   *[other] Lagade { $count } fordon
}
picks-done-refuel = { $count ->
    [one] Fyllde 1 tank
   *[other] Fyllde { $count } tankar
}
picks-done-max = { $count ->
    [one] Maxade färdigheterna för 1 förare
   *[other] Maxade färdigheterna för { $count } förare
}
picks-done-large = { $count ->
    [one] Gjorde 1 garage stort
   *[other] Gjorde { $count } garage stora
}
picks-done-body = Ångra tar tillbaka det. Inget skrivs förrän du sparar.
picks-all = Välj varje rad som visas
picks-row = Välj { $name }

## Statistics
page-statistics = Statistik
page-coach-career = Busskarriär
stats-deliveries = Leveranser
stats-income = Intjänat på leveranser
stats-xp = XP från leveranser
stats-driven = Kört totalt
stats-days = Dagar på vägen
stats-average = Snitt per leverans
stats-favourite = Kört mest
stats-favourite-value = { $cargo } ({ $count })
stats-cancelled = Avbrutna uppdrag
stats-note = Från spelets leveranslogg. Kan inte ändras: spelet håller själv i de här siffrorna.
stats-search = Sök efter last eller stad
stats-filter-damaged = Skadad last
stats-table = Leveranser
stats-none = Inga leveranser än
stats-none-body = Varje last du levererar visas här, den senaste först.
stats-when = { $days ->
    [0] I dag
    [1] I går
   *[other] För { $days } dagar sedan
}
column-when = När
column-income = Inkomst
column-xp = XP
column-damage = Skada
column-passengers = Passagerare
column-satisfaction = Nöjdhet
coach-earned = Intjänat på uppdrag
coach-log-note = Från spelets logg över bussuppdrag. Skrivskyddat: spelet håller själv reda på de här siffrorna.
coach-log-search = Sök på stad
coach-log-table = Långfärdsbussuppdrag
coach-log-none = Inga bussuppdrag än
coach-log-none-body = Varje bussuppdrag du kör visas här, det senaste först.

## What the save needs
needs = DLC och moddar
needs-description = { $mods ->
    [0] Det här sparet gjordes med { $dlcs } DLC:er. Spelet laddar det bara om de som behövs är installerade.
   *[other] Det här sparet gjordes med { $dlcs } DLC:er och { $mods } moddar. Spelet laddar det bara om de som behövs är installerade.
}
needs-unknown = Går inte att kontrollera utan spelet. Välj dess mapp i Inställningar.
needs-all-here = Allt det gjordes med är installerat.
needs-missing = { $count ->
    [one] En saknas som sparet behöver. Installera den, annars laddar spelet inte sparet:
   *[other] { $count } saknas som sparet behöver. Installera dem, annars laddar spelet inte sparet:
}
needs-missing-extras = { $count ->
    [one] Ett tillägg saknas (sparet laddar utan det):
   *[other] { $count } tillägg saknas (sparet laddar utan dem):
}
needs-mods = Moddar
needs-installed = Installerad
needs-not-installed = Inte installerad

## Galleri: utsiktsplatser och fototroféer (spelets ord)

gallery-viewpoint = Utsiktsplats
gallery-viewpoints = Utsiktsplatser
gallery-photo-trophy = Fototrofé
gallery-photo-trophies = Fototroféer
gallery-visited = Besökt
gallery-found = Hittad, inte sedd
gallery-not-found = Inte hittad än
gallery-captured = Fotograferad
gallery-not-captured = Inte fotograferad än
gallery-visited-of = { $visited } av { $count } besökta
gallery-viewpoints-summary = { $visited } av { $count } utsiktsplatser besökta
gallery-photo-trophies-summary = { $captured } av { $count } fototroféer fotograferade
gallery-where-kept = Utsiktsplatser sparas i den här sparfilen, så varje sparfil har sina egna. Fototroféer sparas i profilen och delas av alla dess sparfiler.
gallery-column-name = Namn
gallery-column-kind = Typ
gallery-column-country = { $game ->
    [ats] Delstat
   *[other] Land
}
gallery-column-progress = Framsteg
gallery-filter-viewpoints = Utsiktsplatser
gallery-filter-photo-trophies = Fototroféer
gallery-filter-to-do = Inte klara än
gallery-search = { $game ->
    [ats] Sök på namn eller delstat
   *[other] Sök på namn eller land
}
gallery-table = Utsiktsplatser och fototroféer
gallery-visit-all = Besök varje utsiktsplats
gallery-visit-all-tooltip = Markera varje utsiktsplats som besökt, som om du sett var och en. Galleriet kan då spela upp alla.
gallery-visit = Markera som besökt
gallery-visit-tooltip = Som om du sett den: den räknas som besökt, och Galleriet kan spela upp den.
gallery-forget = Markera som inte hittad
gallery-forget-tooltip = Som om du aldrig varit där: spelet hittar den igen när du kör förbi.
gallery-needs-game = Galleriet behöver spelet installerat på den här datorn: dess data listar utsiktsplatserna och fototroféerna.
gallery-photo-trophy-note = Fototroféer är dina egna foton, sparade med profilen, så editorn bara visar dem: ta en i spelet genom att fotografera den.
gallery-map-not-read = Spelets karta läses fortfarande in, eller hittades inte.
gallery-map-hover = { $name }: { $progress }
