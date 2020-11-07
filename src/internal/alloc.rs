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

use crate::{c_int, c_unsignedint, c_void, internal, stddef::size_t};

use core::{
    alloc::Layout,
    mem,
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    slice,
};

#[link(name = "kns-rpmalloc", kind = "static")]
extern "C" {
    fn rpmalloc_initialize() -> c_int;
    fn rpmalloc_finalize();
    fn rpmalloc_thread_initialize() -> c_int;
    fn rpmalloc_thread_finalize();
    fn rpmalloc(size: size_t) -> *mut c_void;
    fn rpfree(ptr: *mut c_void);
    fn rpcalloc(num: size_t, size: size_t) -> *mut c_void;
    fn rprealloc(ptr: *mut c_void, size: size_t) -> *mut c_void;
    fn rpaligned_realloc(
        ptr: *mut c_void,
        alignment: size_t,
        size: size_t,
        oldsize: size_t,
        flags: c_unsignedint,
    ) -> *mut c_void;
    fn rpaligned_alloc(alignment: size_t, size: size_t) -> *mut c_void;
    fn rpaligned_calloc(alignment: size_t, num: size_t, size: size_t) -> *mut c_void;
}

pub(crate) unsafe fn initialize() {
    assert_eq!(rpmalloc_initialize(), 0);
}

pub(crate) unsafe fn finalize() {
    rpmalloc_finalize();
}

pub(crate) unsafe fn thread_initialize() {
    assert_eq!(rpmalloc_thread_initialize(), 0);
}

pub(crate) unsafe fn thread_finalize() {
    rpmalloc_thread_finalize();
}

pub(crate) unsafe fn alloc(layout: Layout) -> *mut u8 {
    if layout.align() < mem::size_of::<*mut c_void>() {
        rpmalloc(layout.size() as size_t) as *mut u8
    } else {
        let aligned_layout = aligned_alloc_layout(layout);

        rpaligned_alloc(
            aligned_layout.align() as size_t,
            aligned_layout.size() as size_t,
        ) as *mut u8
    }
}

pub(crate) unsafe fn dealloc(ptr: *mut u8, _layout: Layout) {
    rpfree(ptr as *mut c_void);
}

pub(crate) unsafe fn alloc_zeroed(layout: Layout) -> *mut u8 {
    if layout.align() < mem::size_of::<*mut c_void>() {
        rpcalloc(1, layout.size() as size_t) as *mut u8
    } else {
        let aligned_layout = aligned_alloc_layout(layout);

        rpaligned_calloc(
            aligned_layout.align() as size_t,
            1,
            aligned_layout.size() as size_t,
        ) as *mut u8
    }
}

pub(crate) unsafe fn realloc(ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
    if layout.align() < mem::size_of::<*mut c_void>() {
        rprealloc(ptr as *mut c_void, layout.size() as size_t) as *mut u8
    } else {
        let aligned_layout = aligned_alloc_layout(layout);
        let new_layout = Layout::from_size_align(new_size, aligned_layout.align())
            .unwrap()
            .pad_to_align();

        rpaligned_realloc(
            ptr as *mut c_void,
            aligned_layout.align() as size_t,
            new_layout.size() as size_t,
            aligned_layout.size() as size_t,
            0,
        ) as *mut u8
    }
}

fn aligned_alloc_layout(layout: Layout) -> Layout {
    Layout::from_size_align(
        layout.size(),
        internal::round_up_to_nearest_multiple(layout.align(), mem::size_of::<*mut c_void>()),
    )
    .unwrap()
    .pad_to_align()
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

    pub(crate) fn new_zeroed() -> Result<Box<MaybeUninit<T>>, ()> {
        if let Some(raw) = NonNull::new(unsafe {
            alloc_zeroed(Layout::new::<MaybeUninit<T>>()) as *mut MaybeUninit<T>
        }) {
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

    pub(crate) fn new_zeroed_slice(len: usize) -> Result<Box<[MaybeUninit<T>]>, ()> {
        let layout = Layout::array::<MaybeUninit<T>>(len).map_err(|_| ())?;

        if let Some(raw) = NonNull::new(unsafe { alloc_zeroed(layout) as *mut MaybeUninit<T> })
            .map(|p| unsafe { NonNull::new_unchecked(slice::from_raw_parts_mut(p.as_ptr(), len)) })
        {
            Ok(Box { ptr: raw })
        } else {
            Err(())
        }
    }
}

impl<T> Box<[MaybeUninit<T>]> {
    pub(crate) fn realloc(
        mut b: Box<[MaybeUninit<T>]>,
        new_len: usize,
    ) -> Result<Box<[MaybeUninit<T>]>, Box<[MaybeUninit<T>]>> {
        let layout = if let Ok(l) = Layout::array::<MaybeUninit<T>>(b.len()) {
            l
        } else {
            return Err(b);
        };

        let new_layout = if let Ok(l) = Layout::array::<MaybeUninit<T>>(new_len) {
            l
        } else {
            return Err(b);
        };

        if let Some(raw) = NonNull::new(unsafe {
            realloc(b.as_mut_ptr() as *mut u8, layout, new_layout.size()) as *mut MaybeUninit<T>
        })
        .map(|p| unsafe { NonNull::new_unchecked(slice::from_raw_parts_mut(p.as_ptr(), new_len)) })
        {
            Ok(Box { ptr: raw })
        } else {
            Err(b)
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
