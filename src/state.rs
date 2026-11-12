use crate::{
    backend::udev::{Backend, UdevBack},
    config::settings::{Configuration, OutputTransform},
    handlers::input::KeyActions,
    window_maneger::WindowManager,
};
use smallvec::SmallVec;
use smithay::{
    backend::{
        input::{Axis, Event, InputEvent, PointerAxisEvent, PointerMotionEvent},
        libinput::LibinputInputBackend,
    },
    delegate_dispatch2,
    desktop::{LayerSurface, PopupManager, layer_map_for_output},
    input::{
        Seat, SeatState,
        keyboard::Keysym,
        pointer::{AxisFrame, CursorImageStatus, MotionEvent},
    },
    output::Output,
    reexports::{
        calloop::LoopHandle,
        wayland_server::{DisplayHandle, GlobalDispatch, protocol},
    },
    utils::{SERIAL_COUNTER, Transform},
    wayland::{
        GlobalData,
        compositor::CompositorState as CompositorStateImpl,
        shell::{
            wlr_layer::{WlrLayerShellHandler, WlrLayerShellState},
            xdg::XdgShellState,
        },
        shm::ShmState,
    },
};

delegate_dispatch2!(@<B: Backend+'static>RackyState<B>);
pub struct RackyState<B: Backend + 'static> {
    pub(crate) compositor: Compositor<B>,
    pub(crate) backend: B,
}

impl<Back: Backend> RackyState<Back> {
    pub fn new(
        display_handle: DisplayHandle,
        loop_handle: LoopHandle<'static, RackyState<Back>>,
        backend: Back,
        config: Option<&Configuration>,
    ) -> Self {
        Self {
            compositor: Compositor::new(display_handle, loop_handle, config),
            backend,
        }
    }
}

impl<B: Backend + 'static> WlrLayerShellHandler for RackyState<B> {
    fn shell_state(&mut self) -> &mut WlrLayerShellState {
        &mut self.compositor.layer_shell_state
    }
    fn new_layer_surface(
        &mut self,
        surface: smithay::wayland::shell::wlr_layer::LayerSurface,
        output: Option<protocol::wl_output::WlOutput>,
        layer: smithay::wayland::shell::wlr_layer::Layer,
        namespace: String,
    ) {
        let Some(workspace) = &self.compositor.window_manager.active_workspace() else {
            return;
        };
        let Some(output) = output
            .as_ref()
            .and_then(Output::from_resource)
            .or_else(|| workspace.space.outputs().next().cloned())
        else {
            return;
        };
        let mut map = layer_map_for_output(&output);
        let Ok(_) = map.map_layer(&LayerSurface::new(surface, namespace)) else {
            return;
        };
    }
    fn layer_destroyed(&mut self, surface: smithay::wayland::shell::wlr_layer::LayerSurface) {
        let Some(workspace) = &self.compositor.window_manager.active_workspace() else {
            return;
        };
        if let Some((mut map, layer)) = workspace.space.outputs().find_map(|o| {
            let map = layer_map_for_output(o);
            let layer = map
                .layers()
                .find(|&layer| layer.layer_surface() == &surface)
                .cloned();
            layer.map(|layer| (map, layer))
        }) {
            map.unmap_layer(&layer);
        }
    }
}

pub struct Compositor<B: Backend + 'static> {
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
    pub transformation: Transform,
    // Your own window-management policy
    pub window_manager: WindowManager,
    pub running: bool,
}

impl<B: Backend + 'static> Compositor<B> {
    fn new(
        display_handle: DisplayHandle,
        loop_handle: LoopHandle<'static, RackyState<B>>,
        config: Option<&Configuration>,
    ) -> Self {
        let xdg_shell_state = XdgShellState::new::<RackyState<B>>(&display_handle);
        let compositor_state = CompositorStateImpl::new::<RackyState<B>>(&display_handle);
        let mut seat_state = SeatState::<RackyState<B>>::new();
        let popups = PopupManager::default();
        let cursor_status = CursorImageStatus::default_named();
        let window_manager = WindowManager::default(config);
        let seat = seat_state.new_wl_seat(&display_handle, "seat0");
        //FIXME: support more formats
        let shm_state = ShmState::new::<RackyState<B>>(&display_handle, vec![]);
        let layer_shell_state = WlrLayerShellState::new::<RackyState<B>>(&display_handle);

        let transform = config
            .and_then(|config| config.transform.as_ref())
            .map(|transform| Transform::from(transform.clone()))
            .unwrap_or(Transform::Normal);

        Self {
            display_handle,
            layer_shell_state,
            seat,
            transformation: transform,
            xdg_shell_state,
            loop_handle,
            compositor_state,
            cursor_status,
            shm_state,
            suppressed_keys: SmallVec::new(),
            seat_state,
            popups,
            window_manager,
            running: true,
        }
    }
}

impl<B: Backend + 'static> RackyState<B> {
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
                let action = self.keyboard_key_to_action::<LibinputInputBackend>(&event);
                self.handle_key_action(&action);
            }
            InputEvent::PointerButton { event } => {
                //TODO: call on pointer data
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
