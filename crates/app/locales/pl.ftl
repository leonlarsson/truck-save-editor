# Polish. Drafted by machine translation from en.ftl; it needs review by a native speaker
# (wording, plural forms, and the game's own Polish terms).
#
# Polski tekst dla Truck Save Editor.
#
# Format Fluent: https://projectfluent.org/fluent/guide/
# Jedno pojęcie, jedna nazwa: to samo słowo dla tej samej rzeczy wszędzie.
# Liczby w $count wybierają formę liczby mnogiej; kwoty przychodzą już sformatowane.
# Każdy komunikat z en.ftl jest tutaj, z tymi samymi zmiennymi (sprawdza to test).

## Start screen

start-choose = Wybierz zapis gry do edycji
start-looking = Szukam twoich zapisów…
start-none-title = Nie znaleziono zapisów
start-none-body = W żadnym z miejsc poniżej nie ma profilu gry. Jeśli grasz na tym komputerze, uruchom grę i zapisz ją raz: lista odświeży się, gdy wrócisz do tego okna. Jeśli twoje zapisy są gdzie indziej, na przykład w kopii zapasowej lub na innym dysku, dodaj ten folder.
start-look-again = Szukaj ponownie
start-your-saves = Twoje zapisy
start-game-saves = Zapisane przez grę
start-game-saves-hint = Autozapisy i szybkie zapisy. Gra zastępuje je w trakcie gry.
start-game-saves-more = { $count ->
    [one] Pokaż jeszcze 1
    [few] Pokaż jeszcze { $count }
    [many] Pokaż jeszcze { $count }
   *[other] Pokaż jeszcze { $count }
}
start-game-saves-fewer = Pokaż mniej
start-showing-cloud = Profil: { $name } · zapisy: { $count }. Są w Steam Cloud, więc są takie same na każdym komputerze, na którym grasz.
start-showing-local = Profil: { $name } · zapisy: { $count }. Są tylko na tym komputerze.
start-profile-empty-title = Brak zapisów w profilu: { $name }
start-profile-empty-body = Zapisz raz grę, a zapis pojawi się tutaj. Lista odświeża się, gdy wracasz do tego okna.
start-folders = Gdzie szukane są zapisy
start-folders-description = Gra trzyma każdy profil w Steam Cloud albo na tym komputerze, zależnie od ustawień profilu w grze.
start-folders-missing = { $count ->
    [one] Sprawdzono też 1 folder, którego nie ma na tym komputerze
    [few] Sprawdzono też { $count } foldery, których nie ma na tym komputerze
    [many] Sprawdzono też { $count } folderów, których nie ma na tym komputerze
   *[other] Sprawdzono też { $count } folderu, których nie ma na tym komputerze
}
start-folders-missing-hide = Ukryj foldery, których nie ma
start-folder-added = Dodany przez ciebie
start-folder-profiles = { $count ->
    [one] 1 profil
    [few] { $count } profile
    [many] { $count } profili
   *[other] { $count } profilu
}
start-folder-empty = Brak profili
start-folder-missing = Nie istnieje
start-folder-no-steam = Nie znaleziono Steam na tym komputerze
start-folder-show = Pokaż folder
start-folder-remove = Przestań tu szukać
start-folder-add = Dodaj folder…
start-folder-drop = Albo upuść folder zapisu lub profilu na to okno.
start-folder-choose = Wybierz folder z zapisami
start-folder-not-saves = Brak zapisów lub profili w folderze: { $path }. Wybierz folder „profiles” gry, folder profilu albo zapisu.
start-slot-missing = { $count ->
    [one] Wymaga DLC lub modyfikacji, której nie zainstalowano
    [few] Wymaga DLC lub modyfikacji, których nie zainstalowano: { $count }
   *[many] Wymaga DLC lub modyfikacji, których nie zainstalowano: { $count }
}
start-slot-number = Slot { $number }
profile-place = { $name } · { $place }
profile-game-place = { $name } · { $game } · { $place }
start-game-detail = { $profiles ->
    [one] Jeden profil
    [few] { $profiles } profile
    [many] { $profiles } profili
   *[other] { $profiles } profilu
} · zapisy: { $saves }
start-game-none = Nie znaleziono na tym komputerze
location-steam-cloud = Steam Cloud
location-this-pc = Ten komputer
location-loose = Pojedynczy zapis

## Save slot names (for saves without a name of their own)

slot-unnamed = Zapis bez nazwy
slot-autosave = Autozapis
slot-autosave-drive = Autozapis podczas jazdy
slot-autosave-job = Autozapis przy zleceniu
slot-quicksave = Szybki zapis
slot-multiplayer-backup = Przed konwojem

## How long ago

time-just-now = przed chwilą
time-minutes-ago = { $count ->
    [one] 1 minutę temu
    [few] { $count } minuty temu
   *[many] { $count } minut temu
}
time-hours-ago = { $count ->
    [one] 1 godzinę temu
    [few] { $count } godziny temu
   *[many] { $count } godzin temu
}
time-days-ago = { $count ->
    [one] 1 dzień temu
    [few] { $count } dni temu
   *[many] { $count } dni temu
}
time-weeks-ago = { $count ->
    [one] 1 tydzień temu
    [few] { $count } tygodnie temu
   *[many] { $count } tygodni temu
}
time-months-ago = { $count ->
    [one] 1 miesiąc temu
    [few] { $count } miesiące temu
   *[many] { $count } miesięcy temu
}
time-years-ago = { $count ->
    [one] 1 rok temu
    [few] { $count } lata temu
   *[many] { $count } lat temu
}

## Sidebar and pages

sidebar-career = Kariera
sidebar-company = Firma
sidebar-map = Mapa
sidebar-advanced = Zaawansowane
page-overview = Przegląd
page-fleet = Flota
page-garages = Garaże
headquarters = Siedziba firmy
headquarters-pick = Wybierz miasto
page-drivers = Kierowcy
page-map = Mapa
page-cities = Miasta
page-world-map = Mapa świata
page-gallery = Galeria
page-all-data = Wszystkie dane
switch-save = Zmień zapis
theme-light = Tryb jasny
theme-dark = Tryb ciemny

## Header

header-subtitle = { $game } · { $place }
changes-count = { $count ->
    [0] Brak zmian
    [one] 1 zmiana
    [few] { $count } zmiany
   *[many] { $count } zmian
}
discard = Odrzuć
save = Zapisz
save-open = Zapisz…
details-open = Szczegóły…
settings-open = Ustawienia…
game-settings-open = Ustawienia gry…
saving = Zapisywanie…

## Opening, closing and saving

