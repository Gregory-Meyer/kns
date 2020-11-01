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
#![feature(asm, int_error_matching, lang_items)]
#![allow(non_camel_case_types, non_snake_case)]

use kns_syscall::syscall;

#[macro_use]
pub(crate) mod internal;

pub use core::ffi::c_void;

pub mod errno;
pub mod fcntl;
pub mod linux;
pub mod stddef;
pub mod stdio;
pub mod stdlib;
pub mod string;
pub mod strings;
pub mod sys;
pub mod time;
pub mod unistd;

pub type c_char = i8;
pub type c_short = i16;
pub type c_int = i32;
pub type c_long = i64;
pub type c_longlong = i64;

pub type c_signedchar = i8;
pub type c_unsignedchar = i8;

pub type c_unsignedshort = u16;
pub type c_unsignedint = u32;
pub type c_unsignedlong = u64;
pub type c_unsignedlonglong = u64;
