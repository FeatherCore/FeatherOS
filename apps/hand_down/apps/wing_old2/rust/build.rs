use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

mod build_bootstrap_icons;

const MAGIC: &[u8; 4] = b"WRS1";
const HEADER_LEN: u32 = 16;
const RECORD_LEN: u16 = 20;
const MANIFEST_MAGIC: &[u8; 4] = b"WRM1";
const MANIFEST_HEADER_LEN: u32 = 16;
const MANIFEST_RECORD_LEN: u16 = RECORD_LEN;

const FORMAT_RGB565: u8 = 1;
const FORMAT_A8: u8 = 2;
const MAX_MANIFEST_INCLUDE_DEPTH: usize = 8;

const IMAGE_WALLPAPER_AURORA: u16 = 1;
const IMAGE_WALLPAPER_DUSK: u16 = 2;
const IMAGE_WALLPAPER_AURORA_LANDSCAPE: u16 = 3;
const IMAGE_WALLPAPER_DUSK_LANDSCAPE: u16 = 4;
const DEFAULT_DISPLAY_WIDTH: u16 = 480;
const DEFAULT_DISPLAY_HEIGHT: u16 = 640;
const DEFAULT_PACKED_WALLPAPER_LONG_EDGE: u16 = 320;

struct Asset {
    id: u16,
    width: u16,
    height: u16,
    format: u8,
    data: Vec<u8>,
}

struct LoadedAssets {
    assets: Vec<Asset>,
    dependencies: Vec<PathBuf>,
}

#[derive(Clone, Copy)]
struct DisplaySize {
    short: u16,
    long: u16,
}

#[derive(Clone, Copy)]
enum WallpaperOrientation {
    Portrait,
    Landscape,
}

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=build_bootstrap_icons.rs");
    println!("cargo:rerun-if-env-changed=WING_DISPLAY_WIDTH");
    println!("cargo:rerun-if-env-changed=WING_DISPLAY_HEIGHT");
    println!("cargo:rerun-if-env-changed=WING_PACKED_WALLPAPER_LONG_EDGE");
    println!("cargo:rerun-if-env-changed=WING_GLYPH_CACHE_PROFILE");
    println!("cargo:rerun-if-env-changed=WING_GLYPH_CACHE_CAPACITY");
    println!("cargo:rerun-if-env-changed=WING_SVG_CACHE_PROFILE");
    println!("cargo:rerun-if-env-changed=WING_SVG_CACHE_CAPACITY");
    let manifest = resource_manifest_path();
    println!("cargo:rerun-if-changed={}", manifest.display());

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by cargo"));
    let display = display_size(&manifest);
    let loaded = load_manifest_assets(&manifest, display)?.unwrap_or_else(|| LoadedAssets {
        assets: bootstrap_assets(),
        dependencies: Vec::new(),
    });
    validate_unique_asset_ids(&loaded.assets).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: {}", manifest.display(), error),
        )
    })?;
    for dependency in &loaded.dependencies {
        println!("cargo:rerun-if-changed={}", dependency.display());
    }

    let blob = build_resource_blob(&loaded.assets);
    let (manifest_blob, payload_blob) = build_resource_manifest_pair(&loaded.assets);
    fs::write(out_dir.join("wing_assets.bin"), &blob)?;
    fs::write(out_dir.join("wing_manifest.bin"), &manifest_blob)?;
    fs::write(out_dir.join("wing_payload.bin"), &payload_blob)?;

    let generated_dir = resource_generated_dir(&manifest);
    fs::create_dir_all(&generated_dir)?;
    fs::write(generated_dir.join("wing_manifest.bin"), &manifest_blob)?;
    fs::write(generated_dir.join("wing_payload.bin"), &payload_blob)?;

    let resource_root = manifest.parent().unwrap_or_else(|| Path::new("."));
    build_bootstrap_icons::write_bootstrap_icon_module(&out_dir, resource_root)?;
    Ok(())
}

fn resource_manifest_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("Cargo manifest"));
    manifest_dir.join("../resource/manifest.txt")
}

fn resource_generated_dir(manifest: &Path) -> PathBuf {
    manifest
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("generated")
}

