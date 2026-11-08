use crate::state::State;
use smithay::{reexports::wayland_server::protocol::wl_surface::WlSurface, wayland::pointer_constraints::{PointerConstraintsHandler, with_pointer_constraint}};

impl PointerConstraintsHandler for State {
    fn new_constraint(
        &mut self,
        surface: &WlSurface,
        pointer: &smithay::input::pointer::PointerHandle<Self>,
    ) {
        //if not focused skip
        let Some(current_focus) = pointer.current_focus() else {
            return;
        };

        //if the current focus is diffrent from surface skip
        if &current_focus != surface {
            return;
        }
        with_pointer_constraint(surface, pointer, |constraint| {
            let Some(constraint) = constraint else {
                return;
            };
            constraint.activate();
        });
    }
    //TODO: impl now constraint remove
}
