// # [Attr] is an Outer Attribute
// #! [Attr] is an Inner Attribute
// https://doc.rust-lang.org/reference/attributes.html
#![no_std]
#![no_main]
#![feature(rustc_private)]

// This declaration will look for a file name `core_reqs.rs` or `core_reqs/mod.rs` and
// will insert its contents inside a module named `core_reqs` under this scope
mod core_reqs;

// A struct providing information about a panic.
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { }
}

// Used to not change the function name by compiler mangling
#[no_mangle]
fn entry() {
    panit!("APPLES");
}
