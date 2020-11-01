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

use crate::{c_char, c_int, c_void, stddef::size_t};

use core::{cmp::Ordering, ptr, slice};

#[link(name = "kns-asm", kind = "static")]
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: size_t) -> *mut c_void;
    pub fn memset(s: *mut c_void, c: c_int, n: size_t) -> *mut c_void;
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const c_void, s2: *const c_void, n: size_t) -> c_int {
    let lhs = slice::from_raw_parts(s1 as *const u8, n as usize);
    let rhs = slice::from_raw_parts(s2 as *const u8, n as usize);

    for (l, r) in lhs.iter().cloned().zip(rhs.iter().cloned()) {
        match l.cmp(&r) {
            Ordering::Less => return -1,
            Ordering::Greater => return 1,
            _ => (),
        }
    }

    0
}

#[no_mangle]
pub unsafe extern "C" fn strstr(mut haystack: *const c_char, needle: *const c_char) -> *mut c_char {
    if haystack.is_null() || needle.is_null() {
        return ptr::null_mut();
    }

    let mut needle_next = needle;
    let mut haystack_start = haystack;

    while *haystack != 0 && *needle_next != 0 {
        if *needle_next == *haystack {
            needle_next = needle.add(1);
            haystack = haystack.add(1);
        } else {
            haystack_start = haystack_start.add(1);
            haystack = haystack_start;
            needle_next = needle;
        }
    }

    if *needle_next == 0 {
        haystack_start as *mut c_char
    } else {
        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const c_char) -> size_t {
    if s.is_null() {
        return 0;
    }

    // oh boy
    let as_slice = slice::from_raw_parts(s as *mut u8, usize::MAX);

    as_slice.iter().cloned().position(|ch| ch == 0).unwrap() as size_t
}
