#![allow(clippy::missing_safety_doc)]
#![feature(once_cell)]

mod generate;
mod types;
mod window;

use interoptopus::{ffi_function, patterns::slice::FFISlice};
use kelp_2d::{Camera, ImGuiConfig, Kelp, KelpColor, KelpError, KelpTexture};
use std::sync::{Mutex, OnceLock};
use types::{FFIError, FFIResult, InstanceBatch};
use window::Window;

static mut KELP: OnceLock<Kelp> = OnceLock::new();

const KELP_NOT_FOUND: &str = "Cannot call any functions before initialise or after uninitialise.";

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn initialise(window: Window /*, imgui_config: Option<&mut ImGuiConfig>*/) -> FFIError {
    match KELP.set(Kelp::new(&window, window.width, window.height, None)) {
        Ok(_) => {
            env_logger::init();
            FFIError::Success
        }
        Err(_) => FFIError::KelpAlreadyInitialised,
    }
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn begin_frame() -> FFIError {
    match KELP.get_mut().map(Kelp::begin_frame) {
        Some(Ok(_)) => FFIError::Success,
        Some(Err(err)) => err.into(),
        None => FFIError::KelpNotInitialised,
    }
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn draw_frame() -> FFIError {
    match KELP.get_mut().map(Kelp::draw_frame) {
        Some(Ok(_)) => FFIError::Success,
        Some(Err(err)) => err.into(),
        None => FFIError::KelpNotInitialised,
    }
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn render_pass(
    camera: &Camera,
    clear: Option<&KelpColor>,
    batches: FFISlice<InstanceBatch>,
) -> FFIError {
    match KELP.get_mut().map(|kelp| {
        let mut pass = kelp.begin_render_pass(camera, clear.copied())?;
        for batch in batches.as_slice() {
            pass.add_instances(batch.texture, batch.smooth, batch.blend_mode, batch.instances.as_slice());
        }
        kelp.submit_render_pass(pass)?;
        Ok::<(), KelpError>(())
    }) {
        Some(Ok(_)) => FFIError::Success,
        Some(Err(err)) => err.into(),
        None => FFIError::KelpNotInitialised,
    }
}

// TODO: gotta fix this with texture caching
// #[ffi_function]
// #[no_mangle]
// pub unsafe extern "C" fn create_texture_with_data(
//     width: u32,
//     height: u32,
//     data: FFISlice<u8>,
// ) -> FFIResult<KelpTexture> {
//     let kelp = KELP.get().expect(KELP_NOT_FOUND);
//     let kelp_texture = kelp.create_texture_with_data(width, height, data.as_slice());
//     FFIResult::ok(kelp_texture)
// }

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn free_texture(texture_ptr: *mut KelpTexture) -> FFIError {
    // TODO: can we make this safer with interoptopus as well?
    _ = Box::from_raw(texture_ptr);
    FFIError::Success
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn set_surface_size(width: u32, height: u32) -> FFIError {
    let kelp = KELP.get_mut().expect(KELP_NOT_FOUND);
    kelp.set_surface_size(width, height);
    FFIError::Success
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn uninitialise() -> FFIError {
    _ = KELP.take();
    FFIError::Success
}
