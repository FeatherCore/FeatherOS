use fhre::{
    math::Rect,
    pipeline::Gradient,
    render_world::{ExtractedUI, PhaseItem, RenderPhaseType},
    Color, GpuTextures, MainEntity, MainWorld, RenderCommand, RenderWorld, Transform,
};

use crate::{
    components::{
        app_switcher_ball_center, app_switcher_ball_size, surface_index_for_face,
        AppSurface, BrightnessControl, LauncherIcon, NotificationCard, NotificationPanel,
        OverlayLayer, PreviewSoccerBall, QuickControlTile, SettingsPanel, SettingsRow,
        SurfacePreviewCard, SystemNavButton,
    },
    resources::{
        PreviewEffect, ShellContent, ShellMetrics, ShellOverlayAnimation, ShellState, ThemeVariant,
    },
};

use super::{ExtractedShellImage, ExtractedShellText, ExtractedThemeBackdrop};

/// Sort key ranges for proper rendering order within Ui phase.
/// Lower values render first (background), higher values render later (foreground).
mod sort_keys {
    pub const BACKDROP: i32 = -10000;
    pub const BACKDROP_DETAIL: i32 = -9000;
    /// Background elements (rendered first, covered by text)
    pub const BACKGROUND: i32 = 0;
    pub const PREVIEW_BALL: i32 = 9000;
    pub const PREVIEW_WIREFRAME: i32 = 9200;
    pub const PREVIEW_LABEL: i32 = 9500;
    pub const IMAGE: i32 = 12000;
    /// Text elements (rendered last, on top of backgrounds)
    pub const TEXT: i32 = 20000;
}

pub fn queue_wing_primitives(main_world: &MainWorld, render_world: &mut RenderWorld) {
    queue_theme_backdrop(main_world, render_world);

    // Queue UI rectangles first (backgrounds)
    let uis: alloc::vec::Vec<(fhre::Entity, ExtractedUI)> = render_world
        .query::<ExtractedUI>()
        .map(|(entity, ui)| (entity, ui.clone()))
        .collect();

    for (entity, ui) in uis {
        let rect = Rect::from_center_size(ui.position, fhre::Vec2::new(ui.width, ui.height));
        let Some(command) = ui_command_for_entity(main_world, render_world, entity, rect, ui.color) else {
            continue;
        };
        let sort_key = ui_sort_key(main_world, render_world, entity);
        render_world.add_phase_item(RenderPhaseType::Ui, PhaseItem::ui(command, sort_key));
    }

    queue_preview_soccer_ball(main_world, render_world);
    queue_shell_images(render_world);

    // Queue text last (foreground, on top of backgrounds)
    let shell_texts: alloc::vec::Vec<(fhre::Entity, ExtractedShellText)> = render_world
        .query::<ExtractedShellText>()
        .map(|(entity, text)| (entity, text.clone()))
        .collect();

    for (entity, text) in shell_texts {
        let command = RenderCommand::draw_text(text.position, text.text, text.color, text.size);
        let sort_key = sort_keys::TEXT + (entity.id() % 1000) as i32;
        render_world.add_phase_item(RenderPhaseType::Ui, PhaseItem::ui(command, sort_key));
    }
}

fn queue_shell_images(render_world: &mut RenderWorld) {
    let images: alloc::vec::Vec<ExtractedShellImage> = render_world
        .query::<ExtractedShellImage>()
        .map(|(_, image)| image.clone())
        .collect();
    let gpu_textures = render_world.get_resource::<GpuTextures>().cloned();

    for image in images {
        let texture_id = gpu_textures
            .as_ref()
            .and_then(|textures| textures.get(image.image.id()))
            .map(|texture| texture.id);

        let Some(texture_id) = texture_id else {
            continue;
        };

        render_world.add_phase_item(
            RenderPhaseType::Ui,
            PhaseItem::ui(
                RenderCommand::draw_image_full(
                    Rect::from_center_size(
                        image.position,
                        fhre::Vec2::new(image.width, image.height),
                    ),
                    texture_id,
                    image.color,
                ),
                sort_keys::IMAGE + image.sort_key,
            ),
        );
    }
}

