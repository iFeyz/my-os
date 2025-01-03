#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use crate::vga_buffer::{WRITER, BUFFER_HEIGHT, BUFFER_WIDTH};

mod vga_buffer;
mod serial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T where T: Fn(), {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_print!("[ok]");
    }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Testable]) {
    println!("Running {} tests", tests.len());
    for (i, test) in tests.iter().enumerate() {
        print!("test {} ... \n", i);
        test.run();
    }
    println!("\nTest result: ok. {} passed.", tests.len());
    exit_qemu(QemuExitCode::Success);
}   

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Error: {}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info : &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}", info);
    exit_qemu(QemuExitCode::Failed);
    loop {} 
} 

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World!");

    #[cfg(test)]
    test_main();

    loop {}
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}

#[test_case]
fn test_println_simple() {
    for _ in 0..200 {
        println!("test_println_simple output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        serial_println!("Expected '{}', got '{}'", c, char::from(screen_char.ascii_character));
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}