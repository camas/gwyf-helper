use std::{ffi::CString, iter, mem, ptr::null_mut, thread, time::Duration};

use winapi::{
    shared::minwindef::TRUE,
    um::{
        libloaderapi::GetModuleHandleW,
        libloaderapi::GetProcAddress,
        processthreadsapi::GetCurrentProcess,
        psapi::{GetModuleInformation, MODULEINFO},
    },
};

pub struct Module {
    _name: String,
    pub base_addr: usize,
    pub length: usize,
}

impl Module {
    /// Waits for the specified module to load
    pub fn find(name: &str) -> Self {
        // Encode name
        let sent_name = name
            .encode_utf16()
            .chain(iter::once(0))
            .collect::<Vec<u16>>();
        // Get module handle
        let mut handle = unsafe { GetModuleHandleW(sent_name.as_ptr()) };
        while handle.is_null() {
            thread::sleep(Duration::from_millis(100));
            handle = unsafe { GetModuleHandleW(sent_name.as_ptr()) };
        }
        // Get process handle
        let process = unsafe { GetCurrentProcess() };
        // Get module information
        let mut module_info = MODULEINFO {
            lpBaseOfDll: null_mut(),
            SizeOfImage: 0,
            EntryPoint: null_mut(),
        };
        if unsafe {
            GetModuleInformation(
                process,
                handle,
                &mut module_info as *mut _,
                mem::size_of::<MODULEINFO>() as u32,
            )
        } != TRUE
        {
            panic!("Error getting module information");
        }
        assert_eq!(module_info.lpBaseOfDll as usize, handle as usize);

        Self {
            _name: name.to_string(),
            base_addr: handle as usize,
            length: module_info.SizeOfImage as usize,
        }
    }

    pub fn _get_proc_address(&self, proc_name: &str) -> usize {
        let proc_name = CString::new(proc_name).unwrap();
        unsafe { GetProcAddress(self.base_addr as *mut _, proc_name.as_ptr()) as usize }
    }
}
