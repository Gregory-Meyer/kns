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

use core::{
    convert::TryInto,
    fmt::{self, Display, Formatter},
};

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub(crate) struct Header {
    e_ident_magic: [u8; IDENT_MAGIC_LEN],
    e_ident_class: IdentClass,
    e_ident_data: IdentData,
    e_ident_version: IdentVersion,
    e_ident_osabi: IdentOSABI,
    e_ident_abiversion: u8,
    e_type: Type,
    e_machine: Machine,
    e_version: Version,
    e_entry: u64,
    e_phoff: u64,
    e_shoff: u64,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

impl Header {
    pub(crate) fn new(program: &[u8]) -> Result<Self, ParseHeaderError> {
        if program.len() < HEADER_SIZE {
            return Err(ParseHeaderError::Incomplete);
        }

        let mut offset = 0;

        let e_ident_magic = {
            let mut magic = [0; IDENT_MAGIC_LEN];
            magic.copy_from_slice(&program[offset..offset + IDENT_MAGIC_LEN]);
            offset += IDENT_MAGIC_LEN;

            magic
        };

        if e_ident_magic != IDENT_MAGIC {
            return Err(ParseHeaderError::IdentMagicInvalid(e_ident_magic));
        }

        let e_ident_class = program[offset];
        offset += 1;

        let e_ident_class = FromPrimitive::from_u8(e_ident_class)
            .ok_or(ParseHeaderError::IdentClassNot64Bit(e_ident_class))?;

        let e_ident_data = program[offset];
        offset += 1;

        let e_ident_data = FromPrimitive::from_u8(e_ident_data)
            .ok_or(ParseHeaderError::IdentDataNotLittleEndian(e_ident_data))?;

        let e_ident_version = program[offset];
        offset += 1;

        let e_ident_version = FromPrimitive::from_u8(e_ident_version)
            .ok_or(ParseHeaderError::IdentVersionNotRecognized(e_ident_version))?;

        let e_ident_osabi = program[offset];
        offset += 1;

        let e_ident_osabi = FromPrimitive::from_u8(e_ident_osabi)
            .ok_or(ParseHeaderError::IdentOSABINotSysVOrLinux(e_ident_osabi))?;

        // no validation to do here...
        let e_ident_abiversion = program[offset];
        // skip 7 padding bytes as well
        offset += 8;

        let e_type = u16::from_ne_bytes(program[offset..offset + 2].try_into().unwrap());
        offset += 2;

        let e_type =
            FromPrimitive::from_u16(e_type).ok_or(ParseHeaderError::TypeNotRecognized(e_type))?;

        let e_machine = u16::from_ne_bytes(program[offset..offset + 2].try_into().unwrap());
        offset += 2;

        let e_machine = FromPrimitive::from_u16(e_machine)
            .ok_or(ParseHeaderError::MachineNotX86_64(e_machine))?;

        let e_version = u32::from_ne_bytes(program[offset..offset + 4].try_into().unwrap());
        offset += 4;

        let e_version = FromPrimitive::from_u32(e_version)
            .ok_or(ParseHeaderError::VersionNotRecognized(e_version))?;

        let e_entry = u64::from_ne_bytes(program[offset..offset + 8].try_into().unwrap());
        offset += 8;

        let e_phoff = u64::from_ne_bytes(program[offset..offset + 8].try_into().unwrap());
        offset += 8;

        let e_shoff = u64::from_ne_bytes(program[offset..offset + 8].try_into().unwrap());
        offset += 8;

        let e_flags = u32::from_ne_bytes(program[offset..offset + 4].try_into().unwrap());
        offset += 4;

        let e_ehsize = u16::from_ne_bytes(program[offset..offset + 2].try_into().unwrap());
        offset += 2;

        if e_ehsize != HEADER_SIZE as u16 {
            return Err(ParseHeaderError::ELFHeaderSizeWrong(e_ehsize));
        }

        let e_phentsize = u16::from_ne_bytes(program[offset..offset + 2].try_into().unwrap());
        offset += 2;

        let e_phnum = u16::from_ne_bytes(program[offset..offset + 2].try_into().unwrap());
        offset += 2;

        let e_shentsize = u16::from_ne_bytes(program[offset..offset + 2].try_into().unwrap());
        offset += 2;

        let e_shnum = u16::from_ne_bytes(program[offset..offset + 2].try_into().unwrap());
        offset += 2;

        let e_shstrndx = u16::from_ne_bytes(program[offset..offset + 2].try_into().unwrap());
        offset += 2;

        assert_eq!(offset, HEADER_SIZE);

        Ok(Self {
            e_ident_magic,
            e_ident_class,
            e_ident_data,
            e_ident_version,
            e_ident_osabi,
            e_ident_abiversion,
            e_type,
            e_machine,
            e_version,
            e_entry,
            e_phoff,
            e_shoff,
            e_flags,
            e_ehsize,
            e_phentsize,
            e_phnum,
            e_shentsize,
            e_shnum,
            e_shstrndx,
        })
    }

    pub(crate) fn program_headers<'a>(
        &'a self,
        program: &'a [u8],
    ) -> impl Iterator<Item = ProgramHeader> + 'a {
        (0..self.e_phnum).map(move |i| self.get_program_header(i, program).unwrap())
    }

    pub(crate) fn get_program_header(&self, idx: u16, program: &[u8]) -> Option<ProgramHeader> {
        if idx > self.e_phnum {
            return None;
        }

        let mut start =
            &program[self.e_phoff as usize + self.e_phentsize as usize * idx as usize..];
        if start.len() < self.e_phentsize as usize {
            return None;
        }

        let p_type = u32::from_ne_bytes(start[..4].try_into().unwrap());
        start = &start[4..];

        let p_flags = u32::from_ne_bytes(start[..4].try_into().unwrap());
        start = &start[4..];

        let p_offset = u64::from_ne_bytes(start[..8].try_into().unwrap());
        start = &start[8..];

        let p_vaddr = u64::from_ne_bytes(start[..8].try_into().unwrap());
        start = &start[8..];

        let p_paddr = u64::from_ne_bytes(start[..8].try_into().unwrap());
        start = &start[8..];

        let p_filesz = u64::from_ne_bytes(start[..8].try_into().unwrap());
        start = &start[8..];

        let p_memsz = u64::from_ne_bytes(start[..8].try_into().unwrap());
        start = &start[8..];

        let p_align = u64::from_ne_bytes(start[..8].try_into().unwrap());

        Some(ProgramHeader {
            p_type,
            p_flags,
            p_offset,
            p_vaddr,
            p_paddr,
            p_filesz,
            p_memsz,
            p_align,
        })
    }
}

