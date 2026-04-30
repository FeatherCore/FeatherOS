use crate::{
    animation::Animation,
    builder::UiBuilder,
    key::UiKey,
    node::{
        Button, Children, Clip, IconNode, ImageNode, LayoutBox, LayoutResult, Opacity, Parent,
        ResolvedClip, TextNode, UiFrameStats, UiNode, Visual,
    },
    spec::UiSpec,
    tree_layout::project_clip_rect,
};
use fhre::{
    Camera, ComponentStorage, DirtyRegion, Entity, EntityWorld, FontId, Rect,
};

pub struct UiTree<const N: usize, const DIRTY: usize> {
    pub(crate) entities: EntityWorld<N>,
    pub(crate) nodes: ComponentStorage<UiNode, N>,
    pub(crate) parents: ComponentStorage<Parent, N>,
    pub(crate) children: ComponentStorage<Children, N>,
    pub(crate) layouts: ComponentStorage<LayoutBox, N>,
    pub(crate) results: ComponentStorage<LayoutResult, N>,
    pub(crate) visuals: ComponentStorage<Visual, N>,
    pub(crate) opacities: ComponentStorage<Opacity, N>,
    pub(crate) clips: ComponentStorage<Clip, N>,
    pub(crate) resolved_clips: ComponentStorage<ResolvedClip, N>,
    pub(crate) texts: ComponentStorage<TextNode, N>,
    pub(crate) icons: ComponentStorage<IconNode, N>,
    pub(crate) images: ComponentStorage<ImageNode, N>,
    pub(crate) animations: ComponentStorage<Animation, N>,
    pub(crate) buttons: ComponentStorage<Button, N>,
    pub(crate) overflowed: bool,
    pub(crate) frame: u32,
    pub(crate) dirty: DirtyRegion<DIRTY>,
}

impl<const N: usize, const DIRTY: usize> UiTree<N, DIRTY> {
    pub const fn new() -> Self {
        Self {
            entities: EntityWorld::new(),
            nodes: ComponentStorage::new(),
            parents: ComponentStorage::new(),
            children: ComponentStorage::new(),
            layouts: ComponentStorage::new(),
            results: ComponentStorage::new(),
            visuals: ComponentStorage::new(),
            opacities: ComponentStorage::new(),
            clips: ComponentStorage::new(),
            resolved_clips: ComponentStorage::new(),
            texts: ComponentStorage::new(),
            icons: ComponentStorage::new(),
            images: ComponentStorage::new(),
            animations: ComponentStorage::new(),
            buttons: ComponentStorage::new(),
            overflowed: false,
            frame: 0,
            dirty: DirtyRegion::new(),
        }
    }

    pub const fn len(&self) -> usize {
        self.nodes.len()
    }

    pub const fn overflowed(&self) -> bool {
        self.overflowed
    }

    pub const fn frame(&self) -> u32 {
        self.frame
    }

    pub const fn dirty(&self) -> &DirtyRegion<DIRTY> {
        &self.dirty
    }

    pub fn clear(&mut self) {
        self.entities = EntityWorld::new();
        self.nodes.clear();
        self.parents.clear();
        self.children.clear();
        self.layouts.clear();
        self.results.clear();
        self.visuals.clear();
        self.opacities.clear();
        self.clips.clear();
        self.resolved_clips.clear();
        self.texts.clear();
        self.icons.clear();
        self.images.clear();
        self.animations.clear();
        self.buttons.clear();
        self.overflowed = false;
        self.frame = 0;
        self.dirty.clear();
    }

