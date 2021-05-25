use rust_console_game_engine::{RustConsole, RustConsoleGame, RustConsoleGameEngine, RustConsoleSprite};

struct Mode7 {
    world_x: f32,
    world_y: f32,
    world_a: f32,
    near: f32,
    far: f32,
    fov_half: f32,
    sprite_ground: RustConsoleSprite,
    sprite_sky: RustConsoleSprite
}

impl RustConsoleGame for Mode7 {
    fn name(&self) -> &str { "Mode7" }

    fn setup(&mut self) {}

    fn update(&mut self, console: &mut RustConsole, elapsed_time: f32) {

        // control rendering params dynamically
        if console.key('Q' as usize).held {
            self.near += 0.1f32 * elapsed_time;
        }
        if console.key('A' as usize).held {
            self.near -= 0.1f32 * elapsed_time;
        }

        if console.key('W' as usize).held {
            self.far += 0.1f32 * elapsed_time;
        }
        if console.key('S' as usize).held {
            self.far -= 0.1f32 * elapsed_time;
        }

        if console.key('Z' as usize).held {
            self.fov_half += 0.1f32 * elapsed_time;
        }
        if console.key('X' as usize).held {
            self.fov_half -= 0.1f32 * elapsed_time;
        }

        // create frustum corner points
        let far_x1 = self.world_x + (self.world_a - self.fov_half).cos() * self.far;
        let far_y1 = self.world_y + (self.world_a - self.fov_half).sin() * self.far;

        let near_x1 = self.world_x + (self.world_a - self.fov_half).cos() * self.near;
        let near_y1 = self.world_y + (self.world_a - self.fov_half).sin() * self.near;

        let far_x2 = self.world_x + (self.world_a + self.fov_half).cos() * self.far;
        let far_y2 = self.world_y + (self.world_a + self.fov_half).sin() * self.far;

        let near_x2 = self.world_x + (self.world_a + self.fov_half).cos() * self.near;
        let near_y2 = self.world_y + (self.world_a + self.fov_half).sin() * self.near;

        // starting with furthest away line and work towards the camera
        for y in 0..console.height() / 2 {
            // take a sample point for depth linearly related to rows down screen
            let sample_depth = y as f32 / (console.height() as f32 / 2f32);

            // use sample point in non-linear (1/x) way to enable perspective
            // and grab start and end points for lines across screen
            let start_x = (far_x1 - near_x1) / sample_depth + near_x1;
            let start_y = (far_y1 - near_y1) / sample_depth + near_y1;
            let end_x = (far_x2 - near_x2) / sample_depth + near_x2;
            let end_y = (far_y2 - near_y2) / sample_depth + near_y2;

            // linearly interpolate lines across the screen
            for x in 0..console.width() {
                let sample_width = x as f32 / console.width() as f32;
                let mut sample_x = (end_x - start_x) * sample_width + start_x;
                let mut sample_y = (end_y - start_y) * sample_width + start_y;

                // wrap sample coords to give infinite periodicity on maps
                sample_x = sample_x % 1f32;
                sample_y = sample_y % 1f32;

                // sample symbol and color from map sprite, and draw the
                // pixel to the screen
                let mut sym = self.sprite_ground.sample_glyph(sample_x, sample_y);
                let mut col = self.sprite_ground.sample_color(sample_x, sample_y);
                console.draw(x, y + (console.height() / 2), sym, col);

                // sample symbol and color from sky sprite, we can use same
                // coord, but we need to draw the "inverted" y-location
                sym = self.sprite_sky.sample_glyph(sample_x, sample_y);
                col = self.sprite_sky.sample_color(sample_x, sample_y);
                console.draw(x, (console.height() / 2) - y, sym, col);
            }
        }

        // draw a blanking line to fill gap between sky and ground
        console.draw_line(0, console.height() / 2, console.width(), console.height() / 2, RustConsole::PIXEL_SOLID, RustConsole::FG_CYAN);

        // handle navigation with arrow keys
        if console.key(RustConsole::VK_LEFT as usize).held {
            self.world_a -= 1f32 * elapsed_time;
        }
        
        if console.key(RustConsole::VK_RIGHT as usize).held {
            self.world_a += 1f32 * elapsed_time;
        }

        if console.key(RustConsole::VK_UP as usize).held {
            self.world_x += self.world_a.cos() * 0.2f32 * elapsed_time;
            self.world_y += self.world_a.sin() * 0.2f32 * elapsed_time;
        }
        
        if console.key(RustConsole::VK_DOWN as usize).held {
            self.world_x -= self.world_a.cos() * 0.2f32 * elapsed_time;
            self.world_y -= self.world_a.sin() * 0.2f32 * elapsed_time;
        }
    }
}

fn main() {
    let mut game = Mode7 {
        world_x: 1000f32,
        world_y: 1000f32,
        world_a: 0.1f32,
        near: 0.005f32,
        far: 0.03f32,
        fov_half: 3.14159f32 / 4f32,
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