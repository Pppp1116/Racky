use smithay::output::{Mode, Output, Scale};
use smithay::utils::{Size, Transform};
use std::fs::OpenOptions;
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::window_maneger::Layout;

const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub width: i32,
    pub height: i32,
    pub refresh_hz: f64,
    pub scale: f64,
    pub transform: OutputTransform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputTransform {
    Normal,
    Rotate90,
    Rotate180,
    Rotate270,
    Flipped,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowLayout {
    Scrolling,
    Floating,
}

impl From<WindowLayout> for Layout {
    fn from(value: WindowLayout) -> Self {
        match value {
            WindowLayout::Floating => Self::Floating,
            WindowLayout::Scrolling => Self::Scrooling {
                scrool_offset: 0,
                windows: Vec::new(),
            },
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub monitor: Option<OutputConfig>,
    pub default_layout: Option<WindowLayout>,
    pub transform: Option<OutputTransform>,
    pub number_of_workspaces: Option<u16>,
}

impl Configuration {
    pub fn load_settings() -> Option<Self> {
        let config_dir = dirs::config_dir()?.join(APP_NAME).join("config.kdl");

        let contents = std::fs::read_to_string(config_dir).ok()?;
        serde_kdl2::from_str(&contents).ok()
    }
}

impl From<OutputConfig> for Mode {
    fn from(value: OutputConfig) -> Self {
        let size = Size::new(value.width, value.height);

        Self {
            size,
            refresh: (value.refresh_hz * 1000.0).round() as i32,
        }
    }
}

impl From<OutputTransform> for Transform {
    fn from(value: OutputTransform) -> Self {
        match value {
            OutputTransform::Flipped => Self::Flipped,
            OutputTransform::Normal => Self::Normal,
            OutputTransform::Rotate180 => Self::_180,
            OutputTransform::Rotate270 => Self::_270,
            OutputTransform::Rotate90 => Self::_90,
        }
    }
}
