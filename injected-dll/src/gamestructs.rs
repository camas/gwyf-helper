use std::ptr::null;

use crate::{
    api::{il2cpp_string_new, Il2CppArray},
    module::Module,
};

pub static mut BASE_ADDRESS: *const u8 = null();

pub fn init() {
    let module = Module::new("GameAssembly.dll");
    unsafe {
        BASE_ADDRESS = module.base_addr as *const u8;
        let il2cpp_domain_get = std::mem::transmute::<_, extern "system" fn() -> *const ()>(
            BASE_ADDRESS.offset(0x001c7360),
        );
        let il2cpp_thread_attach = std::mem::transmute::<_, extern "system" fn(*const ())>(
            BASE_ADDRESS.offset(0x001c8540),
        );
        il2cpp_thread_attach(il2cpp_domain_get());
    }
}

#[repr(C)]
pub struct VirtualInvokeData {
    method_ptr: *const u8,
    method: *const MethodInfo,
}

#[repr(C)]
pub struct MethodInfo {}

#[repr(C)]
pub struct GameState__Class {
    _filler1: [u8; 184],
    pub static_fields: *const GameState__StaticFields,
}

impl GameState__Class {
    pub fn get() -> &'static GameState__Class {
        unsafe { &**(BASE_ADDRESS.offset(0x02aa4648) as *const *const GameState__Class) }
    }
}

#[repr(C)]
pub struct GameState__StaticFields {
    pub shared_context_info: *const GameContextInformation,
}

#[repr(C)]
pub struct GameContextInformation {
    _filler1: [u8; 16],
    pub online_data: OnlineStateInformation,
    pub region_infos: *const (),
    pub session_infos: *const (),
    pub session_filter: *const (),
    pub session_settings: *const (),
    pub session_info: *const (),
    pub user_infos: *const UserInfosData,
    pub game_management: *const (),
    pub gameplay_settings: *const (),
    pub game_mode_datas: *const (),
    pub options: *const (),
    pub level_editor: *const (),
}

#[repr(C)]
pub struct OnlineStateInformation {
    pub connection_region: i32,
    pub local_player_account_details: *const (),
    pub local_player_id: *const (),
}

#[repr(C)]
pub struct UserInfosData {
    pub klass: *const UserInfosData__Class,
}

#[repr(C)]
pub struct UserInfosData__Class {
    _filler: [u8; 304],
    pub vtable: UserInfosData__VTable,
}

#[repr(C)]
pub struct UserInfosData__VTable {
    _filler1: [u8; 64],
    pub get_item: VirtualInvokeData,
    _filler2: [u8; 64],
    pub get_count: VirtualInvokeData,
}

impl UserInfosData {
    pub fn count(&self) -> i32 {
        unsafe {
            let vid = &(*self.klass).vtable.get_count;
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const UserInfosData, *const MethodInfo) -> i32,
            >(vid.method_ptr);
            method(self, vid.method)
        }
    }

    pub fn get(&self, index: i32) -> &UserInfo {
        unsafe {
            let vid = &(*self.klass).vtable.get_item;
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const UserInfosData, i32, *const MethodInfo) -> *const UserInfo,
            >(vid.method_ptr);
            &*method(self, index, vid.method)
        }
    }
}

#[repr(C)]
pub struct UserInfo {}

impl UserInfo {
    pub fn user(&self) -> &User {
        unsafe { &*(self as *const _ as *const User) }
    }
}

#[repr(C)]
pub struct User {
    pub klass: *const u8,
    pub monitor: *const u8,
    pub fields: User__Fields,
}

#[repr(C)]
pub struct User__Fields {
    pub m_b_is_local: bool,
    pub m_b_is_primary: bool,
    pub m_photon_player: *const PhotonPlayer,
    pub m_i_local_player_id: i32,
    pub m_platform_id: *const u8,
    pub m_display_name: *const Il2CppString,
    pub profile_picture_backing_field: *const u8,
    pub m_ball: *const u8,
    pub m_player_camera: *const u8,
    pub m_network_state: i32,
    pub m_game_flow_state: i32,
    pub m_loading_level_complete: i32,
    pub m_fully_loaded: bool,
    pub m_password: *const Il2CppString,
    pub m_colour: Color,
    pub m_customisation_items: *const u8,
    pub m_hit_counter: i32,
    pub m_in_hole: bool,
    pub m_taking_penalty: bool,
    pub hole_scores: *const Int32__Array,
}

