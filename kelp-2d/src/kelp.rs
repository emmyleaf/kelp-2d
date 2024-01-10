use crate::{ImGuiConfig, KelpError, KelpTextureId, PerFrame, PipelineCache, RenderBatchData, TextureCache};
use bytemuck::NoUninit;
use kelp_2d_imgui_wgpu::ImGuiRenderer;
use naga::ShaderStage;
use pollster::FutureExt;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::{borrow::Cow, num::NonZeroU64, rc::Rc};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, CommandBuffer, CommandEncoderDescriptor,
    Device, DeviceDescriptor, Extent3d, Features, FilterMode, Instance, InstanceDescriptor, Limits, LoadOp, Operations,
    PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, SamplerBindingType,
    SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, Surface, SurfaceConfiguration,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureViewDimension,
};

#[derive(Debug)]
pub struct Kelp {
    pub(crate) window_surface: Surface,
    pub(crate) window_surface_config: SurfaceConfiguration,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) instance_buffer: Buffer,
    pub(crate) vertex_bind_group: BindGroup,
    pub(crate) texture_cache: TextureCache,
    pub(crate) pipeline_cache: PipelineCache,
    pub(crate) imgui_renderer: Option<ImGuiRenderer>,
    pub(crate) per_frame: Option<PerFrame>,
}

