use std::io::{Error, ErrorKind};
use std::mem::{size_of, MaybeUninit};
use std::time::Instant;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;
use libc::memset;

mod bindings {
    windows::include_bindings!();
}

use bindings::{
    Windows::Win32::{
        System::{
            SystemServices::{
                HANDLE,
                TRUE,
                FALSE
            },
            WindowsProgramming::{
                GetStdHandle,
                STD_OUTPUT_HANDLE,
                STD_INPUT_HANDLE
            },
            Console::{
                SetConsoleWindowInfo,
                SetCurrentConsoleFontEx,
                SetConsoleScreenBufferSize,
                SetConsoleActiveScreenBuffer,
                GetConsoleScreenBufferInfoEx,
                SetConsoleScreenBufferInfoEx,
                SetConsoleMode,
                GetConsoleWindow,
                WriteConsoleOutputW,
                FlushConsoleInputBuffer,
                GetNumberOfConsoleInputEvents,
                ReadConsoleInputW,
                SetConsoleTitleW,
                SMALL_RECT,
                CONSOLE_FONT_INFOEX,
                COORD,
                CONSOLE_SCREEN_BUFFER_INFOEX,
                CONSOLE_MODE,
                ENABLE_EXTENDED_FLAGS,
                CHAR_INFO,
                INPUT_RECORD,
                WINDOW_BUFFER_SIZE_EVENT
            }
        },
        UI::{
            WindowsAndMessaging::{
                GetWindowLongW,
                SetWindowLongW,
                GetSystemMenu,
                SetLayeredWindowAttributes,
                GWL_STYLE,
                WS_MAXIMIZEBOX,
                WS_SIZEBOX,
                LWA_ALPHA
            },
            KeyboardAndMouseInput::GetAsyncKeyState
        },
        Graphics::Gdi::{
            FF_DONTCARE,
            /*FF_MODERN,
            TMPF_TRUETYPE,
            TMPF_VECTOR,*/
            FW_NORMAL
        }
    }
};

#[derive(Copy, Clone)]
pub struct KeyState {
    pub pressed: bool,
    pub released: bool,
    pub held: bool
}

pub struct RustConsole {
    width: usize,
    height: usize,
    font_width: i16,
    font_height: i16,
    h_console: HANDLE,
    h_console_input: HANDLE,
    rect_window: SMALL_RECT,
    screen: Vec<CHAR_INFO>,
    keys: [KeyState; 256],
    old_key_states: [i16; 256],
    new_key_states: [i16; 256]
}

