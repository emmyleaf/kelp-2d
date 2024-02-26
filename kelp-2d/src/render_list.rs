use crate::{
    BlendMode, Camera, InstanceBatch, InstanceData, InstanceGPU, Kelp, KelpColor, KelpError, KelpTargetId,
    KelpTextureId,
};

/// The data for a submitted render list
#[derive(Debug)]
pub struct RenderList {
    pub target: Option<KelpTargetId>,
    pub camera: glam::Mat4,
    pub clear: Option<wgpu::Color>,
    pub instances: Vec<InstanceGPU>,
    pub batches: Vec<InstanceBatch>,
}

impl RenderList {
    pub fn new(target: Option<KelpTargetId>, camera: &Camera, clear: Option<&KelpColor>) -> Self {
        Self {
            target,
            camera: camera.into(),
            clear: clear.map(Into::into),
            instances: Vec::new(),
            batches: Vec::new(),
        }
    }

    pub fn add_instances(
        mut self,
        kelp: &Kelp,
        texture: KelpTextureId,
        smooth: bool,
        blend_mode: BlendMode,
        instance_data: &[InstanceData],
    ) -> Result<Self, KelpError> {
        // TODO: document the atlas source transform better lol
        let tex_rect = kelp.texture_cache.borrow().get_texture(texture)?.rectangle;
        self.batches.push(InstanceBatch { blend_mode, instance_count: instance_data.len() as u32 });
        self.instances.extend(instance_data.iter().map(
            |InstanceData { color, mode, source_trans, source_scale, world }| InstanceGPU {
                color: [color.x, color.y, color.z, color.w],
                mode: (*mode).into(),
                layer_smooth: [texture.layer as f32, smooth.into()],
                // TODO: DO NOT hardcode the atlas size yo
                // TODO: ohh could some of this go in the shader with push constants instead???
                source_trans: [
                    (tex_rect.min.x as f32 + source_trans.x) / 2048.0,
                    (tex_rect.min.y as f32 + source_trans.y) / 2048.0,
                ],
                source_scale: [
                    source_scale.x * tex_rect.width() as f32 / 2048.0,
                    source_scale.y * tex_rect.height() as f32 / 2048.0,
                ],
                world_col_1: [world.x.x, world.x.y],
                world_col_2: [world.y.x, world.y.y],
                world_trans: [world.z.x, world.z.y],
            },
        ));
        Ok(self)
    }
}
