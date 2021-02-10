#![feature(assoc_char_funcs)]
#![allow(clippy::zst_offset)]

use std::ffi::c_void;

use gamestructs::{GameState__Class, User, UserService};
use imgui::{im_str, Condition, ImString, Ui, Window};
use log::{info, LevelFilter};
use simplelog::{CombinedLogger, TermLogger, TerminalMode};
use winapi::{
    shared::minwindef::HINSTANCE,
    um::{libloaderapi::DisableThreadLibraryCalls, winnt},
};

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;

mod module;
#[macro_use]
mod signature;
mod gamestructs;
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
    unsafe {
        winapi::um::consoleapi::AllocConsole();
    }

    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        simplelog::Config::default(),
        TerminalMode::Mixed,
    )])
    .unwrap();

    renderer::setup(draw_callback);

    info!("gwyf helper running!");
}

static mut STATE: State = State {
    init_done: false,
    player_to_copy: None,
    set_initial_spacing: false,
};

struct State {
    init_done: bool,
    player_to_copy: Option<(usize, *const User)>,
    set_initial_spacing: bool,
}

pub fn draw_callback(ui: &Ui) {
    // Initialization
    unsafe {
        if !STATE.init_done {
            STATE.init_done = true;
            gamestructs::init();
        }
    }

    let _io = ui.io();
    // ui.show_demo_window(&mut true);

    unsafe {
        // Fetch static links
        let context = GameState__Class::get()
            .static_fields
            .as_ref()
            .unwrap()
            .shared_context_info
            .as_ref()
            .unwrap();
        let user_service = UserService::get();
        let user_infos = context.user_infos.as_ref().unwrap();

        // Read data
        let users = (0..user_infos.count())
            .map(|i| user_infos.get(i).user())
            .collect::<Vec<_>>();
        let player = user_service.primary_local_user();

        // Color copying
        if let Some((i, user_ptr)) = &STATE.player_to_copy {
            if *i < users.len() && users[*i] as *const User == *user_ptr {
                let other = users[*i];
                if player.fields.m_colour != other.fields.m_colour {
                    player.set_color(&other.fields.m_colour);
                }
            } else {
                STATE.player_to_copy = None;
            }
        }

        if let Some(token) = ui.begin_main_menu_bar() {
            ui.text("gwyf helper 1.0 ");

            // Color copying selection
            if let Some(token) = ui.begin_menu(im_str!("Players: "), true) {
                ui.text("Copy color of:");
                for (i, user) in users.iter().enumerate() {
                    ui.radio_button(
                        &ImString::new(user.display_name().read()),
                        &mut STATE.player_to_copy,
                        Some((i, *user)),
                    );
                }
                ui.radio_button(im_str!("None"), &mut STATE.player_to_copy, None);

                // Cleanup
                token.end(&ui);
            }

            // Player names
            // ui.text(format!("{} Players: ", users.len()));
            for user in users.iter() {
                let c = &user.fields.m_colour;
                let c = [c.r, c.g, c.b, c.a];
                ui.text_colored(c, format!("{} ", user.display_name().read()));
            }

            // Cleanup
            token.end(&ui);
        }

        // Scoreboard
        Window::new(im_str!("Scoreboard"))
            .size([560., -1.], Condition::Always)
            .build(&ui, || {
                ui.columns(19, im_str!(""), true);

                // Set column spacing once
                if !STATE.set_initial_spacing {
                    STATE.set_initial_spacing = true;
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
                    if !user.fields.hole_scores.is_null() {
                        let current_score = user.fields.m_hit_counter;
                        let current_hole = user.ball().hole_number();
                        let completed = user.fields.m_in_hole;
                        let scores = (&*user.fields.hole_scores).values();

                        for (i, score) in scores.iter().enumerate() {
                            match (i as i32 + 1).cmp(&current_hole) {
                                std::cmp::Ordering::Greater => (),
                                std::cmp::Ordering::Equal => {
                                    let color = if completed {
                                        [0., 1., 0., 1.]
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
                ui.columns(1, im_str!(""), false);
            });
    }

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
    //             Window::new(im_str!("User Info"))
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
    //                     if ui.button(im_str!("Hit X"), [60., 20.]) {
    //                         body.add_force(&Vector3 {
    //                             x: 10.,
    //                             y: 0.,
    //                             z: 0.,
    //                         });
    //                     }
    //                     ui.same_line(62.);
    //                     if ui.button(im_str!("-X"), [60., 20.]) {
    //                         body.add_force(&Vector3 {
    //                             x: -10.,
    //                             y: 0.,
    //                             z: 0.,
    //                         });
    //                     }
    //                     if ui.button(im_str!("Hit Y"), [60., 20.]) {
    //                         body.add_force(&Vector3 {
    //                             x: 0.,
    //                             y: 10.,
    //                             z: 0.,
    //                         });
    //                     }
    //                     ui.same_line(62.);
    //                     if ui.button(im_str!("-Y"), [60., 20.]) {
    //                         body.add_force(&Vector3 {
    //                             x: 0.,
    //                             y: -10.,
    //                             z: 0.,
    //                         });
    //                     }
    //                     if ui.button(im_str!("Hit Z"), [60., 20.]) {
    //                         body.add_force(&Vector3 {
    //                             x: 0.,
    //                             y: 0.,
    //                             z: 10.,
    //                         });
    //                     }
    //                     ui.same_line(62.);
    //                     if ui.button(im_str!("-Z"), [60., 20.]) {
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
