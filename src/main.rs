#![feature(core_intrinsics)]
#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]


extern crate bootloader;

use core::fmt::Write;
use core::panic::PanicInfo;

pub mod interrupts;
pub mod text;
pub mod gdt;
pub mod driver;

use x86_64::structures::idt::InterruptStackFrame;



#[panic_handler]
#[no_mangle]
fn panic(_info: &PanicInfo) -> ! {
    let mut writer = text::Writer::new();
    writer.clear(text::PANIC_CLR);
    writer.display("Kernel panic: ", text::PANIC_CLR);
    *text::SCREEN_CLR.lock() = text::PANIC_CLR;
    write!(writer, "{}", _info).unwrap();

    loop {
        x86_64::instructions::hlt();
    }
}

#[no_mangle]
pub fn _start() -> ! {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    print!("FiddleOS by ");
    let mut writer= text::WRITER.lock();

    writer.display("<TORUS>\n", 0x0D);
    writer.display("Licensed under DUH (latest edition)\n", 0xB0);

    // Let later programs lock onto writer.
    drop(writer);

    prompt(['\u{0}'; 128]);

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn prompt(inp: [char; 128]) {
    println!("{:?}", inp);
    let mut writer= text::WRITER.lock();
    writer.display(" $", text::PANIC_CLR)
}