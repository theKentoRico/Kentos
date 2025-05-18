use crate::kentos_interrupts::*;

pub extern "C" fn init() -> ()
{
    init_idt();
    unsafe { PICS.lock().initialize(); PICS.lock().write_masks(0b1111_1101, 0b1111_1111); }
    x86_64::instructions::interrupts::enable();
}