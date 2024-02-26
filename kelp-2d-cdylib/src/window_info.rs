use core::ffi::c_void;
use interoptopus::ffi_type;
use raw_window_handle::{
    AppKitDisplayHandle, AppKitWindowHandle, DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle,
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, Win32WindowHandle, WindowHandle,
    WindowsDisplayHandle, XlibDisplayHandle, XlibWindowHandle,
};
use std::{num::NonZeroIsize, ptr::NonNull};

#[ffi_type]
#[repr(C)]
#[derive(Debug)]
pub enum WindowType {
    Win32 = 0,
    Xlib = 1,
    Wayland = 2,
    AppKit = 3,
}

#[ffi_type]
#[repr(C)]
#[derive(Debug)]
pub struct WindowInfo {
    pub window_type: WindowType,
    pub window_handle: *mut c_void,
    pub second_handle: *mut c_void,
    pub width: u32,
    pub height: u32,
}

impl HasWindowHandle for WindowInfo {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let raw = match self.window_type {
            WindowType::Win32 => {
                let hwnd = unsafe { NonZeroIsize::new_unchecked(self.window_handle as isize) };
                let mut win32_handle = Win32WindowHandle::new(hwnd);
                win32_handle.hinstance = NonZeroIsize::new(self.second_handle as isize);
                RawWindowHandle::Win32(win32_handle)
            }
            WindowType::Xlib => RawWindowHandle::Xlib(XlibWindowHandle::new(self.window_handle as u32)),
            WindowType::Wayland => RawWindowHandle::Wayland(WaylandWindowHandle::new(unsafe {
                NonNull::new_unchecked(self.window_handle)
            })),
            WindowType::AppKit => {
                RawWindowHandle::AppKit(AppKitWindowHandle::new(unsafe { NonNull::new_unchecked(self.window_handle) }))
            }
        };
        Ok(unsafe { WindowHandle::borrow_raw(raw) })
    }
}

impl HasDisplayHandle for WindowInfo {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        let raw = match self.window_type {
            WindowType::Win32 => RawDisplayHandle::Windows(WindowsDisplayHandle::new()),
            WindowType::Xlib => RawDisplayHandle::Xlib(XlibDisplayHandle::new(NonNull::new(self.second_handle), 0)),
            WindowType::Wayland => RawDisplayHandle::Wayland(WaylandDisplayHandle::new(unsafe {
                NonNull::new_unchecked(self.second_handle)
            })),
            WindowType::AppKit => RawDisplayHandle::AppKit(AppKitDisplayHandle::new()),
        };
        Ok(unsafe { DisplayHandle::borrow_raw(raw) })
    }
}
