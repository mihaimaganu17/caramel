//! Inner mutability on shared variable through spinlocks
#![no_std]

use core::ops::{Deref, DerefMut};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::hint::spin_loop;

/// A spinlock-guarded variable
pub struct LockCell<T: ?Sized> {
    /// Ticket counter to get new tickets to access the `val`
    ticket: AtomicUsize,

    /// Current ticket value which can be released
    release: AtomicUsize,

    /// Value which is guarded by locks
    val: UnsafeCell<T>,
}

// Sync vs Send
// Sync: 2 threads can have an active reference to the same variable
// Send: move a variable to another thread(passing it with ownership to another
// thread
unsafe impl<T: ?Sized> Sync for LockCell<T> {}

impl<T> LockCell<T> {
    /// Move a `val` into a `LockCell`, a type which allows inner mutability
    /// around ticket spinlocks.
    pub const fn new(val: T) -> Self {
        LockCell {
            val: UnsafeCell::new(val),
            ticket: AtomicUsize::new(0),
            release: AtomicUsize::new(0),
        }
    }
}

impl<T: ?Sized> LockCell<T> {
    /// Acquire exclusive access to `self`
    pub fn lock(&self) -> LockCellGuard<T> {
        // Get a ticket
        let ticket = self.ticket.fetch_add(1, Ordering::SeqCst);

        // Spin while our ticket doesn't match the release
        while self.release.load(Ordering::SeqCst) != ticket {
            spin_loop();
        }

        // At this point we have exclusive access
        LockCellGuard {
            cell: self,
        }
    }
}

/// A guard structure which can implement `Drop` such that locks can be
/// automatically released based on scope.
pub struct LockCellGuard<'a, T: ?Sized> {
    /// A reference to the value we currently exclusive access to
    cell: &'a LockCell<T>,
}

impl<'a, T: ?Sized> Drop for LockCellGuard<'a, T> {
    fn drop(&mut self) {
        // Release the lock
        self.cell.release.fetch_add(1, Ordering::SeqCst);
    }
}

impl<'a, T: ?Sized> Deref for LockCellGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            // Convert a raw pointer to a Rust reference
            &*self.cell.val.get()
        }
    }
}

impl<'a, T: ?Sized> DerefMut for LockCellGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            // Convert a raw pointer to a Rust reference
            &mut *self.cell.val.get()
        }
    }
}

#[cfg(test)]
mod test {
    extern crate std;

    use crate::LockCell;

    #[test]
    fn test_lock() {
        static VAR: LockCell<usize> = LockCell::new(5);

        {
            // We want mutable access to LockCellGuard, but not LockCell itself
            let mut access = VAR.lock();
            assert!(*access == 5);
            *access = 10;

        }
        {
            let access2 = VAR.lock();
            assert!(*access2 == 10);
        }
    }

    #[test]
    #[should_panic]
    fn test_dest() {
        struct Foo;
        impl Drop for Foo {
            fn drop(&mut self) { panic!("Got drop"); }
        }

        let _var = LockCell::new(Foo);
        let _lk = _var.lock();

        std::mem::drop(_lk);
        std::mem::drop(_var);
    }
}
