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

use core::fmt::{Write, Display};
use core::{intrinsics, str};
use core::panic::PanicInfo;
use lazy_static::*;
use crate::kentos_interrupts::LAST_KEY;

mod kentos_memory;
mod kentos_stdio;
mod kentos_interrupts;
mod kentos_gdt; // we will use this later
mod kentos_utils;

use crate::kentos_interrupts::init_idt;
use crate::kentos_stdio::*;
use kentos_memory::active_level_4_page_table;
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
    unsafe 
    { 
        #[cfg(debug_assertions)]
        run_tests();
    };
    // let phys_mem = VirtAddr::new(boot_info.physical_memory_offset);
    // let l4t = unsafe { active_level_4_page_table(phys_mem) };
    // unsafe {
    //     *(l4t[0].addr().as_u64() as *mut [u8; 11]) = *b"Hello world";
    //     writeln!(WRITER.lock(), "{:#?}", str::from_raw_parts(&*(l4t[0].addr().as_u64() as *const u8), 11)); 
    // }
    let mut input_str: [char; 4096] = ['\0'; 4096];
    let mut char_no_idx: u8 = 0;
    write!(SHELL_WRITER.lock(), ">");
    loop
    {
        x86_64::instructions::interrupts::disable();
        if LAST_KEY.lock().to_ascii_lowercase() != '\0'
        {
            if LAST_KEY.lock().to_ascii_lowercase() == '\n'
            {
                match input_str
                {
                    ['g', 'r', 'e', 'e', 't', '\0', '\0', '\0', ..] => 
                    {
                        writeln!(SHELL_WRITER.lock(), "Hello.");
                    },
                    ['e', 'c', 'h', 'o', ' ', ..] => 
                    {
                        for i in 5..4095
                        {
                            if input_str[i] == '\0'
                            {
                                break;
                            }
                            write!(SHELL_WRITER.lock(), "{}", input_str[i]);
                        }
                        writeln!(SHELL_WRITER.lock());
                    },
                    ['r', 'e', 'b', 'o', 'o', 't', '\0', '\0', '\0', ..] =>
                    {
                        for _ in 1..VGA_HEIGHT
                        {
                            writeln!(SHELL_WRITER.lock());
                        }
                        k_start(boot_info);
                    }

                    ['\0', ..] => {},
                    _ =>
                    {
                        write!(ERROR_WRITER.lock(), "Unknown shell command, ");
                        for i in 0..4095
                        {
                            if input_str[i] == '\0'
                            {
                                break;
                            }
                            write!(ERROR_WRITER.lock(), "{}", input_str[i]);
                        }
                        writeln!(ERROR_WRITER.lock());
                    }
                }
                *(LAST_KEY.lock()) = '\0';
                char_no_idx = 0;
                input_str = ['\0'; 4096];
                write!(SHELL_WRITER.lock(), ">");
                continue;
            }
            input_str[char_no_idx as usize] = *(LAST_KEY.lock());
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
    unsafe { test(1, cause_exception, "exception_handling"); }
}

unsafe extern "C" fn cause_exception() -> core::result::Result<(), ()>
{
    x86_64::instructions::interrupts::int3();
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