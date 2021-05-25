use super::RustConsole;

use std::{convert::TryInto, io::Error, mem::size_of};
use std::io::Read;
use std::fs::File;

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

    pub fn from_path(path: &str) -> Result<RustConsoleSprite, Error> {
        let mut f = File::open(path)?;
        let mut buffer = [0; size_of::<u32>()];
        f.read_exact(&mut buffer)?;
        let w = u32::from_le_bytes(buffer) as usize;
        f.read_exact(&mut buffer)?;
        let h = u32::from_le_bytes(buffer) as usize;
        let mut colors = vec![0; size_of::<u16>() * w * h];
        f.read_exact(&mut colors)?;
        let mut glyphs = vec![0; size_of::<u16>() * w * h];
        f.read_exact(&mut glyphs)?;
        Ok(RustConsoleSprite {
            width: w,
            height: h,
            glyphs: glyphs.chunks_exact(2).map(|c| char::from_u32(u16::from_le_bytes(c[..2].try_into().unwrap()) as u32).unwrap()).collect(),
            colors: colors.chunks_exact(2).map(|c| u16::from_le_bytes(c[..2].try_into().unwrap())).collect()
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