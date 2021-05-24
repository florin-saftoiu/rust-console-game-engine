use super::{RustConsole, RustConsoleGame};
use super::bindings::Windows::Win32::System::Console::SetConsoleTitleW;

use std::time::Instant;
use std::io::Error;

pub struct RustConsoleGameEngine<'a> {
    console: RustConsole,
    game: &'a mut dyn RustConsoleGame
}

impl<'a> RustConsoleGameEngine<'a> {
    pub fn new(game: &'a mut dyn RustConsoleGame, width: usize, height: usize, font_width: i16, font_height: i16) -> Result<RustConsoleGameEngine, Error> {
        Ok(RustConsoleGameEngine {
            console: RustConsole::new(width, height, font_width, font_height)?,
            game
        })
    }
    
    pub fn run(&mut self) {
        self.game.setup();
        
        self.console.flush_input_events();
        
        let mut tp1 = Instant::now();
        let mut tp2;
        
        loop {
            tp2 = Instant::now();
            let elapsed_time = tp2.duration_since(tp1).as_secs_f32();
            tp1 = tp2;
            
            self.console.update_key_states();
            
            self.console.handle_input_events();
            
            self.game.update(&mut self.console, elapsed_time);
            
            let title = format!("RustConsoleGameEngine - {} - FPS: {:3.2}", self.game.name(), 1f32 / elapsed_time);
            let ret = unsafe { SetConsoleTitleW(title) };
            if !ret.as_bool() { panic!("Error setting window title: {:?}", Error::last_os_error()); }
            
            self.console.write_output();
        }
    }
}