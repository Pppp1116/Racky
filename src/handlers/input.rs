use smithay::{
    backend::input::{Event, InputBackend, KeyState, KeyboardKeyEvent},
    desktop::layer_map_for_output,
    input::keyboard::{FilterResult, Keysym, ModifiersState},
    utils::SERIAL_COUNTER,
    wayland::shell::wlr_layer::KeyboardInteractivity,
};

use crate::{backend::udev::Backend, state::RackyState};

pub enum KeyActions {
    None,
    QuitCompositor,
    CloseFocus,
}

impl<BackendData: Backend> RackyState<BackendData> {
    pub fn keyboard_key_to_action<B: InputBackend>(
        &mut self,
        event: &B::KeyboardKeyEvent,
    ) -> KeyActions {
        let keycode = event.key_code();
        let state = event.state();
        let serial = SERIAL_COUNTER.next_serial();
        let time = event.time();
        use smithay::wayland::{
            keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitorSeat, seat::WaylandFocus,
            shell::wlr_layer::Layer as WlrLayer,
        };
        let Some(keyboard) = self.compositor.seat.get_keyboard() else {
            return KeyActions::None;
        };
        //TODO: Add suppressed keys config and add here

        for layer in self.compositor.layer_shell_state.layer_surfaces() {
            let exclusive = layer.with_cached_state(|data| {
                data.keyboard_interactivity == KeyboardInteractivity::Exclusive
                    && (data.layer == WlrLayer::Overlay)
            });

            if exclusive {
                let Some(workspace) = self.compositor.window_manager.active_workspace_mut() else {
                    continue;
                };
                let surface = workspace.space.outputs().find_map(|o| {
                    let map = layer_map_for_output(o);
                    map.layers().find(|l| l.layer_surface() == &layer).cloned()
                });

                if let Some(surface) = surface {
                    keyboard.set_focus(self, Some(surface.into()), serial);
                    keyboard.input(self, keycode, state, serial, time, |_, _, _| {
                        FilterResult::<()>::Forward
                    });
                    return KeyActions::None;
                }
            }
        }
        let inhibited = self
            .compositor
            .seat
            .get_pointer()
            .and_then(|pointer| {
                self.compositor
                    .window_manager
                    .active_workspace()
                    .and_then(|workspace| workspace.space.element_under(pointer.current_location()))
                    .and_then(|(window, _)| {
                        let surface = window.wl_surface()?;
                        self.compositor
                            .seat
                            .keyboard_shortcuts_inhibitor_for_surface(&surface)
                    })
            })
            .map(|inhibitor| inhibitor.is_active())
            .unwrap_or(false);

        let mut suppressed_keys = std::mem::take(&mut self.compositor.suppressed_keys);

        let action = keyboard.input(
            self,
            keycode,
            state,
            serial,
            time,
            |_, modifiers, handle| {
                let keysym = handle.modified_sym();

                if state == KeyState::Pressed {
                    if inhibited {
                        return FilterResult::Forward;
                    }

                    let action = process_keyboard_shortcuts(*modifiers, keysym);

                    if action.is_some() {
                        suppressed_keys.push(keysym);
                    }

                    action
                        .map(FilterResult::Intercept)
                        .unwrap_or(FilterResult::Forward)
                } else if let Some(index) = suppressed_keys.iter().position(|key| *key == keysym) {
                    suppressed_keys.swap_remove(index);
                    FilterResult::Intercept(KeyActions::None)
                } else {
                    FilterResult::Forward
                }
            },
        );

        self.compositor.suppressed_keys = suppressed_keys;

        let action = action.unwrap_or(KeyActions::None);
        return action;
    }
}

fn process_keyboard_shortcuts(modefiers: ModifiersState, keysym: Keysym) -> Option<KeyActions> {
    if !modefiers.ctrl && !modefiers.alt {
        if modefiers.logo && modefiers.shift && keysym == Keysym::Escape {
            return Some(KeyActions::QuitCompositor);
        }
    }
    return None;
}
