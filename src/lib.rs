#![no_std]
#![no_main]

mod librs;
mod video_graphics_array;
mod io;

use core::panic::PanicInfo;
use librs::clear;

#[no_mangle]
pub extern "C" fn _start() -> ! {
	clear();
    println!("Hello World{}", "!");
    println!(
        "KFC or {} is an international fast food chain founded in 1930 in North Corbin, Kentucky, USA by Colonel Harland David Sanders.",
        1 / 3
    );
    println!(
        "The fried chicken restaurant is the second largest fast food chain in the entire world with only McDonald's surpassing it in terms of the number of fast food locations{}",
        "with 18,875 restaurants in 118 countries and territories as of December 2013."
    );
    println!(
        "In addition to the famous chicken, the restaurant serves a wide variety of foods depending on where in the world it is located such as tacos, ice cream with Jello or fish donuts{}.",
        22 / 7
    );
    loop {
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
    }
}
