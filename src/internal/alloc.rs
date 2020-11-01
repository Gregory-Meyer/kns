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

use crate::{c_void, stddef::size_t, stdlib};

use core::{
    alloc::Layout,
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    slice,
};

pub(crate) unsafe fn alloc(layout: Layout) -> *mut u8 {
    stdlib::aligned_alloc(layout.align() as size_t, layout.size() as size_t) as *mut u8
}

pub(crate) unsafe fn dealloc(ptr: *mut u8, _layout: Layout) {
    stdlib::free(ptr as *mut c_void)
}

pub(crate) unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
    stdlib::__KNS_aligned_calloc(layout.align() as size_t, 1, layout.size() as size_t) as *mut u8
}

pub(crate) unsafe fn realloc(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    stdlib::__KNS_aligned_realloc(
        ptr as *mut c_void,
        layout.align() as size_t,
        new_size as size_t,
        layout.size() as size_t,
    ) as *mut u8
}

pub(crate) struct Box<T: ?Sized> {
    ptr: NonNull<T>,
}

impl<T> Box<T> {
    pub(crate) fn new(t: T) -> Result<Self, T> {
        if let Some(raw) = NonNull::new(unsafe { alloc(Layout::new::<T>()) as *mut T }) {
            unsafe { ptr::write(raw.as_ptr(), t) };

            Ok(Self { ptr: raw })
        } else {
            Err(t)
        }
    }

    pub(crate) fn new_uninit() -> Result<Box<MaybeUninit<T>>, ()> {
        if let Some(raw) =
            NonNull::new(unsafe { alloc(Layout::new::<MaybeUninit<T>>()) as *mut MaybeUninit<T> })
        {
            Ok(Box { ptr: raw })
        } else {
            Err(())
        }
    }

    pub(crate) fn into_inner(b: Box<T>) -> T {
        let raw = Box::into_raw(b);
        let inner = unsafe { ptr::read(raw) };
        unsafe { dealloc(raw as *mut u8, Layout::new::<T>()) };

        inner
    }
}

impl<T> Box<[T]> {
    pub(crate) fn new_uninit_slice(len: usize) -> Result<Box<[MaybeUninit<T>]>, ()> {
        let layout = Layout::array::<MaybeUninit<T>>(len).map_err(|_| ())?;

        if let Some(raw) = NonNull::new(unsafe { alloc(layout) as *mut MaybeUninit<T> })
            .map(|p| unsafe { NonNull::new_unchecked(slice::from_raw_parts_mut(p.as_ptr(), len)) })
        {
            Ok(Box { ptr: raw })
        } else {
            Err(())
        }
    }
}

impl<T: ?Sized> Box<T> {
    pub(crate) fn into_raw(b: Box<T>) -> *mut T {
        (*ManuallyDrop::new(b)).ptr.as_ptr()
    }

    pub(crate) unsafe fn from_raw(raw: *mut T) -> Self {
        Self {
            ptr: NonNull::new_unchecked(raw),
        }
    }
}

impl<T: ?Sized> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.ptr.as_ptr());

            dealloc(
                self.ptr.as_ptr() as *mut u8,
                Layout::for_value(self.ptr.as_ref()),
            );
        }
    }
}

impl<T: ?Sized> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}
