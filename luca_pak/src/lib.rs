mod entry;
mod header;

use byteorder::{LittleEndian, ReadBytesExt};
use header::Header;
use log::{debug, info};
use std::{
    ffi::CString, fs::File, io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}
};
use thiserror::Error;
use byteorder::WriteBytesExt;

type LE = LittleEndian;

use crate::{entry::Entry, header::PakFlags};

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
    /// The path of the PAK file, can serve as an identifier or name as the
    /// header has no name for the file.
    path: PathBuf,
    header: Header,

    pub unknown_pre_data: Vec<u32>,
    unknown_post_header: Vec<u8>,

    rebuild: bool, // TODO: Look into a better way to indicate this, or if it's needed at all

    entries: Vec<Entry>,
}

struct FileLocation {
    offset: u32,
    length: u32,
}

impl Pak {
    /// Convenience method to open a PAK file from a path and decode it
    pub fn open<P: ?Sized + AsRef<Path>>(path: &P) -> Result<Self, PakError> {
        let mut file = File::open(path)?;

        Pak::decode(&mut file, path.as_ref().to_path_buf())
    }

    /// Decode a PAK file from a byte stream
    pub fn decode<T: Seek + Read>(
        input: &mut T,
        path: PathBuf,
    ) -> Result<Self, PakError> {
        info!("Reading pak from {:?}", path);
        let mut input = BufReader::new(input);

        // Read in all the header bytes
        info!("READING: header");
        let header = Header {
            data_offset: input.read_u32::<LE>()?,
            entry_count: input.read_u32::<LE>()?,
            id_start: input.read_u32::<LE>()?,
            block_size: input.read_u32::<LE>()?,
            unknown1: input.read_u32::<LE>()?,
            unknown2: input.read_u32::<LE>()?,
            unknown3: input.read_u32::<LE>()?,
            unknown4: input.read_u32::<LE>()?,
            flags: PakFlags(input.read_u32::<LE>()?),
        };
        info!("{} entries detected", header.entry_count);
        info!("Block size is {} bytes", header.block_size);
        info!("Flag bits {:#032b}", header.flags().0);

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
        info!("Pre-position bytes: {}", unknown_pre_data.len());

        if input.stream_position()? == header.data_offset() as u64 {
            log::error!("Header length exceeded first data block");
            return Err(PakError::HeaderError);
        }

        // Read all the offsets and lengths
        // TODO: I think a flag controls this
        info!("READING: offsets");
        let mut offsets = Vec::new();
        for _ in 0..header.entry_count() {
            let offset = input.read_u32::<LE>().unwrap();
            let length = input.read_u32::<LE>().unwrap();
            offsets.push(FileLocation {
                offset,
                length,
            });
        }

        // Read all unknown_data1
        let mut unknown_data1 = None;
        if header.flags.has_unknown_data1() {
            info!("READING: unknown_data1");
            unknown_data1 = Some(Vec::new());
            let mut buf = [0u8; 12];
            for _ in 0..header.entry_count() {
                input.read_exact(&mut buf)?;

                unknown_data1.as_mut().unwrap().push(buf);
            }
        }

        // Read all the file names
        let mut file_names = None;
        if header.flags.has_names() {
            info!("READING: file_names");
            let mut string_buf = Vec::new();
            file_names = Some(Vec::new());
            for _ in 0..header.entry_count() {
                string_buf.clear();
                input.read_until(0x00, &mut string_buf)?;
                string_buf.pop();

                let strbuf = String::from_utf8_lossy(&string_buf).to_string();
                file_names.as_mut().unwrap().push(strbuf.clone());
            }
        }

        let unknown_post_header_size = header.data_offset() as u64 - input.stream_position()?;
        let mut unknown_post_header = vec![0u8; unknown_post_header_size as usize];
        input.read_exact(&mut unknown_post_header)?;

        // Read all entry data
        info!("Creating entry list");
        let mut entries: Vec<Entry> = Vec::new();
        for (i, offset_info) in offsets.iter().enumerate().take(header.entry_count() as usize) {
            debug!("Seeking to block {}", offset_info.offset);
            // Seek to and read the entry data
            input
                .seek(SeekFrom::Start(
                    offset_info.offset as u64 * header.block_size() as u64,
                ))
                .unwrap();
            let mut data = vec![0u8; offset_info.length as usize];
            input.read_exact(&mut data).unwrap();

            let name = if let Some(file_names) = &file_names {
                file_names.get(i).cloned()
            } else {
                None
            };

            let unknown1 = if let Some(unknown_data1) = &unknown_data1 {
                unknown_data1.get(i).cloned()
            } else {
                None
            };

            // Build the entry from the data we now know
            let entry = Entry {
                offset: offset_info.offset,
                length: offset_info.length,
                unknown1,
                data,
                name,
                id: header.id_start + i as u32,
            };
            entries.push(entry);
        }
        info!("Entry list contains {} entries", entries.len());

        Ok(Pak {
            header,
            unknown_pre_data,
            entries,
            unknown_post_header,
            path,
            rebuild: false,
        })
    }

