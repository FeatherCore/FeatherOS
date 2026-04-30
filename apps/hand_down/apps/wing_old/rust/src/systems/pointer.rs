use fhre::{Click, Events, Out, Over, Pointer, Press, Query, Res};

use crate::ButtonWidget;

pub fn wing_minimal_button_interaction_system(
    events: Res<Events>,
    mut button_query: Query<&mut ButtonWidget>,
) {
    for (_, button) in button_query.iter_mut() {
        button.clicked = false;
    }

    if let Some(over_events) = events.get_events_current::<Pointer<Over>>() {
        for event in over_events {
            if let Some((_, button)) = button_query.get_pair_mut(event.entity) {
                button.hovered = true;
            }
        }
    }

    if let Some(out_events) = events.get_events_current::<Pointer<Out>>() {
        for event in out_events {
            if let Some((_, button)) = button_query.get_pair_mut(event.entity) {
                button.hovered = false;
                button.pressed = false;
            }
        }
    }

    if let Some(press_events) = events.get_events_current::<Pointer<Press>>() {
        for event in press_events {
            if let Some((_, button)) = button_query.get_pair_mut(event.entity) {
                button.hovered = true;
                button.pressed = true;
            }
        }
    }

    if let Some(click_events) = events.get_events_current::<Pointer<Click>>() {
        for event in click_events {
            if let Some((_, button)) = button_query.get_pair_mut(event.entity) {
                button.hovered = true;
                button.pressed = false;
                button.clicked = true;
            }
        }
    }
}
