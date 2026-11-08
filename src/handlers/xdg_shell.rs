use smithay::{
    desktop::{PopupKind, Window},
    wayland::shell::xdg::XdgShellHandler,
};

use crate::state::RackyState;

impl XdgShellHandler for RackyState {
    fn xdg_shell_state(&mut self) -> &mut smithay::wayland::shell::xdg::XdgShellState {
        &mut self.compositor.xdg_shell_state
    }
    fn new_toplevel(&mut self, surface: smithay::wayland::shell::xdg::ToplevelSurface) {
        let window = Window::new_wayland_window(surface.clone());

        if let Some(workspace) = self.compositor.window_manager.active_workspace_mut() {
            workspace.add_window(window);
        }

        surface.send_configure();
    }
    fn new_popup(
        &mut self,
        surface: smithay::wayland::shell::xdg::PopupSurface,
        positioner: smithay::wayland::shell::xdg::PositionerState,
    ) {
        if let Err(err) = self.compositor.popups.track_popup(PopupKind::from(surface)) {
            tracing::warn!("failed to track popups");
            return;
        }
    }
}
