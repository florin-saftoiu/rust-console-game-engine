extern crate engine;

use engine::{RustConsoleGame, RustConsoleGameEngine};

struct Fps {
    x: f32,
    y: f32,
    velocity: f32
}

impl RustConsoleGame for Fps {
    fn name(&self) -> &str { "FPS" }

    fn setup(&self) {

    }

    fn update(&mut self, engine: &mut RustConsoleGameEngine, elapsed_time: f32) {
        engine.clear();

        self.x += self.velocity * elapsed_time;
        if self.x > 120f32 {
            self.x = 0f32;
        }
        self.y += self.velocity * elapsed_time;
        if self.y > 40f32 {
            self.y = 0f32;
        }
        engine.draw(self.x as i32, self.y as i32, '\u{2588}', 0x000f);
    }
}

fn main() {
    let mut game = Fps {
        x: 0f32,
        y: 0f32,
        velocity: 4f32
    };
    let mut engine = RustConsoleGameEngine::new().unwrap_or_else(|error| {
        panic!("Error creating engine: {:?}", error);
    });
    engine.run(&mut game);
}
