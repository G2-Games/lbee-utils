use byteorder_lite::WriteBytesExt;
use std::io::{self, Write};

use crate::LE;

/// The header of a PAK file
#[derive(Debug, Clone)]
pub struct Header {
    /// The starting position of the data within the PAK file
    pub(super) data_offset: u32,

    /// The number of entries within the PAK
    pub(super) entry_count: u32,
    pub(super) id_start: u32,
    pub(super) block_size: u32,

    /// The offset of the subdirectory name within the PAK
    pub(super) subdir_offset: u32,
    pub(super) unknown2: u32,
    pub(super) unknown3: u32,
    pub(super) unknown4: u32,

    pub(super) flags: PakFlags,
}

impl Header {
    pub fn write_into<T: Write>(&self, output: &mut T) -> Result<(), io::Error> {
        output.write_u32::<LE>(self.data_offset)?;
        output.write_u32::<LE>(self.entry_count)?;
        output.write_u32::<LE>(self.id_start)?;
        output.write_u32::<LE>(self.block_size)?;
        output.write_u32::<LE>(self.subdir_offset)?;
        output.write_u32::<LE>(self.unknown2)?;
        output.write_u32::<LE>(self.unknown3)?;
        output.write_u32::<LE>(self.unknown4)?;
        output.write_u32::<LE>(self.flags.0)?;

        Ok(())
    }

    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    pub fn id_start(&self) -> u32 {
        self.id_start
    }

    pub fn entry_count(&self) -> u32 {
        self.entry_count
    }

    pub fn data_offset(&self) -> u32 {
        self.data_offset
    }

    pub fn flags(&self) -> &PakFlags {
        &self.flags
    }
}

/// Flags which define different features in a PAK file
#[derive(Clone, Debug)]
pub struct PakFlags(pub u32);

impl PakFlags {
    pub fn has_unknown_data1(&self) -> bool {
        // 0b00100000000
        self.0 & 0b00100000000 != 0
    }

    pub fn has_names(&self) -> bool {
        // 0b01000000000
        self.0 & 0b01000000000 != 0
    }

    pub fn extra_pre_count(&self) -> usize {
        match self.0 as usize & 0b111 {
            0 => 1,
            1 => 2,
            2 => 4,
            3 => 5,
            4 => 7,
            _ => 0,
        }
    }
}
