use std::{error::Error, fs::File, io::{BufWriter, Write}, path::Path};

/// A single file entry in a PAK file
#[derive(Debug, Clone)]
pub struct Entry {
    /// The location within the PAK file, this number is multiplied by the
    /// block size
    pub(super) offset: u32,

    /// The size of the entry in bytes
    pub(super) length: u32,

    /// The actual data which make up the entry
    pub(super) data: Vec<u8>,

    /// The name of the entry as stored in the PAK
    pub(super) name: Option<String>,

    pub(super) unknown1: Option<u32>,

    /// The ID of the entry, effectively an index
    pub(super) id: u32,
    pub(super) replace: bool, // TODO: Look into a better way to indicate this
}

impl Entry {
    /// Get the name of the [`Entry`]
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    /// Save an [`Entry`] as its underlying data to a file
    pub fn save<P: ?Sized + AsRef<Path>>(&self, path: &P) -> Result<(), Box<dyn Error>> {
        let mut path = path.as_ref().to_path_buf();
        if !path.is_dir() {
            return Err("Path must be a directory".into());
        }

        // Save the file to <folder> + <file name>
        if let Some(name) = &self.name {
            path.push(&name);
        } else {
            path.push(&self.id.to_string())
        }

        let mut out_file = BufWriter::new(File::create(path)?);

        out_file.write_all(&self.data)?;
        out_file.flush()?;

        Ok(())
    }

    /// Get the raw byte data of an [`Entry`]
    pub fn as_bytes(&self) -> &Vec<u8> {
        &self.data
    }
}
