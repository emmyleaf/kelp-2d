use crate::KelpTexture;
use bytemuck::{Pod, Zeroable};
use std::ops::Range;

#[derive(Debug)]
#[repr(C)]
pub struct SourceTransform {
    pub source_x: f32,
    pub source_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
}

#[derive(Debug)]
#[repr(C)]
pub struct WorldTransform {
    pub render_x: f32,
    pub render_y: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub rotation: f32,
    pub origin_x: f32,
    pub origin_y: f32,
}

#[derive(Debug)]
#[repr(C)]
pub struct InstanceData {
    pub color: [f32; 4],
    pub source: SourceTransform,
    pub world: WorldTransform,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct InstanceGPU {
    pub color: [f32; 4],
    pub source: glam::Mat4,
    pub world: glam::Mat4,
}

#[derive(Debug)]
pub struct InstanceGroup<'a> {
    pub bind_group: &'a wgpu::BindGroup,
    pub range: Range<u32>,
}

#[derive(Debug)]
pub struct SurfaceFrame<'a> {
    pub camera: glam::Mat4,
    pub surface: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
    pub encoder: wgpu::CommandEncoder,
    pub instances: Vec<InstanceGPU>,
    pub groups: Vec<InstanceGroup<'a>>,
}

impl Default for SourceTransform {
    fn default() -> Self {
        Self { source_x: 0.0, source_y: 0.0, scale_x: 1.0, scale_y: 1.0 }
    }
}

impl Default for WorldTransform {
    fn default() -> Self {
        Self {
            render_x: 0.0,
            render_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            origin_x: 0.0,
            origin_y: 0.0,
        }
    }
}

impl From<&SourceTransform> for glam::Mat4 {
    fn from(transform: &SourceTransform) -> Self {
        Self::from_cols(
            glam::vec4(transform.scale_x, 0.0, 0.0, 0.0),
            glam::vec4(0.0, transform.scale_y, 0.0, 0.0),
            glam::vec4(0.0, 0.0, 1.0, 0.0),
            glam::vec4(transform.source_x, transform.source_y, 0.0, 1.0),
        )
    }
}

impl From<&WorldTransform> for glam::Mat4 {
    fn from(transform: &WorldTransform) -> Self {
        let (sin, cos) = transform.rotation.sin_cos();
        let a = cos * transform.scale_x;
        let b = -sin * transform.scale_y;
        let c = sin * transform.scale_x;
        let d = cos * transform.scale_y;
        let x = transform.render_x + transform.origin_x - (a * transform.origin_x) - (b * transform.origin_y);
        let y = transform.render_y + transform.origin_y - (c * transform.origin_x) - (d * transform.origin_y);
        Self::from_cols(
            glam::vec4(a, c, 0.0, 0.0),
            glam::vec4(b, d, 0.0, 0.0),
            glam::vec4(0.0, 0.0, 1.0, 0.0),
            glam::vec4(x, y, 0.0, 1.0),
        )
    }
}

impl From<&InstanceData> for InstanceGPU {
    fn from(data: &InstanceData) -> Self {
        Self {
            color: data.color,
            source: glam::Mat4::from(&data.source),
            world: glam::Mat4::from(&data.world),
        }
    }
}

unsafe impl Zeroable for InstanceGPU {}
unsafe impl Pod for InstanceGPU {}

impl<'a> SurfaceFrame<'a> {
    pub fn add_instances(&mut self, texture: &'a KelpTexture, instance_data: &[InstanceData]) {
        let prev_count = self.instances.len() as u32;
        let range = (prev_count)..(prev_count + instance_data.len() as u32);
        self.instances.extend(instance_data.iter().map(InstanceGPU::from));
        self.groups.push(InstanceGroup { bind_group: &texture.bind_group, range });
    }
}
