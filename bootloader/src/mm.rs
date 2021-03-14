use core::alloc::{GlobalAlloc, Layout};
use rangeset::{Range, RangeSet};
use lockcell::LockCell;
use crate::realmode::{RegisterState, invoke_realmode};

/// Physical memory which is available for use. As reported by E820 with
/// the 1 MiB of memory removed
static PMEM_FREE: LockCell<Option<RangeSet>> = LockCell::new(None);

/// Global allocator for the bootloader. This just uses physical memory as
/// a backing and does not handle any fancy things like fragmentation. Use
/// this carefully.
#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalAllocator = GlobalAllocator;

/// Empty structure that we can implement `GlobalAlloc` for such that we can
/// use the `#[global_allocator]`
struct GlobalAllocator;

unsafe impl GlobalAlloc for GlobalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Get access to physical memory
        let pmem = PMEM_FREE.lock();

        pmem.and_then(|mut x| {
            x.allocate(layout.size() as u64, layout.align() as u64)
        }).unwrap_or(0) as *mut u8
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // We have nothing to free for a zero-size-type
        if layout.size() <= 0 { return; }

        // Get access to physical memory
        let pmem = PMEM_FREE.lock();
        pmem.and_then(|mut x| {
            let end = (ptr as u64).checked_add(layout.size() as u64 - 1)?;
            x.insert(Range { start: ptr as u64, end: end});
            Some(())
        }).expect("Cannot free memory without initialized MM");
    }
}

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    panic!("Out of memory");
}

/// Initialize the physical memory manager. Here we get the memory map from
/// the BIOS via E820 and put i into a `RangeSet` for tracking and allocation.
/// We also subtract off the first 1 MiB of memory to prevent BIOS data
/// structures from being overwritten.
pub fn init() {
    // Create a `RangeSet` to hold the memory that is marked free by the
    // BIOS
    let mut pmem = PMEM_FREE.lock();

    // Make sure we've never initialized the MM before
    assert!(pmem.is_none(),
        "Attempted to re-initialize the memory manager");

    // Create a new empty `RangeSet` for tracking free physical memory
    let mut free_memory = RangeSet::new();

    // Iterate twice as some BIOSes have used memory ranges inside other memory ranges
    // Loop through the memory the BIOS reports twice.
    // The 1st time we accumulate all of the memory that is marked as freee.
    // The 2nd time we remove all ranges that are not marked as free.
    // This sanitizes the BIOS memory map, and makes sure that any memory marked
    // both free and non-free, is not marked free at all.
    for &add_free_mem in &[true, false] {
        // Allocate a register state to use when doing the E820 call
        let mut regs = RegisterState::default();

        // Set the continuation code to 0 for the first E820 call
        regs.ebx = 0;
        loop {
            /// Raw E820 entry, to be filled in by the BIOS
            #[derive(Debug, Default)]
            #[repr(C)]
            struct E820Entry {
                base: u64,
                size: u64,
                typ: u32,
            }

            // Create a zeroed out E820 entry
            let mut entry = E820Entry::default();

            // Set up the args for E820, we use the previous continuation code
            regs.eax = 0xe820;
            regs.edi = &mut entry as *mut E820Entry as u32;
            regs.ecx = core::mem::size_of_val(&entry) as u32;
            regs.edx = u32::from_be_bytes(*b"SMAP");

            // Invoke the BIOS for the E820 memory map
            unsafe { invoke_realmode(0x15, &mut regs); }

            // Check the CF for an error
            if (regs.efl & 1) != 0 {
                panic!("Error reported by BIOS on E820");
            }

            if add_free_mem && entry.typ == 1 && entry.size > 0{
                // If the entry is free, mark the memory as free
                free_memory.insert(Range {
                    start: entry.base,
                    end: entry.base.checked_add(entry.size - 1).unwrap(),
                });
            } else if !add_free_mem && entry.typ != 1 && entry.size > 0 {
                // If the memory is markes as non-free, remove it from the
                // range
                free_memory.remove(Range {
                    start: entry.base,
                    end: entry.base.checked_add(entry.size - 1).unwrap(),
                });
            }

            if regs.ebx == 0 {
                // Last entry
                break;
            }
        }
    }

    // Remove the first 1 MB of memory for use.
    free_memory.remove(Range {
        start: 0x0,
        end: (1024 * 1024) - 1,
    });

    // Set up the global physical memory state with the free memory we have
    // tracked.
    *pmem = Some(free_memory);
}
