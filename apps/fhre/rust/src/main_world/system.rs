//! System Implementation
//!
//! Provides the System trait and related types for ECS.
//!
//! Systems are registered via `app.add_systems(Update, system2::<A, B, _>(my_func))`.
//! The `system1`/`system2`/`system3`/`system4` functions wrap user functions
//! into type-erased `Box<dyn System>` objects.
//!
//! ## Simplified Registration with `sys!` macro
//!
//! ```rust
//! use fhre::{sys, Res, ResMut, Query, Commands};
//!
//! // Instead of:
//! app.add_systems(Update, system2::<Res<Time>, ResMut<State>, _>(my_system));
//!
//! // You can write:
//! app.add_systems(Update, sys!(my_system));
//! ```

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
        pub struct $struct_name<$($param,)+ Func>
        where
            $($param: SystemParam + 'static,)+
            Func: FnMut($($param::Item<'_, '_>),+) + 'static,
        {
            func: Func,
            state: ($($param::State,)+),
        }

        impl<$($param,)+ Func> System for $struct_name<$($param,)+ Func>
        where
            $($param: SystemParam + 'static,)+
            Func: FnMut($($param::Item<'_, '_>),+) + 'static,
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

        pub fn $fn_name<$($param,)+ Func>(f: Func) -> impl IntoSystem
        where
            $($param: SystemParam + 'static,)+
            Func: FnMut($($param::Item<'_, '_>),+) + 'static,
        {
            struct Builder<$($param,)+ Func> {
                func: Func,
                _marker: core::marker::PhantomData<($($param,)+)>,
            }

            impl<$($param,)+ Func> IntoSystem for Builder<$($param,)+ Func>
            where
                $($param: SystemParam + 'static,)+
                Func: FnMut($($param::Item<'_, '_>),+) + 'static,
            {
                type System = $struct_name<$($param,)+ Func>;

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
impl_system!(struct_name = DeclarativeSystem5, fn_name = system5, params = [A, B, C, D, E], indices = [0, 1, 2, 3, 4]);
impl_system!(struct_name = DeclarativeSystem6, fn_name = system6, params = [A, B, C, D, E, F], indices = [0, 1, 2, 3, 4, 5]);
impl_system!(struct_name = DeclarativeSystem7, fn_name = system7, params = [A, B, C, D, E, F, G], indices = [0, 1, 2, 3, 4, 5, 6]);
impl_system!(struct_name = DeclarativeSystem8, fn_name = system8, params = [A, B, C, D, E, F, G, H], indices = [0, 1, 2, 3, 4, 5, 6, 7]);
impl_system!(struct_name = DeclarativeSystem9, fn_name = system9, params = [A, B, C, D, E, F, G, H, I], indices = [0, 1, 2, 3, 4, 5, 6, 7, 8]);

/// Macro to simplify system registration with automatic type inference hints
///
/// This macro helps reduce boilerplate when registering systems.
/// It works by providing type hints to help the compiler infer system parameters.
///
/// # Example
///
/// ```rust
/// use fhre::{sys, Res, ResMut, Query, Commands};
///
/// fn my_system(time: Res<Time>, mut state: ResMut<State>) {
///     // system logic
/// }
///
/// // Register with simplified syntax
/// app.add_systems(Update, sys!(my_system));
/// ```
///
/// # How it works
///
/// The macro expands to the appropriate `systemN` call based on the function signature.
/// Due to Rust's type system limitations in `no_std`, explicit type annotations are
/// still required for the system parameters.
#[macro_export]
macro_rules! sys {
    ($func:expr) => {
        $func
    };
}

/// Macro for declaring systems with explicit parameter types
///
/// This is the recommended way to register systems in FHRE.
/// It provides a cleaner syntax than raw `systemN` calls.
///
/// # Example
///
/// ```rust
/// use fhre::{declare_system, Res, ResMut, Query, Commands};
///
/// fn my_system(time: Res<Time>, mut state: ResMut<State>) {
///     // system logic
/// }
///
/// // Register with explicit types
/// app.add_systems(Update, declare_system!(my_system; Res<Time>, ResMut<State>));
/// ```
#[macro_export]
macro_rules! declare_system {
    ($func:expr; $($param:ty),+ $(,)?) => {
        $crate::system_param_call!($func; $($param),+)
    };
}

/// Internal macro for system parameter calls
#[macro_export]
macro_rules! system_param_call {
    ($func:expr; $a:ty) => {
        $crate::system1::<$a, _>($func)
    };
    ($func:expr; $a:ty, $b:ty) => {
        $crate::system2::<$a, $b, _>($func)
    };
    ($func:expr; $a:ty, $b:ty, $c:ty) => {
        $crate::system3::<$a, $b, $c, _>($func)
    };
    ($func:expr; $a:ty, $b:ty, $c:ty, $d:ty) => {
        $crate::system4::<$a, $b, $c, $d, _>($func)
    };
    ($func:expr; $a:ty, $b:ty, $c:ty, $d:ty, $e:ty) => {
        $crate::system5::<$a, $b, $c, $d, $e, _>($func)
    };
    ($func:expr; $a:ty, $b:ty, $c:ty, $d:ty, $e:ty, $f:ty) => {
        $crate::system6::<$a, $b, $c, $d, $e, $f, _>($func)
    };
    ($func:expr; $a:ty, $b:ty, $c:ty, $d:ty, $e:ty, $f:ty, $g:ty) => {
        $crate::system7::<$a, $b, $c, $d, $e, $f, $g, _>($func)
    };
    ($func:expr; $a:ty, $b:ty, $c:ty, $d:ty, $e:ty, $f:ty, $g:ty, $h:ty) => {
        $crate::system8::<$a, $b, $c, $d, $e, $f, $g, $h, _>($func)
    };
    ($func:expr; $a:ty, $b:ty, $c:ty, $d:ty, $e:ty, $f:ty, $g:ty, $h:ty, $i:ty) => {
        $crate::system9::<$a, $b, $c, $d, $e, $f, $g, $h, $i, _>($func)
    };
}
