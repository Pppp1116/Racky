use smithay::{reexports::wayland_server::backend::ClientData, wayland::compositor::CompositorClientState};

///Shared protocol State for the compositor
#[derive(Default)]
pub(crate) struct ClientState {
    pub(crate) compositor_state: CompositorClientState,
}

//TODO: maybe add debug msg or logs
impl ClientData for ClientState {}

