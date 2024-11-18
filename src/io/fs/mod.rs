use std::{
    io::{self, Read, Seek, Write},
    path::Path,
};

/// Trait for file system operations.
pub trait FileSystem {
    /// Create a new directory.
    fn create_dir_all(&self, path: impl AsRef<Path>) -> Result<(), io::Error>;

    /// List the contents of a directory.
    fn list(&self, path: impl AsRef<Path>) -> Result<Vec<String>, io::Error>;

    /// Check if a file exists.
    fn exists(&self, path: impl AsRef<Path>) -> bool;

    /// Open a file for reading.
    fn open(&self, path: impl AsRef<Path>) -> Result<impl Read + Seek, io::Error>;

    /// Open a file for writing.
    fn create(&self, path: impl AsRef<Path>) -> Result<impl Write + Seek, io::Error>;

    /// Read a file into a String.
    fn read_to_string(&self, path: impl AsRef<Path>) -> Result<String, io::Error>;

    /// Remove a file.
    fn remove_file(&self, path: impl AsRef<Path>) -> Result<(), io::Error>;

    /// Get the size of a file in bytes.
    fn file_size(&self, path: impl AsRef<Path>) -> Result<u64, io::Error>;

    /// Copy a file.
    fn copy(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<(), io::Error>;
}

/// [`FileSystem`] implementation for the local file system.
#[derive(Clone)]
pub struct LocalFileSystem;

impl FileSystem for LocalFileSystem {
    fn create_dir_all(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        std::fs::create_dir_all(path)
    }

    fn list(&self, path: impl AsRef<Path>) -> Result<Vec<String>, io::Error> {
        let mut entries = vec![];
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            entries.push(entry.file_name().to_string_lossy().to_string());
        }
        Ok(entries)
    }

    fn exists(&self, path: impl AsRef<Path>) -> bool {
        path.as_ref().exists()
    }

    fn read_to_string(&self, path: impl AsRef<Path>) -> Result<String, io::Error> {
        std::fs::read_to_string(path)
    }

    fn open(&self, path: impl AsRef<Path>) -> Result<impl Read + Seek, io::Error> {
        std::fs::File::open(path)
    }

    fn create(&self, path: impl AsRef<Path>) -> Result<impl Write + Seek, io::Error> {
        std::fs::File::create(path)
    }

    fn remove_file(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        std::fs::remove_file(path)
    }

    fn file_size(&self, path: impl AsRef<Path>) -> Result<u64, io::Error> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.len())
    }

    fn copy(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<(), io::Error> {
        std::fs::copy(from, to)?;
        Ok(())
    }
}
