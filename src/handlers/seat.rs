use crate::state::State;
use smithay::{input::{SeatHandler, SeatState}, reexports::wayland_server::protocol::wl_surface::WlSurface};

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.compositor.seat_state
    }

    fn focus_changed(
        &mut self,
        seat: &smithay::input::Seat<Self>,
        focused: Option<&Self::KeyboardFocus>,
    ) {
        let wm = &mut self.compositor.window_manager;

        let Some(surface) = focused else {
            return;
        };

        let Some(window) = wm.find_window_from_surface(surface).cloned() else {
            return;
        };

        let Some(workspace) = wm.workspace_collection.get_mut(wm.active_workspace) else {
            return;
        };

        workspace.focused_window = Some(window);
    }
}

