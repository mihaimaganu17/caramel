extern crate compiler_builtins;

/// libc `memcpy` implementation in Rust
///
/// This implementation of `memcpy` is overlap safe, making it technically `memmove`
///
/// # Parameters
///
/// * `dest`    - Pointer to memory to copy to
/// * `src`     - Pointer to memory to copy from
/// * `n`       - Number of bytes to copy
///
#[no_mangle]
pub unsafe extern fn memcpy(dest: *mut u8, src: *const u8, n: usize)
        -> *mut u8 {
    memmove(dest, src, n)
}

/// libc `memmove` implementation in Rust
///
/// # Parameters
///
/// * `dest`    - Pointer to memory to copy to
/// * `src`     - Pointer to memory to copy from
/// * `n`       - Number of bytes to copy
///
#[no_mangle]
pub unsafe extern fn memmove(dest: *mut u8, src: *const u8, n: usize)
        -> *mut u8 {
    // Avoid overlapping
    if src < dest as *const u8 {
        // copy backwards
        let mut i = n;
        while i != 0 {
            i -= 1;
            *dest.offset(i as isize) = *src.offset(i as isize);
        }
    } else {
        // copy forwards
        let mut i = 0;
        while i <n {
            *dest.offset(i as isize) = *src.offset(i as isize);
            i += 1;
        }
    }

    dest
}

/// libc `memset` implementation in Rust
///
/// # Parameters
///
/// * `s`       - Pointer to memory to set
/// * `c`       - Character to set `n` bytes in `s` to
/// * `n`       - Number of bytes to set
///
#[no_mangle]
pub unsafe extern fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.offset(i as isize) = c as u8;
        i += 1;
    }

    s
}

/// libc memcmp implementation in Rust
///
/// # Parameters
///
/// * `s1`      - Pointer to memory to compare with s2
/// * `s2`      - Pointer to memory to compare with s1
/// * `n`       - Number of bytes to set
///
#[no_mangle]
pub unsafe extern fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        let a = *s1.offset(i as isize);
        let b = *s2.offset(i as isize);
        if a != b {
            return a as i32 - b as i32
        }
        i += 1;
    }

    0
}

// ----------------------------------------------------------------------------
// Microsoft specific intrinsics
//
// The intrinsics use the stdcall convetion however are not decorated with
// an @<bytes> suffix. To override LLVM from appending this suffix we 
// have an \x01 escape bytes before the name, which prevents LLVM from all
// name mangling
//-----------------------------------------------------------------------------

/// Perform n % d
#[export_name="\x01__aullrem"]
pub extern "stdcall" fn __aullrem(n: u64, d: u64) -> u64 {
    compiler_builtins::int::udiv::__umoddi3(n, d)
}

/// Perform n / d
#[export_name="\x01__aulldiv"]
pub extern "stdcall" fn __aulldiv(n: u64, d: u64) -> u64 {
    compiler_builtins::int::udiv::__udivdi3(n, d)
}

/// Perform n % d
#[export_name="\x01__allrem"]
pub extern "stdcall" fn __allrem(n: i64, d: i64) -> i64 {
    compiler_builtins::int::sdiv::__moddi3(n, d)
}

/// Perform n / d
#[export_name="\x01__alldiv"]
pub extern "stdcall" fn __alldiv(n: i64, d: i64) -> i64 {
    compiler_builtins::int::sdiv::__divdi3(n, d)
}

#[export_name="_fltused"]
pub static FLTUSED: usize = 0;
