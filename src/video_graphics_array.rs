use core::fmt;
use crate::io::outb;
use lazy_static::lazy_static;
use spin::Mutex;

const VGA_BUFFER_ADDRESS: usize = 0xb8000;
const VGA_COLUMNS: usize = 80;
const VGA_ROWS: usize = 25;
const VGA_BUFFER_SIZE: usize = VGA_COLUMNS * VGA_ROWS * 2;
const VGA_LAST_LINE: usize = VGA_ROWS - 1;

const VGA_CTRL_REGISTER: u16 = 0x3d4;
const VGA_DATA_REGISTER: u16 = 0x3d5;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color: Color::new(ColorCode::Yellow, ColorCode::Red),
        buffer: unsafe {
            &mut *(VGA_BUFFER_ADDRESS as *mut VgaBuffer)
        },
    });
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorCode {
    Black = 0x0,
    Blue = 0x1,
    Green = 0x2,
    Cyan = 0x3,
    Red = 0x4,
    Magenta = 0x5,
    Brown = 0x6,
    LightGray = 0x7,
    DarkGray = 0x8,
    LightBlue = 0x9,
    LightGreen = 0xa,
    LightCyan = 0xb,
    LightRed = 0xc,
    Pink = 0xd,
    Yellow = 0xe,
    White = 0xf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct Color(u8);

impl Color {
    fn new(foreground: ColorCode, background: ColorCode) -> Color {
        Color(((background as u8) << 4) | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color: Color,
}

#[repr(transparent)]
struct VgaBuffer {
    chars: [[ScreenChar; VGA_COLUMNS]; VGA_ROWS],
}

pub struct Writer {
    column_position: usize,
    color: Color,
    buffer: &'static mut VgaBuffer,
}

impl VgaBuffer {
    fn read(&self, row: usize, column: usize) -> ScreenChar {
        self.chars[row][column]
    }

    fn write(&mut self, character: ScreenChar, row: usize, column: usize) {
        self.chars[row][column] = character;
    }
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= VGA_COLUMNS {
                    self.new_line();
                }

                self.buffer.write(
                    ScreenChar {
                        ascii_character: byte,
                        color: self.color,
                    },
                    VGA_LAST_LINE,
                    self.column_position
                );

                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
        self.update_cursor(VGA_LAST_LINE, self.column_position);
    }

    fn new_line(&mut self) {
        for row in 1..VGA_ROWS {
            for column in 0..VGA_COLUMNS {
                let character = self.buffer.read(row, column);
                self.buffer.write(character, row - 1, column);
            }
        }
        self.clear_row(VGA_LAST_LINE);
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color: self.color,
        };
        for column in 0..VGA_COLUMNS {
            self.buffer.write(blank, row, column);
        }
        self.column_position = 0;
    }

    pub fn clear_screen(&mut self) {
        for row in 0..VGA_ROWS {
            self.clear_row(row);
        }
        self.update_cursor(VGA_LAST_LINE, self.column_position);
    }

    fn update_cursor(&mut self, row: usize, column: usize) {
        let position: u16 = unsafe { (row * VGA_COLUMNS + column) as u16 };

        unsafe {
            outb(VGA_CTRL_REGISTER, 0x0f);
            outb(VGA_DATA_REGISTER, (position & 0xff) as u8);
            outb(VGA_CTRL_REGISTER, 0x0e);
            outb(VGA_DATA_REGISTER, ((position >> 8) & 0xff) as u8);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}