fn display_size(manifest: &Path) -> DisplaySize {
    if let Some((width, height)) = env_display_size() {
        return DisplaySize::new(width, height);
    }

    if let Some(config) = nuttx_config_path(manifest) {
        if config.exists() {
            println!("cargo:rerun-if-changed={}", config.display());
            if let Some((width, height)) = config_display_size(&config) {
                return DisplaySize::new(width, height);
            }
        }
    }

    DisplaySize::new(DEFAULT_DISPLAY_WIDTH, DEFAULT_DISPLAY_HEIGHT)
}

impl DisplaySize {
    fn new(width: u16, height: u16) -> Self {
        Self {
            short: width.min(height),
            long: width.max(height),
        }
    }
}

fn env_display_size() -> Option<(u16, u16)> {
    let width = parse_env_u16("WING_DISPLAY_WIDTH")?;
    let height = parse_env_u16("WING_DISPLAY_HEIGHT")?;
    Some((width, height))
}

fn parse_env_u16(name: &str) -> Option<u16> {
    let value = env::var(name).ok()?;
    let parsed = value.parse::<u16>().ok()?;
    if parsed == 0 {
        None
    } else {
        Some(parsed)
    }
}

fn nuttx_config_path(manifest: &Path) -> Option<PathBuf> {
    let resource_root = manifest.parent()?;
    let repo_root = resource_root.ancestors().nth(3)?;
    Some(repo_root.join("nuttx/.config"))
}

fn config_display_size(path: &Path) -> Option<(u16, u16)> {
    let text = fs::read_to_string(path).ok()?;
    let width = config_value_u16(&text, "CONFIG_SIM_FBWIDTH")?;
    let height = config_value_u16(&text, "CONFIG_SIM_FBHEIGHT")?;
    Some((width, height))
}

fn config_value_u16(text: &str, key: &str) -> Option<u16> {
    let prefix = format!("{}=", key);
    text.lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|value| *value != 0)
}

fn load_manifest_assets(path: &Path, display: DisplaySize) -> io::Result<Option<LoadedAssets>> {
    if !path.exists() {
        return Ok(None);
    }

    let text = fs::read_to_string(path)?;
    let base_dir = path.parent().unwrap_or_else(|| Path::new("."));
    parse_manifest(&text, base_dir, display)
        .map(Some)
        .map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{}: {}", path.display(), error),
            )
        })
}

fn parse_manifest(text: &str, base_dir: &Path, display: DisplaySize) -> Result<LoadedAssets, String> {
    let loaded = parse_manifest_inner(text, base_dir, display, 0)?;
    if loaded.assets.is_empty() {
        Err("manifest has no assets".to_owned())
    } else {
        validate_unique_asset_ids(&loaded.assets)?;
        Ok(loaded)
    }
}

fn validate_unique_asset_ids(assets: &[Asset]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for asset in assets {
        if !seen.insert(asset.id) {
            return Err(format!("duplicate asset id {}", asset.id));
        }
    }
    Ok(())
}

fn build_resource_blob(assets: &[Asset]) -> Vec<u8> {
    let data_offset = HEADER_LEN + assets.len() as u32 * RECORD_LEN as u32;
    let data_len = assets.iter().map(|asset| asset.data.len()).sum::<usize>() as u32;
    let mut blob = Vec::with_capacity(data_offset as usize + data_len as usize);

    blob.extend_from_slice(MAGIC);
    push_u16(&mut blob, assets.len() as u16);
    push_u16(&mut blob, RECORD_LEN);
    push_u32(&mut blob, data_offset);
    push_u32(&mut blob, data_offset + data_len);

    let mut offset = 0u32;
    for asset in assets {
        push_resource_record(&mut blob, asset, offset);
        offset += asset.data.len() as u32;
    }

    for asset in assets {
        blob.extend_from_slice(&asset.data);
    }

    blob
}

