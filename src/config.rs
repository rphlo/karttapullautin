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

    pub skipknolldetection: bool,
    pub vegemode: bool,

    pub xfactor: f64,
    pub yfactor: f64,
    pub zfactor: f64,

    pub contour_interval: f64,
    pub basemapcontours: f64,

    pub detectbuildings: bool,

    pub water_class: String,

    pub inidotknolls: f64,
    pub smoothing: f64,
    pub curviness: f64,
    pub indexcontours: f64,
    pub formline: f64,
    pub depression_length: usize,
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
        let vegeonly: bool = gs.get("vegeonly").unwrap_or("0") == "1";
        let cliffsonly: bool = gs.get("cliffsonly").unwrap_or("0") == "1";
        let contoursonly: bool = gs.get("contoursonly").unwrap_or("0") == "1";

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

        let skipknolldetection = gs.get("skipknolldetection").unwrap_or("0") == "1";
        let vegemode: bool = gs.get("vegemode").unwrap_or("0") == "1";
        if vegemode {
            return Err("vegemode=1 not implemented, use perl version"
                .to_string()
                .into());
        }

        let mut xfactor: f64 = gs
            .get("coordxfactor")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        let mut yfactor: f64 = gs
            .get("coordyfactor")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        let mut zfactor: f64 = gs
            .get("coordzfactor")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        if xfactor == 0.0 {
            xfactor = 1.0;
        }
        if yfactor == 0.0 {
            yfactor = 1.0;
        }
        if zfactor == 0.0 {
            zfactor = 1.0;
        }

        let contour_interval: f64 = gs
            .get("contour_interval")
            .unwrap_or("5")
            .parse::<f64>()
            .unwrap_or(5.0);

        let basemapcontours: f64 = gs
            .get("basemapinterval")
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);

        let detectbuildings: bool =
            conf.general_section().get("detectbuildings").unwrap_or("0") == "1";

        let water_class = gs.get("waterclass").unwrap_or("9").to_string();

        let inidotknolls: f64 = conf
            .general_section()
            .get("knolls")
            .unwrap_or("0.8")
            .parse::<f64>()
            .unwrap_or(0.8);
        let smoothing: f64 = conf
            .general_section()
            .get("smoothing")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        let curviness: f64 = conf
            .general_section()
            .get("curviness")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        let indexcontours: f64 = conf
            .general_section()
            .get("indexcontours")
            .unwrap_or("12.5")
            .parse::<f64>()
            .unwrap_or(12.5);
        let formline: f64 = conf
            .general_section()
            .get("formline")
            .unwrap_or("2")
            .parse::<f64>()
            .unwrap_or(2.0);

        let depression_length: usize = conf
            .general_section()
            .get("depression_length")
            .unwrap_or("181")
            .parse::<usize>()
            .unwrap_or(181);
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
            skipknolldetection,
            vegemode,
            xfactor,
            yfactor,
            zfactor,
            contour_interval,
            basemapcontours,
            detectbuildings,
            water_class,
            inidotknolls,
            smoothing,
            curviness,
            indexcontours,
            formline,
            depression_length,
        })
    }
}
