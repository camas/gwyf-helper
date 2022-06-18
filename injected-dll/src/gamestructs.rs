use std::ptr::null;

use log::info;

use crate::{
    api::{il2cpp_domain_get, il2cpp_thread_attach},
    module::Module,
    offsets::OFFSETS,
};

pub static mut BASE_ADDRESS: *const u8 = null();

pub fn init() {
    let module = Module::new("GameAssembly.dll");
    unsafe {
        BASE_ADDRESS = module.base_addr as *const u8;
        info!("base addr: {:#018x}", BASE_ADDRESS as usize);

        // Needs to be called before anything else
        il2cpp_thread_attach(il2cpp_domain_get());
    }
}

macro_rules! il2cpp_fn {
    ($il2cpp_name:expr, ($($arg:ty), *) $(-> $return:ty)?) => {
        std::mem::transmute::<_, extern "system" fn($($arg), *) $(-> $return)? >(
            BASE_ADDRESS.offset(OFFSETS.method($il2cpp_name)),
        )
    }
}

macro_rules! il2cpp_field {
    ($self:expr, $offset:expr, $type:ty) => {
        *((&$self.fields as *const _ as *const u8).add($offset) as *const $type)
    };
}

#[repr(C)]
pub struct MethodInfo {}

pub struct Services {}

impl Services {
    pub fn get_user() -> &'static UserService {
        unsafe {
            &*il2cpp_fn!("Services_get_Users", (*const MethodInfo) -> *const UserService)(null())
        }
    }
}

#[repr(C)]
pub struct User {
    klass: *const u8,
    monitor: *const u8,
    fields: *const u8,
}

impl User {
    pub fn display_name(&self) -> &Il2CppString {
        unsafe {
            let function = il2cpp_fn!("User_get_DisplayName", (*const User, *const MethodInfo) -> *const Il2CppString);
            &*function(self, null())
        }
    }

    pub fn ball(&self) -> &BallMovement {
        unsafe {
            let function = il2cpp_fn!("User_get_Ball", (*const User, *const MethodInfo) -> *const BallMovement);
            &*function(self, null())
        }
    }

    pub fn player_camera(&self) -> &CameraFollow {
        unsafe {
            let function = il2cpp_fn!("User_get_PlayerCamera", (*const User, *const MethodInfo) -> *const CameraFollow);
            &*function(self, null())
        }
    }

    pub fn game_flow_state(&self) -> GameFlowUserState {
        unsafe {
            let value = il2cpp_field!(self, 68, i32);
            assert!(value <= GameFlowUserState::LevelEditor as i32);
            std::mem::transmute(value)
        }
    }

    pub fn set_color(&self, color: &Color) {
        unsafe {
            il2cpp_fn!(
                "User_set_Colour",
                (*const User, *const Color, *const MethodInfo)
            )(self, color, null());
        }
    }

    pub fn update_properties(&self) {
        unsafe {
            il2cpp_fn!("User_UpdateProperties", (*const User, *const MethodInfo))(self, null());
        }
    }

    pub fn m_color(&self) -> &Color {
        unsafe { &il2cpp_field!(self, 88, Color) }
    }

    pub fn hole_scores(&self) -> Option<&Int32__Array> {
        unsafe { il2cpp_field!(self, 120, *const Int32__Array).as_ref() }
    }

    pub fn m_in_hole(&self) -> bool {
        unsafe { il2cpp_field!(self, 116, bool) }
    }

    pub fn m_hit_counter(&self) -> i32 {
        unsafe { il2cpp_field!(self, 112, i32) }
    }
}

#[allow(non_camel_case_types, dead_code)]
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
    pub fields: *const u8,
}

impl CameraFollow {
    pub fn player_pos(&self) -> &Vector3 {
        unsafe { &il2cpp_field!(self, 100, Vector3) }
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
            let function = il2cpp_fn!("UserService_GetPrimaryLocalUser", (*const UserService, *const MethodInfo) -> *const User);
            function(self, null()).as_ref()
        }
    }

    pub fn get_users(&self) -> (&User__Array, i32) {
        unsafe {
            let mut count = 0;
            let function = il2cpp_fn!(
                "UserService_GetUsers",
                (*const UserService, *mut i32, *const MethodInfo) -> *const User__Array);
            let result = function(self, &mut count, null());
            (&*result, count)
        }
    }
}

