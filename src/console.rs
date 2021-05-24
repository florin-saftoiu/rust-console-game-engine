use super::KeyState;
use super::sprite::RustConsoleSprite;

use std::io::{Error, ErrorKind};
use std::mem::{swap, size_of, MaybeUninit};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;
use libc::memset;

use super::bindings::{
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
    
    pub(crate) fn new(width: usize, height: usize, font_width: i16, font_height: i16) -> Result<RustConsole, Error> {
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
    
    pub(crate) fn write_output(&mut self) {
        let ret = unsafe { WriteConsoleOutputW(self.h_console, self.screen.as_ptr(), COORD { X: self.width as i16, Y: self.height as i16 }, COORD { X: 0, Y: 0 }, &mut self.rect_window) };
        if !ret.as_bool() { panic!("Error writing console output: {:?}", Error::last_os_error()); }
    }
    
    pub(crate) fn update_key_states(&mut self) {
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
    
    pub(crate) fn flush_input_events(&self) {
        let ret = unsafe { FlushConsoleInputBuffer(self.h_console_input) };
        if !ret.as_bool() { panic!("Error flushing console input: {:?}", Error::last_os_error()); }
    }
    
    pub(crate) fn handle_input_events(&mut self) {
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
    
    pub fn fill(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, c: char, col: u16) {
        for x in x1..x2 {
            for y in y1..y2 {
                self.draw(x, y, c, col);
            }
        }
    }
    
    pub fn draw_string(&mut self, x: usize, y: usize, s: &str, col: u16) {
        for (i, c) in s.chars().enumerate() {
            self.screen[y * self.width + x + i].Char.UnicodeChar = c as u16;
            self.screen[y * self.width + x + i].Attributes = col;
        }
    }
    
    pub fn draw_string_alpha(&mut self, x: usize, y: usize, s: &str, col: u16) {
        for (i, c) in s.chars().enumerate() {
            if c != ' ' {
                self.screen[y * self.width + x + i].Char.UnicodeChar = c as u16;
                self.screen[y * self.width + x + i].Attributes = col;
            }
        }
    }
    
    pub fn draw_line(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, c: char, col: u16) {
        let dx = x2 as isize - x1 as isize;
        let dy = y2 as isize - y1 as isize;
        let dx1 = dx.abs();
        let dy1 = dy.abs();
        let mut px = 2 * dy1 - dx1;
        let mut py = 2 * dx1 - dy1;
        if dy1 <= dx1 {
            let (mut x, mut y, xe) = if dx >= 0 {
                (x1, y1, x2)
            } else {
                (x2, y2, x1)
            };
            
            self.draw(x, y, c, col);
            
            for _i in x..xe {
                x += 1;
                if px < 0 {
                    px = px + 2 * dy1;
                } else {
                    if (dx < 0 && dy < 0) || (dx > 0 && dy > 0) {
                        y += 1;
                    } else {
                        y -= 1;
                    }
                    px = px + 2 * (dy1 - dx1);
                }
                self.draw(x, y, c, col);
            }
        } else {
            let (mut x, mut y, ye) = if dy >= 0 {
                (x1, y1, y2)
            } else {
                (x2, y2, y1)
            };
            
            self.draw(x, y, c, col);
            
            for _i in y..ye {
                y += 1;
                if py <= 0 {
                    py = py + 2 * dx1;
                } else {
                    if (dx < 0 && dy < 0) || (dx > 0 && dy > 0) {
                        x += 1;
                    } else {
                        x -= 1;
                    }
                    py = py + 2 * (dx1 - dy1);
                }
                self.draw(x, y, c, col);
            }
        }
    }
    
    pub fn draw_triangle(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, x3: usize, y3: usize, c: char, col: u16) {
        self.draw_line(x1, y1, x2, y2, c, col);
        self.draw_line(x2, y2, x3, y3, c, col);
        self.draw_line(x3, y3, x1, y1, c, col);
    }
    
    pub fn fill_triangle(&mut self, mut x1: usize, mut y1: usize, mut x2: usize, mut y2: usize, mut x3: usize, mut y3: usize, c: char, col: u16) {
        let mut changed1 = false;
        let mut changed2 = false;
        
        // sort vertices
        if y1 > y2 {
            swap(&mut y1, &mut y2);
            swap(&mut x1, &mut x2);
        }
        if y1 > y3 {
            swap(&mut y1, &mut y3);
            swap(&mut x1, &mut x3);
        }
        if y2 > y3 {
            swap(&mut y2, &mut y3);
            swap(&mut x2, &mut x3);
        }
        
        // starting points
        let mut t1x = x1 as isize;
        let mut t2x = x1 as isize;
        let mut y = y1;
        let mut dx1 = x2 as isize - x1 as isize;
        let signx1 = if dx1 < 0 {
            dx1 = -dx1;
            -1
        } else {
            1
        };
        let mut dy1 = y2 as isize - y1 as isize;
        
        let mut dx2 = x3 as isize - x1 as isize;
        let signx2 = if dx2 < 0 {
            dx2 = -dx2;
            -1
        } else {
            1
        };
        let mut dy2 = y3 as isize - y1 as isize;
        
        if dy1 > dx1 {
            swap(&mut dx1, & mut dy1);
            changed1 = true;
        }
        if dy2 > dx2 {
            swap(&mut dy2, &mut dx2);
            changed2 = true;
        }
        
        let mut e2 = dx2 >> 1;
        if y1 != y2 { // not flat top, so do the first half
            let mut e1 = dx1 >> 1;
            
            for mut i in 0..dx1 {
                let mut t1xp = 0;
                let mut t2xp = 0;
                let (mut minx, mut maxx) = if t1x < t2x {
                    (t1x, t2x)
                } else {
                    (t2x, t1x)
                };
                // process first line until y value is about to change
                'first_line_1: while i < dx1 {
                    i += 1;
                    e1 += dy1;
                    while e1 >= dx1 {
                        e1 -= dx1;
                        if changed1 {
                            t1xp = signx1;
                        } else {
                            break 'first_line_1;
                        }
                    }
                    if changed1 {
                        break 'first_line_1;
                    } else {
                        t1x += signx1;
                    }
                }
                
                // process second line until y value is about to change
                'second_line_1: loop {
                    e2 += dy2;
                    while e2 >= dx2 {
                        e2 -= dx2;
                        if changed2 {
                            t2xp = signx2;
                        } else {
                            break 'second_line_1;
                        }
                    }
                    if changed2 {
                        break 'second_line_1;
                    } else {
                        t2x += signx2;
                    }
                }
                
                if minx > t1x {
                    minx = t1x;
                }
                if minx > t2x {
                    minx = t2x;
                }
                if maxx < t1x {
                    maxx = t1x;
                }
                if maxx < t2x {
                    maxx = t2x;
                }
                // draw line from min to max points found on the y
                for j in minx..=maxx {
                    self.draw(j as usize, y, c, col);
                }
                
                // now increase y
                if !changed1 {
                    t1x += signx1;
                }
                t1x += t1xp;
                if !changed2 {
                    t2x += signx2;
                }
                t2x += t2xp;
                y += 1;
                if y == y2 {
                    break;
                }
            }
        }
        
        // now, do the second half
        dx1 = x3 as isize - x2 as isize;
        let signx1 = if dx1 < 0 {
            dx1 = -dx1;
            -1
        } else {
            1
        };
        dy1 = y3 as isize - y2 as isize;
        t1x = x2 as isize;
        
        if dy1 > dx1 {
            swap(&mut dy1, &mut dx1);
            changed1 = true;
        } else {
            changed1 = false;
        }
        let mut e1 = dx1 >> 1;
        
        for mut i in 0..=dx1 {
            let mut t1xp = 0;
            let mut t2xp = 0;
            let (mut minx, mut maxx) = if t1x < t2x {
                (t1x, t2x)
            } else {
                (t2x, t1x)
            };
            // process first line until y value is about to change
            'first_line_2: while i < dx1 {
                e1 += dy1;
                while e1 >= dx1 {
                    e1 -= dx1;
                    if changed1 {
                        t1xp = signx1;
                        break;
                    } else {
                        break 'first_line_2;
                    }
                }
                if changed1 {
                    break 'first_line_2;
                } else {
                    t1x += signx1;
                }
                if i < dx1 {
                    i += 1;
                }
            }
            
            // process second line until y value is about to change
            'second_line_2: while t2x != x3 as isize {
                e2 += dy2;
                while e2 >= dx2 {
                    e2 -= dx2;
                    if changed2 {
                        t2xp = signx2;
                    } else {
                        break 'second_line_2;
                    }
                }
                if changed2 {
                    break 'second_line_2;
                } else {
                    t2x += signx2;
                }
            }
            
            if minx > t1x {
                minx = t1x;
            }
            if minx > t2x {
                minx = t2x;
            }
            if maxx < t1x {
                maxx = t1x;
            }
            if maxx < t2x {
                maxx = t2x;
            }
            // draw line from min to max points found on the y
            for j in minx..=maxx {
                self.draw(j as usize, y, c, col);
            }
            
            // now increase y
            if !changed1 {
                t1x += signx1;
            }
            t1x += t1xp;
            if !changed2 {
                t2x += signx2;
            }
            t2x += t2xp;
            y += 1;
            if y > y3 {
                return;
            }
        }
    }
    
    pub fn draw_circle(&mut self, xc: usize, yc: usize, r: usize, c: char, col: u16) {
        let mut x = 0;
        let mut y = r;
        let mut p = 3 - 2 * r as isize;
        if r == 0 { return; }
        
        while y >= x {
            self.draw(xc - x, yc - y, c, col); // upper left left
            self.draw(xc - y, yc - x, c, col); // upper upper left
            self.draw(xc + y, yc - x, c, col); // upper upper right
            self.draw(xc + x, yc - y, c, col); // upper right right
            self.draw(xc - x, yc + y, c, col); // lower left left
            self.draw(xc - y, yc + x, c, col); // lower lower left
            self.draw(xc + y, yc + x, c, col); // lower lower right
            self.draw(xc + x, yc + y, c, col); // lower right right
            if p < 0 {
                p += 4 * x as isize + 6;
                x += 1;
            } else {
                p += 4 * (x as isize - y as isize) + 10;
                x += 1;
                y -= 1;
            }
        }
    }

    pub fn fill_circle(&mut self, xc: usize, yc: usize, r: usize, c: char, col: u16) {
        let mut x = 0;
        let mut y = r;
        let mut p = 3 - 2 * r as isize;
        if r == 0 { return; }
        
        while y >= x {
            for i in xc - x..=xc + x {
                self.draw(i, yc - y, c, col);
            }
            for i in xc - y..=xc + y {
                self.draw(i, yc - x, c, col);
            }
            for i in xc - x..=xc + x {
                self.draw(i, yc + y, c, col);
            }
            for i in xc - y..=xc + y {
                self.draw(i, yc + x, c, col);
            }

            if p < 0 {
                p += 4 * x as isize + 6;
                x += 1;
            } else {
                p += 4 * (x as isize - y as isize) + 10;
                x += 1;
                y -= 1;
            }
        }
    }

    pub fn draw_sprite(&mut self, _x: usize, _y: usize, _sprite: &RustConsoleSprite) {

    }
}