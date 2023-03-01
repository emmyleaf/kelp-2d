#![feature(c_size_t)]
#![allow(clippy::missing_safety_doc)]

mod window;

use core::{ffi::c_size_t, slice};
use kelp_2d::{BlendMode, Camera, ImGuiConfig, InstanceData, Kelp, KelpRenderPass, KelpTexture};
use window::Window;

static mut KELP: Option<Kelp> = None;

const KELP_NOT_FOUND: &str = "Cannot call any functions before initialise or after uninitialise.";

#[no_mangle]
pub unsafe extern "C" fn initialise(window: Window, imgui_config_ptr: *mut ImGuiConfig) {
    env_logger::init();
    let imgui_config = match imgui_config_ptr.is_null() {
        true => None,
        false => Some(&mut *imgui_config_ptr),
    };
    KELP = Some(Kelp::new(&window, window.width, window.height, imgui_config));
}

#[no_mangle]
pub unsafe extern "C" fn add_instances(
    pass_ptr: *mut KelpRenderPass,
    texture_ptr: *mut KelpTexture,
    smooth: bool,
    blend_mode: BlendMode,
    instance_ptr: *const InstanceData,
    count: u32,
) {
    let pass = pass_ptr.as_mut().expect("frame_ptr not set to a valid SurfaceFrame");
    let texture = texture_ptr.as_ref().expect("texture_ptr not set to a valid KelpTexture");
    assert!(!instance_ptr.is_null());
    let instances = slice::from_raw_parts(instance_ptr, count as usize);
    pass.add_instances(texture, smooth, blend_mode, instances);
}

#[no_mangle]
pub unsafe extern "C" fn begin_render_pass(
    camera_ptr: *const Camera,
    clear_ptr: *const [f32; 4],
) -> *const KelpRenderPass {
    let kelp = KELP.as_mut().expect(KELP_NOT_FOUND);
    assert!(!camera_ptr.is_null());
    let clear = match clear_ptr.is_null() {
        true => None,
        false => Some((*clear_ptr).into()),
    };
    Box::into_raw(Box::new(kelp.begin_render_pass(&*camera_ptr, clear)))
}

#[no_mangle]
pub unsafe extern "C" fn create_texture_with_data(
    width: u32,
    height: u32,
    data_ptr: *const u8,
    data_len: u32,
) -> *mut KelpTexture {
    let kelp = KELP.as_ref().expect(KELP_NOT_FOUND);
    assert!(!data_ptr.is_null());
    let data = slice::from_raw_parts(data_ptr, data_len as usize);
    let kelp_texture = kelp.create_texture_with_data(width, height, data);
    Box::into_raw(Box::new(kelp_texture))
}

#[no_mangle]
pub unsafe extern "C" fn end_render_pass(frame_ptr: *mut KelpRenderPass) {
    assert!(!frame_ptr.is_null());
    Box::from_raw(frame_ptr).finish();
}

#[no_mangle]
pub unsafe extern "C" fn free_texture(texture_ptr: *mut KelpTexture) {
    _ = Box::from_raw(texture_ptr);
}

#[no_mangle]
pub unsafe extern "C" fn set_surface_size(width: u32, height: u32) {
    let kelp = KELP.as_mut().expect(KELP_NOT_FOUND);
    kelp.set_surface_size(width, height)
}

#[no_mangle]
pub unsafe extern "C" fn uninitialise() {
    _ = KELP.take();
}
