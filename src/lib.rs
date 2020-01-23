extern crate winapi;

use winapi::shared::minwindef::TRUE;
use winapi::um::winnt::HANDLE;
use winapi::um::winbase::STD_OUTPUT_HANDLE;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::wincontypes::{SMALL_RECT, CHAR_INFO, COORD};
use winapi::um::processenv;
use winapi::um::wincon;
use std::io::Error;
use std::mem::MaybeUninit;
use std::time::Instant;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter;

pub struct RustConsoleGameEngine {
    width: usize,
    height: usize,
    h_console: HANDLE,
    rect_window: SMALL_RECT,
    screen: Vec<CHAR_INFO>
}

impl RustConsoleGameEngine {
    pub fn width(&self) -> usize { self.width }

    pub fn height(&self) -> usize { self.height }

    pub fn new(width: usize, height: usize) -> Result<RustConsoleGameEngine, Error> {
        let h_console = unsafe { processenv::GetStdHandle(STD_OUTPUT_HANDLE) };
        if h_console == INVALID_HANDLE_VALUE { return Err(Error::last_os_error()); }
        
        let mut rect_window = SMALL_RECT { Left: 0, Top: 0, Right: 1, Bottom: 1 };
        let mut ret = unsafe { wincon::SetConsoleWindowInfo(h_console, TRUE, &rect_window) };
        if ret == 0 { return Err(Error::last_os_error()); }

        let coord = COORD { X: width as i16, Y: height as i16 };
        ret = unsafe { wincon::SetConsoleScreenBufferSize(h_console, coord) };
        if ret == 0 { return Err(Error::last_os_error()); }

        ret = unsafe { wincon::SetConsoleActiveScreenBuffer(h_console) };
        if ret == 0 { return Err(Error::last_os_error()); }

        rect_window = SMALL_RECT { Left: 0, Top: 0, Right: width as i16 - 1, Bottom: height as i16 - 1 };
        ret = unsafe { wincon::SetConsoleWindowInfo(h_console, TRUE, &rect_window) };
        if ret == 0 { return Err(Error::last_os_error()); }

        let screen = vec![unsafe { MaybeUninit::<CHAR_INFO>::zeroed().assume_init() }; width * height];

        Ok(RustConsoleGameEngine {
            width,
            height,
            h_console,
            rect_window,
            screen
        })
    }

    pub fn run(&mut self, game: &mut dyn RustConsoleGame) {
        game.setup();

        let mut tp1 = Instant::now();
        let mut tp2;

        loop {
            tp2 = Instant::now();
            let elapsed_time = tp2.duration_since(tp1).as_secs_f32();
            tp1 = tp2;
            
            game.update(self, elapsed_time);
            
            let title = format!("RustConsoleGameEngine - {} - FPS: {:3.2}", game.name(), 1f32 / elapsed_time);
            let wide: Vec<u16> = OsStr::new(&title).encode_wide().chain(iter::once(0)).collect();
            let mut ret = unsafe { wincon::SetConsoleTitleW(wide.as_ptr()) };
            if ret == 0 { panic!("Error setting window title: {:?}", Error::last_os_error()); }
            
            ret = unsafe { wincon::WriteConsoleOutputW(self.h_console, self.screen.as_ptr(), COORD { X: self.width as i16, Y: self.height as i16 }, COORD { X: 0, Y: 0 }, &mut self.rect_window) };
            if ret == 0 { panic!("Error writing console output: {:?}", Error::last_os_error()); }
        }
    }

    pub fn clear(&mut self) {
        for x in 0..self.width {
            for y in 0..self.height {
                self.draw(x, y, 0 as char, 0x0000);
            }
        }
    }

    pub fn draw(&mut self, x: usize, y: usize, c: char, col: u16) {
        if x < self.width && y < self.height {
            unsafe {
                *(self.screen[y * self.width + x].Char.UnicodeChar_mut()) = c as u16;
            }
            self.screen[y * self.width + x].Attributes = col;
        }
    }
}

pub trait RustConsoleGame {
    fn name(&self) -> &str;
    fn setup(&self);
    fn update(&mut self, engine: &mut RustConsoleGameEngine, elapsed_time: f32);
}
