# German. Drafted by machine translation from en.ftl; it needs review by a native speaker
# (wording, and the game's own German terms).
#
# Deutscher Text für Truck Save Editor.
#
# Fluent-Format: https://projectfluent.org/fluent/guide/
# Ein Begriff, ein Wort: Wird hier ein Wort für etwas verwendet, dann überall dasselbe.
# Zahlen in $count wählen die Pluralform; Beträge kommen fertig formatiert.
# Jede Nachricht aus en.ftl steht auch hier, mit denselben Variablen (ein Test prüft das).

## Start screen

start-choose = Wähle einen Spielstand zum Bearbeiten
start-looking = Suche nach deinen Spielständen…
start-none-title = Keine Spielstände gefunden
start-none-body = An keinem der Orte unten gibt es ein Profil des Spiels. Wenn du auf diesem PC spielst, starte das Spiel und speichere einmal: Die Liste wird aktualisiert, wenn du zu diesem Fenster zurückkehrst. Liegen deine Spielstände woanders, etwa in einer Sicherung oder auf einem anderen Laufwerk, füge diesen Ordner hinzu.
start-look-again = Erneut suchen
start-your-saves = Deine Spielstände
start-game-saves = Vom Spiel gespeichert
start-game-saves-hint = Automatische und Schnellspeicherungen. Das Spiel ersetzt sie beim Spielen.
start-game-saves-more = { $count ->
    [one] Einen weiteren anzeigen
   *[other] { $count } weitere anzeigen
}
start-game-saves-fewer = Weniger anzeigen
start-showing-cloud = { $count ->
    [one] Ein Spielstand in „{ $name }“, in Steam Cloud gespeichert und damit auf jedem PC gleich, auf dem du spielst.
   *[other] { $count } Spielstände in „{ $name }“, in Steam Cloud gespeichert und damit auf jedem PC gleich, auf dem du spielst.
}
start-showing-local = { $count ->
    [one] Ein Spielstand in „{ $name }“, nur auf diesem PC.
   *[other] { $count } Spielstände in „{ $name }“, nur auf diesem PC.
}
start-profile-empty-title = Noch keine Spielstände in „{ $name }“
start-profile-empty-body = Speichere einmal im Spiel, dann erscheint der Spielstand hier. Die Liste wird aktualisiert, wenn du zu diesem Fenster zurückkehrst.
start-folders = Wo nach Spielständen gesucht wird
start-folders-description = Das Spiel speichert jedes Profil in Steam Cloud oder auf diesem PC, je nach den Profileinstellungen im Spiel.
start-folders-missing = { $count ->
    [one] Auch in einem Ordner gesucht, den es auf diesem PC nicht gibt
   *[other] Auch in { $count } Ordnern gesucht, die es auf diesem PC nicht gibt
}
start-folders-missing-hide = Nicht vorhandene Ordner ausblenden
start-folder-added = Von dir hinzugefügt
start-folder-profiles = { $count ->
    [one] Ein Profil
   *[other] { $count } Profile
}
start-folder-empty = Keine Profile
start-folder-missing = Nicht vorhanden
start-folder-no-steam = Steam wurde auf diesem PC nicht gefunden
start-folder-show = Ordner anzeigen
start-folder-remove = Hier nicht mehr suchen
start-folder-add = Ordner hinzufügen…
start-folder-drop = Oder zieh einen Spielstand- oder Profilordner auf dieses Fenster.
start-folder-choose = Wähle einen Ordner mit Spielständen
start-folder-not-saves = Keine Spielstände oder Profile in { $path }. Wähle den Ordner „profiles“ des Spiels, den Ordner eines Profils oder eines Spielstands.
start-slot-missing = { $count ->
    [one] Braucht einen DLC oder Mod, der nicht installiert ist
   *[other] Braucht { $count } DLCs oder Mods, die nicht installiert sind
}
start-slot-number = Speicherplatz { $number }
profile-place = { $name } · { $place }
profile-game-place = { $name } · { $game } · { $place }
location-steam-cloud = Steam Cloud
location-this-pc = Dieser PC
location-loose = Ein einzelner Spielstand

## Save slot names (for saves without a name of their own)

slot-unnamed = Unbenannter Spielstand
slot-autosave = Automatische Speicherung
slot-autosave-drive = Automatische Speicherung während der Fahrt
slot-autosave-job = Automatische Speicherung bei einem Auftrag
slot-quicksave = Schnellspeicherung
slot-multiplayer-backup = Vor dem Konvoi

## How long ago

time-just-now = gerade eben
time-minutes-ago = { $count ->
    [one] vor 1 Minute
   *[other] vor { $count } Minuten
}
time-hours-ago = { $count ->
    [one] vor 1 Stunde
   *[other] vor { $count } Stunden
}
time-days-ago = { $count ->
    [one] vor 1 Tag
   *[other] vor { $count } Tagen
}
time-weeks-ago = { $count ->
    [one] vor 1 Woche
   *[other] vor { $count } Wochen
}
time-months-ago = { $count ->
    [one] vor 1 Monat
   *[other] vor { $count } Monaten
}
time-years-ago = { $count ->
    [one] vor 1 Jahr
   *[other] vor { $count } Jahren
}

## Sidebar and pages

sidebar-career = Karriere
sidebar-company = Unternehmen
sidebar-advanced = Erweitert
page-overview = Übersicht
page-fleet = Flotte
page-garages = Garagen
headquarters = Hauptquartier
headquarters-pick = Wähle eine Stadt
page-drivers = Fahrer
page-map = Karte
page-all-data = Alle Daten
switch-save = Spielstand wechseln
theme-light = Heller Modus
theme-dark = Dunkler Modus

## Header

header-subtitle = { $game } · { $place }
changes-count = { $count ->
    [0] Keine Änderungen
    [one] 1 Änderung
   *[other] { $count } Änderungen
}
discard = Verwerfen
save = Speichern
save-open = Speichern…
details-open = Details…
settings-open = Einstellungen…
saving = Wird gespeichert…

