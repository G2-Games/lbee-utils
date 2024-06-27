/// The header of a PAK file
#[derive(Debug, Clone)]
pub struct Header {
    /// The starting position of the data within the PAK file
    pub(super) data_offset: u32,

    /// The number of entries within the PAK
    pub(super) entry_count: u32,
    pub(super) id_start: u32,
    pub(super) block_size: u32,

    pub(super) unknown1: u32,
    pub(super) unknown2: u32,
    pub(super) unknown3: u32,
    pub(super) unknown4: u32,

    pub(super) flags: u32,
}

impl Header {
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

    pub fn flags(&self) -> u32 {
        self.flags
    }
}
