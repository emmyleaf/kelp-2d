use core::ffi::c_void;
use raw_window_handle::{
    AppKitHandle, HasRawWindowHandle, RawWindowHandle, WaylandHandle, Win32Handle, WinRtHandle, XlibHandle,
};

#[repr(u8)]
pub enum WindowType {
    Win32 = 0,
    WinRt = 1,
    Xlib = 2,
    Wayland = 3,
    AppKit = 4,
}

#[repr(C)]
pub struct Window {
    pub window_type: WindowType,
    pub window_handle: *mut c_void,
    pub secondary: *mut c_void,
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        match self.window_type {
            WindowType::Win32 => {
                let mut win32_handle = Win32Handle::empty();
                win32_handle.hwnd = self.window_handle;
                win32_handle.hinstance = self.secondary;
                RawWindowHandle::Win32(win32_handle)
            }
            WindowType::WinRt => {
                let mut winrt_handle = WinRtHandle::empty();
                winrt_handle.core_window = self.window_handle;
                RawWindowHandle::WinRt(winrt_handle)
            }
            WindowType::Xlib => {
                let mut xlib_handle = XlibHandle::empty();
                xlib_handle.window = self.window_handle as u32;
                xlib_handle.display = self.secondary;
                RawWindowHandle::Xlib(xlib_handle)
            }
            WindowType::Wayland => {
                let mut wayland_handle = WaylandHandle::empty();
                wayland_handle.surface = self.window_handle;
                wayland_handle.display = self.secondary;
                RawWindowHandle::Wayland(wayland_handle)
            }
            WindowType::AppKit => {
                let mut appkit_handle = AppKitHandle::empty();
                appkit_handle.ns_window = self.window_handle;
                appkit_handle.ns_view = self.secondary;
                RawWindowHandle::AppKit(appkit_handle)
            }
        }
    }
}
