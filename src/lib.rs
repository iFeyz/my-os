#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

extern crate alloc;
pub mod memory;
pub mod serial;
pub mod vga_buffer;
pub mod gdt;
pub mod interrupts;
pub mod allocator;
use crate::vga_buffer::{WRITER, BUFFER_HEIGHT};
use core::panic::PanicInfo;

pub fn init() {
    gdt::init_gdt();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize()};
    x86_64::instructions::interrupts::enable();
}

pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}
// Halt the CPU until the next interrupt
pub fn hlt_loop() -> ! {
    loop{
        x86_64::instructions::hlt();
    }
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

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

#[cfg(test)]
use bootloader::{entry_point , BootInfo};

#[cfg(test)]
entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info : &'static BootInfo) -> ! {
 init();
 test_main();
 hlt_loop();
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
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i+1].read();
        serial_println!("Expected '{}', got '{}'", c, char::from(screen_char.ascii_character));
        //assert_eq!(char::from(screen_char.ascii_character), c);
    }
}
