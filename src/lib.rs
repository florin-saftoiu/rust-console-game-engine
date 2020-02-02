extern crate winapi;
extern crate libc;

use winapi::shared::ntdef::NULL;
use winapi::shared::minwindef::{TRUE, FALSE};
use winapi::um::winnt::{HANDLE, SHORT};
use winapi::um::winbase::{STD_OUTPUT_HANDLE, STD_INPUT_HANDLE};
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::consoleapi;
use winapi::um::wincontypes::{SMALL_RECT, CHAR_INFO, COORD};
use winapi::um::wincon::{self, CONSOLE_FONT_INFOEX, CONSOLE_SCREEN_BUFFER_INFOEX, ENABLE_EXTENDED_FLAGS};
use winapi::um::wingdi::{FF_DONTCARE, FW_NORMAL};
use winapi::um::processenv;
use winapi::um::winuser::{self, GWL_STYLE, WS_MAXIMIZEBOX, WS_SIZEBOX, LWA_ALPHA};
use std::io::{Error, ErrorKind};
use std::mem::{self, MaybeUninit};
use std::time::Instant;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter;

#[derive(Copy, Clone)]
pub struct KeyState {
    pub pressed: bool,
    pub released: bool,
    pub held: bool
}

pub struct RustConsole {
    width: usize,
    height: usize,
    h_console: HANDLE,
    #[allow(dead_code)]
    h_console_input: HANDLE,
    rect_window: SMALL_RECT,
    screen: Vec<CHAR_INFO>,
    keys: [KeyState; 256],
    old_key_states: [SHORT; 256],
    new_key_states: [SHORT; 256]
}

impl RustConsole {
    fn new(width: usize, height: usize, font_width: i16, font_height: i16) -> Result<RustConsole, Error> {
        let h_console = unsafe { processenv::GetStdHandle(STD_OUTPUT_HANDLE) };
        if h_console == INVALID_HANDLE_VALUE { return Err(Error::last_os_error()); }
        if h_console == NULL { return Err(Error::new(ErrorKind::Other, "NULL console handle")); }

        let h_console_input = unsafe { processenv::GetStdHandle(STD_INPUT_HANDLE) };
        if h_console_input == INVALID_HANDLE_VALUE { return Err(Error::last_os_error()); }
        if h_console_input == NULL { return Err(Error::new(ErrorKind::Other, "NULL console input handle")); }
        
        let mut rect_window = SMALL_RECT { Left: 0, Top: 0, Right: 1, Bottom: 1 };
        let mut ret = unsafe { wincon::SetConsoleWindowInfo(h_console, TRUE, &rect_window) };
        if ret == 0 { return Err(Error::last_os_error()); }

        let coord = COORD { X: width as i16, Y: height as i16 };
        ret = unsafe { wincon::SetConsoleScreenBufferSize(h_console, coord) };
        if ret == 0 { return Err(Error::last_os_error()); }

        ret = unsafe { wincon::SetConsoleActiveScreenBuffer(h_console) };
        if ret == 0 { return Err(Error::last_os_error()); }

        let mut face_name: [u16; 32] = Default::default();
        let v = OsStr::new("Consolas").encode_wide().chain(iter::once(0)).collect::<Vec<u16>>();
        face_name[..v.len()].clone_from_slice(&v[..]);
        let mut cfix = CONSOLE_FONT_INFOEX {
            cbSize: mem::size_of::<CONSOLE_FONT_INFOEX>() as u32,
            nFont: 0,
            dwFontSize: COORD { X: font_width, Y: font_height },
            FontFamily: FF_DONTCARE,
            FontWeight: FW_NORMAL as u32,
            FaceName: face_name
        };
        ret = unsafe { wincon::SetCurrentConsoleFontEx(h_console, FALSE, &mut cfix) };
        if ret == 0 { return Err(Error::last_os_error()); }

        let mut csbix = unsafe { MaybeUninit::<CONSOLE_SCREEN_BUFFER_INFOEX>::zeroed().assume_init() };
        csbix.cbSize = mem::size_of::<CONSOLE_SCREEN_BUFFER_INFOEX>() as u32;
        ret = unsafe { wincon::GetConsoleScreenBufferInfoEx(h_console, &mut csbix) };
        if ret == 0 { return Err(Error::last_os_error()); }
        if width as i16 > csbix.dwMaximumWindowSize.X {
            return Err(Error::new(ErrorKind::Other, "Width / font width too big"));
        }
        if height as i16 > csbix.dwMaximumWindowSize.Y {
            return Err(Error::new(ErrorKind::Other, "Height / font height too big"));
        }
        csbix.bFullscreenSupported = FALSE;
        ret = unsafe { wincon::SetConsoleScreenBufferInfoEx(h_console, &mut csbix) };
        if ret == 0 { return Err(Error::last_os_error()); }

        rect_window = SMALL_RECT { Left: 0, Top: 0, Right: width as i16 - 1, Bottom: height as i16 - 1 };
        ret = unsafe { wincon::SetConsoleWindowInfo(h_console, TRUE, &rect_window) };
        if ret == 0 { return Err(Error::last_os_error()); }

        ret = unsafe { consoleapi::SetConsoleMode(h_console_input, ENABLE_EXTENDED_FLAGS) };
        if ret == 0 { return Err(Error::last_os_error()); }

        let h_window = unsafe { wincon::GetConsoleWindow() };
        let ret = unsafe { winuser::GetWindowLongW(h_window, GWL_STYLE) };
        if ret == 0 { return Err(Error::last_os_error()); }
        let ret = unsafe { winuser::SetWindowLongW(h_window, GWL_STYLE, ret & !WS_MAXIMIZEBOX as i32 & !WS_SIZEBOX as i32) };
        if ret == 0 { return Err(Error::last_os_error()); }
        unsafe { winuser::GetSystemMenu(h_window, TRUE) };
        let ret = unsafe { winuser::SetLayeredWindowAttributes(h_window, 0, 255, LWA_ALPHA) };
        if ret == 0 { return Err(Error::last_os_error()); }

        Ok(RustConsole {
            width,
            height,
            h_console,
            h_console_input,
            rect_window,
            screen: vec![unsafe { MaybeUninit::<CHAR_INFO>::zeroed().assume_init() }; width * height],
            keys: [KeyState { pressed: false, released: false, held: false }; 256],
            old_key_states: unsafe { MaybeUninit::<[SHORT; 256]>::zeroed().assume_init() },
            new_key_states: unsafe { MaybeUninit::<[SHORT; 256]>::zeroed().assume_init() }
        })
    }

