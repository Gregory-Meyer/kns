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

#![no_std]
#![feature(asm)]

use core::{convert::TryFrom, fmt::Debug};

#[macro_export]
macro_rules! syscall {
    ($rax:expr) => {
        $crate::syscall0($rax)
    };
    ($rax:expr, $rdi:expr) => {
        $crate::syscall1($rax, $rdi)
    };
    ($rax:expr, $rdi:expr, $rsi:expr) => {
        $crate::syscall2($rax, $rdi, $rsi)
    };
    ($rax:expr, $rdi:expr, $rsi:expr, $rdx:expr) => {
        $crate::syscall3($rax, $rdi, $rsi, $rdx)
    };
    ($rax:expr, $rdi:expr, $rsi:expr, $rdx:expr, $r10:expr) => {
        $crate::syscall4($rax, $rdi, $rsi, $rdx, $r10)
    };
    ($rax:expr, $rdi:expr, $rsi:expr, $rdx:expr, $r10:expr, $r8:expr) => {
        $crate::syscall5($rax, $rdi, $rsi, $rdx, $r8, $r10)
    };
    ($rax:expr, $rdi:expr, $rsi:expr, $rdx:expr, $r10:expr, $r8:expr, $r9:expr) => {
        $crate::syscall6($rax, $rdi, $rsi, $rdx, $r10, $r8, $r9)
    };
}

pub trait SyscallResult: Sized {
    fn is_error(self) -> bool;
    fn into_value<T: TryFrom<Self>>(self) -> Result<T, i32>
    where
        <T as TryFrom<Self>>::Error: Debug;
}

pub use arch::*;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
pub(crate) mod arch {
    use super::SyscallResult;

    use core::{
        convert::{TryFrom, TryInto},
        fmt::Debug,
    };

    impl SyscallResult for isize {
        fn is_error(self) -> bool {
            self >= -4095 && self <= -1
        }

        fn into_value<T: TryFrom<Self>>(self) -> Result<T, i32>
        where
            <T as TryFrom<Self>>::Error: Debug,
        {
            if !self.is_error() {
                Ok(self.try_into().unwrap())
            } else {
                Err(-self as i32)
            }
        }
    }

    pub unsafe fn syscall0(mut rax: isize) -> isize {
        asm!(
            "syscall",
            inlateout("rax") rax,
            lateout("rcx") _,
            lateout("r11") _,
        );

        rax
    }

    pub unsafe fn syscall1(mut rax: isize, rdi: isize) -> isize {
        asm!(
            "syscall",
            inlateout("rax") rax,
            in("rdi") rdi,
            lateout("rcx") _,
            lateout("r11") _,
        );

        rax
    }

    pub unsafe fn syscall2(mut rax: isize, rdi: isize, rsi: isize) -> isize {
        asm!(
            "syscall",
            inlateout("rax") rax,
            in("rdi") rdi,
            in("rsi") rsi,
            lateout("rcx") _,
            lateout("r11") _,
        );

        rax
    }

    pub unsafe fn syscall3(mut rax: isize, rdi: isize, rsi: isize, rdx: isize) -> isize {
        asm!(
            "syscall",
            inlateout("rax") rax,
            in("rdi") rdi,
            in("rsi") rsi,
            in("rdx") rdx,
            lateout("rcx") _,
            lateout("r11") _,
        );

        rax
    }

    pub unsafe fn syscall4(
        mut rax: isize,
        rdi: isize,
        rsi: isize,
        rdx: isize,
        r10: isize,
    ) -> isize {
        asm!(
            "syscall",
            inlateout("rax") rax,
            in("rdi") rdi,
            in("rsi") rsi,
            in("rdx") rdx,
            in("r10") r10,
            lateout("rcx") _,
            lateout("r11") _,
        );

        rax
    }

    pub unsafe fn syscall5(
        mut rax: isize,
        rdi: isize,
        rsi: isize,
        rdx: isize,
        r10: isize,
        r8: isize,
    ) -> isize {
        asm!(
            "syscall",
            inlateout("rax") rax,
            in("rdi") rdi,
            in("rsi") rsi,
            in("rdx") rdx,
            in("r10") r10,
            in("r8") r8,
            lateout("rcx") _,
            lateout("r11") _,
        );

        rax
    }

    pub unsafe fn syscall6(
        mut rax: isize,
        rdi: isize,
        rsi: isize,
        rdx: isize,
        r10: isize,
        r8: isize,
        r9: isize,
    ) -> isize {
        asm!(
            "syscall",
            inlateout("rax") rax,
            in("rdi") rdi,
            in("rsi") rsi,
            in("rdx") rdx,
            in("r10") r10,
            in("r8") r8,
            in("r9") r9,
            lateout("rcx") _,
            lateout("r11") _,
        );

        rax
    }
}
