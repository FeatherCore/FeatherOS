use crate::{
    spec::{UiKind, UiSpec},
    tree::UiTree,
};
use fhre::{
    fixed_to_i32, Camera, Color, DrawCommand, DrawList, Entity, Point, Rect, Surface, Tween,
};

impl<const N: usize, const DIRTY: usize> UiTree<N, DIRTY> {
    pub fn render(&self, surface: &mut Surface, _camera: &Camera) {
        let mut list: DrawList<N> = DrawList::new();
        self.nodes.for_each(|entity, _| {
            if let Some(command) = self.command_for(entity) {
                list.push_clipped(command, self.clip_for(entity));
            }
        });
        list.sort_by_depth();
        list.execute(surface);
    }

    pub fn hit_test(&self, point: Point, _camera: &Camera) -> Option<crate::node::UiHit> {
        let mut best = None;
        let mut best_depth = i32::MIN;
        self.nodes.for_each(|entity, _| {
            if let (Some(node), Some(button), Some(command)) = (
                self.nodes.get(entity),
                self.buttons.get(entity),
                self.command_for(entity),
            ) {
                if let Some(rect) = command.clipped_bounds(self.clip_for(entity)) {
                    let depth = command.depth();
                    if rect.contains_point(point) && depth >= best_depth {
                        best = Some(crate::node::UiHit {
                            key: node.key,
                            action: button.action,
                            rect,
                        });
                        best_depth = depth;
                    }
                }
            }
        });
        best
    }

    pub(crate) fn find(&self, key: crate::key::UiKey) -> Option<Entity> {
        let mut found = None;
        self.nodes.for_each(|entity, node| {
            if found.is_none() && node.key == key {
                found = Some(entity);
            }
        });
        found
    }

    pub(crate) fn spec_for(&self, entity: Entity) -> Option<UiSpec> {
        let node = *self.nodes.get(entity)?;
        let parent = self.parents.get(entity).map(|parent| parent.key);
        let layout = *self.layouts.get(entity)?;
        let visual = *self.visuals.get(entity)?;
        let opacity = self.opacities.get(entity).map(|opacity| opacity.value).unwrap_or(255);
        let clip = self.clips.get(entity).map(|clip| clip.rect);
        let text = self.texts.get(entity).map(|text| text.text);
        let icon = self.icons.get(entity).map(|icon| icon.icon);
        let image_node = self.images.get(entity).copied();
        let image = image_node.map(|image| image.image);
        let image_fit = image_node.map(|image| image.fit).unwrap_or(fhre::ImageFit::Stretch);
        let animation = self.animations.get(entity).copied();
        let action = self.buttons.get(entity).map(|button| button.action);

        Some(UiSpec {
            key: node.key,
            parent,
            space: layout.space,
            content_inset: layout.content_inset,
            layout: layout.rule,
            kind: node.kind,
            transform: layout.transform,
            size: layout.size,
            color: visual.color,
            opacity,
            clip,
            radius: visual.radius,
            scale: visual.scale,
            line_to: layout.line_to,
            text,
            icon,
            image,
            image_fit,
            animation,
            action,
        })
    }

    pub(crate) fn command_for(&self, entity: Entity) -> Option<DrawCommand> {
        let node = *self.nodes.get(entity)?;
        let result = *self.results.get(entity)?;
        let visual = *self.visuals.get(entity)?;
        let opacity = self.animated_opacity(entity);
        let color = apply_opacity(visual.color, opacity);

        match node.kind {
            UiKind::Panel | UiKind::Tile => Some(DrawCommand::FillRoundRect {
                rect: result.rect,
                depth: result.depth,
                radius: visual.radius,
                color,
            }),
            UiKind::Bar => {
                if visual.radius == 0 {
                    Some(DrawCommand::FillRect {
                        rect: result.rect,
                        depth: result.depth,
                        color,
                    })
                } else {
                    Some(DrawCommand::FillRoundRect {
                        rect: result.rect,
                        depth: result.depth,
                        radius: visual.radius,
                        color,
                    })
                }
            }
            UiKind::Circle => {
                let radius = result.rect.w.min(result.rect.h) / 2;
                Some(DrawCommand::FillCircle {
                    center: Point::new(
                        result.rect.x + result.rect.w as i32 / 2,
                        result.rect.y + result.rect.h as i32 / 2,
                    ),
                    depth: result.depth,
                    radius,
                    color,
                })
            }
            UiKind::Line => Some(DrawCommand::StrokeLine {
                from: Point::new(result.rect.x, result.rect.y),
                to: result.line_to,
                depth: result.depth,
                width: visual.radius.max(1),
                color,
            }),
            UiKind::Text => {
                let text = *self.texts.get(entity)?;
                Some(DrawCommand::DrawText {
                    pos: Point::new(result.rect.x, result.rect.y),
                    depth: result.depth,
                    text: text.text,
                    font: text.font,
                    color,
                    scale: visual.scale,
                })
            }
            UiKind::Icon => {
                let icon = *self.icons.get(entity)?;
                Some(DrawCommand::DrawSvgIcon {
                    rect: result.rect,
                    depth: result.depth,
                    icon: icon.icon,
                    color,
                    opacity: color.a,
                })
            }
            UiKind::Image => {
                let image = *self.images.get(entity)?;
                if visual.color != Color::WHITE {
                    Some(DrawCommand::DrawImageTint {
                        rect: result.rect,
                        depth: result.depth,
                        image: image.image,
                        opacity,
                        fit: image.fit,
                        tint: visual.color,
                    })
                } else if image.fit != fhre::ImageFit::Stretch {
                    Some(DrawCommand::DrawImageFit {
                        rect: result.rect,
                        depth: result.depth,
                        image: image.image,
                        opacity,
                        fit: image.fit,
                    })
                } else {
                    Some(DrawCommand::DrawImage {
                        rect: result.rect,
                        depth: result.depth,
                        image: image.image,
                        opacity,
                    })
                }
            }
        }
    }

    pub(crate) fn clip_for(&self, entity: Entity) -> Option<Rect> {
        self.resolved_clips.get(entity).map(|clip| clip.rect)
    }

    fn animated_opacity(&self, entity: Entity) -> u8 {
        let base = self.opacities.get(entity).map(|opacity| opacity.value).unwrap_or(255);
        let animated = self
            .animations
            .get(entity)
            .and_then(|animation| animation.opacity)
            .map(|tween| tween_u8(tween, self.animation_elapsed_us()))
            .unwrap_or(255);

        ((base as u16 * animated as u16) / 255) as u8
    }

    fn animation_elapsed_us(&self) -> u64 {
        self.frame as u64 * 16_666
    }
}

fn apply_opacity(color: Color, opacity: u8) -> Color {
    Color::rgba(
        color.r,
        color.g,
        color.b,
        ((color.a as u16 * opacity as u16) / 255) as u8,
    )
}

fn tween_u8(tween: Tween, elapsed_us: u64) -> u8 {
    let value = fixed_to_i32(tween.sample(elapsed_us));
    if value <= 0 {
        0
    } else if value >= 255 {
        255
    } else {
        value as u8
    }
}
