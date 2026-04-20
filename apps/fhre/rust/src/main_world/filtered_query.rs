//! Query Implementation
//!
//! Unified Query type that supports both single and multi-component queries.
//! Uses QueryData trait for flexibility: `Query<&Transform>`, `Query<(&A, &mut B)>`, etc.

use super::entity::Entity;
use super::world::MainWorld;
use super::query_data::QueryData;
use super::query_filter::QueryFilter;
use super::system_param::SystemParam;
use alloc::vec::Vec;
use core::marker::PhantomData;

pub struct Query<'w, 's, D: QueryData, F: QueryFilter = ()> {
    world: &'w mut MainWorld,
    entities: Vec<Entity>,
    _marker: PhantomData<(&'s (), D, F)>,
}

impl<'w, 's, D: QueryData, F: QueryFilter> Query<'w, 's, D, F> {
    pub fn new(world: &'w mut MainWorld) -> Self {
        let entities: Vec<Entity> = {
            let mut entities = Vec::new();

            let entity_ids: Vec<u64> = world.entities()
                .iter()
                .map(|e| e.id())
                .collect();

            for id in entity_ids {
                let entity = Entity::new(id);

                if D::matches(world, entity) {
                    if F::matches(entity, world) {
                        entities.push(entity);
                    }
                }
            }

            entities
        };

        Self {
            world,
            entities,
            _marker: PhantomData,
        }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = (Entity, D::Item<'w>)> + '_ {
        self.entities.iter().filter_map(|&entity| {
            unsafe {
                let world_ptr = self.world as *mut MainWorld;
                D::fetch(&mut *world_ptr, entity).map(|item| (entity, item))
            }
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, D::Item<'w>)> + '_ {
        self.iter()
    }

    pub fn single(&mut self) -> Option<D::Item<'w>> {
        self.entities.first().and_then(|&entity| {
            unsafe {
                let world_ptr = self.world as *mut MainWorld;
                D::fetch(&mut *world_ptr, entity)
            }
        })
    }

    pub fn single_mut(&mut self) -> Option<D::Item<'w>> {
        self.single()
    }

    pub fn single_entity(&self) -> Option<Entity> {
        self.entities.first().copied()
    }

    pub fn single_pair(&mut self) -> Option<(Entity, D::Item<'w>)> {
        self.entities.first().and_then(|&entity| {
            unsafe {
                let world_ptr = self.world as *mut MainWorld;
                D::fetch(&mut *world_ptr, entity).map(|item| (entity, item))
            }
        })
    }

    pub fn single_pair_mut(&mut self) -> Option<(Entity, D::Item<'w>)> {
        self.single_pair()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn get(&mut self, entity: Entity) -> Option<D::Item<'w>> {
        self.entities.iter()
            .find(|&e| e.id() == entity.id())
            .and_then(|&entity| {
                unsafe {
                    let world_ptr = self.world as *mut MainWorld;
                    D::fetch(&mut *world_ptr, entity)
                }
            })
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<D::Item<'w>> {
        self.get(entity)
    }

    pub fn get_pair(&mut self, entity: Entity) -> Option<(Entity, D::Item<'w>)> {
        self.entities.iter()
            .find(|&e| e.id() == entity.id())
            .and_then(|&entity| {
                unsafe {
                    let world_ptr = self.world as *mut MainWorld;
                    D::fetch(&mut *world_ptr, entity).map(|item| (entity, item))
                }
            })
    }

    pub fn get_pair_mut(&mut self, entity: Entity) -> Option<(Entity, D::Item<'w>)> {
        self.get_pair(entity)
    }
}

pub struct QueryState<D: QueryData, F: QueryFilter> {
    _marker: PhantomData<(D, F)>,
}

impl<D: QueryData, F: QueryFilter> Default for QueryState<D, F> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<D: QueryData, F: QueryFilter> SystemParam for Query<'_, '_, D, F> {
    type Item<'w, 's> = Query<'w, 's, D, F>;
    type State = QueryState<D, F>;

    unsafe fn get_param<'w, 's>(
        _state: &'s mut Self::State,
        world: &'w mut MainWorld,
    ) -> Self::Item<'w, 's> {
        Query::new(world)
    }
}

pub type FilteredQuery<'w, 's, T, F = ()> = Query<'w, 's, T, F>;
