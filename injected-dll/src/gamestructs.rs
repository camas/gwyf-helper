use std::ptr::null;

use log::info;

use crate::{module::Module, offsets::OFFSETS};

pub static mut BASE_ADDRESS: *const u8 = null();

pub fn init() {
    let module = Module::new("GameAssembly.dll");
    unsafe {
        BASE_ADDRESS = module.base_addr as *const u8;
        info!("base addr: {:#018x}", BASE_ADDRESS as usize);
        let il2cpp_domain_get = std::mem::transmute::<_, extern "system" fn() -> *const u8>(
            BASE_ADDRESS.offset(OFFSETS.api("il2cpp_domain_get")),
        );
        let il2cpp_thread_attach = std::mem::transmute::<_, extern "system" fn(*const u8)>(
            BASE_ADDRESS.offset(OFFSETS.api("il2cpp_thread_attach")),
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

pub struct Services {}

impl Services {
    pub fn get_user() -> &'static UserService {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const MethodInfo) -> *const UserService,
            >(BASE_ADDRESS.offset(OFFSETS.method("Services_get_Users")));
            &*method(null())
        }
    }

    pub fn get_game_state() -> &'static GameStateController {
        unsafe {
            let method =
                std::mem::transmute::<
                    _,
                    extern "system" fn(*const MethodInfo) -> *const GameStateController,
                >(BASE_ADDRESS.offset(OFFSETS.method("Services_get_GameState")));
            &*method(null())
        }
    }
}

#[repr(C)]
pub struct GameStateController {}

impl GameStateController {
    pub fn get_current_state(&self) -> &'static GameState {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(
                    *const GameStateController,
                    *const MethodInfo,
                ) -> *const GameState,
            >(
                BASE_ADDRESS.offset(OFFSETS.method("GameStateController_get_CurrentState")),
            );
            &*method(self, null())
        }
    }
}

#[repr(C)]
pub struct GameState {
    pub klass: *const GameState__Class,
}

impl GameState {
    pub fn klass(&self) -> &'static GameState__Class {
        unsafe { &*self.klass }
    }
}

#[repr(C)]
pub struct GameState__Class {
    _filler1: [u8; 184],
    pub static_fields: *const GameState__StaticFields,
}

impl GameState__Class {
    pub fn static_fields(&self) -> &GameState__StaticFields {
        unsafe { &*self.static_fields }
    }
}

#[repr(C)]
pub struct GameState__StaticFields {
    pub shared_context_info: *const GameContextInformation,
}

impl GameState__StaticFields {
    pub fn shared_context_info(&self) -> &GameContextInformation {
        unsafe { &*self.shared_context_info }
    }
}

#[repr(C)]
pub struct GameContextInformation {
    _filler1: [u8; 16],
    pub online_data: OnlineStateInformation,
    pub region_infos: *const u8,
    pub session_infos: *const u8,
    pub session_filter: *const u8,
    pub session_settings: *const u8,
    pub session_info: SessionInfo,
    pub user_infos: *const UserInfosData,
    pub game_management: *const u8,
    pub gameplay_settings: *const u8,
    pub game_mode_datas: *const u8,
    pub options: *const u8,
    pub level_editor: *const u8,
}

impl GameContextInformation {
    pub fn user_infos(&self) -> &UserInfosData {
        unsafe { &*self.user_infos }
    }
}

#[repr(C)]
pub struct SessionInfo {
    pub room_info_backing_field: *const u8,
}

impl SessionInfo {}