opening = Otwieranie zapisu „{ $name }”…
open-failed = Nie udało się otworzyć zapisu „{ $name }”
close-unsaved-title = Zamknąć zapis „{ $name }” bez zapisywania?
close-unsaved-body = { $count ->
    [one] Twoja zmiana przepadnie.
    [few] Twoje { $count } zmiany przepadną.
   *[many] Twoich { $count } zmian przepadnie.
}
close-unsaved-confirm = Zamknij bez zapisywania
quit-unsaved-title = Wyjść bez zapisywania zmian w zapisie „{ $name }”?
quit-unsaved-confirm = Wyjdź bez zapisywania
save-confirm-title = Zapisać zmiany w zapisie „{ $name }”?
cancel = Anuluj
save-as-new-slot = Zapisz jako nowy slot
save-as-new-slot-tooltip = Zapisz zmiany w nowym slocie, a ten zostaw bez zmian.
save-as-new-slot-loose = Ten zapis jest poza profilem, więc nie ma profilu, do którego można dodać slot.
new-slot-name = { $name } (edytowany)
saved-as-new-title = Zapisano i sprawdzono jako nowy zapis „{ $name }”
saved-as-new-body = Zapis „{ $original }” pozostał bez zmian. Teraz edytujesz nowy slot.
save-confirm-note = Najpierw powstaje kopia zapasowa slotu, a po zapisaniu zapis gry jest wczytywany ponownie i sprawdzany. Zamknij grę przed zapisaniem: nadpisuje ona zapisy przy wyjściu.
save-confirm-cloud = Ten profil jest w Steam Cloud: jeśli Steam zapyta, którą kopię zachować, wybierz tę z tego komputera. Zapisz jako nowy slot jest wyłączone, bo gra pokazuje tylko zapisy, które sama utworzyła. Aby zachować ten zapis bez zmian, zapisz na innym.
save-prepare-failed = Nie udało się przygotować zapisu
save-no-backup-folder-title = Nie udało się zapisać
save-no-backup-folder = Nie ma folderu na kopie zapasowe, więc nic nie zostało zapisane.
save-over-other = Zapisz na innym…
save-over-other-tooltip = Zapisz zmiany na innym zapisie tego profilu, a ten zostaw bez zmian. Działa ze Steam Cloud.
save-over-pick-title = Na którym zapisie zapisać?
save-over-pick-body = Wybrany zapis zostanie skopiowany jako kopia zapasowa, a potem zastąpiony zapisem „{ $original }” z twoimi zmianami. Zachowa swoją nazwę. Zapis „{ $original }” pozostanie bez zmian.
save-over-confirm-title = Zastąpić zapis „{ $name }”?
save-over-confirm-body = Najpierw powstanie kopia zapasowa zapisu „{ $name }”, a potem zostanie on zastąpiony zapisem „{ $original }” z twoimi zmianami, pod swoją nazwą. Zapis „{ $original }” pozostanie bez zmian, a dalej edytujesz „{ $name }”.
save-over-confirm = Zastąp
save-over-no-backups = Nie ma gdzie zapisać kopii zapasowej, więc nic nie zostało zapisane.
saved-over-title = Zapisano na „{ $name }” i sprawdzono
saved-over-body = Zapis „{ $original }” jest bez zmian. Teraz edytujesz drugi zapis.
duplicate-tooltip = Duplikuj…
duplicate-cloud = W Steam Cloud gra pokazuje tylko zapisy, które sama utworzyła, więc kopia zrobiona tutaj nie pojawiłaby się w grze. Zapisz grę w nowym slocie w grze albo skopiuj cały profil jako lokalny (Narzędzia profilu).
duplicate-loose = Pojedynczy zapis nie ma profilu, do którego można dodać kopię.
duplicate-title = Zduplikować zapis „{ $name }”?
duplicate-body = Kopia, „{ $copy }”, trafi do nowego slotu profilu. Zapis „{ $name }” pozostanie bez zmian.
duplicate-confirm = Duplikuj
duplicate-name = { $name } (kopia)
duplicated-title = Zduplikowano jako „{ $name }”
duplicated-body = Zapis „{ $original }” jest bez zmian.
duplicate-failed = Nie udało się zduplikować: „{ $name }”
saved-title = Zapisano i sprawdzono zapis „{ $name }”
saved-backup = Poprzednia wersja jest w Kopiach zapasowych.
save-failed = Nie udało się zapisać: „{ $name }”
save-profile-failed = Zapis gry został zapisany, ale nazwa firmy nie: wciąż czeka na zapisanie

## Warnings

newer-game-warning = Ten zapis pochodzi z nowszej wersji gry niż ta, z którą edytor był testowany. Wszystko, czego edytor nie rozumie, zostaje bez zmian, ale sprawdź swoje zmiany w grze i zachowaj kopię zapasową.
ats-warning = Obsługa American Truck Simulator jest nowa. Zapisy otwierają się i zapisują bezpiecznie, z kopią zapasową. Sprawdź swoje zmiany w grze.

## Undo and redo

undo = Cofnij
redo = Ponów
save-tooltip = Przejrzyj zmiany i zapisz (Ctrl+S)
undo-tooltip = Cofnij (Ctrl+Z)
redo-tooltip = Ponów (Ctrl+Shift+Z)

## Jobs

page-jobs = Zlecenia
jobs-summary = { $count ->
    [one] 1 oferta zlecenia
    [few] { $amount } oferty zleceń
   *[many] { $amount } ofert zleceń
}
jobs-offers = Oferty zleceń
jobs-keep = Zachowaj każdą ofertę na tydzień
jobs-keep-tooltip = Każda oferta zlecenia będzie ważna co najmniej tydzień czasu gry, więc ta, której chcesz, wciąż tam będzie.
jobs-kept-week = Zachowane na co najmniej tydzień
jobs-table = Oferty zleceń
jobs-search = Szukaj po mieście, firmie lub ładunku
jobs-filter-my-garages = Z moich garaży
jobs-filter-urgent = Pilne
jobs-filter-no-ferry = Bez promu
column-from = Skąd
column-to = Dokąd
column-cargo = Ładunek
column-distance = Dystans
column-expires = Wygasa
column-urgency = Pilność
job-place = { $company }, { $city }
job-urgency-standard = Standardowe
job-urgency-important = Ważne
job-urgency-urgent = Pilne
new-job-open = Utwórz zlecenie…
new-job-title = Utwórz zlecenie
new-job-description = Przy zapisie dodaje zlecenie na giełdę ładunków: od firmy, która wysyła ładunek, do takiej, która go przyjmuje, na naczepie, na której gra już wozi ten ładunek.
new-job-from = Skąd
new-job-cargo = Ładunek
new-job-to = Dokąd
new-job-trailer-label = Naczepa
new-job-pick = Wybierz…
new-job-company = { $company }, { $city }
new-job-cargo-on = { $cargo } · { $body }
new-job-trailer = { $body }, { $chassis } · { $count ->
    [one] 1 jednostka
    [few] { $count } jednostki
    [many] { $count } jednostek
   *[other] { $count } jednostki
}
new-job-add = Dodaj zlecenie
new-job-added = Zlecenie trafi na giełdę przy zapisie
new-job-incomplete = Wybierz skąd, ładunek, dokąd i naczepę.
new-job-no-route = Brak oferty między tymi miastami, z której można wziąć dystans, a mapa nie jest jeszcze wczytana.
new-job = Nowe zlecenie
new-job-before = Brak
new-job-after = { $cargo }, { $from } → { $to }
job-expires-in = { $minutes ->
    [one] za 1 minutę
    [few] za { $minutes } minuty
   *[many] za { $minutes } minut
}
job-expires-in-hours = { $hours ->
    [one] za 1 godzinę
    [few] za { $hours } godziny
   *[many] za { $hours } godzin
}
job-expires-in-days = { $days ->
    [one] za 1 dzień
    [few] za { $days } dni
   *[many] za { $days } dni
}
job-expired = Wygasło
job-ferry = { $distance } · prom

## Detail panels

detail-truck = Ciężarówka
detail-coach = Autokar
detail-trailer = Naczepa
detail-driver = Kierowca
detail-garage-in = Garaż: { $city }
detail-about = Informacje
detail-gone = Tego już nie ma w zapisie.
part-engine = Silnik
part-transmission = Skrzynia biegów
part-cabin = Kabina
part-chassis = Podwozie
part-body = Nadwozie
part-wheels = Koła
repair-all-parts = Napraw wszystko
fill-tank = Zatankuj do pełna
driver-from = Miasto rodzinne: { $city }
driver-now-in = Obecnie: { $city }
driver-max = Maksymalizuj tego kierowcę
garage-empty = Brak
garage-make-large = Rozbuduj do dużego
garage-buy-large = Kup jako duży

## Filters

