# Wing Shell Resources

This directory contains concrete runtime resources for the Wing shell.
It is intentionally not a design-file cache.

## Runtime Contract

- `manifest.txt` is the source of truth for stable Shell `ImageId` values.
- `fs/` is copied into the target filesystem as `/etc/wing/resource/`.
- `fs/images/shell/*.png` stores Shell wallpaper image resources used by the
  runtime filesystem path. Landscape wallpapers are kept at `640x480`; portrait
  wallpapers are kept at `480x640`.
- `fs/fonts/*.ttf` stores the default vector font resources used by Wing at runtime.
- `fs/fonts/shell_workset.txt` stores the Shell glyph warmup working set.
- `fs/icons/shell/*.svg` stores the runtime filesystem copy of the selected
  Shell SVG icon set.
- `icons/bootstrap/*.svg` stores the small selected Bootstrap Icons source set
  used by Wing's runtime SVG parser as the built-in fallback source for Shell.
- `apps/<app>/icons/*.svg` stores compile-time private vector resources for
  independent default apps. For example, `apps/settings/icons/*.svg`,
  `apps/system/icons/*.svg`, and `apps/terminal/icons/*.svg` are embedded into
  their own App UI SVG stores and are not copied as Shell runtime filesystem
  state.
- `generated/shell/*.rgb565` stores compact RGB565 wallpaper fallback payloads
  built from the runtime PNG files. The packed fallback keeps the same aspect
  ratio as the display and defaults to a 320-pixel long edge.
- Shell icons are rendered by the built-in `VectorIcon` draw command. The icon
  SVG paths are parsed by Wing itself at startup and rasterized into a bounded
  A8 cache on demand, similar to the font glyph cache. Icon PNG/RGB565/A8
  bitmap resources are intentionally not kept.
- `generated/wing_manifest.bin` and `generated/wing_payload.bin` are produced
  by the Rust crate build script and copied into NuttX sim ROMFS.

Current Shell IDs:

- `1`: Aurora portrait wallpaper, RGB565 generated from
  `fs/images/shell/wallpaper_aurora_portrait.png`
- `2`: Dusk portrait wallpaper, RGB565 generated from
  `fs/images/shell/wallpaper_dusk_portrait.png`
- `3`: Aurora landscape wallpaper, RGB565 generated from
  `fs/images/shell/wallpaper_aurora_landscape.png`
- `4`: Dusk landscape wallpaper, RGB565 generated from
  `fs/images/shell/wallpaper_dusk_landscape.png`
- Icon semantics such as `wifi`, `bluetooth`, `settings`, `mail`, `play`, and
  `close` are now code-level vector symbols backed by parsed Bootstrap SVG
  resources instead of texture IDs.

## Regenerating Assets

After changing wallpaper PNGs or font files, rebuild Wing:

```sh
cargo build --manifest-path ../rust/Cargo.toml --no-default-features
```

The build script reads `manifest.txt`, decodes the Shell PNG wallpapers, scales
them into compact RGB565 fallback textures, emits the selected Shell Bootstrap
SVG table plus default-app private SVG tables, and writes the generated
resource partition under `generated/`. Only the wallpaper PNGs, Shell SVGs, and
vector fonts under `fs/` are copied to `/etc/wing/resource/`; app-private SVGs
under `apps/<app>/icons/` are embedded at compile time. At runtime the Shell
decodes wallpapers from the filesystem when available, loads Shell SVG icons
from `/etc/wing/resource/icons/shell/*.svg`, and falls back to the embedded
Shell Bootstrap set for missing or invalid icon files. Default App UI surfaces
initialize an app SVG store from their compile-time private icon tables.
Shell glass surfaces, notification cards, quick toggles, and app cards are
drawn by code-level `DrawStyle` descriptors, including vertical rounded
gradients, shadows, borders, scissor clipping, and alpha blending. These UI
effects are not stored as bitmap assets. The runtime DrawList emits clip
changes as stateful commands, so repeated elements inside the same clipped
region do not need per-element bitmap masks.
`fs/fonts/simhei.ttf` is loaded as the default runtime font and rasterized by
the software renderer for Shell `DrawCmd::Text` and App UI RGB565 surfaces. All
text rendering now goes through the font store; characters that cannot be mapped
or parsed are drawn with a scaled placeholder cell instead of the old built-in
bitmap glyph table. The generated RGB565 partition remains the deterministic
fallback path. The generated fallback aspect ratio is taken from
`WING_DISPLAY_WIDTH/WING_DISPLAY_HEIGHT`, then from `nuttx/.config`
`CONFIG_SIM_FBWIDTH/CONFIG_SIM_FBHEIGHT`, with a 480x640 fallback. The fallback
long edge defaults to 320 pixels and can be overridden with
`WING_PACKED_WALLPAPER_LONG_EDGE`.
The glyph cache uses a memory-tier policy: `WING_GLYPH_CACHE_PROFILE=tiny`
selects 32 cells, `balanced`/default selects 96 cells, and `large` selects 192
cells. `WING_GLYPH_CACHE_CAPACITY` can override the profile and is clamped to
8..256 cells. The runtime records glyph cache hit rate, fill pressure,
evictions, and cached bitmap bytes; diagnostics can expose these as compact
`GLYPH` and `GHIT` rows. On startup the Shell prewarms a small
ordered glyph working set for the current display size; the warmup never evicts
existing cells, so tiny MCU profiles stay bounded.
The SVG cache follows the same shape: `WING_SVG_CACHE_PROFILE=tiny` selects 16
raster entries, `balanced`/default selects 40 entries, and `large` selects 64
entries. `WING_SVG_CACHE_CAPACITY` can override the profile and is clamped to
4..96 entries. Parsed SVG geometry is kept once in `SvgStore`; raster masks are
cached per icon and target size, so repeated Shell and default-app icon drawing
does not parse or tessellate every frame. System diagnostics can expose `ICON`
for filesystem SVG resource coverage and `SVG` for the raster cache tier/state.

For the current sim target the runtime paths are:

- `/etc/wing/resource/images/shell/wallpaper_aurora_landscape.png`
- `/etc/wing/resource/images/shell/wallpaper_aurora_portrait.png`
- `/etc/wing/resource/images/shell/wallpaper_dusk_landscape.png`
- `/etc/wing/resource/images/shell/wallpaper_dusk_portrait.png`
- `/etc/wing/resource/fonts/simhei.ttf`
- `/etc/wing/resource/fonts/shell_workset.txt`
- `/etc/wing/resource/icons/shell/*.svg`

The selected Shell/app icon source sets are copied from Bootstrap Icons v1.13.1
and kept under MIT license in `icons/bootstrap/LICENSE.bootstrap-icons`.

## Notes

For MCU/MPU targets, large original design resources should stay outside the
runtime filesystem. The compact RGB565 wallpaper payload is kept as the
deterministic fallback path; PNG/font resources are the higher-level source
path used when the platform has enough storage and decode budget. Current
runtime code recognizes PNG/JPEG and TTF/OTF/TTC headers, can decode PNG
wallpapers into RGB565 textures, parses SVG path icons into cached vector
rasters, uses the default TTF for Shell text drawing, and prewarms the Shell
glyph/SVG working sets at startup. Full JPEG pixel decode and persistent
on-disk glyph/SVG atlases are separate renderer backends still to be added.
