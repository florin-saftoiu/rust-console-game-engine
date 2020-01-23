extern crate engine;

use engine::{RustConsoleGame, RustConsoleGameEngine};

struct Fps {
    x: f32,
    y: f32,
    velocity: f32
}

impl RustConsoleGame for Fps {
    fn name(&self) -> &str { "FPS" }

    fn setup(&self) {}

    fn update(&mut self, engine: &mut RustConsoleGameEngine, elapsed_time: f32) {
        engine.clear();

        self.x += self.velocity * elapsed_time;
        if self.x > engine.width() as f32 {
            self.x = 0f32;
        }
        self.y += self.velocity * elapsed_time;
        if self.y > engine.height() as f32 {
            self.y = 0f32;
        }
        engine.draw(self.x as usize, self.y as usize, '\u{2588}', 0x000f);
    }
}

fn main() {
    let mut game = Fps {
        x: 0f32,
        y: 0f32,
        velocity: 4f32
    };
    let mut engine = RustConsoleGameEngine::new(120, 40).unwrap_or_else(|error| {
        panic!("Error creating engine: {:?}", error);
    });
    engine.run(&mut game);
}
