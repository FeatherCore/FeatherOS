//! System Implementation
//!
//! Provides the System trait and related types for ECS.
//!
//! Systems are registered via `app.add_systems(Update, system2::<A, B, _>(my_func))`.
//! The `system1`/`system2`/`system3`/`system4` functions wrap user functions
//! into type-erased `Box<dyn System>` objects.

use super::world::MainWorld;
use super::system_param::SystemParam;
use alloc::boxed::Box;

/// Trait for systems that can be run
pub trait System {
    fn run(&mut self, world: &mut MainWorld);
}

/// Trait for types that can be converted into a System
pub trait IntoSystem {
    type System: System;
    fn into_system(self) -> Self::System;
}

impl System for Box<dyn FnMut(&mut MainWorld)> {
    fn run(&mut self, world: &mut MainWorld) {
        self(world);
    }
}

impl<F: FnMut(&mut MainWorld) + 'static> IntoSystem for F {
    type System = Box<dyn FnMut(&mut MainWorld)>;

    fn into_system(self) -> Self::System {
        Box::new(self)
    }
}

macro_rules! impl_system {
    (
        struct_name = $struct_name:ident,
        fn_name = $fn_name:ident,
        params = [$($param:ident),+],
        indices = [$($idx:tt),+]
    ) => {
        pub struct $struct_name<$($param,)+ F>
        where
            $($param: SystemParam + 'static,)+
            F: FnMut($($param::Item<'_, '_>),+) + 'static,
        {
            func: F,
            state: ($($param::State,)+),
        }

        impl<$($param,)+ F> System for $struct_name<$($param,)+ F>
        where
            $($param: SystemParam + 'static,)+
            F: FnMut($($param::Item<'_, '_>),+) + 'static,
        {
            fn run(&mut self, world: &mut MainWorld) {
                let ($($param,)+) = unsafe {
                    let world_ptr = world as *mut MainWorld;
                    ($($param::get_param(&mut self.state.$idx, &mut *world_ptr),)+)
                };
                (self.func)($($param,)+);
                $(world.flush_commands_from_state(&mut self.state.$idx as &mut dyn core::any::Any);)+
            }
        }

        pub fn $fn_name<$($param,)+ F>(f: F) -> impl IntoSystem
        where
            $($param: SystemParam + 'static,)+
            F: FnMut($($param::Item<'_, '_>),+) + 'static,
        {
            struct Builder<$($param,)+ F> {
                func: F,
                _marker: core::marker::PhantomData<($($param,)+)>,
            }

            impl<$($param,)+ F> IntoSystem for Builder<$($param,)+ F>
            where
                $($param: SystemParam + 'static,)+
                F: FnMut($($param::Item<'_, '_>),+) + 'static,
            {
                type System = $struct_name<$($param,)+ F>;

                fn into_system(self) -> Self::System {
                    $struct_name {
                        func: self.func,
                        state: Default::default(),
                    }
                }
            }

            Builder { func: f, _marker: core::marker::PhantomData }
        }
    };
}

impl_system!(struct_name = DeclarativeSystem1, fn_name = system1, params = [A], indices = [0]);
impl_system!(struct_name = DeclarativeSystem2, fn_name = system2, params = [A, B], indices = [0, 1]);
impl_system!(struct_name = DeclarativeSystem3, fn_name = system3, params = [A, B, C], indices = [0, 1, 2]);
impl_system!(struct_name = DeclarativeSystem4, fn_name = system4, params = [A, B, C, D], indices = [0, 1, 2, 3]);
