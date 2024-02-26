#![allow(clippy::missing_safety_doc, clippy::transmute_ptr_to_ref)]

mod generate;
mod types;
mod window_info;

use interoptopus::{ffi_function, patterns::slice::FFISlice};
use kelp_2d::{Camera, InstanceBatch, InstanceGPU, Kelp, KelpColor, KelpTextureId, RenderList};
use std::{ffi::c_void, mem::transmute, num::NonZeroU64, sync::OnceLock};
use types::FFIError;
use window_info::WindowInfo;

static mut KELP: OnceLock<Kelp> = OnceLock::new();

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn create_empty_texture(width: u32, height: u32, out_id: &mut KelpTextureId) -> FFIError {
    match KELP.get_mut().map(|kelp| kelp.create_empty_texture(width, height)) {
        Some(value) => {
            *out_id = value;
            FFIError::Success
        }
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
pub unsafe extern "C" fn initialise(window: WindowInfo, imgui_config: *const c_void) -> FFIError {
    if KELP.get().is_some() {
        return FFIError::KelpAlreadyInitialised;
    }
    _ = env_logger::try_init();
    match Kelp::new(&window, window.width, window.height, transmute(imgui_config)) {
        Ok(kelp) => {
            _ = KELP.set(kelp);
            FFIError::Success
        }
        Err(err) => err.into(),
    }
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn present_frame() -> FFIError {
    match KELP.get_mut().map(Kelp::present_frame) {
        Some(Ok(_)) => FFIError::Success,
        Some(Err(err)) => err.into(),
        None => FFIError::KelpNotInitialised,
    }
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn render_list(
    target: Option<NonZeroU64>, // KelpTextureId
    camera: Camera,
    clear: Option<&KelpColor>,
    instances: FFISlice<InstanceGPU>,
    batches: FFISlice<InstanceBatch>,
) -> FFIError {
    match KELP.get_mut().map(|kelp| {
        kelp.render_list(RenderList {
            target: transmute(target),
            camera: (&camera).into(),
            clear: clear.map(Into::into),
            instances: instances.iter().map(Into::into).collect(),
            batches: batches.to_vec(),
        })
    }) {
        Some(Ok(_)) => FFIError::Success,
        Some(Err(err)) => err.into(),
        None => FFIError::KelpNotInitialised,
    }
}

#[ffi_function]
#[no_mangle]
pub unsafe extern "C" fn render_imgui(draw_data: *const c_void) -> FFIError {
    match KELP.get_mut().map(|kelp| kelp.render_imgui(transmute(draw_data))) {
        Some(Ok(_)) => FFIError::Success,
        Some(Err(err)) => err.into(),
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