fn ui_command_for_entity(
    main_world: &MainWorld,
    render_world: &RenderWorld,
    render_entity: fhre::Entity,
    rect: Rect,
    color: Color,
) -> Option<RenderCommand> {
    let main_entity = main_entity_for(render_world, render_entity);

    if main_world.get_component::<OverlayLayer>(main_entity).is_some()
        || main_world.get_component::<AppSurface>(main_entity).is_some()
    {
        return Some(RenderCommand::DrawRect { rect, color });
    }

    if main_world.get_component::<NotificationPanel>(main_entity).is_some() {
        return Some(RenderCommand::DrawRectRoundedGradient {
            rect,
            gradient: soft_vertical_gradient(color),
            radius: 22.0,
        });
    }

    if main_world.get_component::<BrightnessControl>(main_entity).is_some() {
        return Some(RenderCommand::DrawRectRoundedGradient {
            rect,
            gradient: soft_vertical_gradient(color),
            radius: rect.height * 0.5,
        });
    }

    if main_world.get_component::<QuickControlTile>(main_entity).is_some()
        || main_world.get_component::<LauncherIcon>(main_entity).is_some()
        || main_world.get_component::<SystemNavButton>(main_entity).is_some()
    {
        return Some(RenderCommand::DrawRectRoundedGradient {
            rect,
            gradient: soft_vertical_gradient(color),
            radius: rect.width.min(rect.height) * 0.24,
        });
    }

    if main_world.get_component::<NotificationCard>(main_entity).is_some()
        || main_world.get_component::<SurfacePreviewCard>(main_entity).is_some()
        || main_world.get_component::<SettingsRow>(main_entity).is_some()
    {
        return Some(RenderCommand::DrawRectRoundedGradient {
            rect,
            gradient: soft_vertical_gradient(color),
            radius: 14.0,
        });
    }

    if main_world.get_component::<SettingsPanel>(main_entity).is_some() {
        return Some(RenderCommand::DrawRectRoundedGradient {
            rect,
            gradient: soft_vertical_gradient(color),
            radius: 18.0,
        });
    }

    Some(RenderCommand::DrawRect { rect, color })
}

fn ui_sort_key(
    main_world: &MainWorld,
    render_world: &RenderWorld,
    render_entity: fhre::Entity,
) -> i32 {
    let main_entity = main_entity_for(render_world, render_entity);
    let z_key = main_world
        .get_component::<Transform>(main_entity)
        .map(|transform| (transform.position.z * 100_000.0) as i32)
        .unwrap_or(0);
    sort_keys::BACKGROUND + z_key + (render_entity.id() % 100) as i32
}

fn main_entity_for(render_world: &RenderWorld, render_entity: fhre::Entity) -> fhre::Entity {
    render_world
        .get_component::<MainEntity>(render_entity)
        .map(|main_entity| main_entity.0)
        .unwrap_or(render_entity)
}

fn soft_vertical_gradient(color: Color) -> Gradient {
    let top = tint(color, Color::WHITE, 0.26);
    let bottom = tint(color, Color::BLACK, 0.05);
    Gradient::vertical(&[top, color, bottom])
}

fn tint(color: Color, target: Color, amount: f32) -> Color {
    let base = Color::new(color.r, color.g, color.b, 255);
    let target = Color::new(target.r, target.g, target.b, 255);
    Color::lerp(base, target, amount).with_alpha(color.a)
}

fn queue_theme_backdrop(_main_world: &MainWorld, render_world: &mut RenderWorld) {
    let backdrops: alloc::vec::Vec<ExtractedThemeBackdrop> = render_world
        .query::<ExtractedThemeBackdrop>()
        .map(|(_, backdrop)| backdrop.clone())
        .collect();
    let gpu_textures = render_world.get_resource::<GpuTextures>().cloned();

    for backdrop in backdrops {
        let texture_id = gpu_textures
            .as_ref()
            .and_then(|textures| textures.get(backdrop.image.id()))
            .map(|texture| texture.id);

        if let Some(texture_id) = texture_id {
            render_world.add_phase_item(
                RenderPhaseType::Ui,
                PhaseItem::ui(
                    RenderCommand::draw_image_full(
                        Rect::from_center_size(
                            backdrop.position,
                            fhre::Vec2::new(backdrop.width, backdrop.height),
                        ),
                        texture_id,
                        Color::WHITE,
                    ),
                    sort_keys::BACKDROP,
                ),
            );
        } else {
            let metrics = ShellMetrics::new(fhre::Vec2::new(backdrop.width, backdrop.height));
            match backdrop.variant {
                ThemeVariant::Aurora => {
                    queue_aurora_backdrop(&metrics, backdrop.palette, false, render_world)
                }
                ThemeVariant::Dusk => {
                    queue_dusk_backdrop(&metrics, backdrop.palette, false, render_world)
                }
            }
        }
    }
}

