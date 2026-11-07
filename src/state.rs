use smithay::{
    desktop::{PopupManager, Space, Window},
    input::{SeatHandler, SeatState, pointer::ClickGrab},
    reexports::{
        calloop::LoopHandle,
        wayland_server::{DisplayHandle, Resource, protocol::wl_surface::WlSurface},
    },
    utils::{Logical, Point},
    wayland::{
        compositor::CompositorState as CompositorStateImpl, seat::WaylandFocus,
        shell::xdg::XdgShellState, shm::ShmState,
    },
};

pub struct State {
    compositor: Compositor,
}

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn focus_changed(
        &mut self,
        seat: &smithay::input::Seat<Self>,
        focused: Option<&Self::KeyboardFocus>,
    ) {
        let active_workspace = self.compositor.window_manager.active_workspace;
        let Some(surface) = focused else {
            return;
        };
        let window = self
            .compositor
            .window_manager
            .find_window_from_surface(surface)
        else {
            return;
        };
        let window = window.cloned();

        let Some(workspace) = self
            .compositor
            .window_manager
            .workspace_collection
            .get_mut(active_workspace)
        else {
            return;
        };

        workspace.focused_widow = window;
    }
}
impl State {
    pub fn new() {}
}

pub struct Compositor {
    // Wayland infrastructure
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, State>,

    // Core protocol state
    pub compositor_state: CompositorStateImpl,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<State>,

    // Desktop objects
    pub popups: PopupManager,

    // Your own window-management policy
    pub window_manager: WindowManager,
    pub running: bool,
}

impl Compositor {
    fn new(display_handle: DisplayHandle) {
        let xdg_shell_state = XdgShellState::new::<State>(&display_handle);
    }
}

pub struct WindowManager {
    workspace_collection: Vec<Workspace>,
    active_workspace: usize,
}
impl WindowManager {
    pub fn find_window_from_surface(&self, surface: &WlSurface) -> Option<&Window> {
        let workspace = self.workspace_collection.get(self.active_workspace)?;

        workspace
            .space
            .elements()
            .find(|window| window.wl_surface().as_deref() == Some(surface))
    }
}

pub struct Workspace {
    pub space: Space<Window>,
    layout: Layout,
    pub focused_widow: Option<Window>,
}

pub enum Layout {
    Tiled,
    Floating,
}
