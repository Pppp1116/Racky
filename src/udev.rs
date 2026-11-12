use std::time::Duration;

use smithay::{
    backend::{
        drm::{CreateDrmNodeError, DrmDeviceFd, DrmNode, NodeType, output::DrmOutput},
        egl::context::ContextPriority,
        input::DeviceCapability::Keyboard,
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{
            gles::GlesRenderer,
            multigpu::{GpuManager, gbm::GbmGlesBackend},
        },
        session::{Session, libseat::LibSeatSession},
        udev::{UdevBackend, all_gpus, primary_gpu},
    },
    reexports::{
        calloop::EventLoop,
        input::Libinput,
        wayland_server::{Display, backend::InitError},
    },
};
use thiserror::Error;

use crate::{
    backend::udev::{Backend, GpuState, OutputState, UdevBack},
    config::settings::Configuration,
    state::RackyState,
};
#[derive(Debug, Error)]
pub enum CompError {
    #[error("Event Loop Error: {0}")]
    EventLoop(#[from] smithay::reexports::calloop::Error),
    #[error("LibSeatError: {0}")]
    LibSeatSession(#[from] smithay::backend::session::libseat::Error),
    #[error("CreateDrmNodeError: {0}")]
    DrmNoneError(#[from] CreateDrmNodeError),
    #[error("Io Error :{0}")]
    Io(#[from] std::io::Error),
    #[error("Display Init Error: {0}")]
    InitDisplay(#[from] InitError),
    #[error("NoGpu Error")]
    NoGpu,
    #[error("Gpu path")]
    GpuPath,
    #[error("Gpu Maneger")]
    Gpus,
}

pub type Result<T> = std::result::Result<T, CompError>;

pub fn run_udev(config: Option<&Configuration>) -> Result<()> {
    let mut event_loop = EventLoop::<RackyState<UdevBack>>::try_new()?;
    let display = Display::<RackyState<UdevBack>>::new()?;
    let mut display_handle = display.handle();

    let (session, notifier) = LibSeatSession::new()?;

    let primary_gpu = find_primary_gpu(&session.seat())?;

    let gpus = GpuManager::new(
        GbmGlesBackend::<GlesRenderer, DrmDeviceFd>::with_context_priority(ContextPriority::High),
    )
    .map_err(|_| CompError::Gpus)?;

    let backend = UdevBack::new(
        session,
        Vec::new(),
        GpuState::new(primary_gpu, gpus),
        Vec::new(),
    );

    let libinput = init_lib_input(&backend.session);

    let mut racky_state =
        RackyState::<UdevBack>::new(display_handle.clone(), event_loop.handle(), backend, config);

    event_loop
        .handle()
        .insert_source(libinput, |event, _, state| {
            state.process_input_event(event);
        });

    while racky_state.compositor.running {
        event_loop.dispatch(Some(Duration::from_millis(16)), &mut racky_state)?;

        display_handle.flush_clients()?;
    }

    Ok(())
}

fn find_primary_gpu(seat: &str) -> Result<DrmNode> {
    //TODO: remove the env limitation
    if let Ok(path) = std::env::var("DRM_DEVICE") {
        return Ok(DrmNode::from_path(path)?);
    }

    if let Some(path) = primary_gpu(seat)? {
        if let Ok(node) = DrmNode::from_path(path) {
            if let Ok(render_node) = node
                .node_with_type(NodeType::Render)
                .ok_or_else(|| CompError::GpuPath)
            {
                return Ok(render_node?);
            }
        }
    }

    for path in all_gpus(seat)? {
        if let Ok(node) = DrmNode::from_path(path) {
            return Ok(node);
        }
    }
    Err(CompError::NoGpu)
}

fn init_lib_input(session: &LibSeatSession) -> LibinputInputBackend {
    let mut libinput =
        Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(session.clone().into());
    libinput.udev_assign_seat(&session.seat()).unwrap();

    LibinputInputBackend::new(libinput)
}
