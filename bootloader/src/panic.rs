// A struct providing information about a panic.
use core::panic::PanicInfo;
use serial::print;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("PANIC:");

    if let Some(loc) = info.location() {
        print!(" {}:{}:{}", loc.file(), loc.line(), loc.column());
    }

    if let Some(msg) = info.message() {
        print!(" {}", msg);
    }

    print!("\n");

    cpu::halt();
}