impl RustConsole {
    fn new(width: usize, height: usize, font_width: i16, font_height: i16) -> Result<RustConsole, Error> {
        let h_console = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
        if h_console.is_invalid() { return Err(Error::last_os_error()); }
        if h_console.is_null() { return Err(Error::new(ErrorKind::Other, "NULL console handle")); }

        let h_console_input = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
        if h_console_input.is_invalid() { return Err(Error::last_os_error()); }
        if h_console_input.is_null() { return Err(Error::new(ErrorKind::Other, "NULL console input handle")); }
        
        let mut rect_window = SMALL_RECT { Left: 0, Top: 0, Right: 1, Bottom: 1 };
        let mut ret = unsafe { SetConsoleWindowInfo(h_console, TRUE, &rect_window) };
        if !ret.as_bool() { return Err(Error::last_os_error()); }

        let mut face_name: [u16; 32] = Default::default();
        let v = OsStr::new("Consolas").encode_wide().chain(once(0)).collect::<Vec<u16>>();
        face_name[..v.len()].clone_from_slice(&v[..]);
        let mut cfix = CONSOLE_FONT_INFOEX {
            cbSize: size_of::<CONSOLE_FONT_INFOEX>() as u32,
            nFont: 0,
            dwFontSize: COORD { X: font_width, Y: font_height },
            FontFamily: FF_DONTCARE.0, // FF_MODERN.0 | TMPF_VECTOR | TMPF_TRUETYPE
            FontWeight: FW_NORMAL,
            FaceName: face_name
        };
        ret = unsafe { SetCurrentConsoleFontEx(h_console, FALSE, &mut cfix) };
        if !ret.as_bool() { return Err(Error::last_os_error()); }

        let coord = COORD { X: width as i16, Y: height as i16 };
        ret = unsafe { SetConsoleScreenBufferSize(h_console, coord) };
        if !ret.as_bool() { return Err(Error::last_os_error()); }

        ret = unsafe { SetConsoleActiveScreenBuffer(h_console) };
        if !ret.as_bool() { return Err(Error::last_os_error()); }

        let mut csbix = unsafe { MaybeUninit::<CONSOLE_SCREEN_BUFFER_INFOEX>::zeroed().assume_init() };
        csbix.cbSize = size_of::<CONSOLE_SCREEN_BUFFER_INFOEX>() as u32;
        ret = unsafe { GetConsoleScreenBufferInfoEx(h_console, &mut csbix) };
        if !ret.as_bool() { return Err(Error::last_os_error()); }
        if width as i16 > csbix.dwMaximumWindowSize.X {
            return Err(Error::new(ErrorKind::Other, "Width / font width too big"));
        }
        if height as i16 > csbix.dwMaximumWindowSize.Y {
            return Err(Error::new(ErrorKind::Other, "Height / font height too big"));
        }
        csbix.bFullscreenSupported = FALSE;
        ret = unsafe { SetConsoleScreenBufferInfoEx(h_console, &mut csbix) };
        if !ret.as_bool() { return Err(Error::last_os_error()); }

        rect_window = SMALL_RECT { Left: 0, Top: 0, Right: width as i16 - 1, Bottom: height as i16 - 1 };
        ret = unsafe { SetConsoleWindowInfo(h_console, TRUE, &rect_window) };
        if !ret.as_bool() { return Err(Error::last_os_error()); }

        ret = unsafe { SetConsoleMode(h_console_input, CONSOLE_MODE::from(ENABLE_EXTENDED_FLAGS)) };
        if !ret.as_bool() { return Err(Error::last_os_error()); }

        let h_window = unsafe { GetConsoleWindow() };
        let ret = unsafe { GetWindowLongW(h_window, GWL_STYLE) };
        if ret == 0 { return Err(Error::last_os_error()); }
        let ret = unsafe { SetWindowLongW(h_window, GWL_STYLE, ret & !WS_MAXIMIZEBOX.0 as i32 & !WS_SIZEBOX.0 as i32) };
        if ret == 0 { return Err(Error::last_os_error()); }
        unsafe { GetSystemMenu(h_window, TRUE) };
        let ret = unsafe { SetLayeredWindowAttributes(h_window, 0, 255, LWA_ALPHA) };
        if !ret.as_bool() { return Err(Error::last_os_error()); }

        Ok(RustConsole {
            width,
            height,
            font_width,
            font_height,
            h_console,
            h_console_input,
            rect_window,
            screen: vec![unsafe { MaybeUninit::<CHAR_INFO>::zeroed().assume_init() }; width * height],
            keys: [KeyState { pressed: false, released: false, held: false }; 256],
            old_key_states: unsafe { MaybeUninit::<[i16; 256]>::zeroed().assume_init() },
            new_key_states: unsafe { MaybeUninit::<[i16; 256]>::zeroed().assume_init() }
        })
    }

    fn write_output(&mut self) {
        let ret = unsafe { WriteConsoleOutputW(self.h_console, self.screen.as_ptr(), COORD { X: self.width as i16, Y: self.height as i16 }, COORD { X: 0, Y: 0 }, &mut self.rect_window) };
        if !ret.as_bool() { panic!("Error writing console output: {:?}", Error::last_os_error()); }
    }