## Opening, closing and saving

opening = „{ $name }“ wird geöffnet…
open-failed = „{ $name }“ konnte nicht geöffnet werden
close-unsaved-title = „{ $name }“ ohne Speichern schließen?
close-unsaved-body = { $count ->
    [one] Deine Änderung geht verloren.
   *[other] Deine { $count } Änderungen gehen verloren.
}
close-unsaved-confirm = Ohne Speichern schließen
quit-unsaved-title = Beenden, ohne „{ $name }“ zu speichern?
quit-unsaved-confirm = Ohne Speichern beenden
save-confirm-title = Änderungen an „{ $name }“ speichern?
cancel = Abbrechen
save-as-new-slot = Als neuen Speicherplatz speichern
save-as-new-slot-tooltip = Schreibt die Änderungen in einen neuen Speicherplatz und lässt diesen unverändert.
save-as-new-slot-loose = Dieser Spielstand liegt einzeln außerhalb eines Profils, daher gibt es kein Profil für einen neuen Speicherplatz.
new-slot-name = { $name } (bearbeitet)
saved-as-new-title = Als „{ $name }“ gespeichert und geprüft
saved-as-new-body = „{ $original }“ ist unverändert. Du bearbeitest jetzt den neuen Speicherplatz.
save-confirm-note = Zuerst wird eine Sicherung des Speicherplatzes angelegt, und nach dem Schreiben wird der Spielstand erneut gelesen und geprüft. Schließe das Spiel vor dem Speichern: Es überschreibt Spielstände beim Beenden.
save-confirm-cloud = Dieses Profil liegt in Steam Cloud: Fragt Steam, welche Kopie behalten werden soll, wähle die auf diesem PC. „Als neuen Speicherplatz speichern“ ist aus, weil das Spiel nur Spielstände anzeigt, die es selbst angelegt hat. Um diesen Spielstand zu behalten, überschreibe stattdessen einen anderen.
save-prepare-failed = Der Spielstand konnte nicht vorbereitet werden
save-no-backup-folder-title = Speichern nicht möglich
save-no-backup-folder = Es gibt keinen Ordner für Sicherungen, daher wurde nichts geschrieben.
save-over-other = Anderen überschreiben…
save-over-other-tooltip = Schreibt die Änderungen über einen anderen Spielstand dieses Profils und lässt diesen unverändert. Funktioniert mit Steam Cloud.
save-over-pick-title = Welchen Spielstand überschreiben?
save-over-pick-body = Der gewählte wird gesichert und dann durch „{ $original }“ mit deinen Änderungen ersetzt. Er behält seinen Namen. „{ $original }“ bleibt unverändert.
save-over-confirm-title = „{ $name }“ ersetzen?
save-over-confirm-body = „{ $name }“ wird zuerst gesichert und dann durch „{ $original }“ mit deinen Änderungen ersetzt, unter seinem eigenen Namen. „{ $original }“ bleibt unverändert, und du bearbeitest danach „{ $name }“.
save-over-confirm = Ersetzen
save-over-no-backups = Es gibt keinen Ort für die Sicherung, daher wurde nichts geschrieben.
saved-over-title = „{ $name }“ überschrieben und geprüft
saved-over-body = „{ $original }“ ist unverändert. Du bearbeitest jetzt den anderen Spielstand.
saved-title = „{ $name }“ gespeichert und geprüft
saved-backup = Die vorherige Version liegt unter Sicherungen.
save-failed = „{ $name }“ konnte nicht gespeichert werden
save-profile-failed = Der Spielstand wurde geschrieben, der Unternehmensname aber nicht: Er wartet noch darauf, gespeichert zu werden

## Warnings

newer-game-warning = Dieser Spielstand stammt aus einer neueren Spielversion, als mit diesem Editor getestet wurde. Alles, was der Editor nicht versteht, bleibt unverändert, aber prüfe deine Änderungen im Spiel und behalte die Sicherung.
ats-warning = American Truck Simulator wird noch nicht vollständig unterstützt. Spielstände lassen sich sicher öffnen und speichern, mit Sicherung, aber Städtenamen, Motoren, die DLC-Prüfung und Entfernungen (in km statt Meilen) stammen aus Euro Truck Simulator 2 oder fehlen. Prüfe deine Änderungen im Spiel.

## Undo and redo

undo = Rückgängig
redo = Wiederholen
save-tooltip = Änderungen prüfen und speichern (Ctrl+S)
undo-tooltip = Rückgängig (Ctrl+Z)
redo-tooltip = Wiederholen (Ctrl+Shift+Z)

## Jobs

page-jobs = Aufträge
jobs-summary = { $count ->
    [one] 1 Auftragsangebot
   *[other] { $amount } Auftragsangebote
}
jobs-offers = Auftragsangebote
jobs-keep = Jedes Angebot eine Woche behalten
jobs-keep-tooltip = Jedes Auftragsangebot bleibt mindestens eine Woche Spielzeit bestehen, damit das gewünschte noch da ist.
jobs-kept-week = Mindestens eine Woche behalten
jobs-table = Auftragsangebote
jobs-search = Nach Stadt, Unternehmen oder Fracht suchen
jobs-filter-my-garages = Von meinen Garagen
jobs-filter-urgent = Dringend
jobs-filter-no-ferry = Ohne Fähre
column-from = Von
column-to = Nach
column-cargo = Fracht
column-distance = Entfernung
column-expires = Läuft ab
column-urgency = Dringlichkeit
job-place = { $company }, { $city }
job-urgency-standard = Standard
job-urgency-important = Wichtig
job-urgency-urgent = Dringend
job-expires-in = { $minutes ->
    [one] in 1 Minute
   *[other] in { $minutes } Minuten
}
job-expires-in-hours = { $hours ->
    [one] in 1 Stunde
   *[other] in { $hours } Stunden
}
job-expires-in-days = { $days ->
    [one] in 1 Tag
   *[other] in { $days } Tagen
}
job-expired = Abgelaufen
job-ferry = { $distance } · Fähre

