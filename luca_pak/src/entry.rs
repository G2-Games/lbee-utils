use std::{error::Error, fs::File, io::{BufWriter, Write}, path::Path};

/// A single file entry in a PAK file
#[derive(Debug, Clone)]
pub struct Entry {
    pub(super) offset: u32,
    pub(super) length: u32,
    pub(super) data: Vec<u8>,
    pub(super) name: String,
    pub(super) id: u8,
    pub(super) replace: bool, // TODO: Look into a better way to indicate this
}

impl Entry {
    /// Get the name of the [`Entry`]
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Save an [`Entry`] as its underlying data to a file
    pub fn save<P: ?Sized + AsRef<Path>>(&self, path: &P) -> Result<(), Box<dyn Error>> {
        let mut path = path.as_ref().to_path_buf();
        if !path.is_dir() {
            return Err("Path must be a directory".into());
        }

        // Save the file to <folder> + <file name>
        path.push(&self.name);

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
