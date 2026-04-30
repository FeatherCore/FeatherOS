#!/usr/bin/env python3
"""Extract FHRE FRAW images from W10M LVGL native-with-alpha C arrays.

Usage:
  python3 extract_fraw.py /path/to/windows-10-mobile-lvgl/src/ui/images raw
"""

from __future__ import annotations

import argparse
import re
import struct
from pathlib import Path


SIGNATURE = b"FHREIMG1"
FORMAT_RGBA8888 = 3


RAW_NAMES = {
    "padlock": "ui_img_padlock_png.c",
    "settings_back": "ui_img_settings_back_png.c",
    "stars_ic": "ui_img_stars_ic_png.c",
    "wifi_icon": "ui_img_wifi_icon_png.c",
    "wp_about": "ui_img_wp_about_png.c",
    "wp_account": "ui_img_wp_account_png.c",
    "wp_apps": "ui_img_wp_apps_png.c",
    "wp_devices": "ui_img_wp_devices_png.c",
    "wp_network": "ui_img_wp_network_png.c",
    "wp_personalization": "ui_img_wp_personalization_png.c",
    "wp_privacy": "ui_img_wp_privacy_png.c",
    "wp_time": "ui_img_wp_time_png.c",
}


def parse_lvgl_c(path: Path) -> tuple[int, int, bytes]:
    text = path.read_text(encoding="utf-8")
    width = int(_match(r"\.w\s*=\s*(\d+)", text, path))
    height = int(_match(r"\.h\s*=\s*(\d+)", text, path))
    values = [int(token, 16) for token in re.findall(r"0x([0-9a-fA-F]{2})", text)]
    if not values:
        raise ValueError(f"{path}: no hex image data found")
    return width, height, bytes(values)


def _match(pattern: str, text: str, path: Path) -> str:
    found = re.search(pattern, text)
    if found is None:
        raise ValueError(f"{path}: missing {pattern}")
    return found.group(1)


def rgba_from_native_with_alpha(width: int, height: int, data: bytes) -> bytes:
    pixels = width * height
    if len(data) == pixels * 4:
        return data
    if len(data) != pixels * 3:
        raise ValueError(f"unsupported data length {len(data)} for {width}x{height}")

    out = bytearray(pixels * 4)
    for index in range(pixels):
        lo = data[index * 3]
        hi = data[index * 3 + 1]
        alpha = data[index * 3 + 2]
        raw = lo | (hi << 8)
        r = ((raw >> 11) & 0x1F) * 255 // 31
        g = ((raw >> 5) & 0x3F) * 255 // 63
        b = (raw & 0x1F) * 255 // 31
        out[index * 4:index * 4 + 4] = bytes((r, g, b, alpha))
    return bytes(out)


def write_fraw(path: Path, width: int, height: int, rgba: bytes) -> None:
    stride = width * 4
    header = SIGNATURE + struct.pack("<HHBBII", width, height, FORMAT_RGBA8888, 0, stride, len(rgba))
    path.write_bytes(header + rgba)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()

    args.output.mkdir(parents=True, exist_ok=True)
    for name, filename in RAW_NAMES.items():
        width, height, data = parse_lvgl_c(args.source / filename)
        rgba = rgba_from_native_with_alpha(width, height, data)
        write_fraw(args.output / f"{name}.fraw", width, height, rgba)


if __name__ == "__main__":
    main()
