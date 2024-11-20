use std::{
    io::{self, Read, Seek, Write},
    path::{Path, PathBuf},
};

pub mod local;
pub mod memory;

/// Trait for file system operations.
pub trait FileSystem: std::fmt::Debug {
    /// Create a new directory.
    fn create_dir_all(&self, path: impl AsRef<Path>) -> Result<(), io::Error>;

    /// List the contents of a directory.
    fn list(&self, path: impl AsRef<Path>) -> Result<Vec<PathBuf>, io::Error>;

    /// Check if a file exists.
    fn exists(&self, path: impl AsRef<Path>) -> bool;

    /// Open a file for reading.
    fn open(&self, path: impl AsRef<Path>) -> Result<impl Read + Seek + Send + 'static, io::Error>;

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

    /// Write an image with the desired format.
    fn read_image(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<image::DynamicImage, image::error::ImageError> {
        image::ImageReader::new(std::io::BufReader::new(
            self.open(path).expect("Could not open file"),
        ))
        .with_guessed_format()?
        .decode()
    }
}