    fn write_output(&mut self) {
        let ret = unsafe { wincon::WriteConsoleOutputW(self.h_console, self.screen.as_ptr(), COORD { X: self.width as i16, Y: self.height as i16 }, COORD { X: 0, Y: 0 }, &mut self.rect_window) };
        if ret == 0 { panic!("Error writing console output: {:?}", Error::last_os_error()); }
    }

    fn update_key_states(&mut self) {
        for v_key in 0..256 {
            self.new_key_states[v_key] = unsafe { winuser::GetAsyncKeyState(v_key as i32) };

            self.keys[v_key].pressed = false;
            self.keys[v_key].released = false;

            if self.new_key_states[v_key] != self.old_key_states[v_key] {
                if self.new_key_states[v_key] as u16 & 0x8000 != 0 {
                    self.keys[v_key].pressed = !self.keys[v_key].held;
                    self.keys[v_key].held = true;
                } else {
                    self.keys[v_key].released = true;
                    self.keys[v_key].held = false;
                }
            }

            self.old_key_states[v_key] = self.new_key_states[v_key];
        }
    }

    pub fn width(&self) -> usize { self.width }

    pub fn height(&self) -> usize { self.height }

    pub fn key(&self, v_key: usize) -> KeyState { self.keys[v_key] }

    pub fn resize(&mut self, new_width: usize, new_height: usize, new_font_width: i16, new_font_height: i16) {
        let mut rect_window = SMALL_RECT { Left: 0, Top: 0, Right: 1, Bottom: 1 };
        let mut ret = unsafe { wincon::SetConsoleWindowInfo(self.h_console, TRUE, &rect_window) };
        if ret == 0 {
            let error = Error::last_os_error();
            panic!("Error resizing console: {:?}", error);
        }

        let coord = COORD { X: new_width as i16, Y: new_height as i16 };
        ret = unsafe { wincon::SetConsoleScreenBufferSize(self.h_console, coord) };
        if ret == 0 {
            let error = Error::last_os_error();
            panic!("Error resizing console: {:?}", error);
        }
        
        let mut cfix = unsafe { MaybeUninit::<CONSOLE_FONT_INFOEX>::zeroed().assume_init() };
        cfix.cbSize = mem::size_of::<CONSOLE_FONT_INFOEX>() as u32;
        let mut ret = unsafe { wincon::GetCurrentConsoleFontEx(self.h_console, FALSE, &mut cfix) };
        if ret == 0 {
            let error = Error::last_os_error();
            panic!("Error resizing console: {:?}", error);
        }
        cfix.dwFontSize = COORD { X: new_font_width, Y: new_font_height };
        ret = unsafe { wincon::SetCurrentConsoleFontEx(self.h_console, FALSE, &mut cfix) };
        if ret == 0 {
            let error = Error::last_os_error();
            panic!("Error resizing console: {:?}", error);
        }

        let mut csbix = unsafe { MaybeUninit::<CONSOLE_SCREEN_BUFFER_INFOEX>::zeroed().assume_init() };
        csbix.cbSize = mem::size_of::<CONSOLE_SCREEN_BUFFER_INFOEX>() as u32;
        ret = unsafe { wincon::GetConsoleScreenBufferInfoEx(self.h_console, &mut csbix) };
        if ret == 0 {
            let error = Error::last_os_error();
            panic!("Error resizing console: {:?}", error);
        }
        if new_width as i16 > csbix.dwMaximumWindowSize.X {
            let error = Error::new(ErrorKind::Other, "Width / font width too big");
            panic!("Error resizing console: {:?}", error);
        }
        if new_height as i16 > csbix.dwMaximumWindowSize.Y {
            let error = Error::new(ErrorKind::Other, "Height / font height too big");
            panic!("Error resizing console: {:?}", error);
        }

        rect_window = SMALL_RECT { Left: 0, Top: 0, Right: new_width as i16 - 1, Bottom: new_height as i16 - 1 };
        ret = unsafe { wincon::SetConsoleWindowInfo(self.h_console, TRUE, &rect_window) };
        if ret == 0 {
            let error = Error::last_os_error();
            panic!("Error resizing console: {:?}", error);
        }

        self.width = new_width;
        self.height = new_height;
        self.rect_window = rect_window;
        self.screen = vec![unsafe { MaybeUninit::<CHAR_INFO>::zeroed().assume_init() }; new_width * new_height];
    }

