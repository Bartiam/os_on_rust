#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os_on_rust::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::{panic::PanicInfo};
use os_on_rust::println;
use bootloader::{BootInfo, entry_point};
use os_on_rust::task::{Task, simple_executor::SimpleExecutor, keyboard};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use os_on_rust::memory::{self, BootInfoFrameAllocator};
    use os_on_rust::allocator;
    use x86_64::{VirtAddr};

    println!("Hello World{}", "!");
    os_on_rust::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");
    
    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    os_on_rust::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
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
