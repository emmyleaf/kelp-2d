use core::ffi::c_void;
use raw_window_handle::{AppKitHandle, HasRawWindowHandle, RawWindowHandle, WaylandHandle, Win32Handle, XlibHandle};

#[repr(u8)]
#[derive(Debug)]
pub enum WindowType {
    Win32 = 0,
    Xlib = 1,
    Wayland = 2,
    AppKit = 3,
}

#[repr(C)]
#[derive(Debug)]
pub struct Window {
    pub window_type: WindowType,
    pub window_handle: *mut c_void,
    pub second_handle: *mut c_void,
    pub width: u32,
    pub height: u32,
}

unsafe impl HasRawWindowHandle for Window {
    fn raw_window_handle(&self) -> RawWindowHandle {
        match self.window_type {
            WindowType::Win32 => {
                let mut win32_handle = Win32Handle::empty();
                win32_handle.hwnd = self.window_handle;
                win32_handle.hinstance = self.second_handle;
                RawWindowHandle::Win32(win32_handle)
            }
            WindowType::Xlib => {
                let mut xlib_handle = XlibHandle::empty();
                xlib_handle.window = self.window_handle as u32;
                xlib_handle.display = self.second_handle;
                RawWindowHandle::Xlib(xlib_handle)
            }
            WindowType::Wayland => {
                let mut wayland_handle = WaylandHandle::empty();
                wayland_handle.surface = self.window_handle;
                wayland_handle.display = self.second_handle;
                RawWindowHandle::Wayland(wayland_handle)
            }
            WindowType::AppKit => {
                let mut appkit_handle = AppKitHandle::empty();
                appkit_handle.ns_window = self.window_handle;
                appkit_handle.ns_view = self.second_handle;
                RawWindowHandle::AppKit(appkit_handle)
            }
        }
    }
}