    pub fn encode<T: Write + Seek>(&self, mut output: &mut T) -> Result<(), PakError> {
        let mut block_offset = 0;
        self.header.write_into(&mut output)?;

        // Write unknown data
        output.write_all(
            &self.unknown_pre_data
                .iter()
                .flat_map(|dw| dw.to_le_bytes())
                .collect::<Vec<u8>>()
        )?;

        // Write offsets and lengths
        for entry in self.entries() {
            output.write_u32::<LE>(entry.offset)?;
            output.write_u32::<LE>(entry.length)?;
        }

        // Write out unknown data if the flags indicate it should have some
        if self.header.flags().has_unknown_data1() {
            for entry in self.entries() {
                output.write_all(entry.unknown1.as_ref().unwrap())?;
            }
        }

        // Write names if the flags indicate it should have them
        if self.header.flags().has_names() {
            for entry in self.entries() {
                let name = entry.name.as_ref().unwrap();
                output.write_all(
                    CString::new(name.as_bytes()).unwrap().to_bytes_with_nul()
                )?;
            }
        }

        output.write_all(&self.unknown_post_header)?;

        block_offset += self.header().data_offset / self.header().block_size;

        for entry in self.entries() {
            let block_size = entry.data.len().div_ceil(self.header().block_size as usize);
            let remainder = 2048 - entry.data.len().rem_euclid(self.header().block_size as usize);

            debug!("entry {:?} len {}", entry.name(), entry.data.len());
            debug!("remainder {}", remainder);
            debug!("block_offset {} - expected offset {}", block_offset, entry.offset);
            output.write_all(&entry.data)?;
            output.write_all(&vec![0u8; remainder as usize])?;
            block_offset += block_size as u32;
        }

        Ok(())
    }

    /// Get the header information from the PAK
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get an individual entry from the PAK by its index
    pub fn get_entry(&self, index: u32) -> Option<&Entry> {
        self.entries.get(index as usize)
    }

    /// Get an individual entry from the PAK by its ID
    pub fn get_entry_by_id(&self, id: u32) -> Option<&Entry> {
        self.entries.get((id - self.header.id_start) as usize)
    }

    pub fn get_entry_by_name(&self, name: &str) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|e| e.name.as_ref()
            .is_some_and(|n| n == &name))
    }

    /// Get a list of all entries from the PAK
    pub fn entries(&self) -> &Vec<Entry> {
        &self.entries
    }

    /// Returns true if the PAK file contains an entry with the given name
    pub fn contains_name(&self, name: &str) -> bool {
        self.entries
            .iter()
            .any(|e| e.name.as_ref()
            .is_some_and(|n| n == &name))
    }
}
