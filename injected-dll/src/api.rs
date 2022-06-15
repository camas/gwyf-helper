use crate::{
    gamestructs::{Il2CppString, BASE_ADDRESS},
    offsets::OFFSETS,
};

#[allow(clippy::mut_from_ref)]
pub fn il2cpp_array_new(element_type_info: &Il2CppClass, size: u64) -> &mut Il2CppArray {
    unsafe {
        let method = std::mem::transmute::<
            _,
            extern "system" fn(*const Il2CppClass, u64) -> *mut Il2CppArray,
        >(BASE_ADDRESS.offset(OFFSETS.api("il2cpp_array_new")));
        &mut *method(element_type_info, size)
    }
}

pub fn il2cpp_string_new(value: &'static [u8]) -> &mut Il2CppString {
    unsafe {
        let method = std::mem::transmute::<_, extern "system" fn(*const u8) -> *mut Il2CppString>(
            BASE_ADDRESS.offset(OFFSETS.api("il2cpp_string_new")),
        );
        &mut *method(value.as_ptr())
    }
}

#[repr(C)]
pub struct Il2CppClass {}

#[repr(C)]
pub struct Il2CppArray {
    pub klass: *const u8,
    pub monitor: *const u8,
    pub bounds: *const u8,
    pub max_length: u64,
    pub vector: [*const u8; 32],
}
