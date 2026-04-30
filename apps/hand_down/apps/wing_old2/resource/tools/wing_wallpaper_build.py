#!/usr/bin/env python3
"""Build a scaled RGB565 wallpaper from an 8-bit PNG source.

This intentionally avoids Pillow/ImageMagick so the Wing resource build keeps
working in small NuttX-oriented host environments.
"""

from __future__ import annotations

import math
import struct
import sys
import zlib
from pathlib import Path


PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


def main() -> int:
    if len(sys.argv) != 5:
        print(
            "usage: wing_wallpaper_build.py <source.png> <out.rgb565> <width> <height>",
            file=sys.stderr,
        )
        return 2

    source = Path(sys.argv[1])
    output = Path(sys.argv[2])
    width = parse_dimension(sys.argv[3], "width")
    height = parse_dimension(sys.argv[4], "height")

    image = read_png(source)
    rgb565 = scale_cover_rgb565(image, width, height)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(rgb565)
    return 0


def parse_dimension(value: str, name: str) -> int:
    try:
        result = int(value, 10)
    except ValueError as exc:
        raise SystemExit(f"{name} must be an integer: {value}") from exc
    if not 0 < result <= 65535:
        raise SystemExit(f"{name} must be in 1..65535: {value}")
    return result


class PngImage:
    def __init__(self, width: int, height: int, color_type: int, pixels: bytes) -> None:
        self.width = width
        self.height = height
        self.color_type = color_type
        self.pixels = pixels

    @property
    def channels(self) -> int:
        return png_channels(self.color_type)

    def rgba_at(self, x: int, y: int) -> tuple[int, int, int, int]:
        index = (y * self.width + x) * self.channels
        if self.color_type == 0:
            gray = self.pixels[index]
            return gray, gray, gray, 255
        if self.color_type == 2:
            return self.pixels[index], self.pixels[index + 1], self.pixels[index + 2], 255
        if self.color_type == 4:
            gray = self.pixels[index]
            return gray, gray, gray, self.pixels[index + 1]
        if self.color_type == 6:
            return (
                self.pixels[index],
                self.pixels[index + 1],
                self.pixels[index + 2],
                self.pixels[index + 3],
            )
        raise ValueError(f"unsupported PNG color type: {self.color_type}")


def read_png(path: Path) -> PngImage:
    data = path.read_bytes()
    if not data.startswith(PNG_SIGNATURE):
        raise SystemExit(f"{path}: not a PNG file")

    offset = len(PNG_SIGNATURE)
    width = 0
    height = 0
    bit_depth = 0
    color_type = 0
    compression = 0
    filter_method = 0
    interlace = 0
    idat = bytearray()

    while offset + 8 <= len(data):
        length = struct.unpack(">I", data[offset : offset + 4])[0]
        chunk_type = data[offset + 4 : offset + 8]
        chunk_start = offset + 8
        chunk_end = chunk_start + length
        if chunk_end + 4 > len(data):
            raise SystemExit(f"{path}: truncated PNG chunk")
        chunk = data[chunk_start:chunk_end]
        offset = chunk_end + 4

        if chunk_type == b"IHDR":
            (
                width,
                height,
                bit_depth,
                color_type,
                compression,
                filter_method,
                interlace,
            ) = struct.unpack(">IIBBBBB", chunk)
        elif chunk_type == b"IDAT":
            idat.extend(chunk)
        elif chunk_type == b"IEND":
            break

    if width <= 0 or height <= 0:
        raise SystemExit(f"{path}: missing IHDR")
    if bit_depth != 8:
        raise SystemExit(f"{path}: only 8-bit PNG is supported")
    if color_type not in (0, 2, 4, 6):
        raise SystemExit(f"{path}: unsupported PNG color type {color_type}")
    if compression != 0 or filter_method != 0 or interlace != 0:
        raise SystemExit(f"{path}: unsupported PNG compression/filter/interlace")

    raw = zlib.decompress(bytes(idat))
    channels = png_channels(color_type)
    stride = width * channels
    expected = (stride + 1) * height
    if len(raw) != expected:
        raise SystemExit(f"{path}: decoded byte length mismatch")

    pixels = unfilter_png(raw, width, height, channels)
    return PngImage(width, height, color_type, pixels)


