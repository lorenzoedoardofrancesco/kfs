use crate::io::{outb};

const VGA_BUFFER_ADDRESS: usize = 0xb8000;
const VGA_COLUMNS: usize = 80;
const VGA_ROWS: usize = 25;
const VGA_CTRL_REGISTER: u16 = 0x3d4;
const VGA_DATA_REGISTER: u16 = 0x3d5;
static mut CURSOR_X: usize = 0;
static mut CURSOR_Y: usize = 0;

#[allow(dead_code)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

pub fn putchar(byte: u8, color: Color) {
    let vga_buffer = VGA_BUFFER_ADDRESS as *mut u8;
    unsafe {
        if CURSOR_X >= VGA_COLUMNS {
            new_line();
        }

        let offset = (CURSOR_Y * VGA_COLUMNS + CURSOR_X) * 2;
        *vga_buffer.offset(offset as isize) = byte;
        *vga_buffer.offset((offset as isize) + 1) = color as u8;
        CURSOR_X += 1;
    }
}

pub fn new_line() {
    unsafe {
        CURSOR_X = 0;
        if CURSOR_Y < VGA_ROWS - 1 {
            CURSOR_Y += 1;
        } else {
            scroll_up();
        }
    }
}

fn scroll_up() {
    let vga_buffer = VGA_BUFFER_ADDRESS as *mut u16;
    for y in 1..VGA_ROWS {
        for x in 0..VGA_COLUMNS {
            let offset = y * VGA_COLUMNS + x;
            let prev_offset = (y - 1) * VGA_COLUMNS + x;
            unsafe {
                *vga_buffer.offset(prev_offset as isize) = *vga_buffer.offset(offset as isize);
            }
        }
    }
    clear_last_line(vga_buffer);
}

fn clear_last_line(vga_buffer: *mut u16) {
    let offset = (VGA_ROWS - 1) * VGA_COLUMNS;
    for x in 0..VGA_COLUMNS {
        unsafe {
            *vga_buffer.offset((offset + x) as isize) = 0;
        }
    }
}

pub fn clear_screen() {
    reset_cursor();
    for _i in 0..VGA_COLUMNS * VGA_ROWS {
        putchar(b' ', Color::White);
    }
    reset_cursor();
}

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