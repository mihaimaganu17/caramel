//! x86 CPU routines

#![feature(llvm_asm)]
#![no_std]

/// Output `val` to I/O port `addr`
#[inline]
pub unsafe fn out8(addr: u16, val: u8) {
    // PUt addr in dx
    // put val in al
    // Does not Clobber(what is clobber)
    // Is volatile
    // Using `intel` syntax
    llvm_asm!("out dx, al" :: "{dx}"(addr), "{al}"(val) :: "volatile", "intel");
}

/// Read an 8-bit value from I/0 port `addr`
#[inline]
pub unsafe fn in8(addr: u16) -> u8{
    let val: u8;
    // al is gonna be updated by this code(an output)
    llvm_asm!("in al, dx" : "={al}"(val) : "{dx}"(addr) :: "volatile", "intel");
    val
}

/// Disable interrupts and halt forever
#[inline]
pub fn halt() -> ! {
    unsafe {
        loop {
            llvm_asm!(r#"
                cli
                hlt
            "# :::: "volatile", "intel");
        }
    }
}
