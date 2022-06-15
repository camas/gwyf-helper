use std::{
    ffi::c_void,
    mem,
    ptr::{null_mut, NonNull},
    time::Instant,
};

use detour::static_detour;
use imgui::{FontConfig, FontSource, Ui};
use imgui_dx11_renderer::Renderer;
use log::info;
use winapi::{
    shared::{
        dxgi::{
            IDXGISwapChain, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH,
            DXGI_SWAP_EFFECT_DISCARD,
        },
        dxgiformat::DXGI_FORMAT_R8G8B8A8_UNORM,
        dxgitype::{
            DXGI_MODE_DESC, DXGI_MODE_SCALING_UNSPECIFIED, DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
            DXGI_RATIONAL, DXGI_SAMPLE_DESC, DXGI_USAGE_RENDER_TARGET_OUTPUT,
        },
        minwindef::LRESULT,
        minwindef::TRUE,
        minwindef::UINT,
        minwindef::WPARAM,
        minwindef::{BOOL, DWORD, FALSE, LPARAM},
        ntdef::HRESULT,
        windef::HWND,
        windef::POINT,
    },
    um::{
        d3d11::{D3D11CreateDeviceAndSwapChain, D3D11_SDK_VERSION},
        d3d11::{
            ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11Texture2D,
            IID_ID3D11Device, IID_ID3D11Texture2D,
        },
        d3dcommon::{
            D3D_DRIVER_TYPE_HARDWARE, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_10_1,
            D3D_FEATURE_LEVEL_11_0,
        },
        processthreadsapi::GetCurrentProcessId,
        winuser::CallWindowProcW,
        winuser::GET_XBUTTON_WPARAM,
        winuser::GWL_WNDPROC,
        winuser::{
            self, EnumWindows, GetCapture, GetCursorPos, GetWindowThreadProcessId, ScreenToClient,
            SetCapture, SetWindowLongPtrW, RAWINPUT, RID_INPUT, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
            WM_INPUT, XBUTTON1,
        },
    },
};
use winuser::{GetForegroundWindow, GetRawInputData, ReleaseCapture, RAWINPUTHEADER};

use self::{
    imgui_winit_support::{HiDpiMode, WinitPlatform},
    window::Window,
};

mod imgui_winit_support;
mod window;

static_detour! {
    static SwapChainPresent: extern "system" fn (*mut IDXGISwapChain, UINT, UINT) -> HRESULT;
}

static mut ORIGINAL_WINDOW_PROC: isize = 0;

pub fn setup<F>(draw_callback: F)
where
    F: Fn(&Ui) + 'static,
{
    unsafe {
        DRAW_CALLBACK = Some(Box::new(draw_callback));
    }
    // Get d3d11 vtable
    let vtable = get_vtable();

    // Hook `Present` function
    unsafe {
        SwapChainPresent
            .initialize(
                mem::transmute(*vtable.get(8).unwrap()),
                swap_chain_present_hook,
            )
            .unwrap()
            .enable()
            .unwrap();
    }

    info!("Hooked d3d11");
}

// bad hack
static mut INIT: bool = false;
static mut RENDERER: Option<Renderer> = None;
static mut LAST_FRAME: Option<Instant> = None;
static mut IMGUI: Option<imgui::Context> = None;
static mut PLATFORM: Option<WinitPlatform> = None;
static mut WINDOW_HWND: HWND = null_mut();
static mut DRAW_CALLBACK: Option<Box<dyn Fn(&Ui)>> = None;

