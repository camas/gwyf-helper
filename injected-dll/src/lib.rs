#![allow(clippy::zst_offset)]

use std::{ffi::c_void, time::Instant};

use api::il2cpp_string_new;
use gamestructs::{Camera, GameFlowUserState, GameObject, Services, User, Vector3};
use imgui::{Condition, Slider, Ui, Window};
use log::{info, LevelFilter};
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode};
use winapi::{
    shared::minwindef::HINSTANCE,
    um::{libloaderapi::DisableThreadLibraryCalls, winnt},
};

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

#[macro_use]
mod signature;
mod api;
mod gamestructs;
mod module;
mod offsets;
mod renderer;

#[no_mangle]
pub extern "stdcall" fn DllMain(hinst_dll: HINSTANCE, fdw_reason: u32, _reserved: c_void) {
    if fdw_reason == winnt::DLL_PROCESS_ATTACH {
        unsafe {
            DisableThreadLibraryCalls(hinst_dll);
        }
        std::thread::spawn(main);
    }
}

pub fn main() {
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

    renderer::setup(draw_callback);

    info!("gwyf helper running!");
}

static mut STATE: State = State {
    init_done: false,
    player_to_copy: None,
    set_initial_spacing: false,
    hitting: None,
    hole_setting: EspSetting::Lines,
    hole_line_opacity: 0.4,
    hole_range: 100.,
    player_setting: EspSetting::Lines,
};

struct State {
    init_done: bool,
    player_to_copy: Option<(usize, *const User)>,
    set_initial_spacing: bool,
    hitting: Option<HittingState>,
    hole_setting: EspSetting,
    hole_line_opacity: f32,
    hole_range: f32,
    player_setting: EspSetting,
}