impl User {
    pub fn display_name(&self) -> &Il2CppString {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const User, *const MethodInfo) -> *const Il2CppString,
            >(BASE_ADDRESS.offset(0x00322f10));
            &*method(self, null())
        }
    }

    pub fn ball(&self) -> &BallMovement {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const User, *const MethodInfo) -> *const BallMovement,
            >(BASE_ADDRESS.offset(0x00323070));
            &*method(self, null())
        }
    }

    pub fn photon_player(&self) -> &PhotonPlayer {
        unsafe { &*self.fields.m_photon_player }
    }

    pub fn set_color(&self, color: &Color) {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const User, *const Color, *const MethodInfo),
            >(BASE_ADDRESS.offset(0x00323a90));
            method(self, color, null());
        }
    }

    pub fn update_properties(&self) {
        unsafe {
            let method = std::mem::transmute::<_, extern "system" fn(*const User, *const MethodInfo)>(
                BASE_ADDRESS.offset(0x00324390),
            );
            method(self, null());
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct Il2CppString {
    _filler1: [u8; 16],
    length: u32,
    first_char: u16,
}

impl Il2CppString {
    pub fn read(&self) -> String {
        unsafe {
            let offset = &self.first_char as *const _ as usize;
            let bytes = (offset..)
                .step_by(std::mem::size_of::<u16>())
                .take(self.length as usize)
                .map(|i| *(i as *const u16))
                .collect::<Vec<_>>();
            String::from_utf16(&bytes).unwrap()
        }
    }
}

#[derive(Debug, PartialEq)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[repr(C)]
pub struct UserService {}

impl UserService {
    pub fn get() -> &'static UserService {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const MethodInfo) -> *const UserService,
            >(BASE_ADDRESS.offset(0x00625eb0));
            &*method(null())
        }
    }

    pub fn primary_local_user(&self) -> &User {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const UserService, *const MethodInfo) -> *const User,
            >(BASE_ADDRESS.offset(0x00328010));
            &*method(self, null())
        }
    }
}

#[repr(C)]
pub struct Int32__Array {
    pub klass: *const u8,
    pub monitor: *const u8,
    pub bounds: *const u8,
    pub max_length: u64,
    pub vector: [i32; 32],
}

impl Int32__Array {
    pub fn values(&self) -> &[i32] {
        &self.vector[0..self.max_length as usize]
    }
}

#[repr(C)]
pub struct BallMovement {
    pub klass: *const u8,
    pub monitor: *const u8,
    pub fields: BallMovement__Fields,
}

#[repr(C)]
pub struct BallMovement__Fields {
    _filler1: [u8; 680],
    pub m_rigid_body: *const RigidBody,
    _filler2: [u8; 40],
    pub m_network_ball_sync: *const NetworkBallSync,
}

impl BallMovement {
    pub fn hole_number(&self) -> i32 {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const BallMovement, *const MethodInfo) -> i32,
            >(BASE_ADDRESS.offset(0x003ee9b0));
            method(self, null())
        }
    }

    pub fn rigid_body(&self) -> &RigidBody {
        unsafe { &*self.fields.m_rigid_body }
    }

    pub fn network_sync(&self) -> &NetworkBallSync {
        unsafe { &*self.fields.m_network_ball_sync }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for Vector3 {
    fn default() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 0.,
        }
    }
}

impl Vector3 {
    pub fn new() -> Self {
        Self::default()
    }
}

#[repr(C)]
pub struct NetworkBallSync {
    pub klass: *const u8,
    pub monitor: *const u8,
    pub fields: NetworkBallSync__Fields,
}

#[repr(C)]
pub struct NetworkBallSync__Fields {
    _filler1: [u8; 208],
    pub rb: *const u8,
    pub bm: *const u8,
    pub pv: *const PhotonView,
}

impl NetworkBallSync {
    pub fn pv(&self) -> &PhotonView {
        unsafe { &*self.fields.pv }
    }
}

#[repr(C)]
pub struct PhotonPlayer {
    pub klass: *const u8,
    pub monitor: *const u8,
    pub fields: PhotonPlayer__Fields,
}

#[repr(C)]
pub struct PhotonPlayer__Fields {
    pub actor_id: i32,
    pub name_field: *const Il2CppString,
    pub used_id_backing_field: *const Il2CppString,
    pub is_local: bool,
    pub in_inactive_backing_field: bool,
    pub tag_object: *const u8,
}

#[repr(C)]
pub struct PhotonView {}

impl PhotonView {
    pub fn rpc(&self, name: &'static [u8], target: PhotonTargets, parameters: &Il2CppArray) {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(
                    *const PhotonView,
                    *const Il2CppString,
                    i32,
                    *const Il2CppArray,
                    *const MethodInfo,
                ),
            >(BASE_ADDRESS.offset(0x010ed620));
            method(
                self,
                il2cpp_string_new(name),
                target as i32,
                parameters,
                null(),
            );
        }
    }
}

#[repr(C)]
pub enum PhotonTargets {
    All,
    Others,
    MasterClient,
    AllBuffered,
    OthersBuffered,
    AllViaServer,
    AllBufferedViaServer,
}

#[repr(C)]
pub struct RigidBody {
    pub klass: *const u8,
    pub monitor: *const u8,
    pub fields: RigidBody__Fields,
}

#[repr(C)]
pub struct RigidBody__Fields {}

impl RigidBody {
    pub fn position(&self) -> Vector3 {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const RigidBody, *const MethodInfo) -> Vector3,
            >(BASE_ADDRESS.offset(0x01b64e70));
            method(self, null())
        }
    }

    pub fn set_position(&self, position: &Vector3) {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const RigidBody, *const Vector3, *const MethodInfo),
            >(BASE_ADDRESS.offset(0x01b64ef0));
            method(self, position, null());
        }
    }

    pub fn set_velocity(&self, velocity: &Vector3) {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const RigidBody, *const Vector3, *const MethodInfo),
            >(BASE_ADDRESS.offset(0x01b648a0));
            method(self, velocity, null());
        }
    }
}
