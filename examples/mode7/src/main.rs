use rust_console_game_engine::{RustConsole, RustConsoleGame, RustConsoleGameEngine, RustConsoleSprite};

struct Mode7 {
    sprite_ground: RustConsoleSprite,
    sprite_sky: RustConsoleSprite
}

impl RustConsoleGame for Mode7 {
    fn name(&self) -> &str { "Mode7" }

    fn setup(&mut self) {}

    fn update(&mut self, console: &mut RustConsole, _elapsed_time: f32) {
        console.draw_sprite(0, 0, &self.sprite_sky);
        console.draw_sprite(0, 120, &self.sprite_ground);
    }
}

fn main() {
    let mut game = Mode7 {
        sprite_ground: RustConsoleSprite::from_path("assets/mariokart.spr").unwrap_or_else(|error| {
            panic!("Error loading sprite: {:?}", error);
        }),
        sprite_sky: RustConsoleSprite::from_path("assets/sky1.spr").unwrap_or_else(|error| {
            panic!("Error loading sprite: {:?}", error);
        })
    };
    let mut engine = RustConsoleGameEngine::new(&mut game, 320, 240, 4, 4).unwrap_or_else(|error| {
        panic!("Error creating console: {:?}", error);
    });
    engine.run();
}