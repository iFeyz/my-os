#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use blog_os::vga_buffer::{WRITER, BUFFER_HEIGHT, BUFFER_WIDTH};

mod vga_buffer;
mod serial;


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    blog_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info : &PanicInfo) -> ! {
    blog_os::test_panic_handler(info)
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    blog_os::init(); // new

    //fn stack_overflow() {
    //    stack_overflow();
    //}

    //stack_overflow();
    
    // invoke a breakpoint exception
    //x86_64::instructions::interrupts::int3(); // Déclenche l'interruption 3 (breakpoint)

    // as before
    #[cfg(test)]
    test_main();

    unsafe { 
        core::arch::asm!(
            "int 4",  // Déclenche directement l'interruption 4 (overflow)
            options(nomem, nostack)
        );
    }

    println!("It did not crash!");
    blog_os::hlt_loop();
}
