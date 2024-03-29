use crate::{BlendMode, KelpError, KelpMap};
use wgpu::{
    BindGroupLayout, BlendComponent, BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrites, Device,
    FragmentState, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, PushConstantRange,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderStages, TextureFormat, VertexAttribute,
    VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

const BLEND_COMPONENT_ADDITIVE: BlendComponent = BlendComponent {
    src_factor: BlendFactor::SrcAlpha,
    dst_factor: BlendFactor::One,
    operation: BlendOperation::Add,
};
const BLEND_STATE_ADDITIVE: BlendState = BlendState {
    color: BLEND_COMPONENT_ADDITIVE,
    alpha: BLEND_COMPONENT_ADDITIVE,
};
const CAMERA_PUSH_CONSTANT: PushConstantRange = PushConstantRange { stages: ShaderStages::VERTEX, range: 0..64 };

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct PipelineId {
    // thanks to NPO, this is transparently zero for None, and the NonZeroU64 from inside wgpu::Id for Some
    shader_id: Option<wgpu::Id<ShaderModule>>,
    blend_mode: BlendMode,
}

#[derive(Debug)]
pub(crate) struct PipelineCache {
    cache: KelpMap<PipelineId, RenderPipeline>,
    default_vertex_shader: ShaderModule,
    default_fragment_shader: ShaderModule,
    vertex_bind_layout: BindGroupLayout,
    surface_texture_format: TextureFormat,
}

impl PipelineCache {
    pub fn new(
        default_vertex_shader: ShaderModule,
        default_fragment_shader: ShaderModule,
        vertex_bind_layout: BindGroupLayout,
        surface_texture_format: TextureFormat,
    ) -> Self {
        Self {
            cache: Default::default(),
            default_vertex_shader,
            default_fragment_shader,
            vertex_bind_layout,
            surface_texture_format,
        }
    }

    pub fn ensure_pipeline(
        &mut self,
        device: &Device,
        shader: Option<&ShaderModule>,
        blend_mode: BlendMode,
    ) -> Result<(), KelpError> {
        let id = Self::to_pipeline_id(shader, blend_mode);
        if !self.cache.contains_key(&id) {
            let pipeline = self.create_pipeline(device, shader, blend_mode);
            self.cache.insert(id, pipeline);
        }
        Ok(())
    }

    pub fn get_pipeline_index(&self, shader: Option<&ShaderModule>, blend_mode: BlendMode) -> Result<usize, KelpError> {
        let id = Self::to_pipeline_id(shader, blend_mode);
        self.cache.get_index_of(&id).ok_or(KelpError::InvalidPipelineId)
    }

    pub fn get_pipeline(&self, index: usize) -> Result<&RenderPipeline, KelpError> {
        self.cache.get_index(index).map(|t| t.1).ok_or(KelpError::InvalidPipelineId)
    }

    /* private */
    fn create_pipeline(&self, device: &Device, shader: Option<&ShaderModule>, blend_mode: BlendMode) -> RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&self.vertex_bind_layout],
            push_constant_ranges: &[CAMERA_PUSH_CONSTANT],
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &self.default_vertex_shader,
                entry_point: "main",
                buffers: &[VertexBufferLayout {
                    array_stride: 8,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[VertexAttribute {
                        format: VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            fragment: Some(FragmentState {
                module: &self.default_fragment_shader,
                entry_point: "main",
                targets: &[Some(ColorTargetState {
                    blend: Some(match blend_mode {
                        BlendMode::ALPHA => BlendState::ALPHA_BLENDING,
                        BlendMode::ADDITIVE => BLEND_STATE_ADDITIVE,
                    }),
                    format: self.surface_texture_format,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        })
    }

    #[inline]
    fn to_pipeline_id(shader: Option<&ShaderModule>, blend_mode: BlendMode) -> PipelineId {
        PipelineId { shader_id: shader.map(ShaderModule::global_id), blend_mode }
    }
}
