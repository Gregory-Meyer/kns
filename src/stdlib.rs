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

use crate::{c_char, c_int, c_long, c_void, errno, internal, stddef::size_t, stdio, syscall};

use core::{hint, mem, num::IntErrorKind, ptr, slice, str};

#[link(name = "kns-rpmalloc", kind = "static")]
extern "C" {
    fn rpmalloc(size: size_t) -> *mut c_void;
    fn rpfree(ptr: *mut c_void);
    fn rpcalloc(num: size_t, size: size_t) -> *mut c_void;
    fn rprealloc(ptr: *mut c_void, size: size_t) -> *mut c_void;
    fn rpposix_memalign(memptr: *mut *mut c_void, alignment: size_t, size: size_t) -> c_int;
    fn rpaligned_alloc(alignment: size_t, size: size_t) -> *mut c_void;
}

#[no_mangle]
pub unsafe extern "C" fn malloc(size: size_t) -> *mut c_void {
    if size == 0 {
        return ptr::null_mut();
    }

    let ptr = rpmalloc(size);

    if ptr.is_null() {
        *internal::errno() = errno::ENOMEM;
    }

    ptr
}

#[no_mangle]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    rpfree(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn calloc(nmemb: size_t, size: size_t) -> *mut c_void {
    if nmemb == 0 || size == 0 {
        return ptr::null_mut();
    }

    let ptr = rpcalloc(nmemb, size);

    if ptr.is_null() {
        *internal::errno() = errno::ENOMEM;
    }

    ptr
}

#[no_mangle]
pub unsafe extern "C" fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void {
    if ptr.is_null() {
        malloc(size)
    } else if size == 0 && !ptr.is_null() {
        free(ptr);

        ptr::null_mut()
    } else {
        let new_ptr = rprealloc(ptr, size);

        if new_ptr.is_null() {
            *internal::errno() = errno::ENOMEM;
        }

        new_ptr
    }
}

#[no_mangle]
pub unsafe extern "C" fn posix_memalign(
    memptr: *mut *mut c_void,
    alignment: size_t,
    size: size_t,
) -> c_int {
    if memptr.is_null()
        || !alignment.is_power_of_two()
        || (alignment % mem::size_of::<*mut c_void>() as size_t) != 0
    {
        errno::EINVAL
    } else if size == 0 {
        *memptr = ptr::null_mut();

        0
    } else {
        rpposix_memalign(memptr, alignment, size)
    }
}

#[no_mangle]
pub unsafe extern "C" fn aligned_alloc(alignment: size_t, size: size_t) -> *mut c_void {
    if !alignment.is_power_of_two()
        || (alignment % mem::size_of::<*mut c_void>() as size_t != 0)
        || (size & (alignment - 1) != 0)
    {
        *internal::errno() = errno::EINVAL;

        ptr::null_mut()
    } else if size == 0 {
        ptr::null_mut()
    } else {
        let ptr = rpaligned_alloc(alignment, size);

        if ptr.is_null() {
            *internal::errno() = errno::ENOMEM;
        }

        ptr
    }
}

#[no_mangle]
pub unsafe extern "C" fn exit(status: c_int) -> ! {
    if !stdio::STDIN.is_null() {
        stdio::fclose(stdio::STDIN);
    }

    if !stdio::STDOUT.is_null() {
        stdio::fclose(stdio::STDOUT);
    }

    if !stdio::STDERR.is_null() {
        stdio::fclose(stdio::STDERR);
    }

    internal::finalize();
    sys::exit_group(status)
}

#[no_mangle]
pub unsafe extern "C" fn strtol(
    nptr: *const c_char,
    endptr: *mut *mut c_char,
    base: c_int,
) -> c_long {
    if nptr.is_null() || base < 2 || base > 36 {
        *internal::errno() = errno::EINVAL;

        return 0;
    }

    if base != 10 {
        *internal::errno() = errno::ENOSYS;

        return 0;
    }

    let mut nptr = nptr as *const u8;

    while (*nptr).is_ascii_whitespace() {
        nptr = nptr.add(1);
    }

    let start = nptr;

    if *start == 0 || (*start != b'+' && *start != b'-' && !(*start).is_ascii_whitespace()) {
        *internal::errno() = errno::EINVAL;

        return 0;
    }

    nptr = nptr.add(1);

    while (*nptr).is_ascii_digit() {
        nptr = nptr.add(1);
    }

    *endptr = nptr as *mut u8 as *mut c_char;

    let as_str = str::from_utf8_unchecked(slice::from_raw_parts(
        start,
        nptr.offset_from(start) as usize,
    ));

    match c_long::from_str_radix(as_str, 10) {
        Ok(r) => r,
        Err(e) => match e.kind() {
            IntErrorKind::Empty => {
                *internal::errno() = errno::EINVAL;

                0
            }
            IntErrorKind::InvalidDigit => unreachable!(),
            IntErrorKind::Overflow => {
                *internal::errno() = errno::ERANGE;

                c_long::MAX
            }
            IntErrorKind::Underflow => {
                *internal::errno() = errno::ERANGE;

                c_long::MIN
            }
            IntErrorKind::Zero => unreachable!(),
            _ => {
                *internal::errno() = errno::EINVAL;

                0
            }
        },
    }
}

pub(crate) mod sys {
    use super::*;

    pub(crate) unsafe fn exit_group(status: c_int) -> ! {
        syscall!(231, status as isize);

        hint::unreachable_unchecked();
    }
}
