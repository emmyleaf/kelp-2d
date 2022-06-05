use rand::Rng;
use std::{fs::File, ops::Range, path::Path};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();

    let mut kelp = kelp::Kelp::new(&window, size.width, size.height).await;

    // Set initial camera matrices
    let projection = glam::Mat4::orthographic_rh(0.0, size.width as f32, size.height as f32, 0.0, 0.0, 1.0);
    let mut camera_data = [0_f32; 32];
    projection.write_cols_to_slice(&mut camera_data);
    glam::Mat4::IDENTITY.write_cols_to_slice(&mut camera_data[16..32]);

    kelp.update_buffer(&kelp.vertex_group.camera_buffer, &camera_data);

    // Create fragment bind group & resources
    let decoder = png::Decoder::new(File::open(Path::new("./examples/instanced/petal.png")).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let (tex_width, tex_height) = reader.info().size();
    let mut data = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut data).unwrap();
    let texture = kelp.create_texture_with_data("Petal Texture", tex_width, tex_height, data.as_slice());

    let fragment_bind_group = kelp.create_fragment_bind_group("Petal Fragment Bind Group", &texture);

    // Set instance buffer
    const INSTANCE_RANGE: Range<u32> = 0..2047;
    let mut instance_data = vec![];
    let mut rng = rand::thread_rng();
    for _ in INSTANCE_RANGE {
        let color = glam::vec4(1.0, 1.0, 1.0, 1.0);
        let source = glam::Mat4::IDENTITY;
        let trans_x = rng.gen_range(0.0..(size.width as f32));
        let trans_y = rng.gen_range(0.0..(size.height as f32));
        let rotation = rng.gen_range(0.0..(2.0 * std::f32::consts::PI));
        let world = glam::Mat4::from_scale_rotation_translation(
            glam::vec3(tex_width as f32, tex_height as f32, 1.0),
            glam::Quat::from_rotation_z(rotation),
            glam::vec3(trans_x, trans_y, 0.0),
        );

        instance_data.extend_from_slice(&color.to_array());
        instance_data.extend_from_slice(&source.to_cols_array());
        instance_data.extend_from_slice(&world.to_cols_array());
    }

    kelp.update_buffer(&kelp.vertex_group.instance_buffer, &instance_data.as_slice());

    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of kelp
        let _ = kelp;

        *control_flow = ControlFlow::Poll;
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                kelp.set_surface_size(size.width, size.height);
                // On macos the window needs to be redrawn manually after resizing
                window.request_redraw();
            }
            Event::MainEventsCleared => {
                let mut frame = kelp.begin_frame();

                {
                    let mut render_pass = frame.begin_render_pass();
                    render_pass.set_pipeline(&kelp.pipeline);
                    render_pass.set_vertex_buffer(0, kelp.vertex_buffer.slice(..));
                    render_pass.set_bind_group(0, &kelp.vertex_group.bind, &[]);
                    render_pass.set_bind_group(1, &fragment_bind_group, &[]);
                    render_pass.draw(0..4, INSTANCE_RANGE);
                }

                kelp.end_frame(frame);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

fn main() {
    let event_loop = EventLoop::new();
    let window = winit::window::Window::new(&event_loop).unwrap();
    env_logger::init();
    pollster::block_on(run(event_loop, window));
}
