use rust_console_game_engine::{RustConsole, RustConsoleGame, RustConsoleGameEngine};

use std::collections::VecDeque;

struct Racer {
    car_pos: f32,
    distance: f32,
    speed: f32,
    curvature: f32,
    track_curvature: f32,
    player_curvature: f32,
    track_distance: f32,
    current_lap_time: f32,
    track: Vec<(f32, f32)>, // (curvature, distance)
    lap_times: VecDeque<f32>
}

impl RustConsoleGame for Racer {
    fn name(&self) -> &str { "Racer" }

    fn setup(&mut self) {
        self.track.push((0f32, 10f32)); // short section to start/finish line
        self.track.push((0f32, 200f32));
        self.track.push((1f32, 200f32));
        self.track.push((0f32, 400f32));
        self.track.push((-1f32, 100f32));
        self.track.push((0f32, 200f32));
        self.track.push((-1f32, 200f32));
        self.track.push((1f32, 200f32));
        self.track.push((0f32, 200f32));
        self.track.push((0.2f32, 500f32));
        self.track.push((0f32, 200f32));

        self.track_distance = self.track.iter().fold(self.track_distance, |td, seg| td + seg.1);

        self.lap_times = vec![0f32; 5].into_iter().collect();
    }

    fn update(&mut self, console: &mut RustConsole, elapsed_time: f32) {
        if console.key(RustConsole::VK_UP as usize).held {
            self.speed += 2f32 * elapsed_time;
        } else {
            self.speed -= 1f32 * elapsed_time;
        }

        let mut car_direction = 0;

        // car curvature is accumulated left/right input, but inversely proportional to speed
        // i.e. it is harder to turn at high speed
        if console.key(RustConsole::VK_LEFT as usize).held {
            self.player_curvature -= 0.7f32 * elapsed_time * (1f32 - self.speed / 2f32);
            car_direction = -1;
        }

        if console.key(RustConsole::VK_RIGHT as usize).held {
            self.player_curvature += 0.7f32 * elapsed_time * (1f32 - self.speed / 2f32);
            car_direction = 1;
        }

        // if car curvature is too different to track curvature, slow down
        // as car has gone off track
        if (self.player_curvature - self.track_curvature).abs() >= 0.8f32 {
            self.speed -= 5f32 * elapsed_time;
        }

        // clamp speed
        if self.speed < 0f32 {
            self.speed = 0f32;
        }
        if self.speed > 1f32 {
            self.speed = 1f32;
        }

        // move car along track according to car speed
        self.distance += (70f32 * self.speed) * elapsed_time;

        // get point on track
        let mut offset = 0f32;
        let mut track_section = 0;

        // lap timing and counting
        self.current_lap_time += elapsed_time;
        if self.distance >= self.track_distance {
            self.distance -= self.track_distance;
            self.lap_times.push_front(self.current_lap_time);
            self.lap_times.pop_back();
            self.current_lap_time = 0f32;
        }

        // find position on track
        while track_section < self.track.len() && offset <= self.distance {
            offset += self.track[track_section].1;
            track_section += 1;
        }

        // interpolate towards target track curvature
        let target_curvature = self.track[track_section - 1].0;
        let track_curve_diff = (target_curvature - self.curvature) * elapsed_time * self.speed;
        
        // accumulate player curvature
        self.curvature += track_curve_diff;

        // accumulate track curvature
        self.track_curvature += self.curvature * elapsed_time * self.speed;

        // draw sky - light blue and dark blue
        for y in 0..console.height() / 2 {
            for x in 0..console.width() {
                console.draw(x, y, if y < console.height() / 4 { RustConsole::PIXEL_HALF } else { RustConsole::PIXEL_SOLID }, RustConsole::FG_DARK_BLUE);
            }
        }

        // draw scenery - hills are a rectified sine wave, where the phase is adjusted by the
		// accumulated track curvature
        for x in 0..console.width() {
            let hill_height = ((x as f32 * 0.01f32 + self.track_curvature).sin() * 16f32).abs() as isize;
            for y in ((console.height() / 2) as isize - hill_height) as usize..console.height() / 2 {
                console.draw(x, y, RustConsole::PIXEL_SOLID, RustConsole::FG_DARK_YELLOW);
            }
        }

        // draw track - each row is split into grass, clip-board and track
        for y in 0..console.height() / 2 {
            for x in 0..console.width() {
                // perspective is used to modify the width of the track row segments
                let perspective = (y as f32) / ((console.height() as f32) / 2.0f32);
                let mut road_width = 0.1f32 + perspective * 0.8f32; // min 10% max 90%
                let clip_width = road_width * 0.15f32;
                road_width *= 0.5f32; // halve it as track is symmetrical around center of track, but offset...

                // ..depending on where the middle point is, which is defined by the current
				// track curvature
                let middle_point = 0.5f32 + self.curvature * (1f32 - perspective).powi(3);

                // work out segment boundaries
                let left_grass = (middle_point - road_width - clip_width) * console.width() as f32;
                let left_clip = (middle_point - road_width) * console.width() as f32;
                let right_clip = (middle_point + road_width) * console.width() as f32;
                let right_grass = (middle_point + road_width + clip_width) * console.width() as f32;

                let row = console.height() / 2 + y;

                // using periodic oscillatory functions to give lines, where the phase is controlled
				// by the distance around the track
                let grass_color = if (20f32 * (1f32 - perspective).powi(3) + self.distance * 0.1f32).sin() > 0f32 { RustConsole::FG_GREEN } else { RustConsole::FG_DARK_GREEN };
                let clip_color = if (80f32 * (1f32 - perspective).powi(2) + self.distance).sin() > 0f32 { RustConsole::FG_RED } else { RustConsole::FG_WHITE };

                let road_color = if track_section - 1 == 0 { RustConsole::FG_WHITE } else { RustConsole::FG_GREY };

                // draw the row segments
                if (x as f32) < left_grass {
                    console.draw(x, row, RustConsole::PIXEL_SOLID, grass_color);
                } else if (x as f32) >= left_grass && (x as f32) < left_clip {
                    console.draw(x, row, RustConsole::PIXEL_SOLID, clip_color);
                } else if (x as f32) >= left_clip && (x as f32) < right_clip {
                    console.draw(x, row, RustConsole::PIXEL_SOLID, road_color);
                } else if (x as f32) >= right_clip && (x as f32) < right_grass {
                    console.draw(x, row, RustConsole::PIXEL_SOLID, clip_color);
                } else if (x as f32) >= right_grass && x < console.width() {
                    console.draw(x, row, RustConsole::PIXEL_SOLID, grass_color);
                }
            }
        }

        // draw car - car position on road is proportional to difference between
		// current accumulated track curvature, and current accumulated player curvature
		// i.e. if they are similar, the car will be in the middle of the track
        self.car_pos = self.player_curvature - self.track_curvature;
        let car_pos = (console.width() as isize / 2 + ((console.width() as f32 * self.car_pos / 2.0f32) as isize)) as usize - 7; // offset for sprite

        if car_direction == 0 {
            console.draw_string_alpha(car_pos, 80, "   ||####||   ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 81, "      ##      ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 82, "     ####     ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 83, "     ####     ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 84, "|||  ####  |||", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 85, "|||########|||", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 86, "|||  ####  |||", RustConsole::FG_WHITE);
        } else if car_direction == 1 {
            console.draw_string_alpha(car_pos, 80, "      //####//", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 81, "         ##   ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 82, "       ####   ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 83, "      ####    ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 84, "///  ####//// ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 85, "//#######///O ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 86, "/// #### //// ", RustConsole::FG_WHITE);
        } else if car_direction == -1 {
            console.draw_string_alpha(car_pos, 80, "\\\\####\\\\      ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 81, "   ##         ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 82, "   ####       ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 83, "    ####      ", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 84, " \\\\\\\\####  \\\\\\", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 85, " O\\\\\\#######\\\\", RustConsole::FG_WHITE);
            console.draw_string_alpha(car_pos, 86, " \\\\\\\\ #### \\\\\\", RustConsole::FG_WHITE);
        }

        // draw stats
		console.draw_string(0, 0, format!("Distance: {}", self.distance).as_str(), RustConsole::FG_WHITE);
		console.draw_string(0, 1, format!("Target Curvature: {}", self.curvature).as_str(), RustConsole::FG_WHITE);
		console.draw_string(0, 2, format!("Player Curvature: {}", self.player_curvature).as_str(), RustConsole::FG_WHITE);
		console.draw_string(0, 3, format!("Player Speed    : {}", self.speed).as_str(), RustConsole::FG_WHITE);
		console.draw_string(0, 4, format!("Track Curvature : {}", self.track_curvature).as_str(), RustConsole::FG_WHITE);

        fn disp_time(t: f32) -> String {
            let minutes = (t / 60f32) as u32;
            let seconds = (t - (minutes as f32 * 60f32)) as u32;
            let milliseconds = ((t - seconds as f32) * 1000f32) as u32;
            return format!("{}.{}:{}", minutes, seconds, milliseconds);
        }

        console.draw_string(10, 8, disp_time(self.current_lap_time).as_str(), RustConsole::FG_WHITE);

        let mut j = 10;
        for l in self.lap_times.iter() {
            console.draw_string(10, j, disp_time(*l).as_str(), RustConsole::FG_WHITE);
            j += 1;
        }
    }
}

fn main() {
    let mut game = Racer {
        car_pos: 0f32,
        distance: 0f32,
        speed: 0f32,
        curvature: 0f32,
        track_curvature: 0f32,
        player_curvature: 0f32,
        track_distance: 0f32,
        current_lap_time: 0f32,
        track: Vec::new(),
        lap_times: VecDeque::new()
    };
    let mut engine = RustConsoleGameEngine::new(&mut game, 160, 100, 8, 8).unwrap_or_else(|error| {
        panic!("Error creating console: {:?}", error);
    });
    engine.run();
}