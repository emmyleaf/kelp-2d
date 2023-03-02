use interoptopus::{ffi_type, patterns::slice::FFISlice};
use kelp_2d::{BlendMode, InstanceData, KelpTexture};

#[ffi_type]
#[repr(C)]
pub struct InstanceBatch<'a> {
    pub texture: &'a KelpTexture,
    pub smooth: bool,
    pub blend_mode: BlendMode,
    pub instances: FFISlice<'a, InstanceData>,
}
