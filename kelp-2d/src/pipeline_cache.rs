use crate::BlendMode;
use rustc_hash::FxHashMap;
use std::rc::Rc;
use wgpu::{
    BindGroupLayout, BlendComponent, BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrites, Device,
    FragmentState, Id, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology,
    PushConstantRange, RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderStages, TextureFormat,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
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
    shader_id: Option<Id>,
    blend_mode: BlendMode,
}

#[derive(Debug)]
pub(crate) struct PipelineCache {
    cache: FxHashMap<PipelineId, RenderPipeline>,
    default_vertex_shader: ShaderModule,
    default_fragment_shader: ShaderModule,
    vertex_bind_layout: BindGroupLayout,
    fragment_texture_bind_layout: Rc<BindGroupLayout>,
}

impl PipelineCache {
    pub fn new(
        default_vertex_shader: ShaderModule,
        default_fragment_shader: ShaderModule,
        vertex_bind_layout: BindGroupLayout,
        fragment_texture_bind_layout: Rc<BindGroupLayout>,
    ) -> Self {
        Self {
            cache: Default::default(),
            default_vertex_shader,
            default_fragment_shader,
            vertex_bind_layout,
            fragment_texture_bind_layout,
        }
    }

    pub fn get_pipeline(&self, id: &PipelineId) -> &RenderPipeline {
        self.cache.get(id).expect("Invalid pipeline id.")
    }

    #[allow(clippy::map_entry)]
    pub fn get_valid_pipeline_id(
        &mut self,
        device: &Device,
        shader: Option<&ShaderModule>,
        blend_mode: BlendMode,
    ) -> PipelineId {
        let id = Self::to_pipeline_id(shader, blend_mode);
        if !self.cache.contains_key(&id) {
            let pipeline = self.create_pipeline(device, shader, blend_mode);
            self.cache.insert(id, pipeline);
        }
        id
    }

    pub fn remove_pipeline(&mut self, shader: Option<&ShaderModule>, blend_mode: BlendMode) {
        _ = self.cache.remove(&Self::to_pipeline_id(shader, blend_mode))
    }

    /* private */
    fn create_pipeline(&self, device: &Device, shader: Option<&ShaderModule>, blend_mode: BlendMode) -> RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&self.vertex_bind_layout, self.fragment_texture_bind_layout.as_ref()],
            push_constant_ranges: &[CAMERA_PUSH_CONSTANT],
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None, // TODO
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
                    format: TextureFormat::Bgra8UnormSrgb, // TODO: unhardcode this! very brittle except on current test scenario
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
