use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec4};
use indexmap::IndexMap;
use interoptopus::ffi_type;
use interoptopus::lang::{
    c::{CType, PrimitiveType},
    rust::CTypeInfo,
};
use kelp_2d_imgui_wgpu::FontTexture;
use thiserror::Error;

pub type KelpMap<K, V> = IndexMap<K, V, ahash::RandomState>;

#[ffi_type]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[repr(u8)]
pub enum BlendMode {
    ALPHA = 0,
    ADDITIVE = 1,
}

#[ffi_type]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct KelpColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
#[repr(C)]
pub struct KelpTextureId {
    pub(crate) layer: u32,
    pub(crate) alloc_id: guillotiere::AllocId,
}

unsafe impl CTypeInfo for KelpTextureId {
    fn type_info() -> CType {
        CType::Primitive(PrimitiveType::U64) // we do a little mischief
    }
}

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct KelpTargetId(pub(crate) wgpu::Id<wgpu::Texture>);

unsafe impl CTypeInfo for KelpTargetId {
    fn type_info() -> CType {
        CType::Primitive(PrimitiveType::U64)
    }
}

#[ffi_type]
#[derive(Debug)]
#[repr(C)]
pub struct Camera {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub angle: f32,
    pub scale: f32,
}

#[ffi_type]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InstanceMode {
    Multiply = 1,
    Wash = 2,
    Veto = 3,
}

#[derive(Debug)]
#[repr(C)]
pub struct InstanceData {
    pub color: mint::Vector4<f32>,
    pub mode: InstanceMode,
    pub source_trans: mint::Vector2<f32>,
    pub source_scale: mint::Vector2<f32>,
    pub world: mint::RowMatrix3x2<f32>,
}

impl From<InstanceMode> for [f32; 4] {
    fn from(value: InstanceMode) -> Self {
        match value {
            InstanceMode::Multiply => [1.0, 0.0, 0.0, 0.0],
            InstanceMode::Wash => [0.0, 1.0, 0.0, 0.0],
            InstanceMode::Veto => [0.0, 0.0, 1.0, 0.0],
        }
    }
}

/// A batch of instances to be added to a render pass
#[ffi_type]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InstanceBatch {
    pub blend_mode: BlendMode,
    pub instance_count: u32,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InstanceGPU {
    pub color: [f32; 4],
    pub mode: [f32; 4],
    pub layer_smooth: [f32; 2],
    pub source_trans: [f32; 2],
    pub source_scale: [f32; 2],
    pub world_col_1: [f32; 2],
    pub world_col_2: [f32; 2],
    pub world_trans: [f32; 2],
}

#[repr(transparent)]
pub struct ImGuiConfig(pub FontTexture);

#[derive(Error, Debug)]
pub enum KelpError {
    #[error("Cannot interact with a frame before beginning one")]
    NoCurrentFrame,
    #[error("Failed to acquire next swap chain texture")]
    SwapchainError(#[from] wgpu::SurfaceError),
    #[error("Invalid texture id")]
    InvalidTextureId,
    #[error("Invalid target id")]
    InvalidTargetId,
    #[error("Invalid bind group id")]
    InvalidBindGroupId,
    #[error("Invalid pipeline id")]
    InvalidPipelineId,
    #[error("Failed to find an appropriate adapter")]
    NoAdapter,
    #[error("Failed to find an appropriate device")]
    NoDevice(#[from] wgpu::RequestDeviceError),
    #[error("Imgui renderer not initialised")]
    NoImgui,
    #[error("Imgui renderer error")]
    ImguiError(#[from] kelp_2d_imgui_wgpu::RendererError),
}

impl From<&KelpColor> for wgpu::Color {
    fn from(c: &KelpColor) -> Self {
        Self { r: c.r as f64, g: c.g as f64, b: c.b as f64, a: c.a as f64 }
    }
}

impl Camera {
    pub fn new(x: f32, y: f32, width: f32, height: f32, angle: f32, scale: f32) -> Self {
        Self { x, y, width, height, angle, scale }
    }
}

impl From<&Camera> for Mat4 {
    fn from(camera: &Camera) -> Self {
        let (sin, cos) = camera.angle.sin_cos();
        let cs = cos * camera.scale;
        let ss = sin * camera.scale;
        let x = 0.5 * camera.width - (cs * camera.x) + (ss * camera.y);
        let y = 0.5 * camera.height - (ss * camera.x) - (cs * camera.y);
        let view = Self::from_cols(
            Vec4::new(cs, ss, 0.0, 0.0),
            Vec4::new(-ss, cs, 0.0, 0.0),
            Vec4::new(0.0, 0.0, 1.0, 0.0),
            Vec4::new(x, y, 0.0, 1.0),
        );
        let projection = Self::orthographic_rh(0.0, camera.width, camera.height, 0.0, 0.0, 1.0);
        projection * view
    }
}

unsafe impl Zeroable for InstanceGPU {}
unsafe impl Pod for InstanceGPU {}
