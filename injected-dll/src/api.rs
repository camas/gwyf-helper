use crate::{
    gamestructs::{Il2CppString, BASE_ADDRESS},
    offsets::api_offset,
};

pub fn il2cpp_string_new(value: &'static [u8]) -> &Il2CppString {
    unsafe {
        let method = std::mem::transmute::<_, extern "system" fn(*const u8) -> *mut Il2CppString>(
            BASE_ADDRESS.offset(api_offset!(il2cpp_string_new)),
        );
        &mut *method(value.as_ptr())
    }
}

pub fn il2cpp_domain_get() -> *const u8 {
    unsafe {
        let method = std::mem::transmute::<_, extern "system" fn() -> *const u8>(
            BASE_ADDRESS.offset(api_offset!(il2cpp_domain_get)),
        );
        method()
    }
}

pub fn il2cpp_thread_attach(domain: *const u8) {
    unsafe {
        let method = std::mem::transmute::<_, extern "system" fn(*const u8)>(
            BASE_ADDRESS.offset(api_offset!(il2cpp_thread_attach)),
        );
        method(domain)
    }
}
