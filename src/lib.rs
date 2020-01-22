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
    h_console: HANDLE,
    rect_window: SMALL_RECT,
    screen: Box<[CHAR_INFO]>
}

impl RustConsoleGameEngine {
    pub fn new() -> Result<RustConsoleGameEngine, Error> {
        let h_console = unsafe {
            processenv::GetStdHandle(STD_OUTPUT_HANDLE)
        };
        if h_console == INVALID_HANDLE_VALUE {
            return Err(Error::last_os_error());
        }
        
        let mut rect_window = SMALL_RECT { Left: 0, Top: 0, Right: 1, Bottom: 1 };
        let mut ret = unsafe {
            wincon::SetConsoleWindowInfo(h_console, TRUE, &rect_window)
        };
        if ret == 0 {
            return Err(Error::last_os_error());
        }

        let coord = COORD { X: 120, Y: 40 };
        ret = unsafe {
            wincon::SetConsoleScreenBufferSize(h_console, coord)
        };
        if ret == 0 {
            return Err(Error::last_os_error());
        }

        ret = unsafe {
            wincon::SetConsoleActiveScreenBuffer(h_console)
        };
        if ret == 0 {
            return Err(Error::last_os_error());
        }

        rect_window = SMALL_RECT { Left: 0, Top: 0, Right: 120 - 1, Bottom: 40 - 1 };
        ret = unsafe {
            wincon::SetConsoleWindowInfo(h_console, TRUE, &rect_window)
        };
        if ret == 0 {
            return Err(Error::last_os_error());
        }

        let screen = Box::new(unsafe {
            MaybeUninit::<[CHAR_INFO; 120 * 40]>::zeroed().assume_init()
        });

        Ok(RustConsoleGameEngine {
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
            
            let title = format!("RustConsoleGameEngine - RustConsoleGameExample - FPS: {:3.2}", 1f32 / elapsed_time);
            let wide: Vec<u16> = OsStr::new(&title).encode_wide().chain(iter::once(0)).collect();
            let mut ret = unsafe {
                wincon::SetConsoleTitleW(wide.as_ptr())
            };
            if ret == 0 {
                panic!("Error setting window title: {:?}", Error::last_os_error());
            }
            
            ret = unsafe {
                wincon::WriteConsoleOutputW(self.h_console, self.screen.as_ptr(), COORD { X: 120, Y: 40 }, COORD { X: 0, Y: 0 }, &mut self.rect_window)
            };
            if ret == 0 {
                panic!("Error writing console output: {:?}", Error::last_os_error());
            }
        }
    }

    pub fn clear(&mut self) {
        for x in 0..120 {
            for y in 0..40 {
                self.draw(x, y, 0 as char, 0x0000);
            }
        }
    }

    pub fn draw(&mut self, x: i32, y: i32, c: char, col: u16) {
        if x >= 0 && x < 120 && y >= 0 && y < 40 {
            unsafe {
                *(self.screen[(y * 120 + x) as usize].Char.UnicodeChar_mut()) = c as u16;
            }
            self.screen[(y * 120 + x) as usize].Attributes = col;
        }
    }
}

pub trait RustConsoleGame {
    fn setup(&self);
    fn update(&mut self, engine: &mut RustConsoleGameEngine, elapsed_time: f32);
}
