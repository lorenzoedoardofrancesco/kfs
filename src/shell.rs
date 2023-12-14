use crate::librs;
use crate::prompt::PROMPT;
use crate::video_graphics_array::WRITER;
use lazy_static::lazy_static;
use spin::Mutex;

const CMOS_ADDRESS: u16 = 0x70;
const CMOS_DATA: u16 = 0x71;

const MAX_LINE_LENGTH: usize = 76;
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

    fn print_prompt(&self, index: usize) {
        for c in self.get(index).iter().take_while(|&&c| c != 0) {
            PROMPT.lock().insert_char(*c, false);
        }
    }

    pub fn scroll_up(&mut self) {
        if self.index == 0 {
            return;
        }
        PROMPT.lock().init();
        self.index = (self.index - 1) % MAX_HISTORY_LINES;
        self.print_prompt(self.index);
    }

    pub fn scroll_down(&mut self) {
        if self.index == MAX_HISTORY_LINES - 1 {
            return;
        }

        PROMPT.lock().init();
        self.index = (self.index + 1) % MAX_HISTORY_LINES;
        self.print_prompt(self.index);
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
    ((bcd & 0xf0) >> 4) * 10 + (bcd & 0x0f)
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
    // println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    // println!("│ Available commands:                                                         │");
    // println!("├─────────────────────────────────────────────────────────────────────────────┤");
    // println!("│    help   | display this help message                                       │");
    // println!("│    clear  | clear the screen                                                │");
    // println!("│    echo   | display the arguments                                           │");
    // println!("│    printk | print the stack                                                 │");
    // println!("│    time   | print the time                                                  │");
    // println!("│    miao   | print a cat                                                     │");
    // println!("│    reboot | reboot the system                                               │");
    // println!("│    halt   | halt the system                                                 │");
    // println!("│    shutdown | shutdown the system                                           │");
    // println!("│    F1-F4   | switch to tty1-4                                               │");
    // println!("│    F11-F12 | switch tty colors                                              │");
    // println!("├─────────────────────────────────────────────────────────────────────────────┤");
    // println!("│ Type 'history' to view command history                                      │");
    // println!("└─────────────────────────────────────────────────────────────────────────────┘");
	librs::printraw("ABCDEFGHIJKLMNOPQRSTUVWXYZabc");
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

fn date() {
    let (hours, minutes, seconds) = get_rtc_time();
    let (year, month, day) = (read_cmos(0x09), read_cmos(0x08), read_cmos(0x07));
    println!(
        "{:02}/{:02}/{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    );
}

fn uname() {
    println!(
        "{} {} {} {} {} {}",
        "KFC",
        "0.0.1-kfc1-i386",
        "DeepFryer 0.0.1-1kfc1 (2023-12-13)",
        "i386",
        "KFC/Deepnux",
        "A|L"
    );
}

pub fn readline(raw_line: &str) {
    let line = raw_line.trim();
    if line.is_empty() {
        return;
    }
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
        "date" => date(),
        "uname" => uname(),
        _ => {
            if line.starts_with("echo") {
                echo(line);
            } else {
                let mut len = line.len();
                if len > 50 {
                    len = 50;
                }
                println!("Unknown command: {}", line[0..len].trim());
            }
        }
    }
}

pub fn print_welcome_message() {
    librs::clear();
    println!("                                     :---------:    .---------:---------- ");
    println!("                                   :#@@@@@@@@%=     +@@@@@@@#::@@@@@@@@@@.");
    println!("                                 :#@@@@@@@@%=       +@@@@@%:  :@@@@@@@@@@.");
    println!("                               :#@@@@@@@@%=         +@@@%-    :@@@@@@@@@@.");
    println!("                             :#@@@@@@@@@=           +@%-      :@@@@@@@@@@.");
    println!("                           :#@@@@@@@@@=             =-        -@@@@@@@@@@ ");
    println!("                         :#@@@@@@@@@=                        +@@@@@@@@@*. ");
    println!("                       :#@@@@@@@@@=                        +@@@@@@@@@*.   ");
    println!("                     :#@@@@@@@@@=                        +@@@@@@@@@*.     ");
    println!("                   :#@@@@@@@@@=                        +@@@@@@@@@*.       ");
    println!("                 :#@@@@@@@@@=                        +@@@@@@@@@+.         ");
    println!("                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@    +@@@@@@@@@#        :#.");
    println!("                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@    +@@@@@@@@@#      :#@@.");
    println!("                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@    +@@@@@@@@@#    :#@@@@.");
    println!("                 @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@    +@@@@@@@@@#  :#@@@@@@.");
    println!("                 ....................=@@@@@@@@@@    +@@@@@@@@@#:#@@@@@@@@.");
    println!("                                     -@@@@@@@@@@     .................... ");
    println!("                                     -@@@@@@@@@@     by                   ");
    println!("                                     -@@@@@@@@@@          Alix Muller     ");
    println!("                                     -@@@@@@@@@@       Lorenzo Simanic    ");
    println!("                                     .----------                          ");
    println!("");
    println!("                       Welcome to KFC! Type 'help' for a list of commands!");
    PROMPT.lock().init();
}
