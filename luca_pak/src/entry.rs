use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

/// A single file entry in a PAK file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub(super) index: usize,

    /// The location within the PAK file, this number is multiplied by the
    /// block size
    pub(super) offset: u32,

    /// The size of the entry in bytes
    pub(super) length: u32,

    /// ???
    pub(super) unknown1: Option<[u8; 12]>,

    /// The name of the entry as stored in the PAK
    pub(super) name: Option<String>,

    /// The ID of the entry, effectively an index
    pub(super) id: u32,

    /// The actual data which makes up the entry
    pub(super) data: Vec<u8>,
}

impl Entry {
    /// Get the name of the [`Entry`]
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    /// Save an [`Entry`] as its underlying data to a file
    pub fn save<P: ?Sized + AsRef<Path>>(&self, path: &P) -> Result<(), Box<dyn Error>> {
        let mut out_file = BufWriter::new(File::create(path)?);

        out_file.write_all(&self.data)?;
        out_file.flush()?;

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.length as usize
    }

    /// Get the raw byte data of an [`Entry`]
    pub fn as_bytes(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn display_name(&self) -> String {
        let mut name = self.name().clone().unwrap_or(self.id().to_string());
        let entry_type = self.file_type();
        name.push_str(&entry_type.extension());

        name
    }

    pub fn file_type(&self) -> EntryType {
        if self.data[0..2] == [b'C', b'Z'] {
            match self.data[2] {
                b'0' => EntryType::CZ0,
                b'1' => EntryType::CZ1,
                b'2' => EntryType::CZ2,
                b'3' => EntryType::CZ3,
                b'4' => EntryType::CZ4,
                b'5' => EntryType::CZ5,
                _ => EntryType::Unknown,
            }
        } else if self.data[0..3] == [b'M', b'V', b'T'] {
            EntryType::MVT
        } else {
            EntryType::Unknown
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    CZ0,
    CZ1,
    CZ2,
    CZ3,
    CZ4,
    CZ5,

    /// An MVT video file
    MVT,

    /// Who knows!
    Unknown,
}

impl EntryType {
    /// Get the file extension for the file
    pub fn extension(&self) -> &'static str {
        match self {
            Self::CZ0 => ".cz0",
            Self::CZ1 => ".cz1",
            Self::CZ2 => ".cz2",
            Self::CZ3 => ".cz3",
            Self::CZ4 => ".cz4",
            Self::CZ5 => ".cz5",
            Self::MVT => ".mvt",
            Self::Unknown => "",
        }
    }
}
