use crate::io::outb;

const VGA_BUFFER_ADDRESS: usize = 0xb8000;
const VGA_COLUMNS: usize = 80;
const VGA_ROWS: usize = 25;
const VGA_BUFFER_SIZE: usize = VGA_COLUMNS * VGA_ROWS;
const VGA_LAST_LINE: usize = VGA_ROWS - 1;

const VGA_CTRL_REGISTER: u16 = 0x3d4;
const VGA_DATA_REGISTER: u16 = 0x3d5;
static mut CURSOR_X: usize = 0;
static mut CURSOR_Y: usize = 0;

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

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= VGA_COLUMNS {
                    self.new_line();
                }

                self.buffer.chars[VGA_LAST_LINE][self.column_position] = ScreenChar {
                    ascii_character: byte,
                    color: self.color,
                };
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
	}

	
    fn new_line(&mut self) {
        /*extern "C" {
            fn memcpy(dest: *mut usize, src: *const usize, n: usize) -> *mut u8;
        }
        unsafe {
            memcpy(VGA_BUFFER_ADDRESS, VGA_BUFFER_ADDRESS + 0x50, VGA_BUFFER_SIZE - 0x50);
        }
        self.clear_row(VGA_LAST_LINE);*/
    } 
/*
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color: self.color,
        };
        for col in 0..VGA_COLUMNS - 1 {
            self.buffer.chars[row][col] = blank;
        }
    } */
}

/*

fn reset_cursor() {
    unsafe {
        CURSOR_X = 0;
        CURSOR_Y = 0;
        update_cursor();
    }
}

pub fn update_cursor() {
    let position: u16 = unsafe { (CURSOR_Y * VGA_COLUMNS + CURSOR_X) as u16 };

    unsafe {
        outb(VGA_CTRL_REGISTER, 0x0f);
        outb(VGA_DATA_REGISTER, (position & 0xff) as u8);
        outb(VGA_CTRL_REGISTER, 0x0e);
        outb(VGA_DATA_REGISTER, ((position >> 8) & 0xff) as u8);
    }
}
*/