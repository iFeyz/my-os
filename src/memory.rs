use x86_64::{
    structures::paging::{Page, PhysFrame, Mapper, Size4KiB, FrameAllocator , PageTable},
    VirtAddr,
};

use x86_64::structures::paging::OffsetPageTable;
use x86_64::PhysAddr;

pub unsafe fn init(physical_memory_offset : VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub fn translate_addr(addr : VirtAddr , physical_memory_offset : VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}
// Private function to limite the unsafe scope of rust
fn translate_addr_inner(addr : VirtAddr , physical_memory_offset : VirtAddr) -> Option<PhysAddr> {
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;
    // read the active level 4 table directly from the CR3 register
    let (level_4_table_frame, _) = Cr3::read();
    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    // go through all the table pages
    for &index in &table_indexes {
        // frame -> page table ref
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr : *const PageTable = virt.as_ptr();
        let table = unsafe {&*table_ptr};
        // read entry then update the frame
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge frame at {:?}", addr),
        };
    }
    // calculate the physical addres by adding the page offset
    Some(frame.start_address() + u64::from(addr.page_offset()))    
}

// Returns a mutable ref to the active level 4 PageTable
unsafe fn active_level_4_table(physical_memory_offset : VirtAddr) -> &'static mut PageTable 
{
   use x86_64::registers::control::Cr3;

    
let (level_4_table_frame, _) = Cr3::read();
let phys = level_4_table_frame.start_address();
let virt = physical_memory_offset + phys.as_u64();
let page_table_ptr : *mut PageTable = virt.as_mut_ptr();
&mut *page_table_ptr

}

// Create a mapping to the frame "0xb8000"
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        // FIXME: this is not safe, we do it only for testing
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}
pub struct EmptyFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for EmptyFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        None
    }
}



// Frame allocator return usable frames from the bootloader memory map
pub struct BootInfoFrameAllocator {
    memory_map : &'static MemoryMap,
    next : usize,
}
use bootloader::bootinfo::MemoryMap;
use bootloader::bootinfo::MemoryRegionType;

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map : &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next : 0,
        }
    }
    // Return an iterator over the usable frames
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions
        .map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_address = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_address.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}