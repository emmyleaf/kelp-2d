use imgui::{Condition, Context};
use kelp_2d::{BlendMode, Camera, ImGuiConfig, InstanceData, InstanceMode, Kelp, KelpColor, RenderList};
use kelp_2d_imgui_wgpu::FontTexture;
use rand::Rng;
use std::{f32::consts::TAU, fs::File, mem::transmute, path::Path};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

pub fn world_mat(render_x: f32, render_y: f32, rotation: f32, scale_x: f32, scale_y: f32) -> mint::RowMatrix3x2<f32> {
    let (sin, cos) = rotation.sin_cos();
    mint::RowMatrix3x2 {
        x: mint::Vector2 { x: cos * scale_x, y: sin * scale_x },
        y: mint::Vector2 { x: -sin * scale_y, y: cos * scale_y },
        z: mint::Vector2 { x: render_x, y: render_y },
    }
}

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();

    // Set up imgui
    let mut imgui = Context::create();
    imgui.io_mut().display_size = [size.width as f32, size.height as f32];
    let font_atlas = imgui.fonts().build_rgba32_texture();
    let font_texture = FontTexture {
        tex_id: None,
        width: font_atlas.width,
        height: font_atlas.height,
        data: font_atlas.data as *const [u8] as *const u8,
        data_length: font_atlas.data.len() as u32,
    };

    let mut kelp = Kelp::new(&window, size.width, size.height, Some(&mut ImGuiConfig(font_texture))).unwrap();

    // Set initial camera matrix
    let camera =
        Camera::new(size.width as f32 / 2.0, size.height as f32 / 2.0, size.width as f32, size.height as f32, 0.0, 1.0);
    let clear = Some(&KelpColor { r: 0.5, g: 0.0, b: 0.5, a: 1.0 });

    // Create petal texture & bind group
    let decoder = png::Decoder::new(File::open(Path::new("./kelp-2d/examples/petal.png")).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let (tex_width, tex_height) = reader.info().size();
    let mut data = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut data).unwrap();
    let petal_texture = kelp.create_texture_with_data(tex_width, tex_height, data.as_slice()).unwrap();

    // Set instance buffer
    let mut instance_data: Vec<InstanceData> = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..128 {
        let color = [1.0, 1.0, 1.0, 1.0].into();
        let mode = InstanceMode::Multiply;
        let source_trans = [0.0, 0.0].into();
        let source_scale = [1.0, 1.0].into();
        let world = world_mat(
            rng.gen_range(0.0..(size.width as f32)),
            rng.gen_range(0.0..(size.height as f32)),
            rng.gen_range(0.0..(TAU)),
            tex_width as f32,
            tex_height as f32,
        );

        instance_data.push(InstanceData { color, mode, source_trans, source_scale, world });
    }

    event_loop
        .run(move |event, event_loop_window_target| {
            // Have the closure take ownership of kelp
            let _ = kelp;

            match event {
                Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                    kelp.set_surface_size(size.width, size.height);
                    // On macos the window needs to be redrawn manually after resizing
                    window.request_redraw();
                }
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    let sprite_list = RenderList::new(None, &camera, clear).add_instances(
                        &kelp,
                        petal_texture,
                        true,
                        BlendMode::ALPHA,
                        instance_data.as_slice(),
                    );
                    kelp.render_list(sprite_list.unwrap()).unwrap();

                    // do some imgui stuff!
                    let ui = imgui.frame();
                    ui.window("Kelp Imgui!")
                        .size([300.0, 100.0], Condition::FirstUseEver)
                        .build(|| ui.text_wrapped("Hello world!"));
                    _ = kelp.render_imgui(unsafe { transmute(imgui.render()) });

                    kelp.present_frame().unwrap();
                }
                Event::AboutToWait { .. } => window.request_redraw(),
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => event_loop_window_target.exit(),
                _ => {}
            }
        })
        .unwrap()
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = Window::new(&event_loop).unwrap();
    pollster::block_on(run(event_loop, window));
}
