/// A modified version of winit's Window that only needs a HWND to initialize
use std::{ffi::c_void, io, mem, ptr, sync::atomic::AtomicBool, sync::atomic::Ordering};

use winapi::{
    shared::{
        minwindef::{BOOL, DWORD, FALSE, UINT},
        windef::{DPI_AWARENESS_CONTEXT, HMONITOR, HWND, LPRECT, POINT, RECT},
        winerror::S_OK,
    },
    um::{
        libloaderapi::{GetProcAddress, LoadLibraryA},
        shellscalingapi::MDT_EFFECTIVE_DPI,
        shellscalingapi::MONITOR_DPI_TYPE,
        shellscalingapi::PROCESS_DPI_AWARENESS,
        wingdi::{GetDeviceCaps, LOGPIXELSX},
        winnt::HRESULT,
        winnt::LPCSTR,
        winuser,
        winuser::MONITOR_DEFAULTTONEAREST,
    },
};
use winit::{
    dpi::PhysicalSize,
    dpi::Position,
    window::{CursorIcon, WindowId},
};

bitflags! {
    pub struct CursorFlags: u8 {
        const GRABBED   = 1 << 0;
        const HIDDEN    = 1 << 1;
        const IN_WINDOW = 1 << 2;
    }
}
pub const BASE_DPI: u32 = 96;

pub struct Window {
    hwnd: HWND,
    cursor_flags: CursorFlags,
    scale_factor: f64,
}

impl Window {
    pub fn from_hwnd(hwnd: HWND) -> Self {
        let mut result = Self {
            hwnd,
            cursor_flags: CursorFlags::empty(),
            scale_factor: 0.,
        };
        result.scale_factor = dpi_to_scale_factor(unsafe { result.hwnd_dpi() });
        result
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    #[inline]
    pub fn id(&self) -> WindowId {
        let window_id = unsafe { mem::transmute(self.hwnd) };
        println!("window id fetched {:?} {:?}", self.hwnd, window_id);
        window_id
    }

    pub fn inner_size(&self) -> PhysicalSize<u32> {
        let mut rect: RECT = unsafe { mem::zeroed() };
        if unsafe { winuser::GetClientRect(self.hwnd, &mut rect) } == 0 {
            panic!("Unexpected GetClientRect failure: please report this error to https://github.com/rust-windowing/winit")
        }
        PhysicalSize::new(
            (rect.right - rect.left) as u32,
            (rect.bottom - rect.top) as u32,
        )
    }

    #[inline]
    pub fn set_cursor_position(&self, position: Position) -> Result<(), ()> {
        let scale_factor = self.scale_factor();
        let (x, y) = position.to_physical::<i32>(scale_factor).into();

        let mut point = POINT { x, y };
        unsafe {
            if winuser::ClientToScreen(self.hwnd, &mut point) == 0 {
                return Err(());
            }
            if winuser::SetCursorPos(point.x, point.y) == 0 {
                return Err(());
            }
        }
        Ok(())
    }

    #[inline]
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.set_cursor_flags(|f| f.set(CursorFlags::HIDDEN, !visible))
            .map_err(|e| e.to_string())
            .ok();
    }

    #[inline]
    pub fn set_cursor_icon(&self, cursor: CursorIcon) {
        println!("set_cursor_icon call ignored");
    }

    pub fn set_cursor_flags<F>(&mut self, f: F) -> Result<(), io::Error>
    where
        F: FnOnce(&mut CursorFlags),
    {
        let old_flags = self.cursor_flags;
        f(&mut self.cursor_flags);
        match self.refresh_os_cursor() {
            Ok(()) => (),
            Err(e) => {
                self.cursor_flags = old_flags;
                return Err(e);
            }
        }

        Ok(())
    }

