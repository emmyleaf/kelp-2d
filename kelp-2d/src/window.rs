use core::ffi::c_void;
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle,
    RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
    XlibDisplayHandle, XlibWindowHandle,
};

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
                let mut win32_handle = Win32WindowHandle::empty();
                win32_handle.hwnd = self.window_handle;
                win32_handle.hinstance = self.second_handle;
                RawWindowHandle::Win32(win32_handle)
            }
            WindowType::Xlib => {
                let mut xlib_handle = XlibWindowHandle::empty();
                xlib_handle.window = self.window_handle as u32;
                RawWindowHandle::Xlib(xlib_handle)
            }
            WindowType::Wayland => {
                let mut wayland_handle = WaylandWindowHandle::empty();
                wayland_handle.surface = self.window_handle;
                RawWindowHandle::Wayland(wayland_handle)
            }
            WindowType::AppKit => {
                let mut appkit_handle = AppKitWindowHandle::empty();
                appkit_handle.ns_window = self.window_handle;
                appkit_handle.ns_view = self.second_handle;
                RawWindowHandle::AppKit(appkit_handle)
            }
        }
    }
}

unsafe impl HasRawDisplayHandle for Window {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        match self.window_type {
            WindowType::Win32 => RawDisplayHandle::Windows(WindowsDisplayHandle::empty()),
            WindowType::Xlib => {
                let mut xlib_handle = XlibDisplayHandle::empty();
                xlib_handle.display = self.second_handle;
                RawDisplayHandle::Xlib(xlib_handle)
            }
            WindowType::Wayland => {
                let mut wayland_handle = WaylandDisplayHandle::empty();
                wayland_handle.display = self.second_handle;
                RawDisplayHandle::Wayland(wayland_handle)
            }
            WindowType::AppKit => RawDisplayHandle::AppKit(AppKitDisplayHandle::empty()),
        }
    }
}
