use crate::BlendMode;
use rustc_hash::FxHashMap;
use std::rc::Rc;
use wgpu::{Device, Id, RenderPipeline, ShaderModule};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct PipelineId {
    // thanks to NPO, this is transparently zero for None, and the NonZeroU64 from inside wgpu::Id for Some
    shader_id: Option<Id>,
    blend_mode: BlendMode,
}

#[derive(Debug)]
pub(crate) struct PipelineCache {
    cache: FxHashMap<PipelineId, RenderPipeline>,
    default_vertex_shader: ShaderModule,
    default_fragment_shader: ShaderModule,
}

impl PipelineCache {
    pub fn new(default_vertex_shader: ShaderModule, default_fragment_shader: ShaderModule) -> Self {
        Self {
            cache: Default::default(),
            default_vertex_shader,
            default_fragment_shader,
        }
    }

    pub fn get_pipeline(
        &mut self,
        device: &Device,
        shader: Option<&ShaderModule>,
        blend_mode: BlendMode,
    ) -> &RenderPipeline {
        let id = Self::to_pipeline_id(shader, blend_mode);
        let contains = self.cache.contains_key(&id);
        if !contains {
            let bind_group = self.create_pipeline(device, shader, blend_mode);
            self.cache.insert(id, bind_group);
        }
        self.cache.get(&id).expect("Failed to get or create pipeline.")
    }

    pub fn remove_pipeline(&mut self, shader: Option<&ShaderModule>, blend_mode: BlendMode) {
        _ = self.cache.remove(&Self::to_pipeline_id(shader, blend_mode))
    }

    /* private */
    fn create_pipeline(
        &mut self,
        device: &Device,
        shader: Option<&ShaderModule>,
        blend_mode: BlendMode,
    ) -> RenderPipeline {
        todo!()
    }

    #[inline]
    fn to_pipeline_id(shader: Option<&ShaderModule>, blend_mode: BlendMode) -> PipelineId {
        PipelineId { shader_id: shader.map(ShaderModule::global_id), blend_mode }
    }
}
