#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(blog_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
use core::panic::PanicInfo;
use blog_os::vga_buffer::{WRITER, BUFFER_HEIGHT, BUFFER_WIDTH};
mod vga_buffer;
mod serial;
use blog_os::memory;
use bootloader::{BootInfo , entry_point};

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



entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use blog_os::memory::BootInfoFrameAllocator;
    use x86_64::{structures::paging::{Translate , Page} , VirtAddr , };

    blog_os::init();
    println!("Hello World{}", "!");
    
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset)};
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map)};

    let page = Page::containing_address(VirtAddr::new(0));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr : *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

    //Loop through the L4 table 
    //for(i , entry) in l4_table.iter().enumerate() {
       // use x86_64::structures::paging::PageTable;
        //if !entry.is_unused() {
        //    println!("L4 Entry {}: {:?}",i,entry);
            //Get the physical address of the frame
          //  let phys = entry.frame().unwrap().start_address();
            //Convert the physical address to a virtual address
            //let virt = phys.as_u64() + boot_info.physical_memory_offset;
            //Convert the virtual address to a pointer
            //let ptr = VirtAddr::new(virt).as_mut_ptr();
            //Get the L3 table from the pointer
            //let l3_table : &PageTable = unsafe { &*ptr};
            //Loop through the L3 table
            //for (j , entry) in l3_table.iter().enumerate() {
                //if !entry.is_unused() {
    //                    println!("L3 Entry {}: {:?}",j,entry);
    //                }
    //            }
    //    }
    //}

    let addresses = [
        // vga buffer page
        0xb8000,
        // code page
        0x201008,
        // stack page
        0x010_0020_1a10,
        boot_info.physical_memory_offset,
    ];
    for &addr in &addresses {
        let virt = VirtAddr::new(addr);
        let phys = mapper.translate_addr(virt);
        println!("virtual address: {:?} -> physical address: {:?}", virt, phys);
    }

    #[cfg(test)]
    test_main();

    
    println!("It did not crash!");
    blog_os::hlt_loop();
}

    //fn stack_overflow() {
    //    stack_overflow();
    //}

    //stack_overflow();
    
    // invoke a breakpoint exception
    //x86_64::instructions::interrupts::int3(); // Déclenche l'interruption 3 (breakpoint)

    // as before

    //let ptr =  0xdeadbeaf as *mut u8;
    //unsafe { *ptr = 42;}