fn queue_aurora_backdrop(
    metrics: &ShellMetrics,
    palette: crate::ThemePalette,
    home_focus_visible: bool,
    render_world: &mut RenderWorld,
) {
    let w = metrics.width();
    let h = metrics.height();
    let rect = Rect::new(0.0, 0.0, w, h);

    render_world.add_phase_item(
        RenderPhaseType::Ui,
        PhaseItem::ui(
            RenderCommand::DrawRectGradient {
                rect,
                gradient: Gradient::vertical(&[
                    Color::rgb(5, 16, 38),
                    palette.background,
                    Color::rgb(27, 78, 120),
                    Color::rgb(8, 27, 58),
                ]),
            },
            sort_keys::BACKDROP,
        ),
    );

    queue_polygon(
        render_world,
        &[
            (0.12, 0.12),
            (0.25, 0.16),
            (0.40, 0.74),
            (0.31, 0.80),
            (0.18, 0.34),
        ],
        w,
        h,
        Color::new(110, 244, 210, 86),
        sort_keys::BACKDROP_DETAIL + 10,
    );
    queue_polygon(
        render_world,
        &[
            (0.28, 0.18),
            (0.40, 0.13),
            (0.62, 0.76),
            (0.53, 0.82),
            (0.36, 0.40),
        ],
        w,
        h,
        Color::new(176, 104, 255, 72),
        sort_keys::BACKDROP_DETAIL + 11,
    );
    queue_polygon(
        render_world,
        &[
            (0.56, 0.16),
            (0.74, 0.10),
            (0.88, 0.70),
            (0.75, 0.78),
            (0.62, 0.35),
        ],
        w,
        h,
        Color::new(73, 220, 245, 68),
        sort_keys::BACKDROP_DETAIL + 12,
    );

    queue_stars(render_world, w, h, Color::new(236, 248, 255, 125), sort_keys::BACKDROP_DETAIL + 20);
    queue_mountains(render_world, w, h, Color::new(7, 22, 44, 235), Color::new(22, 54, 86, 190));
    queue_reflection(render_world, w, h, Color::new(88, 178, 216, 48), Color::new(148, 94, 230, 36));

    if home_focus_visible {
        queue_home_focus(render_world, metrics, palette, Color::new(180, 232, 255, 70));
    }
}

fn queue_dusk_backdrop(
    metrics: &ShellMetrics,
    palette: crate::ThemePalette,
    home_focus_visible: bool,
    render_world: &mut RenderWorld,
) {
    let w = metrics.width();
    let h = metrics.height();
    let rect = Rect::new(0.0, 0.0, w, h);

    render_world.add_phase_item(
        RenderPhaseType::Ui,
        PhaseItem::ui(
            RenderCommand::DrawRectGradient {
                rect,
                gradient: Gradient::vertical(&[
                    Color::rgb(5, 6, 16),
                    palette.background,
                    Color::rgb(48, 28, 76),
                    Color::rgb(142, 82, 134),
                    Color::rgb(17, 16, 28),
                ]),
            },
            sort_keys::BACKDROP,
        ),
    );

    queue_circle(
        render_world,
        fhre::Vec2::new(w * 0.32, h * 0.21),
        w.min(h) * 0.055,
        Color::new(120, 94, 178, 88),
        sort_keys::BACKDROP_DETAIL + 4,
    );
    queue_circle(
        render_world,
        fhre::Vec2::new(w * 0.71, h * 0.30),
        w.min(h) * 0.12,
        Color::new(190, 145, 255, 42),
        sort_keys::BACKDROP_DETAIL + 5,
    );
    queue_polygon(
        render_world,
        &[
            (0.00, 0.70),
            (0.22, 0.58),
            (0.40, 0.66),
            (0.58, 0.54),
            (0.82, 0.62),
            (1.00, 0.57),
            (1.00, 1.00),
            (0.00, 1.00),
        ],
        w,
        h,
        Color::new(38, 23, 56, 220),
        sort_keys::BACKDROP_DETAIL + 30,
    );
    queue_mountains(render_world, w, h, Color::new(14, 12, 28, 238), Color::new(70, 48, 90, 190));
    queue_reflection(render_world, w, h, Color::new(180, 112, 210, 42), Color::new(84, 58, 130, 40));
    queue_stars(render_world, w, h, Color::new(245, 236, 255, 96), sort_keys::BACKDROP_DETAIL + 22);

    if home_focus_visible {
        queue_home_focus(render_world, metrics, palette, Color::new(214, 174, 255, 72));
    }
}

