use crate::state::RackyState;
use smithay::{
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::pointer_constraints::{
        ConstraintRemove, PointerConstraint, PointerConstraintsHandler, with_pointer_constraint,
    },
};

impl PointerConstraintsHandler for RackyState {
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
            let Some(origin) = self
                .compositor
                .window_manager
                .get_window_location_from_surface(surface)
            else {
                return;
            };

            let surface_local = pointer.current_location() - origin.to_f64();

            let Some(constraint) = constraint else {
                return;
            };

            if constraint
                .region()
                .is_none_or(|region| region.contains(surface_local.to_i32_round()))
            {
                constraint.activate();
            }
        });
    }
    fn remove_constraint(
        &mut self,
        _surface: &WlSurface,
        _pointer: &smithay::input::pointer::PointerHandle<Self>,
        _constraint_remove: smithay::wayland::pointer_constraints::ConstraintRemove,
    ) {
    }
}