pub(crate) struct ProgramHeader {
    pub(crate) p_type: u32,
    pub(crate) p_flags: u32,
    pub(crate) p_offset: u64,
    pub(crate) p_vaddr: u64,
    pub(crate) p_paddr: u64,
    pub(crate) p_filesz: u64,
    pub(crate) p_memsz: u64,
    pub(crate) p_align: u64,
}

pub(crate) const HEADER_SIZE: usize = 64;
pub(crate) const IDENT_MAGIC_LEN: usize = 4;
pub(crate) const IDENT_MAGIC: [u8; IDENT_MAGIC_LEN] = *b"\x7fELF";

#[repr(u8)]
#[derive(FromPrimitive)]
pub(crate) enum IdentClass {
    X64 = 2,
}

#[repr(u8)]
#[derive(FromPrimitive)]
pub(crate) enum IdentData {
    LittleEndian = 1,
}

#[repr(u8)]
#[derive(FromPrimitive)]
pub(crate) enum IdentVersion {
    Current = 1,
}

#[repr(u8)]
#[derive(FromPrimitive)]
pub(crate) enum IdentOSABI {
    SystemV = 0,
    Linux = 3,
}

#[repr(u16)]
#[derive(FromPrimitive)]
pub(crate) enum Type {
    None = 0x00,
    Relocatable = 0x01,
    Executable = 0x02,
    SharedObject = 0x03,
    Core = 0x04,
}

#[repr(u16)]
#[derive(FromPrimitive)]
pub(crate) enum Machine {
    X86_64 = 62,
}

#[repr(u16)]
#[derive(FromPrimitive)]
pub(crate) enum Version {
    Current = 1,
}

#[derive(Debug)]
pub(crate) enum ParseHeaderError {
    Incomplete,
    IdentMagicInvalid([u8; 4]),
    IdentClassNot64Bit(u8),
    IdentDataNotLittleEndian(u8),
    IdentVersionNotRecognized(u8),
    IdentOSABINotSysVOrLinux(u8),
    TypeNotRecognized(u16),
    MachineNotX86_64(u16),
    VersionNotRecognized(u32),
    ELFHeaderSizeWrong(u16),
}

impl Display for ParseHeaderError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Incomplete => f.write_str("incomplete"),
            Self::IdentMagicInvalid(m) => write!(
                f,
                "wrong magic number: expected {:?}, got {:?}",
                IDENT_MAGIC, m
            ),
            Self::IdentClassNot64Bit(c) => write!(f, "unrecognized identification class {}", c),
            Self::IdentDataNotLittleEndian(d) => {
                write!(f, "unrecognized identification data format {}", d)
            }
            Self::IdentVersionNotRecognized(v) => {
                write!(f, "unrecognized identification version {}", v)
            }
            Self::IdentOSABINotSysVOrLinux(a) => write!(f, "unrecognized OS ABI {}", a),
            Self::TypeNotRecognized(t) => write!(f, "unrecognized type {}", t),
            Self::MachineNotX86_64(m) => write!(f, "unrecognized machine {}", m),
            Self::VersionNotRecognized(v) => write!(f, "unrecognized version {}", v),
            Self::ELFHeaderSizeWrong(s) => {
                write!(f, "wrong header size: expected {}, got {}", HEADER_SIZE, s)
            }
        }
    }
}
