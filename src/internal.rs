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

pub(crate) mod alloc;
pub(crate) mod elf;
pub(crate) mod errno;
pub(crate) mod sync;

use crate::{
    c_char, c_int, c_unsignedint, c_void, internal::errno::ErrorNumber, stddef::size_t, stdlib,
    syscall, unistd,
};

use core::{
    convert::{TryFrom, TryInto},
    fmt::{self, Display, Formatter},
    mem, ptr,
};

#[macro_export]
macro_rules! wrap_syscall {
    ($rax:expr) => {{
        let rax: isize = $rax;

        if rax >= -4095 && rax <= -1 {
            *$crate::internal::errno() = -rax as i32;

            -1
        } else {
            rax
        }
    }};
}

#[derive(Debug, Hash)]
pub(crate) struct FileDescriptor {
    fd: c_unsignedint,
}

impl FileDescriptor {
    pub(crate) fn as_raw(&self) -> c_int {
        self.fd.try_into().unwrap()
    }

    pub(crate) fn try_drop(self) -> Result<(), ErrorNumber> {
        let fd = self.as_raw();
        mem::forget(self);

        ErrorNumber::from_syscall(unsafe { unistd::sys::close(fd) }).map(|_: isize| ())
    }
}

impl TryFrom<isize> for FileDescriptor {
    type Error = InvalidFileDescriptor;

    fn try_from(value: isize) -> Result<FileDescriptor, InvalidFileDescriptor> {
        if let Ok(fd) = value.try_into() {
            Ok(FileDescriptor { fd })
        } else {
            Err(InvalidFileDescriptor(value))
        }
    }
}

impl TryFrom<c_int> for FileDescriptor {
    type Error = InvalidFileDescriptor;

    fn try_from(value: i32) -> Result<FileDescriptor, InvalidFileDescriptor> {
        if let Ok(fd) = value.try_into() {
            Ok(FileDescriptor { fd })
        } else {
            Err(InvalidFileDescriptor(value as isize))
        }
    }
}

impl Into<c_int> for FileDescriptor {
    fn into(self) -> c_int {
        self.fd.try_into().unwrap()
    }
}

impl Drop for FileDescriptor {
    fn drop(&mut self) {
        unsafe { unistd::sys::close(self.as_raw()) };
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct InvalidFileDescriptor(isize);

impl Display for InvalidFileDescriptor {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "invalid file descriptor: {} is not representable by int",
            self.0
        )
    }
}

pub(crate) fn write_all(fd: &FileDescriptor, mut data: &[u8]) -> Result<(), ErrorNumber> {
    while !data.is_empty() {
        let this_written = ErrorNumber::from_syscall(unsafe {
            unistd::sys::write(
                fd.as_raw(),
                data.as_ptr() as *mut c_void,
                data.len() as size_t,
            )
        })?;

        data = &data[this_written..];
    }

    Ok(())
}

extern "C" {
    fn main(argc: c_int, argv: *mut *mut c_char, envp: *mut *mut c_char) -> c_int;
}

#[link(name = "kns-rpmalloc", kind = "static")]
extern "C" {
    pub fn rpmalloc_initialize() -> c_int;
    pub fn rpmalloc_finalize() -> c_int;
}

#[no_mangle]
pub unsafe extern "C" fn __KNS_start(
    argc: isize,
    argv: *mut *mut c_char,
    envp: *mut *mut c_char,
) -> ! {
    let mut main_tcb = ThreadControlBlock {
        this: ptr::null_mut(),
        errno: 0,
    };
    main_tcb.this = &mut main_tcb;
    syscall!(158, 0x1002, &mut main_tcb as *mut _ as isize);

    assert_eq!(rpmalloc_initialize(), 0, "couldn't initialize rpmalloc");

    stdlib::exit(main(argc.try_into().unwrap(), argv, envp))
}

#[no_mangle]
pub unsafe extern "C" fn __KNS_errno() -> *mut i32 {
    errno()
}

pub(crate) unsafe fn errno<'a>() -> &'a mut i32 {
    &mut tcb().errno
}

pub(crate) struct ThreadControlBlock {
    this: *mut ThreadControlBlock,
    errno: c_int,
}

pub(crate) unsafe fn tcb<'a>() -> &'a mut ThreadControlBlock {
    let ptr: *mut ThreadControlBlock;

    asm!(
        "mov {}, fs:0",
        lateout(reg) ptr,
    );

    &mut *ptr
}

#[cfg(not(test))]
mod handler {
    use super::*;

    use core::{
        fmt::{self, Write},
        panic::PanicInfo,
    };

    #[panic_handler]
    fn panic(info: &PanicInfo) -> ! {
        struct StdErr;

        impl Write for StdErr {
            fn write_str(&mut self, mut s: &str) -> fmt::Result {
                while !s.is_empty() {
                    let write_res = ErrorNumber::from_syscall(unsafe {
                        unistd::sys::write(
                            unistd::STDERR_FILENO,
                            s.as_ptr() as *const c_void,
                            s.len() as size_t,
                        )
                    })
                    .map_err(|_| fmt::Error)?;

                    s = &s[write_res..];
                }

                Ok(())
            }
        }

        writeln!(&mut StdErr, "{}", info).ok();

        unsafe {
            stdlib::exit(130);
        }
    }
}
