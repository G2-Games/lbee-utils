pub mod entry;
pub mod header;

use byteorder::{LittleEndian, ReadBytesExt};
use header::Header;
use log::{debug, info};
use std::{
    ffi::CString, fs::File, io::{self, BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}
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

    #[error("Index not found")]
    IndexError,
}

/// A full PAK file with a header and its contents
#[derive(Debug, Clone)]
pub struct Pak {
    subdirectory: Option<String>,

    /// The path of the PAK file, can serve as an identifier or name as the
    /// header has no name for the file.
    path: PathBuf,
    header: Header,

    unknown_pre_data: Vec<u32>,
    unknown_post_header: Vec<u8>,

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

    /// Decode a PAK file from a byte stream.
    pub fn decode<T: Seek + Read>(
        input: &mut T,
        path: PathBuf,
    ) -> Result<Self, PakError> {
        info!("Reading pak from {:?}", path);
        let mut input = BufReader::new(input);

        // Read in all the header bytes
        debug!("READING: header");
        let header = Header {
            data_offset: input.read_u32::<LE>()?,
            entry_count: input.read_u32::<LE>()?,
            id_start: input.read_u32::<LE>()?,
            block_size: input.read_u32::<LE>()?,
            subdir_offset: input.read_u32::<LE>()?,
            unknown2: input.read_u32::<LE>()?,
            unknown3: input.read_u32::<LE>()?,
            unknown4: input.read_u32::<LE>()?,
            flags: PakFlags(input.read_u32::<LE>()?),
        };
        info!("{} entries detected", header.entry_count);
        debug!("Block size is {} bytes", header.block_size);
        debug!("Flag bits {:#032b}", header.flags().0);

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
        debug!("Pre-position bytes: {}", unknown_pre_data.len());

        if input.stream_position()? == header.data_offset() as u64 {
            log::error!("Header length exceeded first data block");
            return Err(PakError::HeaderError);
        }

        // Read all the offsets and lengths
        // TODO: I think a flag controls this
        debug!("READING: offsets");
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
            debug!("READING: unknown_data1");
            unknown_data1 = Some(Vec::new());
            let mut buf = [0u8; 12];
            for _ in 0..header.entry_count() {
                input.read_exact(&mut buf)?;

                unknown_data1.as_mut().unwrap().push(buf);
            }
        }

        // Read all the file names
        let mut file_names = None;
        let mut subdirectory = None;
        if header.flags.has_names() {
            debug!("READING: file_names");
            if header.subdir_offset != 0 {
                subdirectory = Some(read_cstring(&mut input)?);
            }
            file_names = Some(Vec::new());
            for _ in 0..header.entry_count() {
                let strbuf = read_cstring(&mut input)?;
                file_names.as_mut().unwrap().push(strbuf.clone());
            }
        }

        let unknown_post_header_size = header.data_offset() as u64 - input.stream_position()?;
        let mut unknown_post_header = vec![0u8; unknown_post_header_size as usize];
        input.read_exact(&mut unknown_post_header)?;

