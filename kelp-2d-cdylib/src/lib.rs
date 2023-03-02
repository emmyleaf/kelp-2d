#![allow(clippy::missing_safety_doc)]
#![feature(once_cell)]

mod generate;
mod types;
mod window;

use interoptopus::{ffi_function, patterns::slice::FFISlice};
use kelp_2d::{Camera, ImGuiConfig, Kelp, KelpColor, KelpTexture};
use std::sync::OnceLock;
use types::InstanceBatch;
use window::Window;

static mut KELP: OnceLock<Kelp> = OnceLock::new();

const KELP_NOT_FOUND: &str = "Cannot call any functions before initialise or after uninitialise.";

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn initialise(window: Window /*, imgui_config_ptr: *mut ImGuiConfig*/) {
    env_logger::init();
    // let imgui_config = match imgui_config_ptr.is_null() {
    //     true => None,
    //     false => Some(&mut *imgui_config_ptr),
    // };
    KELP.set(Kelp::new(&window, window.width, window.height, None)); // TODO: start doing some error handling...
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn render_pass(camera: &Camera, clear: Option<&KelpColor>, batches: FFISlice<InstanceBatch>) {
    let kelp = KELP.get_mut().expect(KELP_NOT_FOUND);
    let mut pass = kelp.begin_render_pass(camera, clear.copied());

    for batch in batches.as_slice() {
        pass.add_instances(batch.texture, batch.smooth, batch.blend_mode, batch.instances.as_slice());
    }

    pass.finish()
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn create_texture_with_data(width: u32, height: u32, data: FFISlice<u8>) -> *mut KelpTexture {
    let kelp = KELP.get().expect(KELP_NOT_FOUND);
    let kelp_texture = kelp.create_texture_with_data(width, height, data.as_slice());
    Box::into_raw(Box::new(kelp_texture))
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn free_texture(texture_ptr: *mut KelpTexture) {
    // TODO: can we make this safer with interoptopus as well?
    _ = Box::from_raw(texture_ptr);
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn set_surface_size(width: u32, height: u32) {
    let kelp = KELP.get_mut().expect(KELP_NOT_FOUND);
    kelp.set_surface_size(width, height)
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn uninitialise() {
    _ = KELP.take();
}
