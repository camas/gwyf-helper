use std::time::Instant;

use crate::api::il2cpp_string_new;
use crate::gamestructs::{self, Camera, GameFlowUserState, GameObject, Services, User, Vector3};
use hudhook::hooks::ImguiRenderLoop;
use imgui::{Condition, DrawListMut, Io, Ui};

pub struct State {
    init_done: bool,
    player_to_copy_addr: Option<usize>,
    set_initial_spacing: bool,
    hitting: Option<HittingState>,
    hole_setting: EspSetting,
    hole_line_opacity: f32,
    hole_range: f32,
    player_setting: EspSetting,
}

struct HittingState {
    original_position: Vector3,
    last_time: Instant,
    to_hit: Vec<usize>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum EspSetting {
    None,
    Points,
    Lines,
}

struct Drawer<'a> {
    state: &'a mut State,
    ui: &'a Ui,
    _io: &'a Io,
    draw_list: DrawListMut<'a>,
    screen_width: f32,
    screen_height: f32,
}

impl Default for State {
    fn default() -> Self {
        State {
            init_done: false,
            player_to_copy_addr: None,
            set_initial_spacing: false,
            hitting: None,
            hole_setting: EspSetting::Lines,
            hole_line_opacity: 0.4,
            hole_range: 100.,
            player_setting: EspSetting::Lines,
        }
    }
}

impl ImguiRenderLoop for State {
    fn render(&mut self, ui: &mut Ui) {
        Drawer::draw(self, ui);
    }
}

impl<'a> Drawer<'a> {
    fn draw(state: &mut State, ui: &Ui) {
        let io = ui.io();
        let mut drawer = Drawer {
            state,
            ui,
            _io: io,
            draw_list: ui.get_background_draw_list(),
            screen_width: io.display_size[0],
            screen_height: io.display_size[1],
        };
        drawer.run();
    }