fn build_resource_manifest_pair(assets: &[Asset]) -> (Vec<u8>, Vec<u8>) {
    let manifest_len = MANIFEST_HEADER_LEN + assets.len() as u32 * MANIFEST_RECORD_LEN as u32;
    let payload_len = assets.iter().map(|asset| asset.data.len()).sum::<usize>() as u32;
    let mut manifest = Vec::with_capacity(manifest_len as usize);
    let mut payload = Vec::with_capacity(payload_len as usize);

    manifest.extend_from_slice(MANIFEST_MAGIC);
    push_u16(&mut manifest, assets.len() as u16);
    push_u16(&mut manifest, MANIFEST_RECORD_LEN);
    push_u32(&mut manifest, payload_len);
    push_u32(&mut manifest, manifest_len);

    let mut offset = 0u32;
    for asset in assets {
        push_resource_record(&mut manifest, asset, offset);
        payload.extend_from_slice(&asset.data);
        offset += asset.data.len() as u32;
    }

    (manifest, payload)
}

fn push_resource_record(blob: &mut Vec<u8>, asset: &Asset, offset: u32) {
    let start = blob.len();
    push_u16(blob, asset.id);
    push_u16(blob, asset.width);
    push_u16(blob, asset.height);
    blob.push(asset.format);
    blob.push(0);
    push_u16(blob, 0);
    push_u32(blob, offset);
    push_u32(blob, asset.data.len() as u32);
    push_u16(blob, 0);
    debug_assert_eq!(blob.len() - start, RECORD_LEN as usize);
}

fn parse_manifest_inner(
    text: &str,
    base_dir: &Path,
    display: DisplaySize,
    include_depth: usize,
) -> Result<LoadedAssets, String> {
    let lines = text.lines().collect::<Vec<_>>();
    let mut assets = Vec::new();
    let mut dependencies = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let line_no = index + 1;
        let line = manifest_line(lines[index]);
        index += 1;

        if line.is_empty() {
            continue;
        }

        let parts = line.split_whitespace().collect::<Vec<_>>();
        match parts.first().copied() {
            Some("asset") => {
                let (asset, dependency) =
                    parse_asset_line(&parts, line_no, &lines, &mut index, base_dir)?;
                if let Some(dependency) = dependency {
                    dependencies.push(dependency);
                }
                assets.push(asset);
            }
            Some("wallpaper") => {
                let (asset, mut wallpaper_dependencies) =
                    parse_wallpaper_line(&parts, line_no, base_dir, display)?;
                dependencies.append(&mut wallpaper_dependencies);
                assets.push(asset);
            }
            Some("wallpaper_gradient") => {
                let asset = parse_wallpaper_gradient_line(&parts, line_no, base_dir, display)?;
                assets.push(asset);
            }
            Some("include") => {
                if parts.len() != 2 {
                    return Err(format!("line {}: include requires a path", line_no));
                }
                if include_depth >= MAX_MANIFEST_INCLUDE_DEPTH {
                    return Err(format!("line {}: manifest include depth exceeded", line_no));
                }
                let path = resolve_asset_path(base_dir, parts[1], line_no)?;
                let text = fs::read_to_string(&path).map_err(|error| {
                    format!(
                        "line {}: cannot read include `{}`: {}",
                        line_no,
                        path.display(),
                        error
                    )
                })?;
                dependencies.push(path);
                let mut included = parse_manifest_inner(&text, base_dir, display, include_depth + 1)?;
                dependencies.append(&mut included.dependencies);
                assets.append(&mut included.assets);
            }
            Some("atlas") => parse_atlas_block(
                &parts,
                line_no,
                &lines,
                &mut index,
                base_dir,
                &mut assets,
                &mut dependencies,
            )?,
            Some(kind) => {
                return Err(format!(
                    "line {}: expected `asset`, `wallpaper`, `wallpaper_gradient`, `atlas` or `include`, got `{}`",
                    line_no, kind
                ));
            }
            None => {}
        }
    }

    Ok(LoadedAssets {
        assets,
        dependencies,
    })
}

