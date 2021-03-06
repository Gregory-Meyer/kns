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
pub(crate) mod tcb;

use elf::Header as ELFHeader;

use crate::{
    c_char, c_int, c_unsignedint, c_void,
    internal::errno::ErrorNumber,
    stddef::size_t,
    stdlib,
    sys::{mman, stat, types},
    syscall, unistd,
};

use core::{
    convert::{TryFrom, TryInto},
    fmt::{self, Display, Formatter},
    mem,
    num::NonZeroUsize,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    slice,
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

static mut MAIN_TCB: Option<TCBBox> = None;

pub(crate) unsafe fn finalize() {
    alloc::finalize();
    mem::drop(MAIN_TCB.take().unwrap());
}

#[no_mangle]
pub unsafe extern "C" fn __KNS_start(
    argc: isize,
    argv: *mut *mut c_char,
    envp: *mut *mut c_char,
) -> ! {
    let main_result = {
        const PATH_MAX: usize = 4096;
        let mut self_path_buf = [0u8; PATH_MAX + 1];
        let self_path_len = match ErrorNumber::from_syscall(unistd::sys::readlink(
            (&b"/proc/self/exe\0").as_ptr() as *const c_char,
            (&mut self_path_buf).as_mut_ptr() as *mut c_char,
            PATH_MAX as size_t,
        )) {
            Ok(l) => l,
            Err(e) => panic!("couldn't read path to self: {}", e),
        };
        let self_path = &self_path_buf[..self_path_len];

        let self_fd: FileDescriptor = match ErrorNumber::from_syscall(unistd::sys::open(
            self_path.as_ptr() as *const c_char,
            unistd::O_RDONLY,
            0,
        )) {
            Ok(fd) => fd,
            Err(e) => panic!("couldn't open self: {}", e),
        };

        let mut self_stat = types::stat::default();
        if let Err(e) =
            ErrorNumber::from_syscall::<isize>(stat::sys::fstat(self_fd.as_raw(), &mut self_stat))
        {
            panic!("couldn't fstat self: {}", e);
        };

        let self_len = self_stat.st_size as usize;

        let self_ptr = match ErrorNumber::from_syscall::<isize>(mman::sys::mmap(
            ptr::null_mut(),
            self_len as size_t,
            mman::PROT_READ,
            mman::MAP_SHARED,
            self_fd.as_raw(),
            0,
        )) {
            Ok(p) => p as *mut c_void,
            Err(e) => panic!("couldn't mmap self: {}", e),
        };

        if let Err(e) = self_fd.try_drop() {
            panic!("couldn't close self: {}", e);
        }

        let self_slice = slice::from_raw_parts(self_ptr as *const u8, self_len);
        let self_header = match ELFHeader::new(self_slice) {
            Ok(h) => h,
            Err(e) => panic!("couldn't parse ELF header of self: {}", e),
        };

        if let Some(tls_header) = self_header
            .program_headers(self_slice)
            .find(|h| h.p_type == 7)
        {
            tls::set_template(TLSTemplate {
                initialized: slice::from_raw_parts(
                    tls_header.p_vaddr as *const u8,
                    tls_header.p_filesz as usize,
                ),
                uninitialized_len: (tls_header.p_memsz - tls_header.p_filesz) as usize,
                alignment: NonZeroUsize::new(tls_header.p_align as usize).unwrap(),
            });
        }

        if let Err(e) =
            ErrorNumber::from_syscall::<isize>(mman::sys::munmap(self_ptr, self_len as size_t))
        {
            panic!("couldn't unmap self: {}", e);
        }

        let mut main_tcb = match TCBBox::new() {
            Ok(b) => b,
            Err(e) => panic!("couldn't map thread control block for main thread: {}", e),
        };
        syscall!(
            158,
            0x1002,
            &mut *main_tcb as *mut ThreadControlBlock as isize
        );
        MAIN_TCB = Some(main_tcb);

        alloc::initialize();
        main(argc.try_into().unwrap(), argv, envp)
    };

    stdlib::exit(main_result)
}

#[no_mangle]
pub unsafe extern "C" fn __KNS_errno() -> *mut i32 {
    errno()
}

pub(crate) unsafe fn errno<'a>() -> &'a mut i32 {
    &mut tcb().errno
}

use tls::Template as TLSTemplate;

pub(crate) const fn round_up_to_nearest_multiple(x: usize, multiple: usize) -> usize {
    if x % multiple != 0 {
        x + (multiple - x % multiple)
    } else {
        x
    }
}

mod tls {
    use core::num::NonZeroUsize;

    // DO NOT CHANGE OUTSIDE OF __KNS_start !!!
    static mut TEMPLATE: Template = Template {
        initialized: &[],
        uninitialized_len: 0,
        alignment: unsafe { NonZeroUsize::new_unchecked(1) },
    };

    #[derive(Copy, Clone, Debug)]
    pub(crate) struct Template {
        pub(crate) initialized: &'static [u8],
        pub(crate) uninitialized_len: usize,
        pub(crate) alignment: NonZeroUsize,
    }

    /// # Safety
    ///
    /// This function must be called *exactly* once, from __KNS_start. It must
    /// be called before any TCBs are built.
    pub(crate) unsafe fn set_template(template: Template) {
        TEMPLATE = template;
    }

    fn template() -> Template {
        unsafe { TEMPLATE }
    }

    pub(crate) fn len() -> usize {
        template().initialized.len() + template().uninitialized_len
    }

    pub(crate) fn mmap_len() -> usize {
        super::round_up_to_nearest_multiple(len(), 4096)
    }

    pub(crate) fn initialize(tls: &mut [u8]) {
        assert_eq!(tls.len(), len());

        tls[..template().initialized.len()].copy_from_slice(template().initialized);
        // assume uninitialized memory is already zeroed -- only user is mmap
    }
}

pub(crate) struct ThreadControlBlock {
    _self: NonNull<ThreadControlBlock>,
    pub(crate) errno: c_int,
}

pub(crate) struct TCBBox {
    tcb_ptr: NonNull<ThreadControlBlock>,
}

impl TCBBox {
    pub(crate) fn new() -> Result<Self, ErrorNumber> {
        let ptr = ErrorNumber::from_syscall::<isize>(unsafe {
            mman::sys::mmap(
                ptr::null_mut(),
                Self::mmap_len() as size_t,
                mman::PROT_READ | mman::PROT_WRITE,
                mman::MAP_PRIVATE | mman::MAP_ANONYMOUS,
                -1,
                0,
            )
        })?;

        let tcb_ptr = NonNull::new(unsafe {
            (ptr as *mut u8).add(tls::mmap_len()) as *mut ThreadControlBlock
        })
        .unwrap();

        unsafe {
            ptr::write(
                tcb_ptr.as_ptr(),
                ThreadControlBlock {
                    _self: tcb_ptr,
                    errno: 0,
                },
            )
        };

        let this_tls = unsafe {
            slice::from_raw_parts_mut((tcb_ptr.as_ptr() as *mut u8).sub(tls::len()), tls::len())
        };
        tls::initialize(this_tls);

        Ok(Self { tcb_ptr })
    }

    fn mmap_len() -> usize {
        tls::mmap_len() + mem::size_of::<ThreadControlBlock>()
    }
}

impl Drop for TCBBox {
    fn drop(&mut self) {
        unsafe {
            mman::sys::munmap(
                (self.tcb_ptr.as_ptr() as *mut u8).sub(tls::mmap_len()) as *mut c_void,
                Self::mmap_len() as size_t,
            )
        };
    }
}

impl Deref for TCBBox {
    type Target = ThreadControlBlock;

    fn deref(&self) -> &ThreadControlBlock {
        unsafe { self.tcb_ptr.as_ref() }
    }
}

impl DerefMut for TCBBox {
    fn deref_mut(&mut self) -> &mut ThreadControlBlock {
        unsafe { self.tcb_ptr.as_mut() }
    }
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
