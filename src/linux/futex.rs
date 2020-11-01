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

use crate::{c_int, syscall, time::timespec};

pub(crate) mod sys {
    use super::*;

    const SYS_FUTEX: isize = 202;
    const FUTEX_WAIT: c_int = 0;
    const FUTEX_WAKE: c_int = 1;
    const FUTEX_PRIVATE_FLAG: c_int = 128;

    pub(crate) unsafe fn futex_wait(
        uaddr: *mut c_int,
        val: c_int,
        timeout: *const timespec,
    ) -> isize {
        syscall!(
            SYS_FUTEX,
            uaddr as isize,
            FUTEX_WAIT as isize,
            val as isize,
            timeout as isize
        )
    }

    pub(crate) unsafe fn futex_wait_private(
        uaddr: *mut c_int,
        val: c_int,
        timeout: *const timespec,
    ) -> isize {
        syscall!(
            SYS_FUTEX,
            uaddr as isize,
            (FUTEX_WAIT | FUTEX_PRIVATE_FLAG) as isize,
            val as isize,
            timeout as isize
        )
    }

    pub(crate) unsafe fn futex_wake(uaddr: *mut c_int, val: c_int) -> isize {
        syscall!(SYS_FUTEX, uaddr as isize, FUTEX_WAKE as isize, val as isize)
    }

    pub(crate) unsafe fn futex_wake_private(uaddr: *mut c_int, val: c_int) -> isize {
        syscall!(
            SYS_FUTEX,
            uaddr as isize,
            (FUTEX_WAKE | FUTEX_PRIVATE_FLAG) as isize,
            val as isize
        )
    }
}
