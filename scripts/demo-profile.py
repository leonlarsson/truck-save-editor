#!/usr/bin/env python3
"""Makes a made-up demo profile from a copy of a real save, for screenshots that show nothing of
the player's own: `scripts/screenshots.sh` runs it. Run it as `python3 -I`.

    python3 -I scripts/demo-profile.py <decoded game.txt> <decoded info.txt> <out profiles folder> [ets2|ats]

The two inputs are the slot's `game.sii` and `info.sii` decoded to text (`cargo run -p sii
--example decode`), from a copy under `fixtures/`, never a live save. The output is a profiles
folder holding one profile, "Leon" (folder `4C656F6E`), with one save in slot 1: the same save
with every value that could say whose it is replaced by an invented one. Money, both careers'
experience, the headquarters city, where the truck stands, every plate, the save's name, the company's and profile's
names, and the photo trophies captured are made up. The rest (trucks, garages, cities, jobs) is
game data. For an ATS save (`ats`), the headquarters are in Sacramento and no photo trophy is
captured (the list below is ETS2's). Nothing it writes is ever committed: only the screenshots, after a look.
"""

import random
import re
import sys
from pathlib import Path

# Invented values, the same every run, so the screenshots only change when the editor does.
MONEY = 2_480_350
TRUCK_XP = 486_120
COACH_XP = 98_400
HQ_CITY = "hamburg"
HQ_CITY_ATS = "sacramento"
SAVE_NAME = "My career"
PROFILE_NAME = "Leon"
COMPANY_NAME = "Northwind Haulage"
# Photo trophies "captured" in the demo: ids from the game's own list (game data, not the player's).
LANDMARKS = ["prague", "nurnberg", "munchen", "kiel", "rostock", "oslo", "dresdenmill"]
PLATE_LETTERS = "ABCDEFGHJKLMNPRSTUVWXZ"


def made_up_plate(rng: random.Random) -> str:
    letters = "".join(rng.choice(PLATE_LETTERS) for _ in range(3))
    return f"{letters} {rng.randint(100, 999)}"


def demo_game(text: str, hq_city: str) -> str:
    rng = random.Random(7)
    text = re.sub(r"(\n money_account: )-?\d+", rf"\g<1>{MONEY}", text, count=1)
    text = re.sub(r"(\n experience_points: )\d+", rf"\g<1>{TRUCK_XP}", text, count=1)
    text = re.sub(r"(\n bus_experience_points: )\d+", rf"\g<1>{COACH_XP}", text, count=1)
    text = re.sub(r"(\n hq_city: )\w+", rf"\g<1>{hq_city}", text, count=1)
    # Where the player's truck stands (World map says "Your truck is in …").
    text = re.sub(r"(\n stored_vehicle_city: )city\.\w+", rf"\g<1>city.{hq_city}", text)
    # Plates keep their country (`"AB 123|france"`); plates without one get a plain made-up one.
    text = re.sub(
        r'(\n license_plate: )"[^"|]*\|([^"]*)"',
        lambda m: f'{m.group(1)}"{made_up_plate(rng)}|{m.group(2)}"',
        text,
    )
    text = re.sub(
        r'(\n license_plate: )"[^"|]*"',
        lambda m: f'{m.group(1)}"{made_up_plate(rng)}"',
        text,
    )
    return text


def demo_info(text: str) -> str:
    text = re.sub(r'(\n name: )"?[^"\r\n]*"?', rf'\g<1>"{SAVE_NAME}"', text, count=1)
    text = re.sub(r"(info_players_experience: )\d+", rf"\g<1>{TRUCK_XP}", text)
    text = re.sub(r"(info_money_account: )-?\d+", rf"\g<1>{MONEY}", text)
    return text


def profile_text() -> str:
    return (
        "SiiNunit\r\n{\r\nuser_profile : _nameless.9 {\r\n"
        f' company_name: "{COMPANY_NAME}"\r\n'
        f' profile_name: "{PROFILE_NAME}"\r\n'
        f" cached_experience: {TRUCK_XP}\r\n"
        "}\r\n\r\n}\r\n"
    )


def album_text(landmarks: list[str]) -> str:
    units = []
    links = []
    for index, landmark in enumerate(landmarks, start=2):
        links.append(f" landmarks[{index - 2}]: _nameless.{index}\r\n")
        units.append(
            f"album_landmarks_data : _nameless.{index} {{\r\n"
            f" landmark_id: {landmark}\r\n"
            f' screenshot_file: "demo{index}.png"\r\n'
            "}\r\n\r\n"
        )
    return (
        "SiiNunit\r\n{\r\nalbum_landmarks_info : _nameless.1 {\r\n"
        f" landmarks: {len(landmarks)}\r\n" + "".join(links) + "}\r\n\r\n"
        + "".join(units)
        + "}\r\n"
    )


def main() -> None:
    if len(sys.argv) not in (4, 5):
        sys.exit(__doc__)
    game_txt, info_txt, out = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3])
    is_ats = len(sys.argv) == 5 and sys.argv[4] == "ats"
    hq_city = HQ_CITY_ATS if is_ats else HQ_CITY
    landmarks = [] if is_ats else LANDMARKS
    profile = out / PROFILE_NAME.encode().hex().upper()
    slot = profile / "save" / "1"
    slot.mkdir(parents=True, exist_ok=True)
    (profile / "album").mkdir(exist_ok=True)
    (slot / "game.sii").write_text(
        demo_game(game_txt.read_text(encoding="utf-8"), hq_city), encoding="utf-8"
    )
    (slot / "info.sii").write_text(demo_info(info_txt.read_text(encoding="utf-8")), encoding="utf-8")
    (profile / "profile.sii").write_text(profile_text(), encoding="utf-8")
    (profile / "album" / "landmarks.sii").write_text(album_text(landmarks), encoding="utf-8")
    print(slot)


if __name__ == "__main__":
    main()
