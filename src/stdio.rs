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
    c_char, c_int, c_void, errno,
    internal::{
        self,
        alloc::Box,
        errno::ErrorNumber,
        sync::{Mutex, Once},
        FileDescriptor,
    },
    stddef::size_t,
    sys::{stat, types::stat as Stat},
    unistd,
};

use core::{convert::TryInto, mem::MaybeUninit, ptr, slice};

pub struct FILE {
    inner: Mutex<FileInner>,
}

impl FILE {
    pub(crate) fn from_parts(
        fd: FileDescriptor,
        is_readable: bool,
        is_writable: bool,
        buffering: BufferingKind,
    ) -> Result<Box<FILE>, ErrorNumber> {
        let read = if is_readable {
            Some(Buffer::try_new(DEFAULT_BUFLEN).map_err(|_| ErrorNumber::Nomem)?)
        } else {
            None
        };

        let write = if is_writable {
            Some(Buffer::try_new(DEFAULT_BUFLEN).map_err(|_| ErrorNumber::Nomem)?)
        } else {
            None
        };

        let file = FILE {
            inner: Mutex::new(FileInner {
                fd,
                read,
                write,
                buffering,
                is_eof: false,
                is_error: false,
            }),
        };

        Box::new(file).map_err(|_| ErrorNumber::Nomem)
    }
}

pub const EOF: c_int = -1;

const DEFAULT_BUFLEN: usize = 64 * (1 << 10); // 64 KiB; it's big I know!

pub const STDIN_FILENO: c_int = 0;
pub const STDOUT_FILENO: c_int = 1;
pub const STDERR_FILENO: c_int = 2;

pub(crate) static mut STDIN: *mut FILE = ptr::null_mut();
pub(crate) static mut STDOUT: *mut FILE = ptr::null_mut();
pub(crate) static mut STDERR: *mut FILE = ptr::null_mut();

#[no_mangle]
pub unsafe extern "C" fn __KNS_stdin() -> *mut FILE {
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        let stdin = FILE::from_parts(
            STDIN_FILENO.try_into().unwrap(),
            true,
            false,
            BufferingKind::None,
        )
        .expect("couldn't create FILE * for stdin");

        STDIN = Box::into_raw(stdin);
    });

    STDIN.as_mut().unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn __KNS_stdout() -> *mut FILE {
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        let mut statbuf = Stat::default();

        let buffering =
            if ErrorNumber::from_syscall::<isize>(stat::sys::fstat(STDOUT_FILENO, &mut statbuf))
                .is_ok()
            {
                if (statbuf.st_mode & stat::S_IFMT) == stat::S_IFCHR {
                    let buf = [0u8; b"/dev/null".len()];

                    if let Ok(buflen) = ErrorNumber::from_syscall(unistd::sys::readlink(
                        b"/proc/self/fd/1\0".as_ptr() as *const c_char,
                        buf.as_ptr() as *mut c_char,
                        buf.len() as size_t,
                    )) {
                        let linkdir = &buf[..buflen];

                        if linkdir == b"/dev/null" {
                            BufferingKind::Full
                        } else {
                            BufferingKind::Line
                        }
                    } else {
                        BufferingKind::Line
                    }
                } else {
                    BufferingKind::Full
                }
            } else {
                BufferingKind::Line
            };

        let stdout = FILE::from_parts(STDOUT_FILENO.try_into().unwrap(), false, true, buffering)
            .expect("couldn't create FILE * for stdout");

        STDOUT = Box::into_raw(stdout);
    });

    STDOUT.as_mut().unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn __KNS_stderr() -> *mut FILE {
    static ONCE: Once = Once::new();

    ONCE.call_once(|| {
        let stderr = FILE::from_parts(
            STDERR_FILENO.try_into().unwrap(),
            false,
            true,
            BufferingKind::None,
        )
        .expect("couldn't create FILE * for stderr");

        STDERR = Box::into_raw(stderr);
    });

    STDERR.as_mut().unwrap()
}