    pub fn clear(&mut self) {
        unsafe {
            libc::memset(self.screen.as_mut_ptr() as _, 0, self.screen.len() * mem::size_of::<CHAR_INFO>());
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
    
    pub fn draw_string(&mut self, x: usize, y: usize, s: String, col: u16) {
        for (i, c) in s.chars().enumerate() {
            unsafe {
                *(self.screen[y * self.width + x + i].Char.UnicodeChar_mut()) = c as u16;
            }
            self.screen[y * self.width + x + i].Attributes = col;
        }
    }
}

pub trait RustConsoleGame {
    fn name(&self) -> &str;
    fn setup(&mut self);
    fn update(&mut self, console: &mut RustConsole, elapsed_time: f32);
}

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

        let mut tp1 = Instant::now();
        let mut tp2;

        loop {
            tp2 = Instant::now();
            let elapsed_time = tp2.duration_since(tp1).as_secs_f32();
            tp1 = tp2;
            
            self.console.update_key_states();

            self.game.update(&mut self.console, elapsed_time);
            
            let title = format!("RustConsoleGameEngine - {} - FPS: {:3.2}", self.game.name(), 1f32 / elapsed_time);
            let wide: Vec<u16> = OsStr::new(&title).encode_wide().chain(iter::once(0)).collect();
            let ret = unsafe { wincon::SetConsoleTitleW(wide.as_ptr()) };
            if ret == 0 { panic!("Error setting window title: {:?}", Error::last_os_error()); }
            
            self.console.write_output();
        }
    }
}