fn queue_home_focus(
    render_world: &mut RenderWorld,
    metrics: &ShellMetrics,
    palette: crate::ThemePalette,
    halo: Color,
) {
    let center = fhre::Vec2::new(metrics.center_x(), metrics.height() * 0.66);
    let diameter = metrics.width().min(metrics.height()) * 0.30;

    queue_circle(render_world, center, diameter, halo, sort_keys::BACKDROP_DETAIL + 60);
    queue_circle(
        render_world,
        center,
        diameter * 0.62,
        Color::new(palette.background.r, palette.background.g, palette.background.b, 128),
        sort_keys::BACKDROP_DETAIL + 61,
    );

    let wing_color = Color::new(palette.surface.r, palette.surface.g, palette.surface.b, 210);
    let y = center.y;
    let x = center.x;
    let s = diameter * 0.20;
    for (idx, points) in [
        [
            fhre::Vec2::new(x - s * 0.2, y + s * 0.2),
            fhre::Vec2::new(x - s * 1.1, y - s * 0.3),
            fhre::Vec2::new(x - s * 1.5, y - s * 0.9),
        ],
        [
            fhre::Vec2::new(x - s * 0.1, y + s * 0.3),
            fhre::Vec2::new(x - s * 0.9, y + s * 0.0),
            fhre::Vec2::new(x - s * 1.3, y - s * 0.5),
        ],
        [
            fhre::Vec2::new(x + s * 0.2, y + s * 0.2),
            fhre::Vec2::new(x + s * 1.1, y - s * 0.3),
            fhre::Vec2::new(x + s * 1.5, y - s * 0.9),
        ],
        [
            fhre::Vec2::new(x + s * 0.1, y + s * 0.3),
            fhre::Vec2::new(x + s * 0.9, y + s * 0.0),
            fhre::Vec2::new(x + s * 1.3, y - s * 0.5),
        ],
    ]
    .iter()
    .enumerate()
    {
        render_world.add_phase_item(
            RenderPhaseType::Ui,
            PhaseItem::ui(
                RenderCommand::DrawLine {
                    start: points[0],
                    end: points[1],
                    color: wing_color,
                    thickness: 2.0,
                },
                sort_keys::BACKDROP_DETAIL + 70 + idx as i32,
            ),
        );
        render_world.add_phase_item(
            RenderPhaseType::Ui,
            PhaseItem::ui(
                RenderCommand::DrawLine {
                    start: points[1],
                    end: points[2],
                    color: wing_color,
                    thickness: 2.0,
                },
                sort_keys::BACKDROP_DETAIL + 78 + idx as i32,
            ),
        );
    }

    let dot_y = metrics.height() * 0.92;
    let dot_x = metrics.center_x() - 12.0;
    for i in 0..3 {
        queue_circle(
            render_world,
            fhre::Vec2::new(dot_x + i as f32 * 12.0, dot_y),
            4.0,
            if i == 0 {
                Color::new(palette.surface.r, palette.surface.g, palette.surface.b, 210)
            } else {
                Color::new(palette.surface.r, palette.surface.g, palette.surface.b, 88)
            },
            sort_keys::BACKDROP_DETAIL + 90 + i,
        );
    }
}