fn swap_chain_present_hook(
    swap_chain: *mut IDXGISwapChain,
    sync_interval: UINT,
    flags: UINT,
) -> HRESULT {
    // Get drawing structs
    let mut device: *mut ID3D11Device = null_mut();
    let mut context: *mut ID3D11DeviceContext = null_mut();
    let mut back_buffer: *mut ID3D11Texture2D = null_mut();
    let mut render_target_view: *mut ID3D11RenderTargetView = null_mut();
    unsafe {
        (*swap_chain).GetDevice(
            &IID_ID3D11Device as *const _,
            &mut device as *mut _ as *mut _,
        );
        (*device).GetImmediateContext(&mut context as *mut _);
        (*swap_chain).GetBuffer(
            0,
            &IID_ID3D11Texture2D as *const _,
            &mut back_buffer as *mut _ as *mut _,
        );
        (*device).CreateRenderTargetView(
            back_buffer as *mut _,
            null_mut(),
            &mut render_target_view as *mut _,
        );
        (*back_buffer).Release();
    }

    // Initialise
    if unsafe { !INIT } {
        unsafe {
            INIT = true;
            LAST_FRAME = Some(Instant::now());
            IMGUI = Some(imgui::Context::create());
        }
        let imgui = unsafe { IMGUI.as_mut().unwrap() };
        let mut app_data = dirs::config_dir().unwrap();
        app_data.push("gwyf_helper.ini");
        imgui.set_ini_filename(app_data);

        let mut desc = DXGI_SWAP_CHAIN_DESC {
            BufferDesc: DXGI_MODE_DESC {
                Width: 0,
                Height: 0,
                RefreshRate: DXGI_RATIONAL {
                    Numerator: 0,
                    Denominator: 0,
                },
                Format: 0,
                ScanlineOrdering: 0,
                Scaling: 0,
            },
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 0,
                Quality: 0,
            },
            BufferUsage: 0,
            BufferCount: 0,
            OutputWindow: null_mut(),
            Windowed: 0,
            SwapEffect: 0,
            Flags: 0,
        };
        unsafe {
            (*swap_chain).GetDesc(&mut desc as *mut _);
            WINDOW_HWND = desc.OutputWindow;
            ORIGINAL_WINDOW_PROC = SetWindowLongPtrW(
                desc.OutputWindow,
                GWL_WNDPROC,
                window_event_callback as *const () as isize,
            );
        }

        unsafe {
            PLATFORM = Some(WinitPlatform::init(imgui));
        }
        let platform = unsafe { PLATFORM.as_mut().unwrap() };
        platform.attach_window(
            imgui.io_mut(),
            &Window::from_hwnd(desc.OutputWindow),
            HiDpiMode::Rounded,
        );
        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13. * hidpi_factor) as f32;
        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(FontConfig {
                size_pixels: font_size,
                ..FontConfig::default()
            }),
        }]);
        imgui.io_mut().font_global_scale = (1. / hidpi_factor) as f32;

        // Hack to turn raw pointer into a windows ID3D11Device
        struct HackIUnknown(NonNull<c_void>);
        struct HackID3D11Device(HackIUnknown);
        let device = HackID3D11Device(HackIUnknown(NonNull::new(device as *mut c_void).unwrap()));
        unsafe {
            let device = std::mem::transmute(device);
            RENDERER = Some(imgui_dx11_renderer::Renderer::new(imgui, &device).unwrap());
            LAST_FRAME = Some(Instant::now());
        }
    }

    // Unwrap static variables
    let imgui = unsafe { IMGUI.as_mut().unwrap() };
    let renderer = unsafe { RENDERER.as_mut().unwrap() };
    let _platform = unsafe { PLATFORM.as_mut().unwrap() };
    let hwnd = unsafe { WINDOW_HWND.as_mut().unwrap() };
    let last_frame = unsafe { LAST_FRAME.as_mut().unwrap() };
    let draw_callback = unsafe { DRAW_CALLBACK.as_mut().unwrap() };

    // Update imgui
    let io = imgui.io_mut();
    let now = Instant::now();
    io.update_delta_time(now - *last_frame);
    unsafe {
        LAST_FRAME = Some(now);
    }
    unsafe {
        if GetForegroundWindow() == hwnd {
            let mut pos: POINT = POINT { x: 0, y: 0 };
            GetCursorPos(&mut pos);
            ScreenToClient(hwnd, &mut pos);
            io.mouse_pos = [pos.x as f32, pos.y as f32];
        }
    }

    // Draw
    let ui = imgui.frame();
    draw_callback(&ui);

    // Render
    unsafe {
        (*context).OMSetRenderTargets(1, &render_target_view, null_mut());
    }
    renderer.render(ui.render()).unwrap();

    // Call original function
    #[allow(unused_unsafe)]
    unsafe {
        SwapChainPresent.call(swap_chain, sync_interval, flags)
    }
}

