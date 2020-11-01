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

use crate::{c_int, c_void, stddef::size_t, sys::types::off_t, syscall, wrap_syscall};

pub const MAP_FAILED: *mut c_void = -1isize as *mut c_void;

pub const PROT_READ: c_int = 0x1;
pub const PROT_WRITE: c_int = 0x2;
pub const PROT_EXEC: c_int = 0x4;
pub const PROT_SEM: c_int = 0x8;
pub const PROT_NONE: c_int = 0x0;

pub const MAP_32BIT: c_int = 0;
pub const MAP_HUGE_2MB: c_int = 0;
pub const MAP_HUGE_1GB: c_int = 0;
pub const MAP_UNINITIALIZED: c_int = 0;

pub const MAP_SHARED: c_int = 0x01;
pub const MAP_PRIVATE: c_int = 0x02;
pub const MAP_SHARED_VALIDATE: c_int = 0x03;
pub const MAP_FIXED: c_int = 0x10;
pub const MAP_ANONYMOUS: c_int = 0x20;

pub const MAP_POPULATE: c_int = 0x008000;
pub const MAP_NONBLOCK: c_int = 0x010000;
pub const MAP_STACK: c_int = 0x020000;
pub const MAP_HUGETLB: c_int = 0x040000;
pub const MAP_SYNC: c_int = 0x080000;
pub const MAP_FIXED_NOREPLACE: c_int = 0x100000;

pub const MADV_NORMAL: c_int = 0;
pub const MADV_RANDOM: c_int = 1;
pub const MADV_SEQUENTIAL: c_int = 2;
pub const MADV_WILLNEED: c_int = 3;
pub const MADV_DONTNEED: c_int = 4;

pub const MADV_FREE: c_int = 8;
pub const MADV_REMOVE: c_int = 9;
pub const MADV_DONTFORK: c_int = 10;
pub const MADV_DOFORK: c_int = 11;
pub const MADV_HWPOISON: c_int = 100;
pub const MADV_SOFT_OFFLINE: c_int = 101;

pub const MADV_MERGEABLE: c_int = 12;
pub const MADV_UNMERGEABLE: c_int = 13;

pub const MADV_HUGEPAGE: c_int = 14;
pub const MADV_NOHUGEPAGE: c_int = 15;

pub const MADV_DONTDUMP: c_int = 16;
pub const MADV_DODUMP: c_int = 17;

pub const MADV_WIPEONFORK: c_int = 18;
pub const MADV_KEEPONFORK: c_int = 19;

pub const MADV_COLD: c_int = 20;
pub const MADV_PAGEOUT: c_int = 21;

pub const POSIX_MADV_NORMAL: c_int = MADV_NORMAL;
pub const POSIX_MADV_SEQUENTIAL: c_int = MADV_SEQUENTIAL;
pub const POSIX_MADV_RANDOM: c_int = MADV_RANDOM;
pub const POSIX_MADV_WILLNEED: c_int = MADV_WILLNEED;
pub const POSIX_MADV_DONTNEED: c_int = MADV_DONTNEED;

#[no_mangle]
pub unsafe extern "C" fn mmap(
    addr: *mut c_void,
    length: size_t,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    offset: off_t,
) -> *mut c_void {
    wrap_syscall!(sys::mmap(addr, length, prot, flags, fd, offset,)) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn munmap(addr: *mut c_void, length: size_t) -> c_int {
    wrap_syscall!(sys::munmap(addr, length)) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn madvise(addr: *mut c_void, length: size_t, advice: c_int) -> c_int {
    wrap_syscall!(sys::madvise(addr, length, advice)) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn posix_madvise(addr: *mut c_void, length: size_t, advice: c_int) -> c_int {
    madvise(addr, length, advice)
}

pub(crate) mod sys {
    use super::*;

    pub(crate) unsafe fn mmap(
        addr: *mut c_void,
        length: size_t,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: off_t,
    ) -> isize {
        syscall!(
            9,
            addr as isize,
            length as isize,
            prot as isize,
            flags as isize,
            fd as isize,
            offset as isize
        )
    }

    pub(crate) unsafe fn munmap(addr: *mut c_void, length: size_t) -> isize {
        syscall!(11, addr as isize, length as isize)
    }

    pub(crate) unsafe fn madvise(addr: *mut c_void, length: size_t, advice: c_int) -> isize {
        syscall!(28, addr as isize, length as isize, advice as isize)
    }
}
