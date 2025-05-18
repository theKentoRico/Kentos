use x86_64::{registers::control::Cr3, structures::paging::PageTable, VirtAddr};

pub unsafe extern "C" fn active_level_4_page_table(physmem_off: VirtAddr) -> &'static mut PageTable
{
    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physmem_off + phys.as_u64();
    let pg_table_ptr: *mut PageTable = virt.as_mut_ptr();
    unsafe { &mut *pg_table_ptr }
}