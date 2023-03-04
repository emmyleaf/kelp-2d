use crate::{
    BlendMode, InstanceData, InstanceGPU, KelpColor, KelpResources, KelpTexture, PipelineId, TextureBindGroupId,
};
use glam::Mat4;
use std::{ops::Range, rc::Rc};
use wgpu::{
    Color, CommandEncoder, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, ShaderStages,
    TextureView,
};

#[derive(Debug)]
struct InstanceGroup {
    range: Range<u32>,
    bind_group_id: TextureBindGroupId,
    pipeline_id: PipelineId,
}

#[derive(Debug)]
pub struct KelpRenderPass {
    camera: Mat4,
    clear: Option<KelpColor>,
    target_view: TextureView,
    resources: Rc<KelpResources>,
    instances: Vec<InstanceGPU>,
    groups: Vec<InstanceGroup>,
}

impl KelpRenderPass {
    pub(crate) fn new(
        camera: Mat4,
        clear: Option<KelpColor>,
        target_view: TextureView,
        resources: Rc<KelpResources>,
    ) -> Self {
        Self {
            camera,
            clear,
            target_view,
            resources,
            instances: Vec::new(),
            groups: Vec::new(),
        }
    }

    pub fn add_instances(
        &mut self,
        texture: &KelpTexture,
        smooth: bool,
        blend_mode: BlendMode,
        instance_data: &[InstanceData],
    ) {
        let prev_count = self.instances.len() as u32;
        let range = (prev_count)..(prev_count + instance_data.len() as u32);

        let mut texture_bind_group_cache = self.resources.texture_bind_group_cache.borrow_mut();
        let bind_group_id = texture_bind_group_cache.get_valid_bind_group_id(&self.resources.device, texture, smooth);

        let mut pipeline_cache = self.resources.pipeline_cache.borrow_mut();
        let pipeline_id = pipeline_cache.get_valid_pipeline_id(&self.resources.device, None, blend_mode);

        self.instances.extend(instance_data.iter().map(InstanceGPU::from));
        self.groups.push(InstanceGroup { range, bind_group_id, pipeline_id });
    }

    pub(crate) fn finish(self, encoder: &mut CommandEncoder) {
        if self.groups.is_empty() || self.instances.is_empty() {
            return;
        }

        let instances_bytes = bytemuck::cast_slice(self.instances.as_slice());
        self.resources.queue.write_buffer(&self.resources.instance_buffer, 0, instances_bytes);

        {
            let texture_bind_group_cache = self.resources.texture_bind_group_cache.borrow();
            let pipeline_cache = self.resources.pipeline_cache.borrow();

            let load = self.clear.map_or(LoadOp::Load, |c| {
                LoadOp::Clear(Color { r: c.r as f64, g: c.g as f64, b: c.b as f64, a: c.a as f64 })
            });
            let mut wgpu_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.target_view,
                    resolve_target: None,
                    ops: Operations { load, store: true },
                })],
                ..Default::default()
            });

            // Always set the pipeline first time around
            let mut current_pipeline_id = self.groups[0].pipeline_id;
            wgpu_pass.set_pipeline(pipeline_cache.get_pipeline(&current_pipeline_id));
            wgpu_pass.set_push_constants(ShaderStages::VERTEX, 0, bytemuck::cast_slice(self.camera.as_ref()));
            wgpu_pass.set_vertex_buffer(0, self.resources.vertex_buffer.slice(..));
            wgpu_pass.set_bind_group(0, &self.resources.vertex_bind_group, &[]);

            for group in self.groups {
                // Only set if the pipeline id changes
                if current_pipeline_id != group.pipeline_id {
                    current_pipeline_id = group.pipeline_id;
                    wgpu_pass.set_pipeline(pipeline_cache.get_pipeline(&current_pipeline_id));
                    wgpu_pass.set_push_constants(ShaderStages::VERTEX, 0, bytemuck::cast_slice(self.camera.as_ref()));
                    wgpu_pass.set_vertex_buffer(0, self.resources.vertex_buffer.slice(..));
                    wgpu_pass.set_bind_group(0, &self.resources.vertex_bind_group, &[]);
                }

                let bind_group_1 = texture_bind_group_cache.get_bind_group(&group.bind_group_id);
                wgpu_pass.set_bind_group(1, bind_group_1, &[]);
                wgpu_pass.draw(0..4, group.range);
            }
        }
    }
}
