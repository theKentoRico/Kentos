use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{registers::control::Cr3, structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB}, PhysAddr, VirtAddr};

pub unsafe extern "C" fn active_level_4_page_table(physmem_off: VirtAddr) -> &'static mut PageTable
{
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physmem_off + phys.as_u64();
    let pg_table_ptr: *mut PageTable = virt.as_mut_ptr();
    unsafe { &mut *pg_table_ptr }
}

pub unsafe fn init(phys_mem: VirtAddr) -> OffsetPageTable<'static>
{
    unsafe
    {
        let l4t = active_level_4_page_table(phys_mem);
        OffsetPageTable::new(l4t, phys_mem)
    }
}

#[allow(dead_code)]
pub struct BootInfoFrameAllocator
{
    memory_map: &'static MemoryMap,
    next: usize
}

impl BootInfoFrameAllocator
{
    pub unsafe fn init(mem_map: &'static MemoryMap) -> Self
    {
        BootInfoFrameAllocator
        {
            memory_map: mem_map,
            next: 0
        }
    }
    pub unsafe fn usable(&self) -> impl Iterator<Item = PhysFrame>
    {
        self.memory_map.
            iter().
            filter(|r| r.region_type == MemoryRegionType::Usable).
            map(|s| s.range.start_addr()..s.range.end_addr()).
            flat_map(|t| t.
                step_by(4096)).map(|addr| 
                    PhysFrame::containing_address(PhysAddr::new(addr))
            )
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator
{
    fn allocate_frame(&mut self) -> Option<PhysFrame>
    {
        let frame = unsafe { self.usable() }.nth(self.next);
        self.next += 1;
        frame
    }
}