filter-count = { $shown } z { $total }
filter-no-matches = Nic nie pasuje
filter-no-matches-body = Spróbuj innego wyszukiwania albo wyczyść filtry.
drivers-none = Nie zatrudniono jeszcze kierowców
drivers-none-body = Zatrudnij kierowców w agencji pracy w grze, a pojawią się tutaj.
jobs-none = Brak ofert zleceń
jobs-none-body = Gra dodaje oferty zleceń podczas jazdy; ten zapis jeszcze żadnych nie ma.
filter-clear = Wyczyść filtry
fleet-search = Szukaj po modelu, rejestracji lub garażu
fleet-filter-worn = Wymaga naprawy
fleet-filter-no-garage = Bez garażu
fleet-filter-yours = Twoje
fleet-yours = Twoja
garages-search = Szukaj po mieście
garages-filter-owned = Posiadane
garages-filter-not-large = Nie duże
drivers-search = Szukaj po mieście
drivers-filter-not-maxed = Niezmaksymalizowani
map-search = Szukaj po mieście
map-filter-not-visited = Nieodwiedzone
map-filter-dealer-locked = Dealer jeszcze nieodkryty
map-filter-recruitment-locked = Agencja jeszcze nieodkryta

## Quick start

quick-start = Szybki start
quick-start-open = Szybki start…
quick-start-description = Typowe zmiany dla nowej kariery, za jednym razem. Załóż nowy profil w grze, otwórz tutaj jego zapis i wybierz, co chcesz. Jedno Cofnij wycofuje wszystko, a nic nie jest zapisywane, dopóki nie zapiszesz.
quick-start-rich = Bogaty start
quick-start-everything = Wszystko odblokowane
quick-start-money = Ustaw pieniądze
quick-start-hq-garage = Duży garaż: { $city }
quick-start-apply = Zastosuj
quick-start-no-coaches = Ten zapis nie ma kariery autokarowej (DLC Coaches).
quick-start-no-hq = Ten zapis nie ma jeszcze miasta siedziby firmy.
quick-start-hq-large = Garaż w siedzibie firmy jest już duży.
quick-start-needs-game = Wymaga gry zainstalowanej na tym komputerze: jej dane mówią, które miasta je mają.

## Settings

page-settings = Ustawienia
page-game-settings = Ustawienia gry
settings-language = Język
settings-language-description = Język aplikacji. Nazwy z samej gry (miasta, ładunki, rangi) też się zmieniają, jeśli gra ma je w tym języku.
settings-appearance = Wygląd
settings-appearance-system = Jak w systemie
settings-appearance-light = Jasny
settings-appearance-dark = Ciemny
settings-game = Folder gry
settings-game-description = Gdzie jest zainstalowany { $game }. Edytor czyta z niego tabele poziomów, nazwy i ikony i nigdy niczego tam nie zmienia.
settings-game-found = Znaleziony automatycznie
settings-game-picked = Wybrany przez ciebie
settings-game-not-found = Nie znaleziono. Wybierz folder, w którym zainstalowana jest gra.
settings-game-not-game = W tym folderze nie ma gry: brakuje pliku def.scs.
settings-game-choose = Wybierz folder…
settings-game-automatic = Znajdź automatycznie
settings-backups = Kopie zapasowe
settings-backups-description = Kopia zapasowa slotu powstaje przy każdym zapisaniu i jest przechowywana tutaj, poza folderami gry.
settings-backups-open = Otwórz folder
game-settings = W grze
game-settings-description = Ustawienia samej gry, w jej pliku config.cfg. Przed każdą zmianą tworzona jest kopia zapasowa pliku.
game-settings-console = Konsola deweloperska (otwierasz ją w grze klawiszem ~)
game-settings-convoy = Konwoje do { $count } ciężarówek
game-settings-running = Zamknij grę, aby to zmienić: zapisuje ona swoje ustawienia przy wyjściu.
game-settings-none = Nie znaleziono pliku ustawień gry. Uruchom grę raz, a potem wróć.
game-settings-failed = Nie udało się zmienić ustawień gry: { $error }
settings-updates = Aktualizacje
settings-updates-description = Edytor może zapytać GitHub, czy wyszła nowsza wersja. Tylko wtedy korzysta z internetu i nie wysyła nic o tobie ani o twoich zapisach. Aktualizacja to pobranie nowej wersji i podmiana pliku .exe; twoje ustawienia i kopie zapasowe zostają.
updates-not-checked = Jeszcze nie sprawdzono.
updates-checking = Sprawdzanie…
updates-up-to-date = Masz najnowszą wersję ({ $version }).
updates-available = Dostępna jest wersja { $version }.
updates-download = Pobierz…
updates-failed = Nie udało się połączyć z GitHubem. Sprawdź połączenie i spróbuj ponownie.
updates-check-now = Sprawdź teraz
updates-check-on-start = Sprawdzaj przy uruchomieniu edytora
updates-banner = Dostępna jest wersja { $version }
settings-reset = Resetuj edytor
settings-reset-description = Przywraca ustawienia edytora do stanu z pierwszego uruchomienia: język, wygląd, folder gry, foldery dodane na ekranie startowym i sprawdzanie aktualizacji. Twoje zapisy i kopie zapasowe pozostają bez zmian.
settings-reset-button = Resetuj ustawienia…
settings-reset-confirm-title = Zresetować ustawienia edytora?
settings-reset-confirm = Resetuj
settings-about = Informacje
settings-version = Wersja { $version }
settings-source = Kod źródłowy na GitHubie
licences-open = Licencje open source
licences-title = Licencje open source
licences-description = Edytor jest zbudowany z kodu innych projektów, na ich licencjach. Najpierw kod do niego dostosowany, potem każda biblioteka w edytorze z jej licencją (otwórz jedną, aby ją przeczytać).
settings-not-affiliated = Niepowiązany z SCS Software ani przez nią niepopierany.

## Command palette

palette-open = Polecenia…
palette-shortcut = Ctrl+K
settings-shortcut = Ctrl+,
palette-placeholder = Wpisz polecenie lub stronę…
palette-group-pages = Przejdź do
palette-group-actions = Akcje
palette-group-app = Zapis i aplikacja
palette-go = { $page }
palette-max-truck-skills = Maksymalizuj wszystkie zdolności kierowcy ciężarówki
palette-max-coach-skills = Maksymalizuj wszystkie zdolności kierowcy autokaru
palette-all-adr = { $game ->
    [ats] Wszystkie klasy zagrożenia
   *[other] Wszystkie klasy ADR
}
palette-repair-trucks = Napraw wszystkie ciężarówki
palette-repair-coaches = Napraw wszystkie autokary
palette-repair-trailers = Napraw wszystkie naczepy
palette-refuel-all = Zatankuj wszystkie zbiorniki
palette-save = Zapisz…
palette-buy-all-garages = Kup każdy garaż, wszystkie duże
palette-upgrade-owned-garages = Rozbuduj posiadane garaże do dużych
palette-discard = Odrzuć zmiany
palette-theme = Przełącz tryb jasny lub ciemny
report-problem = Zgłoś problem…
report-this = Zgłoś ten problem
report-description = Otwiera nowe zgłoszenie na GitHubie w przeglądarce, z już wpisaną wersją edytora. Nic nie zostanie wysłane, dopóki go nie opublikujesz.
crash-title = Ostatnim razem edytor nieoczekiwanie się zamknął
crash-body = Coś poszło nie tak i musiał się zatrzymać. Twoje zapisy nie zostały naruszone: nic nie jest zapisywane bez kopii zapasowej. Zgłoszenie otwiera zgłoszenie na GitHubie z opisem zdarzenia, do sprawdzenia i opublikowania.
crash-dismiss = Zamknij
palette-report = Zgłoś problem na GitHubie

## Backups

