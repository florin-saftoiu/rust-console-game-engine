mod bindings {
    windows::include_bindings!();
}
pub use bindings::Windows::Win32::UI::WindowsAndMessaging::{
    VK_UP,
    VK_LEFT,
    VK_RIGHT
};

mod engine;
pub use engine::RustConsoleGameEngine;

mod console;
pub use console::RustConsole;

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