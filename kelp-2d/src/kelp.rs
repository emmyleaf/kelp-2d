use crate::{
    ImGuiConfig, InstanceGPU, KelpError, KelpTargetId, KelpTextureId, PipelineCache, RenderList, TextureCache,
};
use bytemuck::NoUninit;
use kelp_2d_imgui_wgpu::{DrawData, ImGuiRenderer, RendererConfig};
use pollster::FutureExt;
use std::{
    borrow::Cow,
    cell::{OnceCell, RefCell},
    mem::size_of,
    num::NonZeroU64,
    rc::Rc,
};
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct PerFrame {
    pub(crate) surface: wgpu::SurfaceTexture,
    pub(crate) buffer_encoder: wgpu::CommandEncoder,
    pub(crate) draw_encoder: wgpu::CommandEncoder,
    pub(crate) imgui_encoder: Option<wgpu::CommandEncoder>,
    pub(crate) instance_offset: u32,
}

pub struct Kelp {
    pub(crate) window_surface: wgpu::Surface<'static>,
    pub(crate) window_surface_config: wgpu::SurfaceConfiguration,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) instance_buffer: wgpu::Buffer,
    pub(crate) instance_staging_buffer: wgpu::Buffer,
    pub(crate) main_bind_group: wgpu::BindGroup,
    pub(crate) texture_array: Rc<wgpu::Texture>,
    pub(crate) texture_cache: RefCell<TextureCache>,
    pub(crate) pipeline_cache: PipelineCache,
    pub(crate) imgui_renderer: Option<ImGuiRenderer>,
    pub(crate) per_frame: OnceCell<PerFrame>,
}

