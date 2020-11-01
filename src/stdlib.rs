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

use core::{hint, num::IntErrorKind, slice, str};

#[link(name = "kns-rpmalloc", kind = "static")]
extern "C" {
    pub fn malloc(size: size_t) -> *mut c_void;
    pub fn free(ptr: *mut c_void);
    pub fn calloc(nmemb: size_t, size: size_t) -> *mut c_void;
    pub fn realloc(ptr: *mut c_void, size: size_t) -> *mut c_void;

    pub fn posix_memalign(memptr: *mut *mut c_void, alignment: size_t, size: size_t) -> c_int;
    pub fn aligned_alloc(alignment: size_t, size: size_t) -> *mut c_void;
    pub fn __KNS_aligned_calloc(alignment: size_t, nmemb: size_t, size: size_t) -> *mut c_void;
    pub fn __KNS_aligned_realloc(
        ptr: *mut c_void,
        alignment: size_t,
        size: size_t,
        oldsize: size_t,
    ) -> *mut c_void;
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

    internal::rpmalloc_finalize();
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