backups = Kopie zapasowe
backups-intro = Kopia zapasowa slotu powstaje przy każdym zapisaniu i jest przechowywana poza folderami gry. Przywracanie najpierw tworzy kopię tego, co jest tam teraz, więc też można je cofnąć.
backups-none = Brak kopii zapasowych
backups-none-body = Pierwsza powstanie, gdy po raz pierwszy zapiszesz ten slot.
backup-made-at = Utworzona { $time } (UTC)
backup-show = Pokaż w folderze
backup-restore = Przywróć
restore-title = Przywrócić zapis „{ $name }” z tej kopii zapasowej?
restore-body = { $changes ->
    [0] Najpierw powstanie kopia zapasowa tego, co jest teraz w slocie.
    [one] Najpierw powstanie kopia zapasowa tego, co jest teraz w slocie. Twoja niezapisana zmiana przepadnie.
    [few] Najpierw powstanie kopia zapasowa tego, co jest teraz w slocie. Twoje { $changes } niezapisane zmiany przepadną.
   *[many] Najpierw powstanie kopia zapasowa tego, co jest teraz w slocie. Twoich { $changes } niezapisanych zmian przepadnie.
}
restored-title = Przywrócono zapis „{ $name }”
restored-body = Slot jest znów otwarty w stanie z kopii zapasowej.
restore-failed = Nie udało się przywrócić zapisu „{ $name }”

## Overview

amount-hint = Spróbuj 2.5m, 10k, +1m lub -500k.
company-section = Twoja firma
profile-name = Nazwa profilu
company-name = Nazwa firmy
company-name-description = Nazwa twojej firmy i twojego profilu, dla całego profilu: każdego zapisu w nim. Są przechowywane w pliku profilu, którego kopia zapasowa powstaje przed zapisaniem.
money = Pieniądze
money-description = Saldo konta twojej firmy.
money-label = Saldo konta
experience = Doświadczenie kierowcy ciężarówki
experience-description = Twój poziom jako kierowcy ciężarówki.
experience-label = Punkty doświadczenia kierowcy ciężarówki
coach-experience = Doświadczenie kierowcy autokaru
coach-experience-description = Autokary to osobna kariera.
coach-experience-label = Punkty doświadczenia kierowcy autokaru
experience-unit = XP
experience-amount = { $amount } XP
skills = Zdolności kierowcy ciężarówki
level-number = Poziom { $level }
level-progress = { $percent }% do poziomu { $next }
level-progress-label = Postęp do następnego poziomu
level-step = { $count ->
    [one] 1 poziom
    [few] { $count } poziomy
   *[many] { $count } poziomów
}
coach-skills = Zdolności kierowcy autokaru
coach-skills-description = Każda zdolność kierowcy autokaru ma do 3 poziomów.
coach-services = Kursy autokarowe
coach-services-description = Twoja kariera kierowcy autokaru tak, jak pokazują ją statystyki gry. Gra wylicza je z zapisanych kursów, więc pasażerowie są rozdzielani między nie.
coach-service-count = Kursy
coach-passengers = Przewiezieni pasażerowie
coach-average-satisfaction = Średnie zadowolenie
coach-best-satisfaction = Najlepsze zadowolenie
coach-distance = Dystans autokarem
coach-stops = Odkryte przystanki
coach-stops-found = { $found } z { $count }
coach-perfect-satisfaction = Pełne zadowolenie
coach-perfect-satisfaction-tooltip = Pasażerowie każdego zapisanego kursu wysiadają w pełni zadowoleni, więc średnia i najlepszy wynik to 100 %.
coach-discover-stops = Odkryj wszystkie przystanki
coach-discover-stops-tooltip = Oznacza każdy przystanek na mapie jako odkryty, jakbyś był na każdym.
coach-skill-longer-duties = Dłuższe kursy
coach-skill-longer-duties-description = Więcej przystanków na kurs
coach-skill-demanding-timetables = Wymagające rozkłady jazdy
coach-skill-demanding-timetables-description = Surowsze rozkłady, za które jest więcej PD
coach-skill-premium-service = Obsługa premium
coach-skill-premium-service-description = Pasażerowie, którzy płacą więcej, ale mniej wybaczają
coach-skill-cushioned-journey = Komfortowa jazda
coach-skill-cushioned-journey-description = Płynna jazda daje więcej pieniędzy i PD
coach-skill-passenger-favourite = Ulubieniec pasażerów
coach-skill-passenger-favourite-description = Bezbłędna jazda podnosi zadowolenie
coach-skill-ecodriving = Ekologiczna jazda
coach-skill-ecodriving-description = Mniejsze zużycie paliwa
skills-description = Kliknij poziom, aby go ustawić. Kliknij ponownie najwyższy wypełniony poziom, aby go obniżyć.
skills-max = Maksymalizuj wszystkie zdolności
skill-level = Poziom { $level }
skill-level-of = { $skill }: poziom { $level } z { $max }
skill-long-distance = Długodystansowiec
skill-long-distance-description = Odblokowuje zlecenia na dłuższych trasach
skill-high-value-cargo = Ładunek o wysokiej wartości
skill-high-value-cargo-description = Cenne ładunki, za które płacą więcej
skill-fragile-cargo = Delikatny ładunek
skill-fragile-cargo-description = Ładunki, które łatwo uszkodzić
skill-just-in-time = Dostawa na czas
skill-just-in-time-description = Pilne zlecenia z krótkimi terminami
skill-ecodriving = Ekologiczna jazda
skill-ecodriving-description = Mniejsze zużycie paliwa
adr = { $game ->
    [ats] Niebezpieczny ładunek
   *[other] ADR
}
adr-description = Towary niebezpieczne, które może przewozić twój kierowca.
adr-all = Wszystkie klasy
adr-none = Brak
adr-some = { $count } z { $total } klas
adr-explosives = Materiały wybuchowe
adr-gases = Gazy
adr-flammable-liquids = Ciecze łatwopalne
adr-flammable-solids = Materiały stałe zapalne
adr-toxic = Materiały trujące
adr-corrosive = Substancje żrące
hq = Siedziba

## All data

all-data-intro = Każda jednostka w zapisie i każda wartość w niej. Wybierz rodzaj, potem jednostkę, a potem wartość do zmiany.
all-data-search = Szukaj w każdej wartości, polu i jednostce
all-data-search-count = { $count ->
    [0] Nic nie pasuje
    [one] 1 trafienie
    [few] { $amount } trafienia
   *[many] { $amount } trafień
}
all-data-search-more = Pierwsze trafienia: { $count }
all-data-results = Wyniki wyszukiwania
all-data-kinds = Rodzaje
all-data-units = Jednostki
all-data-values = Wartości
all-data-filter = Filtruj rodzaje
all-data-kind = Rodzaj
all-data-count = Jednostki
all-data-unit = Jednostka
all-data-field = Pole
all-data-value = Wartość
all-data-items = { $count ->
    [one] 1 element
    [few] { $count } elementy
   *[many] { $count } elementów
}
all-data-pick-value = Wybierz wartość
all-data-apply = Zastosuj
all-data-revert = Przywróć poprzednią
all-data-wrong-shape = Ta wartość przechowuje { $expected }; wpisz coś takiego (tekst zachowuje cudzysłowy).
shape-bool = true lub false
shape-number = liczbę
shape-text = tekst w cudzysłowie
shape-id = identyfikator lub nazwę
shape-vector = liczby w nawiasach
shape-value = wartość

## Map