fn parse_wallpaper_line(
    parts: &[&str],
    line_no: usize,
    base_dir: &Path,
    display: DisplaySize,
) -> Result<(Asset, Vec<PathBuf>), String> {
    if parts.len() != 5 {
        return Err(format!(
            "line {}: wallpaper requires id orientation source.png output.rgb565",
            line_no
        ));
    }

    let id = parse_u16(parts[1], "id", line_no)?;
    let orientation = parse_wallpaper_orientation(parts[2], line_no)?;
    let source = resolve_asset_path(base_dir, parts[3], line_no)?;
    let output = resolve_asset_path(base_dir, parts[4], line_no)?;
    let tool = base_dir.join("tools/wing_wallpaper_build.py");
    let (width, height) = packed_wallpaper_dimensions(display, orientation);

    println!("cargo:rerun-if-changed={}", tool.display());
    build_wallpaper_rgb565(&tool, &source, &output, width, height, line_no)?;
    let data = fs::read(&output).map_err(|error| {
        format!(
            "line {}: cannot read generated wallpaper `{}`: {}",
            line_no,
            output.display(),
            error
        )
    })?;
    let expected = expected_asset_len(FORMAT_RGB565, width, height, line_no)?;
    if data.len() != expected {
        return Err(format!(
            "line {}: generated wallpaper `{}` has {} bytes, expected {} for rgb565 {}x{}",
            line_no,
            output.display(),
            data.len(),
            expected,
            width,
            height
        ));
    }

    Ok((
        Asset {
            id,
            width,
            height,
            format: FORMAT_RGB565,
            data,
        },
        vec![source, tool],
    ))
}

fn parse_wallpaper_gradient_line(
    parts: &[&str],
    line_no: usize,
    base_dir: &Path,
    display: DisplaySize,
) -> Result<Asset, String> {
    if parts.len() != 7 {
        return Err(format!(
            "line {}: wallpaper_gradient requires id orientation output.rgb565 top bottom accent",
            line_no
        ));
    }

    let id = parse_u16(parts[1], "id", line_no)?;
    let orientation = parse_wallpaper_orientation(parts[2], line_no)?;
    let output = resolve_asset_path(base_dir, parts[3], line_no)?;
    let top = parse_hex_color(parts[4], line_no)?;
    let bottom = parse_hex_color(parts[5], line_no)?;
    let accent = parse_hex_color(parts[6], line_no)?;
    let (width, height) = packed_wallpaper_dimensions(display, orientation);
    let data = wallpaper_rgb565(width, height, top, bottom, accent);

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "line {}: cannot create wallpaper output directory `{}`: {}",
                line_no,
                parent.display(),
                error
            )
        })?;
    }
    fs::write(&output, &data).map_err(|error| {
        format!(
            "line {}: cannot write generated wallpaper `{}`: {}",
            line_no,
            output.display(),
            error
        )
    })?;

    Ok(Asset {
        id,
        width,
        height,
        format: FORMAT_RGB565,
        data,
    })
}

fn parse_wallpaper_orientation(
    value: &str,
    line_no: usize,
) -> Result<WallpaperOrientation, String> {
    match value {
        "portrait" | "vertical" => Ok(WallpaperOrientation::Portrait),
        "landscape" | "horizontal" => Ok(WallpaperOrientation::Landscape),
        _ => Err(format!(
            "line {}: wallpaper orientation must be portrait or landscape",
            line_no
        )),
    }
}

fn packed_wallpaper_dimensions(
    display: DisplaySize,
    orientation: WallpaperOrientation,
) -> (u16, u16) {
    let long = packed_wallpaper_long_edge().min(display.long).max(1);
    let short = scale_short_edge(display.short, display.long, long);
    match orientation {
        WallpaperOrientation::Portrait => (short, long),
        WallpaperOrientation::Landscape => (long, short),
    }
}

fn packed_wallpaper_long_edge() -> u16 {
    parse_env_u16("WING_PACKED_WALLPAPER_LONG_EDGE")
        .unwrap_or(DEFAULT_PACKED_WALLPAPER_LONG_EDGE)
        .max(1)
}

fn scale_short_edge(short: u16, long: u16, target_long: u16) -> u16 {
    if long == 0 {
        return target_long;
    }

    let scaled = (short as u32 * target_long as u32 + long as u32 / 2) / long as u32;
    scaled.clamp(1, u16::MAX as u32) as u16
}