    fn refresh_os_cursor(&self) -> Result<(), io::Error> {
        let client_rect = self.get_client_rect()?;

        if self.is_focused() {
            let cursor_clip = match self.cursor_flags.contains(CursorFlags::GRABBED) {
                true => Some(client_rect),
                false => None,
            };

            let rect_to_tuple = |rect: RECT| (rect.left, rect.top, rect.right, rect.bottom);
            let active_cursor_clip = rect_to_tuple(get_cursor_clip()?);
            let desktop_rect = rect_to_tuple(get_desktop_rect());

            let active_cursor_clip = match desktop_rect == active_cursor_clip {
                true => None,
                false => Some(active_cursor_clip),
            };

            // We do this check because calling `set_cursor_clip` incessantly will flood the event
            // loop with `WM_MOUSEMOVE` events, and `refresh_os_cursor` is called by `set_cursor_flags`
            // which at times gets called once every iteration of the eventloop.
            if active_cursor_clip != cursor_clip.map(rect_to_tuple) {
                set_cursor_clip(cursor_clip)?;
            }
        }

        let cursor_in_client = self.cursor_flags.contains(CursorFlags::IN_WINDOW);
        if cursor_in_client {
            set_cursor_hidden(self.cursor_flags.contains(CursorFlags::HIDDEN));
        } else {
            set_cursor_hidden(false);
        }

        Ok(())
    }

    pub fn get_client_rect(&self) -> Result<RECT, io::Error> {
        unsafe {
            let mut rect = mem::zeroed();
            let mut top_left = mem::zeroed();

            win_to_err(|| winuser::ClientToScreen(self.hwnd, &mut top_left))?;
            win_to_err(|| winuser::GetClientRect(self.hwnd, &mut rect))?;
            rect.left += top_left.x;
            rect.top += top_left.y;
            rect.right += top_left.x;
            rect.bottom += top_left.y;

            Ok(rect)
        }
    }

    pub fn is_focused(&self) -> bool {
        self.hwnd == unsafe { winuser::GetActiveWindow() }
    }

    pub unsafe fn hwnd_dpi(&self) -> u32 {
        let hdc = winuser::GetDC(self.hwnd);
        if hdc.is_null() {
            panic!("[winit] `GetDC` returned null!");
        }
        if let Some(get_dpi_for_window) = *GET_DPI_FOR_WINDOW {
            // We are on Windows 10 Anniversary Update (1607) or later.
            match get_dpi_for_window(self.hwnd) {
                0 => BASE_DPI, // 0 is returned if hwnd is invalid
                dpi => dpi as u32,
            }
        } else if let Some(get_dpi_for_monitor) = *GET_DPI_FOR_MONITOR {
            // We are on Windows 8.1 or later.
            let monitor = winuser::MonitorFromWindow(self.hwnd, MONITOR_DEFAULTTONEAREST);
            if monitor.is_null() {
                return BASE_DPI;
            }

            let mut dpi_x = 0;
            let mut dpi_y = 0;
            if get_dpi_for_monitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) == S_OK {
                dpi_x as u32
            } else {
                BASE_DPI
            }
        } else {
            // We are on Vista or later.
            if winuser::IsProcessDPIAware() != FALSE {
                // If the process is DPI aware, then scaling must be handled by the application using
                // this DPI value.
                GetDeviceCaps(hdc, LOGPIXELSX) as u32
            } else {
                // If the process is DPI unaware, then scaling is performed by the OS; we thus return
                // 96 (scale factor 1.0) to prevent the window from being re-scaled by both the
                // application and the WM.
                BASE_DPI
            }
        }
    }
}

pub fn dpi_to_scale_factor(dpi: u32) -> f64 {
    dpi as f64 / BASE_DPI as f64
}

pub fn set_cursor_hidden(hidden: bool) {
    static HIDDEN: AtomicBool = AtomicBool::new(false);
    let changed = HIDDEN.swap(hidden, Ordering::SeqCst) ^ hidden;
    if changed {
        unsafe { winuser::ShowCursor(!hidden as BOOL) };
    }
}

pub fn get_cursor_clip() -> Result<RECT, io::Error> {
    unsafe {
        let mut rect: RECT = mem::zeroed();
        win_to_err(|| winuser::GetClipCursor(&mut rect)).map(|_| rect)
    }
}

/// Sets the cursor's clip rect.
///
/// Note that calling this will automatically dispatch a `WM_MOUSEMOVE` event.
pub fn set_cursor_clip(rect: Option<RECT>) -> Result<(), io::Error> {
    unsafe {
        let rect_ptr = rect
            .as_ref()
            .map(|r| r as *const RECT)
            .unwrap_or(ptr::null());
        win_to_err(|| winuser::ClipCursor(rect_ptr))
    }
}