#[repr(C)]
pub struct OnlineStateInformation {
    pub connection_region: i32,
    pub local_player_account_details: *const u8,
    pub local_player_id: *const u8,
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
            >(BASE_ADDRESS.offset(OFFSETS.method("User_get_DisplayName")));
            &*method(self, null())
        }
    }

    pub fn ball(&self) -> &BallMovement {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const User, *const MethodInfo) -> *const BallMovement,
            >(BASE_ADDRESS.offset(OFFSETS.method("User_get_Ball")));
            &*method(self, null())
        }
    }

    pub fn hole_scores(&self) -> &Int32__Array {
        unsafe { &*self.fields.hole_scores }
    }

    pub fn photon_player(&self) -> &PhotonPlayer {
        unsafe { &*self.fields.m_photon_player }
    }

    pub fn player_camera(&self) -> &CameraFollow {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const User, *const MethodInfo) -> *const CameraFollow,
            >(BASE_ADDRESS.offset(OFFSETS.method("User_get_PlayerCamera")));
            &*method(self, null())
        }
    }

    pub fn game_flow_state(&self) -> GameFlowUserState {
        assert!(self.fields.m_game_flow_state <= GameFlowUserState::LevelEditor as i32);
        unsafe { std::mem::transmute(self.fields.m_game_flow_state) }
    }

    pub fn set_color(&self, color: &Color) {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const User, *const Color, *const MethodInfo),
            >(BASE_ADDRESS.offset(OFFSETS.method("User_set_Colour")));
            method(self, color, null());
        }
    }

    pub fn update_properties(&self) {
        unsafe {
            let method = std::mem::transmute::<_, extern "system" fn(*const User, *const MethodInfo)>(
                BASE_ADDRESS.offset(OFFSETS.method("User_UpdateProperties")),
            );
            method(self, null());
        }
    }
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub enum GameFlowUserState {
    Invalid,
    LevelSetup,
    LevelSetupRunning_ComponentCache,
    LevelSetupRunning_CullingSetup,
    LevelSetupFinished,
    SpawnPlayers,
    SpawnedPlayers,
    GameplayStarting,
    GameplayStarted,
    HoleStarting,
    HoleStarted,
    InHoleStarting,
    InHoleStarted,
    AllInHoleStarting,
    AllInHoleStarted,
    IntermissionStarting,
    IntermissionStarted,
    NextHoleStarting,
    NextHoleStarted,
    GameOutroStarting,
    GameOutroStarted,
    GameEnding,
    GameEnded,
    LevelEditor,
}

#[repr(C)]
pub struct CameraFollow {
    pub klass: *const u8,
    pub monitor: *const u8,
    pub fields: CameraFollow_Fields,
}

#[repr(C)]
pub struct CameraFollow_Fields {
    pub mono_behaviour_fields: *const u8,
    pub hitpoint: *const GameObject,
    pub pivot_point: *const GameObject,
    filler1: [u8; 76],
    pub player_pos: Vector3,
}

impl CameraFollow {
    pub fn pivot_point(&self) -> &GameObject {
        unsafe { &*self.fields.pivot_point }
    }

    pub fn hitpoint(&self) -> &GameObject {
        unsafe { &*self.fields.hitpoint }
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
    pub fn primary_local_user(&self) -> Option<&User> {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const UserService, *const MethodInfo) -> *const User,
            >(
                BASE_ADDRESS.offset(OFFSETS.method("UserService_GetPrimaryLocalUser"))
            );
            method(self, null()).as_ref()
        }
    }

    pub fn get_users(&self) -> (&User__Array, i32) {
        let mut count = 0;
        unsafe {
            let addr = BASE_ADDRESS.offset(OFFSETS.method("UserService_GetUsers"));
            let method = std::mem::transmute::<
                _,
                extern "system" fn(
                    *const UserService,
                    *mut i32,
                    *const MethodInfo,
                ) -> *const User__Array,
            >(addr);
            let user_array = method(self, &mut count, null());
            (&*user_array, count)
        }
    }
}

#[repr(C)]
pub struct User__Array {
    pub klass: *const u8,
    pub monitor: *const u8,
    pub bounds: *const u8,
    pub max_length: u64,
    pub vector: [*const User; 32],
}

impl User__Array {
    pub fn get(&self, index: usize) -> &User {
        unsafe { &*self.vector[index] }
    }

    pub fn as_vec(&self, count: usize) -> Vec<&User> {
        (0..count).map(|i| self.get(i)).collect::<Vec<_>>()
    }
}

#[repr(C)]
pub struct Int32__Array {
    pub klass: *const u8,
    pub monitor: *const u8,
    pub bounds: *const u8,
    pub max_length: u64,
    pub vector: i32,
}

impl Int32__Array {
    pub fn values(&self) -> &[i32] {
        unsafe { std::slice::from_raw_parts(&self.vector as *const i32, self.max_length as usize) }
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
    _filler: [u8; 680],
    pub rigid_body: *const RigidBody,
}

impl BallMovement {
    pub fn hole_number(&self) -> i32 {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const BallMovement, *const MethodInfo) -> i32,
            >(
                BASE_ADDRESS.offset(OFFSETS.method("BallMovement_get_HoleNumber"))
            );
            method(self, null())
        }
    }

    pub fn last_ground_hit(&self) -> &Transform {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const BallMovement, *const MethodInfo) -> *const Transform,
            >(
                BASE_ADDRESS.offset(OFFSETS.method("BallMovement_get_LastGroundHit"))
            );
            &*method(self, null())
        }
    }

    pub fn rigid_body(&self) -> &RigidBody {
        unsafe { &*self.fields.rigid_body }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn distance(&self, other: &Vector3) -> f32 {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const Vector3, *const Vector3, *const MethodInfo) -> f32,
            >(BASE_ADDRESS.offset(OFFSETS.method("Vector3_Distance")));
            method(self, other, null())
        }
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
pub struct RigidBody {
    pub klass: *const u8,
    pub monitor: *const u8,
    pub fields: RigidBody__Fields,
}

