use kelp_2d::{BlendMode, Camera, InstanceData, InstanceMode, Kelp, KelpColor, RenderList};
use rand::Rng;
use std::{f32::consts::TAU, fs::File, path::Path};
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

    let mut kelp = Kelp::new(&window, size.width, size.height, None).unwrap();

    // Set initial camera matrix
    let camera =
        Camera::new(size.width as f32 / 2.0, size.height as f32 / 2.0, size.width as f32, size.height as f32, 0.0, 1.0);
    let clear = Some(&KelpColor { r: 0.5, g: 0.0, b: 0.5, a: 1.0 });

    // Create petal texture
    let decoder = png::Decoder::new(File::open(Path::new("./kelp-2d/examples/petal.png")).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let (tex_width, tex_height) = reader.info().size();
    let mut data = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut data).unwrap();
    let petal_texture = kelp.create_texture_with_data(tex_width, tex_height, data.as_slice()).unwrap();

    // Create render texture
    let render_texture = kelp.create_render_target(size.width, size.height);

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
    let instance_data_rt = [InstanceData {
        color: [1.0, 1.0, 1.0, 1.0].into(),
        mode: InstanceMode::Multiply,
        source_trans: [0.0, 0.0].into(),
        source_scale: [1.0, 1.0].into(),
        world: world_mat(0.0, 0.0, 0.0, size.width as f32, size.width as f32),
    }];

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
                    let render_list_to_rt = RenderList::new(
                        Some(render_texture),
                        &camera,
                        Some(&KelpColor { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                    )
                    .add_instances(
                        &kelp,
                        petal_texture,
                        false,
                        BlendMode::ALPHA,
                        instance_data.as_slice(),
                    );
                    // TODO: v simple render target pipeline!
                    let render_list_to_surf = RenderList::new(None, &camera, clear).add_instances(
                        &kelp,
                        render_texture,
                        false,
                        BlendMode::ALPHA,
                        instance_data_rt.as_slice(),
                    );
                    kelp.render_list(render_list_to_rt.unwrap()).unwrap();
                    kelp.render_list(render_list_to_surf.unwrap()).unwrap();
                    kelp.present_frame().unwrap();
                }
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