    fn run(&mut self) {
        // Initialization
        if !self.state.init_done {
            self.state.init_done = true;
            gamestructs::init();
        }

        // Fetch static links
        let user_service = Services::get_users();

        // Get users
        let (users, user_count) = user_service.get_users();
        let users = users.as_vec(user_count as usize);
        let Some(player) = user_service.get_in_control_player() else {
            if let Some(token) = self.ui.begin_main_menu_bar() {
                self.ui.text("gwyf helper 1.0 - game loading");
                token.end();
            }
            return;
        };
        let flow_state = player.game_flow_state();
        let playing = matches!(
            flow_state,
            GameFlowUserState::HoleStarted | GameFlowUserState::InHoleStarted
        );

        // Color copying
        if let Some(player_to_copy_addr) = &self.state.player_to_copy_addr {
            let other: Option<&&User> = users
                .iter()
                .find(|u| (**u as *const User).addr() == *player_to_copy_addr);
            match other {
                Some(other) => {
                    if player.m_color() != other.m_color() {
                        player.set_color(other.m_color());
                        player.update_properties();
                    }
                }
                None => {
                    self.state.player_to_copy_addr = None;
                }
            }
        }

        // Hitting
        let player_ball = player.ball();
        if player_ball.is_some() && self.state.hitting.is_some() {
            let hit_state = self.state.hitting.as_mut().unwrap();
            // Wait a short time so collisions register
            if hit_state.last_time.elapsed().as_millis() >= 100 {
                let next = hit_state.to_hit.pop().unwrap();
                if next >= users.len() {
                    // Indexes have changed. Reset
                    let body = player_ball.unwrap().rigid_body();
                    body.set_position(&hit_state.original_position);
                    body.set_velocity(&Vector3::new());
                    self.state.hitting = None;
                } else {
                    // Hit player
                    hit_other_player(player, users[next]);
                    if hit_state.to_hit.is_empty() {
                        // Reset
                        let body = player_ball.unwrap().rigid_body();
                        body.set_position(&hit_state.original_position);
                        body.set_velocity(&Vector3::new());
                        self.state.hitting = None;
                    } else {
                        // Update last hit time
                        hit_state.last_time = Instant::now();
                    }
                }
            }
        }

        if let Some(_token) = self.ui.begin_main_menu_bar() {
            self.ui.text("gwyf helper 1.0 ");

            // ESP Settings
            if let Some(_token) = self.ui.begin_menu("esp") {
                // Hole options
                self.ui.text("Hole esp:");
                self.ui
                    .radio_button("None", &mut self.state.hole_setting, EspSetting::None);
                self.ui
                    .radio_button("Points", &mut self.state.hole_setting, EspSetting::Points);
                self.ui
                    .radio_button("Lines", &mut self.state.hole_setting, EspSetting::Lines);
                self.ui
                    .slider("Line opacity", 0., 1., &mut self.state.hole_line_opacity);
                self.ui
                    .slider("Within range", 0., 1000., &mut self.state.hole_range);
            }

            // Hit button
            if playing
                && users.len() > 1
                && self.ui.button_with_size("bam", [40., 20.])
                && player_ball.is_some()
            {
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
                self.state.hitting = Some(HittingState {
                    last_time: Instant::now(),
                    to_hit,
                    original_position: player_ball.unwrap().rigid_body().position(),
                });
            }

            // Color copying selection
            if let Some(_token) = self.ui.begin_menu("Players:") {
                self.ui.text("Copy color of:");
                for user in users.iter() {
                    self.ui.radio_button(
                        &user.display_name().read(),
                        &mut self.state.player_to_copy_addr,
                        Some((*user as *const User).addr()),
                    );
                }
                self.ui
                    .radio_button("None", &mut self.state.player_to_copy_addr, None);
            }

            // Player names
            for user in users.iter() {
                let color = user.m_color();
                let color = [color.r, color.g, color.b, color.a];
                self.ui
                    .text_colored(color, format!("{} ", user.display_name().read()));
            }

            // FlowState
            let text = format!("{flow_state:?}");
            let size = self.ui.calc_text_size(&text);
            let new_pos = [
                self.ui.cursor_pos()[0] + self.ui.column_width(0) - size[0],
                self.ui.cursor_pos()[1],
            ];
            self.ui.set_cursor_pos(new_pos);
            self.ui.text(text);
        }

        let Some(camera) = Camera::main() else {
            return;
        };

        if playing {
            self.draw_player_esp(camera, player, &users);
            self.draw_hole_esp(camera, player);
        }

        // Scoreboard
        self.ui
            .window("Scoreboard")
            .size([560., -1.], Condition::Always)
            .build(|| {
                self.ui.columns(19, "", true);

                // Set column spacing once
                if !self.state.set_initial_spacing {
                    self.state.set_initial_spacing = true;
                    self.ui.set_column_width(0, 80.);
                    for i in 1..=18 {
                        self.ui.set_column_width(i, 26.);
                    }
                }

                // Header
                self.ui.next_column();
                for i in 1..=18 {
                    self.ui.text(i.to_string());
                    self.ui.next_column();
                }

                // Scores
                self.ui.separator();
                for user in users.iter() {
                    // Name
                    self.ui.text(user.display_name().read());
                    self.ui.next_column();

                    // Hit counts
                    let user_ball = user.ball();
                    let user_hole_scores = user.hole_scores();
                    if user_hole_scores.is_some() && user_ball.is_some() {
                        let current_score = user.m_hit_counter();
                        let current_hole = user_ball.unwrap().hole_number();
                        let completed = user.m_in_hole();
                        let mut scores = user_hole_scores.unwrap().values().to_vec();
                        while scores.len() < 18 {
                            scores.push(0);
                        }

                        for (i, score) in scores.iter().enumerate() {
                            match (i as i32 + 1).cmp(&current_hole) {
                                std::cmp::Ordering::Greater => (),
                                std::cmp::Ordering::Equal => {
                                    let color = if completed {
                                        [1., 1., 1., 1.]
                                    } else {
                                        [0., 1., 1., 1.]
                                    };
                                    self.ui.text_colored(color, current_score.to_string());
                                }
                                std::cmp::Ordering::Less => {
                                    self.ui.text(score.to_string());
                                }
                            }
                            self.ui.next_column();
                        }
                    } else {
                        for _ in 0..18 {
                            self.ui.next_column();
                        }
                    }

                    self.ui.separator();
                }
                self.ui.columns(1, "", false);
            });
    }

