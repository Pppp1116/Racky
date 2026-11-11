use crate::{
    backend::udev::{Backend, UdevBack},
    handlers::input::KeyActions,
    window_maneger::WindowManager,
};
use smallvec::SmallVec;
use smithay::{
    backend::{
        input::{Axis, Event, InputEvent, PointerAxisEvent, PointerMotionEvent},
        libinput::{LibinputInputBackend, PointerScrollAxis},
    },
    desktop::PopupManager,
    input::{
        Seat, SeatState,
        keyboard::Keysym,
        pointer::{AxisFrame, CursorImageStatus, MotionEvent},
    },
    reexports::{
        calloop::LoopHandle,
        wayland_server::{DisplayHandle, protocol::wl_seat::WlSeat},
    },
    utils::SERIAL_COUNTER,
    wayland::{
        compositor::CompositorState as CompositorStateImpl,
        shell::{wlr_layer::WlrLayerShellState, xdg::XdgShellState},
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
    pub seat: Seat<RackyState<B>>,

    pub suppressed_keys: SmallVec<[Keysym; 10]>,
    // Desktop objects
    pub popups: PopupManager,
    pub cursor_status: CursorImageStatus,
    pub layer_shell_state: WlrLayerShellState,
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
        let seat = seat_state.new_wl_seat(&display_handle, "seat0");
        //FIXME: support more formats
        let shm_state = ShmState::new::<RackyState<B>>(&display_handle, vec![]);
        let layer_shell_state = WlrLayerShellState::new(&display_handle);

        Self {
            display_handle,
            layer_shell_state,
            seat,
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
            InputEvent::PointerAxis { event } => {
                let Some(pointer) = self.compositor.seat.get_pointer() else {
                    return;
                };
                let mut frame = AxisFrame::new(event.time()).source(event.source());

                //FIXME: maybe add likely hint
                if let Some(ammount) = event.amount(Axis::Vertical) {
                    frame = frame.value(Axis::Vertical, ammount);
                }
                if let Some(ammount) = event.amount(Axis::Horizontal) {
                    frame = frame.value(Axis::Horizontal, ammount);
                }
                pointer.axis(self, frame);
                pointer.frame(self);
            }
            InputEvent::PointerMotion { event } => {
                let Some(pointer) = self.compositor.seat.get_pointer() else {
                    return;
                };
                let position = pointer.current_location() + event.delta();

                let focus = self.compositor.window_manager.surface_under(position);

                pointer.motion(
                    self,
                    focus,
                    &MotionEvent {
                        location: position,
                        serial: SERIAL_COUNTER.next_serial(),
                        time: event.time(),
                    },
                );
                pointer.frame(self);
            }
            InputEvent::Keyboard { event } => {
                let action = self.keyboard_key_to_action(&event);
            }
        }
    }
    fn handle_key_action(&mut self, action: &KeyActions) {
        match action {
            KeyActions::QuitCompositor => {
                self.compositor.running = false;
            }
            KeyActions::CloseFocus => {
                let Some(workspace) = self.compositor.window_manager.active_workspace_mut() else {
                    return;
                };
                self.compositor.window_manager.close_focused_window();
            }
            _ => (),
        }
    }
}
