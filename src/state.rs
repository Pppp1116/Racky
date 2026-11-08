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
    fn new(display_handle: DisplayHandle) -> Self {
        let xdg_shell_state = XdgShellState::new::<State>(&display_handle);
        let compositor_state = CompositorStateImpl::new(&display_handle);
        let seat_state = SeatState::<State>::new();
        let popups = PopupManager::default();
        let cursor_status = CursorImageStatus::default_named();
        let windowmaneger = WindowManager::default();

        //FIXME: support more formats
        let shm_state = ShmState::new(&display_handle, vec![]);

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
            running,
        }
    }
}
