use crate::window_maneger::WindowManager;
use smithay::{
    desktop::PopupManager,
    input::{SeatState, pointer::CursorImageStatus},
    reexports::{calloop::LoopHandle, wayland_server::DisplayHandle},
    wayland::{
        compositor::CompositorState as CompositorStateImpl, shell::xdg::XdgShellState,
        shm::ShmState,
    },
};

pub struct State {
    pub(crate) compositor: Compositor,
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
    pub cursor_status: CursorImageStatus,
    // Your own window-management policy
    pub window_manager: WindowManager,
    pub running: bool,
}

impl Compositor {
    fn new(display_handle: DisplayHandle) {
        let xdg_shell_state = XdgShellState::new::<State>(&display_handle);
    }
}