fn build_wallpaper_rgb565(
    tool: &Path,
    source: &Path,
    output: &Path,
    width: u16,
    height: u16,
    line_no: usize,
) -> Result<(), String> {
    if !source.exists() {
        return Err(format!(
            "line {}: wallpaper source `{}` does not exist",
            line_no,
            source.display()
        ));
    }
    if !tool.exists() {
        return Err(format!(
            "line {}: wallpaper tool `{}` does not exist",
            line_no,
            tool.display()
        ));
    }

    let result = Command::new("python3")
        .arg(tool)
        .arg(source)
        .arg(output)
        .arg(width.to_string())
        .arg(height.to_string())
        .output()
        .map_err(|error| {
            format!(
                "line {}: failed to run wallpaper converter `{}`: {}",
                line_no,
                tool.display(),
                error
            )
        })?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        return Err(format!(
            "line {}: wallpaper converter failed for `{}`: {}",
            line_no,
            source.display(),
            stderr.trim()
        ));
    }

    Ok(())
}

fn parse_asset_line(
    parts: &[&str],
    line_no: usize,
    lines: &[&str],
    index: &mut usize,
    base_dir: &Path,
) -> Result<(Asset, Option<PathBuf>), String> {
    if parts.len() < 6 {
        return Err(format!("line {}: asset requires id format width height kind", line_no));
    }

    let id = parse_u16(parts[1], "id", line_no)?;
    let format = parse_format(parts[2], line_no)?;
    let width = parse_u16(parts[3], "width", line_no)?;
    let height = parse_u16(parts[4], "height", line_no)?;
    validate_dimensions(width, height, line_no)?;

    let (data, dependency) = match (format, parts[5]) {
        (FORMAT_RGB565, "gradient") => {
            if parts.len() != 9 {
                return Err(format!("line {}: rgb565 gradient requires 3 colors", line_no));
            }
            (
                wallpaper_rgb565(
                    width,
                    height,
                    parse_hex_color(parts[6], line_no)?,
                    parse_hex_color(parts[7], line_no)?,
                    parse_hex_color(parts[8], line_no)?,
                ),
                None,
            )
        }
        (FORMAT_A8, "bitmap") => {
            if parts.len() != 6 {
                return Err(format!("line {}: a8 bitmap does not take inline args", line_no));
            }
            let mut rows = Vec::with_capacity(height as usize);
            while rows.len() < height as usize && *index < lines.len() {
                let row_no = *index + 1;
                let row = manifest_line(lines[*index]);
                *index += 1;
                if row.is_empty() {
                    continue;
                }
                if row == "end" {
                    break;
                }
                rows.push((row_no, row.to_owned()));
            }

            if rows.len() != height as usize {
                return Err(format!("line {}: bitmap expected {} rows", line_no, height));
            }

            if *index < lines.len() && manifest_line(lines[*index]) == "end" {
                *index += 1;
            }

            (bitmap_a8(width, height, &rows)?, None)
        }
        (FORMAT_RGB565, "file") | (FORMAT_A8, "file") => {
            if parts.len() != 7 {
                return Err(format!("line {}: file asset requires a path", line_no));
            }
            let path = resolve_asset_path(base_dir, parts[6], line_no)?;
            let data = fs::read(&path).map_err(|error| {
                format!("line {}: cannot read `{}`: {}", line_no, path.display(), error)
            })?;
            let expected = expected_asset_len(format, width, height, line_no)?;
            if data.len() != expected {
                return Err(format!(
                    "line {}: `{}` has {} bytes, expected {} for {} {}x{}",
                    line_no,
                    path.display(),
                    data.len(),
                    expected,
                    format_label(format),
                    width,
                    height
                ));
            }
            (data, Some(path))
        }
        (FORMAT_RGB565, "atlas") | (FORMAT_A8, "atlas") => {
            if parts.len() != 11 {
                return Err(format!(
                    "line {}: atlas asset requires path atlas_width atlas_height x y",
                    line_no
                ));
            }
            let path = resolve_asset_path(base_dir, parts[6], line_no)?;
            let atlas_width = parse_u16(parts[7], "atlas_width", line_no)?;
            let atlas_height = parse_u16(parts[8], "atlas_height", line_no)?;
            let x = parse_u16(parts[9], "x", line_no)?;
            let y = parse_u16(parts[10], "y", line_no)?;
            validate_dimensions(atlas_width, atlas_height, line_no)?;

            let atlas = fs::read(&path).map_err(|error| {
                format!("line {}: cannot read `{}`: {}", line_no, path.display(), error)
            })?;
            let expected = expected_asset_len(format, atlas_width, atlas_height, line_no)?;
            if atlas.len() != expected {
                return Err(format!(
                    "line {}: `{}` has {} bytes, expected {} for {} atlas {}x{}",
                    line_no,
                    path.display(),
                    atlas.len(),
                    expected,
                    format_label(format),
                    atlas_width,
                    atlas_height
                ));
            }

            (
                atlas_slice(
                    format,
                    width,
                    height,
                    atlas_width,
                    atlas_height,
                    x,
                    y,
                    &atlas,
                    line_no,
                )?,
                Some(path),
            )
        }
        (FORMAT_RGB565, kind) | (FORMAT_A8, kind) => {
            return Err(format!("line {}: unsupported asset kind `{}`", line_no, kind));
        }
        _ => unreachable!(),
    };

    Ok((
        Asset {
            id,
            width,
            height,
            format,
            data,
        },
        dependency,
    ))
}

