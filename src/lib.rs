extern crate winapi;

pub struct RustConsoleGameEngine {
    h_console: winapi::um::winnt::HANDLE,
    rect_window: winapi::um::wincontypes::SMALL_RECT,
    screen: std::boxed::Box<[winapi::um::wincontypes::CHAR_INFO]>
}

impl RustConsoleGameEngine {
    pub fn new() -> std::result::Result<RustConsoleGameEngine, std::io::Error> {
        let h_console = unsafe {
            winapi::um::processenv::GetStdHandle(winapi::um::winbase::STD_OUTPUT_HANDLE)
        };
        if h_console == winapi::um::handleapi::INVALID_HANDLE_VALUE {
            return std::result::Result::Err(std::io::Error::last_os_error());
        }
        
        let mut rect_window = winapi::um::wincontypes::SMALL_RECT { Left: 0, Top: 0, Right: 1, Bottom: 1 };
        let mut ret = unsafe {
            winapi::um::wincon::SetConsoleWindowInfo(h_console, winapi::shared::minwindef::TRUE, &rect_window)
        };
        if ret == 0 {
            return std::result::Result::Err(std::io::Error::last_os_error());
        }

        let coord = winapi::um::wincontypes::COORD { X: 120, Y: 40 };
        ret = unsafe {
            winapi::um::wincon::SetConsoleScreenBufferSize(h_console, coord)
        };
        if ret == 0 {
            return std::result::Result::Err(std::io::Error::last_os_error());
        }

        ret = unsafe {
            winapi::um::wincon::SetConsoleActiveScreenBuffer(h_console)
        };
        if ret == 0 {
            return std::result::Result::Err(std::io::Error::last_os_error());
        }

        rect_window = winapi::um::wincontypes::SMALL_RECT { Left: 0, Top: 0, Right: 120 - 1, Bottom: 40 - 1 };
        ret = unsafe {
            winapi::um::wincon::SetConsoleWindowInfo(h_console, winapi::shared::minwindef::TRUE, &rect_window)
        };
        if ret == 0 {
            return std::result::Result::Err(std::io::Error::last_os_error());
        }

        let screen = std::boxed::Box::new(unsafe {
            std::mem::MaybeUninit::<[winapi::um::wincontypes::CHAR_INFO; 120 * 40]>::zeroed().assume_init()
        });

        std::result::Result::Ok(RustConsoleGameEngine {
            h_console,
            rect_window,
            screen
        })
    }

    pub fn run(&mut self, game: &mut dyn RustConsoleGame) {
        game.setup();

        let mut tp1 = std::time::Instant::now();
        let mut tp2;

        loop {
            tp2 = std::time::Instant::now();
            let elapsed_time = tp2.duration_since(tp1).as_secs_f32();
            tp1 = tp2;
            
            game.update(self, elapsed_time);
            
            use std::os::windows::ffi::OsStrExt;
            let title = format!("RustConsoleGameEngine - RustConsoleGameExample - FPS: {:3.2}", 1f32 / elapsed_time);
            let wide: std::vec::Vec<u16> = std::ffi::OsStr::new(&title).encode_wide().chain(std::iter::once(0)).collect();
            let mut ret = unsafe {
                winapi::um::wincon::SetConsoleTitleW(wide.as_ptr())
            };
            if ret == 0 {
                panic!("Error setting window title: {:?}", std::io::Error::last_os_error());
            }
            
            ret = unsafe {
                winapi::um::wincon::WriteConsoleOutputW(self.h_console, self.screen.as_ptr(), winapi::um::wincontypes::COORD { X: 120, Y: 40 }, winapi::um::wincontypes::COORD { X: 0, Y: 0 }, &mut self.rect_window)
            };
            if ret == 0 {
                panic!("Error writing console output: {:?}", std::io::Error::last_os_error());
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
