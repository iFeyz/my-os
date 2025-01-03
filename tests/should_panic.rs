#![no_std] // Pas de bibliothèque standard
#![no_main] // Pas de fonction standard d'entrée main
#![feature(custom_test_frameworks)] // Utilisation du framework de test personnalisé
#![test_runner(test_runner)] // Définition de la fonction test_runner comme fonction de test
#![reexport_test_harness_main = "test_main"] // Réexportation de la fonction test_main

use blog_os::{exit_qemu, serial_print, serial_println, QemuExitCode, Testable, test_runner};
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn should_fail() {
    serial_println!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

// Specified panic handler that returns [ok] if panic occurs
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}