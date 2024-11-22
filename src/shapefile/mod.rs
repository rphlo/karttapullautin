use std::{error::Error, path::Path};

use log::info;

use crate::{config::Config, io::fs::FileSystem};

mod canvas;
mod mapping;
mod mtkrender;

pub use mtkrender::mtkshaperender;

/// Unzips the shape files and renders them to a canvas.
pub fn unzipmtk(
    fs: &impl FileSystem,
    config: &Config,
    tmpfolder: &Path,
    filenames: &[String],
) -> Result<(), Box<dyn Error>> {
    let low_file = tmpfolder.join("low.png");
    if fs.exists(&low_file) {
        fs.remove_file(low_file).unwrap();
    }

    let high_file = tmpfolder.join("high.png");
    if fs.exists(&high_file) {
        fs.remove_file(high_file).unwrap();
    }

    for zip_name in filenames.iter() {
        info!("Opening zip file {}", zip_name);
        let file = fs.open(zip_name).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        info!(
            "Extracting {:?} MB from {zip_name}",
            archive.decompressed_size().map(|s| s / 1024 / 1024)
        );
        archive.extract(tmpfolder).unwrap();
        mtkrender::mtkshaperender(fs, config, tmpfolder).unwrap();
    }
    Ok(())
}
