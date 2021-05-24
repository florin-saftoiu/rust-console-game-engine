use rust_console_game_engine::{RustConsole, RustConsoleGame, RustConsoleGameEngine, RustConsoleSprite};

struct Mode7 {
    sprite: RustConsoleSprite
}

impl RustConsoleGame for Mode7 {
    fn name(&self) -> &str { "Mode7" }

    fn setup(&mut self) {
        self.sprite.set_glyph(0, 0, RustConsole::PIXEL_HALF);
        self.sprite.set_color(0, 0, RustConsole::FG_RED | RustConsole::BG_GREY);
        self.sprite.set_glyph(6, 4, 'S');
        self.sprite.set_color(6, 4, RustConsole::FG_RED | RustConsole::BG_BLUE);
    }

    fn update(&mut self, console: &mut RustConsole, _elapsed_time: f32) {
        for x in 0..console.width() {
            for y in 0..console.height() {
                console.draw(x, y, '#', RustConsole::FG_RED);
            }
        }
        console.draw_sprite(140, 0, &self.sprite);
    }
}

fn main() {
    let mut game = Mode7 {
        sprite: RustConsoleSprite::new(8, 8).unwrap_or_else(|error| {
            panic!("Error creating sprite: {:?}", error);
        })
    };
    let mut engine = RustConsoleGameEngine::new(&mut game, 320, 240, 4, 4).unwrap_or_else(|error| {
        panic!("Error creating console: {:?}", error);
    });
    engine.run();
}