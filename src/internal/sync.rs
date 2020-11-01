// Copyright (C) 2020 Gregory Meyer
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{c_int, linux::futex};

use core::{
    cell::UnsafeCell,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr,
    sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering},
};

pub(crate) struct Mutex<T: ?Sized> {
    is_locked: AtomicI32,
    sleeping_count: AtomicUsize,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub(crate) const fn new(t: T) -> Self {
        Self {
            is_locked: AtomicI32::new(0),
            sleeping_count: AtomicUsize::new(0),
            data: UnsafeCell::new(t),
        }
    }

    pub(crate) fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<T: ?Sized> Mutex<T> {
    pub fn lock(&self) -> MutexGuard<'_, T> {
        const SPIN_COUNT: usize = 40;

        let mut spins_since_sleep = 0;
        let mut is_locked = if let Err(e) =
            self.is_locked
                .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
        {
            e
        } else {
            return MutexGuard {
                parent: self,
                phantom: PhantomData,
            };
        };

        loop {
            if spins_since_sleep >= SPIN_COUNT && is_locked == 1 {
                self.sleeping_count.fetch_add(1, Ordering::Relaxed);
                unsafe {
                    futex::sys::futex_wait_private(
                        &self.is_locked as *const _ as *mut c_int,
                        1,
                        ptr::null(),
                    )
                };
                self.sleeping_count.fetch_sub(1, Ordering::Relaxed);

                spins_since_sleep = 0;
            }

            is_locked = if let Err(e) =
                self.is_locked
                    .compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed)
            {
                spins_since_sleep += 1;

                e
            } else {
                return MutexGuard {
                    parent: self,
                    phantom: PhantomData,
                };
            }
        }
    }
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

pub(crate) struct MutexGuard<'p, T: ?Sized + 'p> {
    parent: &'p Mutex<T>,
    phantom: PhantomData<*mut ()>,
}

unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}

impl<'p, T: ?Sized + 'p> Drop for MutexGuard<'p, T> {
    fn drop(&mut self) {
        self.parent.is_locked.store(0, Ordering::Release);

        if self.parent.sleeping_count.load(Ordering::Relaxed) > 0 {
            unsafe {
                futex::sys::futex_wake_private(&self.parent.is_locked as *const _ as *mut c_int, 1)
            };
        }
    }
}

impl<'p, T: ?Sized + 'p> Deref for MutexGuard<'p, T> {
    type Target = T;

    fn deref(&self) -> &'p T {
        unsafe { &*self.parent.data.get() }
    }
}

impl<'p, T: ?Sized + 'p> DerefMut for MutexGuard<'p, T> {
    fn deref_mut(&mut self) -> &'p mut T {
        unsafe { &mut *self.parent.data.get() }
    }
}

pub(crate) struct Once {
    has_run: AtomicBool,
    mtx: Mutex<()>,
}

impl Once {
    pub(crate) const fn new() -> Self {
        Self {
            has_run: AtomicBool::new(false),
            mtx: Mutex::new(()),
        }
    }

    pub(crate) fn call_once<F: FnOnce()>(&self, f: F) -> bool {
        if self.has_run.load(Ordering::Acquire) {
            return false;
        }

        let _guard = self.mtx.lock();

        if self.has_run.load(Ordering::Relaxed) {
            return false;
        }

        f();

        self.has_run.store(true, Ordering::Release);

        true
    }
}
