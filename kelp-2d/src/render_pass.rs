use crate::{InstanceData, InstanceGPU, InstanceGroup, Kelp, KelpTexture};
use glam::{Mat4, Vec4};
use wgpu::{
    Color, CommandEncoder, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, ShaderStages,
    SurfaceTexture, TextureView,
};

#[derive(Debug)]
pub struct KelpRenderPass<'a> {
    pub kelp: &'a mut Kelp,
    pub camera: Mat4,
    pub clear: Option<Vec4>,
    pub surface: SurfaceTexture,
    pub view: TextureView,
    pub encoder: CommandEncoder,
    pub instances: Vec<InstanceGPU>,
    pub groups: Vec<InstanceGroup>,
}

impl<'a> KelpRenderPass<'a> {
    pub fn add_instances(&mut self, texture: &KelpTexture, smooth: bool, instance_data: &[InstanceData]) {
        let prev_count = self.instances.len() as u32;
        let range = (prev_count)..(prev_count + instance_data.len() as u32);
        let bind_group = self.kelp.texture_binding_cache.get_texture_bind_group(&self.kelp.device, texture, smooth);
        self.instances.extend(instance_data.iter().map(InstanceGPU::from));
        self.groups.push(InstanceGroup { bind_group, range });
    }

    pub fn finish(mut self) {
        if self.groups.is_empty() || self.instances.is_empty() {
            return;
        }

        let kelp = self.kelp;
        kelp.update_buffer(&kelp.instance_buffer, self.instances.as_slice());

        {
            let load = self.clear.map_or(LoadOp::Load, |v| {
                LoadOp::Clear(Color { r: v.x as f64, g: v.y as f64, b: v.z as f64, a: v.w as f64 })
            });
            let mut wgpu_pass = self.encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.view,
                    resolve_target: None,
                    ops: Operations { load, store: true },
                })],
                ..Default::default()
            });

            wgpu_pass.set_pipeline(&kelp.pipeline);
            wgpu_pass.set_push_constants(ShaderStages::VERTEX, 0, bytemuck::bytes_of(self.camera.as_ref()));
            wgpu_pass.set_vertex_buffer(0, kelp.vertex_buffer.slice(..));
            wgpu_pass.set_bind_group(0, &kelp.vertex_bind_group, &[]);

            for group in self.groups.iter() {
                wgpu_pass.set_bind_group(1, group.bind_group.as_ref(), &[]);
                wgpu_pass.draw(0..4, group.range.clone());
            }
        }

        kelp.queue.submit(Some(self.encoder.finish()));
        self.surface.present();
    }
}
