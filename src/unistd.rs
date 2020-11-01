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

use crate::{
    c_char, c_int, c_long, c_void, errno, internal,
    stddef::{size_t, ssize_t},
    sys::types::mode_t,
    syscall, wrap_syscall,
};

pub const STDIN_FILENO: c_int = 0;
pub const STDOUT_FILENO: c_int = 1;
pub const STDERR_FILENO: c_int = 2;

pub const S_ISUID: mode_t = 0o4000;
pub const S_ISGID: mode_t = 0o2000;
pub const S_ISVTX: mode_t = 0o1000;
pub const S_IRUSR: mode_t = 0o0400;
pub const S_IWUSR: mode_t = 0o0200;
pub const S_IXUSR: mode_t = 0o0100;
pub const S_IRGRP: mode_t = 0o0040;
pub const S_IWGRP: mode_t = 0o0020;
pub const S_IXGRP: mode_t = 0o0010;
pub const S_IROTH: mode_t = 0o0004;
pub const S_IWOTH: mode_t = 0o0002;
pub const S_IXOTH: mode_t = 0o0001;

pub const O_RDONLY: c_int = 0o0000000;
pub const O_WRONLY: c_int = 0o0000001;
pub const O_RDWR: c_int = 0o0000002;
pub const O_CREAT: c_int = 0o0000100;
pub const O_EXCL: c_int = 0o0000200;
pub const O_NOCTTY: c_int = 0o0000400;
pub const O_TRUNC: c_int = 0o0001000;
pub const O_APPEND: c_int = 0o0002000;
pub const O_NONBLOCK: c_int = 0o0004000;
pub const O_DSYNC: c_int = 0o0010000;
pub const FASYNC: c_int = 0o0020000;
pub const O_DIRECT: c_int = 0o0040000;
pub const O_LARGEFILE: c_int = 0o0100000;
pub const O_DIRECTORY: c_int = 0o0200000;
pub const O_NOFOLLOW: c_int = 0o0400000;
pub const O_CLOEXEC: c_int = 0o2000000;

pub const _SC_PAGESIZE: c_int = 0;

#[no_mangle]
pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    wrap_syscall!(sys::read(fd, buf, count)) as ssize_t
}

#[no_mangle]
pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    wrap_syscall!(sys::write(fd, buf, count)) as ssize_t
}

#[no_mangle]
pub unsafe extern "C" fn open(pathname: *const c_char, flags: c_int, mode: mode_t) -> c_int {
    wrap_syscall!(sys::open(pathname, flags, mode)) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn close(fd: c_int) -> c_int {
    wrap_syscall!(sys::close(fd)) as c_int
}

#[no_mangle]
pub unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    match name {
        _SC_PAGESIZE => 4096, // TODO: get this from sysfs
        _ => {
            *internal::errno() = errno::EINVAL;

            -1
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn readlink(
    pathname: *const c_char,
    buf: *mut c_char,
    bufsiz: size_t,
) -> ssize_t {
    wrap_syscall!(sys::readlink(pathname, buf, bufsiz)) as ssize_t
}

pub(crate) mod sys {
    use super::*;

    pub(crate) unsafe fn read(fd: c_int, buf: *mut c_void, count: size_t) -> isize {
        syscall!(0, fd as isize, buf as isize, count as isize)
    }

    pub(crate) unsafe fn write(fd: c_int, buf: *const c_void, count: size_t) -> isize {
        syscall!(1, fd as isize, buf as isize, count as isize)
    }

    pub(crate) unsafe fn open(pathname: *const c_char, flags: c_int, mode: mode_t) -> isize {
        syscall!(2, pathname as isize, flags as isize, mode as isize)
    }

    pub(crate) unsafe fn close(fd: c_int) -> isize {
        syscall!(3, fd as isize)
    }

    pub(crate) unsafe fn readlink(
        pathname: *const c_char,
        buf: *mut c_char,
        bufsiz: size_t,
    ) -> isize {
        syscall!(89, pathname as isize, buf as isize, bufsiz as isize)
    }
}
