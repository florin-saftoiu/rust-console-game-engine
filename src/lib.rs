#[cfg_attr(target_os = "windows", path = "winconsole.rs")]
#[cfg_attr(not(target_os = "windows"), path = "noconsole.rs")]
mod console;
pub use console::RustConsole;

mod engine;
pub use engine::RustConsoleGameEngine;

mod sprite;
pub use sprite::RustConsoleSprite;

#[derive(Copy, Clone)]
pub struct KeyState {
    pub pressed: bool,
    pub released: bool,
    pub held: bool
}

pub trait RustConsoleGame {
    fn name(&self) -> &str;
    fn setup(&mut self);
    fn update(&mut self, console: &mut RustConsole, elapsed_time: f32);
}