#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_on_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os_on_rust::println;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    
    os_on_rust::init();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    os_on_rust::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    os_on_rust::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os_on_rust::test_panic_handler(info)
}