def png_channels(color_type: int) -> int:
    if color_type == 0:
        return 1
    if color_type == 2:
        return 3
    if color_type == 4:
        return 2
    if color_type == 6:
        return 4
    raise ValueError(f"unsupported PNG color type: {color_type}")


def unfilter_png(raw: bytes, width: int, height: int, bpp: int) -> bytes:
    stride = width * bpp
    out = bytearray(stride * height)
    source = 0
    prev = bytearray(stride)

    for row in range(height):
        filter_type = raw[source]
        source += 1
        scan = bytearray(raw[source : source + stride])
        source += stride

        for i, value in enumerate(scan):
            left = scan[i - bpp] if i >= bpp else 0
            up = prev[i]
            up_left = prev[i - bpp] if i >= bpp else 0
            if filter_type == 0:
                recon = value
            elif filter_type == 1:
                recon = value + left
            elif filter_type == 2:
                recon = value + up
            elif filter_type == 3:
                recon = value + ((left + up) >> 1)
            elif filter_type == 4:
                recon = value + paeth(left, up, up_left)
            else:
                raise SystemExit(f"unsupported PNG row filter {filter_type}")
            scan[i] = recon & 0xFF

        start = row * stride
        out[start : start + stride] = scan
        prev = scan

    return bytes(out)


def paeth(left: int, up: int, up_left: int) -> int:
    estimate = left + up - up_left
    pa = abs(estimate - left)
    pb = abs(estimate - up)
    pc = abs(estimate - up_left)
    if pa <= pb and pa <= pc:
        return left
    if pb <= pc:
        return up
    return up_left


def scale_cover_rgb565(image: PngImage, dst_w: int, dst_h: int) -> bytes:
    scale = max(dst_w / image.width, dst_h / image.height)
    crop_w = dst_w / scale
    crop_h = dst_h / scale
    src_x0 = (image.width - crop_w) / 2.0
    src_y0 = (image.height - crop_h) / 2.0
    out = bytearray(dst_w * dst_h * 2)
    offset = 0

    for y in range(dst_h):
        sy = src_y0 + (y + 0.5) / scale - 0.5
        for x in range(dst_w):
            sx = src_x0 + (x + 0.5) / scale - 0.5
            r, g, b = sample_rgb(image, sx, sy)
            packed = ((r >> 3) << 11) | ((g >> 2) << 5) | (b >> 3)
            out[offset] = packed & 0xFF
            out[offset + 1] = packed >> 8
            offset += 2

    return bytes(out)


def sample_rgb(image: PngImage, sx: float, sy: float) -> tuple[int, int, int]:
    x0 = clamp(math.floor(sx), 0, image.width - 1)
    y0 = clamp(math.floor(sy), 0, image.height - 1)
    x1 = clamp(x0 + 1, 0, image.width - 1)
    y1 = clamp(y0 + 1, 0, image.height - 1)
    tx = sx - math.floor(sx)
    ty = sy - math.floor(sy)

    c00 = premul_rgb(image.rgba_at(x0, y0))
    c10 = premul_rgb(image.rgba_at(x1, y0))
    c01 = premul_rgb(image.rgba_at(x0, y1))
    c11 = premul_rgb(image.rgba_at(x1, y1))

    return tuple(
        clamp(round(lerp(lerp(c00[i], c10[i], tx), lerp(c01[i], c11[i], tx), ty)), 0, 255)
        for i in range(3)
    )


def premul_rgb(color: tuple[int, int, int, int]) -> tuple[int, int, int]:
    r, g, b, a = color
    if a >= 255:
        return r, g, b
    return (r * a) // 255, (g * a) // 255, (b * a) // 255


def lerp(a: int, b: int, t: float) -> float:
    return a + (b - a) * t


def clamp(value: int, low: int, high: int) -> int:
    return max(low, min(high, value))


if __name__ == "__main__":
    raise SystemExit(main())
