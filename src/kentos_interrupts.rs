use x86_64::structures::idt::InterruptDescriptorTable;

pub fn InitIdt()
{
    let mut idt = InterruptDescriptorTable::new();
    
}