## Detail panels

detail-truck = Lkw
detail-coach = Reisebus
detail-trailer = Auflieger
detail-driver = Fahrer
detail-garage-in = Garage in { $city }
detail-about = Info
detail-gone = Das ist nicht mehr im Spielstand.
part-engine = Motor
part-transmission = Getriebe
part-cabin = Kabine
part-chassis = Fahrgestell
part-body = Aufbau
part-wheels = Räder
repair-all-parts = Alles reparieren
fill-tank = Volltanken
driver-from = Aus { $city }
driver-now-in = Jetzt in { $city }
driver-max = Diesen Fahrer maximieren
garage-empty = Keine
garage-make-large = Groß ausbauen
garage-buy-large = Kaufen, groß

## Filters

filter-count = { $shown } von { $total }
filter-no-matches = Keine Treffer
filter-no-matches-body = Versuche eine andere Suche oder setze die Filter zurück.
drivers-none = Noch keine Fahrer eingestellt
drivers-none-body = Stelle im Spiel Fahrer bei einer Arbeitsagentur ein, dann erscheinen sie hier.
jobs-none = Keine Auftragsangebote
jobs-none-body = Das Spiel füllt die Auftragsangebote, während du fährst; dieser Spielstand hat noch keine.
filter-clear = Filter zurücksetzen
fleet-search = Nach Modell, Kennzeichen oder Garage suchen
fleet-filter-worn = Reparaturbedürftig
fleet-filter-no-garage = Ohne Garage
fleet-filter-yours = Deine
fleet-yours = Deiner
garages-search = Nach Stadt suchen
garages-filter-owned = Im Besitz
garages-filter-not-large = Nicht groß
drivers-search = Nach Stadt suchen
drivers-filter-not-maxed = Nicht maximiert
map-search = Nach Stadt suchen
map-filter-not-visited = Nicht besucht
map-filter-dealer-locked = Händler noch nicht gefunden
map-filter-recruitment-locked = Agentur noch nicht gefunden

## Quick start

quick-start = Schnellstart
quick-start-open = Schnellstart…
quick-start-description = Die üblichen Änderungen für eine neue Karriere, auf einmal. Lege im Spiel ein neues Profil an, öffne hier seinen Spielstand und wähle, was du willst. Ein Rückgängig nimmt alles zurück, und nichts wird geschrieben, bevor du speicherst.
quick-start-rich = Reicher Start
quick-start-everything = Alles freigeschaltet
quick-start-money = Kontostand festlegen
quick-start-hq-garage = Eine große Garage in { $city }
quick-start-apply = Anwenden
quick-start-no-coaches = Dieser Spielstand hat keine Reisebus-Karriere (Coaches-DLC).
quick-start-no-hq = Dieser Spielstand hat noch keine Hauptquartier-Stadt.
quick-start-hq-large = Deine Hauptquartier-Garage ist bereits groß.
quick-start-needs-game = Dafür muss das Spiel auf diesem PC installiert sein: Seine Daten sagen, welche Städte sie haben.

## Settings

page-settings = Einstellungen
settings-language = Sprache
settings-language-description = Die Sprache der App. Die Namen aus dem Spiel (Städte, Fracht, Ränge) folgen ihr, wenn das Spiel sie in dieser Sprache hat.
settings-appearance = Darstellung
settings-appearance-system = Wie das System
settings-appearance-light = Hell
settings-appearance-dark = Dunkel
settings-game = Spielordner
settings-game-description = Wo Euro Truck Simulator 2 installiert ist. Der Editor liest daraus Level-Tabellen, Namen und Symbole und ändert dort nie etwas.
settings-game-found = Automatisch gefunden
settings-game-picked = Von dir gewählt
settings-game-not-found = Nicht gefunden. Wähle den Ordner, in dem das Spiel installiert ist.
settings-game-not-game = Dieser Ordner enthält das Spiel nicht: Es gibt keine def.scs.
settings-game-choose = Ordner wählen…
settings-game-automatic = Automatisch finden
settings-backups = Sicherungen
settings-backups-description = Bei jedem Speichern wird eine Sicherung des Speicherplatzes angelegt und hier aufbewahrt, außerhalb der Ordner des Spiels.
settings-backups-open = Ordner öffnen
game-settings = Im Spiel
game-settings-description = Einstellungen des Spiels selbst, in seiner config.cfg. Die Datei wird vor jeder Änderung gesichert.
game-settings-console = Entwicklerkonsole (im Spiel mit ~ öffnen)
game-settings-convoy = Konvois mit bis zu { $count } Lkw
game-settings-running = Schließe das Spiel, um das zu ändern: Es schreibt seine Einstellungen beim Beenden.
game-settings-none = Die Einstellungsdatei des Spiels wurde nicht gefunden. Starte das Spiel einmal und komm dann zurück.
game-settings-failed = Die Einstellungen des Spiels konnten nicht geändert werden: { $error }
settings-updates = Updates
settings-updates-description = Der Editor kann bei GitHub nachfragen, ob eine neuere Version erschienen ist. Nur dann nutzt er das Internet, und nichts über dich oder deine Spielstände wird gesendet. Zum Aktualisieren lädst du die neue Version herunter und ersetzt die .exe; deine Einstellungen und Sicherungen bleiben.
updates-not-checked = Noch nicht geprüft.
updates-checking = Wird geprüft…
updates-up-to-date = Du hast die neueste Version ({ $version }).
updates-available = Version { $version } ist verfügbar.
updates-download = Herunterladen…
updates-failed = GitHub nicht erreichbar. Prüfe die Verbindung und versuche es erneut.
updates-check-now = Jetzt prüfen
updates-check-on-start = Beim Start des Editors prüfen
updates-banner = Version { $version } ist verfügbar
settings-reset = Editor zurücksetzen
settings-reset-description = Setzt die Einstellungen des Editors auf den ersten Start zurück: Sprache, Darstellung, Spielordner, auf dem Startbildschirm hinzugefügte Ordner und die Suche nach Updates. Deine Spielstände und Sicherungen bleiben, wie sie sind.
settings-reset-button = Einstellungen zurücksetzen…
settings-reset-confirm-title = Einstellungen des Editors zurücksetzen?
settings-reset-confirm = Zurücksetzen
settings-about = Info
settings-version = Version { $version }
settings-source = Quellcode auf GitHub
settings-not-affiliated = Nicht mit SCS Software verbunden oder von SCS Software unterstützt.

