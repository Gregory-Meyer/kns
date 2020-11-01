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
    c_int,
    sys::types::{mode_t, stat},
    wrap_syscall,
};

pub const S_IFMT: mode_t = 0o170000; // bit mask for the file type bit field

pub const S_IFSOCK: mode_t = 0o140000; // socket
pub const S_IFLNK: mode_t = 0o120000; // symbolic link
pub const S_IFREG: mode_t = 0o100000; // regular file
pub const S_IFBLK: mode_t = 0o060000; // block device
pub const S_IFDIR: mode_t = 0o040000; // directory
pub const S_IFCHR: mode_t = 0o020000; // character device
pub const S_IFIFO: mode_t = 0o010000; // FIFO

#[no_mangle]
pub unsafe extern "C" fn fstat(fd: c_int, statbuf: *mut stat) -> c_int {
    wrap_syscall!(sys::fstat(fd, statbuf)) as c_int
}

pub(crate) mod sys {
    use super::*;

    use crate::syscall;

    pub(crate) unsafe fn fstat(fd: c_int, statbuf: *mut stat) -> isize {
        syscall!(5, fd as isize, statbuf as isize)
    }
}
