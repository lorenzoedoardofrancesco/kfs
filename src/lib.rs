#![no_std]
#![no_main]

use core::panic::PanicInfo;

const VGA_BUFFER_ADDRESS: usize = 0xb8000;
const VGA_COLUMNS: usize = 80;
const VGA_ROWS: usize = 25;
static mut CURSOR_X: usize = 0;
static mut CURSOR_Y: usize = 0;

#[derive(Copy, Clone)]
enum Color {
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

fn clear_screen() {
    let vga_buffer = VGA_BUFFER_ADDRESS as *mut u8;
    reset_cursor();
    for i in 0..VGA_COLUMNS * VGA_ROWS {
        putchar(b' ', Color::White, vga_buffer);
    }
    reset_cursor();
}

fn putstr(s: &str, color: Color) {
    let vga_buffer = VGA_BUFFER_ADDRESS as *mut u8;

    for byte in s.bytes() {
        match byte {
            b'\n' => new_line(),
            byte => putchar(byte, color, vga_buffer),
        }
    }
}

fn putchar(byte: u8, color: Color, vga_buffer: *mut u8) {
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

fn new_line() {
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

fn reset_cursor() {
    unsafe {
        CURSOR_X = 0;
        CURSOR_Y = 0;
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen();
    putstr("KFC", Color::Red);
    putstr(
        " or Kentucky Fired Chicken is an international fast food chain founded in 1930 in North Corbin, Kentucky, USA by Colonel Harland David Sanders. The fried chicken restaurant is the second largest fast food chain in the entire world with only McDonald's surpassing it in terms of the number of fast food locations. In addition to the famous chicken, the restaurant serves a wide variety of foods depending on where in the world it is located such as tacos, ice cream with Jello or fish donuts.",
        Color::White
    );

    loop {
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
    }
}
