//! Input Processing Systems
//!
//! Systems that process input events and update input resources.
//! These should run in the PreUpdate schedule.

use crate::main_world::MainWorld;
use crate::event::{Events, EventReader};
use super::{
    ButtonInput,
    MouseButton, MouseButtonInput, MouseButtonState,
    MouseMotion, MouseWheel,
    AccumulatedMouseMotion, AccumulatedMouseScroll,
    KeyCode, Key, KeyboardInput, KeyboardKeyState, KeyboardFocusLost,
};

/// Process mouse button events and update ButtonInput<MouseButton>
pub fn mouse_button_input_system(world: &mut MainWorld) {
    // Collect events first to avoid borrow issues
    let mouse_events: alloc::vec::Vec<MouseButtonInput> = {
        if let Some(events) = world.resources().get::<Events>() {
            let reader = EventReader::<MouseButtonInput>::new(events);
            reader.read()
        } else {
            alloc::vec::Vec::new()
        }
    };

    // Get mutable access to button input and process events
    if let Some(button_input) = world.resources_mut().get_mut::<ButtonInput<MouseButton>>() {
        // Clear just_pressed/just_released from previous frame
        button_input.clear();

        // Process all mouse button events
        for event in mouse_events {
            match event.state {
                MouseButtonState::Pressed => button_input.press(event.button),
                MouseButtonState::Released => button_input.release(event.button),
            }
        }
    }
}

/// Accumulate mouse motion events
pub fn accumulate_mouse_motion_system(world: &mut MainWorld) {
    // Collect events first to avoid borrow issues
    let motion_events: alloc::vec::Vec<MouseMotion> = {
        if let Some(events) = world.resources().get::<Events>() {
            let reader = EventReader::<MouseMotion>::new(events);
            reader.read()
        } else {
            alloc::vec::Vec::new()
        }
    };

    // Get mutable access to accumulated motion and process events
    if let Some(accumulated) = world.resources_mut().get_mut::<AccumulatedMouseMotion>() {
        // Clear accumulated motion from previous frame
        accumulated.clear();

        // Accumulate all motion deltas
        for event in motion_events {
            accumulated.accumulate(event.delta);
        }
    }
}

/// Accumulate mouse scroll events
pub fn accumulate_mouse_scroll_system(world: &mut MainWorld) {
    // Collect events first to avoid borrow issues
    let scroll_events: alloc::vec::Vec<MouseWheel> = {
        if let Some(events) = world.resources().get::<Events>() {
            let reader = EventReader::<MouseWheel>::new(events);
            reader.read()
        } else {
            alloc::vec::Vec::new()
        }
    };

    // Get mutable access to accumulated scroll and process events
    if let Some(accumulated) = world.resources_mut().get_mut::<AccumulatedMouseScroll>() {
        // Clear accumulated scroll from previous frame
        accumulated.clear();

        // Accumulate all scroll events
        for event in scroll_events {
            let delta = crate::math::Vec2::new(event.x, event.y);
            accumulated.accumulate(delta, event.unit);
        }
    }
}

/// Process keyboard events and update ButtonInput<KeyCode> and ButtonInput<Key>
pub fn keyboard_input_system(world: &mut MainWorld) {
    // First, check for focus lost and clear states
    let focus_lost = world.resources().get::<KeyboardFocusLost>().is_some();

    if focus_lost {
        // Release all keys when focus is lost
        if let Some(keycode_input) = world.resources_mut().get_mut::<ButtonInput<KeyCode>>() {
            keycode_input.release_all();
        }
        if let Some(key_input) = world.resources_mut().get_mut::<ButtonInput<Key>>() {
            key_input.release_all();
        }
        world.resources_mut().remove::<KeyboardFocusLost>();
        return;
    }

    // Collect events first to avoid borrow issues
    let keyboard_events: alloc::vec::Vec<KeyboardInput> = {
        if let Some(events) = world.resources().get::<Events>() {
            let reader = EventReader::<KeyboardInput>::new(events);
            reader.read()
        } else {
            alloc::vec::Vec::new()
        }
    };

    // Clear just_pressed/just_released from previous frame
    if let Some(keycode_input) = world.resources_mut().get_mut::<ButtonInput<KeyCode>>() {
        keycode_input.clear();
    }
    if let Some(key_input) = world.resources_mut().get_mut::<ButtonInput<Key>>() {
        key_input.clear();
    }

    // Process keyboard events
    for event in keyboard_events {
        match event.state {
            KeyboardKeyState::Pressed => {
                if let Some(keycode_input) = world.resources_mut().get_mut::<ButtonInput<KeyCode>>() {
                    keycode_input.press(event.key_code);
                }
                if let Some(key_input) = world.resources_mut().get_mut::<ButtonInput<Key>>() {
                    key_input.press(event.logical_key);
                }
            }
            KeyboardKeyState::Released => {
                if let Some(keycode_input) = world.resources_mut().get_mut::<ButtonInput<KeyCode>>() {
                    keycode_input.release(event.key_code);
                }
                if let Some(key_input) = world.resources_mut().get_mut::<ButtonInput<Key>>() {
                    key_input.release(event.logical_key);
                }
            }
        }
    }
}

/// Update all input systems - convenience function
/// This should be called in the PreUpdate phase
pub fn input_systems_update(world: &mut MainWorld) {
    // Update event buffers first (swap current/read buffers)
    if let Some(events) = world.resources_mut().get_mut::<Events>() {
        events.update_type::<MouseButtonInput>();
        events.update_type::<MouseMotion>();
        events.update_type::<MouseWheel>();
        events.update_type::<KeyboardInput>();
    }
    
    // Then process input events
    mouse_button_input_system(world);
    accumulate_mouse_motion_system(world);
    accumulate_mouse_scroll_system(world);
    keyboard_input_system(world);
}

// Resource trait implementations are in their respective module files
