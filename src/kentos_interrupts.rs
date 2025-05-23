use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::kentos_gdt::DOUBLE_FLT_IDX;
use core::{fmt::Write, iter::Chain};
use crate::kentos_stdio::*;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::*;
use x86_64::instructions::port::Port;
use core::cell::RefCell;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[macro_export]
macro_rules! scancode_to_ascii 
{
    ($code:expr) => 
    {
        match $code 
        {
            0x02 => Some('1'),
            0x03 => Some('2'),
            0x04 => Some('3'),
            0x05 => Some('4'),
            0x06 => Some('5'),
            0x07 => Some('6'),
            0x08 => Some('7'),
            0x09 => Some('8'),
            0x0A => Some('9'),
            0x0B => Some('0'),
            0x0C => Some('-'),
            0x0D => Some('='),
            0x10 => Some('q'),
            0x11 => Some('w'),
            0x12 => Some('e'),
            0x13 => Some('r'),
            0x14 => Some('t'),
            0x15 => Some('y'),
            0x16 => Some('u'),
            0x17 => Some('i'),
            0x18 => Some('o'),
            0x19 => Some('p'),
            0x1E => Some('a'),
            0x1F => Some('s'),
            0x20 => Some('d'),
            0x21 => Some('f'),
            0x22 => Some('g'),
            0x23 => Some('h'),
            0x24 => Some('j'),
            0x25 => Some('k'),
            0x26 => Some('l'),
            0x27 => Some(';'),
            0x2C => Some('z'),
            0x2D => Some('x'),
            0x2E => Some('c'),
            0x2F => Some('v'),
            0x30 => Some('b'),
            0x31 => Some('n'),
            0x32 => Some('m'),
            0x1C => Some('\n'),
            0x39 => Some(' '),
            0x0E => Some('\x08'),
            _ => None
        }
    }
}

lazy_static! 
{
    static ref IDT: InterruptDescriptorTable =
    {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(break_handler);
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        idt.double_fault.set_handler_fn(double_flt_handler);
        idt
    };
}
pub fn init_idt()
{
    IDT.load();
}

extern "x86-interrupt" fn break_handler(stack_frame: InterruptStackFrame)
{
    write!(ERROR_WRITER.lock(), "EXCEPTION BREAKPOINT\n{:#?}\nContinuing execution.\n", stack_frame);
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
#[allow(dead_code)]
enum InterruptIndex
{
    Timer = PIC_1_OFFSET,
    Keyboard
}

impl InterruptIndex
{
    fn as_u8(self) -> u8
    {
        self as u8
    }

    fn as_usize(self) -> usize
    {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) -> ()
{
    #[cfg(debug_assertions)]
    write!(WRITER.lock(), ".");
    unsafe
    {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) -> ()
{
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    let res = scancode_to_ascii!(scancode);
    match res
    {
        Some(x) => unsafe
        {
            *(LAST_KEY.lock()) = x;
            write!(SHELL_WRITER.lock(), "{}", x);
        },
        None => {}
    }
    unsafe
    {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn double_flt_handler(stack_frame: InterruptStackFrame, error: u64) -> !
{
    panic!("EXCEPTION DOUBLE FAULT\n{:#?}\n, code: {} \nContinuing execution.\n", stack_frame, error);
}

lazy_static!
{
    pub static ref LAST_KEY: Mutex<char> = Mutex::new('\0');
}