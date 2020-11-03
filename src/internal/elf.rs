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

use core::convert::TryInto;

pub(crate) struct ELFHeader {
    e_ident_magic: [u8; IDENT_MAGIC_LEN],
    e_ident_class: u8,
    e_ident_data: u8,
    e_ident_version: u8,
    e_ident_osabi: u8,
    e_ident_abiversion: u8,
    e_type: u16,
    e_machine: u16,
    e_version: u32,
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

impl ELFHeader {
    pub fn new(program: &[u8]) -> Result<Self, ParseELFHeaderError> {
        if program.len() < HEADER_SIZE {
            return Err(ParseELFHeaderError::Incomplete);
        }

        let mut offset = 0;

        let e_ident_magic = {
            let mut magic = [0; IDENT_MAGIC_LEN];
            magic.copy_from_slice(&program[offset..offset + IDENT_MAGIC_LEN]);
            offset += IDENT_MAGIC_LEN;

            magic
        };

        if e_ident_magic != IDENT_MAGIC {
            return Err(ParseELFHeaderError::IdentMagicInvalid(e_ident_magic));
        }

        let e_ident_class = program[offset];
        offset += 1;

        if e_ident_class != IdentClass::X64 as u8 {
            return Err(ParseELFHeaderError::IdentClassNot64Bit(e_ident_class));
        }

        let e_ident_data = program[offset];
        offset += 1;

        if e_ident_data != IdentData::LittleEndian as u8 {
            return Err(ParseELFHeaderError::IdentDataNotLittleEndian(e_ident_data));
        }

        let e_ident_version = program[offset];
        offset += 1;

        if e_ident_version != IdentVersion::Current as u8 {
            return Err(ParseELFHeaderError::IdentVersionNotRecognized(
                e_ident_version,
            ));
        }

        let e_ident_osabi = program[offset];
        offset += 1;

        if e_ident_osabi != IdentOSABI::SystemV as u8 && e_ident_osabi != IdentOSABI::Linux as u8 {
            return Err(ParseELFHeaderError::IdentOSABINotSysVOrLinux(e_ident_osabi));
        }

        // no validation to do here...
        let e_ident_abiversion = program[offset];
        // skip 7 padding bytes as well
        offset += 8;

        let e_type = u16::from_ne_bytes(program[offset..offset + 2].try_into().unwrap());
        offset += 2;

        if e_type != Type::None as u16
            && e_type != Type::Relocatable as u16
            && e_type != Type::Executable as u16
            && e_type != Type::SharedObject as u16
            && e_type != Type::Core as u16
        {
            return Err(ParseELFHeaderError::TypeNotRecognized(e_type));
        }

        let e_machine = u16::from_ne_bytes(program[offset..offset + 2].try_into().unwrap());
        offset += 2;

        if e_machine != Machine::X86_64 as u16 {
            return Err(ParseELFHeaderError::MachineNotX86_64(e_machine));
        }

        let e_version = u32::from_ne_bytes(program[offset..offset + 4].try_into().unwrap());
        offset += 4;

        if e_version != Version::Current as u32 {
            return Err(ParseELFHeaderError::VersionNotRecognized(e_version));
        }

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
            return Err(ParseELFHeaderError::ELFHeaderSizeWrong(e_ehsize));
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
}

pub(crate) const HEADER_SIZE: usize = 64;
pub(crate) const IDENT_MAGIC_LEN: usize = 4;
pub(crate) const IDENT_MAGIC: [u8; IDENT_MAGIC_LEN] = *b"\x7fELF";

#[repr(u8)]
pub(crate) enum IdentClass {
    X64 = 2,
}

#[repr(u8)]
pub(crate) enum IdentData {
    LittleEndian = 1,
}

#[repr(u8)]
pub(crate) enum IdentVersion {
    Current = 1,
}

#[repr(u8)]
pub(crate) enum IdentOSABI {
    SystemV = 0,
    Linux = 3,
}

#[repr(u16)]
pub(crate) enum Type {
    None = 0x00,
    Relocatable = 0x01,
    Executable = 0x02,
    SharedObject = 0x03,
    Core = 0x04,
}

#[repr(u16)]
pub(crate) enum Machine {
    X86_64 = 62,
}

#[repr(u16)]
pub(crate) enum Version {
    Current = 1,
}

pub(crate) enum ParseELFHeaderError {
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