#[no_mangle]
pub unsafe extern "C" fn fopen(pathname: *const c_char, mode: *const c_char) -> *mut FILE {
    if *mode.offset(0) == 0
        || (*mode.offset(1) != 0 && *mode.offset(2) != 0 && *mode.offset(3) != 0)
    {
        *internal::errno() = errno::EINVAL;

        return ptr::null_mut();
    }

    let modelen = if *mode.offset(1) == 0 {
        1
    } else if *mode.offset(2) == 0 {
        2
    } else if *mode.offset(3) == 0 {
        3
    } else {
        unreachable!()
    };

    let mode_slice = slice::from_raw_parts(mode as *const u8, modelen);

    let (flags, is_readable, is_writable) = match mode_slice {
        b"r" | b"rb" => (unistd::O_RDONLY | unistd::O_CLOEXEC, true, false),
        b"r+" | b"r+b" | b"rb+" => (unistd::O_RDWR | unistd::O_CLOEXEC, true, true),
        b"w" | b"wb" => (
            unistd::O_WRONLY | unistd::O_CREAT | unistd::O_TRUNC | unistd::O_CLOEXEC,
            false,
            true,
        ),
        b"w+" | b"w+b" | b"wb+" => (
            unistd::O_RDWR | unistd::O_CREAT | unistd::O_TRUNC | unistd::O_CLOEXEC,
            true,
            true,
        ),
        b"a" | b"ab" => (
            unistd::O_WRONLY | unistd::O_CREAT | unistd::O_APPEND | unistd::O_CLOEXEC,
            false,
            true,
        ),
        b"a+" | b"a+b" | b"ab+" => (
            unistd::O_RDWR | unistd::O_CREAT | unistd::O_APPEND | unistd::O_CLOEXEC,
            true,
            true,
        ),
        _ => {
            *internal::errno() = errno::EINVAL;

            return ptr::null_mut();
        }
    };

    let fd: FileDescriptor = match ErrorNumber::from_syscall(unistd::sys::open(
        pathname,
        flags,
        unistd::S_IRUSR
            | unistd::S_IWUSR
            | unistd::S_IRGRP
            | unistd::S_IWGRP
            | unistd::S_IROTH
            | unistd::S_IWOTH,
    )) {
        Ok(fd) => fd,
        Err(e) => {
            *internal::errno() = e.into_int();

            return ptr::null_mut();
        }
    };

    if let Ok(file_ptr) = FILE::from_parts(fd, is_readable, is_writable, BufferingKind::Full) {
        Box::into_raw(file_ptr)
    } else {
        *internal::errno() = errno::ENOMEM;

        ptr::null_mut()
    }
}

