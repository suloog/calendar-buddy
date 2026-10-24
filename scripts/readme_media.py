"""The README's pictures: the app on made-up data, copied from the previews, and a short tour.

Run the previews first, then this script from the repository's root:

    cargo test -p calendar-buddy preview -- --ignored
    python3 scripts/readme_media.py

It writes docs/screenshots/: one PNG a screen and tour.gif, which steps through the views.
"""

from pathlib import Path

from PIL import Image

PREVIEWS = Path("target/preview")
OUT = Path("docs/screenshots")

# The screens shown in the README.
SCREENS = [
    "week-light",
    "week-dark",
    "month-light",
    "agenda-light",
    "details-rich-light",
    "editor-light",
    "new-event-light",
    "settings-light",
]

# The tour, a frame a screen.
TOUR = [
    "week-light",
    "three-days-light",
    "day-light",
    "month-light",
    "agenda-light",
    "details-rich-light",
    "editor-light",
    "week-dark",
]
TOUR_WIDTH = 960
FRAME_MS = 1800


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for name in SCREENS:
        Image.open(PREVIEWS / f"{name}.png").save(OUT / f"{name}.png", optimize=True)

    frames = []
    for name in TOUR:
        frame = Image.open(PREVIEWS / f"{name}.png").convert("RGB")
        height = round(frame.height * TOUR_WIDTH / frame.width)
        frame = frame.resize((TOUR_WIDTH, height), Image.LANCZOS)
        frames.append(frame.quantize(colors=256, method=Image.Quantize.FASTOCTREE))
    frames[0].save(
        OUT / "tour.gif",
        save_all=True,
        append_images=frames[1:],
        duration=FRAME_MS,
        loop=0,
        # Whole frames: kept as differences, their palettes leave traces of the frame before.
        optimize=False,
        disposal=1,
    )


if __name__ == "__main__":
    main()