fn parse_atlas_block(
    parts: &[&str],
    line_no: usize,
    lines: &[&str],
    index: &mut usize,
    base_dir: &Path,
    assets: &mut Vec<Asset>,
    dependencies: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if parts.len() != 5 {
        return Err(format!(
            "line {}: atlas block requires format path atlas_width atlas_height",
            line_no
        ));
    }

    let format = parse_format(parts[1], line_no)?;
    let path = resolve_asset_path(base_dir, parts[2], line_no)?;
    let atlas_width = parse_u16(parts[3], "atlas_width", line_no)?;
    let atlas_height = parse_u16(parts[4], "atlas_height", line_no)?;
    validate_dimensions(atlas_width, atlas_height, line_no)?;

    let atlas = fs::read(&path)
        .map_err(|error| format!("line {}: cannot read `{}`: {}", line_no, path.display(), error))?;
    let expected = expected_asset_len(format, atlas_width, atlas_height, line_no)?;
    if atlas.len() != expected {
        return Err(format!(
            "line {}: `{}` has {} bytes, expected {} for {} atlas {}x{}",
            line_no,
            path.display(),
            atlas.len(),
            expected,
            format_label(format),
            atlas_width,
            atlas_height
        ));
    }

    let mut slice_count = 0usize;
    while *index < lines.len() {
        let slice_line_no = *index + 1;
        let line = manifest_line(lines[*index]);
        *index += 1;

        if line.is_empty() {
            continue;
        }

        let slice_parts = line.split_whitespace().collect::<Vec<_>>();
        match slice_parts.first().copied() {
            Some("end") => {
                if slice_parts.len() != 1 {
                    return Err(format!("line {}: end does not take arguments", slice_line_no));
                }
                if slice_count == 0 {
                    return Err(format!("line {}: atlas block has no slices", line_no));
                }
                dependencies.push(path);
                return Ok(());
            }
            Some("slice") => {
                if slice_parts.len() != 6 {
                    return Err(format!(
                        "line {}: slice requires id width height x y",
                        slice_line_no
                    ));
                }
                let id = parse_u16(slice_parts[1], "id", slice_line_no)?;
                let width = parse_u16(slice_parts[2], "width", slice_line_no)?;
                let height = parse_u16(slice_parts[3], "height", slice_line_no)?;
                let x = parse_u16(slice_parts[4], "x", slice_line_no)?;
                let y = parse_u16(slice_parts[5], "y", slice_line_no)?;
                validate_dimensions(width, height, slice_line_no)?;

                let data = atlas_slice(
                    format,
                    width,
                    height,
                    atlas_width,
                    atlas_height,
                    x,
                    y,
                    &atlas,
                    slice_line_no,
                )?;
                assets.push(Asset {
                    id,
                    width,
                    height,
                    format,
                    data,
                });
                slice_count += 1;
            }
            Some(kind) => {
                return Err(format!(
                    "line {}: expected `slice` or `end` in atlas block, got `{}`",
                    slice_line_no, kind
                ));
            }
            None => {}
        }
    }

    Err(format!("line {}: atlas block missing end", line_no))
}

