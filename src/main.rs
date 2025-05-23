#![allow(unused_variables)]
#![allow(internal_features)]
#![allow(stable_features)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(unused_must_use)]
#![allow(improper_ctypes_definitions)]

#![feature(ascii_char)]
#![feature(panic_handler)]
#![feature(core_intrinsics)]
#![feature(abi_x86_interrupt)]
#![feature(c_variadic)]
#![feature(str_from_raw_parts)]

#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::{self, *};
use alloc::*;
use core::fmt::{Write, Display};
use core::{intrinsics, str};
use core::panic::PanicInfo;
use lazy_static::*;
use crate::kentos_interrupts::LAST_KEY;
use crate::kentos_alloc::*;

mod kentos_memory;
mod kentos_stdio;
mod kentos_interrupts;
mod kentos_gdt; // we will use this later
mod kentos_utils;
mod kentos_alloc;

use crate::kentos_interrupts::init_idt;
use crate::kentos_stdio::*;
use kentos_memory::{active_level_4_page_table, BootInfoFrameAllocator};
use kentos_stdio::SHELL_WRITER;
use kentos_utils::*;
use bootloader::*;
use x86_64::VirtAddr;

entry_point!(k_start);

#[panic_handler]
fn panic(info: &PanicInfo) -> ! 
{
    write!(ERROR_WRITER.lock(), "{:?}\nRestarting now...\r", info);
    unsafe
    {
        if !cfg!(debug_assertions)
        {
            intrinsics::abort();
        }
        else 
        {
            loop {}
        }
    }
}
#[unsafe(no_mangle)]
fn k_start(boot_info: &'static BootInfo) -> !
{
    init();
    x86_64::instructions::interrupts::enable();

    writeln!(WRITER.lock(), "Booting up.");

    let phys_mem = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { kentos_memory::init(phys_mem) };
    let mut frame_alloc = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    match init_heap(&mut mapper, &mut frame_alloc)
    {
        Ok(v) =>
        {
            writeln!(WRITER.lock(), "Heap init successful");
        }

        Err(v) =>
        {
            writeln!(WRITER.lock(), "Heap init unsuccessful {:#?}", v);
        }
    }
    unsafe 
    { 
        #[cfg(debug_assertions)]
        run_tests();
    };

    let mut input_str: Vec<char> = vec![];
    let mut char_no_idx: u8 = 0;
    write!(SHELL_WRITER.lock(), ">");
    loop
    {
        x86_64::instructions::interrupts::disable();
        if LAST_KEY.lock().to_ascii_lowercase() != '\0'
        {
            if LAST_KEY.lock().to_ascii_lowercase() == '\n'
            {
                match input_str.as_slice()
                {
                    ['g', 'r', 'e', 'e', 't'] => 
                    {
                        writeln!(SHELL_WRITER.lock(), "Hello.");
                    },
                    ['e', 'c', 'h', 'o', ' ', ..] => 
                    {
                        for (i, c) in input_str.iter().enumerate()
                        {
                            if i >= 5
                            {
                                write!(SHELL_WRITER.lock(), "{}", c);
                            }
                        }
                        writeln!(SHELL_WRITER.lock());
                    },
                    ['e', 'c', 'h', 'o'] => { writeln!(SHELL_WRITER.lock()); },
                    ['r', 'e', 'b', 'o', 'o', 't'] =>
                    {
                        for _ in 1..VGA_HEIGHT
                        {
                            writeln!(SHELL_WRITER.lock());
                        }
                        loop { k_start(boot_info); }
                    },
                    ['c', 'l', 'e', 'a', 'r'] => 
                    {
                        for _ in 1..VGA_HEIGHT
                        {
                            writeln!(SHELL_WRITER.lock());
                        }
                    },

                    [] => {},
                    _ =>
                    {
                        write!(ERROR_WRITER.lock(), "Unknown shell command, ");
                        for c in input_str
                        {
                            write!(ERROR_WRITER.lock(), "{}", c);
                        }
                        writeln!(ERROR_WRITER.lock());
                    }
                }
                *(LAST_KEY.lock()) = '\0';
                char_no_idx = 0;
                input_str = vec![];
                write!(SHELL_WRITER.lock(), ">");
                continue;
            }
            input_str.push(*(LAST_KEY.lock()));
            char_no_idx += 1;
            *(LAST_KEY.lock()) = '\0';
            continue;
        }
        x86_64::instructions::interrupts::enable();
        x86_64::instructions::hlt();
    }
}

unsafe extern "C" fn run_tests() -> ()
{
    writeln!(WRITER.lock(), "Tests beginning");
    unsafe 
    {
        test(1, cause_exception, "exception_handling");
        test(2, test_heap, "heap_allocation");
    }
}

unsafe extern "C" fn cause_exception() -> core::result::Result<(), ()>
{
    x86_64::instructions::interrupts::int3();
    Ok(())
}

unsafe extern "C" fn test_heap() -> core::result::Result<(), ()>
{
    let n: Box<u8> = Box::new(3 as u8);
    writeln!(WRITER.lock(), "{}", n);
    let s: String = String::from("Hello world");
    writeln!(WRITER.lock(), "{}", s);
    Ok(())
}

unsafe extern "C" fn test(test_no: u64, tobe_tested: unsafe extern "C" fn() -> Result<(), ()>, test_name: &'_ str) -> core::result::Result<(), ()>
{
    unsafe
    {
        if tobe_tested() == Ok(())
        {
            writeln!(SUCCESS_WRITER.lock(), "Test no. {}, {}_test, passed.", test_no, test_name);
            Ok(())
        }
        else 
        {
            writeln!(ERROR_WRITER.lock(), "Test no. {}, {}_test, failed.", test_no, test_name);
            Err(())
        }
    }
}