mod entry;
mod header;

use std::{fs::File, io::{self, BufRead, BufReader, Read, Seek, SeekFrom}, path::{Path, PathBuf}};
use byteorder::{LittleEndian, ReadBytesExt};
use header::Header;
use thiserror::Error;

use crate::entry::Entry;

/// An error associated with a PAK file
#[derive(Error, Debug)]
pub enum PakError {
    #[error("Could not read/write file")]
    IoError(#[from] io::Error),

    #[error("Expected {} files, got {} in {}", 0, 1, 2)]
    FileCountMismatch(usize, usize, &'static str),

    #[error("Malformed header information")]
    HeaderError,
}

/// A full PAK file with a header and its contents
#[derive(Debug, Clone)]
pub struct Pak {
    header: Header,

    unknown_pre_data: Vec<u32>,

    entries: Vec<Entry>,

    unknown_flag_data: Vec<u8>,

    path: PathBuf,
    rebuild: bool, // TODO: Look into a better way to indicate this, or if it's needed at all
}

pub struct PakFlags(u32);

impl PakFlags {
    pub fn has_names(&self) -> bool {
        // 0b01000000000
        self.0 & 0x200 != 0
    }

    pub fn has_offsets(&self) -> bool {
        // 0b10000000000
        self.0 & 0x400 != 0
    }
}

type LE = LittleEndian;

impl Pak {
    /// Convenience method to open a PAK file from a path and decode it
    pub fn open<P: ?Sized + AsRef<Path>>(path: &P) -> Result<Self, PakError> {
        let mut file = File::open(path)?;

        Pak::decode(&mut file, path.as_ref().to_path_buf())
    }

    /// Decode a PAK file from a byte stream
    pub fn decode<T: Seek + ReadBytesExt + Read>(input: &mut T, path: PathBuf) -> Result<Self, PakError> {
        let mut input = BufReader::new(input);

        // Read in all the header bytes
        let header = Header {
            data_offset: input.read_u32::<LE>()?,
            entry_count: input.read_u32::<LE>()?,
            id_start: input.read_u32::<LE>()?,
            block_size: input.read_u32::<LE>()?,
            unknown1: input.read_u32::<LE>()?,
            unknown2: input.read_u32::<LE>()?,
            unknown3: input.read_u32::<LE>()?,
            unknown4: input.read_u32::<LE>()?,
            flags: input.read_u32::<LE>()?,
        };

        let first_offset = header.data_offset() / header.block_size();

        // Read some unknown data before the data we want
        let mut unknown_pre_data = Vec::new();
        while input.stream_position()? < header.data_offset() as u64 {
            let unknown = input.read_u32::<LE>()?;
            if unknown == first_offset {
                input.seek_relative(-4)?;
                break;
            }

            unknown_pre_data.push(unknown);
        }
        dbg!(unknown_pre_data.len());

        if input.stream_position()? == header.data_offset() as u64 {
            return Err(PakError::HeaderError)
        }

        // Read all the offsets and lengths
        let mut offsets = Vec::new();
        for _ in 0..header.entry_count() {
            let offset = input.read_u32::<LE>().unwrap();
            let length = input.read_u32::<LE>().unwrap();
            offsets.push((offset, length));
        }

        // Read all the file names
        let mut file_names = Vec::new();
        let mut string_buf = Vec::new();
        for _ in 0..header.entry_count() {
            string_buf.clear();
            input.read_until(0x00, &mut string_buf)?;
            string_buf.pop();

            let strbuf = String::from_utf8_lossy(&string_buf).to_string();
            file_names.push(strbuf.clone());
        }

        let unknown_flag_size = header.data_offset() as u64 - input.stream_position()?;
        let mut unknown_flag_data = vec![0u8; unknown_flag_size as usize];
        input.read_exact(&mut unknown_flag_data)?;

        // Read all entry data
        let mut entries: Vec<Entry> = Vec::new();
        for i in 0..header.entry_count() as usize {
            // Seek to and read the entry data
            input.seek(SeekFrom::Start(offsets[i].0 as u64 * header.block_size() as u64)).unwrap();
            let mut data = vec![0u8; offsets[i].1 as usize];
            input.read_exact(&mut data).unwrap();

            // Build the entry from the data we now know
            let entry = Entry {
                offset: offsets[i].0,
                length: offsets[i].1,
                data,
                name: Some(file_names[i].clone()),
                unknown1: todo!(),
                id: header.id_start + i as u32,
                replace: false,
            };
            entries.push(entry);
        }

        Ok(Pak {
            header,
            unknown_pre_data,
            entries,
            unknown_flag_data,
            path,
            rebuild: false,
        })
    }

    /// Get the header information from the PAK
    pub fn header(&self) -> &Header {
        &self.header
    }

    /// Get an individual entry from the PAK by its index
    pub fn get_entry(&self, index: u32) -> Option<&Entry> {
        self.entries.get(index as usize)
    }

    /// Get an individual entry from the PAK by its ID
    pub fn get_entry_by_id(&self, id: u32) -> Option<&Entry> {
        self.entries.get((id - self.header.id_start) as usize)
    }

    /// Get a list of all entries from the PAK
    pub fn entries(&self) -> &Vec<Entry> {
        &self.entries
    }

    pub fn contains_name(&self, name: String) -> bool {
        self.entries
            .iter()
            .find(|e|
                e.name.as_ref().is_some_and(|n| n == &name)
            ).is_some()
    }
}
