use super::FileSystem;
use rustc_hash::FxHashMap as HashMap;

use core::str;
use std::io::{self, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// An in-memory implementation of [`FileSystem`] for use whenever there is no access to a local
/// file system (such as on WASM), or to speed up the processing when there is a lot of RAM available.
///
/// This object is thread-safe and can be shared between threads. Uses [`Arc`] internally so it is
/// cheap to clone.
#[derive(Debug, Clone)]
pub struct MemoryFileSystem {
    root: Arc<RwLock<Directory>>,
}

impl Default for MemoryFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryFileSystem {
    /// Create a new empty memory file system.
    pub fn new() -> Self {
        Self {
            root: Arc::new(RwLock::new(Directory::new())),
        }
    }

    /// Load the contents of a file on the local file system into the memory file system.
    pub fn load_from_disk(
        &self,
        from_disk: impl AsRef<Path>,
        to_internal: impl AsRef<Path>,
    ) -> io::Result<()> {
        let bytes = std::fs::read(from_disk)?;
        let mut writer = self.create(to_internal)?;
        writer.write_all(&bytes)?;
        Ok(())
    }
    /// Write the contents of a  file in the memory file system to the local file system.
    pub fn save_to_disk(
        &self,
        from_internal: impl AsRef<Path>,
        to_external: impl AsRef<Path>,
    ) -> io::Result<()> {
        let mut reader = io::BufReader::new(self.open(from_internal)?);
        let mut writer = io::BufWriter::new(std::fs::File::create(to_external)?);
        std::io::copy(&mut reader, &mut writer)?;
        Ok(())
    }
}

#[derive(Debug)]
struct Directory {
    subdirs: HashMap<String, Directory>,
    files: HashMap<String, FileEntry>,
}

impl Directory {
    /// Create a new empty directory.
    fn new() -> Self {
        Self {
            subdirs: HashMap::default(),
            files: HashMap::default(),
        }
    }

    fn get_directory(&self, path: impl AsRef<Path>) -> Result<&Directory, io::Error> {
        let path = path.as_ref();
        let mut dir: &Directory = self;
        for component in path.components() {
            let name = component.as_os_str().to_string_lossy().to_string();
            dir = match dir.subdirs.get(&name) {
                Some(subdir) => subdir,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "directory not found",
                    ))
                }
            };
        }
        Ok(dir)
    }

    fn get_directory_mut(&mut self, path: impl AsRef<Path>) -> Result<&mut Directory, io::Error> {
        let path = path.as_ref();
        let mut dir: &mut Directory = self;
        for component in path.components() {
            let name = component.as_os_str().to_string_lossy().to_string();
            dir = match dir.subdirs.get_mut(&name) {
                Some(subdir) => subdir,
                None => {
                    return Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "directory not found",
                    ))
                }
            };
        }
        Ok(dir)
    }
}

#[derive(Debug)]
struct FileEntry {
    /// data is stored as an Arc to allow for multiple readers.
    /// Wrapped in an Arc to allow for swapping the value when the Writer is dropped / finished.
    data: Arc<RwLock<FileData>>,
}

impl FileEntry {
    /// Create a new empty file entry.
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(FileData::new())),
        }
    }
}
impl std::fmt::Debug for FileData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let len = self.0.len();
        f.debug_struct("FileData").field("len", &len).finish()
    }
}

// TODO: these should implement Read, Write, Seek and be returned by the FileSystem methods
struct WritableFile {
    /// The data beeing written to the file
    data: io::Cursor<Vec<u8>>,
    /// links back to the file entry so we can swap the data when the writer is dropped
    data_link: Arc<RwLock<FileData>>,
}

impl Write for WritableFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.data.flush()
    }
}

impl Seek for WritableFile {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.data.seek(pos)
    }
}

impl Drop for WritableFile {
    // swap the data into the file entry on drop
    fn drop(&mut self) {
        let data = core::mem::replace(&mut self.data, io::Cursor::new(Vec::new()));
        let mut data_link = self.data_link.write().unwrap();
        *data_link = FileData(Arc::new(data.into_inner()));
    }
}

/// Holds the data of a file. Cheap to clone because the data is behind an [`Arc`].
#[derive(Clone)]
struct FileData(Arc<Vec<u8>>);

impl FileData {
    fn new() -> Self {
        Self(Arc::new(Vec::new()))
    }
}

/// this allows us to treat [`FileData`] as a slice of bytes, which is useful for the [`Read`] trait
impl AsRef<[u8]> for FileData {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

fn file_parent(path: &Path) -> Result<&Path, io::Error> {
    path.parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "parent directory not found"))
}

impl FileSystem for MemoryFileSystem {
    fn create_dir_all(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        let mut root = self.root.write().unwrap();
        let path = path.as_ref();

        // make sure all directories in the path exist
        let mut dir: &mut Directory = &mut root;
        for component in path.components() {
            let name = component.as_os_str().to_string_lossy().to_string();
            dir = dir.subdirs.entry(name).or_insert_with(Directory::new);
        }
        Ok(())
    }

    fn list(&self, path: impl AsRef<Path>) -> Result<Vec<PathBuf>, io::Error> {
        let root = self.root.read().unwrap();
        let path = path.as_ref();

        // find the directory
        let dir = root.get_directory(path)?;

        // list the contents
        let mut entries = vec![];
        for name in dir.subdirs.keys() {
            entries.push(path.join(name));
        }
        for name in dir.files.keys() {
            entries.push(path.join(name));
        }
        Ok(entries)
    }

