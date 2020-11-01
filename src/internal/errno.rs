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

use crate::{c_int, errno};

use core::{
    convert::TryFrom,
    fmt::{self, Debug, Display, Formatter},
};

use kns_syscall::SyscallResult;

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErrorNumber {
    Badf,
    Nomem,
    Inval,
    Range,
    Nosys,
}

impl ErrorNumber {
    pub(crate) fn from_syscall<T: TryFrom<isize>>(result: isize) -> Result<T, ErrorNumber>
    where
        <T as TryFrom<isize>>::Error: Debug,
    {
        result.into_value().map_err(|e| match e {
            errno::EBADF => ErrorNumber::Badf,
            errno::ENOMEM => ErrorNumber::Nomem,
            errno::EINVAL => ErrorNumber::Inval,
            errno::ERANGE => ErrorNumber::Range,
            errno::ENOSYS => ErrorNumber::Nosys,
            _ => unimplemented!("unrecognized error number {}", e),
        })
    }

    pub(crate) fn into_int(self) -> c_int {
        match self {
            ErrorNumber::Badf => errno::EBADF,
            ErrorNumber::Nomem => errno::ENOMEM,
            ErrorNumber::Inval => errno::EINVAL,
            ErrorNumber::Range => errno::ERANGE,
            ErrorNumber::Nosys => errno::ENOSYS,
        }
    }

    pub(crate) fn into_str(self) -> &'static str {
        match self {
            ErrorNumber::Badf => "EBADF",
            ErrorNumber::Nomem => "ENOMEM",
            ErrorNumber::Inval => "EINVAL",
            ErrorNumber::Range => "ERANGE",
            ErrorNumber::Nosys => "ENOSYS",
        }
    }

    pub(crate) fn description(self) -> &'static str {
        match self {
            ErrorNumber::Badf => "Bad file descriptor",
            ErrorNumber::Nomem => "Cannot allocate memory",
            ErrorNumber::Inval => "Invalid argument",
            ErrorNumber::Range => "Numerical result out of range",
            ErrorNumber::Nosys => "Function not implemented",
        }
    }
}

impl Display for ErrorNumber {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} ({})", self.description(), self.into_str())
    }
}
