"""Draws the app's icon, crates/app/assets/icon/calendar-buddy.png: a blue tile with a white
calendar page holding three events, each a light tint of its colour with a strip of the colour
along its left edge, as the views draw them. Run: python3 scripts/icon.py (needs Pillow)."""

from pathlib import Path

from PIL import Image, ImageDraw

SIZE = 256
SCALE = 4  # drawn larger, then scaled down, for smooth edges
OUT = Path(__file__).resolve().parent.parent / "crates/app/assets/icon/calendar-buddy.png"


def mix(color, strength, under=(255, 255, 255)):
    return tuple(round(c * strength + u * (1 - strength)) for c, u in zip(color, under))


def main():
    s = SIZE * SCALE
    u = s / 256  # one unit of the 256-unit grid
    image = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)

    # The tile: the accent, lighter towards the top.
    tile = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    top, bottom = (0x4C, 0x8D, 0xF6), (0x0B, 0x57, 0xD0)
    gradient = ImageDraw.Draw(tile)
    for y in range(s):
        t = y / (s - 1)
        gradient.line([(0, y), (s, y)], fill=tuple(round(a + (b - a) * t) for a, b in zip(top, bottom)))
    mask = Image.new("L", (s, s), 0)
    ImageDraw.Draw(mask).rounded_rectangle([8 * u, 8 * u, 248 * u, 248 * u], radius=56 * u, fill=255)
    image.paste(tile, (0, 0), mask)

    # The page, with a soft shadow under it.
    shadow = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    ImageDraw.Draw(shadow).rounded_rectangle(
        [52 * u, 62 * u, 204 * u, 214 * u], radius=22 * u, fill=(0, 0, 40, 70)
    )
    image.alpha_composite(shadow)
    draw.rounded_rectangle([48 * u, 56 * u, 208 * u, 208 * u], radius=22 * u, fill=(255, 255, 255))
    # Its header band and the two rings it hangs from.
    draw.rounded_rectangle([48 * u, 56 * u, 208 * u, 92 * u], radius=22 * u, fill=(0xE8, 0xF0, 0xFE))
    draw.rectangle([48 * u, 76 * u, 208 * u, 92 * u], fill=(0xE8, 0xF0, 0xFE))
    for x in (92, 164):
        draw.rounded_rectangle([(x - 7) * u, 40 * u, (x + 7) * u, 74 * u], radius=7 * u, fill=(0x0B, 0x57, 0xD0))

    # Three events.
    events = [
        ((0x03, 0x9B, 0xE5), 106, 132, 184),
        ((0x33, 0xB6, 0x79), 142, 168, 160),
        ((0xF4, 0x51, 0x1E), 178, 196, 176),
    ]
    for color, y0, y1, x1 in events:
        draw.rounded_rectangle([64 * u, y0 * u, x1 * u, y1 * u], radius=6 * u, fill=mix(color, 0.35))
        draw.rounded_rectangle([64 * u, y0 * u, 72 * u, y1 * u], radius=4 * u, fill=color)

    OUT.parent.mkdir(parents=True, exist_ok=True)
    image.resize((SIZE, SIZE), Image.LANCZOS).save(OUT)
    print(OUT)


if __name__ == "__main__":
    main()
