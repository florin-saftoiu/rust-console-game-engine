use super::KeyState;
use super::sprite::RustConsoleSprite;

use std::io::Error;

pub struct RustConsole {}

impl RustConsole {
    pub const FG_BLACK: u16 = 0x0000;
    pub const FG_DARK_BLUE: u16 = 0x0001; 
    pub const FG_DARK_GREEN: u16 = 0x0002;
    pub const FG_DARK_CYAN: u16 = 0x0003;
    pub const FG_DARK_RED: u16 = 0x0004;
    pub const FG_DARK_MAGENTA: u16 = 0x0005;
    pub const FG_DARK_YELLOW: u16 = 0x0006;
    pub const FG_GREY: u16 = 0x0007;
    pub const FG_DARK_GREY: u16 = 0x0008;
    pub const FG_BLUE: u16 = 0x0009;
    pub const FG_GREEN: u16 = 0x000a;
    pub const FG_CYAN: u16 = 0x000b;
    pub const FG_RED: u16 = 0x000c;
    pub const FG_MAGENTA: u16 = 0x000d;
    pub const FG_YELLOW: u16 = 0x000e;
    pub const FG_WHITE: u16 = 0x000f;
    pub const BG_BLACK: u16 = 0x0000;
    pub const BG_DARK_BLUE: u16 = 0x0010;
    pub const BG_DARK_GREEN: u16 = 0x0020;
    pub const BG_DARK_CYAN: u16 = 0x0030;
    pub const BG_DARK_RED: u16 = 0x0040;
    pub const BG_DARK_MAGENTA: u16 = 0x0050;
    pub const BG_DARK_YELLOW: u16 = 0x0060;
    pub const BG_GREY: u16 = 0x0070;
    pub const BG_DARK_GREY: u16 = 0x0080;
    pub const BG_BLUE: u16 = 0x0090;
    pub const BG_GREEN: u16 = 0x00a0;
    pub const BG_CYAN: u16 = 0x00b0;
    pub const BG_RED: u16 = 0x00c0;
    pub const BG_MAGENTA: u16 = 0x00d0;
    pub const BG_YELLOW: u16 = 0x00e0;
    pub const BG_WHITE: u16 = 0x00f0;
    
    pub const PIXEL_SOLID: char = '\u{2588}';
    pub const PIXEL_THREEQUARTER: char = '\u{2593}';
    pub const PIXEL_HALF: char  = '\u{2592}';
    pub const PIXEL_QUARTER: char = '\u{2591}';

    pub const VK_UP: u32 = 38u32;
    pub const VK_LEFT: u32 = 37u32;
    pub const VK_RIGHT: u32 = 39u32;
    
    pub(crate) fn new(_width: usize, _height: usize, _font_width: i16, _font_height: i16) -> Result<RustConsole, Error> {
        Ok(RustConsole {})
    }
    
    pub(crate) fn write_output(&mut self) {}
    
    pub(crate) fn update_key_states(&mut self) {}
    
    pub(crate) fn flush_input_events(&self) {}
    
    pub(crate) fn handle_input_events(&mut self) {}
    
    pub fn width(&self) -> usize { 0 }
    
    pub fn height(&self) -> usize { 0 }
    
    pub fn font_width(&self) -> i16 { 0 }
    
    pub fn font_height(&self) -> i16 { 0 }
    
    pub fn key(&self, _v_key: usize) -> KeyState { KeyState { held: false, pressed: false, released: false } }

    pub fn set_title(&self, _title: String) {}
    
    pub fn resize(&mut self, _new_width: usize, _new_height: usize, _new_font_width: i16, _new_font_height: i16) {}
    
    pub fn clear(&mut self) {}
    
    pub fn draw(&mut self, _x: usize, _y: usize, _c: char, _col: u16) {}
    
    pub fn fill(&mut self, _x1: usize, _y1: usize, _x2: usize, _y2: usize, _c: char, _col: u16) {}
    
    pub fn draw_string(&mut self, _x: usize, _y: usize, _s: &str, _col: u16) {}
    
    pub fn draw_string_alpha(&mut self, _x: usize, _y: usize, _s: &str, _col: u16) {}
    
    pub fn draw_line(&mut self, _x1: usize, _y1: usize, _x2: usize, _y2: usize, _c: char, _col: u16) {}
    
    pub fn draw_triangle(&mut self, _x1: usize, _y1: usize, _x2: usize, _y2: usize, _x3: usize, _y3: usize, _c: char, _col: u16) {}
    
    pub fn fill_triangle(&mut self, mut _x1: usize, mut _y1: usize, mut _x2: usize, mut _y2: usize, mut _x3: usize, mut _y3: usize, _c: char, _col: u16) {}
    
    pub fn draw_circle(&mut self, _xc: usize, _yc: usize, _r: usize, _c: char, _col: u16) {}

    pub fn fill_circle(&mut self, _xc: usize, _yc: usize, _r: usize, _c: char, _col: u16) {}

    pub fn draw_sprite(&mut self, _x: usize, _y: usize, _sprite: &RustConsoleSprite) {}
}