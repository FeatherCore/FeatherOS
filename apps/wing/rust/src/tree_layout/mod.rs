use crate::{
    layout::{ContentInset, LayoutRule, LayoutSpace},
    node::{LayoutResult, ResolvedClip},
    spec::UiSpec,
    tree::UiTree,
};
use fhre::{
    fixed_from_i32, Camera, Entity, Fixed16, Point, Rect, RenderNode, Size, Transform3D,
};

impl<const N: usize, const DIRTY: usize> UiTree<N, DIRTY> {
    pub(crate) fn resolve_layouts(&mut self, camera: &Camera, screen: Rect) {
        let mut entities: [Option<Entity>; N] = [None; N];
        let mut count = 0;
        self.nodes.for_each(|entity, _| {
            if count < N {
                entities[count] = Some(entity);
                count += 1;
            }
        });

        let mut pass = 0;
        while pass < N {
            let mut changed = false;
            let mut index = 0;
            while index < count {
                if let Some(entity) = entities[index] {
                    if let Some(spec) = self.spec_for(entity) {
                        let old = *self.results.get(entity).unwrap_or(&LayoutResult::EMPTY);
                        let next = self.project_resolved(entity, spec, camera);
                        if old != next {
                            self.dirty.push(old.rect, screen);
                            self.dirty.push(next.rect, screen);
                            if !self.results.insert(entity, next) {
                                self.overflowed = true;
                            }
                            changed = true;
                        }
                    }
                }
                index += 1;
            }

            if !changed {
                break;
            }
            pass += 1;
        }
    }

    pub(crate) fn resolve_clips(&mut self, screen: Rect) {
        let mut entities: [Option<Entity>; N] = [None; N];
        let mut count = 0;
        self.nodes.for_each(|entity, _| {
            if count < N {
                entities[count] = Some(entity);
                count += 1;
            }
        });

        let mut index = 0;
        while index < count {
            if let Some(entity) = entities[index] {
                let old = self.resolved_clips.get(entity).map(|clip| clip.rect);
                let next = self.resolve_effective_clip(entity, 0);

                if old != next {
                    if let Some(command) = self.command_for(entity) {
                        if let Some(rect) = command.clipped_bounds(old) {
                            self.dirty.push(rect, screen);
                        }
                        if let Some(rect) = command.clipped_bounds(next) {
                            self.dirty.push(rect, screen);
                        }
                    }
                }

                match next {
                    Some(rect) => {
                        if !self.resolved_clips.insert(entity, ResolvedClip { rect }) {
                            self.overflowed = true;
                        }
                    }
                    None => {
                        self.resolved_clips.remove(entity);
                    }
                }
            }
            index += 1;
        }
    }

    fn resolve_effective_clip(&self, entity: Entity, depth: usize) -> Option<Rect> {
        if depth >= N {
            return self.explicit_clip_screen(entity);
        }

        let own = self.explicit_clip_screen(entity);
        let inherited = self
            .parents
            .get(entity)
            .and_then(|parent| parent.entity)
            .and_then(|parent_entity| self.resolve_effective_clip(parent_entity, depth + 1));

        intersect_optional_clip(own, inherited)
    }

    fn explicit_clip_screen(&self, entity: Entity) -> Option<Rect> {
        let clip = self.clips.get(entity)?;
        let layout = self.layouts.get(entity)?;

        if matches!(layout.space, LayoutSpace::Parent | LayoutSpace::Content) {
            if let Some((origin, _)) = self.parent_origin_for_space(entity, layout.space) {
                return Some(Rect::new(
                    clip.rect.x.saturating_add(origin.x),
                    clip.rect.y.saturating_add(origin.y),
                    clip.rect.w,
                    clip.rect.h,
                ));
            }
        }

        Some(clip.projected)
    }

    fn project_resolved(&self, entity: Entity, spec: UiSpec, camera: &Camera) -> LayoutResult {
        LayoutResult::project(self.resolve_parent_space(entity, self.resolve_layout_rule(spec)), camera)
    }

    fn resolve_layout_rule(&self, mut spec: UiSpec) -> UiSpec {
        match spec.layout {
            LayoutRule::None => {}
            LayoutRule::GridCell { grid, index } => {
                let rect = grid.cell_rect(index);
                spec.transform.position.x = fixed_from_i32(rect.x);
                spec.transform.position.y = fixed_from_i32(rect.y);
                spec.size = Size::new(rect.w, rect.h);
            }
            LayoutRule::StackItem { stack, index } => {
                let rect = stack.item_rect(index);
                spec.transform.position.x = fixed_from_i32(rect.x);
                spec.transform.position.y = fixed_from_i32(rect.y);
                spec.size = Size::new(rect.w, rect.h);
            }
        }
        spec
    }

    fn resolve_parent_space(&self, entity: Entity, mut spec: UiSpec) -> UiSpec {
        if spec.space == LayoutSpace::Screen {
            return spec;
        }

        if let Some((origin, depth)) = self.parent_origin_for_space(entity, spec.space) {
            spec.transform.position.x = spec
                .transform
                .position
                .x
                .saturating_add(fixed_from_i32(origin.x));
            spec.transform.position.y = spec
                .transform
                .position
                .y
                .saturating_add(fixed_from_i32(origin.y));
            spec.transform.position.z = spec.transform.position.z.saturating_add(depth);
            spec.line_to = Point::new(
                spec.line_to.x.saturating_add(origin.x),
                spec.line_to.y.saturating_add(origin.y),
            );
            spec.clip = spec.clip.map(|clip| {
                Rect::new(
                    clip.x.saturating_add(origin.x),
                    clip.y.saturating_add(origin.y),
                    clip.w,
                    clip.h,
                )
            });
        }

        spec.space = LayoutSpace::Screen;
        spec
    }

    fn parent_origin_for_space(&self, entity: Entity, space: LayoutSpace) -> Option<(Point, Fixed16)> {
        let parent_entity = self.parents.get(entity).and_then(|parent| parent.entity)?;
        let parent_result = self.results.get(parent_entity)?;
        let origin = match space {
            LayoutSpace::Screen => return None,
            LayoutSpace::Parent => Point::new(parent_result.rect.x, parent_result.rect.y),
            LayoutSpace::Content => {
                let inset = self
                    .layouts
                    .get(parent_entity)
                    .map(|layout| layout.content_inset)
                    .unwrap_or(ContentInset::ZERO);
                let content = inset.content_rect(parent_result.rect);
                Point::new(content.x, content.y)
            }
        };

        Some((origin, parent_result.depth))
    }
}

pub(crate) fn project_clip_rect(clip: Rect, camera: &Camera) -> Rect {
    RenderNode::new(Transform3D::screen(clip.x, clip.y, 0), Size::new(clip.w, clip.h))
        .project(camera)
        .rect
}

fn intersect_optional_clip(own: Option<Rect>, inherited: Option<Rect>) -> Option<Rect> {
    match (own, inherited) {
        (Some(own), Some(inherited)) => Some(own.clipped_to(inherited)),
        (Some(own), None) => Some(own),
        (None, Some(inherited)) => Some(inherited),
        (None, None) => None,
    }
}
