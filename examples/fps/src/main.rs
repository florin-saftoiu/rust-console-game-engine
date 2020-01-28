extern crate engine;

use engine::{RustConsole, RustConsoleGame, RustConsoleGameEngine};

struct Fps {
    x: f32,
    y: f32,
    velocity: f32
}

impl RustConsoleGame for Fps {
    fn name(&self) -> &str { "FPS" }

    fn setup(&self) {}

    fn update(&mut self, console: &mut RustConsole, elapsed_time: f32) {
        console.clear();

        self.x += self.velocity * elapsed_time;
        if self.x > console.width() as f32 {
            self.x = 0f32;
        }
        self.y += self.velocity * elapsed_time;
        if self.y > console.height() as f32 {
            self.y = 0f32;
        }
        console.draw(self.x as usize, self.y as usize, '\u{2588}', 0x000f);
    }
}

fn main() {
    let mut game = Fps {
        x: 0f32,
        y: 0f32,
        velocity: 4f32
    };
    let mut engine = RustConsoleGameEngine::new(&mut game, 320, 240, 4, 4).unwrap_or_else(|error| {
        panic!("Error creating console: {:?}", error);
    });
    engine.run();
}