fn queue_mountains(
    render_world: &mut RenderWorld,
    w: f32,
    h: f32,
    near: Color,
    far: Color,
) {
    let horizon = h * 0.74;
    for (idx, points) in [
        [(0.00, 0.84), (0.18, 0.63), (0.40, 0.84)],
        [(0.20, 0.84), (0.48, 0.61), (0.73, 0.84)],
        [(0.54, 0.84), (0.84, 0.62), (1.08, 0.84)],
    ]
    .iter()
    .enumerate()
    {
        let color = if idx == 1 { near } else { far };
        render_world.add_phase_item(
            RenderPhaseType::Ui,
            PhaseItem::ui(
                RenderCommand::DrawTriangle {
                    p0: scaled(points[0], w, h),
                    p1: scaled(points[1], w, h),
                    p2: scaled(points[2], w, h),
                    color,
                },
                sort_keys::BACKDROP_DETAIL + 38 + idx as i32,
            ),
        );
    }

    render_world.add_phase_item(
        RenderPhaseType::Ui,
        PhaseItem::ui(
            RenderCommand::DrawRectGradient {
                rect: Rect::new(0.0, horizon, w, h - horizon),
                gradient: Gradient::vertical(&[Color::new(10, 18, 36, 70), Color::new(8, 14, 28, 185)]),
            },
            sort_keys::BACKDROP_DETAIL + 45,
        ),
    );
}

fn queue_reflection(
    render_world: &mut RenderWorld,
    w: f32,
    h: f32,
    left: Color,
    right: Color,
) {
    let water_top = h * 0.76;
    render_world.add_phase_item(
        RenderPhaseType::Ui,
        PhaseItem::ui(
            RenderCommand::DrawRectGradient {
                rect: Rect::new(0.0, water_top, w, h - water_top),
                gradient: Gradient::horizontal(&[left, right, left]),
            },
            sort_keys::BACKDROP_DETAIL + 50,
        ),
    );

    for i in 0..5 {
        let y = water_top + 18.0 + i as f32 * 22.0;
        render_world.add_phase_item(
            RenderPhaseType::Ui,
            PhaseItem::ui(
                RenderCommand::DrawLine {
                    start: fhre::Vec2::new(w * (0.12 + i as f32 * 0.03), y),
                    end: fhre::Vec2::new(w * (0.88 - i as f32 * 0.02), y),
                    color: Color::new(240, 248, 255, 24),
                    thickness: 1.0,
                },
                sort_keys::BACKDROP_DETAIL + 52 + i,
            ),
        );
    }
}

fn queue_stars(
    render_world: &mut RenderWorld,
    w: f32,
    h: f32,
    color: Color,
    base_sort: i32,
) {
    const STARS: [(f32, f32, f32); 16] = [
        (0.16, 0.12, 1.8),
        (0.30, 0.18, 1.2),
        (0.46, 0.11, 1.6),
        (0.62, 0.19, 1.1),
        (0.82, 0.15, 1.5),
        (0.22, 0.28, 1.0),
        (0.38, 0.31, 1.4),
        (0.70, 0.30, 1.2),
        (0.90, 0.26, 1.0),
        (0.12, 0.43, 1.0),
        (0.52, 0.42, 1.3),
        (0.78, 0.44, 1.1),
        (0.26, 0.52, 1.0),
        (0.66, 0.55, 1.2),
        (0.86, 0.51, 1.0),
        (0.44, 0.62, 1.0),
    ];

    for (idx, (x, y, size)) in STARS.iter().copied().enumerate() {
        queue_circle(
            render_world,
            fhre::Vec2::new(w * x, h * y),
            size,
            color,
            base_sort + idx as i32,
        );
    }
}

fn queue_polygon(
    render_world: &mut RenderWorld,
    points: &[(f32, f32)],
    w: f32,
    h: f32,
    color: Color,
    sort_key: i32,
) {
    let vertices = points
        .iter()
        .map(|point| scaled(*point, w, h))
        .collect();
    render_world.add_phase_item(
        RenderPhaseType::Ui,
        PhaseItem::ui(RenderCommand::DrawPolygon { vertices, color }, sort_key),
    );
}

fn queue_circle(
    render_world: &mut RenderWorld,
    center: fhre::Vec2,
    diameter: f32,
    color: Color,
    sort_key: i32,
) {
    let rect = Rect::from_center_size(center, fhre::Vec2::new(diameter, diameter));
    render_world.add_phase_item(
        RenderPhaseType::Ui,
        PhaseItem::ui(
            RenderCommand::DrawRectRounded {
                rect,
                color,
                radius: diameter * 0.5,
            },
            sort_key,
        ),
    );
}

fn scaled(point: (f32, f32), w: f32, h: f32) -> fhre::Vec2 {
    fhre::Vec2::new(point.0 * w, point.1 * h)
}

