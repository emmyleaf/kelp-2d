use crate::{InstanceData, InstanceGPU, InstanceGroup, KelpTexture};
use glam::{Mat4, Vec4};
use wgpu::{CommandEncoder, SurfaceTexture, TextureView};

#[derive(Debug)]
pub struct KelpRenderPass<'a> {
    pub camera: Mat4,
    pub clear: Option<Vec4>,
    pub surface: SurfaceTexture,
    pub view: TextureView,
    pub encoder: CommandEncoder,
    pub instances: Vec<InstanceGPU>,
    pub groups: Vec<InstanceGroup<'a>>,
}

impl<'a> KelpRenderPass<'a> {
    pub fn add_instances(&mut self, texture: &'a KelpTexture, instance_data: &[InstanceData]) {
        let prev_count = self.instances.len() as u32;
        let range = (prev_count)..(prev_count + instance_data.len() as u32);
        self.instances.extend(instance_data.iter().map(InstanceGPU::from));
        self.groups.push(InstanceGroup { bind_group: &texture.bind_group, range });
    }
}
