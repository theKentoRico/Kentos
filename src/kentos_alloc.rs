use alloc::alloc::*;
use linked_list_allocator::LockedHeap;
use x86_64::{structures::paging::{mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB}, VirtAddr};
use core::ptr::null_mut;

pub static HEAP_START: usize = 0x_4444_4444_0000;
pub static HEAP_SIZE: usize = 1000 * 1024;

pub struct Dummy;

unsafe impl GlobalAlloc for Dummy
{
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8
    {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout)
    {
        panic!("DO NOT CALL THIS FUNCTION");
    }
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(mapper: &mut impl Mapper<Size4KiB>, frame_alloc: &mut impl FrameAllocator<Size4KiB>) -> Result<(), MapToError<Size4KiB>>
{
    let page_range = Page::range_inclusive
    (
        Page::<Size4KiB>::containing_address
        (
            VirtAddr::new(HEAP_START as u64)
        ),
        Page::<Size4KiB>::containing_address
        (
            VirtAddr::new((HEAP_START + HEAP_SIZE) as u64)
        )
    );
    for page in page_range
    {
        let frame = frame_alloc.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::WRITABLE | PageTableFlags::PRESENT;
        unsafe { mapper.map_to(page, frame, flags, frame_alloc)?.flush(); }
    };

    unsafe
    {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    Ok(())
}