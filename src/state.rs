use crate::window_maneger::WindowManager;
use smithay::{
    desktop::PopupManager,
    input::{SeatState, pointer::CursorImageStatus},
    reexports::{calloop::LoopHandle, wayland_server::DisplayHandle},
    wayland::{
        buffer::BufferHandler, compositor::CompositorState as CompositorStateImpl,
        shell::xdg::XdgShellState, shm::ShmState,
    },
};

pub struct RackyState {
    pub(crate) compositor: Compositor,
}

impl RackyState {
    pub fn new(
        display_handle: DisplayHandle,
        loop_handle: LoopHandle<'static, RackyState>,
    ) -> Self {
        Self {
            compositor: Compositor::new(display_handle, loop_handle),
        }
    }
}

pub struct Compositor {
    // Wayland infrastructure
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, RackyState>,

    // Core protocol state
    pub compositor_state: CompositorStateImpl,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<RackyState>,

    // Desktop objects
    pub popups: PopupManager,
    pub cursor_status: CursorImageStatus,
    // Your own window-management policy
    pub window_manager: WindowManager,
    pub running: bool,
}

impl Compositor {
    fn new(display_handle: DisplayHandle, loop_handle: LoopHandle<'static, RackyState>) -> Self {
        let xdg_shell_state = XdgShellState::new::<RackyState>(&display_handle);
        let compositor_state = CompositorStateImpl::new::<RackyState>(&display_handle);
        let seat_state = SeatState::<RackyState>::new();
        let popups = PopupManager::default();
        let cursor_status = CursorImageStatus::default_named();
        let window_manager = WindowManager::default();

        //FIXME: support more formats
        let shm_state = ShmState::new::<RackyState>(&display_handle, vec![]);

        Self {
            display_handle,
            xdg_shell_state,
            loop_handle,
            compositor_state,
            cursor_status,
            shm_state,
            seat_state,
            popups,
            window_manager,
            running: true,
        }
    }
}