## Command palette

palette-open = Befehle…
palette-shortcut = Ctrl+K
palette-placeholder = Befehl oder Seite eingeben…
palette-group-pages = Gehe zu
palette-group-actions = Aktionen
palette-group-app = Spielstand und App
palette-go = { $page }
palette-max-truck-skills = Alle Lkw-Fähigkeiten maximieren
palette-max-coach-skills = Alle Reisebus-Fähigkeiten maximieren
palette-all-adr = Alle ADR-Klassen
palette-repair-trucks = Alle Lkw reparieren
palette-repair-coaches = Alle Reisebusse reparieren
palette-repair-trailers = Alle Auflieger reparieren
palette-refuel-all = Alle volltanken
palette-save = Speichern…
palette-buy-all-garages = Alle Garagen kaufen, alle groß
palette-upgrade-owned-garages = Eigene Garagen groß ausbauen
palette-discard = Änderungen verwerfen
palette-theme = Hellen oder dunklen Modus wechseln
report-problem = Problem melden…
report-this = Dieses Problem melden
report-description = Öffnet ein neues Issue auf GitHub in deinem Browser, mit der Version des Editors schon ausgefüllt. Gesendet wird erst, wenn du es veröffentlichst.
palette-report = Problem auf GitHub melden

## Backups

backups = Sicherungen
backups-intro = Bei jedem Speichern wird eine Sicherung des Speicherplatzes angelegt, außerhalb der Ordner des Spiels. Beim Wiederherstellen wird zuerst gesichert, was jetzt da ist, sodass auch das rückgängig gemacht werden kann.
backups-none = Noch keine Sicherungen
backups-none-body = Eine wird angelegt, wenn du diesen Speicherplatz zum ersten Mal speicherst.
backup-made-at = Angelegt { $time } (UTC)
backup-show = Im Ordner zeigen
backup-restore = Wiederherstellen
restore-title = „{ $name }“ aus dieser Sicherung wiederherstellen?
restore-body = { $changes ->
    [0] Was jetzt im Speicherplatz ist, wird zuerst gesichert.
    [one] Was jetzt im Speicherplatz ist, wird zuerst gesichert. Deine ungespeicherte Änderung geht verloren.
   *[other] Was jetzt im Speicherplatz ist, wird zuerst gesichert. Deine { $changes } ungespeicherten Änderungen gehen verloren.
}
restored-title = „{ $name }“ wiederhergestellt
restored-body = Der Speicherplatz ist wieder so geöffnet, wie er in der Sicherung war.
restore-failed = „{ $name }“ konnte nicht wiederhergestellt werden

## Overview

amount-hint = Versuche 2.5m, 10k, +1m oder -500k.
company-section = Dein Unternehmen
profile-name = Profilname
company-name = Unternehmensname
company-name-description = Der Name deines Unternehmens und deines Profils, für das ganze Profil: jeden Spielstand darin. Sie stehen in der Profildatei, die vor dem Schreiben gesichert wird.
money = Kontostand
money-description = Der Kontostand deines Unternehmens.
money-label = Kontostand
experience = Lkw-Erfahrung
experience-description = Dein Level als Lkw-Fahrer.
experience-label = Lkw-Erfahrungspunkte
coach-experience = Reisebus-Erfahrung
coach-experience-description = Reisebusfahren ist eine eigene Karriere.
coach-experience-label = Reisebus-Erfahrungspunkte
experience-unit = XP
experience-amount = { $amount } XP
skills = Lkw-Fähigkeiten
level-number = Level { $level }
level-progress = { $percent }% bis Level { $next }
level-progress-label = Fortschritt bis zum nächsten Level
level-step = { $count ->
    [one] 1 Level
   *[other] { $count } Level
}
coach-skills = Reisebus-Fähigkeiten
coach-skills-description = Jede Reisebus-Fähigkeit geht bis Level 3.
coach-skill-longer-duties = Längere Dienste
coach-skill-longer-duties-description = Mehr Haltestellen pro Dienst
coach-skill-demanding-timetables = Anspruchsvolle Fahrpläne
coach-skill-demanding-timetables-description = Strengere Fahrpläne, die mehr XP bringen
coach-skill-premium-service = Premium-Service
coach-skill-premium-service-description = Fahrgäste, die mehr zahlen und weniger verzeihen
coach-skill-cushioned-journey = Sanftes Fahren
coach-skill-cushioned-journey-description = Ruhiges Fahren bringt mehr Geld und mehr XP
coach-skill-passenger-favourite = Fahrgastliebling
coach-skill-passenger-favourite-description = Fehlerfreies Fahren steigert die Zufriedenheit
coach-skill-ecodriving = Sparsame Fahrweise
coach-skill-ecodriving-description = Verbraucht weniger Kraftstoff
skills-description = Klicke auf ein Level, um es festzulegen. Klicke erneut auf das höchste gefüllte Level, um es zu senken.
skills-max = Alle Fähigkeiten maximieren
skill-level = Level { $level }
skill-level-of = { $skill }: Level { $level } von { $max }
skill-long-distance = Fernfahrten
skill-long-distance-description = Schaltet Aufträge über längere Strecken frei
skill-high-value-cargo = Hochwertige Fracht
skill-high-value-cargo-description = Hochwertige Ladungen, die mehr einbringen
skill-fragile-cargo = Zerbrechliche Fracht
skill-fragile-cargo-description = Ladungen, die leicht zerbrechen
skill-just-in-time = Just-In-Time-Lieferung
skill-just-in-time-description = Dringende Aufträge mit knappen Fristen
skill-ecodriving = Sparsame Fahrweise
skill-ecodriving-description = Verbraucht weniger Kraftstoff
adr = ADR
adr-description = Gefahrgut, das dein Fahrer transportieren darf.
adr-all = Alle Klassen
adr-none = Keine
adr-some = { $count } von { $total } Klassen
adr-explosives = Explosivstoffe
adr-gases = Gase
adr-flammable-liquids = Entzündbare Flüssigkeiten
adr-flammable-solids = Entzündbare Feststoffe
adr-toxic = Giftige Stoffe
adr-corrosive = Ätzende Stoffe
hq = HQ