fn bootstrap_assets() -> Vec<Asset> {
    vec![
        Asset {
            id: IMAGE_WALLPAPER_AURORA,
            width: 64,
            height: 96,
            format: FORMAT_RGB565,
            data: wallpaper_rgb565(
                64,
                96,
                Rgb::new(12, 23, 46),
                Rgb::new(16, 90, 98),
                Rgb::new(78, 176, 154),
            ),
        },
        Asset {
            id: IMAGE_WALLPAPER_DUSK,
            width: 64,
            height: 96,
            format: FORMAT_RGB565,
            data: wallpaper_rgb565(
                64,
                96,
                Rgb::new(35, 24, 50),
                Rgb::new(112, 54, 70),
                Rgb::new(204, 118, 78),
            ),
        },
        Asset {
            id: IMAGE_WALLPAPER_AURORA_LANDSCAPE,
            width: 96,
            height: 64,
            format: FORMAT_RGB565,
            data: wallpaper_rgb565(
                96,
                64,
                Rgb::new(12, 23, 46),
                Rgb::new(16, 90, 98),
                Rgb::new(78, 176, 154),
            ),
        },
        Asset {
            id: IMAGE_WALLPAPER_DUSK_LANDSCAPE,
            width: 96,
            height: 64,
            format: FORMAT_RGB565,
            data: wallpaper_rgb565(
                96,
                64,
                Rgb::new(35, 24, 50),
                Rgb::new(112, 54, 70),
                Rgb::new(204, 118, 78),
            ),
        },
    ]
}

fn manifest_line(line: &str) -> &str {
    line.split_once('#').map(|(body, _)| body).unwrap_or(line).trim()
}

fn parse_format(value: &str, line_no: usize) -> Result<u8, String> {
    match value {
        "rgb565" => Ok(FORMAT_RGB565),
        "a8" => Ok(FORMAT_A8),
        _ => Err(format!("line {}: unsupported format `{}`", line_no, value)),
    }
}

fn parse_u16(value: &str, label: &str, line_no: usize) -> Result<u16, String> {
    value
        .parse::<u16>()
        .map_err(|_| format!("line {}: invalid {} `{}`", line_no, label, value))
}

fn validate_dimensions(width: u16, height: u16, line_no: usize) -> Result<(), String> {
    if width == 0 || height == 0 {
        Err(format!("line {}: width and height must be non-zero", line_no))
    } else {
        Ok(())
    }
}

fn parse_hex_color(value: &str, line_no: usize) -> Result<Rgb, String> {
    let hex = value.trim_start_matches("0x");
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("line {}: invalid color `{}`", line_no, value));
    }

    Ok(Rgb::new(
        parse_hex_byte(&hex[0..2], line_no)?,
        parse_hex_byte(&hex[2..4], line_no)?,
        parse_hex_byte(&hex[4..6], line_no)?,
    ))
}

fn parse_hex_byte(value: &str, line_no: usize) -> Result<u8, String> {
    u8::from_str_radix(value, 16)
        .map_err(|_| format!("line {}: invalid hex byte `{}`", line_no, value))
}

fn resolve_asset_path(base_dir: &Path, value: &str, line_no: usize) -> Result<PathBuf, String> {
    let path = Path::new(value);
    if path.is_absolute() {
        return Err(format!("line {}: asset file path must be relative", line_no));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            _ => {
                return Err(format!(
                    "line {}: asset file path cannot escape resource directory",
                    line_no
                ))
            }
        }
    }

    Ok(base_dir.join(path))
}

fn expected_asset_len(format: u8, width: u16, height: u16, line_no: usize) -> Result<usize, String> {
    let pixels = (width as usize)
        .checked_mul(height as usize)
        .ok_or_else(|| format!("line {}: asset dimensions overflow", line_no))?;
    let bytes_per_pixel = bytes_per_pixel(format, line_no)?;

    pixels
        .checked_mul(bytes_per_pixel)
        .ok_or_else(|| format!("line {}: asset byte length overflow", line_no))
}

fn bytes_per_pixel(format: u8, line_no: usize) -> Result<usize, String> {
    match format {
        FORMAT_RGB565 => Ok(2),
        FORMAT_A8 => Ok(1),
        _ => Err(format!("line {}: unsupported format", line_no)),
    }
}

fn format_label(format: u8) -> &'static str {
    match format {
        FORMAT_RGB565 => "rgb565",
        FORMAT_A8 => "a8",
        _ => "unknown",
    }
}