    fn draw_player_esp(&mut self, camera: &Camera, player: &User, users: &[&User]) {
        if self.state.player_setting == EspSetting::None {
            return;
        }

        // Get player position for calculating distance to holes
        let player_pos = player.player_camera().player_pos();

        // Draw lines from bottom of screen
        let from = [self.screen_width / 2., self.screen_height];
        for &other_player in users
            .iter()
            .filter(|u| **u as *const _ != player as *const _)
        {
            let Some(other_player_ball) = other_player.ball() else {
                continue;
            };
            let obj_pos = other_player_ball.network_ball_sync().current_position();
            let to = camera.world_to_screen_point_1(obj_pos);
            let distance = player_pos.distance(obj_pos);
            // Ignore if behind camera
            if to.z < 0. {
                continue;
            }
            if self.state.player_setting == EspSetting::Lines {
                // Unity y axis is inverted
                self.draw_list
                    .add_line(
                        from,
                        [to.x, self.screen_height - to.y],
                        [0., 0., 1., self.state.hole_line_opacity],
                    )
                    .build();
            }
            self.draw_list
                .add_circle([to.x, self.screen_height - to.y], 2., [0., 0., 1., 1.])
                .build();

            let text = format!("{} {:.1}m", other_player.display_name().read(), distance);
            let text_width = self.ui.calc_text_size(&text)[0];
            self.draw_list.add_text(
                [to.x - (text_width / 2.), self.screen_height - to.y - 20.],
                [1., 1., 1.],
                text,
            );
        }
    }

    fn draw_hole_esp(&mut self, camera: &Camera, player: &User) {
        if self.state.hole_setting == EspSetting::None {
            return;
        }
        // Find all holes
        let holes = GameObject::find_game_objects_with_tag(il2cpp_string_new(b"Hole\0"));

        // Get player position for calculating distance to holes
        let player_pos = player.player_camera().player_pos();

        // Draw lines from bottom of screen
        let from = [self.screen_width / 2., self.screen_height];
        for obj in holes.values() {
            let obj_pos = obj.transform().position();
            let to = camera.world_to_screen_point_1(&obj_pos);
            let distance = player_pos.distance(&obj_pos);
            // Ignore if behind camera
            if to.z < 0. || distance > self.state.hole_range {
                continue;
            }
            if self.state.hole_setting == EspSetting::Lines {
                // Unity y axis is inverted
                self.draw_list
                    .add_line(
                        from,
                        [to.x, self.screen_height - to.y],
                        [1., 0., 0., self.state.hole_line_opacity],
                    )
                    .build();
            }
            self.draw_list
                .add_circle([to.x, self.screen_height - to.y], 2., [1., 0., 0., 1.])
                .build();

            let text = format!("{distance:.1}m");
            let text_width = self.ui.calc_text_size(&text)[0];
            self.draw_list.add_text(
                [to.x - (text_width / 2.), self.screen_height - to.y - 20.],
                [1., 1., 1.],
                text,
            );
        }
    }
}

fn hit_other_player(player: &User, other: &User) {
    let Some(player_ball) = player.ball() else {
        return;
    };
    let Some(other_ball) = other.ball() else {
        return;
    };
    let body = player_ball.rigid_body();
    let other_body = other_ball.rigid_body();

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