## All data

all-data-intro = Jede Einheit im Spielstand und jeder Wert darin. Wähle einen Typ, dann eine Einheit, dann einen Wert zum Ändern.
all-data-search = Alle Werte, Felder und Einheiten durchsuchen
all-data-search-count = { $count ->
    [0] Keine Treffer
    [one] 1 Treffer
   *[other] { $amount } Treffer
}
all-data-search-more = Die ersten { $count } Treffer
all-data-results = Suchergebnisse
all-data-kinds = Typen
all-data-units = Einheiten
all-data-values = Werte
all-data-filter = Typen filtern
all-data-kind = Typ
all-data-count = Einheiten
all-data-unit = Einheit
all-data-field = Feld
all-data-value = Wert
all-data-items = { $count ->
    [one] 1 Eintrag
   *[other] { $count } Einträge
}
all-data-pick-value = Wähle einen Wert
all-data-apply = Anwenden
all-data-revert = Zurücksetzen
all-data-wrong-shape = Dieser Wert enthält { $expected }; gib so etwas ein (Text behält seine Anführungszeichen).
shape-bool = true oder false
shape-number = eine Zahl
shape-text = Text in Anführungszeichen
shape-id = eine ID oder einen Namen
shape-vector = Zahlen in Klammern
shape-value = einen Wert

## Map

map-summary = { $visited } von { $count } Städten besucht
map-visit-all = Alle Städte besuchen
map-visit-all-tooltip = Markiert jede Stadt als besucht, als wärst du durch jede gefahren.
map-visited-all = { $count ->
    [one] 1 Stadt mehr besucht
   *[other] { $count } Städte mehr besucht
}
map-visits = { $count ->
    [0] Nicht besucht
    [one] Einmal
   *[other] { $count }-mal
}
column-visits = Besucht
map-table = Städte
map-dealers = Lkw-Händler
map-recruitments = Arbeitsagenturen
map-dealers-summary = { $unlocked } von { $count ->
    [one] 1 Lkw-Händler
   *[other] { $count } Lkw-Händlern
}
map-recruitments-summary = { $unlocked } von { $count ->
    [one] 1 Arbeitsagentur
   *[other] { $count } Arbeitsagenturen
}
map-unlocked-count = { $count } freigeschaltet
map-unlocked-of = { $unlocked } von { $count } freigeschaltet
map-unlock-dealers = Alle Händler freischalten
map-unlock-dealers-tooltip = Schaltet den Lkw-Händler in jeder Stadt frei, die einen hat, als wärst du an jedem vorbeigefahren.
map-unlock-recruitments = Alle Agenturen freischalten
map-unlock-recruitments-tooltip = Schaltet die Arbeitsagentur in jeder Stadt frei, die eine hat, als wärst du an jeder vorbeigefahren.
map-needs-game = Zum Freischalten von Händlern und Agenturen muss das Spiel auf diesem PC installiert sein: Seine Daten sagen, welche Städte sie haben.
column-dealer = Lkw-Händler
column-recruitment = Arbeitsagentur
map-place-unlocked = Freigeschaltet
map-place-locked = Noch nicht gefunden

## Garages

garages = Garagen
garages-before = Wie sie waren
garages-after = { $bought ->
    [0] { $upgraded ->
        [one] 1 groß ausgebaut
       *[other] { $upgraded } groß ausgebaut
    }
   *[other] { $upgraded ->
        [0] { $bought } gekauft, alle groß
       *[other] { $bought } gekauft und { $upgraded } ausgebaut, alle groß
    }
}
garages-summary = { $count ->
    [one] 1 Stadt
   *[other] { $count } Städte
} · { $owned } im Besitz · { $large } groß
garages-upgrade-owned = Eigene ausbauen
garages-upgrade-owned-tooltip = Baut jede Garage in deinem Besitz groß aus: 5 Lkw und 5 Fahrer.
garages-buy-all = Alle kaufen
garages-buy-all-tooltip = Kauft die Garage jeder Stadt, alle groß.
garage-not-owned = Nicht im Besitz
garage-large = Groß
garage-slots = { $count ->
    [one] 1 Stellplatz
   *[other] { $count } Stellplätze
}
garage-filled = { $filled } von { $slots }
column-city = Stadt
column-garage-size = Garage
column-trucks = Lkw
column-drivers = Fahrer
garages-table = Garagen

## Drivers

drivers-summary = { $count ->
    [0] Noch keine Fahrer eingestellt
    [one] 1 Fahrer
   *[other] { $count } Fahrer
} · { $maxed ->
    [0] keiner voll ausgebildet
   *[other] { $maxed } voll ausgebildet
}
drivers-max-all = Alle Fahrer maximieren
drivers-max-all-tooltip = Jede Fähigkeit auf dem höchsten Level und alle ADR-Klassen, für jeden eingestellten Fahrer.
drivers-maxed = { $count ->
    [one] 1 Fahrer maximiert
   *[other] { $count } Fahrer maximiert
}
column-hometown = Heimatstadt
column-location = Jetzt in
column-experience = Erfahrung
column-skills = Fähigkeiten
column-adr = ADR
drivers-table = Fahrer
skills-short = { $long } · { $value } · { $fragile } · { $urgent } · { $eco }
adr-count = { $count } von { $total }

## Fleet