        // Read all entry data
        debug!("Creating entry list");
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
                index: i,
                offset: offset_info.offset,
                length: offset_info.length,
                unknown1,
                data,
                name,
                id: header.id_start + i as u32,
            };
            entries.push(entry);
        }
        debug!("Entry list contains {} entries", entries.len());

        Ok(Pak {
            subdirectory,
            header,
            unknown_pre_data,
            entries,
            unknown_post_header,
            path,
        })
    }

    /// Convenience method to save the PAK to a file.
    pub fn save<P: ?Sized + AsRef<Path>>(&self, path: &P) -> Result<(), PakError> {
        let mut output = BufWriter::new(File::create(path).unwrap());

        self.encode(&mut output).unwrap();

        Ok(())
    }

    /// Encode a PAK file into a byte stream.
    pub fn encode<T: Write>(
        &self,
        mut output: &mut T
    ) -> Result<(), PakError> {
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
            if let Some(subdir) = &self.subdirectory {
                output.write_all(
                    CString::new(subdir.as_bytes()).unwrap().to_bytes_with_nul()
                )?;
            }
            for entry in self.entries() {
                let name = entry.name.as_ref().unwrap();
                output.write_all(
                    CString::new(name.as_bytes()).unwrap().to_bytes_with_nul()
                )?;
            }
        }

        output.write_all(&self.unknown_post_header)?;

        //let mut block_offset = self.header().data_offset / self.header().block_size;

        for entry in self.entries() {
            //let block_size = entry.data.len().div_ceil(self.header().block_size as usize);
            let mut remainder = 2048 - entry.data.len().rem_euclid(self.header().block_size as usize);
            if remainder == 2048 {
                remainder = 0;
            }
            output.write_all(&entry.data)?;
            output.write_all(&vec![0u8; remainder])?;

            //println!("entry len {}", entry.data.len());
            //println!("remainder {}", remainder);
            //println!("block_offset {} - expected offset {}", block_offset, entry.offset);

            //block_offset += block_size as u32;
        }

        Ok(())
    }

    /// Replace the data of an entry with some other bytes.
    ///
    /// This function updates the offsets of all entries to fit within the
    /// chunk size specified in the header.
    pub fn replace(
        &mut self,
        index: usize,
        replacement_bytes: &[u8],
    ) -> Result<(), PakError> {
        let block_size = self.header().block_size();

        let replaced_entry;
        if let Some(entry) = self.entries.get_mut(index) {
            replaced_entry = entry
        } else {
            log::error!("Entry {} not found!", index);
            return Err(PakError::IndexError)
        };

        if let Some(name) = replaced_entry.name() {
            info!("Replacing entry {}: {}", index, name);
        } else {
            info!("Replacing entry {}: {}", index, replaced_entry.id());
        }

        // Replace the entry data
        replaced_entry.data = replacement_bytes.to_vec();
        replaced_entry.length = replaced_entry.data.len() as u32;

        // Get the offset of the next entry based on the current one
        let mut next_offset =
            replaced_entry.offset + replaced_entry.length.div_ceil(block_size);

        // Update the position of all subsequent entries
        let mut i = 0;
        for entry in self.entries.iter_mut().skip(index + 1) {
            entry.offset = next_offset;

            next_offset = entry.offset + entry.length.div_ceil(block_size);
            i += 1;
        }

        info!("Aligned {} subsequent entries", i);

        Ok(())
    }

    /// Replace the data of an entry with some other bytes, indexing by name.
    ///
    /// Read more in [`Pak::replace()`]
    pub fn replace_by_name(
        &mut self,
        name: String,
        replacement_bytes: &[u8],
    ) -> Result<(), PakError> {
        let entry = self.get_entry_by_name(&name);
        let index = if let Some(entry) = entry {
            entry.index
        } else {
            return Err(PakError::IndexError)
        };

        self.replace(index, replacement_bytes)?;

        Ok(())
    }

    pub fn replace_by_id(
        &mut self,
        id: u32,
        replacement_bytes: &[u8],
    ) -> Result<(), PakError> {
        let entry = self.get_entry_by_id(id);
        let index = if let Some(entry) = entry {
            entry.index
        } else {
            return Err(PakError::IndexError)
        };

        self.replace(index, replacement_bytes)?;

        Ok(())
    }

    /// Get the header information from the PAK
    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get an individual entry from the PAK by its ID
    pub fn get_entry_by_id(&mut self, id: u32) -> Option<&mut Entry> {
        self.entries
            .get_mut((id - self.header.id_start) as usize)
    }

    pub fn get_entry_by_name(&mut self, name: &str) -> Option<&mut Entry> {
        self.entries
            .iter_mut()
            .find(|e|
                e.name.as_ref().is_some_and(|n| n == name)
            )
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
            .is_some_and(|n| n == name))
    }
}

fn read_cstring<T: Seek + Read + BufRead>(input: &mut T) -> Result<String, io::Error> {
    let mut string_buf = vec![];
    input.read_until(0x00, &mut string_buf)?;
    string_buf.pop();

    Ok(String::from_utf8_lossy(&string_buf).to_string())
}
