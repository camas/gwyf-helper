use std::{
    ffi::{c_void, CString, OsString},
    mem,
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        raw::HANDLE,
    },
    ptr::null_mut,
    time::Duration,
};

use winapi::{
    shared::minwindef,
    um::{memoryapi, processthreadsapi, tlhelp32, winnt},
};

fn main() {
    // Wait for game to open and get pid
    let pid = match find_game() {
        Some(value) => value,
        None => {
            println!("Waiting for GWYF to open...");
            loop {
                std::thread::sleep(Duration::from_millis(200));
                if let Some(value) = find_game() {
                    break value;
                }
            }
        }
    };
    println!("GWYF open. PID {}. Injecting", pid);

    // Open process
    let proc = unsafe {
        processthreadsapi::OpenProcess(
            winnt::PROCESS_CREATE_THREAD
                | winnt::PROCESS_QUERY_INFORMATION
                | winnt::PROCESS_VM_OPERATION
                | winnt::PROCESS_VM_WRITE
                | winnt::PROCESS_VM_READ,
            minwindef::FALSE,
            pid,
        )
    };
    if proc.is_null() {
        println!("Failed to open process. Exiting");
        return;
    }

    // Extract dll
    #[cfg(debug_assertions)]
    let dll_bytes = include_bytes!("../../target/debug/injected_dll.dll");
    #[cfg(not(debug_assertions))]
    let dll_bytes = include_bytes!("../../target/release/injected_dll.dll");

    let mut temp_dll_path = std::env::temp_dir();
    temp_dll_path.push("gwyf_helper.dll");
    std::fs::write(&temp_dll_path, dll_bytes).expect("Error extracting dll for injection");

    let path = temp_dll_path.as_os_str();
    let path_size = ((path.len() + 1) * mem::size_of::<u16>()) as usize;

    // Allocate memory for dll path
    let remote_mem: HANDLE = unsafe {
        memoryapi::VirtualAllocEx(
            proc,
            null_mut(),
            path_size,
            winnt::MEM_COMMIT | winnt::MEM_RESERVE,
            winnt::PAGE_READWRITE,
        )
    };
    if remote_mem.is_null() {
        println!("Failed to allocate memory. Exiting");
        return;
    }

    // Write null-terminated dll path to newly allocated memory
    let path_to_write = path
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut bytes_written: winapi::shared::basetsd::SIZE_T = 0;
    let write_result = unsafe {
        memoryapi::WriteProcessMemory(
            proc,
            remote_mem,
            path_to_write.as_ptr() as *const std::ffi::c_void,
            path_size,
            &mut bytes_written as *mut _ as *mut winapi::shared::basetsd::SIZE_T,
        )
    };
    if write_result != minwindef::TRUE || bytes_written < path_size {
        println!("Error writing path to memory. Exiting");
        return;
    }

    // Start remote thread
    let kernel32_module = {
        let kernel32_string = CString::new("kernel32.dll").unwrap();
        unsafe { kernel32::GetModuleHandleA(kernel32_string.as_ptr()) }
    };
    if kernel32_module.is_null() {
        println!("Failed to find kernel32 module. Exiting");
        return;
    }
    let load_library_addr = {
        let load_library_str = CString::new("LoadLibraryW").unwrap();
        unsafe { kernel32::GetProcAddress(kernel32_module, load_library_str.as_ptr()) }
    };
    if load_library_addr.is_null() {
        println!("Failed to find LoadLibraryA address. Exiting");
        return;
    }
    let load_library = unsafe {
        std::mem::transmute::<
            *const c_void,
            unsafe extern "system" fn(lpThreadParameter: minwindef::LPVOID) -> minwindef::DWORD,
        >(load_library_addr)
    };
    let mut thread_id: minwindef::DWORD = 0;
    let thread_handle = unsafe {
        kernel32::CreateRemoteThread(
            proc,
            null_mut(),
            0,
            Some(load_library),
            remote_mem,
            0,
            &mut thread_id as *mut _ as *mut minwindef::DWORD,
        )
    };
    if thread_handle.is_null() {
        println!("Failed to start remote thread. Exiting");
        return;
    }
    println!("Injected and running");
}

fn find_game() -> Option<u32> {
    const TARGET_NAME: &str = "Golf With Your Friends.exe";

    // Get list of all processes
    let snapshot_handle: HANDLE =
        unsafe { tlhelp32::CreateToolhelp32Snapshot(tlhelp32::TH32CS_SNAPPROCESS, 0) };

    // Try find GWYF
    let mut entry = tlhelp32::PROCESSENTRY32W {
        dwSize: mem::size_of::<tlhelp32::PROCESSENTRY32W>() as u32,
        cntUsage: 0,
        th32ProcessID: 0,
        th32DefaultHeapID: 0,
        th32ModuleID: 0,
        cntThreads: 0,
        th32ParentProcessID: 0,
        pcPriClassBase: 0,
        dwFlags: 0,
        szExeFile: [0; minwindef::MAX_PATH],
    };
    if unsafe { tlhelp32::Process32FirstW(snapshot_handle, &mut entry) } == minwindef::TRUE {
        loop {
            let exe_file = OsString::from_wide(&entry.szExeFile);
            if exe_file.as_os_str().to_string_lossy().contains(TARGET_NAME) {
                return Some(entry.th32ProcessID);
            }
            if unsafe { tlhelp32::Process32NextW(snapshot_handle, &mut entry) } != minwindef::TRUE {
                break;
            }
        }
    }

    None
}
