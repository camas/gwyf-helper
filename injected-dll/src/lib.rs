#![allow(clippy::zst_offset)]
#![feature(strict_provenance)]
#![feature(inline_const)]

use std::ffi::c_void;

use hudhook::hooks::{dx11::ImguiDx11Hooks, ImguiRenderLoop};
use log::{error, info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode};
use winapi::{
    shared::minwindef::HINSTANCE,
    um::{libloaderapi::DisableThreadLibraryCalls, winnt},
};

use crate::ui::State;

mod api;
mod gamestructs;
mod module;
mod offsets;
mod ui;

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "stdcall" fn DllMain(dll_hinst: HINSTANCE, fdw_reason: u32, _reserved: c_void) {
    if fdw_reason != winnt::DLL_PROCESS_ATTACH {
        return;
    }
    unsafe {
        DisableThreadLibraryCalls(dll_hinst);
    }
    let dll_hinst_address = dll_hinst.addr();
    std::thread::spawn(move || main(dll_hinst_address));
}

pub fn main(dll_hinst_address: usize) {
    // Enable console if in debug mode
    #[cfg(debug_assertions)]
    unsafe {
        winapi::um::consoleapi::AllocConsole();
    }

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    if let Err(e) = hudhook::Hudhook::builder()
        .with(State::default().into_hook::<ImguiDx11Hooks>())
        .with_hmodule(hudhook::HINSTANCE(dll_hinst_address as isize))
        .build()
        .apply()
    {
        error!("Couldn't apply hooks: {e:?}");
    }

    info!("gwyf helper running!");
}
