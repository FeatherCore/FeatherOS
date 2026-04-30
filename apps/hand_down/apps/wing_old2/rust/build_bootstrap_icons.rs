use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

struct BootstrapIconSpec {
    variant: &'static str,
    file: &'static str,
}

const SHELL_BOOTSTRAP_ICONS: &[BootstrapIconSpec] = &[
    BootstrapIconSpec {
        variant: "Wifi",
        file: "wifi.svg",
    },
    BootstrapIconSpec {
        variant: "Bluetooth",
        file: "bluetooth.svg",
    },
    BootstrapIconSpec {
        variant: "Phone",
        file: "phone-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Chat",
        file: "chat-dots-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Settings",
        file: "gear-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Camera",
        file: "camera-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Flashlight",
        file: "lightbulb-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Airplane",
        file: "airplane-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Moon",
        file: "moon-stars-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Sync",
        file: "arrow-repeat.svg",
    },
    BootstrapIconSpec {
        variant: "Mail",
        file: "envelope-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Cloud",
        file: "cloud-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Folder",
        file: "folder-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Music",
        file: "music-note-beamed.svg",
    },
    BootstrapIconSpec {
        variant: "Play",
        file: "play-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Wing",
        file: "stars.svg",
    },
    BootstrapIconSpec {
        variant: "Check",
        file: "check-lg.svg",
    },
    BootstrapIconSpec {
        variant: "Close",
        file: "x-lg.svg",
    },
    BootstrapIconSpec {
        variant: "Alert",
        file: "exclamation-triangle-fill.svg",
    },
    BootstrapIconSpec {
        variant: "More",
        file: "three-dots.svg",
    },
    BootstrapIconSpec {
        variant: "System",
        file: "cpu-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Terminal",
        file: "terminal.svg",
    },
    BootstrapIconSpec {
        variant: "Surface",
        file: "window-stack.svg",
    },
];

const SETTINGS_APP_ICONS: &[BootstrapIconSpec] = &[
    BootstrapIconSpec {
        variant: "Settings",
        file: "gear-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Brightness",
        file: "brightness-high-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Haptic",
        file: "phone-vibrate-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Motion",
        file: "speedometer2.svg",
    },
    BootstrapIconSpec {
        variant: "Back",
        file: "chevron-left.svg",
    },
    BootstrapIconSpec {
        variant: "Palette",
        file: "palette-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Preview",
        file: "collection-fill.svg",
    },
];

const SYSTEM_APP_ICONS: &[BootstrapIconSpec] = &[
    BootstrapIconSpec {
        variant: "System",
        file: "cpu-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Back",
        file: "chevron-left.svg",
    },
    BootstrapIconSpec {
        variant: "Surface",
        file: "window-stack.svg",
    },
    BootstrapIconSpec {
        variant: "Terminal",
        file: "terminal.svg",
    },
    BootstrapIconSpec {
        variant: "Settings",
        file: "gear-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Preview",
        file: "collection-fill.svg",
    },
];

const TERMINAL_APP_ICONS: &[BootstrapIconSpec] = &[
    BootstrapIconSpec {
        variant: "Terminal",
        file: "terminal.svg",
    },
    BootstrapIconSpec {
        variant: "Back",
        file: "chevron-left.svg",
    },
    BootstrapIconSpec {
        variant: "System",
        file: "cpu-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Surface",
        file: "window-stack.svg",
    },
    BootstrapIconSpec {
        variant: "Alert",
        file: "exclamation-triangle-fill.svg",
    },
];

const SURFACE_DEMO_APP_ICONS: &[BootstrapIconSpec] = &[
    BootstrapIconSpec {
        variant: "Wing",
        file: "stars.svg",
    },
    BootstrapIconSpec {
        variant: "Preview",
        file: "collection-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Palette",
        file: "palette-fill.svg",
    },
    BootstrapIconSpec {
        variant: "Brightness",
        file: "brightness-high-fill.svg",
    },
];

