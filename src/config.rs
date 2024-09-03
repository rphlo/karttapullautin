use std::path::Path;

use ini::Ini;

/// The config parsed from the .ini configuration file.
pub struct Config {
    pub batch: bool,
    pub proc: u64,

    // only one can be set at a time
    pub vegeonly: bool,
    pub cliffsonly: bool,
    pub contoursonly: bool,

    pub pnorthlinesangle: f64,
    pub pnorthlineswidth: usize,

    pub lazfolder: String,
    pub batchoutfolder: String,
    pub savetempfiles: bool,
    pub savetempfolders: bool,

    pub scalefactor: f64,
    pub vege_bitmode: bool,
    pub zoff: f64,
    pub thinfactor: f64,
}

impl Config {
    pub fn load_or_create_default() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new("pullauta.ini");
        // populate the default if no file was found
        if !path.exists() {
            std::fs::write(path, include_bytes!("../pullauta.default.ini"))?;
        }

        let conf = Ini::load_from_file(path)?;

        let gs = conf.general_section();

        // only one can be set at a time
        let vegeonly: bool = conf.general_section().get("vegeonly").unwrap_or("0") == "1";
        let cliffsonly: bool = conf.general_section().get("cliffsonly").unwrap_or("0") == "1";
        let contoursonly: bool = conf.general_section().get("contoursonly").unwrap_or("0") == "1";

        if (vegeonly && (cliffsonly || contoursonly))
            || (cliffsonly && (vegeonly || contoursonly))
            || (contoursonly && (vegeonly || cliffsonly))
        {
            return Err(
                "Only one of vegeonly, cliffsonly, or contoursonly can be set!"
                    .to_string()
                    .into(),
            );
        }

        let pnorthlinesangle: f64 = gs
            .get("northlinesangle")
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);
        let pnorthlineswidth: usize = gs
            .get("northlineswidth")
            .unwrap_or("0")
            .parse::<usize>()
            .unwrap_or(0);

        let proc: u64 = conf
            .general_section()
            .get("processes")
            .unwrap()
            .parse::<u64>()
            .unwrap();

        let lazfolder = gs.get("lazfolder").unwrap_or("").to_string();
        let batchoutfolder = gs.get("batchoutfolder").unwrap_or("").to_string();
        let savetempfiles: bool = gs.get("savetempfiles").unwrap() == "1";
        let savetempfolders: bool = gs.get("savetempfolders").unwrap() == "1";

        let scalefactor: f64 = gs
            .get("scalefactor")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        let vege_bitmode: bool = gs.get("vege_bitmode").unwrap_or("0") == "1";
        let zoff = gs
            .get("zoffset")
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);
        let mut thinfactor: f64 = gs
            .get("thinfactor")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        if thinfactor == 0.0 {
            thinfactor = 1.0;
        }

        Ok(Self {
            batch: gs.get("batch").unwrap() == "1",
            proc,
            vegeonly,
            cliffsonly,
            contoursonly,
            pnorthlinesangle,
            pnorthlineswidth,
            lazfolder,
            batchoutfolder,
            savetempfolders,
            savetempfiles,
            scalefactor,
            vege_bitmode,
            zoff,
            thinfactor,
        })
    }
}