#[repr(C)]
pub struct RigidBody__Fields {}

impl RigidBody {
    pub fn position(&self) -> Vector3 {
        let mut result = Vector3::new();
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const RigidBody, *mut Vector3, *const MethodInfo),
            >(
                BASE_ADDRESS.offset(OFFSETS.method("Rigidbody_get_position_Injected"))
            );
            method(self, &mut result, null());
        }
        result
    }

    pub fn set_position(&self, position: &Vector3) {
        unsafe {
            let method =
                std::mem::transmute::<
                    _,
                    extern "system" fn(*const RigidBody, *const Vector3, *const MethodInfo),
                >(BASE_ADDRESS.offset(OFFSETS.method("Rigidbody_set_position")));
            method(self, position, null());
        }
    }

    pub fn set_velocity(&self, velocity: &Vector3) {
        unsafe {
            let method =
                std::mem::transmute::<
                    _,
                    extern "system" fn(*const RigidBody, *const Vector3, *const MethodInfo),
                >(BASE_ADDRESS.offset(OFFSETS.method("Rigidbody_set_velocity")));
            method(self, velocity, null());
        }
    }
}

#[repr(C)]
pub struct GameObject {}

impl GameObject {
    pub fn find_game_objects_with_tag(tag: &Il2CppString) -> &GameObject__Array {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(
                    *const Il2CppString,
                    *const MethodInfo,
                ) -> *const GameObject__Array,
            >(
                BASE_ADDRESS.offset(OFFSETS.method("GameObject_FindGameObjectsWithTag"))
            );
            &*method(tag, null())
        }
    }

    pub fn transform(&self) -> &Transform {
        unsafe {
            let method =
                std::mem::transmute::<
                    _,
                    extern "system" fn(*const GameObject, *const MethodInfo) -> *const Transform,
                >(BASE_ADDRESS.offset(OFFSETS.method("GameObject_get_transform")));
            &*method(self, null())
        }
    }
}

#[repr(C)]
pub struct GameObject__Array {
    pub klass: *const u8,
    pub monitor: *const u8,
    pub bounds: *const u8,
    pub max_length: u64,
    pub vector: *const GameObject,
}

impl GameObject__Array {
    pub fn values(&self) -> &[&GameObject] {
        unsafe {
            std::slice::from_raw_parts(
                &self.vector as *const *const GameObject as *const &GameObject,
                self.max_length as usize,
            )
        }
    }
}

#[repr(C)]
pub struct Debug {}

impl Debug {
    pub fn draw_line_1(
        start: &Vector3,
        end: &Vector3,
        color: &Color,
        duration: f32,
        depth_test: bool,
    ) {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(
                    *const Vector3,
                    *const Vector3,
                    *const Color,
                    f32,
                    bool,
                    *const MethodInfo,
                ),
            >(BASE_ADDRESS.offset(OFFSETS.method("Debug_2_DrawLine_1")));
            method(start, end, color, duration, depth_test, null());
        }
    }
}

#[repr(C)]
pub struct Transform {}

impl Transform {
    pub fn position(&self) -> Vector3 {
        unsafe {
            let method =
                std::mem::transmute::<
                    _,
                    extern "system" fn(*const Transform, *const MethodInfo) -> Vector3,
                >(BASE_ADDRESS.offset(OFFSETS.method("Transform_get_position")));
            method(self, null())
        }
    }
}

#[repr(C)]
pub struct Camera {}

impl Camera {
    pub fn main() -> &'static Camera {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const MethodInfo) -> *const Camera,
            >(BASE_ADDRESS.offset(OFFSETS.method("Camera_get_main")));
            &*method(null())
        }
    }

    pub fn current() -> &'static Camera {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const MethodInfo) -> *const Camera,
            >(BASE_ADDRESS.offset(OFFSETS.method("Camera_get_current")));
            &*method(null())
        }
    }

    pub fn world_to_screen_point_1(&self, position: &Vector3) -> Vector3 {
        unsafe {
            let method = std::mem::transmute::<
                _,
                extern "system" fn(*const Camera, *const Vector3, *const MethodInfo) -> Vector3,
            >(
                BASE_ADDRESS.offset(OFFSETS.method("Camera_WorldToScreenPoint_1"))
            );
            method(self, position, null())
        }
    }
}
