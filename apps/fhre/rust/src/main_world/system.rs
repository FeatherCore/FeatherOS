//! System Implementation
//!
//! Provides the System trait and related types for ECS.
//! Supports both imperative and declarative system styles.

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

// Implement System for function pointers
impl System for Box<dyn FnMut(&mut MainWorld)> {
    fn run(&mut self, world: &mut MainWorld) {
        self(world);
    }
}

// Implement IntoSystem for closures
impl<F: FnMut(&mut MainWorld) + 'static> IntoSystem for F {
    type System = Box<dyn FnMut(&mut MainWorld)>;

    fn into_system(self) -> Self::System {
        Box::new(self)
    }
}

// === Declarative Systems ===

/// System with 1 parameter
pub struct DeclarativeSystem1<A, F>
where
    A: SystemParam + 'static,
    F: FnMut(A::Item<'_, '_>) + 'static,
{
    func: F,
    state: A::State,
}

impl<A, F> System for DeclarativeSystem1<A, F>
where
    A: SystemParam + 'static,
    F: FnMut(A::Item<'_, '_>) + 'static,
{
    fn run(&mut self, world: &mut MainWorld) {
        let param = unsafe { A::get_param(&mut self.state, world) };
        (self.func)(param);
        
        // Apply commands after system runs
        // The state now contains any commands that were queued
        self.apply_commands(world);
    }
}

impl<A, F> DeclarativeSystem1<A, F>
where
    A: SystemParam + 'static,
    F: FnMut(A::Item<'_, '_>) + 'static,
{
    /// Apply commands from state if this is a Commands parameter
    fn apply_commands(&mut self, world: &mut MainWorld) {
        // Try to downcast state to CommandsState and apply
        use super::commands::CommandsState;
        
        // SAFETY: We use a type check via TypeId to ensure safety
        if core::any::TypeId::of::<A::State>() == core::any::TypeId::of::<CommandsState>() {
            unsafe {
                let state_ptr = &mut self.state as *mut A::State as *mut CommandsState;
                (*state_ptr).apply(world);
            }
        }
    }
}

/// System with 2 parameters
pub struct DeclarativeSystem2<A, B, F>
where
    A: SystemParam + 'static,
    B: SystemParam + 'static,
    F: FnMut(A::Item<'_, '_>, B::Item<'_, '_>) + 'static,
{
    func: F,
    state: (A::State, B::State),
}

impl<A, B, F> System for DeclarativeSystem2<A, B, F>
where
    A: SystemParam + 'static,
    B: SystemParam + 'static,
    F: FnMut(A::Item<'_, '_>, B::Item<'_, '_>) + 'static,
{
    fn run(&mut self, world: &mut MainWorld) {
        let (a, b) = unsafe {
            let world_ptr = world as *mut MainWorld;
            let a_item = A::get_param(&mut self.state.0, &mut *world_ptr);
            let b_item = B::get_param(&mut self.state.1, &mut *world_ptr);
            (a_item, b_item)
        };
        (self.func)(a, b);
        
        // Apply commands after system runs
        self.apply_commands(world);
    }
}

impl<A, B, F> DeclarativeSystem2<A, B, F>
where
    A: SystemParam + 'static,
    B: SystemParam + 'static,
    F: FnMut(A::Item<'_, '_>, B::Item<'_, '_>) + 'static,
{
    /// Apply commands from any Commands parameters
    fn apply_commands(&mut self, world: &mut MainWorld) {
        use super::commands::CommandsState;
        
        // Check and apply for A
        if core::any::TypeId::of::<A::State>() == core::any::TypeId::of::<CommandsState>() {
            unsafe {
                let state_ptr = &mut self.state.0 as *mut A::State as *mut CommandsState;
                (*state_ptr).apply(world);
            }
        }
        
        // Check and apply for B
        if core::any::TypeId::of::<B::State>() == core::any::TypeId::of::<CommandsState>() {
            unsafe {
                let state_ptr = &mut self.state.1 as *mut B::State as *mut CommandsState;
                (*state_ptr).apply(world);
            }
        }
    }
}

/// System with 3 parameters
pub struct DeclarativeSystem3<A, B, C, F>
where
    A: SystemParam + 'static,
    B: SystemParam + 'static,
    C: SystemParam + 'static,
    F: FnMut(A::Item<'_, '_>, B::Item<'_, '_>, C::Item<'_, '_>) + 'static,
{
    func: F,
    state: (A::State, B::State, C::State),
}