impl Kelp {
    pub fn new<W: wgpu::rwh::HasRawDisplayHandle + wgpu::rwh::HasRawWindowHandle>(
        window: &W,
        width: u32,
        height: u32,
        imgui_config: Option<&mut ImGuiConfig>,
    ) -> Result<Kelp, KelpError> {
        let instance =
            wgpu::Instance::new(wgpu::InstanceDescriptor { backends: wgpu::Backends::PRIMARY, ..Default::default() });
        let surface_target = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: window.raw_display_handle().unwrap(),
            raw_window_handle: window.raw_window_handle().unwrap(),
        };
        let window_surface = unsafe { instance.create_surface_unsafe(surface_target).unwrap() };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&window_surface),
                ..Default::default()
            })
            .block_on()
            .ok_or(KelpError::NoAdapter)?;

        // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
        let mut required_limits = wgpu::Limits::downlevel_defaults().using_resolution(adapter.limits());
        required_limits.max_push_constant_size = 128;

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::PUSH_CONSTANTS,
                    required_limits,
                },
                None,
            )
            .block_on()?;

        // Configure surface
        let window_surface_config = wgpu::SurfaceConfiguration {
            present_mode: wgpu::PresentMode::Fifo,
            ..window_surface.get_default_config(&adapter, width, height).unwrap()
        };

        window_surface.configure(&device, &window_surface_config);

        // Load the default shaders from disk
        let default_vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: Cow::Borrowed(include_str!("../shaders/glsl/sprite.vert")),
                stage: wgpu::naga::ShaderStage::Vertex,
                defines: Default::default(),
            },
        });

        let default_fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Glsl {
                shader: Cow::Borrowed(include_str!("../shaders/glsl/sprite.frag")),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: Default::default(),
            },
        });

        // Create layouts for vertex shader bind group
        let instance_buffer_layout = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: NonZeroU64::new(16 + 16 + 16 + 64),
            },
            count: None,
        };

        let texture_array_bind_entry = wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
        };

        let point_sampler_bind_entry = wgpu::BindGroupLayoutEntry {
            binding: 2,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            count: None,
        };

        let linear_sampler_bind_entry = wgpu::BindGroupLayoutEntry {
            binding: 3,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };

        let sprite_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Vertex Bind Group Layout"),
            entries: &[
                instance_buffer_layout,
                texture_array_bind_entry,
                point_sampler_bind_entry,
                linear_sampler_bind_entry,
            ],
        });

        // Create buffers
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            // Vertices (0, 0), (1, 0), (0, 1), (1, 1)
            contents: bytemuck::bytes_of(&[0_f32, 0_f32, 1_f32, 0_f32, 0_f32, 1_f32, 1_f32, 1_f32]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: 8 << 20, // 8MB
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let instance_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Staging Buffer"),
            size: 8 << 20, // 8MB
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        // Create point sampler
        let point_sampler =
            device.create_sampler(&wgpu::SamplerDescriptor { label: Some("Point Sampler"), ..Default::default() });

        // Create linear sampler
        let linear_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Linear Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create texture array - initially only 1 layer
        let texture_array = Rc::new(device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width: 2048, height: 2048, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        }));

        // Create sprite bind group
        // TODO: fill this in
        let sprite_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Sprite Bind Group"),
            layout: &sprite_bind_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: instance_buffer.as_entire_binding() },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_array.create_view(
                        &wgpu::TextureViewDescriptor {
                            dimension: Some(wgpu::TextureViewDimension::D2Array),
                            ..Default::default()
                        },
                    )),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&point_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&linear_sampler),
                },
            ],
        });

        // Create caches
        let texture_cache = RefCell::new(TextureCache::new(texture_array.as_ref(), point_sampler, linear_sampler));
        let pipeline_cache = PipelineCache::new(
            default_vertex_shader,
            default_fragment_shader,
            sprite_bind_layout,
            window_surface_config.format,
        );

        // Create ImGui renderer if passed a config, otherwise do not
        let imgui_renderer = imgui_config.map(|config| {
            ImGuiRenderer::new(
                &mut config.0,
                &device,
                &queue,
                RendererConfig {
                    texture_format: window_surface_config.format,
                    ..Default::default()
                },
            )
        });

        Ok(Self {
            window_surface,
            window_surface_config,
            device,
            queue,
            vertex_buffer,
            instance_buffer,
            instance_staging_buffer,
            main_bind_group: sprite_bind_group,
            texture_array,
            texture_cache,
            pipeline_cache,
            imgui_renderer,
            per_frame: OnceCell::new(),
        })
    }

    pub fn present_frame(&mut self) -> Result<(), KelpError> {
        if let Some(PerFrame { surface, mut buffer_encoder, draw_encoder, imgui_encoder, .. }) = self.per_frame.take() {
            // Copy to the shader's instance buffer
            buffer_encoder.copy_buffer_to_buffer(
                &self.instance_staging_buffer,
                0,
                &self.instance_buffer,
                0,
                self.instance_buffer.size(),
            );
            // Submit and present the frame!
            let mut commands = vec![buffer_encoder.finish(), draw_encoder.finish()];
            if let Some(encoder) = imgui_encoder {
                commands.push(encoder.finish());
            }
            self.queue.submit(commands);
            surface.present()
        } else {
            self.window_surface.get_current_texture()?.present()
        }
        Ok(())
    }

    pub fn render_list(&mut self, render_list: RenderList) -> Result<(), KelpError> {
        if render_list.batches.is_empty() || render_list.instances.is_empty() {
            return Ok(()); // TODO: this could be an error instead
        }

        // TODO: Error if too many instances also

        // Initialise per frame resources if this is the first pass this frame
        _ = self.per_frame.get_or_try_init(|| self.init_per_frame())?;
        let frame = self.per_frame.get_mut().unwrap();

        let camera_bytes = bytemuck::bytes_of(&render_list.camera);
        let instances_bytes = bytemuck::cast_slice(&render_list.instances);
        let instances_length = instances_bytes.len() as u64;

        // Write instances to the staging buffer
        let byte_offset = frame.instance_offset as u64 * size_of::<InstanceGPU>() as u64;
        let instance_range = byte_offset..byte_offset + instances_length;
        let staging_buffer_slice = self.instance_staging_buffer.slice(instance_range);
        staging_buffer_slice.map_async(wgpu::MapMode::Write, move |_| {});
        self.device.poll(wgpu::Maintain::Wait);
        staging_buffer_slice.get_mapped_range_mut().copy_from_slice(instances_bytes);
        self.instance_staging_buffer.unmap();

        // Create wgpu render pass with correct target texture
        let tex_cache = self.texture_cache.borrow();
        let target_tex = match render_list.target {
            Some(target_id) => tex_cache.get_target(target_id)?,
            None => &frame.surface.texture,
        };
        let target_view = target_tex.create_view(&Default::default());
        let load = render_list.clear.map_or(wgpu::LoadOp::Load, wgpu::LoadOp::Clear);
        let mut wgpu_pass = frame.draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                resolve_target: None,
                ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
            })],
            ..Default::default()
        });

        // Create any new pipelines we will need up front
        for batch in &render_list.batches {
            self.pipeline_cache.ensure_pipeline(&self.device, None, batch.blend_mode)?;
        }

        // TODO: we don't really need the concept of batches in here anymore!
        let mut pipeline_index = usize::MAX; // starts invalid
        for batch in &render_list.batches {
            let next_index = self.pipeline_cache.get_pipeline_index(None, batch.blend_mode)?;

            if pipeline_index != next_index {
                pipeline_index = next_index;
                wgpu_pass.set_pipeline(self.pipeline_cache.get_pipeline(pipeline_index)?);
                wgpu_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, camera_bytes);
                wgpu_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                wgpu_pass.set_bind_group(0, &self.main_bind_group, &[]);
            }

            let instance_range_end = frame.instance_offset + batch.instance_count;
            wgpu_pass.draw(0..4, frame.instance_offset..instance_range_end);
            frame.instance_offset = instance_range_end;
        }

        Ok(())
    }

    pub fn create_texture_empty(&mut self, width: u32, height: u32) -> KelpTextureId {
        self.texture_cache.borrow_mut().new_texture_alloc(width, height)
    }

    pub fn create_render_target(&mut self, width: u32, height: u32) -> KelpTargetId {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Match the texture format with the surface, so we can reuse the pipelines
            format: self.window_surface_config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.texture_cache.borrow_mut().insert_target(texture)
    }

    pub fn create_texture_with_data(
        &mut self,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Result<KelpTextureId, KelpError> {
        let id = self.texture_cache.borrow_mut().new_texture_alloc(width, height);
        self.update_texture(id, data)?;
        Ok(id)
    }

    pub fn render_imgui(&mut self, draw_data: &DrawData) -> Result<(), KelpError> {
        if self.imgui_renderer.is_none() {
            Err(KelpError::NoImgui)
        } else {
            _ = self.per_frame.get_or_try_init(|| self.init_per_frame())?;
            let frame = self.per_frame.get_mut().unwrap();
            let encoder_desc = &wgpu::CommandEncoderDescriptor { label: Some("Kelp Imgui Commands") };
            let mut encoder = self.device.create_command_encoder(encoder_desc);
            let tex_view = frame.surface.texture.create_view(&Default::default());
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &tex_view,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });
            self.imgui_renderer.as_mut().unwrap().render(draw_data, &self.queue, &self.device, &mut rpass)?;
            drop(rpass);
            frame.imgui_encoder.replace(encoder);
            Ok(())
        }
    }

    pub fn set_surface_size(&mut self, width: u32, height: u32) {
        self.window_surface_config.width = width;
        self.window_surface_config.height = height;
        self.window_surface.configure(&self.device, &self.window_surface_config);
    }

    pub fn update_buffer<T: NoUninit>(&self, buffer: &wgpu::Buffer, data: &[T]) {
        // TODO: check that the data does not overflow buffer
        let bytes = bytemuck::cast_slice(data);
        self.queue.write_buffer(buffer, 0, bytes);
    }

    pub fn update_texture(&self, texture_id: KelpTextureId, data: &[u8]) -> Result<(), KelpError> {
        // TODO: check that the data length is correct for tex dims
        let allocation = self.texture_cache.borrow().get_texture(texture_id)?;
        let copy_texture = wgpu::ImageCopyTexture {
            texture: self.texture_array.as_ref(),
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: allocation.rectangle.min.x as u32,
                y: allocation.rectangle.min.y as u32,
                z: allocation.id.layer,
            },
            aspect: wgpu::TextureAspect::All,
        };
        let write_size = wgpu::Extent3d {
            width: allocation.rectangle.width() as u32,
            height: allocation.rectangle.height() as u32,
            depth_or_array_layers: 1,
        };
        let data_layout = wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * write_size.width),
            rows_per_image: Some(write_size.height),
        };
        self.queue.write_texture(copy_texture, data, data_layout, write_size);
        Ok(())
    }

    fn init_per_frame(&self) -> Result<PerFrame, KelpError> {
        let surface = self.window_surface.get_current_texture()?;
        let buffer_encoder_desc = &wgpu::CommandEncoderDescriptor { label: Some("Kelp Buffer Commands") };
        let buffer_encoder = self.device.create_command_encoder(buffer_encoder_desc);
        let draw_encoder_desc = &wgpu::CommandEncoderDescriptor { label: Some("Kelp Draw Commands") };
        let draw_encoder = self.device.create_command_encoder(draw_encoder_desc);
        Ok(PerFrame {
            surface,
            buffer_encoder,
            draw_encoder,
            instance_offset: 0,
            imgui_encoder: None,
        })
    }
}
