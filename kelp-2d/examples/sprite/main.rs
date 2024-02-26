use kelp_2d::{BlendMode, Camera, InstanceData, InstanceMode, Kelp, KelpColor, RenderList};
use std::{fs::File, path::Path};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

async fn run(event_loop: EventLoop<()>, window: Window) {
    let size = window.inner_size();

    let mut kelp = Kelp::new(&window, size.width, size.height, None).unwrap();

    // Set initial camera matrix
    let mut camera =
        Camera::new(size.width as f32 / 2.0, size.height as f32 / 2.0, size.width as f32, size.height as f32, 0.0, 1.0);
    let clear = Some(&KelpColor { r: 0.5, g: 0.0, b: 0.5, a: 1.0 });

    // Create petal texture & bind group
    let decoder = png::Decoder::new(File::open(Path::new("./kelp-2d/examples/tester.png")).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let (tex_width, tex_height) = reader.info().size();
    let mut data = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut data).unwrap();
    let petal_texture = kelp.create_texture_with_data(tex_width, tex_height, data.as_slice()).unwrap();

    // Set instance buffer
    let mut instance_data: Vec<InstanceData> = vec![];
    for _ in 0..2 {
        let color = [1.0, 1.0, 1.0, 1.0].into();
        let mode = InstanceMode::Multiply;
        let source_trans = [0.0, 0.0].into();
        let source_scale = [1.0, 1.0].into();
        let world = mint::RowMatrix3x2 {
            x: mint::Vector2 { x: tex_width as f32, y: 0.0 },
            y: mint::Vector2 { x: 0.0, y: tex_height as f32 },
            z: mint::Vector2 { x: 0.0, y: 0.0 },
        };

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
                    // camera.scale += 0.001;
                    let list = RenderList::new(None, &camera, clear)
                        .add_instances(&kelp, petal_texture, false, BlendMode::ALPHA, instance_data.as_slice())
                        .unwrap();
                    // dbg!(&list);
                    kelp.render_list(list).unwrap();
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
