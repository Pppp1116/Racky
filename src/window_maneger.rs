use smithay::{
    desktop::{Space, Window},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
    wayland::seat::WaylandFocus,
};

pub struct WindowManager {
    pub(crate) workspace_collection: Vec<Workspace>,
    pub(crate) active_workspace: usize,
}

impl WindowManager {
    pub fn find_window_from_surface(&self, surface: &WlSurface) -> Option<&Window> {
        let workspace = self.workspace_collection.get(self.active_workspace)?;

        workspace
            .space
            .elements()
            .find(|window| window.wl_surface().as_deref() == Some(surface))
    }
    pub fn active_workspace_mut(&mut self) -> Option<&mut Workspace> {
        self.workspace_collection.get_mut(self.active_workspace)
    }
    pub(crate) fn apply_buffer_offset(&mut self, window: &Window, offset: Point<i32, Logical>) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };

        let Some(location) = workspace.space.element_location(window) else {
            return;
        };

        workspace.space.relocate_element(window, location + offset);
    }
}

pub struct Workspace {
    pub space: Space<Window>,
    layout: Layout,
    pub focused_window: Option<Window>,
}

pub enum Layout {
    Tiled,
    Floating,
}
