use std::{
    io::{self, Read, Seek, Write},
    path::Path,
};

/// Trait for file system operations.
pub trait FileSystem {
    /// Create a new directory.
    fn mkdir(&self, path: &Path) -> Result<(), io::Error>;

    /// List the contents of a directory.
    fn list(&self, path: &Path) -> Result<Vec<String>, io::Error>;

    /// Check if a file exists.
    fn exists(&self, path: &Path) -> Result<bool, io::Error>;

    /// Open a file for reading.
    fn read(&self, path: &Path) -> Result<impl Read + Seek, io::Error>;

    /// Open a file for writing.
    fn write(&self, path: &Path) -> Result<impl Write, io::Error>;

    /// Remove a file.
    fn remove(&self, path: &Path) -> Result<(), io::Error>;

    /// Copy a file.
    fn copy(&self, from: &Path, to: &Path) -> Result<(), io::Error>;
}

/// [`FileSystem`] implementation for the local file system.
pub struct LocalFileSystem;

impl FileSystem for LocalFileSystem {
    fn mkdir(&self, path: &Path) -> Result<(), io::Error> {
        std::fs::create_dir_all(path)
    }

    fn list(&self, path: &Path) -> Result<Vec<String>, io::Error> {
        let mut entries = vec![];
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            entries.push(entry.file_name().to_string_lossy().to_string());
        }
        Ok(entries)
    }

    fn exists(&self, path: &Path) -> Result<bool, io::Error> {
        Ok(path.exists())
    }

    fn read(&self, path: &Path) -> Result<impl Read + Seek, io::Error> {
        std::fs::File::open(path)
    }

    fn write(&self, path: &Path) -> Result<impl Write, io::Error> {
        std::fs::File::create(path)
    }

    fn remove(&self, path: &Path) -> Result<(), io::Error> {
        std::fs::remove_file(path)
    }

    fn copy(&self, from: &Path, to: &Path) -> Result<(), io::Error> {
        std::fs::copy(from, to)?;
        Ok(())
    }
}
