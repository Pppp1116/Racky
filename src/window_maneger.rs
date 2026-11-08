use smallvec::SmallVec;
use smithay::{
    desktop::{Space, Window},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    utils::{Logical, Point},
    wayland::seat::WaylandFocus,
};
use std::borrow::Cow;

pub struct WindowManager {
    workspace_collection: Vec<Workspace>,
    active_workspace: usize,
}
impl Default for WindowManager {
    fn default() -> Self {
        //TODO: configurable workspace num
        let mut workspace_collection = Vec::with_capacity(10);

        for _i in 0..=10 {
            workspace_collection.push(Workspace::default());
        }

        Self {
            workspace_collection,
            active_workspace: 0,
        }
    }
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
    pub fn active_workspace_mut(&mut self) -> Option<&mut Workspace> {
        self.workspace_collection.get_mut(self.active_workspace)
    }

    pub fn get_window_location_from_surface(
        &self,
        surface: &WlSurface,
    ) -> Option<Point<i32, Logical>> {
        self.workspace_collection.iter().find_map(|workspace| {
            workspace
                .space
                .elements()
                .find(|window| window.wl_surface().as_deref() == Some(surface))
                .and_then(|window| workspace.space.element_location(window))
        })
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
impl Default for Workspace {
    fn default() -> Self {
        Self {
            space: Space::default(),
            //TODO: add default layout based on settigns file
            layout: Layout::Floating,
            focused_window: None,
        }
    }
}

impl Workspace {
    fn add_window(&mut self, window: Window) {
        let Some((x, y)) = self.arrange_window(&window) else {
            return;
        };

        if let Layout::Scrooling { windows, .. } = &mut self.layout {
            windows.push(window.clone());
        }
        self.space.map_element(window, (x, y), true);
    }
    fn arrange_window(&mut self, window: &Window) -> Option<(i32, i32)> {
        let Some(output) = self.space.outputs().next() else {
            return None;
        };
        let Some(screen) = self.space.output_geometry(output) else {
            return None;
        };

        let size = window.geometry().size;
        let y = screen.loc.y + (screen.size.h - size.h) / 2;

        match &self.layout {
            Layout::Floating => {
                let x = screen.loc.x + (screen.size.w - size.w) / 2;
                Some((x, y))
            }
            Layout::Scrooling {
                scrool_offset,
                windows,
            } => {
                let width = screen.size.w / 2;

                let mut x = 10;

                for existing in windows {
                    let position = (x - *scrool_offset, 10);

                    self.space.relocate_element(existing, position);

                    x += width + 10;
                }

                Some((x - *scrool_offset, y))
            }
        }
    }
}

pub enum Layout {
    Scrooling {
        scrool_offset: i32,
        windows: Vec<Window>,
    },
    Floating,
}
