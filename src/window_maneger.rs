use smithay::{
    desktop::{Space, Window},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
    wayland::seat::WaylandFocus,
};

pub struct WindowManager {
    workspace_collection: Vec<Workspace>,
    active_workspace: usize,
}

impl WindowManager {
    ///finds a window in all the window maneger
    pub fn find_window_from_surface(&self, surface: &WlSurface) -> Option<&Window> {
        self.workspace_collection
            .iter()
            .flat_map(|workspace| workspace.space.elements())
            .find(|window| window.wl_surface().as_deref() == Some(surface))
    }
    pub fn remember_focused_window(&mut self, window: Window) {
        let Some(workspace) = self.active_workspace_mut() else {
            return;
        };

        workspace.focused_window = Some(window)
    }
    /// gets a mutable ref to the workspace collection
    fn active_workspace_mut(&mut self) -> Option<&mut Workspace> {
        self.workspace_collection.get_mut(self.active_workspace)
    }

    //applies the buffer `offset` for the window
    pub(crate) fn apply_buffer_offset(&mut self, window: &Window, offset: Point<i32, Logical>) {
        for workspace in &mut self.workspace_collection {
            if let Some(location) = workspace.space.element_location(window) {
                workspace.space.relocate_element(window, location + offset);
                return;
            }
        }
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
