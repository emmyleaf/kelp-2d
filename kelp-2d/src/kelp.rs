use crate::{Camera, KelpRenderPass, KelpTexture};
use bytemuck::NoUninit;
use glam::Vec4;
use naga::{FastHashMap, ShaderStage};
use pollster::FutureExt;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle};
use std::{borrow::Cow, num::NonZeroU64};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendState, Buffer, BufferBindingType, BufferDescriptor,
    BufferUsages, Color, ColorTargetState, CommandEncoderDescriptor, Device, DeviceDescriptor, Extent3d, Features,
    FragmentState, Instance, InstanceDescriptor, Limits, LoadOp, MultisampleState, Operations,
    PipelineLayoutDescriptor, PresentMode, PrimitiveState, PrimitiveTopology, PushConstantRange, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions,
    Sampler, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, Surface,
    SurfaceConfiguration, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureViewDescriptor, TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
    VertexStepMode,
};

#[derive(Debug)]
pub struct Kelp {
    pub window_handle: RawWindowHandle,
    pub window_surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub pipeline: RenderPipeline,
    pub config: SurfaceConfiguration,
    pub point_sampler: Sampler,
    pub vertex_buffer: Buffer,
    pub instance_buffer: Buffer,
    pub vertex_bind_group: BindGroup,
    pub fragment_bind_layout: BindGroupLayout,
}

impl Kelp {
    pub fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(window: W, width: u32, height: u32) -> Kelp {
        let instance = Instance::new(InstanceDescriptor { backends: Backends::PRIMARY, ..Default::default() });
        let window_surface = unsafe { instance.create_surface(&window).unwrap() };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&window_surface),
                ..Default::default()
            })
            .block_on()
            .expect("Failed to find an appropriate adapter");

        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
        let mut limits = Limits::downlevel_defaults().using_resolution(adapter.limits());
        limits.max_push_constant_size = 128;

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor { label: None, features: Features::PUSH_CONSTANTS, limits }, None)
            .block_on()
            .expect("Failed to create device");

        // Configure surface
        let config = SurfaceConfiguration {
            present_mode: PresentMode::Fifo,
            ..window_surface.get_default_config(&adapter, width, height).unwrap()
        };

        window_surface.configure(&device, &config);

        // Load the shaders from disk
        let vertex_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Glsl {
                shader: Cow::Borrowed(include_str!("../shaders/shader.vert")),
                stage: ShaderStage::Vertex,
                defines: FastHashMap::default(),
            },
        });

        let fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Glsl {
                shader: Cow::Borrowed(include_str!("../shaders/shader.frag")),
                stage: ShaderStage::Fragment,
                defines: FastHashMap::default(),
            },
        });

        // Create layouts for vertex shader bind group
        let camera_push_constant = PushConstantRange { stages: ShaderStages::VERTEX, range: 0..64 };

        let instance_buffer_layout = BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(16 + 64 + 64),
            },
            count: None,
        };

        let vertex_bind_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Vertex Bind Group Layout"),
            entries: &[instance_buffer_layout],
        });

        // Create layouts for fragment shader bind group
        let texture_bind_layout = BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };

        let sampler_bind_layout = BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        };

        let fragment_bind_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Fragment Bind Group Layout"),
            entries: &[texture_bind_layout, sampler_bind_layout],
        });

        // Create main render pipeline
        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Main Render Pipeline Layout"),
            bind_group_layouts: &[&vertex_bind_layout, &fragment_bind_layout],
            push_constant_ranges: &[camera_push_constant],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Main Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &vertex_shader,
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
                module: &fragment_shader,
                entry_point: "main",
                targets: &[Some(ColorTargetState {
                    blend: Some(BlendState::ALPHA_BLENDING),
                    ..config.format.into()
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        // Create buffers
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            // Vertices (0, 0), (1, 0), (0, 1), (1, 1)
            contents: bytemuck::bytes_of(&[0_f32, 0_f32, 1_f32, 0_f32, 0_f32, 1_f32, 1_f32, 1_f32]),
            usage: BufferUsages::VERTEX,
        });

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Instance Buffer"),
            size: 4 << 20, // 4MB
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create point sampler
        let point_sampler =
            device.create_sampler(&SamplerDescriptor { label: Some("Point Sampler"), ..SamplerDescriptor::default() });

        // Create vertex bind group
        let vertex_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Vertex Bind Group"),
            layout: &vertex_bind_layout,
            entries: &[BindGroupEntry { binding: 0, resource: instance_buffer.as_entire_binding() }],
        });

        Self {
            window_handle: window.raw_window_handle(),
            window_surface,
            device,
            queue,
            pipeline,
            config,
            point_sampler,
            vertex_buffer,
            instance_buffer,
            vertex_bind_group,
            fragment_bind_layout,
        }
    }

    pub fn begin_render_pass(&self, camera: &Camera, clear: Option<Vec4>) -> KelpRenderPass {
        let surface = self.window_surface.get_current_texture().expect("Failed to acquire next swap chain texture");
        let view = surface.texture.create_view(&TextureViewDescriptor::default());
        let encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        KelpRenderPass {
            camera: camera.into(),
            clear,
            surface,
            view,
            encoder,
            // TODO: at some point we could reuse these rather than allocating each time
            instances: Vec::new(),
            groups: Vec::new(),
        }
    }

    pub fn create_texture_with_data(&mut self, width: u32, height: u32, data: &[u8]) -> KelpTexture {
        let texture = self.device.create_texture_with_data(
            &self.queue,
            &TextureDescriptor {
                label: None,
                size: Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
            data,
        );
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.fragment_bind_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture.create_view(&TextureViewDescriptor::default())),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.point_sampler),
                },
            ],
        });
        KelpTexture { texture, bind_group }
    }

    pub fn end_render_pass(&self, mut render: KelpRenderPass) {
        if render.groups.is_empty() || render.instances.is_empty() {
            return;
        }

        self.update_buffer(&self.instance_buffer, render.instances.as_slice());

        {
            let load = render.clear.map_or(LoadOp::Load, |v| {
                LoadOp::Clear(Color { r: v.x as f64, g: v.y as f64, b: v.z as f64, a: v.w as f64 })
            });
            let mut wgpu_pass = render.encoder.begin_render_pass(&RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &render.view,
                    resolve_target: None,
                    ops: Operations { load, store: true },
                })],
                ..Default::default()
            });

            wgpu_pass.set_pipeline(&self.pipeline);
            wgpu_pass.set_push_constants(ShaderStages::VERTEX, 0, bytemuck::bytes_of(render.camera.as_ref()));
            wgpu_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            wgpu_pass.set_bind_group(0, &self.vertex_bind_group, &[]);

            for group in render.groups {
                wgpu_pass.set_bind_group(1, group.bind_group, &[]);
                wgpu_pass.draw(0..4, group.range);
            }
        }

        self.queue.submit(Some(render.encoder.finish()));
        render.surface.present();
    }

    pub fn set_surface_size(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.window_surface.configure(&self.device, &self.config);
    }

    pub fn update_buffer<T: NoUninit>(&self, buffer: &Buffer, data: &[T]) {
        let bytes = bytemuck::cast_slice(data);
        self.queue.write_buffer(buffer, 0, bytes);
    }
}
