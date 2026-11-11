use crate::{
    backend::udev::{Backend, UdevBack},
    client::ClientState,
    state::RackyState,
};
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_dispatch2,
    input::pointer::{CursorImageStatus, CursorImageSurfaceData},
    reexports::wayland_server::protocol::wl_surface::WlSurface,
    wayland::{
        buffer::BufferHandler,
        compositor::{
            CompositorHandler, CompositorState as CompositorStateImpl, SurfaceAttributes,
            get_parent, is_sync_subsurface, with_states,
        },
        shm::ShmHandler,
    },
};

impl<Data: Backend + 'static> CompositorHandler for RackyState<Data> {
    fn compositor_state(&mut self) -> &mut CompositorStateImpl {
        &mut self.compositor.compositor_state
    }
    fn client_compositor_state<'a>(
        &self,
        client: &'a smithay::reexports::wayland_server::Client,
    ) -> &'a smithay::wayland::compositor::CompositorClientState {
        &client
            .get_data::<ClientState>()
            .expect("Wayland Client registed without wayland ClientSate")
            .compositor_state
    }
    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);

        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();

            //try to go upwards until we find a parent
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }

            if let Some(window) = self
                .compositor
                .window_manager
                .find_window_from_surface(&root)
                .cloned()
            {
                window.on_commit();

                if root == *surface {
                    let buffer_offset = with_states(surface, |states| {
                        states
                            .cached_state
                            .get::<SurfaceAttributes>()
                            .current()
                            .buffer_delta
                            .take()
                    });

                    if let Some(buffer_offset) = buffer_offset {
                        self.compositor
                            .window_manager
                            .apply_buffer_offset(&window, buffer_offset);
                    }
                }
            }
        }
        self.compositor.popups.commit(surface);

        //check and render cursor attributes
        if matches!(&self.compositor.cursor_status, CursorImageStatus::Surface(cursor_surface) if cursor_surface == surface)
        {
            with_states(surface, |states| {
                let cursor_image_attributes = states.data_map.get::<CursorImageSurfaceData>();

                if let Some(mut cursor_image_attributes) = cursor_image_attributes
                    .map(|attrs| attrs.lock().expect("error locking cursor attributes"))
                {
                    let buffer_delta = states
                        .cached_state
                        .get::<SurfaceAttributes>()
                        .current()
                        .buffer_delta
                        .take();

                    if let Some(buffer_delta) = buffer_delta {
                        cursor_image_attributes.hotspot -= buffer_delta;
                    }
                }
            });
        }
        //TODO: add a DrangAndDropOtption icon (DndIcon)
    }
}

impl<Data: Backend> BufferHandler for RackyState<Data> {
    fn buffer_destroyed(
        &mut self,
        _buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    ) {
    }
}
impl<Data: Backend> ShmHandler for RackyState<Data> {
    fn shm_state(&self) -> &smithay::wayland::shm::ShmState {
        &self.compositor.shm_state
    }
}
delegate_dispatch2!(RackyState<UdevBack>);