fleet-trucks = { $count ->
    [one] 1 Lkw
   *[other] { $amount } Lkw
}
fleet-worn = { $count ->
    [0] alle in gutem Zustand
    [one] 1 braucht Reparaturen
   *[other] { $amount } brauchen Reparaturen
}
fleet-coaches = { $count ->
    [one] 1 Reisebus
   *[other] { $amount } Reisebusse
}
fleet-tab-trucks = Lkw · { $count }
fleet-tab-coaches = Reisebusse · { $count }
fleet-summary = { $vehicles } · { $worn }
repair-all = Alle reparieren
repairs = Reparaturen
repairs-before = Verschlissen
repairs-after = { $count ->
    [one] 1 Fahrzeug repariert
   *[other] { $count } Fahrzeuge repariert
}
column-truck = Modell
column-plate = Kennzeichen
column-garage = Garage
column-odometer = Kilometerstand
column-wear = Verschleiß
repair = Reparieren
refuel-all = Alle volltanken
refuel-all-tooltip = Tankt jeden Lkw oder Reisebus in dieser Liste voll.
column-fuel = Kraftstoff
vehicle-in-use = Wird gerade gefahren
vehicle-parked = Geparkt in { $city }
fuel = Kraftstoff
fuel-before = Wie es war
fuel-after = { $count ->
    [one] 1 Tank gefüllt
   *[other] { $count } Tanks gefüllt
}
fleet-trailers = { $count ->
    [one] 1 Auflieger
   *[other] { $amount } Auflieger
}
fleet-tab-trailers = Auflieger · { $count }
no-garage = Keine Garage
distance-km = { $amount } km
percent = { $amount }%
trucks-table = Fahrzeuge
unknown-truck = Unbekanntes Modell

# Moving trucks and drivers; the current job
move-garage-full = { $city } · voll
move-what-truck = { $model } · { $plate }
move-what-driver = Fahrer aus { $city }
driver-truck = Lkw
driver-truck-none = Kein Lkw
driver-truck-open = Lkw öffnen
move-hint = Wähle eine andere Garage, um dorthin zu verlegen.
move-with-driver = Sein Fahrer wird mitverlegt.
move-with-truck = Sein Lkw wird mitverlegt.
move-no-room = Keine andere Garage hat einen freien Stellplatz. Kaufe oder baue eine unter Garagen aus.
move-is-players = Diesen fährst du selbst: Er ist dort, wo du ihn hinfährst.
current-job = Aktueller Auftrag
current-job-summary = { $cargo } nach { $city }
current-job-no-deadline = Keine Frist
current-job-cancelled = Abgebrochen
current-job-deadline = Frist
current-job-late = Verspätet
current-job-remove-deadline = Frist entfernen
current-job-cancel = Auftrag abbrechen
current-job-cancelled-note = Wird beim Speichern abgebrochen, ohne Abbruchgebühr. Mit Rückgängig kommt er zurück.
current-job-freight-market = Aus den Frachtaufträgen, in deinem Lkw mit dem Auflieger des Unternehmens. Hier abzubrechen kostet keine Gebühr.
current-job-external = Aus den Externen Aufträgen, in deinem Lkw mit dem Auflieger des Unternehmens. Hier abzubrechen kostet keine Gebühr.
current-job-quick-job = Ein Schnellauftrag im Lkw des Unternehmens. Solche kann der Editor noch nicht abbrechen: Brich ihn im Spiel ab.
current-job-own-trailer = Ein Auftrag vom Frachtmarkt mit deinem eigenen Auflieger. Solche kann der Editor noch nicht abbrechen: Brich ihn im Spiel ab.
current-job-special = Ein Spezialtransport. Solche kann der Editor nicht abbrechen: Brich ihn im Spiel ab.

# Removing from the company; roads
remove-truck = Lkw entfernen
remove-trailer = Auflieger entfernen
remove-driver = Fahrer entfernen
remove-garage = Garage entfernen
remove-tooltip = Wird beim Speichern aus deinem Unternehmen entfernt. Anders als beim Verkaufen im Spiel bekommst du kein Geld dafür. Mit Rückgängig kommt es zurück.
remove-why-yours = Diesen fährst du selbst: Er bleibt.
remove-why-your-trailer = Diesen Auflieger ziehst du gerade: Er bleibt.
remove-why-has-driver = Ein Fahrer fährt diesen Lkw. Entferne oder verlege zuerst den Fahrer.
remove-why-in-use = Dieser Auflieger gehört zum laufenden Auftrag.
remove-why-headquarters = Das ist dein Hauptquartier. Mache zuerst eine andere Garage zum Hauptquartier.
remove-why-not-empty = Verlege oder entferne zuerst ihre Lkw, Fahrer und Auflieger.
remove-why-unknown = Die Listen dieses Spielstands haben eine Form, die der Editor nicht kennt (eine neuere Spielversion?), daher rät er nicht.
removal-before = In deinem Unternehmen
removal-after = Entfernt
change-career = Karriere
change-you = Du
change-cities = Städte
change-km = { $km } km
change-tonnes = { $tonnes } t
change-wear = Verschleiß: { $part }
change-on = An
roads-what = Befahrene Straßen
roads-count = { $count ->
    [one] Ein Straßenstück
   *[other] { $amount } Straßenstücke
}
roads-none = Keine
map-forget-roads = Befahrene Straßen vergessen
map-forget-roads-tooltip = Die entdeckten Straßen der Karte beginnen von vorn, wie bei einem neuen Profil. Besuchte Städte bleiben besucht.
map-discover-roads = Alle Straßen entdecken
map-discover-roads-tooltip = Markiert jede Straße und Kreuzung der Karte als befahren, auch in deinen Karten-DLCs, gelesen aus der Karte des Spiels. Städte, in denen du nicht warst, bleiben unbesucht.
map-discover-roads-needs-game = Braucht das auf diesem PC installierte Spiel: Seine Karte sagt, wo die Straßen sind.
map-discover-roads-confirm-title = Alle Straßen entdecken?
map-discover-roads-confirm-body = Jede Straße und Kreuzung der Karte, auch in deinen Karten-DLCs, wird als befahren markiert. Das Spiel rechnet diese Straßen vielleicht auf seine Erfolge an, etwa Pathfinder (40.000 km entdeckte Straßen), aber ob und wann, ist nicht bekannt. Rückgängig nimmt es bis zum Speichern zurück, und der Spielstand wird vorher gesichert.