#[repr(C)]
pub struct User__Array {
    klass: *const u8,
    monitor: *const u8,
    bounds: *const u8,
    max_length: u64,
    vector: [*const User; 32],
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
    klass: *const u8,
    monitor: *const u8,
    bounds: *const u8,
    max_length: u64,
    vector: i32,
}

impl Int32__Array {
    pub fn values(&self) -> &[i32] {
        unsafe { std::slice::from_raw_parts(&self.vector as *const i32, self.max_length as usize) }
    }
}

#[repr(C)]
pub struct BallMovement {
    klass: *const u8,
    monitor: *const u8,
    fields: *const u8,
}

impl BallMovement {
    pub fn hole_number(&self) -> i32 {
        unsafe {
            let function = il2cpp_fn!("BallMovement_get_HoleNumber", (*const BallMovement, *const MethodInfo) -> i32);
            function(self, null())
        }
    }

    pub fn rigid_body(&self) -> &RigidBody {
        unsafe { &*il2cpp_field!(self, 680, *const RigidBody) }
    }

    pub fn network_ball_sync(&self) -> &NetworkBallSync {
        unsafe { &*il2cpp_field!(self, 728, *const NetworkBallSync) }
    }
}

#[repr(C)]
pub struct NetworkBallSync {
    klass: *const u8,
    monitor: *const u8,
    fields: *const u8,
}

impl NetworkBallSync {
    pub fn current_position(&self) -> &Vector3 {
        unsafe { &il2cpp_field!(self, 60, Vector3) }
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
            let function = il2cpp_fn!("Vector3_Distance", (*const Vector3, *const Vector3, *const MethodInfo) -> f32);
            function(self, other, null())
        }
    }
}

#[repr(C)]
pub struct RigidBody {
    klass: *const u8,
    monitor: *const u8,
    fields: *const u8,
}

impl RigidBody {
    pub fn position(&self) -> Vector3 {
        let mut result = Vector3::new();
        unsafe {
            let function = il2cpp_fn!(
                "Rigidbody_get_position",
                (*mut Vector3, *const RigidBody, *const MethodInfo)
            );
            function(&mut result, self, null());
        }
        result
    }

    pub fn set_position(&self, position: &Vector3) {
        unsafe {
            let function = il2cpp_fn!(
                "Rigidbody_set_position",
                (*const RigidBody, *const Vector3, *const MethodInfo)
            );
            function(self, position, null());
        }
    }

    pub fn set_velocity(&self, velocity: &Vector3) {
        unsafe {
            let function = il2cpp_fn!(
                "Rigidbody_set_velocity",
                (*const RigidBody, *const Vector3, *const MethodInfo)
            );
            function(self, velocity, null());
        }
    }
}

#[repr(C)]
pub struct GameObject {}

impl GameObject {
    pub fn find_game_objects_with_tag(tag: &Il2CppString) -> &GameObject__Array {
        unsafe {
            let function = il2cpp_fn!(
                "GameObject_FindGameObjectsWithTag",
                (*const Il2CppString, *const MethodInfo) -> *const GameObject__Array);
            &*function(tag, null())
        }
    }

    pub fn transform(&self) -> &Transform {
        unsafe {
            let function = il2cpp_fn!(
                "GameObject_get_transform",
                (*const GameObject, *const MethodInfo) -> *const Transform);
            &*function(self, null())
        }
    }
}

#[repr(C)]
pub struct GameObject__Array {
    klass: *const u8,
    monitor: *const u8,
    bounds: *const u8,
    max_length: u64,
    vector: *const GameObject,
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
pub struct Transform {}

impl Transform {
    pub fn position(&self) -> Vector3 {
        unsafe {
            let function = il2cpp_fn!(
                "Transform_get_position",
                (*const Transform, *const MethodInfo) -> Vector3);
            function(self, null())
        }
    }
}

#[repr(C)]
pub struct Camera {}

impl Camera {
    pub fn main() -> &'static Camera {
        unsafe {
            let function = il2cpp_fn!("Camera_get_main", (*const MethodInfo) -> *const Camera);
            &*function(null())
        }
    }

    pub fn world_to_screen_point_1(&self, position: &Vector3) -> Vector3 {
        unsafe {
            let function = il2cpp_fn!(
                "Camera_WorldToScreenPoint_1",
                (*const Camera, *const Vector3, *const MethodInfo) -> Vector3);
            function(self, position, null())
        }
    }
}
