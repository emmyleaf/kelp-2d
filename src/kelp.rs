use crate::{KelpTexture, SurfaceFrame};
use naga::FastHashMap;
use pollster::FutureExt;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawWindowHandle};
use std::{borrow::Cow, num::NonZeroU64};
use wgpu::{util::DeviceExt, InstanceDescriptor, RenderPassDescriptor};

#[derive(Debug)]
pub struct VertexGroup {
    pub camera_buffer: wgpu::Buffer,
    pub instance_buffer: wgpu::Buffer,
    pub bind: wgpu::BindGroup,
}

#[derive(Debug)]
pub struct Kelp {
    pub window_handle: RawWindowHandle,
    pub window_surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub pipeline: wgpu::RenderPipeline,
    pub config: wgpu::SurfaceConfiguration,
    pub point_sampler: wgpu::Sampler,
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_group: VertexGroup,
    pub fragment_bind_layout: wgpu::BindGroupLayout,
}

impl Kelp {
    pub fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(window: W, width: u32, height: u32) -> Kelp {
        let instance =
            wgpu::Instance::new(InstanceDescriptor { backends: wgpu::Backends::PRIMARY, ..Default::default() });
        let surface = unsafe { instance.create_surface(&window).unwrap() };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions { compatible_surface: Some(&surface), ..Default::default() })
            .block_on()
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    limits: wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits()),
                },
                None,
            )
            .block_on()
            .expect("Failed to create device");

        // Configure surface
        let config = wgpu::SurfaceConfiguration {
            present_mode: wgpu::PresentMode::Fifo,
            ..surface.get_default_config(&adapter, width, height).unwrap()
        };

        surface.configure(&device, &config);

        // Load the shaders from disk
        let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: Cow::Borrowed(include_str!("../shaders/shader.vert")),
                stage: naga::ShaderStage::Vertex,
                defines: FastHashMap::default(),
            },
        });

        let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: Cow::Borrowed(include_str!("../shaders/shader.frag")),
                stage: naga::ShaderStage::Fragment,
                defines: FastHashMap::default(),
            },
        });

        // Create layouts for vertex shader bind group
        let camera_buffer_layout = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(64),
            },
            count: None,
        };

        let instance_buffer_layout = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(16 + 64 + 64),
            },
            count: None,
        };

        let vertex_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Vertex Bind Group Layout"),
            entries: &[camera_buffer_layout, instance_buffer_layout],
        });

        // Create layouts for fragment shader bind group
        let texture_bind_layout = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };

        let sampler_bind_layout = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };

        let fragment_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fragment Bind Group Layout"),
            entries: &[texture_bind_layout, sampler_bind_layout],
        });

        // Create main render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Main Render Pipeline Layout"),
            bind_group_layouts: &[&vertex_bind_layout, &fragment_bind_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Main Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 8,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x2,
                        offset: 0,
                        shader_location: 0,
                    }],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    ..config.format.into()
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..wgpu::PrimitiveState::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create buffers
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            // Vertices (0, 0), (1, 0), (0, 1), (1, 1)
            contents: bytemuck::bytes_of(&[0_f32, 0_f32, 1_f32, 0_f32, 0_f32, 1_f32, 1_f32, 1_f32]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Buffer"),
            size: device.limits().min_uniform_buffer_offset_alignment.into(),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: 4 << 20, // 4MB
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create point sampler
        let point_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Point Sampler"),
            ..wgpu::SamplerDescriptor::default()
        });

        // Create vertex bind group
        let vertex_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Vertex Bind Group"),
            layout: &vertex_bind_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: camera_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: instance_buffer.as_entire_binding() },
            ],
        });

        Self {
            window_handle: window.raw_window_handle(),
            window_surface: surface,
            device,
            queue,
            pipeline,
            config,
            point_sampler,
            vertex_buffer,
            vertex_group: VertexGroup { camera_buffer, instance_buffer, bind: vertex_bind_group },
            fragment_bind_layout,
        }
    }

    pub fn begin_surface_frame(&self) -> SurfaceFrame {
        let surface = self.window_surface.get_current_texture().expect("Failed to acquire next swap chain texture");
        let view = surface.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        SurfaceFrame {
            surface,
            view,
            encoder,
            instances: Vec::new(),
            groups: Vec::new(),
        }
    }

    pub fn create_texture_with_data(&mut self, width: u32, height: u32, data: &[u8]) -> KelpTexture {
        let texture = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            data,
        );
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.fragment_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture.create_view(&wgpu::TextureViewDescriptor::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.point_sampler),
                },
            ],
        });
        KelpTexture { texture, bind_group }
    }

    pub fn end_surface_frame(&self, mut frame: SurfaceFrame) {
        if frame.groups.len() == 0 || frame.instances.len() == 0 {
            return;
        }

        self.update_buffer(&self.vertex_group.instance_buffer, frame.instances.as_slice());

        {
            let mut pass = frame.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.3, g: 0.1, b: 0.2, a: 1.0 }),
                        store: true,
                    },
                })],
                ..RenderPassDescriptor::default()
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_bind_group(0, &self.vertex_group.bind, &[]);

            for group in frame.groups {
                pass.set_bind_group(1, group.bind_group, &[]);
                pass.draw(0..4, group.range);
            }
        }

        self.queue.submit(Some(frame.encoder.finish()));
        frame.surface.present();
    }

    pub fn set_surface_size(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.window_surface.configure(&self.device, &self.config);
    }

    pub fn update_buffer<T: bytemuck::NoUninit>(&self, buffer: &wgpu::Buffer, data: &[T]) {
        let bytes = bytemuck::cast_slice(data);
        self.queue.write_buffer(buffer, 0, bytes);
    }
}
