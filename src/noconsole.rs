use super::KeyState;
use super::sprite::RustConsoleSprite;

use std::io::Error;
use std::mem::{swap, MaybeUninit};

pub struct RustConsole {
    width: usize,
    height: usize,
    font_width: i16,
    font_height: i16,
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

    pub const VK_UP: u32 = 38u32;
    pub const VK_DOWN: u32 = 40u32;
    pub const VK_LEFT: u32 = 37u32;
    pub const VK_RIGHT: u32 = 39u32;
    
    pub(crate) fn new(width: usize, height: usize, font_width: i16, font_height: i16) -> Result<RustConsole, Error> {
        Ok(RustConsole {
            width,
            height,
            font_width,
            font_height,
            keys: [KeyState { pressed: false, released: false, held: false }; 256],
            old_key_states: unsafe { MaybeUninit::<[i16; 256]>::zeroed().assume_init() },
            new_key_states: unsafe { MaybeUninit::<[i16; 256]>::zeroed().assume_init() }
        })
    }
    
    pub(crate) fn write_output(&mut self) {}
    
    pub(crate) fn update_key_states(&mut self) {
        for v_key in 0..256 {
            self.new_key_states[v_key] = 0;
            
            self.keys[v_key].pressed = false;
            self.keys[v_key].released = false;
            
            if self.new_key_states[v_key] != self.old_key_states[v_key] {
                if self.new_key_states[v_key] != 0 {
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
    
    pub(crate) fn flush_input_events(&self) {}
    
    pub(crate) fn handle_input_events(&mut self) {}
    
    pub fn width(&self) -> usize { self.width }
    
    pub fn height(&self) -> usize { self.height }
    
    pub fn font_width(&self) -> i16 { self.font_width }
    
    pub fn font_height(&self) -> i16 { self.font_height }
    
    pub fn key(&self, v_key: usize) -> KeyState { self.keys[v_key] }

    pub fn set_title(&self, _title: String) {}
    
    pub fn resize(&mut self, new_width: usize, new_height: usize, new_font_width: i16, new_font_height: i16) {
        self.flush_input_events();
        
        self.width = new_width;
        self.height = new_height;
        self.font_width = new_font_width;
        self.font_height = new_font_height;
    }
    
    pub fn clear(&mut self) {}
    
    pub fn draw(&mut self, _x: usize, _y: usize, _c: char, _col: u16) {}
    
    pub fn fill(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, c: char, col: u16) {
        for x in x1..x2 {
            for y in y1..y2 {
                self.draw(x, y, c, col);
            }
        }
    }
    
    pub fn draw_string(&mut self, _x: usize, _y: usize, _s: &str, _col: u16) {}
    
    pub fn draw_string_alpha(&mut self, _x: usize, _y: usize, _s: &str, _col: u16) {}
    
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

    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &RustConsoleSprite) {
        for i in 0..sprite.width() {
            for j in 0..sprite.height() {
                if sprite.get_glyph(i, j) != ' ' {
                    self.draw(x + i, y + j, sprite.get_glyph(i, j), sprite.get_color(i, j));
                }
            }
        }
    }
}