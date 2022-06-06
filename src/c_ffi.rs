use crate::{FrameState, Kelp, Window};
use core::{ffi::c_size_t, slice};

// type BufferId = wgpu::backend::Context::BufferId;

static mut KELP: Option<Kelp> = None;
static mut FRAME: Option<FrameState> = None;
static mut PASS: Option<wgpu::RenderPass> = None;

#[no_mangle]
pub unsafe extern "C" fn initialise(window: Window, width: u32, height: u32) {
    KELP = Some(Kelp::new(&window, width, height));
}

#[no_mangle]
pub unsafe extern "C" fn begin_frame() {
    let kelp = KELP.as_mut().expect("Cannot call begin_frame before initialise or after dispose.");
    FRAME = Some(kelp.begin_frame());
}

// TODO: finish this function & the rest :)
#[no_mangle]
pub unsafe extern "C" fn begin_render_pass() {
    let kelp = KELP.as_ref().expect("Cannot call begin_render_pass before initialise or after dispose.");
    let frame = FRAME.as_mut().expect("Cannot call begin_render_pass before begin_frame.");
    let mut render_pass = frame.begin_render_pass();
    render_pass.set_pipeline(&kelp.pipeline);
    render_pass.set_vertex_buffer(0, kelp.vertex_buffer.slice(..));
    render_pass.set_bind_group(0, &kelp.vertex_group.bind, &[]);
    PASS = Some(render_pass);
}

#[no_mangle]
pub unsafe extern "C" fn dispose() {
    _ = KELP.take();
    _ = FRAME.take();
    _ = PASS.take();
}

#[no_mangle]
pub unsafe extern "C" fn end_frame() {
    let kelp = KELP.as_mut().expect("Cannot call end_frame before initialise or after dispose.");
    let frame = FRAME.take().expect("Cannot call end_frame before begin_frame.");
    kelp.end_frame(frame);
}

#[no_mangle]
pub unsafe extern "C" fn set_surface_size(width: u32, height: u32) {
    let kelp = KELP.as_mut().expect("Cannot call set_surface_size before initialise or after dispose.");
    kelp.set_surface_size(width, height)
}

#[no_mangle]
pub unsafe extern "C" fn update_camera_buffer(pointer: *const u8, length: c_size_t) {
    let kelp = KELP.as_mut().expect("Cannot call update_camera_buffer before initialise or after dispose.");
    assert!(!pointer.is_null());
    assert!(length == 128);
    let data = slice::from_raw_parts(pointer, length);
    kelp.update_buffer(&kelp.vertex_group.camera_buffer, data)
}

#[no_mangle]
pub unsafe extern "C" fn update_instance_buffer(pointer: *const u8, length: c_size_t) {
    let kelp = KELP.as_mut().expect("Cannot call update_instance_buffer before initialise or after dispose.");
    assert!(!pointer.is_null());
    let data = slice::from_raw_parts(pointer, length);
    kelp.update_buffer(&kelp.vertex_group.instance_buffer, data)
}
