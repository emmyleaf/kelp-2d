use crate::{BlendMode, Camera, InstanceBatch, InstanceData, InstanceGPU, KelpColor, KelpTextureId};
use glam::Mat4;
use wgpu::Color;

/// The data for a submitted render list
pub struct RenderList {
    pub target: Option<KelpTextureId>,
    pub camera: Mat4,
    pub clear: Option<Color>,
    pub instances: Vec<InstanceGPU>,
    pub batches: Vec<InstanceBatch>,
}

impl RenderList {
    pub fn new(target: Option<KelpTextureId>, camera: &Camera, clear: Option<&KelpColor>) -> Self {
        Self {
            target,
            camera: Mat4::from(camera),
            clear: clear.map(Into::into),
            instances: Vec::new(),
            batches: Vec::new(),
        }
    }

    pub fn add_instances(
        mut self,
        texture: KelpTextureId,
        smooth: bool,
        blend_mode: BlendMode,
        instance_data: &[InstanceData],
    ) -> Self {
        self.batches.push(InstanceBatch {
            texture,
            smooth,
            blend_mode,
            instance_count: instance_data.len() as u32,
        });
        self.instances.extend(instance_data.iter().map(InstanceGPU::from));
        self
    }
}
