use crate::{backend::udev::Backend, state::RackyState};
use smithay::{
    input::{SeatHandler, SeatState},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
};

impl<B: Backend + 'static> SeatHandler for RackyState<B> {
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
        let Some(surface) = focused else {
            return;
        };

        let Some(window) = self
            .compositor
            .window_manager
            .find_window_from_surface(surface)
            .cloned()
        else {
            return;
        };

        self.compositor
            .window_manager
            .remember_focused_window(window);
    }
}