    fn exists(&self, path: impl AsRef<Path>) -> bool {
        let root = self.root.read().unwrap();
        let path = path.as_ref();

        let Some(parent) = path.parent() else {
            return false;
        };

        // find the directory or return false if it does not exist
        let Some(dir) = root.get_directory(parent).ok() else {
            return false;
        };

        // get file / directory name
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        // check if it exists as a directory or file
        dir.subdirs.contains_key(&name) || dir.files.contains_key(&name)
    }

    fn open(&self, path: impl AsRef<Path>) -> Result<impl Read + Seek + Send + 'static, io::Error> {
        let root = self.root.read().unwrap();
        let path = path.as_ref();

        let parent = file_parent(path)?;

        // find the directory
        let dir = root.get_directory(parent)?;

        // get file name
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        // get the file entry
        let file = match dir.files.get(&name) {
            Some(file) => file,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
        };

        // create a reader by cloning the Arc
        let data = file.data.read().unwrap().clone();
        Ok(io::Cursor::new(data))
    }

    fn create(&self, path: impl AsRef<Path>) -> Result<impl Write + Seek, io::Error> {
        let mut root = self.root.write().unwrap();
        let path = path.as_ref();

        let parent = file_parent(path)?;

        // find the parent directory
        let dir = root.get_directory_mut(parent)?;

        // get file name
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        // open or create new file
        let file = dir.files.entry(name).or_insert(FileEntry::new());

        // now we replace the arc with a new one which we will write to. This way existing readers
        // will continue to read the old data, while we start filling up some new data)
        let writer = WritableFile {
            data: io::Cursor::new(Vec::new()),
            data_link: file.data.clone(), // linked to the place where the data is stored
        };
        Ok(writer)
    }

    fn read_to_string(&self, path: impl AsRef<Path>) -> Result<String, io::Error> {
        let root = self.root.read().unwrap();
        let path = path.as_ref();

        let parent = file_parent(path)?;

        // find the directory
        let dir = root.get_directory(parent)?;

        // get file name
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        // get the file entry
        let file = match dir.files.get(&name) {
            Some(file) => file,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
        };

        // create a reader by cloning the Arc
        let data = file.data.read().unwrap();

        // convert to string lossily expecting all data to be valid utf8
        let str = str::from_utf8(&data.0).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidInput, format!("invalid UTF-8: {e} "))
        })?;

        Ok(str.to_string())
    }

    fn remove_file(&self, path: impl AsRef<Path>) -> Result<(), io::Error> {
        let mut root = self.root.write().unwrap();
        let path = path.as_ref();

        let parent = file_parent(path)?;

        // find the directory
        let dir = root.get_directory_mut(parent)?;

        // get file name
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        // remove the file
        dir.files
            .remove(&name)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))?;

        Ok(())
    }

    fn file_size(&self, path: impl AsRef<Path>) -> Result<u64, io::Error> {
        let root = self.root.read().unwrap();
        let path = path.as_ref();

        let parent = file_parent(path)?;

        // find the directory
        let dir = root.get_directory(parent)?;

        // get file name
        let name = path.file_name().unwrap().to_string_lossy().to_string();

        // get the file entry
        let file = match dir.files.get(&name) {
            Some(file) => file,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
        };

        let data = file.data.read().expect("file data lock poisoned");
        Ok(data.0.len() as u64)
    }

    fn copy(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<(), io::Error> {
        let mut root = self.root.write().unwrap();
        let from = from.as_ref();
        let to = to.as_ref();

        let from_parent = file_parent(from)?;
        let to_parent = file_parent(to)?;

        // find the from directory
        let from_dir = root.get_directory(from_parent)?;

        // get the from file entry and clone the data
        let from_name = from.file_name().unwrap().to_string_lossy().to_string();
        let from_file = match from_dir.files.get(&from_name) {
            Some(file) => file,
            None => return Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
        };
        let from_data = from_file
            .data
            .read()
            .expect("file data lock poisoned")
            .clone();

        // find the to directory
        let to_dir = root.get_directory_mut(to_parent)?;

        // get file names
        let to_name = to.file_name().unwrap().to_string_lossy().to_string();
        let to_file = to_dir.files.entry(to_name).or_insert(FileEntry::new());
        // copy the data
        let mut to_data = to_file.data.write().expect("file data lock poisoned");
        *to_data = from_data;

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_write_read_to_string_root() {
        let fs = super::MemoryFileSystem::new();
        let path = "test.txt";
        let content = "Hello, World!";

        fs.create(path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let read = fs.read_to_string(path).unwrap();

        assert_eq!(read, content);
    }

    #[test]
    fn test_write_read_to_string_subdir() {
        let fs = super::MemoryFileSystem::new();
        let folder = Path::new("folder");
        fs.create_dir_all(folder).unwrap();
        let path = folder.join("test.txt");
        let content = "Hello, World!";

        fs.create(&path)
            .unwrap()
            .write_all(content.as_bytes())
            .unwrap();

        let read = fs.read_to_string(path).unwrap();

        assert_eq!(read, content);
    }
}