map-summary = Odwiedzone miasta: { $visited } z { $count }
map-visit-all = Odwiedź każde miasto
map-visit-all-tooltip = Oznacz każde miasto jako odwiedzone, jakbyś przejechał przez każde z nich.
map-visits = { $count ->
    [0] Nieodwiedzone
    [one] Raz
    [few] { $count } razy
   *[many] { $count } razy
}
column-visits = Odwiedzone
map-table = Miasta
map-dealers = Dealerzy ciężarówek
map-recruitments = Agencje pracy
map-dealers-summary = { $unlocked } z { $count ->
    [one] 1 dealera ciężarówek
    [few] { $count } dealerów ciężarówek
   *[many] { $count } dealerów ciężarówek
}
map-recruitments-summary = { $unlocked } z { $count ->
    [one] 1 agencji pracy
    [few] { $count } agencji pracy
   *[many] { $count } agencji pracy
}
map-unlocked-count = Odblokowane: { $count }
map-unlocked-of = Odblokowane: { $unlocked } z { $count }
map-unlock-dealers = Odblokuj każdego dealera
map-unlock-dealers-tooltip = Odblokuj dealera ciężarówek w każdym mieście, które go ma, jakbyś przejechał obok każdego.
map-unlock-recruitments = Odblokuj każdą agencję
agency-unlock-off = Gra musi sama odkryć agencję pracy: przejedź obok niej. Odblokowana w zapisie pozostaje w grze zamknięta (czarny ekran po wejściu).
agency-lock-tooltip = Zablokuj ją ponownie, jakbyś nigdy obok niej nie przejeżdżał. Jeśli jej ekran w grze pozostaje czarny, zablokuj ją tutaj i przejedź obok: gra odkryje ją na nowo i się otworzy.
map-needs-game = Odblokowanie dealerów i agencji wymaga gry zainstalowanej na tym komputerze: jej dane mówią, które miasta je mają.
map-agencies-reading = To, które miasta mają agencję pracy, jest odczytywane z mapy gry, chwilę po reszcie jej danych.
column-dealer = Dealer ciężarówek
column-recruitment = Agencja pracy
map-place-unlocked = Odblokowane
map-place-locked = Jeszcze nieodkryte
city-place-missing = Brak w tym mieście
city-place-unknown = Nieznane
city-visit = Oznacz jako odwiedzone
city-visit-tooltip = Oznacz miasto jako odwiedzone, jakbyś przez nie przejechał.
city-unlock = Odblokuj
city-unlock-tooltip = Odblokuj, jakbyś przejechał obok.
city-unvisit = Oznacz jako nieodwiedzone
city-unvisit-tooltip = Oznacz miasto jako nigdy nieodwiedzone, jak w nowym profilu. Gra policzy je ponownie, gdy następnym razem przez nie przejedziesz.
city-lock = Zablokuj
city-lock-tooltip = Zablokuj ponownie, jakbyś nigdy obok nie przejechał.
city-open-garage = Otwórz
city-show-on-map = Pokaż na mapie

## Garages

garages = Garaże
garages-before = Tak jak były
garages-after = { $bought ->
    [0] { $upgraded ->
        [one] 1 rozbudowany do dużego
        [few] { $upgraded } rozbudowane do dużych
       *[many] { $upgraded } rozbudowanych do dużych
    }
   *[other] { $upgraded ->
        [0] Kupione: { $bought }, wszystkie duże
       *[other] Kupione: { $bought }, rozbudowane: { $upgraded }, wszystkie duże
    }
}
garages-summary = { $count ->
    [one] 1 miasto
    [few] { $count } miasta
   *[many] { $count } miast
} · posiadane: { $owned } · duże: { $large }
garages-upgrade-owned = Rozbuduj posiadane
garages-upgrade-owned-tooltip = Rozbuduj każdy posiadany garaż do dużego: 5 ciężarówek i 5 kierowców.
garages-buy-all = Kup wszystkie
garages-buy-all-tooltip = Kup garaż w każdym mieście, wszystkie duże.
garage-not-owned = Nieposiadany
garage-large = Duży
garage-slots = { $count ->
    [one] 1 miejsce
    [few] { $count } miejsca
   *[many] { $count } miejsc
}
garage-filled = { $filled } z { $slots }
column-city = Miasto
column-garage-size = Garaż
column-trucks = Ciężarówki
column-drivers = Kierowcy
garages-table = Garaże

## Drivers

drivers-summary = { $count ->
    [0] Nie zatrudniono jeszcze kierowców
    [one] 1 kierowca
    [few] { $count } kierowców
   *[many] { $count } kierowców
} · { $maxed ->
    [0] żaden w pełni wyszkolony
   *[other] w pełni wyszkoleni: { $maxed }
}
drivers-max-all = Maksymalizuj wszystkich kierowców
drivers-max-all-tooltip = { $game ->
    [ats] Każda zdolność na najwyższym poziomie i wszystkie klasy zagrożenia, dla każdego zatrudnionego kierowcy.
   *[other] Każda zdolność na najwyższym poziomie i wszystkie klasy ADR, dla każdego zatrudnionego kierowcy.
}
drivers-maxed = { $count ->
    [one] Zmaksymalizowano 1 kierowcę
    [few] Zmaksymalizowano { $count } kierowców
   *[many] Zmaksymalizowano { $count } kierowców
}
column-hometown = Miasto rodzinne
column-location = Teraz w
column-experience = Doświadczenie
column-skills = Zdolności
column-adr = { $game ->
    [ats] Niebezpieczny ładunek
   *[other] ADR
}
drivers-table = Kierowcy
skills-short = { $long } · { $value } · { $fragile } · { $urgent } · { $eco }
adr-count = { $count } z { $total }

## Fleet

fleet-trucks = { $count ->
    [one] 1 ciężarówka
    [few] { $amount } ciężarówki
   *[many] { $amount } ciężarówek
}
fleet-worn = { $count ->
    [0] wszystkie w dobrym stanie
    [one] 1 wymaga naprawy
    [few] { $amount } wymagają naprawy
   *[many] { $amount } wymaga naprawy
}
fleet-coaches = { $count ->
    [one] 1 autokar
    [few] { $amount } autokary
   *[many] { $amount } autokarów
}
fleet-tab-trucks = Ciężarówki · { $count }
fleet-tab-coaches = Autokary · { $count }
fleet-summary = { $vehicles } · { $worn }
fleet-none = Nie masz jeszcze własnych ciężarówek
fleet-none-body = Kup ciężarówkę u dealera w grze, a pojawi się tutaj.
repair-all = Napraw wszystkie
repairs = Naprawy
repairs-before = Zużyte
repairs-after = { $count ->
    [one] Naprawiono 1 pojazd
    [few] Naprawiono { $count } pojazdy
   *[many] Naprawiono { $count } pojazdów
}
column-truck = Model
column-plate = Rejestracja
column-garage = Garaż
column-odometer = Przebieg
column-wear = Zużycie
repair = Napraw
refuel-all = Zatankuj wszystkie
refuel-all-tooltip = Zatankuj do pełna każdą ciężarówkę lub autokar z tej listy.
column-fuel = Paliwo
vehicle-in-use = W trasie
vehicle-parked = Zaparkowany: { $city }
fuel = Paliwo
fuel-before = Tak jak było
fuel-after = { $count ->
    [one] Zatankowano 1 zbiornik
    [few] Zatankowano { $count } zbiorniki
   *[many] Zatankowano { $count } zbiorników
}
fleet-trailers = { $count ->
    [one] 1 naczepa
    [few] { $amount } naczepy
   *[many] { $amount } naczep
}
fleet-tab-trailers = Naczepy · { $count }
no-garage = Bez garażu
percent = { $amount }%
amount-with-unit = { $amount } { $unit }
unit-km = km
unit-mi = mil
unit-kg = kg
unit-lb = lb
unit-t = t
trucks-table = Pojazdy
unknown-truck = Nieznany model

# Moving trucks and drivers; the current job
move-garage-full = { $city } · pełny
move-what-truck = { $model } · { $plate }
move-what-driver = Kierowca · { $city }
driver-truck = Ciężarówka
driver-truck-none = Brak ciężarówki
driver-truck-open = Otwórz ciężarówkę
garage-driver-open = Otwórz kierowcę
move-hint = Wybierz inny garaż, aby go tam przenieść.
move-with-driver = Jej kierowca przenosi się razem z nią.
move-with-truck = Jego ciężarówka przenosi się razem z nim.
move-no-room = Żaden inny garaż nie ma wolnego miejsca. Kup lub rozbuduj jakiś w zakładce Garaże.
move-is-players = Tą jeździsz ty: jedzie tam, gdzie ją zabierzesz.
current-job = Bieżące zlecenie
current-job-summary = { $cargo } · cel: { $city }
current-job-no-deadline = Bez terminu
current-job-cancelled = Anulowane
current-job-deadline = Termin
current-job-late = Spóźnione
current-job-remove-deadline = Usuń termin
current-job-cancel = Anuluj zlecenie
current-job-cancelled-note = Anulowane przy zapisaniu, bez opłaty za anulowanie. Cofnij przywraca je.
current-job-freight-market = Z rynku przesyłek, twoją ciężarówką z naczepą firmy. Anulowanie tutaj nic nie kosztuje.
current-job-external = Zlecenie zewnętrzne, twoją ciężarówką z naczepą firmy. Anulowanie tutaj nic nie kosztuje.
current-job-quick-job = Szybkie zlecenie ciężarówką firmy. Edytor nie umie jeszcze go anulować: anuluj je w grze.
current-job-own-trailer = Zlecenie z rynku ładunków z twoją własną naczepą. Edytor nie umie jeszcze go anulować: anuluj je w grze.
current-job-special = Transport specjalny. Edytor nie może go anulować: anuluj go w grze.

