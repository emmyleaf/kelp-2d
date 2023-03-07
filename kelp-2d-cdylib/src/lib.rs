#![allow(clippy::missing_safety_doc)]
#![feature(once_cell)]

mod generate;
mod types;
mod window_info;

use interoptopus::{ffi_function, patterns::slice::FFISlice};
use kelp_2d::{Camera, ImGuiConfig, Kelp, KelpColor, KelpError, KelpTextureId};
use std::sync::OnceLock;
use types::{FFIError, InstanceBatch};
use window_info::WindowInfo;

static mut KELP: OnceLock<Kelp> = OnceLock::new();

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn initialise(window: WindowInfo /*, imgui_config: Option<&mut ImGuiConfig>*/) -> FFIError {
    // Why is `OnceLock::is_initialized()` private..?
    if KELP.get().is_some() {
        return FFIError::KelpAlreadyInitialised;
    }
    _ = env_logger::try_init();
    match Kelp::new(&window, window.width, window.height, None) {
        Ok(kelp) => {
            _ = KELP.set(kelp);
            FFIError::Success
        }
        Err(err) => err.into(),
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
pub unsafe extern "C" fn submit_render_pass(
    camera: Camera,
    clear: Option<&KelpColor>,
    batches: FFISlice<InstanceBatch>,
) -> FFIError {
    match KELP.get_mut().map(|kelp| {
        let mut pass = kelp.begin_render_pass(&camera, clear.copied())?;
        for batch in batches.as_slice() {
            pass.add_instances(batch.texture, batch.smooth, batch.blend_mode, batch.instances.as_slice())?;
        }
        kelp.submit_render_pass(pass)?;
        Ok::<(), KelpError>(())
    }) {
        Some(Ok(_)) => FFIError::Success,
        Some(Err(err)) => err.into(),
        None => FFIError::KelpNotInitialised,
    }
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn create_texture_with_data(
    width: u32,
    height: u32,
    data: FFISlice<u8>,
    out_id: &mut KelpTextureId,
) -> FFIError {
    match KELP.get_mut().map(|kelp| kelp.create_texture_with_data(width, height, data.as_slice())) {
        Some(value) => {
            *out_id = value;
            FFIError::Success
        }
        None => FFIError::KelpNotInitialised,
    }
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn set_surface_size(width: u32, height: u32) -> FFIError {
    match KELP.get_mut().map(|kelp| kelp.set_surface_size(width, height)) {
        Some(_) => FFIError::Success,
        None => FFIError::KelpNotInitialised,
    }
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn uninitialise() -> FFIError {
    match KELP.take() {
        Some(_) => FFIError::Success,
        None => FFIError::KelpNotInitialised,
    }
}
