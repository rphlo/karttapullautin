use std::path::Path;

use ini::Ini;

/// The config parsed from the .ini configuration file.
pub struct Config {
    /// the main ini file
    pub conf: Ini,
}

impl Config {
    pub fn load_or_create_default() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new("pullauta.ini");
        // populate the default if no file was found
        if !path.exists() {
            std::fs::write(path, include_bytes!("../pullauta.default.ini"))?;
        }

        let conf = Ini::load_from_file(path)?;

        Ok(Self { conf })
    }
}