# Truck and trailer details
plate-colours = Kennzeichenfarben
plate-colours-pick = Wählen…
plate-world-of-trucks = Dein Lkw trägt immer dein World-of-Trucks-Kennzeichen: Das Spiel bringt es bei jedem Laden an dem Lkw an, der deiner ist. Ändere es auf World of Trucks: My page → Profile customization (Game default überlässt das Kennzeichen wieder dem Spiel).
plate-plain = Das Kennzeichen des Spiels
plate-black-on-yellow = Schwarz auf Gelb
plate-black-on-white = Schwarz auf Weiß
plate-white-on-black = Weiß auf Schwarz
plate-white-on-blue = Weiß auf Blau
plate-white-on-red = Weiß auf Rot
plate-white-on-green = Weiß auf Grün
plate-gold-on-black = Gold auf Schwarz
plate-custom = Andere Farben
truck-engine = Motor
truck-transmission = Getriebe
truck-part-pick = Wählen…
truck-part-loading = Spiel wird gelesen…
truck-badge = Seitenemblem
truck-remove-badge = Entfernen
truck-remove-badge-tooltip = Entfernt das Motoremblem an den Seiten des Fahrerhauses (das Emblem vorn bleibt).
trailer-load = Ladung
trailer-cargo = Fracht
trailer-empty-weight = Leergewicht
trailer-countries = Länder
trailer-everywhere = Überall
trailer-allow-everywhere = Überall erlauben
edit-cargo = { $tonnes } t Fracht
edit-empty = { $tonnes } t leer
edit-restricted = Nur einige Länder
edit-unrestricted = Überall
edit-fuel = { $percent } % Kraftstoff
edit-badge = Mit Seitenemblem
edit-no-badge = Ohne Seitenemblem
edit-plate = { $plate } · { $colours }
edit-odometer = { $km } km auf dem Tacho
edit-wear = { $part } { $percent } % verschlissen

# Profile tools
profile-tools-open = Profilwerkzeuge…
profile-tools-title = Profil „{ $name }“
profile-tools-backup = Profil sichern
profile-tools-backup-description = Eine Kopie des ganzen Profils (jeder Spielstand, seine Fotos und Einstellungen), bei den anderen Sicherungen aufbewahrt.
profile-tools-backup-now = Jetzt sichern
profile-tools-backed-up = Gesichert ({ $size } MB).
profile-tools-copy = Kopie anlegen
profile-tools-copy-description = Ein zweites Profil mit allem aus diesem, unter neuem Namen. Es wird auf diesem PC angelegt (Steam Cloud zeigt nur Profile, die das Spiel angelegt hat), und ohne deine World-of-Trucks-Anmeldung: Verknüpfe es im Spiel, wenn du willst.
profile-tools-copy-suggestion = { $name } Kopie
profile-tools-copy-now = Kopie anlegen
profile-tools-copied = „{ $name }“ angelegt. Es ist in der Profilliste des Spiels.
profile-tools-copy-name-empty = Gib der Kopie einen Namen.
profile-tools-copy-name-taken = Es gibt bereits ein Profil mit diesem Namen.
profile-tools-copy-no-documents = Der Ordner des Spiels in Dokumente wurde nicht gefunden. Starte das Spiel einmal und versuche es dann erneut.
profile-tools-settings = Einstellungen kopieren
profile-tools-settings-description = Grafik, Steuerung und Spieloptionen dieses Profils, über die eines anderen Profils. Die eigenen Einstellungen des anderen Profils werden zuerst gesichert.
profile-tools-settings-pick = Profil wählen…
profile-tools-settings-now = Einstellungen kopieren
profile-tools-settings-copied = Nach „{ $name }“ kopiert.
profile-tools-settings-none = Es gibt kein anderes Profil, in das die Einstellungen kopiert werden können.
profile-tools-running = Schließe zuerst das Spiel: Es schreibt Profile beim Beenden.
profile-tools-failed = Das hat nicht geklappt: { $error }

# The player's truck and position
player-truck = Dein Lkw
player-truck-yours = Diesen fährst du selbst
player-drive = Selbst fahren
player-drive-tooltip = Macht ihn zu deinem Lkw: Wer ihn gefahren hat, bekommt deinen, und seine Garage wird dein Hauptquartier. Er steht an deiner Position, wenn du den Spielstand lädst.
player-why-on-job = Beende oder brich zuerst den laufenden Auftrag ab.
player-why-not-in-garage = Nur ein Lkw in einer Garage kann dein Lkw werden.
player-position = Deine Position
player-position-somewhere = Irgendwo auf der Karte
player-position-from = { $place }, wie in „{ $save }“
player-position-now = Dein Lkw ist in { $place }.
player-quick-travel = Schnellreise beim Laden
player-quick-travel-tooltip = Beim Laden meldet das Spiel „Game change detected“ und lässt dich für deinen Lkw (und Bus) kostenlos eine beliebige besuchte Stadt wählen. Während eines Auftrags geht das nicht, also wird nichts abgebrochen.
player-quick-travel-asked = Beim Laden meldet das Spiel „Game change detected“ und lässt dich kostenlos eine Stadt für deinen Lkw (und Bus) wählen. Mit Rückgängig lässt es sich zurücknehmen.
player-quick-travel-after = Schnellreise beim Laden
player-quick-travel-unavailable = Dieser Spielstand kann das Spiel nicht um eine Schnellreise bitten (eine ältere Spielversion).
player-move = Wo ein anderer Spielstand ist…
player-move-pick = Spielstand wählen…
player-move-description = Dein Lkw wird dorthin versetzt, wo er in einem anderen Spielstand dieses Profils steht.
player-move-reading = „{ $save }“ wird gelesen…
player-move-no-position = „{ $save }“ sagt nicht, wo dein Lkw ist.
player-move-hitched = Kupple zuerst deinen Auflieger im Spiel ab: Er bräuchte auch einen Platz.