# Removing from the company; roads
remove-truck = Usuń ciężarówkę
remove-trailer = Usuń naczepę
remove-driver = Usuń kierowcę
remove-garage = Usuń garaż
remove-tooltip = Usuwa to z twojej firmy przy zapisaniu. W przeciwieństwie do sprzedaży w grze nie dostajesz za to pieniędzy. Cofnij przywraca je.
remove-why-yours = Tą jeździsz ty: zostaje.
remove-why-your-trailer = Ciągniesz tę naczepę: zostaje.
remove-why-has-driver = Tą ciężarówką jeździ kierowca. Najpierw usuń lub przenieś kierowcę.
remove-why-in-use = Ta naczepa jest używana w bieżącym zleceniu.
remove-why-headquarters = To twoja siedziba firmy. Najpierw ustaw inny garaż jako siedzibę firmy.
remove-why-not-empty = Najpierw przenieś lub usuń jego ciężarówki, kierowców i naczepy.
remove-why-unknown = Listy w tym zapisie mają kształt, którego edytor nie zna (nowsza wersja gry?), więc nie będzie zgadywać.
removal-before = W twojej firmie
removal-after = Usunięte
copy-to-garage = Kopiuj do garażu
copy-tooltip = Dodaje kopię do twojej firmy przy zapisie, w wybranym garażu: te same części, lakier, tablica i zużycie. Cofnij odwraca zmianę.
copy-no-room = Żaden z twoich garaży nie ma miejsca na kolejny: kup lub rozbuduj garaż w zakładce Garaże albo zwolnij miejsce.
copy-change = Nowa kopia
copy-change-before = Brak
buy-truck = Kup ciężarówkę…
buy-truck-title = Kup ciężarówkę
buy-truck-description = Dodaje nową ciężarówkę do twojej firmy przy zapisie, tak jak dealerzy w grze sprzedają ją gotową, w garażu z wolnym miejscem. Jest za darmo: żadne pieniądze nie są pobierane. Cofnij odwraca zmianę.
buy-truck-model = Ciężarówka
buy-truck-option = { $model } · { $chassis } · { $engine } · { $price }
buy-truck-buy = Kup
buy-truck-added = Ciężarówka kupiona: dołączy do twojej firmy przy zapisie.
buy-truck-choose = Wybierz ciężarówkę i garaż.
buy-truck-reading = Odczytywanie dealerów gry…
buy-truck-needs-game = Ciężarówki pochodzą od dealerów gry, co wymaga gry zainstalowanej na tym komputerze (zobacz Ustawienia gry).
buy-truck-no-room = Żaden z twoich garaży nie ma wolnego miejsca na ciężarówkę: kup lub rozbuduj garaż w zakładce Garaże albo zwolnij miejsce.
buy-change = Nowa ciężarówka
change-career = Kariera
change-you = Ty
change-cities = Miasta
change-wear = Zużycie: { $part }
change-on = Jest
changes-none = Nic nie różni się od zapisu na dysku.
roads-what = Odwiedzone drogi
roads-count = { $count ->
    [one] Jeden odcinek drogi
    [few] { $amount } odcinki dróg
   *[many] { $amount } odcinków dróg
}
roads-none = Brak
map-forget-roads = Zapomnij odwiedzone drogi
map-forget-roads-tooltip = Odkryte drogi na mapie zaczynają się od nowa, jak na nowym profilu. Odwiedzone miasta pozostają odwiedzone.
map-discover-roads = Odkryj wszystkie drogi
map-discover-roads-tooltip = Oznacza każdą drogę i skrzyżowanie na mapie jako przejechane, także w twoich DLC z mapami, na podstawie mapy z gry. Miasta, w których nie byłeś, pozostają nieodwiedzone.
map-discover-roads-needs-game = Wymaga gry zainstalowanej na tym komputerze: jej mapa mówi, gdzie są drogi.
map-discover-roads-confirm-title = Odkryć wszystkie drogi?
map-discover-roads-confirm-body = Każda droga i skrzyżowanie na mapie, także w twoich DLC z mapami, zostaną oznaczone jako przejechane. Gra może zaliczyć te drogi do swoich osiągnięć{ $game ->
    [ats] {""}
   *[other] , np. Pathfinder (40 000 km odkrytych dróg)
}, ale nie wiadomo, czy i kiedy to zrobi. Cofnij odwraca zmianę do czasu zapisu, a zapis jest najpierw kopiowany.
map-view-driven = Przejechane
map-view-not-driven = Jeszcze nie przejechane
map-view-truck = Twoja ciężarówka
map-view-garages = Twoje garaże
map-view-names = Nazwy miast
map-view-hide = Kliknij, aby ukryć
map-view-show = Kliknij, aby pokazać
map-view-drawing = Rysowanie mapy…
map-view-needs-game = Wymaga gry zainstalowanej na tym komputerze: mapa pochodzi z jej plików.
map-view-hint = Przewiń, aby powiększyć, przeciągnij, aby przesunąć, kliknij, aby otworzyć
map-view-zoom-in = Powiększ
map-view-zoom-out = Pomniejsz
map-view-fit = Cała mapa
map-view-find = Znajdź miasto…

