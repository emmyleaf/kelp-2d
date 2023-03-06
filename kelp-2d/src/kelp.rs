use crate::{
    Camera, ImGuiConfig, KelpColor, KelpError, KelpFrame, KelpRenderPass, KelpTextureId, PipelineCache, TextureCache,
};
use bytemuck::NoUninit;
use kelp_2d_imgui_wgpu::ImGuiRenderer;
use naga::{FastHashMap, ShaderStage};
use pollster::FutureExt;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::{borrow::Cow, cell::RefCell, num::NonZeroU64, rc::Rc};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingType, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Device,
    DeviceDescriptor, Extent3d, Features, FilterMode, Instance, InstanceDescriptor, Limits, PresentMode, Queue,
    RequestAdapterOptions, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    Surface, SurfaceConfiguration, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureViewDescriptor, TextureViewDimension,
};

#[derive(Debug)]
pub struct Kelp {
    pub(crate) window_surface: Surface,
    pub(crate) window_surface_config: SurfaceConfiguration,
    pub(crate) resources: Rc<KelpResources>,
    pub(crate) current_frame: Option<KelpFrame>,
}

#[derive(Debug)]
pub(crate) struct KelpResources {
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) vertex_buffer: Buffer,
    pub(crate) instance_buffer: Buffer,
    pub(crate) vertex_bind_group: BindGroup,
    pub(crate) texture_cache: RefCell<TextureCache>,
    pub(crate) pipeline_cache: RefCell<PipelineCache>,
    pub(crate) imgui_renderer: Option<ImGuiRenderer>,
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
                defines: FastHashMap::default(),
            },
        });

        let default_fragment_shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Glsl {
                shader: Cow::Borrowed(include_str!("../shaders/shader.frag")),
                stage: ShaderStage::Fragment,
                defines: FastHashMap::default(),
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
        let texture_bind_group_cache =
            RefCell::new(TextureCache::new(fragment_texture_bind_layout.clone(), linear_sampler, point_sampler));

        let pipeline_cache = RefCell::new(PipelineCache::new(
            default_vertex_shader,
            default_fragment_shader,
            vertex_bind_layout,
            fragment_texture_bind_layout,
        ));

        // Create ImGui renderer if passed a config, otherwise do not
        let imgui_renderer =
            imgui_config.map(|config| ImGuiRenderer::new(&mut config.0, &device, &queue, Default::default()));

        Ok(Self {
            window_surface,
            window_surface_config,
            resources: Rc::new(KelpResources {
                device,
                queue,
                vertex_buffer,
                instance_buffer,
                vertex_bind_group,
                texture_cache: texture_bind_group_cache,
                pipeline_cache,
                imgui_renderer,
            }),
            current_frame: None,
        })
    }

    pub fn begin_frame(&mut self) -> Result<(), KelpError> {
        let surface = self.window_surface.get_current_texture()?;
        let encoder = self.resources.device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        self.current_frame = Some(KelpFrame { surface, encoder });
        Ok(())
    }

    pub fn draw_frame(&mut self) -> Result<(), KelpError> {
        let frame = self.current_frame.take().ok_or(KelpError::NoCurrentFrame)?;
        self.resources.queue.submit(Some(frame.encoder.finish()));
        frame.surface.present();
        Ok(())
    }

    pub fn begin_render_pass(
        &mut self,
        camera: &Camera,
        clear: Option<KelpColor>,
    ) -> Result<KelpRenderPass, KelpError> {
        let frame = self.current_frame.as_ref().ok_or(KelpError::NoCurrentFrame)?;
        let target_view = frame.surface.texture.create_view(&TextureViewDescriptor::default());
        Ok(KelpRenderPass::new(camera.into(), clear, target_view, self.resources.clone()))
    }

    pub fn submit_render_pass(&mut self, pass: KelpRenderPass) -> Result<(), KelpError> {
        let frame = self.current_frame.as_mut().ok_or(KelpError::NoCurrentFrame)?;
        pass.finish(&mut frame.encoder)
    }

    pub fn create_texture_with_data(&mut self, width: u32, height: u32, data: &[u8]) -> KelpTextureId {
        let texture = self.resources.device.create_texture_with_data(
            &self.resources.queue,
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
        self.resources.texture_cache.borrow_mut().insert_texture(texture)
    }

    pub fn render_imgui() {
        todo!()
    }

    pub fn set_surface_size(&mut self, width: u32, height: u32) {
        self.window_surface_config.width = width;
        self.window_surface_config.height = height;
        self.window_surface.configure(&self.resources.device, &self.window_surface_config);
    }

    pub fn update_buffer<T: NoUninit>(&self, buffer: &Buffer, data: &[T]) {
        let bytes = bytemuck::cast_slice(data);
        self.resources.queue.write_buffer(buffer, 0, bytes);
    }
}
