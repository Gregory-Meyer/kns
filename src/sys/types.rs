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

use crate::{c_int, c_long, c_unsignedint, c_unsignedlong};

pub use crate::{
    stddef::{size_t, ssize_t},
    time::timespec,
};

pub type off_t = c_long;
pub type dev_t = c_unsignedlong;
pub type ino_t = c_unsignedlong;
pub type mode_t = c_unsignedint;
pub type nlink_t = c_unsignedlong;
pub type uid_t = c_unsignedint;
pub type gid_t = c_unsignedint;
pub type blksize_t = c_long;
pub type blkcnt_t = c_long;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct stat {
    pub st_dev: dev_t,
    pub st_ino: ino_t,
    pub st_nlink: nlink_t,

    pub st_mode: mode_t,
    pub st_uid: uid_t,
    pub st_gid: gid_t,
    pad0: c_int,
    pub st_rdev: dev_t,
    pub st_size: off_t,
    pub st_blksize: blksize_t,
    pub st_blocks: blkcnt_t,

    // note -- in the kernel, timespec has unsigned long nsec
    // in userspace/glibc, we have long nsec
    pub st_atim: timespec,
    pub st_mtim: timespec,
    pub st_ctim: timespec,

    unused: [c_long; 3],
}