# Truck and trailer details
plate-colours = Kolory tablicy
plate-colours-pick = Wybierz…
plate-world-of-trucks = Twoja ciężarówka zawsze ma tablicę z World of Trucks: gra zakłada ją na ciężarówkę, która jest twoja, przy każdym wczytaniu zapisu. Zmień ją na World of Trucks: My page → Profile customization (Game default oddaje tablicę z powrotem grze).
plate-plain = Tablica z gry
plate-black-on-yellow = Czarne na żółtym
plate-black-on-white = Czarne na białym
plate-white-on-black = Białe na czarnym
plate-white-on-blue = Białe na niebieskim
plate-white-on-red = Białe na czerwonym
plate-white-on-green = Białe na zielonym
plate-gold-on-black = Złote na czarnym
plate-custom = Inne kolory
truck-engine = Silnik
truck-transmission = Skrzynia biegów
truck-part-pick = Wybierz…
truck-part-loading = Czytanie danych gry…
paint = Lakier
paint-job = Malowanie
paint-job-pick = Wybierz malowanie
paint-base = Kolor bazowy
paint-colour-1 = Pierwszy kolor
paint-colour-2 = Drugi kolor
paint-colour-3 = Trzeci kolor
paint-locked = Kolory tego malowania ustala gra.
saved-colours = Zapisane kolory
saved-colours-count = { $count ->
    [0] Brak
    [one] 1 kolor
    [few] { $count } kolory
    [many] { $count } kolorów
   *[other] { $count } koloru
}
saved-colours-open = Zapisane kolory…
saved-colours-description = Kolory zapisane na ekranie lakierowania w grze, dla całej firmy: 8, każdy do czterech kolorów (gra zapisuje kolory malowania razem; pojedynczy kolor używa pierwszego). Pojawiają się jako szybki wybór w lakierze każdego pojazdu.
saved-colours-slot = Zapisany kolor { $slot }
saved-colours-colour = Zapisany kolor { $slot }, kolor { $colour }
saved-colours-copy = Kopiuj jako tekst
saved-colours-copy-tooltip = Kopiuje kolory jako tekst do udostępnienia: #RRGGBB dla każdego zapisanego koloru, kolory zestawu połączone znakiem /.
saved-colours-copied = Skopiowano kolory
saved-colours-paste = Wklej tekst
saved-colours-paste-tooltip = Pobiera ze schowka kolory udostępnione jako tekst (#RRGGBB, zestaw połączony znakiem /, - dla pustego), od pierwszego zapisanego koloru.
saved-colours-not-colours = Schowek nie zawiera kolorów jako tekstu: #RRGGBB dla każdego zapisanego koloru, oddzielone spacjami.
truck-badge = Emblemat boczny
truck-remove-badge = Usuń go
truck-remove-badge-tooltip = Usuwa emblemat silnika z boków kabiny (emblemat z przodu zostaje).
trailer-load = Ładunek
trailer-cargo = Ładunek
trailer-empty-weight = Masa własna
trailer-countries = { $game ->
    [ats] Stany
   *[other] Kraje
}
trailer-everywhere = Wszędzie
trailer-allow-everywhere = Zezwól wszędzie
edit-restricted = { $game ->
    [ats] Tylko niektóre stany
   *[other] Tylko niektóre kraje
}
edit-unrestricted = Wszędzie
edit-fuel = { $percent } % paliwa
edit-badge = Z emblematem bocznym
edit-no-badge = Bez emblematu bocznego
edit-plate = { $plate } · { $colours }
edit-wear = { $part }: { $percent } % zużycia

# Profile tools
profile-tools-open = Narzędzia profilu…
profile-tools-title = Profil „{ $name }”
profile-tools-backup = Utwórz kopię zapasową profilu
profile-tools-backup-description = Kopia całego profilu (każdy zapis, jego zdjęcia i ustawienia), przechowywana z innymi kopiami zapasowymi.
profile-tools-backup-now = Utwórz kopię teraz
profile-tools-backed-up = Utworzono kopię zapasową ({ $size } MB).
profile-tools-copy = Utwórz kopię
profile-tools-copy-description = Drugi profil ze wszystkim, co jest w tym, pod nową nazwą. Powstaje na tym komputerze (Steam Cloud pokazuje tylko profile utworzone przez grę) i bez twojego logowania do World of Trucks: połącz go w grze, jeśli chcesz.
profile-tools-copy-suggestion = { $name } kopia
profile-tools-copy-now = Utwórz kopię
profile-tools-copied = Utworzono profil „{ $name }”. Jest na liście profili w grze.
profile-tools-copy-name-empty = Nadaj kopii nazwę.
profile-tools-copy-name-taken = Profil o tej nazwie już istnieje.
profile-tools-copy-no-documents = Nie znaleziono folderu gry w Dokumentach. Uruchom grę raz, a potem spróbuj ponownie.
profile-tools-settings = Skopiuj ustawienia
profile-tools-settings-description = Grafika, sterowanie i opcje rozgrywki tego profilu, w miejsce ustawień innego profilu. Najpierw powstaje kopia zapasowa ustawień tamtego profilu.
profile-tools-settings-pick = Wybierz profil…
profile-tools-settings-now = Skopiuj ustawienia
profile-tools-settings-copied = Skopiowano do profilu „{ $name }”.
profile-tools-settings-none = Nie ma innego profilu z ustawieniami, do którego można by je skopiować.
profile-tools-running = Najpierw zamknij grę: zapisuje ona profile przy wyjściu.
profile-tools-failed = Nie udało się: { $error }

# The player's truck and position
player-truck = Twoja ciężarówka
player-truck-yours = Tą jeździsz ty
player-drive = Jedź nią sam
player-drive-tooltip = Czyni ją twoją ciężarówką: kto nią jeździł, dostaje twoją, a jej garaż staje się twoją siedzibą firmy. Będzie w twojej pozycji, gdy wczytasz zapis.
player-why-on-job = Najpierw ukończ lub anuluj bieżące zlecenie.
player-why-not-in-garage = Twoją może być tylko ciężarówka w garażu.
player-position = Twoja pozycja
player-position-somewhere = Gdzieś na mapie
player-position-from = { $place }, jak w zapisie „{ $save }”
player-position-now = Pozycja twojej ciężarówki: { $place }.
player-position-away = Twoja ciężarówka jest w trasie, poza miastami.
player-position-no-truck = Nie masz jeszcze własnej ciężarówki.
player-position-unknown = Zapis nie mówi, gdzie jest twoja ciężarówka.
player-quick-travel = Szybka podróż po wczytaniu
player-quick-travel-tooltip = Po wczytaniu zapisu gra pokaże „Game change detected” i pozwoli bezpłatnie wybrać dowolne odwiedzone miasto dla ciężarówki (i autobusu). Nie działa w trakcie zlecenia, więc nic nie zostanie anulowane.
player-quick-travel-asked = Po wczytaniu zapisu gra pokaże „Game change detected” i pozwoli bezpłatnie wybrać miasto dla ciężarówki (i autobusu). Cofnij odwraca zmianę.
player-quick-travel-after = Szybka podróż po wczytaniu
player-quick-travel-unavailable = Ten zapis nie może poprosić gry o szybką podróż (starsza wersja gry).
player-why-no-truck = Szybka podróż przenosi twoje własne pojazdy, więc wymaga własnej ciężarówki.
player-move = Miejsce z innego zapisu…
player-move-pick = Wybierz zapis…
player-move-description = Twoja ciężarówka przeniesie się tam, gdzie stoi w innym zapisie tego profilu.
player-move-reading = Czytanie zapisu „{ $save }”…
player-move-no-position = „{ $save }” nie mówi, gdzie jest twoja ciężarówka.
player-move-hitched = Najpierw odczep naczepę w grze: ona też potrzebowałaby miejsca.

# Your day
day-section = Twój dzień
day-description = Jak długo jedziesz, twoja kartoteka policyjna i zegar. Zmiany tutaj są zapisywane przy zapisaniu, jak pozostałe.
day-since-break = Od ostatniej przerwy
day-rest = Odpocznij
day-rest-tooltip = Jakbyś się wyspał: bez zmęczenia i bez zaległej przerwy.
day-rest-what = Zmęczenie
day-rested = Wypoczęty
day-record = Kartoteka policyjna
day-record-summary = { $count ->
    [0] Czysta
    [one] Jedno wykroczenie, mandaty: { $amount }
    [few] { $count } wykroczenia, mandaty: { $amount }
   *[many] { $count } wykroczeń, mandaty: { $amount }
}
day-record-clear-button = Wyczyść kartotekę
day-record-tooltip = Usuwa każde wykroczenie z kartoteki. Już zapłacone mandaty pozostają zapłacone.
day-record-clear = Czysta
day-cargo-damage = Uszkodzenia ładunku
day-repair-cargo = Napraw ładunek
day-time = Czas
day-time-on-job = Przestawienie zegara spóźniłoby twoje zlecenie.
day-skip-morning = Rano (08:00)
day-skip-noon = Południe (12:00)
day-skip-evening = Wieczór (18:00)
day-skip-night = Noc (22:00)
day-and-time = { $day } { $time }
day-monday = Poniedziałek
day-tuesday = Wtorek
day-wednesday = Środa
day-thursday = Czwartek
day-friday = Piątek
day-saturday = Sobota
day-sunday = Niedziela
duration-minutes = { $minutes } min
duration-hours = { $hours } h { $minutes } min
player-trailer = Twoja naczepa
player-trailer-none = Brak
player-trailer-yours = Ciągniesz tę
player-pull = Ciągnij ją sam
player-pull-tooltip = Czyni ją naczepą, którą ciągniesz. Nie jest podczepiona: podczep ją w grze.

# Picking rows
picks-count = { $count ->
    [one] Wybrano jeden
    [few] Wybrano { $count }
   *[many] Wybrano { $count }
}
picks-max = Maksymalizuj ich zdolności
picks-large = Rozbuduj je do dużych
picks-move = Przenieś do garażu…
picks-refuel = Zatankuj zbiorniki
picks-remove = Usuń
picks-moved = Przeniesiono { $moved } z { $count }: pozostałe się tam nie zmieściły.
picks-removed = Usunięto { $removed } z { $count }: reszty nie można usunąć (otwórz jeden, aby zobaczyć dlaczego).
picks-moved-all = { $count ->
    [one] Przeniesiono.
    [few] Przeniesiono wszystkie { $count }.
   *[many] Przeniesiono wszystkie { $count }.
}
picks-removed-all = { $count ->
    [one] Usunięto.
    [few] Usunięto wszystkie { $count }.
   *[many] Usunięto wszystkie { $count }.
}
picks-clear = Wyczyść
picks-done-repair = { $count ->
    [one] Naprawiono 1 pojazd
    [few] Naprawiono { $count } pojazdy
    [many] Naprawiono { $count } pojazdów
   *[other] Naprawiono { $count } pojazdu
}
picks-done-refuel = { $count ->
    [one] Zatankowano 1 zbiornik
    [few] Zatankowano { $count } zbiorniki
    [many] Zatankowano { $count } zbiorników
   *[other] Zatankowano { $count } zbiornika
}
picks-done-max = { $count ->
    [one] Zmaksymalizowano umiejętności 1 kierowcy
    [few] Zmaksymalizowano umiejętności { $count } kierowców
    [many] Zmaksymalizowano umiejętności { $count } kierowców
   *[other] Zmaksymalizowano umiejętności { $count } kierowcy
}
picks-done-large = { $count ->
    [one] Powiększono 1 garaż
    [few] Powiększono { $count } garaże
    [many] Powiększono { $count } garaży
   *[other] Powiększono { $count } garażu
}
picks-done-body = Cofnij odwraca zmianę. Nic nie zostanie zapisane, dopóki nie zapiszesz.
picks-all = Wybierz każdy widoczny wiersz
picks-row = Wybierz: { $name }

## Statistics
page-statistics = Statystyki
page-coach-career = Kariera kierowcy autokaru
stats-deliveries = Dostawy
stats-income = Zarobione na dostawach
stats-xp = PD z dostaw
stats-driven = Przejechane łącznie
stats-days = Dni w trasie
stats-average = Średnio na dostawę
stats-favourite = Najczęściej przewożony
stats-favourite-value = { $cargo } ({ $count })
stats-cancelled = Anulowane zlecenia
stats-note = Z dziennika dostaw gry. Tylko do odczytu: gra sama prowadzi te liczby.
stats-search = Szukaj po ładunku lub mieście
stats-filter-damaged = Uszkodzony ładunek
stats-table = Dostawy
stats-none = Brak dostaw
stats-none-body = Każdy dostarczony ładunek pojawia się tutaj, od najnowszego.
stats-when = { $days ->
    [0] Dzisiaj
    [1] Wczoraj
    [few] { $days } dni temu
   *[many] { $days } dni temu
}
column-when = Kiedy
column-income = Dochód
column-xp = PD
column-damage = Uszkodzenia
column-passengers = Pasażerowie
column-satisfaction = Zadowolenie
coach-earned = Zarobione na kursach
coach-log-note = Z dziennika kursów autokarowych gry. Tylko do odczytu: gra sama prowadzi te liczby.
coach-log-search = Szukaj po mieście
coach-log-table = Kursy autokarowe
coach-log-none = Brak kursów autokarowych
coach-log-none-body = Każdy przejechany kurs autokarowy pojawi się tutaj, najnowszy na górze.

## What the save needs
needs = DLC i modyfikacje
needs-description = { $mods ->
    [0] Ten zapis powstał z użyciem DLC: { $dlcs }. Gra wczyta go tylko z zainstalowanymi tymi, których potrzebuje.
   *[other] Ten zapis powstał z użyciem DLC: { $dlcs } i modyfikacji: { $mods }. Gra wczyta go tylko z zainstalowanymi tymi, których potrzebuje.
}
needs-unknown = Nie można sprawdzić bez gry. Wybierz jej folder w Ustawieniach.
needs-all-here = Wszystko, z czym powstał, jest zainstalowane.
needs-missing = { $count ->
    [one] Brakuje jednego elementu, którego potrzebuje zapis. Zainstaluj go, inaczej gra nie wczyta tego zapisu:
    [few] Brakuje { $count } elementów, których potrzebuje zapis. Zainstaluj je, inaczej gra nie wczyta tego zapisu:
   *[many] Brakuje { $count } elementów, których potrzebuje zapis. Zainstaluj je, inaczej gra nie wczyta tego zapisu:
}
needs-missing-extras = { $count ->
    [one] Brakuje jednego opcjonalnego elementu (zapis wczyta się bez niego):
    [few] Brakuje { $count } opcjonalnych elementów (zapis wczyta się bez nich):
   *[many] Brakuje { $count } opcjonalnych elementów (zapis wczyta się bez nich):
}
needs-mods = Modyfikacje
needs-installed = Zainstalowane
needs-not-installed = Niezainstalowane

## Galeria: punkty widokowe i trofea fotograficzne (słowa gry)

gallery-viewpoint = Punkt widokowy
gallery-viewpoints = Punkty widokowe
gallery-photo-trophy = Trofeum fotograficzne
gallery-photo-trophies = Trofea fotograficzne
gallery-visited = Odwiedzony
gallery-found = Odkryty, nieobejrzany
gallery-not-found = Jeszcze nieodkryty
gallery-captured = Zdobyte
gallery-not-captured = Jeszcze niezdobyte
gallery-visited-of = Odwiedzone: { $visited } z { $count }
gallery-viewpoints-summary = Odwiedzone punkty widokowe: { $visited } z { $count }
gallery-photo-trophies-summary = Zdobyte trofea fotograficzne: { $captured } z { $count }
gallery-where-kept = Punkty widokowe są zapisane w tym zapisie gry, więc każdy zapis ma własne. Trofea fotograficzne są zapisane w profilu i wspólne dla wszystkich jego zapisów.
gallery-column-name = Nazwa
gallery-column-kind = Rodzaj
gallery-column-country = { $game ->
    [ats] Stan
   *[other] Kraj
}
gallery-column-progress = Postęp
gallery-filter-viewpoints = Punkty widokowe
gallery-filter-photo-trophies = Trofea fotograficzne
gallery-filter-to-do = Jeszcze nie zrobione
gallery-search = { $game ->
    [ats] Szukaj po nazwie lub stanie
   *[other] Szukaj po nazwie lub kraju
}
gallery-table = Punkty widokowe i trofea fotograficzne
gallery-visit-all = Odwiedź każdy punkt widokowy
gallery-visit-all-tooltip = Oznacz każdy punkt widokowy jako odwiedzony, jakbyś obejrzał każdy z nich. Galeria może wtedy odtworzyć wszystkie.
gallery-visit = Oznacz jako odwiedzony
gallery-visit-tooltip = Jakbyś go obejrzał: liczy się jako odwiedzony, a Galeria może go odtworzyć.
gallery-forget = Oznacz jako nieodkryty
gallery-forget-tooltip = Jakbyś nigdy tam nie był: gra odkryje go znowu, gdy będziesz przejeżdżać obok.
gallery-needs-game = Galeria potrzebuje gry zainstalowanej na tym komputerze: jej dane zawierają listę punktów widokowych i trofeów fotograficznych.
gallery-photo-trophy-note = Trofea fotograficzne to twoje własne zdjęcia, zapisane w profilu, więc edytor tylko je pokazuje: w grze zdobędziesz je, robiąc im zdjęcie.
gallery-map-not-read = Mapa gry jest jeszcze wczytywana albo nie została znaleziona.
gallery-map-hover = { $name }: { $progress }