    pub fn apply<const M: usize>(&mut self, builder: &UiBuilder<M>, camera: &Camera) -> UiFrameStats {
        let screen = camera.viewport;
        self.frame = self.frame.wrapping_add(1);
        self.overflowed = builder.overflowed();
        self.dirty.clear();

        self.nodes.for_each_mut(|_, node| {
            node.seen = false;
        });

        let mut created = 0;
        let mut updated = 0;
        let mut index = 0;
        while index < builder.len {
            let spec = builder.specs[index];
            if let Some(entity) = self.find(spec.key) {
                let old = self.spec_for(entity).unwrap_or(UiSpec::EMPTY);
                if old != spec {
                    self.mark_entity_dirty(entity, screen);
                    if !self.write_spec(entity, spec, camera) {
                        self.overflowed = true;
                    } else {
                        self.mark_entity_dirty(entity, screen);
                        updated += 1;
                    }
                } else if let Some(node) = self.nodes.get_mut(entity) {
                    node.seen = true;
                }
            } else {
                match self.entities.spawn() {
                    Some(entity) => {
                        if self.write_spec(entity, spec, camera) {
                            self.mark_entity_dirty(entity, screen);
                            created += 1;
                        } else {
                            self.remove_entity(entity);
                            self.overflowed = true;
                        }
                    }
                    None => {
                        self.overflowed = true;
                    }
                }
            }
            index += 1;
        }

        let mut removed = 0;
        let mut remove_entities: [Option<Entity>; N] = [None; N];
        let mut remove_count = 0;
        self.nodes.for_each(|entity, node| {
            if !node.seen && remove_count < N {
                remove_entities[remove_count] = Some(entity);
                remove_count += 1;
            }
        });

        let mut remove_index = 0;
        while remove_index < remove_count {
            if let Some(entity) = remove_entities[remove_index] {
                self.mark_entity_dirty(entity, screen);
                self.remove_entity(entity);
            }
            removed += 1;
            remove_index += 1;
        }

        self.rebuild_hierarchy();
        self.resolve_layouts(camera, screen);
        self.resolve_clips(screen);
        self.mark_animated_dirty(screen);

        UiFrameStats {
            node_count: self.nodes.len(),
            created,
            updated,
            removed,
            dirty_count: self.dirty.len(),
            overflowed: self.overflowed || self.dirty.overflowed(),
        }
    }

    fn write_spec(&mut self, entity: Entity, spec: UiSpec, camera: &Camera) -> bool {
        let mut ok = true;
        ok &= self.nodes.insert(
            entity,
            UiNode {
                key: spec.key,
                kind: spec.kind,
                seen: true,
            },
        );
        match spec.parent {
            Some(parent) => {
                ok &= self.parents.insert(
                    entity,
                    Parent {
                        key: parent,
                        entity: None,
                    },
                );
            }
            None => {
                self.parents.remove(entity);
            }
        }
        ok &= self.layouts.insert(
            entity,
            LayoutBox {
                space: spec.space,
                content_inset: spec.content_inset,
                rule: spec.layout,
                transform: spec.transform,
                size: spec.size,
                line_to: spec.line_to,
            },
        );
        ok &= self.results.insert(entity, LayoutResult::project(spec, camera));
        ok &= self.visuals.insert(
            entity,
            Visual {
                color: spec.color,
                radius: spec.radius,
                scale: spec.scale,
            },
        );
        ok &= self.opacities.insert(entity, Opacity { value: spec.opacity });

        match spec.clip {
            Some(rect) => {
                ok &= self.clips.insert(
                    entity,
                    Clip {
                        rect,
                        projected: project_clip_rect(rect, camera),
                    },
                );
            }
            None => {
                self.clips.remove(entity);
            }
        }

        match spec.text {
            Some(text) => {
                ok &= self.texts.insert(
                    entity,
                    TextNode {
                        text,
                        font: FontId(0),
                    },
                );
            }
            None => {
                self.texts.remove(entity);
            }
        }

        match spec.icon {
            Some(icon) => {
                ok &= self.icons.insert(entity, IconNode { icon });
            }
            None => {
                self.icons.remove(entity);
            }
        }

        match spec.image {
            Some(image) => {
                ok &= self.images.insert(
                    entity,
                    ImageNode {
                        image,
                        fit: spec.image_fit,
                    },
                );
            }
            None => {
                self.images.remove(entity);
            }
        }

        match spec.animation {
            Some(animation) => {
                ok &= self.animations.insert(entity, animation);
            }
            None => {
                self.animations.remove(entity);
            }
        }

        match spec.action {
            Some(action) => {
                ok &= self.buttons.insert(entity, Button { action });
            }
            None => {
                self.buttons.remove(entity);
            }
        }

        if !ok {
            self.nodes.remove(entity);
            self.parents.remove(entity);
            self.children.remove(entity);
            self.layouts.remove(entity);
            self.results.remove(entity);
            self.visuals.remove(entity);
            self.opacities.remove(entity);
            self.clips.remove(entity);
            self.resolved_clips.remove(entity);
            self.texts.remove(entity);
            self.icons.remove(entity);
            self.images.remove(entity);
            self.animations.remove(entity);
            self.buttons.remove(entity);
        }

        ok
    }

