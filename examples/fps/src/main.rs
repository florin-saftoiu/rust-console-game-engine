extern crate engine;

use engine::{RustConsole, RustConsoleGame, RustConsoleGameEngine};
use std::f32;

struct Fps {
    player_x: f32,
    player_y: f32,
    player_a: f32,
    speed: f32,
    fov: f32,
    depth: f32,
    map: String,
    map_width: i32,
    map_height: i32
}

impl RustConsoleGame for Fps {
    fn name(&self) -> &str { "FPS" }

    fn setup(&mut self) {
        self.player_x = 14.7f32;
        self.player_y = 5.09f32;
        self.map_width = 16;
        self.map_height = 16;
        self.map = String::from("################\
                                 #..............#\
                                 #.......########\
                                 #..............#\
                                 #......##......#\
	                             #......##......#\
	                             #..............#\
	                             ###............#\
	                             ##.............#\
	                             #......####..###\
	                             #......#.......#\
	                             #......#.......#\
	                             #..............#\
	                             #......#########\
	                             #..............#\
                                 ################");
    }

    fn update(&mut self, console: &mut RustConsole, elapsed_time: f32) {
        if console.key('H' as usize).released {
            if console.width() == 120 {
                console.resize(180, 60, 5, 11);
            } else if console.width() == 180 {
                console.resize(320, 240, 4, 4);
            } else if console.width() == 320 {
                console.resize(60, 20, 16, 32);
            } else {
                console.resize(120, 40, 8, 16);
            }
        }

        if console.key('A' as usize).held {
            self.player_a -= self.speed * 0.75f32 * elapsed_time;

            if self.player_a < -f32::consts::PI {
                self.player_a = f32::consts::PI + (self.player_a + f32::consts::PI);
            }
        }

        if console.key('D' as usize).held {
            self.player_a += self.speed * 0.75f32 * elapsed_time;

            if self.player_a > f32::consts::PI {
                self.player_a = -f32::consts::PI + (self.player_a - f32::consts::PI);
            }
        }

        if console.key('W' as usize).held {
            self.player_x += self.player_a.sin() * self.speed * elapsed_time;
            self.player_y += self.player_a.cos() * self.speed * elapsed_time;
            
            if self.map.as_bytes()[(self.player_x as i32 * self.map_width + self.player_y as i32) as usize] == b'#' {
                self.player_x -= self.player_a.sin() * self.speed * elapsed_time;
                self.player_y -= self.player_a.cos() * self.speed * elapsed_time;
            }
        }

        if console.key('S' as usize).held {
            self.player_x -= self.player_a.sin() * self.speed * elapsed_time;
            self.player_y -= self.player_a.cos() * self.speed * elapsed_time;
            
            if self.map.as_bytes()[(self.player_x as i32 * self.map_width + self.player_y as i32) as usize] == b'#' {
                self.player_x += self.player_a.sin() * self.speed * elapsed_time;
                self.player_y += self.player_a.cos() * self.speed * elapsed_time;
            }
        }

        if console.key('Q' as usize).held {
            self.player_x -= self.player_a.cos() * self.speed * elapsed_time;
            self.player_y += self.player_a.sin() * self.speed * elapsed_time;

            if self.map.as_bytes()[(self.player_x as i32 * self.map_width + self.player_y as i32) as usize] == b'#' {
                self.player_x += self.player_a.cos() * self.speed * elapsed_time;
                self.player_y -= self.player_a.sin() * self.speed * elapsed_time;
            }
        }

        if console.key('E' as usize).held {
            self.player_x += self.player_a.cos() * self.speed * elapsed_time;
            self.player_y -= self.player_a.sin() * self.speed * elapsed_time;

            if self.map.as_bytes()[(self.player_x as i32 * self.map_width + self.player_y as i32) as usize] == b'#' {
                self.player_x -= self.player_a.cos() * self.speed * elapsed_time;
                self.player_y += self.player_a.sin() * self.speed * elapsed_time;
            }
        }

        for x in 0..console.width() {
            let ray_angle = (self.player_a - self.fov / 2f32) + (x as f32 / console.width() as f32) * self.fov;

            let step_size = 0.1f32;
            let mut distance_to_wall = 0f32;

            let mut hit_wall = false;
            let mut boundary = false;

            let eye_x = ray_angle.sin();
            let eye_y = ray_angle.cos();

            while !hit_wall && distance_to_wall < self.depth {
                distance_to_wall += step_size;

                let test_x = (self.player_x + eye_x * distance_to_wall) as i32;
                let test_y = (self.player_y + eye_y * distance_to_wall) as i32;

                if test_x < 0 || test_x >= self.map_width || test_y < 0 || test_y >= self.map_height {
                    hit_wall = true;
                    distance_to_wall = self.depth;
                } else if self.map.as_bytes()[(test_x * self.map_width + test_y) as usize] == b'#' {
                    hit_wall = true;

                    let mut p = Vec::<(f32, f32)>::new();

                    for tx in 0..2 {
                        for ty in 0..2 {
                            let vy = test_y as f32 + ty as f32 - self.player_y;
                            let vx = test_x as f32 + tx as f32 - self.player_x;
                            let d = (vx * vx + vy * vy).sqrt();
                            let dot = (eye_x * vx / d) + (eye_y * vy / d);
                            p.push((d, dot));
                        }
                    }

                    p.sort_by(|left, right| left.0.partial_cmp(&right.0).unwrap());

                    let bound = 0.01f32;
                    if p[0].1.acos() < bound {
                        boundary = true;
                    }
                    if p[1].1.acos() < bound {
                        boundary = true;
                    }
                    if p[2].1.acos() < bound {
                        boundary = true;
                    }
                }
            }

            let ceiling = ((console.height() as f32 / 2f32) - console.height() as f32 / distance_to_wall) as i32;
            let floor = console.height() as i32 - ceiling;

            let shade = if boundary {
                ' '
            } else if distance_to_wall <= self.depth / 4f32 {
                '\u{2588}'
            } else if distance_to_wall < self.depth / 3f32 {
                '\u{2593}'
            } else if distance_to_wall < self.depth / 2f32 {
                '\u{2592}'
            } else if distance_to_wall < self.depth {
                '\u{2591}'
            } else {
                ' '
            };

            for y in 0..console.height() {
                if (y as i32) <= ceiling {
                    console.draw(x, y, ' ', RustConsole::FG_WHITE);
                } else if y as i32 > ceiling && y as i32 <= floor {
                    console.draw(x, y, shade, RustConsole::FG_WHITE);
                } else {
                    let b = 1f32 - ((y as f32 - console.height() as f32 / 2f32) / (console.height() as f32 / 2f32));
                    let floor_shade = if b < 0.25f32 {
                        '#'
                    } else if b < 0.5f32 {
                        'x'
                    } else if b < 0.75f32 {
                        '.'
                    } else if b < 0.9f32 {
                        '-'
                    } else {
                        ' '
                    };
                    console.draw(x, y, floor_shade, RustConsole::FG_WHITE);
                }
            }
        }

        console.draw_string(0, 0, format!("x={:3.2},y={:3.2},a={:3.2}\u{00b0}", self.player_x, self.player_y, self.player_a * 180f32 / f32::consts::PI), 0x000f);

        for mx in 0..self.map_width as usize {
            for my in 0..self.map_height as usize {
                console.draw(mx, my + 1, self.map.as_bytes()[my * self.map_width as usize + mx] as char, RustConsole::FG_WHITE);
            }
        }

        let p = if self.player_a < -(3f32 * f32::consts::PI / 4f32) {
            '<'
        } else if -(3f32 * f32::consts::PI / 4f32) <= self.player_a && self.player_a < -(f32::consts::PI / 4f32) {
            '^'
        } else if -(f32::consts::PI / 4f32) <= self.player_a && self.player_a < f32::consts::PI / 4f32 {
            '>'
        } else if f32::consts::PI / 4f32 <= self.player_a && self.player_a < 3f32 * f32::consts::PI / 4f32 {
            'v'
        } else if 3f32 * f32::consts::PI / 4f32 <= self.player_a {
            '<'
        } else {
            '*'
        };
        console.draw(self.player_y as usize, self.player_x as usize + 1, p, RustConsole::FG_WHITE);
    }
}

fn main() {
    let mut game = Fps {
        player_x: 0f32,
        player_y: 0f32,
        player_a: 0f32,
        speed: 5f32,
        fov: f32::consts::PI / 4f32,
        depth: 16f32,
        map: String::default(),
        map_width: 0,
        map_height: 0
    };
    let mut engine = RustConsoleGameEngine::new(&mut game, 120, 40, 8, 16).unwrap_or_else(|error| {
        panic!("Error creating console: {:?}", error);
    });
    engine.run();
}