fn queue_preview_soccer_ball(main_world: &MainWorld, render_world: &mut RenderWorld) {
    let app_switcher_open = main_world
        .resources()
        .get::<ShellState>()
        .map(|state| state.app_switcher_open())
        .unwrap_or(false);
    let app_progress = main_world
        .resources()
        .get::<ShellOverlayAnimation>()
        .map(|animation| animation.app_switcher_eased())
        .unwrap_or(if app_switcher_open { 1.0 } else { 0.0 });
    let card_alpha = main_world
        .resources()
        .get::<ShellOverlayAnimation>()
        .map(|animation| animation.card_alpha_eased())
        .unwrap_or(if app_switcher_open { 1.0 } else { 0.0 });
    let Some(content) = main_world.resources().get::<ShellContent>() else {
        return;
    };
    let Some(metrics) = main_world.resources().get::<ShellMetrics>() else {
        return;
    };

    if content.preview_effect != PreviewEffect::Soccer {
        return;
    }

    if (app_progress <= 0.001 && !app_switcher_open) || content.surfaces.is_empty() {
        return;
    }

    let alpha = (230.0 * card_alpha.clamp(0.0, 1.0)) as u8;
    if alpha == 0 {
        return;
    }

    let screen_size = fhre::Vec2::new(metrics.width(), metrics.height());
    let center = app_switcher_ball_center(screen_size, app_progress);
    let base_size = app_switcher_ball_size(screen_size, app_progress);

    for (_, ball) in main_world.query::<PreviewSoccerBall>() {
        let faces = ball.projected_faces(center, base_size, alpha);
        if faces.is_empty() {
            continue;
        }

        for (face_idx, face) in faces.iter().enumerate() {
            let color = surface_index_for_face(face.face_index, content.surfaces.len())
                .map(|surface_index| preview_surface_color(surface_index, alpha))
                .unwrap_or(face.color);
            let command = RenderCommand::DrawPolygon {
                vertices: face.vertices.clone(),
                color,
            };
            render_world.add_phase_item(
                RenderPhaseType::Ui,
                PhaseItem::ui(command, sort_keys::PREVIEW_BALL + face_idx as i32),
            );
        }

        if ball.wireframe {
            let mut line_idx = 0;
            let wire_alpha = (alpha as u16 * ball.wireframe_color.a as u16 / 255) as u8;
            let wire_color = fhre::Color::new(
                ball.wireframe_color.r,
                ball.wireframe_color.g,
                ball.wireframe_color.b,
                wire_alpha,
            );

            for face in &faces {
                if face.vertices.len() < 2 {
                    continue;
                }

                for i in 0..face.vertices.len() {
                    let command = RenderCommand::DrawLine {
                        start: face.vertices[i],
                        end: face.vertices[(i + 1) % face.vertices.len()],
                        color: wire_color,
                        thickness: 1.0,
                    };
                    render_world.add_phase_item(
                        RenderPhaseType::Ui,
                        PhaseItem::ui(command, sort_keys::PREVIEW_WIREFRAME + line_idx),
                    );
                    line_idx += 1;
                }
            }
        }

        for face in faces {
            let Some(surface_index) = surface_index_for_face(face.face_index, content.surfaces.len()) else {
                continue;
            };
            let surface = content.surfaces[surface_index];
            let label_position = fhre::Vec2::new(face.center.x - 22.0, face.center.y - 6.0);
            let command = RenderCommand::draw_text(
                label_position,
                surface.label,
                fhre::Color::new(20, 26, 36, alpha),
                10.0,
            );
            render_world.add_phase_item(
                RenderPhaseType::Ui,
                PhaseItem::ui(command, sort_keys::PREVIEW_LABEL + surface_index as i32),
            );
        }

        break;
    }
}

fn preview_surface_color(surface_index: usize, alpha: u8) -> fhre::Color {
    const COLORS: [fhre::Color; 8] = [
        fhre::Color::rgb(148, 197, 255),
        fhre::Color::rgb(160, 232, 190),
        fhre::Color::rgb(255, 214, 128),
        fhre::Color::rgb(245, 166, 180),
        fhre::Color::rgb(193, 174, 255),
        fhre::Color::rgb(134, 224, 224),
        fhre::Color::rgb(242, 190, 142),
        fhre::Color::rgb(210, 224, 148),
    ];

    let color = COLORS[surface_index % COLORS.len()];
    fhre::Color::new(color.r, color.g, color.b, alpha)
}