impl Kelp {
    pub fn new<W: HasRawWindowHandle + HasRawDisplayHandle>(
        window: W,
        width: u32,
        height: u32,
        imgui_config: Option<&mut ImGuiConfig>,
    ) -> Result<Kelp, KelpError> {
        let instance = Instance::new(InstanceDescriptor { backends: Backends::PRIMARY, ..Default::default() });
        let window_surface = unsafe { instance.create_surface(&window).unwrap() };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&window_surface),
                ..Default::default()
            })
            .block_on()
            .ok_or(KelpError::NoAdapter)?;

        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
        let mut limits = Limits::downlevel_defaults().using_resolution(adapter.limits());
        limits.max_push_constant_size = 128;

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor { label: None, features: Features::PUSH_CONSTANTS, limits }, None)
            .block_on()?;

        // Configure surface
        let window_surface_config = SurfaceConfiguration {
            present_mode: PresentMode::Fifo,
            ..window_surface.get_default_config(&adapter, width, height).unwrap()
        };

        window_surface.configure(&device, &window_surface_config);

        // Load the default shaders from disk
        let default_vertex_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Glsl {
                shader: Cow::Borrowed(include_str!("../shaders/shader.vert")),
                stage: ShaderStage::Vertex,
                defines: Default::default(),
            },
        });

        let default_fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Glsl {
                shader: Cow::Borrowed(include_str!("../shaders/shader.frag")),
                stage: ShaderStage::Fragment,
                defines: Default::default(),
            },
        });

        // Create layouts for vertex shader bind group
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

        // Create layouts for fragment shader texture bind group
        let texture_bind_entry = BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };

        let sampler_bind_entry = BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Sampler(SamplerBindingType::Filtering),
            count: None,
        };

        let fragment_texture_bind_layout = Rc::new(device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Fragment Texture Bind Group Layout"),
            entries: &[texture_bind_entry, sampler_bind_entry],
        }));

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
            device.create_sampler(&SamplerDescriptor { label: Some("Point Sampler"), ..Default::default() });

        // Create linear sampler
        let linear_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Linear Sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        // Create vertex bind group
        let vertex_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Vertex Bind Group"),
            layout: &vertex_bind_layout,
            entries: &[BindGroupEntry { binding: 0, resource: instance_buffer.as_entire_binding() }],
        });

        // Create caches
        let texture_cache = TextureCache::new(fragment_texture_bind_layout.clone(), linear_sampler, point_sampler);
        let pipeline_cache = PipelineCache::new(
            default_vertex_shader,
            default_fragment_shader,
            vertex_bind_layout,
            fragment_texture_bind_layout,
        );

        // Create ImGui renderer if passed a config, otherwise do not
        let imgui_renderer =
            imgui_config.map(|config| ImGuiRenderer::new(&mut config.0, &device, &queue, Default::default()));

        Ok(Self {
            window_surface,
            window_surface_config,
            device,
            queue,
            vertex_buffer,
            instance_buffer,
            vertex_bind_group,
            texture_cache,
            pipeline_cache,
            imgui_renderer,
            per_frame: None,
        })
    }

    pub fn present_frame(&mut self) -> Result<(), KelpError> {
        if let Some(PerFrame { surface, encoder }) = self.per_frame.take() {
            self.queue.submit(Some(encoder.finish()));
            surface.present()
        } else {
            self.window_surface.get_current_texture()?.present()
        }
        Ok(())
    }

    pub fn render_batch(&mut self, batch_data: RenderBatchData) -> Result<(), KelpError> {
        if batch_data.batches.is_empty() || batch_data.instances.is_empty() {
            return Ok(()); // TODO: this could be an error instead
        }

        // Initialise per frame resources if this is the first pass this frame
        if self.per_frame.is_none() {
            let surface = self.window_surface.get_current_texture()?;
            let encoder_desc = &CommandEncoderDescriptor { label: Some("Kelp Render Commands") };
            let encoder = self.device.create_command_encoder(encoder_desc);
            self.per_frame.replace(PerFrame { surface, encoder });
        }
        let frame = self.per_frame.as_mut().unwrap();

        let camera_bytes = bytemuck::bytes_of(&batch_data.camera);
        let instances_bytes = bytemuck::cast_slice(&batch_data.instances);
        self.queue.write_buffer(&self.instance_buffer, 0, instances_bytes);

        // TODO: different render targets
        let target_tex = &frame.surface;
        let target_view = target_tex.texture.create_view(&Default::default());
        let load = batch_data.clear.map_or(LoadOp::Load, LoadOp::Clear);
        let mut wgpu_pass = frame.encoder.begin_render_pass(&RenderPassDescriptor {
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: Operations { load, store: StoreOp::Store },
            })],
            ..Default::default()
        });

        // Create any pipelines and bind groups we will need up front
        for batch in &batch_data.batches {
            self.pipeline_cache.ensure_pipeline(&self.device, None, batch.blend_mode)?;
            self.texture_cache.ensure_bind_group(&self.device, batch.texture, batch.smooth)?;
        }

        let mut pipeline_index = usize::MAX; // starts invalid
        let mut instance_index = 0;
        for batch in &batch_data.batches {
            let next_index = self.pipeline_cache.get_pipeline_index(None, batch.blend_mode)?;

            if pipeline_index != next_index {
                pipeline_index = next_index;
                wgpu_pass.set_pipeline(self.pipeline_cache.get_pipeline(pipeline_index)?);
                wgpu_pass.set_push_constants(ShaderStages::VERTEX, 0, camera_bytes);
                wgpu_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                wgpu_pass.set_bind_group(0, &self.vertex_bind_group, &[]);
            }

            let bind_group_1 = self.texture_cache.get_bind_group(batch.texture, batch.smooth)?;
            wgpu_pass.set_bind_group(1, bind_group_1, &[]);

            let instance_range_end = instance_index + batch.instance_count;
            wgpu_pass.draw(0..4, instance_index..instance_range_end);
            instance_index = instance_range_end;
        }

        Ok(())
    }

    pub fn create_texture_with_data(&mut self, width: u32, height: u32, data: &[u8]) -> KelpTextureId {
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
        self.texture_cache.insert_texture(texture)
    }

    pub fn render_imgui() {
        todo!()
    }

    pub fn set_surface_size(&mut self, width: u32, height: u32) {
        self.window_surface_config.width = width;
        self.window_surface_config.height = height;
        self.window_surface.configure(&self.device, &self.window_surface_config);
    }

    pub fn update_buffer<T: NoUninit>(&self, buffer: &Buffer, data: &[T]) {
        let bytes = bytemuck::cast_slice(data);
        self.queue.write_buffer(buffer, 0, bytes);
    }
}