    fn remove_entity(&mut self, entity: Entity) {
        self.nodes.remove(entity);
        self.parents.remove(entity);
        self.children.remove(entity);
        self.layouts.remove(entity);
        self.results.remove(entity);
        self.visuals.remove(entity);
        self.opacities.remove(entity);
        self.clips.remove(entity);
        self.resolved_clips.remove(entity);
        self.texts.remove(entity);
        self.icons.remove(entity);
        self.images.remove(entity);
        self.animations.remove(entity);
        self.buttons.remove(entity);
        self.entities.despawn(entity);
    }

    fn rebuild_hierarchy(&mut self) {
        self.children.clear();
        self.parents.for_each_mut(|_, parent| {
            parent.entity = None;
        });

        let mut all_entities: [Option<Entity>; N] = [None; N];
        let mut all_count = 0;
        self.nodes.for_each(|entity, _| {
            if all_count < N {
                all_entities[all_count] = Some(entity);
                all_count += 1;
            }
        });

        let mut index = 0;
        while index < all_count {
            if let Some(entity) = all_entities[index] {
                if !self.children.insert(entity, Children::EMPTY) {
                    self.overflowed = true;
                }
            }
            index += 1;
        }

        let mut relations: [Option<(Entity, UiKey)>; N] = [None; N];
        let mut relation_count = 0;
        self.parents.for_each(|entity, parent| {
            if relation_count < N {
                relations[relation_count] = Some((entity, parent.key));
                relation_count += 1;
            }
        });

        let mut relation_index = 0;
        while relation_index < relation_count {
            if let Some((child, parent_key)) = relations[relation_index] {
                let parent_entity = self.find(parent_key);
                if let Some(parent) = self.parents.get_mut(child) {
                    parent.entity = parent_entity;
                }

                if let Some(parent_entity) = parent_entity {
                    let first_child = self
                        .children
                        .get(parent_entity)
                        .map(|children| children.first_child)
                        .unwrap_or(None);

                    if let Some(child_links) = self.children.get_mut(child) {
                        child_links.next_sibling = first_child;
                    }
                    if let Some(parent_links) = self.children.get_mut(parent_entity) {
                        parent_links.first_child = Some(child);
                    }
                }
            }
            relation_index += 1;
        }
    }

    fn mark_animated_dirty(&mut self, screen: Rect) {
        let mut entities: [Option<Entity>; N] = [None; N];
        let mut count = 0;
        self.animations.for_each(|entity, _| {
            if count < N {
                entities[count] = Some(entity);
                count += 1;
            }
        });

        let mut index = 0;
        while index < count {
            if let Some(entity) = entities[index] {
                self.mark_entity_dirty(entity, screen);
            }
            index += 1;
        }
    }

    fn mark_entity_dirty(&mut self, entity: Entity, screen: Rect) {
        if let Some(bounds) = self
            .command_for(entity)
            .and_then(|command| command.clipped_bounds(self.clip_for(entity)))
        {
            self.dirty.push(bounds, screen);
        }
    }
}

pub fn apply_ui_frame<const TREE: usize, const DIRTY: usize, const SPECS: usize>(
    tree: &mut UiTree<TREE, DIRTY>,
    builder: &UiBuilder<SPECS>,
    camera: &Camera,
) -> UiFrameStats {
    tree.apply(builder, camera)
}
