#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_on_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use os_on_rust::println;
use bootloader::{BootInfo, entry_point};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use os_on_rust::memory;
    use os_on_rust::memory::BootInfoFrameAllocator;
    use x86_64::{structures::paging::Page, VirtAddr};

    println!("Hello World{}", "!");
    os_on_rust::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

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
