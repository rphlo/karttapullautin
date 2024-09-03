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

        Ok(Self {
            batch: gs.get("batch").unwrap() == "1",
            proc,
            vegeonly,
            cliffsonly,
            contoursonly,
            pnorthlinesangle,
            pnorthlineswidth,
        })
    }
}
