use super::RustConsole;

use std::io::Error;

pub struct RustConsoleSprite {
    width: usize,
    height: usize,
    glyphs: Vec<char>,
    colors: Vec<u16>
}

impl RustConsoleSprite {
    pub fn new(w: usize, h: usize) -> Result<RustConsoleSprite, Error> {
        Ok(RustConsoleSprite {
            width: w,
            height: h,
            glyphs: vec![' '; w * h],
            colors: vec![RustConsole::FG_BLACK; w * h]
        })
    }

    pub fn width(&self) -> usize { self.width }
    
    pub fn height(&self) -> usize { self.height }

    pub fn set_glyph(&mut self, x: usize, y: usize, c: char) {
        if !(x >= self.width || y >= self.height) {
            self.glyphs[y * self.width + x] = c;
        }
    }

    pub fn set_color(&mut self, x: usize, y: usize, col: u16) {
        if !(x >= self.width || y >= self.height) {
            self.colors[y * self.width + x] = col;
        }
    }

    pub fn get_glyph(&self, x: usize, y: usize) -> char {
        if !(x >= self.width || y >= self.height) {
            return self.glyphs[y * self.width + x];
        } else {
            return ' ';
        }
    }

    pub fn get_color(&self, x: usize, y: usize) -> u16 {
        if !(x >= self.width || y >= self.height) {
            return self.colors[y * self.width + x];
        } else {
            return RustConsole::FG_BLACK;
        }
    }

    pub fn sample_glyph(&self, x: f32, y: f32) -> char {
        let sx = (x * self.width as f32) as isize;
        let sy = (y * self.height as f32 - 1f32) as isize;
        if !(sx < 0  || sx >= self.width as isize || sy < 0 || sy >= self.height as isize) {
            return self.glyphs[sy as usize * self.width + sx as usize];
        } else {
            return ' ';
        }
    }

    pub fn sample_color(&self, x: f32, y: f32) -> u16 {
        let sx = (x * self.width as f32) as isize;
        let sy = (y * self.height as f32 - 1f32) as isize;
        if !(sx < 0  || sx >= self.width as isize || sy < 0 || sy >= self.height as isize) {
            return self.colors[sy as usize * self.width + sx as usize];
        } else {
            return RustConsole::FG_BLACK;
        }
    }
}