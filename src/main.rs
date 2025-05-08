#![allow(unused_variables)]
#![allow(internal_features)]
#![allow(stable_features)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

#![feature(panic_handler)]
#![feature(core_intrinsics)]
#![feature(abi_x86_interrupt)]
#![feature(c_variadic)]

#![no_std]
#![no_main]

use core::fmt::Write;
use core::intrinsics;
use core::panic::PanicInfo;

mod kentos_stdio;
mod kentos_interrupts;

use kentos_stdio::{Colour, ColourCode, ScreenChar, Buffer, Writer, k_puts, VGA_WIDTH, VGA_HEIGHT};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! 
{
    unsafe { intrinsics::abort(); }
}
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! 
{
    write!(
        Writer 
    {
        row: VGA_HEIGHT - 1,
        column: 0,
        colourc: ColourCode::New(Colour::Yellow, Colour::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    },
     "{}", "Hello\n");
    k_puts("Hi\nHi\n", Colour::Yellow, Colour::Black);
    k_puts("Hi\nHi\n", Colour::Yellow, Colour::Black);
    k_puts("Hi\nHi\n", Colour::Yellow, Colour::Black);
    loop {}
}