fn atlas_slice(
    format: u8,
    width: u16,
    height: u16,
    atlas_width: u16,
    atlas_height: u16,
    x: u16,
    y: u16,
    atlas: &[u8],
    line_no: usize,
) -> Result<Vec<u8>, String> {
    let end_x = x
        .checked_add(width)
        .ok_or_else(|| format!("line {}: atlas slice x range overflow", line_no))?;
    let end_y = y
        .checked_add(height)
        .ok_or_else(|| format!("line {}: atlas slice y range overflow", line_no))?;
    if end_x > atlas_width || end_y > atlas_height {
        return Err(format!(
            "line {}: atlas slice {}x{} at {},{} exceeds atlas {}x{}",
            line_no, width, height, x, y, atlas_width, atlas_height
        ));
    }

    let bytes_per_pixel = bytes_per_pixel(format, line_no)?;
    let row_bytes = width as usize * bytes_per_pixel;
    let atlas_stride = atlas_width as usize * bytes_per_pixel;
    let mut data = Vec::with_capacity(row_bytes * height as usize);

    for row in 0..height as usize {
        let src_y = y as usize + row;
        let start = src_y
            .checked_mul(atlas_stride)
            .and_then(|offset| offset.checked_add(x as usize * bytes_per_pixel))
            .ok_or_else(|| format!("line {}: atlas slice offset overflow", line_no))?;
        let end = start
            .checked_add(row_bytes)
            .ok_or_else(|| format!("line {}: atlas slice row overflow", line_no))?;
        let Some(slice) = atlas.get(start..end) else {
            return Err(format!("line {}: atlas slice row outside data", line_no));
        };
        data.extend_from_slice(slice);
    }

    Ok(data)
}

fn bitmap_a8(width: u16, height: u16, rows: &[(usize, String)]) -> Result<Vec<u8>, String> {
    let mut data = Vec::with_capacity(width as usize * height as usize);
    for (line_no, row) in rows {
        if row.len() != width as usize {
            return Err(format!("line {}: bitmap row must be {} pixels", line_no, width));
        }

        for byte in row.bytes() {
            match byte {
                b'0' | b'.' => data.push(0),
                b'1' | b'x' | b'X' => data.push(255),
                _ => {
                    return Err(format!(
                        "line {}: bitmap row only supports 0/1/./x",
                        line_no
                    ))
                }
            }
        }
    }

    Ok(data)
}

#[derive(Clone, Copy)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

fn wallpaper_rgb565(width: u16, height: u16, top: Rgb, bottom: Rgb, accent: Rgb) -> Vec<u8> {
    let mut data = vec![0u8; width as usize * height as usize * 2];
    let height_denom = height.saturating_sub(1).max(1) as u32;
    let band_width = (width as i32 / 4).max(6);

    for y in 0..height {
        let t = ((y as u32 * 255) / height_denom) as u8;
        let base = blend_rgb(top, bottom, t);
        for x in 0..width {
            let diagonal = (y as i32 * width as i32 / height as i32) + width as i32 / 5;
            let distance = (x as i32 - diagonal).abs();
            let band = if distance < band_width {
                ((band_width - distance) * 180 / band_width) as u8
            } else {
                0
            };
            let sparkle = if (x as u32 * 17 + y as u32 * 31) % 89 == 0 {
                90
            } else {
                0
            };
            let color = rgb565(blend_rgb(base, accent, band.max(sparkle)));
            let offset = (y as usize * width as usize + x as usize) * 2;
            data[offset] = (color & 0xff) as u8;
            data[offset + 1] = (color >> 8) as u8;
        }
    }
    data
}

fn blend_rgb(a: Rgb, b: Rgb, t: u8) -> Rgb {
    let inv = 255u16.saturating_sub(t as u16);
    let t = t as u16;
    Rgb::new(
        ((a.r as u16 * inv + b.r as u16 * t) / 255) as u8,
        ((a.g as u16 * inv + b.g as u16 * t) / 255) as u8,
        ((a.b as u16 * inv + b.b as u16 * t) / 255) as u8,
    )
}

fn rgb565(color: Rgb) -> u16 {
    (((color.r as u16 >> 3) & 0x1f) << 11)
        | (((color.g as u16 >> 2) & 0x3f) << 5)
        | ((color.b as u16 >> 3) & 0x1f)
}

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}
