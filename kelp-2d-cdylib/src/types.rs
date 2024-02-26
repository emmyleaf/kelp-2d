use interoptopus::ffi_type;
use kelp_2d::KelpError;

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
    InvalidTextureId = 102,
    InvalidTargetId = 103,
    InvalidBindGroupId = 104,
    InvalidPipelineId = 105,
    NoAdapter = 106,
    NoDevice = 107,
    NoImgui = 108,
    ImguiError = 109,
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
            KelpError::InvalidTextureId => FFIError::InvalidTextureId,
            KelpError::InvalidTargetId => FFIError::InvalidTargetId,
            KelpError::InvalidBindGroupId => FFIError::InvalidBindGroupId,
            KelpError::InvalidPipelineId => FFIError::InvalidPipelineId,
            KelpError::NoAdapter => FFIError::NoAdapter,
            KelpError::NoDevice(_) => FFIError::NoDevice,
            KelpError::NoImgui => FFIError::NoImgui,
            KelpError::ImguiError(_) => FFIError::ImguiError,
        }
    }
}
