use crate::{
    backend::udev::{Backend, UdevBack},
    window_maneger::WindowManager,
};
use smithay::{
    backend::{input::InputEvent, libinput::LibinputInputBackend},
    desktop::PopupManager,
    input::{SeatState, pointer::CursorImageStatus},
    reexports::{calloop::LoopHandle, wayland_server::DisplayHandle},
    wayland::{
        compositor::CompositorState as CompositorStateImpl, shell::xdg::XdgShellState,
        shm::ShmState,
    },
};

pub struct RackyState<B: Backend> {
    pub(crate) compositor: Compositor<B>,
    pub(crate) backend: B,
}

impl<Back: Backend> RackyState<Back> {
    pub fn new(
        display_handle: DisplayHandle,
        loop_handle: LoopHandle<'static, RackyState<Back>>,
        backend: Back,
    ) -> Self {
        Self {
            compositor: Compositor::new(display_handle, loop_handle),
            backend,
        }
    }
}

pub struct Compositor<B: Backend> {
    // Wayland infrastructure
    pub display_handle: DisplayHandle,
    pub loop_handle: LoopHandle<'static, RackyState<B>>,

    // Core protocol state
    pub compositor_state: CompositorStateImpl,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<RackyState<B>>,

    // Desktop objects
    pub popups: PopupManager,
    pub cursor_status: CursorImageStatus,
    // Your own window-management policy
    pub window_manager: WindowManager,
    pub running: bool,
}

impl<B: Backend> Compositor<B> {
    fn new(display_handle: DisplayHandle, loop_handle: LoopHandle<'static, RackyState<B>>) -> Self {
        let xdg_shell_state = XdgShellState::new(&display_handle);
        let compositor_state = CompositorStateImpl::new::<RackyState<B>>(&display_handle);
        let seat_state = SeatState::<RackyState<B>>::new();
        let popups = PopupManager::default();
        let cursor_status = CursorImageStatus::default_named();
        let window_manager = WindowManager::default();

        //FIXME: support more formats
        let shm_state = ShmState::new::<RackyState<B>>(&display_handle, vec![]);

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

impl<B: Backend> RackyState<B> {
    pub fn process_input_event(&mut self, event: InputEvent<LibinputInputBackend>) {
        match event {
            InputEvent::DeviceRemoved { device } => self.backend.remove_device(&device),
            InputEvent::DeviceAdded { device } => {
                self.backend.add_device(device);
            }
            InputEvent::PointerAxis { event } => {}
        }
    }
}