# Your day
day-section = Dein Tag
day-description = Wie lange du gefahren bist, dein Polizeiregister und die Uhr. Änderungen hier werden beim Speichern geschrieben, wie der Rest.
day-since-break = Seit deiner letzten Pause
day-rest = Ausruhen
day-rest-tooltip = Als hättest du geschlafen: keine Müdigkeit und keine fällige Pause.
day-rest-what = Müdigkeit
day-rested = Ausgeruht
day-record = Polizeiregister
day-record-summary = { $count ->
    [0] Sauber
    [one] Ein Verstoß, { $amount } Bußgeld
   *[other] { $count } Verstöße, { $amount } Bußgeld
}
day-record-clear-button = Register löschen
day-record-tooltip = Entfernt jeden Verstoß aus dem Register. Bereits bezahlte Bußgelder bleiben bezahlt.
day-record-clear = Sauber
day-cargo-damage = Frachtbeschädigung
day-repair-cargo = Fracht reparieren
day-time = Uhrzeit
day-time-on-job = Die Uhr vorzustellen würde deinen Auftrag verspäten.
day-skip-morning = Morgen (08:00)
day-skip-noon = Mittag (12:00)
day-skip-evening = Abend (18:00)
day-skip-night = Nacht (22:00)
day-and-time = { $day } { $time }
day-monday = Montag
day-tuesday = Dienstag
day-wednesday = Mittwoch
day-thursday = Donnerstag
day-friday = Freitag
day-saturday = Samstag
day-sunday = Sonntag
duration-minutes = { $minutes } min
duration-hours = { $hours } h { $minutes } min
player-trailer = Dein Auflieger
player-trailer-none = Keiner
player-trailer-yours = Diesen ziehst du selbst
player-pull = Selbst ziehen
player-pull-tooltip = Macht ihn zu deinem Auflieger. Er ist nicht angekuppelt: Kupple ihn im Spiel an.

# Picking rows
picks-count = { $count ->
    [one] Einer ausgewählt
   *[other] { $count } ausgewählt
}
picks-max = Ihre Fähigkeiten maximieren
picks-large = Groß ausbauen
picks-move = In Garage verlegen…
picks-refuel = Volltanken
picks-remove = Entfernen
picks-moved = { $moved } von { $count } verlegt: Die anderen passten dort nicht hin.
picks-removed = { $removed } von { $count } entfernt: Die übrigen gehen nicht (öffne einen, um zu sehen, warum).
picks-moved-all = { $count ->
    [one] Verlegt.
   *[other] Alle { $count } verlegt.
}
picks-removed-all = { $count ->
    [one] Entfernt.
   *[other] Alle { $count } entfernt.
}
picks-clear = Auswahl aufheben
picks-done-repair = { $count ->
    [one] 1 Fahrzeug repariert
   *[other] { $count } Fahrzeuge repariert
}
picks-done-refuel = { $count ->
    [one] 1 Tank gefüllt
   *[other] { $count } Tanks gefüllt
}
picks-done-max = { $count ->
    [one] Fähigkeiten von 1 Fahrer maximiert
   *[other] Fähigkeiten von { $count } Fahrern maximiert
}
picks-done-large = { $count ->
    [one] 1 Garage vergrößert
   *[other] { $count } Garagen vergrößert
}
picks-done-body = Mit Rückgängig lässt es sich zurücknehmen. Geschrieben wird erst beim Speichern.
picks-all = Jede angezeigte Zeile auswählen
picks-row = { $name } auswählen

## Statistics
page-statistics = Statistiken
stats-deliveries = Abgeschlossene Aufträge
stats-income = Vergütung aus Aufträgen
stats-xp = XP aus Aufträgen
stats-driven = Insgesamt gefahren
stats-days = Tage unterwegs
stats-average = Durchschnitt pro Auftrag
stats-favourite = Am häufigsten transportiert
stats-favourite-value = { $cargo } ({ $count })
stats-cancelled = Abgebrochene Aufträge
stats-note = Aus dem Auftragsbuch des Spiels. Nur lesbar: Das Spiel führt diese Zahlen selbst.
stats-search = Nach Fracht oder Stadt suchen
stats-filter-damaged = Beschädigte Fracht
stats-table = Abgeschlossene Aufträge
stats-none = Noch keine abgeschlossenen Aufträge
stats-none-body = Jede Fracht, die du lieferst, erscheint hier, die neueste zuerst.
stats-when = { $days ->
    [0] Heute
    [1] Gestern
   *[other] Vor { $days } Tagen
}
column-when = Wann
column-income = Vergütung
column-xp = XP
column-damage = Beschädigung

## What the save needs
needs = DLCs und Mods
needs-description = { $mods ->
    [0] Dieser Spielstand wurde mit { $dlcs } DLCs angelegt. Das Spiel lädt ihn nur, wenn die nötigen davon installiert sind.
   *[other] Dieser Spielstand wurde mit { $dlcs } DLCs und { $mods } Mods angelegt. Das Spiel lädt ihn nur, wenn die nötigen davon installiert sind.
}
needs-unknown = Ohne das Spiel nicht prüfbar. Wähle seinen Ordner in den Einstellungen.
needs-all-here = Alles, womit er angelegt wurde, ist installiert.
needs-missing = { $count ->
    [one] Eines fehlt, das der Spielstand braucht. Installiere es, sonst lädt das Spiel diesen Spielstand nicht:
   *[other] { $count } fehlen, die der Spielstand braucht. Installiere sie, sonst lädt das Spiel diesen Spielstand nicht:
}
needs-missing-extras = { $count ->
    [one] Ein Extra fehlt (der Spielstand lädt auch ohne):
   *[other] { $count } Extras fehlen (der Spielstand lädt auch ohne sie):
}
needs-mods = Mods
needs-installed = Installiert
needs-not-installed = Nicht installiert
