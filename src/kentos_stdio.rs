use core::ops::{Deref, DerefMut};

use volatile::Volatile;
use spin::Mutex;
use lazy_static::lazy_static;

pub const VGA_WIDTH: usize = 80;
pub const VGA_HEIGHT: usize = 25;

#[repr(u8)]
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]

pub enum Colour
{
    Black = 0,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    LightMagenta,
    Yellow,
    White
}

#[allow(dead_code)]
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColourCode(u8);

impl ColourCode
{
    pub fn new(fg /*foreground*/: Colour, bg /*background*/: Colour) -> ColourCode
    {
        ColourCode((bg as u8) << 4 | (fg as u8))
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenChar
{
    pub ascii_char: u8,
    pub colour_code: ColourCode
}

pub struct Buffer
{
    pub chars: [[Volatile<ScreenChar>; VGA_WIDTH]; VGA_HEIGHT]
}

pub struct Writer
{
    pub row: usize,
    pub column: usize,
    pub colourc: ColourCode,
    pub buffer: &'static mut Buffer
}

impl Writer
{
    pub fn write_char(&mut self, byte: u8) -> ()
    {
        match byte
        {
            b'\x08' =>
            {
                if self.column == 0
                {
                    self.column = VGA_WIDTH - 1;
                    self.row -= 1;
                }
                else 
                {
                    self.column -= 1;
                }
                self.buffer.chars[self.row][self.column].write
                (
                    ScreenChar { ascii_char: b' ', colour_code: self.colourc }
                );
                
            }
            b'\n' => self.add_newline(),
            byte =>
            {
                if self.column >= VGA_WIDTH
                {
                    self.add_newline();
                }
                let row = self.row;
                let col = self.column;
                self.buffer.chars[row][col].write
                (
                    ScreenChar
                    {
                        ascii_char: byte,
                        colour_code: self.colourc
                    }
                );
                self.column += 1;
            }
        }
    }
    pub fn add_newline(&mut self) -> ()
    {
        for i in 1..VGA_HEIGHT
        {
            for j in 0..VGA_WIDTH
            {
                let character = self.buffer.chars[i][j].read();
                self.buffer.chars[i - 1][j].write(character);
            }
        }
        self.clear_row(VGA_HEIGHT - 1);
        self.column = 0;
    }
    pub fn write_str(&mut self, s: &str) -> ()
    {
        for byte in s.bytes()
        {
            match byte
            {
                0x20..=0x7e | b'\n' | b'\x08' => { self.write_char(byte) }
                _ => { self.write_char(0xfe) }
            }
        }
    }
    pub fn clear_row(&mut self, r: usize)
    {
        let BLANK: ScreenChar = ScreenChar
        {
            ascii_char: b' ',
            colour_code: self.colourc
        };
        for col in 0..VGA_WIDTH 
        {
            self.buffer.chars[r][col].write(BLANK);
        }

    }
}

pub extern "C" fn k_puts(s: &str, fg: Colour, bg: Colour) -> ()
{
    let mut writer = Mutex::new(Writer 
    {
        row: VGA_HEIGHT - 1,
        column: 0,
        colourc: ColourCode::new(fg , bg),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
    writer.lock().write_str(s)
}

impl core::fmt::Write for Writer
{
    fn write_str(&mut self, s: &str) -> core::fmt::Result
    {
        self.write_str(s);
        Ok(())
    }
}

lazy_static! 
{
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer
    {
        column: 0,
        row: VGA_HEIGHT - 1,
        colourc: ColourCode::new(Colour::Yellow, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}


lazy_static! 
{
    pub static ref ERROR_WRITER: Mutex<Writer> = Mutex::new(Writer
    {
        column: 0,
        row: VGA_HEIGHT - 1,
        colourc: ColourCode::new(Colour::LightRed, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

lazy_static! 
{
    pub static ref SUCCESS_WRITER: Mutex<Writer> = Mutex::new(Writer
    {
        column: 0,
        row: VGA_HEIGHT - 1,
        colourc: ColourCode::new(Colour::LightGreen, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

lazy_static! 
{
    pub static ref SHELL_WRITER: Mutex<Writer> = Mutex::new(Writer
    {
        column: 0,
        row: VGA_HEIGHT - 1,
        colourc: ColourCode::new(Colour::White, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}