    fn update_key_states(&mut self) {
        for v_key in 0..256 {
            self.new_key_states[v_key] = unsafe { GetAsyncKeyState(v_key as i32) };

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

    fn flush_input_events(&self) {
        let ret = unsafe { FlushConsoleInputBuffer(self.h_console_input) };
        if !ret.as_bool() { panic!("Error flushing console input: {:?}", Error::last_os_error()); }
    }

    fn handle_input_events(&mut self) {
        let mut events = 0;
        let mut buffer = [unsafe { MaybeUninit::<INPUT_RECORD>::zeroed().assume_init() }; 32];
        let mut ret = unsafe { GetNumberOfConsoleInputEvents(self.h_console_input, &mut events) };
        if !ret.as_bool() { panic!("Error getting number of console input events: {:?}", Error::last_os_error()); }
        if events > 0 {
            ret = unsafe { ReadConsoleInputW(self.h_console_input, buffer.as_mut_ptr(), events, &mut events) };
            if !ret.as_bool() { panic!("Error reading console input: {:?}", Error::last_os_error()); }
        }

        for i in (0..events).rev() {
            match buffer[i as usize].EventType as u32 {
                WINDOW_BUFFER_SIZE_EVENT => {
                    /*let wbsr = unsafe { buffer[i as usize].Event.WindowBufferSizeEvent };
                    self.width = wbsr.dwSize.X as usize;
                    self.height = wbsr.dwSize.Y as usize;
                    self.screen = vec![unsafe { MaybeUninit::<CHAR_INFO>::zeroed().assume_init() }; self.width * self.height];*/
                },
                _ => {}
            }
        }
    }

    pub fn width(&self) -> usize { self.width }

    pub fn height(&self) -> usize { self.height }

    pub fn font_width(&self) -> i16 { self.font_width }

    pub fn font_height(&self) -> i16 { self.font_height }

    pub fn key(&self, v_key: usize) -> KeyState { self.keys[v_key] }

    pub fn resize(&mut self, new_width: usize, new_height: usize, new_font_width: i16, new_font_height: i16) {
        let mut rect_window = SMALL_RECT { Left: 0, Top: 0, Right: 1, Bottom: 1 };
        let mut ret = unsafe { SetConsoleWindowInfo(self.h_console, TRUE, &rect_window) };
        if !ret.as_bool() { panic!("Error resizing console window: {:?}", Error::last_os_error()); }

        let mut face_name: [u16; 32] = Default::default();
        let v = OsStr::new("Consolas").encode_wide().chain(once(0)).collect::<Vec<u16>>();
        face_name[..v.len()].clone_from_slice(&v[..]);
        let mut cfix = CONSOLE_FONT_INFOEX {
            cbSize: size_of::<CONSOLE_FONT_INFOEX>() as u32,
            nFont: 0,
            dwFontSize: COORD { X: new_font_width, Y: new_font_height },
            FontFamily: FF_DONTCARE.0, // FF_MODERN.0 | TMPF_VECTOR | TMPF_TRUETYPE
            FontWeight: FW_NORMAL as u32,
            FaceName: face_name
        };
        ret = unsafe { SetCurrentConsoleFontEx(self.h_console, FALSE, &mut cfix) };
        if !ret.as_bool() { panic!("Error resizing console font: {:?}", Error::last_os_error()); }

        let coord = COORD { X: new_width as i16, Y: new_height as i16 };
        ret = unsafe { SetConsoleScreenBufferSize(self.h_console, coord) };
        if !ret.as_bool() { panic!("Error resizing console buffer: {:?}", Error::last_os_error()); }

        let mut csbix = unsafe { MaybeUninit::<CONSOLE_SCREEN_BUFFER_INFOEX>::zeroed().assume_init() };
        csbix.cbSize = size_of::<CONSOLE_SCREEN_BUFFER_INFOEX>() as u32;
        ret = unsafe { GetConsoleScreenBufferInfoEx(self.h_console, &mut csbix) };
        if !ret.as_bool() { panic!("Error getting console extended info: {:?}", Error::last_os_error()); }
        if new_width as i16 > csbix.dwMaximumWindowSize.X {
            panic!("Error resizing console: {:?}", Error::new(ErrorKind::Other, "Width / font width too big"));
        }
        if new_height as i16 > csbix.dwMaximumWindowSize.Y {
            panic!("Error resizing console: {:?}", Error::new(ErrorKind::Other, "Height / font height too big"));
        }

        rect_window = SMALL_RECT { Left: 0, Top: 0, Right: new_width as i16 - 1, Bottom: new_height as i16 - 1 };
        ret = unsafe { SetConsoleWindowInfo(self.h_console, TRUE, &rect_window) };
        if !ret.as_bool() { panic!("Error resizing console window: {:?}", Error::last_os_error()); }

        self.flush_input_events();

        self.width = new_width;
        self.height = new_height;
        self.font_width = new_font_width;
        self.font_height = new_font_height;
        self.rect_window = rect_window;
        self.screen = vec![unsafe { MaybeUninit::<CHAR_INFO>::zeroed().assume_init() }; new_width * new_height];
    }

    pub fn clear(&mut self) {
        unsafe {
            memset(self.screen.as_mut_ptr() as _, 0, self.screen.len() * size_of::<CHAR_INFO>());
        }
    }

    pub fn draw(&mut self, x: usize, y: usize, c: char, col: u16) {
        if x < self.width && y < self.height {
            self.screen[y * self.width + x].Char.UnicodeChar = c as u16;
            self.screen[y * self.width + x].Attributes = col;
        }
    }
    
    pub fn draw_string(&mut self, x: usize, y: usize, s: String, col: u16) {
        for (i, c) in s.chars().enumerate() {
            self.screen[y * self.width + x + i].Char.UnicodeChar = c as u16;
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