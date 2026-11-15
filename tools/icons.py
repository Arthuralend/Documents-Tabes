#!/usr/bin/env python3
"""Write FondOffice's app icons (needs Pillow).

- The two apps' icons are the owner's pictures (FondSoftware): Документы, a page of text with a
  terracotta D tile, and Таблицы, the same page with a green T tile. Their sources are
  assets/app-icon/source/documents.webp and tables.webp; `cut` takes the rounded tile out of the
  picture (its faint shadow dropped, its body made opaque) and squares it.
- The suite icon is the owner's picture assets/app-icon/source/office-pro-max.png: the installer's
  and the disk image's icon.

Outputs keep the old file names (wordcraft-*.png, wordcraft.ico/.icns, hicolor/…), so the build
scripts and packaging find the documents app's icon unchanged; Таблицы's are sheets-* here and
gridcraft-* in tables/assets/app-icon (the Таблицы program's own build).

Usage: python3 tools/icons.py
"""

import os

from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DIR = os.path.join(ROOT, "assets", "app-icon")
SRC = os.path.join(DIR, "source")
TABLES_DIR = os.path.join(ROOT, "tables", "assets", "app-icon")
RES = os.path.join(ROOT, "android", "app", "src", "main", "res")
APP_ID = "ai.storyteller.wordcraft"
TABLES_ID = "ai.storyteller.gridcraft"
SIZES = (16, 24, 32, 48, 64, 128, 256, 512)


def cut(path):
    """The rounded tile of the picture at `path`, opaque, on a transparent square of 1024 px."""
    im = Image.open(path).convert("RGBA")
    a = im.getchannel("A")
    box = a.point(lambda v: 255 if v >= 128 else 0).getbbox()
    tile = im.crop(box)
    # The body is drawn at 253/255 and its shadow fades out around it: make the body opaque and
    # keep only the tile's anti-aliased edge.
    lo, hi = 60, 200
    alpha = tile.getchannel("A").point(lambda v: 0 if v <= lo else 255 if v >= hi else round((v - lo) * 255 / (hi - lo)))
    tile.putalpha(alpha)
    return tile.resize((1024, 1024), Image.LANCZOS)


def framed(src, size, body=1.0):
    """`src` scaled to `body` of a transparent `size` px square, centred."""
    inner = max(1, round(size * body))
    out = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    out.alpha_composite(src.resize((inner, inner), Image.LANCZOS), ((size - inner) // 2, (size - inner) // 2))
    return out


def write_set(folder, prefix, src):
    # macOS icons sit on Apple's grid: an 824/1024 body with a transparent margin.
    mac = 824 / 1024
    framed(src, 1024).save(os.path.join(folder, f"{prefix}-1024.png"))
    framed(src, 512, mac).save(os.path.join(folder, f"{prefix}-macos-512.png"))
    sizes = [16, 20, 24, 32, 40, 48, 64, 128, 256]
    framed(src, 256).save(os.path.join(folder, f"{prefix}.ico"), sizes=[(s, s) for s in sizes])
    framed(src, 1024, mac).save(os.path.join(folder, f"{prefix}.icns"))


def hicolor(folder, app_id, src):
    for s in SIZES:
        d = os.path.join(folder, "hicolor", f"{s}x{s}", "apps")
        os.makedirs(d, exist_ok=True)
        framed(src, s).save(os.path.join(d, f"{app_id}.png"))


def android(name, src):
    """An adaptive icon (the whole tile as the foreground, inside the 66 dp safe zone, on the tile's
    own cream) and a legacy icon for older launchers."""
    cream = src.getpixel((512, 40))[:3] + (255,)
    for dpi, scale in [("mdpi", 1), ("hdpi", 1.5), ("xhdpi", 2), ("xxhdpi", 3), ("xxxhdpi", 4)]:
        folder = os.path.join(RES, f"mipmap-{dpi}")
        os.makedirs(folder, exist_ok=True)
        px = round(108 * scale)
        Image.new("RGBA", (px, px), cream).save(os.path.join(folder, f"{name}_background.png"))
        framed(src, px, 0.66).save(os.path.join(folder, f"{name}_foreground.png"))
        framed(src, round(48 * scale)).save(os.path.join(folder, f"{name}.png"))


def main():
    documents = cut(os.path.join(SRC, "documents.webp"))
    tables = cut(os.path.join(SRC, "tables.webp"))
    write_set(DIR, "wordcraft", documents)
    write_set(DIR, "sheets", tables)
    write_set(TABLES_DIR, "gridcraft", tables)
    hicolor(DIR, APP_ID, documents)
    hicolor(TABLES_DIR, TABLES_ID, tables)
    android("ic_launcher", documents)
    print("icons written to", DIR, "and", TABLES_DIR)


if __name__ == "__main__":
    main()