impl<A, B, C, F> System for DeclarativeSystem3<A, B, C, F>
where
    A: SystemParam + 'static,
    B: SystemParam + 'static,
    C: SystemParam + 'static,
    F: FnMut(A::Item<'_, '_>, B::Item<'_, '_>, C::Item<'_, '_>) + 'static,
{
    fn run(&mut self, world: &mut MainWorld) {
        let (a, b, c) = unsafe {
            let world_ptr = world as *mut MainWorld;
            let a_item = A::get_param(&mut self.state.0, &mut *world_ptr);
            let b_item = B::get_param(&mut self.state.1, &mut *world_ptr);
            let c_item = C::get_param(&mut self.state.2, &mut *world_ptr);
            (a_item, b_item, c_item)
        };
        (self.func)(a, b, c);
        
        // Apply commands after system runs
        self.apply_commands(world);
    }
}

impl<A, B, C, F> DeclarativeSystem3<A, B, C, F>
where
    A: SystemParam + 'static,
    B: SystemParam + 'static,
    C: SystemParam + 'static,
    F: FnMut(A::Item<'_, '_>, B::Item<'_, '_>, C::Item<'_, '_>) + 'static,
{
    /// Apply commands from any Commands parameters
    fn apply_commands(&mut self, world: &mut MainWorld) {
        use super::commands::CommandsState;
        
        // Check and apply for A
        if core::any::TypeId::of::<A::State>() == core::any::TypeId::of::<CommandsState>() {
            unsafe {
                let state_ptr = &mut self.state.0 as *mut A::State as *mut CommandsState;
                (*state_ptr).apply(world);
            }
        }
        
        // Check and apply for B
        if core::any::TypeId::of::<B::State>() == core::any::TypeId::of::<CommandsState>() {
            unsafe {
                let state_ptr = &mut self.state.1 as *mut B::State as *mut CommandsState;
                (*state_ptr).apply(world);
            }
        }
        
        // Check and apply for C
        if core::any::TypeId::of::<C::State>() == core::any::TypeId::of::<CommandsState>() {
            unsafe {
                let state_ptr = &mut self.state.2 as *mut C::State as *mut CommandsState;
                (*state_ptr).apply(world);
            }
        }
    }
}

/// Create a system with 1 parameter
pub fn system1<A, F>(f: F) -> impl IntoSystem
where
    A: SystemParam + 'static,
    F: FnMut(A::Item<'_, '_>) + 'static,
{
    struct Builder<A, F> {
        func: F,
        _marker: core::marker::PhantomData<A>,
    }
    
    impl<A, F> IntoSystem for Builder<A, F>
    where
        A: SystemParam + 'static,
        F: FnMut(A::Item<'_, '_>) + 'static,
    {
        type System = DeclarativeSystem1<A, F>;

        fn into_system(self) -> Self::System {
            DeclarativeSystem1 {
                func: self.func,
                state: Default::default(),
            }
        }
    }
    
    Builder { func: f, _marker: core::marker::PhantomData }
}

/// Create a system with 2 parameters
pub fn system2<A, B, F>(f: F) -> impl IntoSystem
where
    A: SystemParam + 'static,
    B: SystemParam + 'static,
    F: FnMut(A::Item<'_, '_>, B::Item<'_, '_>) + 'static,
{
    struct Builder<A, B, F> {
        func: F,
        _marker: core::marker::PhantomData<(A, B)>,
    }
    
    impl<A, B, F> IntoSystem for Builder<A, B, F>
    where
        A: SystemParam + 'static,
        B: SystemParam + 'static,
        F: FnMut(A::Item<'_, '_>, B::Item<'_, '_>) + 'static,
    {
        type System = DeclarativeSystem2<A, B, F>;

        fn into_system(self) -> Self::System {
            DeclarativeSystem2 {
                func: self.func,
                state: Default::default(),
            }
        }
    }
    
    Builder { func: f, _marker: core::marker::PhantomData }
}

/// Create a system with 3 parameters
pub fn system3<A, B, C, F>(f: F) -> impl IntoSystem
where
    A: SystemParam + 'static,
    B: SystemParam + 'static,
    C: SystemParam + 'static,
    F: FnMut(A::Item<'_, '_>, B::Item<'_, '_>, C::Item<'_, '_>) + 'static,
{
    struct Builder<A, B, C, F> {
        func: F,
        _marker: core::marker::PhantomData<(A, B, C)>,
    }
    
    impl<A, B, C, F> IntoSystem for Builder<A, B, C, F>
    where
        A: SystemParam + 'static,
        B: SystemParam + 'static,
        C: SystemParam + 'static,
        F: FnMut(A::Item<'_, '_>, B::Item<'_, '_>, C::Item<'_, '_>) + 'static,
    {
        type System = DeclarativeSystem3<A, B, C, F>;

        fn into_system(self) -> Self::System {
            DeclarativeSystem3 {
                func: self.func,
                state: Default::default(),
            }
        }
    }
    
    Builder { func: f, _marker: core::marker::PhantomData }
}
