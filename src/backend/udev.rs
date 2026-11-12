use std::os::unix::raw::dev_t;

use constructor_derive::New;
use smithay::backend::allocator::gbm::GbmAllocator;
use smithay::backend::drm::exporter::gbm::GbmFramebufferExporter;
use smithay::backend::drm::output::DrmOutput;
use smithay::backend::drm::{DrmDeviceFd, DrmNode};
use smithay::backend::input::{InputEvent, PointerAxisEvent};
use smithay::backend::libinput::{LibinputInputBackend, PointerScrollAxis};
use smithay::backend::renderer::gles::GlesRenderer;
use smithay::backend::renderer::multigpu::GpuManager;
use smithay::backend::renderer::multigpu::gbm::GbmGlesBackend;
use smithay::backend::session::libseat::LibSeatSession;
use smithay::desktop::utils::OutputPresentationFeedback;
use smithay::output::Output;
use smithay::reexports::calloop::EventSource;
use smithay::reexports::input::Device as LibInputDevice;

pub type GbmOutput = DrmOutput<
    GbmAllocator<DrmDeviceFd>,
    GbmFramebufferExporter<DrmDeviceFd>,
    Option<OutputPresentationFeedback>,
    DrmDeviceFd,
>;

pub type GpuCollection = GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>;

#[derive(New)]
pub struct OutputState {
    pub output: Output,
    pub drm_output: GbmOutput,
}
#[derive(New)]
pub struct GpuState {
    pub primary: DrmNode,
    pub gpus: GpuCollection,
}

pub trait Backend: 'static {
    fn remove_device(&mut self, device: &LibInputDevice);
    fn add_device(&mut self, device: LibInputDevice);
}

#[derive(New)]
pub struct UdevBack {
    pub session: LibSeatSession,
    output: Vec<OutputState>,
    gpu: GpuState,
    keybords: Vec<smithay::reexports::input::Device>,
}

impl Backend for UdevBack {
    fn remove_device(&mut self, device: &LibInputDevice) {
        if let Some(index) = self.keybords.iter().position(|target| target == device) {
            self.keybords.swap_remove(index);
        };
    }
    fn add_device(&mut self, device: LibInputDevice) {
        self.keybords.push(device);
    }
}