#[no_mangle]
pub unsafe extern "C" fn fclose(stream: *mut FILE) -> c_int {
    if stream.is_null() {
        *internal::errno() = errno::EINVAL;

        return EOF;
    }

    // lots of inners here -- go from *mut FILE to Box<FILE> to FILE to FileInner
    let FileInner { fd, write, .. } = Box::into_inner(Box::from_raw(stream)).inner.into_inner();

    if let Some(write) = write {
        if let Err(e) = internal::write_all(&fd, write.get_slice()) {
            *internal::errno() = e.into_int();

            return EOF;
        }
    }

    if let Err(e) = fd.try_drop() {
        *internal::errno() = e.into_int();

        EOF
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "C" fn fgets(s: *mut c_char, size: c_int, stream: *mut FILE) -> *mut c_char {
    if s.is_null() || size < 1 || stream.is_null() {
        *internal::errno() = errno::EINVAL;

        return ptr::null_mut();
    }

    let stream = &*stream; // immutable ref -- we need to lock the mutex!
    let mut out_slice = slice::from_raw_parts_mut(s as *mut u8, size as usize);

    let mut guard = stream.inner.lock();

    let FileInner {
        fd,
        read,
        is_eof,
        is_error,
        ..
    } = &mut *guard;

    let read = if let Some(r) = read {
        r
    } else {
        *internal::errno() = errno::EBADF;

        return ptr::null_mut();
    };

    if *is_eof {
        return ptr::null_mut();
    }

    while out_slice.len() > 1 {
        let mut get_slice = read.get_slice();

        if get_slice.is_empty() {
            let put_slice = read.put_slice();

            let read_len = unistd::read(
                fd.as_raw(),
                put_slice.as_ptr() as *mut c_void,
                put_slice.len() as size_t,
            );

            if read_len == -1 {
                *is_error = true;

                return ptr::null_mut();
            }

            if read_len == 0 {
                *is_eof = true;

                break;
            }

            read.put(read_len as usize);
            get_slice = read.get_slice();
        }

        let (copylen, will_copy_newline) =
            if let Some(newline_pos) = get_slice.iter().copied().position(|ch| ch == b'\n') {
                if newline_pos + 1 >= out_slice.len() {
                    (out_slice.len() - 1, false)
                } else {
                    (newline_pos + 1, true)
                }
            } else if get_slice.len() >= out_slice.len() {
                // no newline found, but we don't have enough space to get the entire line if we did
                // find one
                (out_slice.len() - 1, false)
            } else {
                (get_slice.len(), false)
            };

        assert!(out_slice.len() > copylen);
        assert!(get_slice.len() >= copylen);

        out_slice[..copylen].copy_from_slice(&get_slice[..copylen]);
        read.get(copylen);

        out_slice = &mut out_slice[copylen..];

        if will_copy_newline {
            break;
        }
    }

    out_slice[0] = b'\0';

    s
}

#[no_mangle]
pub unsafe extern "C" fn fputs(s: *const c_char, stream: *mut FILE) -> c_int {
    if s.is_null() || stream.is_null() {
        *internal::errno() = errno::EINVAL;

        return EOF;
    }

    let stream = &*stream; // immutable ref -- we need to lock the mutex!
    let mut as_slice = slice::from_raw_parts(s as *const u8, usize::MAX);
    let mut has_newline = false;
    let mut num_written = 0;

    let mut guard = stream.inner.lock();

    let FileInner {
        fd,
        write,
        buffering,
        ..
    } = &mut *guard;
    let buffering = *buffering;

    let write = if let Some(w) = write {
        w
    } else {
        *internal::errno() = errno::EBADF;

        return EOF;
    };

    loop {
        let put_slice = write.put_slice();

        let (copy_len, is_done) = if let Some(null_or_line_ending_idx) = as_slice
            .iter()
            .take(put_slice.len())
            .copied()
            .position(|ch| ch == b'\0' || ch == b'\n')
        {
            if as_slice[null_or_line_ending_idx] == b'\0' {
                (null_or_line_ending_idx, true)
            } else {
                has_newline = true;

                (null_or_line_ending_idx + 1, false)
            }
        } else {
            (put_slice.len(), false)
        };

        assert!(copy_len <= put_slice.len());

        put_slice[..copy_len].copy_from_slice(&as_slice[..copy_len]);
        let put_len = put_slice.len();
        write.put(copy_len);
        num_written += copy_len;
        as_slice = &as_slice[copy_len..];

        if copy_len == put_len {
            let flush_result = write.flush(&fd);
            has_newline = false;

            if let Err(e) = flush_result {
                *internal::errno() = e.into_int();

                return EOF;
            }
        }

        if is_done {
            break;
        }
    }

    if (has_newline && buffering == BufferingKind::Line) || buffering == BufferingKind::None {
        if let Err(e) = write.flush(&fd) {
            *internal::errno() = e.into_int();

            return EOF;
        }
    }

    if let Ok(int_size_result) = num_written.try_into() {
        int_size_result
    } else {
        *internal::errno() = errno::ERANGE;

        c_int::MAX
    }
}

struct Buffer {
    data: Box<[MaybeUninit<u8>]>,
    get_pos: usize,
    put_pos: usize,
}

impl Buffer {
    fn try_new(len: usize) -> Result<Self, ()> {
        Ok(Self {
            data: Box::new_uninit_slice(len)?,
            get_pos: 0,
            put_pos: 0,
        })
    }

    fn get_slice(&self) -> &[u8] {
        let ptr = (&*self.data).as_ptr() as *const u8;

        unsafe { slice::from_raw_parts(ptr.add(self.get_pos), self.put_pos - self.get_pos) }
    }

    fn put_slice(&mut self) -> &mut [u8] {
        let ptr = (&mut *self.data).as_mut_ptr() as *mut u8;

        unsafe { slice::from_raw_parts_mut(ptr.add(self.put_pos), self.data.len() - self.put_pos) }
    }

    fn get(&mut self, n: usize) {
        assert!(n <= self.put_pos - self.get_pos);
        self.get_pos += n;

        if self.get_pos == self.put_pos {
            self.get_pos = 0;
            self.put_pos = 0;
        }
    }

    fn put(&mut self, n: usize) {
        assert!(n <= self.data.len() - self.put_pos);

        self.put_pos += n;
    }

    fn flush(&mut self, fd: &FileDescriptor) -> Result<(), ErrorNumber> {
        let get_slice_len = self.get_slice().len();
        let flush_result = internal::write_all(fd, self.get_slice());
        self.get(get_slice_len);

        flush_result
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub(crate) enum BufferingKind {
    None,
    Line,
    Full,
}

struct FileInner {
    fd: FileDescriptor,
    read: Option<Buffer>,
    write: Option<Buffer>,
    buffering: BufferingKind,
    is_eof: bool,
    is_error: bool,
}