fn window_event_callback(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // Unwrap static vars
    let imgui = unsafe { IMGUI.as_mut().unwrap() };

    let io = imgui.io_mut();

    // Handle event
    match msg {
        winuser::WM_LBUTTONDOWN
        | winuser::WM_LBUTTONDBLCLK
        | winuser::WM_RBUTTONDOWN
        | winuser::WM_RBUTTONDBLCLK
        | winuser::WM_MBUTTONDOWN
        | winuser::WM_MBUTTONDBLCLK
        | winuser::WM_XBUTTONDOWN
        | winuser::WM_XBUTTONDBLCLK => {
            let button = match msg {
                winuser::WM_LBUTTONDOWN | winuser::WM_LBUTTONDBLCLK => 0,
                winuser::WM_RBUTTONDOWN | winuser::WM_RBUTTONDBLCLK => 1,
                winuser::WM_MBUTTONDOWN | winuser::WM_MBUTTONDBLCLK => 2,
                winuser::WM_XBUTTONDOWN | winuser::WM_XBUTTONDBLCLK => {
                    if GET_XBUTTON_WPARAM(wparam) == XBUTTON1 {
                        3
                    } else {
                        4
                    }
                }
                _ => unreachable!(),
            };
            if !io.mouse_down.iter().any(|e| *e) && unsafe { GetCapture() }.is_null() {
                unsafe {
                    SetCapture(hwnd);
                }
            }
            io.mouse_down[button] = true;

            if io.want_capture_mouse {
                return 0;
            }
        }
        winuser::WM_LBUTTONUP
        | winuser::WM_RBUTTONUP
        | winuser::WM_MBUTTONUP
        | winuser::WM_XBUTTONUP => {
            let button = match msg {
                winuser::WM_LBUTTONUP => 0,
                winuser::WM_RBUTTONUP => 1,
                winuser::WM_MBUTTONUP => 2,
                winuser::WM_XBUTTONUP => {
                    if GET_XBUTTON_WPARAM(wparam) == XBUTTON1 {
                        3
                    } else {
                        4
                    }
                }
                _ => unreachable!(),
            };
            io.mouse_down[button] = false;
            if !io.mouse_down.iter().any(|e| *e) && unsafe { GetCapture() } == hwnd {
                unsafe {
                    ReleaseCapture();
                }
            }

            if io.want_capture_mouse {
                return 0;
            }
        }
        _ => (),
    }

    // Check if imgui captured input
    if msg == WM_INPUT {
        let mut raw_input: RAWINPUT = unsafe { mem::zeroed() };
        unsafe {
            GetRawInputData(
                lparam as *mut _,
                RID_INPUT,
                &mut raw_input as *mut _ as *mut _,
                &mut mem::size_of::<RAWINPUT>() as *mut _ as *mut _,
                mem::size_of::<RAWINPUTHEADER>() as _,
            );
        }
        if io.want_capture_mouse && raw_input.header.dwType == RIM_TYPEMOUSE {
            return 0;
        }
        if io.want_capture_keyboard && raw_input.header.dwType == RIM_TYPEKEYBOARD {
            return 0;
        }
    }

    // Call original function
    unsafe {
        CallWindowProcW(
            Some(mem::transmute(ORIGINAL_WINDOW_PROC)),
            hwnd,
            msg,
            wparam,
            lparam,
        )
    }
}

fn get_vtable() -> Vec<*const usize> {
    // Get window pointer
    let window = get_process_window();

    // Create a dummy device to get the d3d11 vtable offset
    let buffer_desc = DXGI_MODE_DESC {
        Width: 100,
        Height: 100,
        RefreshRate: DXGI_RATIONAL {
            Numerator: 60,
            Denominator: 1,
        },
        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
        ScanlineOrdering: DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
        Scaling: DXGI_MODE_SCALING_UNSPECIFIED,
    };
    let sample_desc = DXGI_SAMPLE_DESC {
        Count: 1,
        Quality: 0,
    };
    let swap_chain_desc = DXGI_SWAP_CHAIN_DESC {
        BufferDesc: buffer_desc,
        SampleDesc: sample_desc,
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 1,
        OutputWindow: window as *mut _,
        Windowed: 1,
        SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
        Flags: DXGI_SWAP_CHAIN_FLAG_ALLOW_MODE_SWITCH,
    };
    let mut swap_chain: *mut IDXGISwapChain = null_mut();
    let mut device: *mut ID3D11Device = null_mut();
    let mut feature_level: D3D_FEATURE_LEVEL = 0;
    let mut context: *mut ID3D11DeviceContext = null_mut();

    let d3d11_result = unsafe {
        D3D11CreateDeviceAndSwapChain(
            null_mut(),
            D3D_DRIVER_TYPE_HARDWARE,
            null_mut(),
            0,
            &[D3D_FEATURE_LEVEL_10_1, D3D_FEATURE_LEVEL_11_0] as *const _,
            2,
            D3D11_SDK_VERSION,
            &swap_chain_desc as *const _,
            &mut swap_chain as *mut _ as *mut _,
            &mut device as *mut _ as *mut _,
            &mut feature_level as *mut _,
            &mut context as *mut _ as *mut _,
        )
    };
    if d3d11_result != 0 {
        panic!("Error creating dummy d3d11 device");
    }

    unsafe {
        [
            std::slice::from_raw_parts((swap_chain as *const *const *const usize).read(), 18),
            std::slice::from_raw_parts((device as *const *const *const usize).read(), 43),
            std::slice::from_raw_parts((context as *const *const *const usize).read(), 144),
        ]
    }
    .concat()
}

fn get_process_window() -> usize {
    extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let mut wnd_proc_id: DWORD = 0;
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut wnd_proc_id as *mut _);
            if GetCurrentProcessId() != wnd_proc_id {
                return TRUE;
            }
            *(lparam as *mut _) = hwnd;
        }
        FALSE
    }

    let mut hwnd: HWND = null_mut();
    unsafe {
        EnumWindows(Some(enum_windows_callback), &mut hwnd as *mut _ as LPARAM);
    }
    if hwnd.is_null() {
        panic!("Error finding current process window");
    }

    hwnd as usize
}