pub fn write_bootstrap_icon_module(out_dir: &Path, resource_root: &Path) -> io::Result<()> {
    println!("cargo:rerun-if-env-changed=WING_BOOTSTRAP_ICON_DIR");
    println!("cargo:rerun-if-env-changed=WING_SETTINGS_APP_ICON_DIR");
    println!("cargo:rerun-if-env-changed=WING_SYSTEM_APP_ICON_DIR");
    println!("cargo:rerun-if-env-changed=WING_TERMINAL_APP_ICON_DIR");
    println!("cargo:rerun-if-env-changed=WING_SURFACE_DEMO_APP_ICON_DIR");

    let shell_icon_dir = env::var_os("WING_BOOTSTRAP_ICON_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| resource_root.join("icons/bootstrap"));
    let settings_icon_dir = env::var_os("WING_SETTINGS_APP_ICON_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| resource_root.join("apps/settings/icons"));
    let system_icon_dir = env::var_os("WING_SYSTEM_APP_ICON_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| resource_root.join("apps/system/icons"));
    let terminal_icon_dir = env::var_os("WING_TERMINAL_APP_ICON_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| resource_root.join("apps/terminal/icons"));
    let surface_demo_icon_dir = env::var_os("WING_SURFACE_DEMO_APP_ICON_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| resource_root.join("apps/surface_demo/icons"));
    let output = out_dir.join("wing_bootstrap_icons.rs");
    let mut module = String::new();

    module.push_str("// @generated by apps/wing/rust/build_bootstrap_icons.rs\n");
    module.push_str("// Source: Bootstrap Icons v1.13.1, MIT License.\n");
    write_register_fn(
        &mut module,
        "register_bootstrap_svg_icons",
        SHELL_BOOTSTRAP_ICONS,
        &shell_icon_dir,
    )?;
    write_register_fn(
        &mut module,
        "register_settings_app_svg_icons",
        SETTINGS_APP_ICONS,
        &settings_icon_dir,
    )?;
    write_register_fn(
        &mut module,
        "register_system_app_svg_icons",
        SYSTEM_APP_ICONS,
        &system_icon_dir,
    )?;
    write_register_fn(
        &mut module,
        "register_terminal_app_svg_icons",
        TERMINAL_APP_ICONS,
        &terminal_icon_dir,
    )?;
    write_register_fn(
        &mut module,
        "register_surface_demo_app_svg_icons",
        SURFACE_DEMO_APP_ICONS,
        &surface_demo_icon_dir,
    )?;
    module.push_str("pub(crate) fn register_default_app_svg_icons(store: &mut SvgStore) {\n");
    module.push_str("    register_settings_app_svg_icons(store);\n");
    module.push_str("    register_system_app_svg_icons(store);\n");
    module.push_str("    register_terminal_app_svg_icons(store);\n");
    module.push_str("    register_surface_demo_app_svg_icons(store);\n");
    module.push_str("}\n");

    fs::write(output, module)
}

fn write_register_fn(
    module: &mut String,
    name: &str,
    specs: &[BootstrapIconSpec],
    icon_dir: &Path,
) -> io::Result<()> {
    writeln!(module, "pub(crate) fn {}(store: &mut SvgStore) {{", name).unwrap();
    for spec in specs {
        let path = icon_dir.join(spec.file);
        println!("cargo:rerun-if-changed={}", path.display());
        fs::metadata(&path).map_err(|error| {
            io::Error::new(
                error.kind(),
                format!("cannot find Bootstrap icon `{}`: {}", path.display(), error),
            )
        })?;
        writeln!(
            module,
            "    let _ = store.register_vector_icon(VectorIcon::{}, include_bytes!(r#\"{}\"#));",
            spec.variant,
            path.display()
        )
        .unwrap();
    }
    module.push_str("}\n");
    Ok(())
}
