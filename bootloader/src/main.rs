// # [Attr] is an Outer Attribute
// #! [Attr] is an Inner Attribute
// https://doc.rust-lang.org/reference/attributes.html
#![no_std]
#![no_main]
#![feature(rustc_private, llvm_asm, panic_info_message, alloc_error_handler)]

extern crate alloc;

// This declaration will look for a file name `core_reqs.rs` or 
// `core_reqs/mod.rs` and
// will insert its contents inside a module named `core_reqs` under this scope
mod core_reqs;
mod realmode;
mod mm;
mod panic;
mod pxe;

// Used to not change the function name by compiler mangling
#[no_mangle]
extern fn entry(_bootloader_size: usize) -> ! {
    serial::init();
    mm::init();

    pxe::download();

    cpu::halt();
}
