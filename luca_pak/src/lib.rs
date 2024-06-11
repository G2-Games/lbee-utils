mod entry;

use std::{fs::File, io::{self, BufRead, BufReader, Read, Seek, SeekFrom}, path::Path};
use byteorder::{LittleEndian, ReadBytesExt};
use thiserror::Error;

use crate::entry::Entry;

/// A full PAK file with a header and its contents
#[derive(Debug, Clone)]
pub struct Pak {
    header: Header,
    files: Vec<Entry>,

    file_name: String,
    rebuild: bool, // TODO: Look into a better way to indicate this
}

/// The header of a PAK file
#[derive(Debug, Clone)]
struct Header {
    data_offset: u32,
    file_count: u32,
    id_start: u32,
    block_size: u32,

    unknown1: u32,
    unknown2: u32,
    unknown3: u32,
    unknown4: u32,

    flags: u32,
}

impl Header {
    pub fn block_size(&self) -> u32 {
        self.block_size
    }

    pub fn file_count(&self) -> u32 {
        self.file_count
    }

    pub fn data_offset(&self) -> u32 {
        self.data_offset
    }
}

#[derive(Error, Debug)]
pub enum PakError {
    #[error("Could not read/write file")]
    IoError(#[from] io::Error),

    #[error("Expected {} files, got {} in {}", 0, 1, 2)]
    FileCountMismatch(usize, usize, &'static str),

    #[error("Malformed header information")]
    HeaderError,
}

type LE = LittleEndian;

impl Pak {
    pub fn open<P: ?Sized + AsRef<Path>>(path: &P) -> Result<Self, PakError> {
        let mut file = File::open(path)?;

        let filename = path.as_ref().file_name().unwrap().to_string_lossy().to_string();

        Pak::decode(&mut file, filename)
    }

    pub fn decode<T: Seek + ReadBytesExt + Read>(input: &mut T, file_name: String) -> Result<Self, PakError> {
        let mut input = BufReader::new(input);

        // Read in all the header bytes
        let header = Header {
            data_offset: input.read_u32::<LE>().unwrap(),
            file_count: input.read_u32::<LE>().unwrap(),
            id_start: input.read_u32::<LE>().unwrap(),
            block_size: input.read_u32::<LE>().unwrap(),
            unknown1: input.read_u32::<LE>().unwrap(),
            unknown2: input.read_u32::<LE>().unwrap(),
            unknown3: input.read_u32::<LE>().unwrap(),
            unknown4: input.read_u32::<LE>().unwrap(),
            flags: input.read_u32::<LE>().unwrap(),
        };
        dbg!(&header);

        let first_offset = header.data_offset() / header.block_size();

        // Seek to the end of the header
        input.seek(io::SeekFrom::Start(0x24))?;
        while input.stream_position()? < header.data_offset() as u64 {
            if input.read_u32::<LE>().unwrap() == first_offset {
                input.seek_relative(-4)?;
                break;
            }
        }

        if input.stream_position()? == header.data_offset() as u64 {
            return Err(PakError::HeaderError)
        }

        // Read all the offsets and lengths
        let mut offsets = Vec::new();
        for _ in 0..header.file_count() {
            let offset = input.read_u32::<LE>().unwrap();
            let length = input.read_u32::<LE>().unwrap();

            dbg!(offset);
            dbg!(length);

            offsets.push((offset, length));
        }

        // Read all the file names
        let mut file_names = Vec::new();
        let mut buf = Vec::new();
        for _ in 0..header.file_count() {
            buf.clear();
            input.read_until(0x00, &mut buf)?;
            buf.pop();

            let strbuf = String::from_utf8(buf.clone()).unwrap();
            file_names.push(strbuf.clone());
        }
        dbg!(&file_names);

        let mut entries: Vec<Entry> = Vec::new();
        for i in 0..header.file_count() as usize {
            dbg!(i);

            // Seek to and read the entry data
            input.seek(SeekFrom::Start(offsets[i].0 as u64 * header.block_size() as u64)).unwrap();
            let mut data = vec![0u8; offsets[i].1 as usize];
            input.read_exact(&mut data).unwrap();

            // Build the entry from the data we know
            let entry = Entry {
                offset: offsets[i].0,
                length: offsets[i].1,
                data,
                name: file_names[i].clone(),
                id: 0,
                replace: false,
            };
            entries.push(entry);
        }

        println!("Got entries for {} files", entries.len());

        Ok(Pak {
            header,
            files: entries,
            file_name,
            rebuild: false,
        })
    }

    pub fn get_file(&self, index: u32) -> Option<&Entry> {
        self.files.get(index as usize)
    }

    pub fn files(&self) -> &Vec<Entry> {
        &self.files
    }
}
