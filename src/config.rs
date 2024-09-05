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

    // merge
    pub inidotknolls: f64,
    pub smoothing: f64,
    pub curviness: f64,
    pub indexcontours: f64,
    pub formline: f64,
    pub depression_length: usize,

    // cliffs
    pub c1_limit: f64,
    pub c2_limit: f64,
    pub cliff_thin: f64,
    pub steep_factor: f64,
    pub flat_place: f64,
    pub no_small_ciffs: f64,

    // vegetation
    pub zones: Vec<String>,               // TODO
    pub thresholds: Vec<(f64, f64, f64)>, //TODO
    pub greenshades: Vec<f64>,
    pub yellowheight: f64,
    pub yellowthreshold: f64,
    pub greenground: f64,
    pub pointvolumefactor: f64,
    pub pointvolumeexponent: f64,
    pub greenhigh: f64,
    pub topweight: f64,
    pub greentone: f64,
    pub zoffset: f64,
    pub uglimit: f64,
    pub uglimit2: f64,
    pub addition: i32,
    pub firstandlastreturnasground: u64,
    pub firstandlastfactor: f64,
    pub lastfactor: f64,
    pub yellowfirstlast: u64,
    pub vegethin: u32,
    pub greendetectsize: f64,
    pub proceed_yellows: bool,
    pub med: u32,
    pub med2: u32,
    pub water: u64,
    pub buildings: u64,
    pub waterele: f64,

    // render
    pub buildingcolor: Vec<String>,
    pub vectorconf: String,
    pub mtkskiplayers: Vec<String>,
    pub cliffdebug: bool,

    pub formlinesteepness: f64,
    // pub formline: f64,
    pub formlineaddition: f64,
    pub dashlength: f64,
    pub gaplength: f64,
    pub minimumgap: u32,
    pub label_depressions: bool,
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

        let proc: u64 = gs.get("processes").unwrap().parse::<u64>().unwrap();

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

        let inidotknolls: f64 = gs
            .get("knolls")
            .unwrap_or("0.8")
            .parse::<f64>()
            .unwrap_or(0.8);
        let smoothing: f64 = gs
            .get("smoothing")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        let curviness: f64 = gs
            .get("curviness")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        let indexcontours: f64 = gs
            .get("indexcontours")
            .unwrap_or("12.5")
            .parse::<f64>()
            .unwrap_or(12.5);
        let formline: f64 = gs
            .get("formline")
            .unwrap_or("2")
            .parse::<f64>()
            .unwrap_or(2.0);

        let depression_length: usize = gs
            .get("depression_length")
            .unwrap_or("181")
            .parse::<usize>()
            .unwrap_or(181);

        // cliffs
        let c1_limit: f64 = gs
            .get("cliff1")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        let c2_limit: f64 = gs
            .get("cliff2")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);

        let cliff_thin: f64 = gs
            .get("cliffthin")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);

        let steep_factor: f64 = gs
            .get("cliffsteepfactor")
            .unwrap_or("0.33")
            .parse::<f64>()
            .unwrap_or(0.33);

        let flat_place: f64 = gs
            .get("cliffflatplace")
            .unwrap_or("6.6")
            .parse::<f64>()
            .unwrap_or(6.6);

        let no_small_ciffs: f64 = gs
            .get("cliffnosmallciffs")
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);

        // vegetation

        let mut zones = vec![];
        let mut i: u32 = 1;
        loop {
            let last_zone = conf
                .general_section()
                .get(format!("zone{}", i))
                .unwrap_or("");
            if last_zone.is_empty() {
                break;
            }
            zones.push(last_zone.into());
            i += 1;
        }
        let thresholds = {
            let mut thresholds = vec![];
            let mut i: u32 = 1;
            loop {
                let last_threshold = conf
                    .general_section()
                    .get(format!("thresold{}", i))
                    .unwrap_or("");
                if last_threshold.is_empty() {
                    break;
                }
                // parse the threshold values
                let mut parts = last_threshold.split('|');
                let v0: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                let v1: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                let v2: f64 = parts.next().unwrap().parse::<f64>().unwrap();

                thresholds.push((v0, v1, v2));
                i += 1;
            }
            thresholds
        };

        let greenshades = conf
            .general_section()
            .get("greenshades")
            .unwrap_or("")
            .split('|')
            .map(|v| v.parse::<f64>().unwrap())
            .collect::<Vec<f64>>();
        let yellowheight: f64 = conf
            .general_section()
            .get("yellowheight")
            .unwrap_or("0.9")
            .parse::<f64>()
            .unwrap_or(0.9);
        let yellowthreshold: f64 = conf
            .general_section()
            .get("yellowthresold")
            .unwrap_or("0.9")
            .parse::<f64>()
            .unwrap_or(0.9);
        let greenground: f64 = conf
            .general_section()
            .get("greenground")
            .unwrap_or("0.9")
            .parse::<f64>()
            .unwrap_or(0.9);
        let pointvolumefactor: f64 = conf
            .general_section()
            .get("pointvolumefactor")
            .unwrap_or("0.1")
            .parse::<f64>()
            .unwrap_or(0.1);
        let pointvolumeexponent: f64 = conf
            .general_section()
            .get("pointvolumeexponent")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        let greenhigh: f64 = conf
            .general_section()
            .get("greenhigh")
            .unwrap_or("2")
            .parse::<f64>()
            .unwrap_or(2.0);
        let topweight: f64 = conf
            .general_section()
            .get("topweight")
            .unwrap_or("0.8")
            .parse::<f64>()
            .unwrap_or(0.8);
        let greentone: f64 = conf
            .general_section()
            .get("lightgreentone")
            .unwrap_or("200")
            .parse::<f64>()
            .unwrap_or(200.0);
        let zoffset: f64 = conf
            .general_section()
            .get("vegezoffset")
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);
        let uglimit: f64 = conf
            .general_section()
            .get("undergrowth")
            .unwrap_or("0.35")
            .parse::<f64>()
            .unwrap_or(0.35);
        let uglimit2: f64 = conf
            .general_section()
            .get("undergrowth2")
            .unwrap_or("0.56")
            .parse::<f64>()
            .unwrap_or(0.56);
        let addition: i32 = conf
            .general_section()
            .get("greendotsize")
            .unwrap_or("0")
            .parse::<i32>()
            .unwrap_or(0);
        let firstandlastreturnasground = conf
            .general_section()
            .get("firstandlastreturnasground")
            .unwrap_or("")
            .parse::<u64>()
            .unwrap_or(1);
        let firstandlastfactor = conf
            .general_section()
            .get("firstandlastreturnfactor")
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);
        let lastfactor = conf
            .general_section()
            .get("lastreturnfactor")
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);

        let yellowfirstlast = conf
            .general_section()
            .get("yellowfirstlast")
            .unwrap_or("")
            .parse::<u64>()
            .unwrap_or(1);
        let vegethin: u32 = conf
            .general_section()
            .get("vegethin")
            .unwrap_or("0")
            .parse::<u32>()
            .unwrap_or(0);

        let greendetectsize: f64 = conf
            .general_section()
            .get("greendetectsize")
            .unwrap_or("3")
            .parse::<f64>()
            .unwrap_or(3.0);
        let proceed_yellows: bool = conf
            .general_section()
            .get("yellow_smoothing")
            .unwrap_or("0")
            == "1";
        let med: u32 = conf
            .general_section()
            .get("medianboxsize")
            .unwrap_or("0")
            .parse::<u32>()
            .unwrap_or(0);
        let med2: u32 = conf
            .general_section()
            .get("medianboxsize2")
            .unwrap_or("0")
            .parse::<u32>()
            .unwrap_or(0);
        let water = conf
            .general_section()
            .get("waterclass")
            .unwrap_or("")
            .parse::<u64>()
            .unwrap_or(0);
        let buildings = conf
            .general_section()
            .get("buildingsclass")
            .unwrap_or("")
            .parse::<u64>()
            .unwrap_or(0);
        let waterele = conf
            .general_section()
            .get("waterelevation")
            .unwrap_or("")
            .parse::<f64>()
            .unwrap_or(-999999.0);

        // render
        let buildingcolor: Vec<String> = conf
            .general_section()
            .get("buildingcolor")
            .unwrap_or("0,0,0")
            .split(',')
            .map(Into::into)
            .collect();
        let vectorconf = conf
            .general_section()
            .get("vectorconf")
            .unwrap_or("")
            .into();
        let mtkskiplayers: Vec<String> = conf
            .general_section()
            .get("mtkskiplayers")
            .unwrap_or("")
            .split(',')
            .map(Into::into)
            .collect();

        let cliffdebug: bool = conf.general_section().get("cliffdebug").unwrap_or("0") == "1";

        let formlinesteepness: f64 = conf
            .general_section()
            .get("formlinesteepness")
            .unwrap_or("0.37")
            .parse::<f64>()
            .unwrap_or(0.37);
        let formlineaddition: f64 = conf
            .general_section()
            .get("formlineaddition")
            .unwrap_or("13")
            .parse::<f64>()
            .unwrap_or(13.0);
        let dashlength: f64 = conf
            .general_section()
            .get("dashlength")
            .unwrap_or("60")
            .parse::<f64>()
            .unwrap_or(60.0);
        let gaplength: f64 = conf
            .general_section()
            .get("gaplength")
            .unwrap_or("12")
            .parse::<f64>()
            .unwrap_or(12.0);
        let minimumgap: u32 = conf
            .general_section()
            .get("minimumgap")
            .unwrap_or("30")
            .parse::<u32>()
            .unwrap_or(30);
        let label_depressions: bool = conf
            .general_section()
            .get("label_formlines_depressions")
            .unwrap_or("0")
            == "1";
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
            c1_limit,
            c2_limit,
            cliff_thin,
            steep_factor,
            flat_place,
            no_small_ciffs,
            zones,
            thresholds,
            greenshades,
            yellowheight,
            yellowthreshold,
            greenground,
            pointvolumefactor,
            pointvolumeexponent,
            greenhigh,
            topweight,
            greentone,
            zoffset,
            uglimit,
            uglimit2,
            addition,
            firstandlastreturnasground,
            firstandlastfactor,
            lastfactor,
            yellowfirstlast,
            vegethin,
            greendetectsize,
            proceed_yellows,
            med,
            med2,
            water,
            buildings,
            waterele,
            buildingcolor,
            vectorconf,
            mtkskiplayers,
            cliffdebug,
            formlinesteepness,
            formlineaddition,
            dashlength,
            gaplength,
            minimumgap,
            label_depressions,
        })
    }
}
