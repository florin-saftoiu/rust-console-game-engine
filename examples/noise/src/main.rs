extern crate engine;

use engine::{RustConsole, RustConsoleGame, RustConsoleGameEngine};
use rand::prelude::*;

struct Noise {}

impl RustConsoleGame for Noise {
    fn name(&self) -> &str { "Noise" }

    fn setup(&mut self) {}

    fn update(&mut self, console: &mut RustConsole, _elapsed_time: f32) {
        for x in 0..console.width() {
            for y in 0..console.height() {
                let random: u16 = random();
                console.draw(x, y, '#', random % 16);
            }
        }
    }
}

fn main() {
    let mut game = Noise {};
    let mut engine = RustConsoleGameEngine::new(&mut game, 160, 100, 8, 8).unwrap_or_else(|error| {
        panic!("Error creating console: {:?}", error);
    });
    engine.run();
}