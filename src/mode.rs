use smithay::utils::{Physical, Size};

#[derive(Clone)]
pub enum Type {
    Tiling,
    Free,
}

/// An output mode
///
/// A possible combination of dimensions and refresh rate for an output.
///
/// This should only describe the characteristics of the video driver,
/// not taking into account any global scaling.
pub struct Mode {
    ty: Type,
    size: Size<i32, Physical>,
    refresh_rate: i32,
}

impl Mode {
    pub fn new(ty: Type, size: Size<i32, Physical>, refresh_rate: i32) -> Self {
        Self {
            ty: ty,
            size,
            refresh_rate,
        }
    }
}
