//! Query Filter Implementation
//!
//! Provides filters for Query: With<T>, Without<T>, and Or<...>
//! Aligned with Bevy's query filter system.

use super::component::Component;
use super::entity::Entity;
use super::world::MainWorld;
use core::marker::PhantomData;

pub trait QueryFilter {
    fn matches(entity: Entity, world: &MainWorld) -> bool;
}

pub struct With<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for With<T> {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        world.get_component::<T>(entity).is_some()
    }
}

pub struct Without<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for Without<T> {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        world.get_component::<T>(entity).is_none()
    }
}

pub struct Or<T> {
    _marker: PhantomData<T>,
}

pub struct And<T> {
    _marker: PhantomData<T>,
}

impl QueryFilter for () {
    fn matches(_entity: Entity, _world: &MainWorld) -> bool {
        true
    }
}

macro_rules! impl_tuple_query_filter {
    () => {};
    ($($T:ident),*) => {
        impl<$($T: QueryFilter),*> QueryFilter for ($($T,)*) {
            fn matches(entity: Entity, world: &MainWorld) -> bool {
                true $(&& $T::matches(entity, world))*
            }
        }

        impl<$($T: QueryFilter),*> QueryFilter for Or<($($T,)*)> {
            fn matches(entity: Entity, world: &MainWorld) -> bool {
                false $(|| $T::matches(entity, world))*
            }
        }

        impl<$($T: QueryFilter),*> QueryFilter for And<($($T,)*)> {
            fn matches(entity: Entity, world: &MainWorld) -> bool {
                true $(&& $T::matches(entity, world))*
            }
        }
    };
}

crate::all_tuples!(impl_tuple_query_filter, 0, 15, F);

pub struct Added<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for Added<T> {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        let last_run_tick = world.change_detection().last_run_tick();
        world.change_detection().is_added::<T>(entity, last_run_tick)
    }
}

pub struct Changed<T: Component> {
    _marker: PhantomData<T>,
}

impl<T: Component> QueryFilter for Changed<T> {
    fn matches(entity: Entity, world: &MainWorld) -> bool {
        let last_run_tick = world.change_detection().last_run_tick();
        world.change_detection().is_changed::<T>(entity, last_run_tick)
    }
}
