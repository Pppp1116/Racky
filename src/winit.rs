use smithay::{
    backend::{self, renderer::gles::GlesRenderer},
    reexports::{
        calloop::EventLoop,
        wayland_server::{Display, backend::InitError},
    },
};
use thiserror::Error;

use crate::mode::{Mode, Type};

#[derive(Debug, Error)]
pub enum CompError {
    #[error("Event Loop Error: {0}")]
    EventLoop(#[from] smithay::reexports::calloop::Error),
    #[error("Display Init Error: {0}")]
    Display(#[from] InitError),
    #[error("Winit error: {0}")]
    Winit(#[from] backend::winit::Error),
}

pub type Result<T> = std::result::Result<T, CompError>;

fn run_udev() -> Result<()> {
    let mut event_loop = EventLoop::try_new()?;

    let display = Display::new()?;
    let mut display_handle = display.handle();

    let (mut backend, winit) = backend::winit::init::<GlesRenderer>()?;
    let window_size = backend.window_size();

    let mode = Mode::new(Type::Free, window_size, 120_000);

    Ok(())
}