pub fn draw_callback(ui: &Ui) {
    // Initialization
    let state = unsafe { &mut STATE };
    if !state.init_done {
        state.init_done = true;
        gamestructs::init();
    }

    let io = ui.io();
    let draw_list = ui.get_background_draw_list();
    let screen_width = io.display_size[0];
    let screen_height = io.display_size[1];

    // Fetch static links
    let user_service = Services::get_user();

    // Get users
    let (users, user_count) = user_service.get_users();
    let users = users.as_vec(user_count as usize);
    let player = user_service.primary_local_user();

    // End early if no player object
    if player.is_none() {
        if let Some(token) = ui.begin_main_menu_bar() {
            ui.text("gwyf helper 1.0 - game loading");
            token.end();
        }
        return;
    }
    let player = player.unwrap();
    let flow_state = player.game_flow_state();
    let playing = matches!(
        flow_state,
        GameFlowUserState::HoleStarted | GameFlowUserState::InHoleStarted
    );
    let camera = Camera::main();

    // Color copying
    if let Some((i, user_ptr)) = &state.player_to_copy {
        if *i < users.len() && std::ptr::eq(users[*i], *user_ptr) {
            let other = users[*i];
            if player.m_color() != other.m_color() {
                player.set_color(other.m_color());
                player.update_properties();
            }
        } else {
            state.player_to_copy = None;
        }
    }

    // Hitting
    if let Some(hit_state) = &mut state.hitting {
        // Wait a short time so collisions register
        if hit_state.last_time.elapsed().as_millis() >= 100 {
            let next = hit_state.to_hit.pop().unwrap();
            if next >= users.len() {
                // Indexes have changed. Reset
                let body = player.ball().rigid_body();
                body.set_position(&hit_state.original_position);
                body.set_velocity(&Vector3::new());
                state.hitting = None;
            } else {
                // Hit player
                hit_other_player(player, users[next]);
                if hit_state.to_hit.is_empty() {
                    // Reset
                    let body = player.ball().rigid_body();
                    body.set_position(&hit_state.original_position);
                    body.set_velocity(&Vector3::new());
                    state.hitting = None;
                } else {
                    // Update last hit time
                    (*hit_state).last_time = Instant::now();
                }
            }
        }
    }

    // Player esp
    if playing && state.player_setting != EspSetting::None {
        // Get player position for calculating distance to holes
        let player_pos = &player.player_camera().player_pos();

        // Draw lines from bottom of screen
        let from = [screen_width / 2., screen_height];
        for &other_player in users
            .iter()
            .filter(|u| **u as *const _ != player as *const _)
        {
            // let obj_pos = other_player.ball().last_ground_hit().position();
            let obj_pos = other_player.ball().network_ball_sync().current_position();
            let to = camera.world_to_screen_point_1(obj_pos);
            let distance = player_pos.distance(obj_pos);
            // Ignore if behind camera
            if to.z < 0. {
                continue;
            }
            if state.player_setting == EspSetting::Lines {
                // Unity y axis is inverted
                draw_list
                    .add_line(
                        from,
                        [to.x, screen_height - to.y],
                        [0., 0., 1., state.hole_line_opacity],
                    )
                    .build();
            }
            draw_list
                .add_circle([to.x, screen_height - to.y], 2., [0., 0., 1., 1.])
                .build();

            let text = format!("{} {:.1}m", other_player.display_name().read(), distance);
            let text_width = ui.calc_text_size(&text)[0];
            draw_list.add_text(
                [to.x - (text_width / 2.), screen_height - to.y - 20.],
                [1., 1., 1.],
                text,
            );
        }
    }

    // Hole esp
    if playing && state.hole_setting != EspSetting::None {
        // Find all holes
        let holes = GameObject::find_game_objects_with_tag(il2cpp_string_new(b"Hole\0"));

        // Get player position for calculating distance to holes
        let player_pos = &player.player_camera().player_pos();

        // Draw lines from bottom of screen
        let from = [screen_width / 2., screen_height];
        for obj in holes.values() {
            let obj_pos = obj.transform().position();
            let to = camera.world_to_screen_point_1(&obj_pos);
            let distance = player_pos.distance(&obj_pos);
            // Ignore if behind camera
            if to.z < 0. || distance > state.hole_range {
                continue;
            }
            if state.hole_setting == EspSetting::Lines {
                // Unity y axis is inverted
                draw_list
                    .add_line(
                        from,
                        [to.x, screen_height - to.y],
                        [1., 0., 0., state.hole_line_opacity],
                    )
                    .build();
            }
            draw_list
                .add_circle([to.x, screen_height - to.y], 2., [1., 0., 0., 1.])
                .build();

            let text = format!("{:.1}m", distance);
            let text_width = ui.calc_text_size(&text)[0];
            draw_list.add_text(
                [to.x - (text_width / 2.), screen_height - to.y - 20.],
                [1., 1., 1.],
                text,
            );
        }
    }

    if let Some(_token) = ui.begin_main_menu_bar() {
        ui.text("gwyf helper 1.0 ");

        // ESP Settings
        if let Some(_token) = ui.begin_menu("esp") {
            // Hole options
            ui.text("Hole esp:");
            ui.radio_button("None", &mut state.hole_setting, EspSetting::None);
            ui.radio_button("Points", &mut state.hole_setting, EspSetting::Points);
            ui.radio_button("Lines", &mut state.hole_setting, EspSetting::Lines);
            Slider::new("Line opacity", 0., 1.).build(ui, &mut state.hole_line_opacity);
            Slider::new("Within range", 0., 1000.).build(ui, &mut state.hole_range);
        }

        // Hit button
        if playing && users.len() > 1 && ui.button_with_size("bam", [40., 20.]) {
            // Hit everyone who isn't in the hole yet
            let to_hit = users
                .iter()
                .enumerate()
                .filter_map(|(i, u)| {
                    if !std::ptr::eq(*u, player) && !u.m_in_hole() {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            state.hitting = Some(HittingState {
                last_time: Instant::now(),
                to_hit,
                original_position: player.ball().rigid_body().position(),
            });
        }

        // Color copying selection
        if let Some(_token) = ui.begin_menu("Players:") {
            ui.text("Copy color of:");
            for (i, user) in users.iter().enumerate() {
                ui.radio_button(
                    &user.display_name().read(),
                    &mut state.player_to_copy,
                    Some((i, *user)),
                );
            }
            ui.radio_button("None", &mut state.player_to_copy, None);
        }

        // Player names
        // ui.text(format!("{} Players: ", users.len()));
        for user in users.iter() {
            let c = user.m_color();
            let c = [c.r, c.g, c.b, c.a];
            ui.text_colored(c, format!("{} ", user.display_name().read()));
        }

        // FlowState
        let text = format!("{:?}", flow_state);
        let size = ui.calc_text_size(&text);
        let new_pos = [
            ui.cursor_pos()[0] + ui.column_width(0) - size[0],
            ui.cursor_pos()[1],
        ];
        ui.set_cursor_pos(new_pos);
        ui.text(text);
    }

    // Scoreboard
    Window::new("Scoreboard")
        .size([560., -1.], Condition::Always)
        .build(ui, || {
            ui.columns(19, "", true);

            // Set column spacing once
            if !state.set_initial_spacing {
                state.set_initial_spacing = true;
                ui.set_column_width(0, 80.);
                for i in 1..=18 {
                    ui.set_column_width(i, 26.);
                }
            }

            // Header
            ui.next_column();
            for i in 1..=18 {
                ui.text(i.to_string());
                ui.next_column();
            }

            // Scores
            ui.separator();
            for user in users.iter() {
                // Name
                ui.text(user.display_name().read());
                ui.next_column();

                // Hit counts
                if let Some(hole_scores) = user.hole_scores() {
                    let current_score = user.m_hit_counter();
                    let current_hole = user.ball().hole_number();
                    let completed = user.m_in_hole();
                    let scores = hole_scores.values();

                    for (i, score) in scores.iter().enumerate() {
                        match (i as i32 + 1).cmp(&current_hole) {
                            std::cmp::Ordering::Greater => (),
                            std::cmp::Ordering::Equal => {
                                let color = if completed {
                                    [1., 1., 1., 1.]
                                } else {
                                    [0., 1., 1., 1.]
                                };
                                ui.text_colored(color, current_score.to_string());
                            }
                            std::cmp::Ordering::Less => {
                                ui.text(score.to_string());
                            }
                        }
                        ui.next_column();
                    }
                } else {
                    ui.new_line();
                }

                ui.separator();
            }
            ui.columns(1, "", false);
        });

    // unsafe {
    //     let user_service = UserService::get_instance();
    //     if user_service.is_null() {
    //         return;
    //     }
    //     let user_service = &*user_service;
    //     let control_player = user_service.get_in_control_player();
    //     let control_ball = if !control_player.is_null() {
    //         (*control_player).ball
    //     } else {
    //         null_mut()
    //     };
    //     ui.show_demo_window(&mut true);

    //     if !control_ball.is_null() {
    //         let control_player = &*control_player;
    //         let ball = &*control_ball;
    //         let body = ball.rigid_body;
    //         if !body.is_null() && (*body).exists() {
    //             let body = &*body;
    //             let pos = body.get_position();
    //             let power = control_player.hit_force * 4. / 10500.;
    //             Window::new("User Info")
    //                 .flags(
    //                     WindowFlags::NO_DECORATION
    //                         | WindowFlags::ALWAYS_AUTO_RESIZE
    //                         | WindowFlags::NO_FOCUS_ON_APPEARING
    //                         | WindowFlags::NO_SAVED_SETTINGS
    //                         | WindowFlags::NO_NAV,
    //                 )
    //                 .position(
    //                     [io.display_size[0] / 2., io.display_size[1] - 10.],
    //                     Condition::Always,
    //                 )
    //                 .position_pivot([0.5, 1.])
    //                 .bg_alpha(0.35)
    //                 .build(&ui, || {
    //                     ui.text(format!("Power: {}", power));
    //                     ui.text(format!(
    //                         "Last: {}",
    //                         control_player.last_hit_force * 4. / 10500.
    //                     ));
    //                     ui.text(format!(
    //                         "X: {:0.2} Y: {:0.2} Z: {:0.2}",
    //                         pos.x, pos.y, pos.z
    //                     ));
    //                     if ui.button("Hit X", [60., 20.]) {
    //                         body.add_force(&Vector3 {
    //                             x: 10.,
    //                             y: 0.,
    //                             z: 0.,
    //                         });
    //                     }
    //                     ui.same_line(62.);
    //                     if ui.button("-X", [60., 20.]) {
    //                         body.add_force(&Vector3 {
    //                             x: -10.,
    //                             y: 0.,
    //                             z: 0.,
    //                         });
    //                     }
    //                     if ui.button("Hit Y", [60., 20.]) {
    //                         body.add_force(&Vector3 {
    //                             x: 0.,
    //                             y: 10.,
    //                             z: 0.,
    //                         });
    //                     }
    //                     ui.same_line(62.);
    //                     if ui.button("-Y", [60., 20.]) {
    //                         body.add_force(&Vector3 {
    //                             x: 0.,
    //                             y: -10.,
    //                             z: 0.,
    //                         });
    //                     }
    //                     if ui.button("Hit Z", [60., 20.]) {
    //                         body.add_force(&Vector3 {
    //                             x: 0.,
    //                             y: 0.,
    //                             z: 10.,
    //                         });
    //                     }
    //                     ui.same_line(62.);
    //                     if ui.button("-Z", [60., 20.]) {
    //                         body.add_force(&Vector3 {
    //                             x: 0.,
    //                             y: 0.,
    //                             z: -10.,
    //                         });
    //                     }
    //                 });
    //         }
    //     }
}

struct HittingState {
    original_position: Vector3,
    last_time: Instant,
    to_hit: Vec<usize>,
}

fn hit_other_player(player: &User, other: &User) {
    let body = player.ball().rigid_body();
    let other_body = other.ball().rigid_body();

    let mut new_pos = other_body.position();
    new_pos.y += 1.;
    new_pos.z += 0.01;
    body.set_position(&new_pos);
    body.set_velocity(&Vector3 {
        x: 0.,
        y: -200.,
        z: 0.,
    });
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum EspSetting {
    None,
    Points,
    Lines,
}