pub fn get_desktop_rect() -> RECT {
    unsafe {
        let left = winuser::GetSystemMetrics(winuser::SM_XVIRTUALSCREEN);
        let top = winuser::GetSystemMetrics(winuser::SM_YVIRTUALSCREEN);
        RECT {
            left,
            top,
            right: left + winuser::GetSystemMetrics(winuser::SM_CXVIRTUALSCREEN),
            bottom: top + winuser::GetSystemMetrics(winuser::SM_CYVIRTUALSCREEN),
        }
    }
}

fn win_to_err<F: FnOnce() -> BOOL>(f: F) -> Result<(), io::Error> {
    if f() != 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

// Helper function to dynamically load function pointer.
// `library` and `function` must be zero-terminated.
fn get_function_impl(library: &str, function: &str) -> Option<*const c_void> {
    assert_eq!(library.chars().last(), Some('\0'));
    assert_eq!(function.chars().last(), Some('\0'));

    // Library names we will use are ASCII so we can use the A version to avoid string conversion.
    let module = unsafe { LoadLibraryA(library.as_ptr() as LPCSTR) };
    if module.is_null() {
        return None;
    }

    let function_ptr = unsafe { GetProcAddress(module, function.as_ptr() as LPCSTR) };
    if function_ptr.is_null() {
        return None;
    }

    Some(function_ptr as _)
}

macro_rules! get_function {
    ($lib:expr, $func:ident) => {
        get_function_impl(concat!($lib, '\0'), concat!(stringify!($func), '\0'))
            .map(|f| unsafe { std::mem::transmute::<*const _, $func>(f) })
    };
}

pub type SetProcessDPIAware = unsafe extern "system" fn() -> BOOL;
pub type SetProcessDpiAwareness =
    unsafe extern "system" fn(value: PROCESS_DPI_AWARENESS) -> HRESULT;
pub type SetProcessDpiAwarenessContext =
    unsafe extern "system" fn(value: DPI_AWARENESS_CONTEXT) -> BOOL;
pub type GetDpiForWindow = unsafe extern "system" fn(hwnd: HWND) -> UINT;
pub type GetDpiForMonitor = unsafe extern "system" fn(
    hmonitor: HMONITOR,
    dpi_type: MONITOR_DPI_TYPE,
    dpi_x: *mut UINT,
    dpi_y: *mut UINT,
) -> HRESULT;
pub type EnableNonClientDpiScaling = unsafe extern "system" fn(hwnd: HWND) -> BOOL;
pub type AdjustWindowRectExForDpi = unsafe extern "system" fn(
    rect: LPRECT,
    dwStyle: DWORD,
    bMenu: BOOL,
    dwExStyle: DWORD,
    dpi: UINT,
) -> BOOL;
lazy_static! {
    pub static ref GET_DPI_FOR_WINDOW: Option<GetDpiForWindow> =
        get_function!("user32.dll", GetDpiForWindow);
    pub static ref ADJUST_WINDOW_RECT_EX_FOR_DPI: Option<AdjustWindowRectExForDpi> =
        get_function!("user32.dll", AdjustWindowRectExForDpi);
    pub static ref GET_DPI_FOR_MONITOR: Option<GetDpiForMonitor> =
        get_function!("shcore.dll", GetDpiForMonitor);
    pub static ref ENABLE_NON_CLIENT_DPI_SCALING: Option<EnableNonClientDpiScaling> =
        get_function!("user32.dll", EnableNonClientDpiScaling);
    pub static ref SET_PROCESS_DPI_AWARENESS_CONTEXT: Option<SetProcessDpiAwarenessContext> =
        get_function!("user32.dll", SetProcessDpiAwarenessContext);
    pub static ref SET_PROCESS_DPI_AWARENESS: Option<SetProcessDpiAwareness> =
        get_function!("shcore.dll", SetProcessDpiAwareness);
    pub static ref SET_PROCESS_DPI_AWARE: Option<SetProcessDPIAware> =
        get_function!("user32.dll", SetProcessDPIAware);
}
