use std::{ffi::c_void, marker::PhantomData, mem, ptr::null_mut};

use detour::static_detour;
use winapi::{
    shared::minwindef::LPVOID,
    um::{memoryapi::VirtualProtect, winnt::PAGE_EXECUTE_READWRITE},
};

use crate::module::Module;

static_detour! {
    static UserServiceCtor: extern "system" fn (*mut UserService, LPVOID);
    static WorldPowerBarCosmetic_OnShootPressed: extern "system" fn (*mut WorldPowerBarCosmetic, bool, LPVOID);
}

// pub static mut USER_SERVICE: *mut UserService = null_mut();
static mut BASE_ADDRESS: usize = 0;

#[repr(C)]
#[derive(Debug)]
pub struct UserService {
    _unk1: [u8; 16],
    pub users: *mut FastList_1_User_,
    pub local_users: *mut FastList_1_User_,
}

#[repr(C)]
#[derive(Debug)]
pub struct User {
    _unk1: [u8; 16],
    pub is_local: bool,
    pub is_primary: bool,
    _unk2: [u8; 26],
    pub display_name: *mut NetString,
    _unk3: [u8; 8],
    pub ball: *mut BallMovement,
    _unk4: [u8; 32],
    pub color: Color,
    _unk5: [u8; 8],
    pub hit_counter: i32,
    pub hole_scores: *mut Int32_Array,
    pub hit_force: f32,
    pub last_hit_force: f32,
    pub spin_force: f32,
    pub hole_time: i32,
    pub freecam_time: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ball() {
        let obj: BallMovement = unsafe { mem::zeroed() };
        let base = &obj._unk1 as *const _ as usize;
        println!("{:x}", base);
        println!(
            "+{:}",
            &obj.successfully_completed_hole as *const _ as usize - base
        );
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Int32_Array {
    _unk1: [u8; 32],
    pub vector: [i32; 32],
}

#[repr(C)]
#[derive(Debug)]
pub struct BallMovement {
    _class_unk: [u8; 16],
    _unk1: [u8; 80],
    pub player_position: Vector3,
    _unk2: [u8; 40],
    pub velocity: Vector3,
    _unk3: [u8; 136],
    pub max_hit_count: i32,
    pub max_time_penalty: i32,
    pub out_of_bounds: bool,
    pub is_master_ball: bool,
    pub is_shot_ready_to_start: bool,
    _unk4: [u8; 3],
    pub current_hole_num: i32,
    pub waiting_for_timer: bool,
    pub waiting_one_sec_flycam: bool,
    pub max_reset_time: f32,
    _unk5: [u8; 12],
    pub pre_hit_location: Vector3,
    pub oob_counting_down: bool,
    pub third_party_reset: bool,
    pub can_reset: bool,
    _unk6: [u8; 338],
    pub rigid_body: *mut RigidBody,
    _unk7: [u8; 376],
    pub successfully_completed_hole: bool,
}

#[repr(C)]
#[derive(Debug)]
pub struct RigidBody {
    _unk6: [u8; 16],
    pointer: *mut c_void,
}

#[repr(C)]
#[derive(Debug)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[repr(C)]
#[derive(Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[repr(C)]
#[derive(Debug)]
pub struct FastList_1_User_ {
    _unk1: [u8; 16],
    pub _items: *mut Array<User>,
    pub count: i32,
}

#[repr(C)]
#[derive(Debug)]
pub struct User__Array {
    _unk1: [u8; 32],
    pub vector: [*mut User; 32],
}

#[repr(C)]
#[derive(Debug)]
pub struct NetString {
    _unk1: [u8; 16],
    length: u32,
    first_char: u16,
}

#[repr(C)]
#[derive(Debug)]
pub struct GameObject {}

#[repr(C)]
#[derive(Debug)]
pub struct CameraFollow {}

#[repr(C)]
#[derive(Debug)]
pub struct PowerLevelIndicator {
    _class_unk: [u8; 16],
    _unk1: [u8; 64],
    pub max_force: *mut FloatData,
}

#[repr(C)]
#[derive(Debug)]
pub struct WorldPowerBarCosmetic {
    _class_unk: [u8; 16],
    _unk1: [u8; 168],
    current_user: *mut User,
}

#[repr(C)]
#[derive(Debug)]
pub struct FloatData {
    _class_unk: [u8; 16],
    _unk1: [u8; 8],
    pub value: f32,
}

#[repr(C)]
#[derive(Debug)]
pub struct Array<T> {
    phantom: PhantomData<T>,
}

impl<T> Array<T> {
    pub unsafe fn len(&self) -> i32 {
        std::mem::transmute::<_, extern "system" fn(*const Array<T>, LPVOID) -> i32>(
            BASE_ADDRESS + 0x010b8c60,
        )(self, null_mut())
    }

    pub unsafe fn get_values(&self) -> Vec<*mut T> {
        (0..self.len()).map(|i| self.get_value(i)).collect()
    }

    pub unsafe fn get_value(&self, index: i32) -> *mut T {
        std::mem::transmute::<_, extern "system" fn(*const Array<T>, i32, LPVOID) -> *mut T>(
            BASE_ADDRESS + 0x010b8ff0,
        )(self, index, null_mut())
    }
}

impl User {
    pub unsafe fn get_player_camera(&self) -> *mut CameraFollow {
        std::mem::transmute::<_, extern "system" fn(*const Self, LPVOID) -> *mut CameraFollow>(
            BASE_ADDRESS + 0x003234a0,
        )(self, null_mut())
    }
}

impl PowerLevelIndicator {
    pub unsafe fn get_power_percentage(&self) -> f32 {
        std::mem::transmute::<_, extern "system" fn(*const Self, LPVOID) -> f32>(
            BASE_ADDRESS + 0x00393e40,
        )(self, null_mut())
    }
}

impl CameraFollow {
    pub unsafe fn get_power_level_indicator(&self) -> *mut PowerLevelIndicator {
        std::mem::transmute::<_, extern "system" fn(*const Self, LPVOID) -> *mut PowerLevelIndicator>(
            BASE_ADDRESS + 0x004454f0,
        )(self, null_mut())
    }
}

impl GameObject {
    pub unsafe fn find_game_objects_by_tag(tag: &str) -> *mut Array<GameObject> {
        let tag = NetString::from_str(tag);
        std::mem::transmute::<_, extern "system" fn(*mut NetString, LPVOID) -> *mut Array<GameObject>>(
            BASE_ADDRESS + 0x013ad760,
        )(tag, null_mut())
    }
}

impl FastList_1_User_ {
    pub unsafe fn get_values(&self) -> Vec<*mut User> {
        if self._items.is_null() {
            return Vec::new();
        }
        (*self._items).get_values()
    }
}

impl RigidBody {
    pub fn exists(&self) -> bool {
        !self.pointer.is_null()
    }

    pub unsafe fn get_position(&self) -> Vector3 {
        let mut value: Vector3 = mem::zeroed();
        std::mem::transmute::<_, extern "system" fn(*const RigidBody, *mut Vector3, LPVOID)>(
            BASE_ADDRESS + 0x01b657c0,
        )(self, &mut value, null_mut());
        value
    }

    pub unsafe fn add_force(&self, force: &Vector3) {
        std::mem::transmute::<_, extern "system" fn(*const RigidBody, *const Vector3, i32, LPVOID)>(
            BASE_ADDRESS + 0x01b652d0,
        )(self, force, 0x02, null_mut());
    }
}

impl NetString {
    pub unsafe fn get_value(&self) -> String {
        let offset = &self.first_char as *const _ as usize;
        let bytes = (offset..)
            .step_by(mem::size_of::<u16>())
            .take(self.length as usize)
            .map(|i| *(i as *const u16))
            .collect::<Vec<_>>();
        String::from_utf16(&bytes).unwrap()
    }

    pub unsafe fn from_str(data: &str) -> *mut Self {
        let data_bytes = data.encode_utf16().collect::<Vec<_>>();
        let data_slice = data_bytes.as_slice();
        std::mem::transmute::<
            _,
            extern "system" fn(*const NetString, *const u16, i32, i32, LPVOID) -> *mut NetString,
        >(BASE_ADDRESS + 0x00f173e0)(
            null_mut(),
            data_slice.as_ptr(),
            0,
            data_slice.len() as i32,
            null_mut(),
        )
    }
}

impl UserService {
    pub unsafe fn get_instance() -> *mut Self {
        std::mem::transmute::<_, extern "system" fn() -> *mut UserService>(
            BASE_ADDRESS + 0x0029c290,
        )()
    }

    pub unsafe fn count(&self) -> i32 {
        std::mem::transmute::<_, extern "system" fn(*const UserService, LPVOID) -> i32>(
            BASE_ADDRESS + 0x00327AF0,
        )(self, null_mut())
    }

    pub unsafe fn get_users(&self) -> *mut Array<User> {
        let mut count = 0;
        std::mem::transmute::<
            _,
            extern "system" fn(*const UserService, *mut i32, LPVOID) -> *mut Array<User>,
        >(BASE_ADDRESS + 0x00328230)(self, &mut count, null_mut())
    }

    pub unsafe fn get_in_control_player(&self) -> *mut User {
        std::mem::transmute::<_, extern "system" fn(*const UserService, LPVOID) -> *mut User>(
            BASE_ADDRESS + 0x00328330,
        )(self, null_mut())
    }
}

pub fn setup(module: &Module) {
    unsafe {
        BASE_ADDRESS = module.base_addr;

        // Patch last power
        let address = (BASE_ADDRESS + 0x003fbd90) as *mut [u8; 3];
        let mut old: u32 = 0;
        VirtualProtect(address as *mut _, 3, PAGE_EXECUTE_READWRITE, &mut old);
        // JNZ -> JMP
        address.write([0x48, 0xe9, 0xae]);
        VirtualProtect(address as *mut _, 3, old, null_mut());
    }
}
