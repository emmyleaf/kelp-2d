use kelp_2d::{InstanceData, Kelp, SourceTransform, WorldTransform};
use rand::Rng;
use std::{fs::File, path::Path};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();

    let mut kelp = Kelp::new(&window, size.width, size.height);

    // Set initial camera matrix
    let projection = glam::Mat4::orthographic_rh(0.0, size.width as f32, size.height as f32, 0.0, 0.0, 1.0);
    kelp.update_buffer(&kelp.vertex_group.camera_buffer, &projection.to_cols_array());

    // Create petal texture & bind group
    let decoder = png::Decoder::new(File::open(Path::new("./examples/instanced/petal.png")).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let (tex_width, tex_height) = reader.info().size();
    let mut data = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut data).unwrap();
    let petal_texture = kelp.create_texture_with_data(tex_width, tex_height, data.as_slice());

    // Set instance buffer
    let mut instance_data: Vec<InstanceData> = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..128 {
        let color = [1.0, 1.0, 1.0, 1.0];
        let source = SourceTransform::default();
        let world = WorldTransform {
            render_x: rng.gen_range(0.0..(size.width as f32)),
            render_y: rng.gen_range(0.0..(size.height as f32)),
            rotation: rng.gen_range(0.0..(2.0 * std::f32::consts::PI)),
            scale_x: tex_width as f32,
            scale_y: tex_height as f32,
            ..WorldTransform::default()
        };

        instance_data.push(InstanceData { color, source, world });
    }
    let mut instance_data_2: Vec<InstanceData> = vec![];
    for _ in 0..(1 << 14) {
        let color =
            [rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0), rng.gen_range(0.0..1.0)];
        let source = SourceTransform::default();
        let world = WorldTransform {
            render_x: rng.gen_range(0.0..(size.width as f32)),
            render_y: rng.gen_range(0.0..(size.height as f32)),
            rotation: rng.gen_range(0.0..(2.0 * std::f32::consts::PI)),
            scale_x: tex_width as f32,
            scale_y: tex_height as f32,
            ..WorldTransform::default()
        };

        instance_data_2.push(InstanceData { color, source, world });
    }

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of kelp
        let _ = kelp;

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                kelp.set_surface_size(size.width, size.height);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::MainEventsCleared => {
                let mut frame = kelp.begin_surface_frame();
                frame.add_instances(&petal_texture, instance_data.as_slice());
                frame.add_instances(&petal_texture, instance_data_2.as_slice());
                kelp.end_surface_frame(frame);
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    pollster::block_on(run(event_loop, window));
}
