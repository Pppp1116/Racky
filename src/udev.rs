use std::time::Duration;

use smithay::{
    backend::{
        drm::{CreateDrmNodeError, DrmDeviceFd, DrmNode, NodeType},
        egl::context::ContextPriority,
        renderer::{
            gles::GlesRenderer,
            multigpu::{GpuManager, gbm::GbmGlesBackend},
        },
        session::{Session, libseat::LibSeatSession},
        udev::{all_gpus, primary_gpu},
    },
    reexports::{
        calloop::EventLoop,
        wayland_server::{Display, backend::InitError},
    },
};
use thiserror::Error;

use crate::state::RackyState;
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
}

pub type Result<T> = std::result::Result<T, CompError>;

pub fn run_udev() -> Result<()> {
    let mut event_loop = EventLoop::<RackyState>::try_new()?;
    let display = Display::<RackyState>::new()?;
    let mut display_handle = display.handle();

    let (session, notifier) = LibSeatSession::new()?;

    let primary_gpu = find_primary_gpu(&session.seat())?;

    let gpus = GpuManager::new(
        GbmGlesBackend::<GlesRenderer, DrmDeviceFd>::with_context_priority(ContextPriority::High),
    );

    let mut racky_state = RackyState::new(display_handle.clone(), event_loop.handle());

    while racky_state.compositor.running {
        event_loop.dispatch(Some(Duration::from_millis(16)), &mut racky_state)?;

        display_handle.flush_clients()?;
    }
    Ok(())
}

fn find_primary_gpu(seat: &str) -> Result<DrmNode> {
    //TODO: remove the env limitation
    if let Ok(path) = std::env::var("MY_DRM_DEVICE") {
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
