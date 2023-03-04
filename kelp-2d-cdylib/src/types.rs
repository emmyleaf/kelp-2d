use interoptopus::{
    ffi_type,
    patterns::{option::FFIOption, slice::FFISlice},
};
use kelp_2d::{BlendMode, InstanceData, KelpError, KelpTexture};

/// The main return type for unit returning functions with error handling
#[ffi_type(patterns(ffi_error))]
#[repr(C)]
pub enum FFIError {
    Success = 0,
    Null = 1,
    Panic = 2,
    // Kelp API specific errors
    NoCurrentFrame = 100,
    SwapchainError = 101,
    // Kelp FFI specific errors
    KelpAlreadyInitialised = 200,
    KelpNotInitialised = 201,
}

impl Default for FFIError {
    fn default() -> Self {
        Self::Success
    }
}

impl interoptopus::patterns::result::FFIError for FFIError {
    const SUCCESS: Self = Self::Success;
    const NULL: Self = Self::Null;
    const PANIC: Self = Self::Panic;
}

impl From<KelpError> for FFIError {
    fn from(error: KelpError) -> Self {
        match error {
            KelpError::NoCurrentFrame => FFIError::NoCurrentFrame,
            KelpError::SwapchainError(_) => FFIError::SwapchainError,
        }
    }
}

/// The main return type for for type returning functions with error handling
#[ffi_type]
#[repr(C)]
pub struct FFIResult<T> {
    value: FFIOption<T>,
    error: FFIError,
}

impl<T> FFIResult<T> {
    pub const fn ok(value: T) -> Self {
        Self { value: FFIOption::some(value), error: FFIError::Success }
    }
}

impl<T: Default> FFIResult<T> {
    pub fn error(error: FFIError) -> Self {
        Self { value: FFIOption::none(), error }
    }
}

impl<T: Default> From<Result<T, KelpError>> for FFIResult<T> {
    fn from(result: Result<T, KelpError>) -> Self {
        match result {
            Ok(value) => Self::ok(value),
            Err(error) => Self::error(error.into()),
        }
    }
}

/// A batch of instances to be added to a render pass
#[ffi_type]
#[repr(C)]
pub struct InstanceBatch<'a> {
    pub texture: &'a KelpTexture,
    pub smooth: bool,
    pub blend_mode: BlendMode,
    pub instances: FFISlice<'a, InstanceData>,
}
