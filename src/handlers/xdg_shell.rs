use smithay::{desktop::Window, wayland::shell::xdg::XdgShellHandler};

use crate::state::State;

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut smithay::wayland::shell::xdg::XdgShellState {
        &mut self.compositor.xdg_shell_state
    }
    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        let Some(current_workspace) = self.compositor.window_manager.active_workspace_mut() else {
            return;
        };

        let window = Window::new_wayland_window(surface.clone());
        current_workspace
            .space
            .map_element(window, , activate);
    }
}
