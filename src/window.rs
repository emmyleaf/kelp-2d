use std::ffi::c_void;

use raw_window_handle::{HasRawWindowHandle, RawWindowHandle, Win32Handle};

#[repr(C)]
pub struct Window {
    pub window: *mut c_void,
    pub secondary: *mut c_void,
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        // TODO: enum of supported types to be sent through ffi
        // TODO: handle all supported cases in a switch here
        // Example of encoding for win32
        let mut win32_handle = Win32Handle::empty();
        win32_handle.hwnd = self.window;
        win32_handle.hinstance = self.secondary;
        return RawWindowHandle::Win32(win32_handle);
    }
}
