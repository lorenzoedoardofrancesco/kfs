use crate::librs;
use crate::video_graphics_array::WRITER;
use lazy_static::lazy_static;
use spin::Mutex;

const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

const MAX_LINE_LENGTH: usize = 75;
const MAX_HISTORY_LINES: usize = 16;

pub struct History {
    buffer: [[u8; MAX_LINE_LENGTH]; MAX_HISTORY_LINES],
    index: usize,
}

impl History {
    fn new() -> History {
        History {
            buffer: [[0; MAX_LINE_LENGTH]; MAX_HISTORY_LINES],
            index: 0,
        }
    }

    fn add(&mut self, line: &str) {
        self.buffer[self.index] = str_to_array(line);
        self.index = (self.index + 1) % MAX_HISTORY_LINES;
    }

    fn get(&self, index: usize) -> &[u8; MAX_LINE_LENGTH] {
        &self.buffer[index]
    }

    fn print(&self) {
        for i in 0..MAX_HISTORY_LINES {
            let line = self.get(i);
            if line[0] != 0 {
                for &c in line.iter().take_while(|&&c| c != 0) {
                    print!("{}", c as char);
                }
				println!();
            }
        }
    }
}

lazy_static! {
    pub static ref HISTORY: Mutex<History> = Mutex::new(History::new());
}

fn str_to_array(s: &str) -> [u8; MAX_LINE_LENGTH] {
    let mut array = [0; MAX_LINE_LENGTH];
    for (i, c) in s.bytes().enumerate() {
        array[i] = c;
    }
    array
}

fn bcd_to_binary(bcd: u8) -> u8 {
    ((bcd & 0xF0) >> 4) * 10 + (bcd & 0x0F)
}

fn read_cmos(register: u8) -> u8 {
    unsafe {
        use crate::io::{inb, outb};
        outb(CMOS_ADDRESS, register);
        inb(CMOS_DATA)
    }
}

fn get_rtc_time() -> (u8, u8, u8) {
    let seconds = bcd_to_binary(read_cmos(0x00));
    let minutes = bcd_to_binary(read_cmos(0x02));
    let hours = bcd_to_binary(read_cmos(0x04));

    (hours, minutes, seconds)
}

fn help() {
    println!("Available commands:");
    println!("    help   | display this help message");
    println!("    clear  | clear the screen");
    println!("    echo   | display the arguments");
    println!("    printk | print the stack");
    println!("    time   | print the time");
    println!("    miao   | print a cat");
    println!("    reboot | reboot the system");
    println!("    halt   | halt the system");
    println!("    shutdown | shutdown the system");
    println!("    F1-F4   | switch to tty1-4");
    println!("    F11-F12 | switch tty colors");
}

fn clear() {
    WRITER.lock().clear_screen();
}

fn echo(line: &str) {
    let message: &str = &line["echo".len()..];
    if message.starts_with(" ") && message.len() > 1 {
        println!("{}", message[1..].trim());
    } else {
        println!("echo: missing argument");
    }
}

fn time() {
    let (hours, minutes, seconds) = get_rtc_time();
    println!("{:02}:{:02}:{:02}", hours, minutes, seconds);
}

fn miao() {
    println!("  /\\_/\\");
    println!("=( ^.^ )=");
    println!("  )   (   //");
    println!(" (__ __)//");
}

fn reboot() {
    unsafe {
        use crate::io::outb;
        outb(0x64, 0xfe);
    }
}

fn shutdown() {
    unsafe {
        use crate::io::outw;
        outw(0x604, 0x2000);
    }
}

pub fn readline(raw_line: &str) {
    let line = raw_line.trim();
    HISTORY.lock().add(raw_line);
    match line {
        "help" | "man" => help(),
        "clear" => clear(),
        "printk" => librs::print_stack(),
        "time" => time(),
        "miao" => miao(),
        "reboot" => reboot(),
        "halt" => librs::hlt(),
        "shutdown" => shutdown(),
        "history" => HISTORY.lock().print(),
        _ => {
            if line.starts_with("echo") {
                echo(line);
            } else {
                println!("Unknown command: {}", line)
            }
        }
    }
}
