use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    desktop::{PopupManager, Space, Window},
    input::{
        SeatHandler, SeatState,
        pointer::{CursorImageStatus, CursorImageSurfaceData},
    },
    reexports::{
        calloop::LoopHandle,
        wayland_server::{DisplayHandle, Resource, protocol::wl_surface::WlSurface},
    },
    wayland::{
        compositor::{
            CompositorClientState, CompositorHandler, CompositorState as CompositorStateImpl,
            SurfaceAttributes, get_parent, is_sync_subsurface, with_states,
        },
        seat::WaylandFocus,
        shell::xdg::XdgShellState,
        shm::ShmState,
    },
};

pub struct State {
    compositor: Compositor,
}

impl CompositorHandler for State {
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
                        let Some(workspace) = self.compositor.window_manager.active_workspace_mut()
                        else {
                            return;
                        };

                        let Some(current_loc) = workspace.space.element_location(&window) else {
                            return;
                        };
                        workspace
                            .space
                            .relocate_element(&window, current_loc + buffer_offset);
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
                    .map(|attrs| attrs.lock().expected("error locking cursor attributes"))
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

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.compositor.seat_state
    }

    fn focus_changed(
        &mut self,
        seat: &smithay::input::Seat<Self>,
        focused: Option<&Self::KeyboardFocus>,
    ) {
        let wm = &mut self.compositor.window_manager;

        let Some(surface) = focused else {
            return;
        };

        let Some(window) = wm.find_window_from_surface(surface).cloned() else {
            return;
        };

        let Some(workspace) = wm.workspace_collection.get_mut(wm.active_workspace) else {
            return;
        };

        workspace.focused_window = Some(window);
    }
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
///Shared protocol State for the compositor
#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
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
    pub fn active_workspace_mut(&mut self) -> Option<&mut Workspace> {
        self.workspace_collection.get_mut(self.active_workspace)
    }
}

pub struct Workspace {
    pub space: Space<Window>,
    layout: Layout,
    pub focused_window: Option<Window>,
}

pub enum Layout {
    Tiled,
    Floating,
}
