use image::{GrayImage, Luma, Rgb, RgbImage, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_line_segment_mut};
use ini::Ini;
use las::{raw::Header, Read, Reader};
use pullauta::util::read_lines;
use rand::distributions;
use rand::prelude::*;
use regex::Regex;
use rustc_hash::FxHashMap as HashMap;
use shapefile::dbase::{FieldValue, Record};
use shapefile::{Shape, ShapeType};
use std::env;
use std::error::Error;
use std::f64::consts::PI;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::{thread, time};

mod canvas;
use canvas::Canvas;

fn main() {
    let mut thread: String = String::new();
    if !Path::new("pullauta.ini").exists() {
        let f =
            File::create(Path::new(&"pullauta.ini".to_string())).expect("Unable to create file");
        let mut f = BufWriter::new(f);
        f.write_all("#------------------------------------------------------#
# Parameters for the Karttapullautin pullautus process #
#----------------------------------------------------- #

################## PARAMETERS #############################
# Experimental undergrowth parameters. Smaller figures will give more undergrowth stripes
# normal undergrowth 
undergrowth=0.35

# undergrowth walk
undergrowth2=0.56

# Note, you will need to iterate this if you use this mode. with commands 'pullauta makevegenew' and then 'pullauta' you can process only this part again. 
# Elevation for hits below green. For green mapping hits below this will be calculated as points gone trough vegetation ~ ground.
greenground=0.9
greenhigh=2
topweight=0.80
vegezoffset=0
greendetectsize=3

### Here we calculate points. We can use elevation zones and factors for green. Example:
# low|high|roof|factor
# zone1=1|5|99|1  # points 1 to 5 meters will be calculates as one hit if tallest trees there as lower than 99 moters high 
# zone2=5|9|11.0|0.75 # in additon, poitns 5 to 9 meters will be calculated as 0.75 point's worth if tallest trees are lower than 11 meters.
# There can be as many zones as you like

# low|high|roof|factor
zone1=1.0|2.65|99|1
zone2=2.65|3.4|99|0.1
zone3=3.4|5.5|8|0.2


## Here we fine how sensitively we get green for different (hight or low) forest types. 
# For example tf tall forest with big trees gets too green compared to low forest, we can here tune it right. 
# roof low|roof high| greenhits/ground ratio to trigger green factor 1
thresold1=0.20|3|0.1
thresold2=3|4|0.1  
thresold3=4|7|0.1
thresold4=7|20|0.1
thresold5=20|99|0.1

## areas where scanning lines overlap we have two or three times bigger point density. That may make those areas more or less green. Use these parameters to balance it. 
# formula is:    * (1-pointvolumefactor * mydensity/averagedensity) ^ pointvolumeexponent
# so pointvolumefactor = 0 gives no balancing/effect

pointvolumefactor=0.1
pointvolumeexponent=1 

# green weighting if point is the only return - these are usually boulders or such 
# so these are only partly counted
firstandlastreturnfactor=1

# green weighting for last return - these may be vegetation but less likely that earlier returns
lastreturnfactor =1

firstandlastreturnasground=3
# green values for triggering green shades. Use high number like 99 to avoid some of the shades.
#greenshades=0.0|0.1|0.2|0.3|0.4|0.5|0.6|0.7|0.8|0.9|1.0|1.1|1.2|1.3|1.4|1.5|1.6|1.7|1.8|1.9|2.0|2.1|2.2|2.3|2.4|2.5|2.6|2.7|2.8|2.9|3.0

greenshades=0.2|0.35|0.5|0.7|1.3|2.6|4|99|99|99|99

# tone for the lightest green. 255 is white.
lightgreentone=200

# dont change this now
greendotsize=0

# block size for calculating hits-below-green ratio. use 3 if  greendetectsize is smaller than 5, if 
# it is bigger then use 1
groundboxsize=1

# green raster image filtering with median filter. Two rounds
# use 1 to do no filtering.
medianboxsize=9
medianboxsize2=1

## yellow parameters
### hits below this will be calculated as yellow
yellowheight=0.9  

### how big part or the points must be below yellowheight to trigger yellow
yellowthresold=0.9

#############################################
## cliff maker min height values for each cliff type. vertical drop per 1 meter horisontal distance
##  cliff1 = these cliffs will be erased if steepness is bigger than steepness value below
##  cliff2 = impassable cliff

cliff1 = 1.15
cliff2 = 2.0
cliffthin=1

cliffsteepfactor=0.38
cliffflatplace=3.5
cliffnosmallciffs=5.5

cliffdebug=0
## north lines rotation angle (clockwise) and width. Width 0 means no northlines.
northlinesangle=0
northlineswidth=0

## Form line mode, options:
# 0 = 2.5m interval, no formlines
# 1 = 2.5m interval, every second contour thin/thick
# 2 = 5m interval, with some dashed form lines in between if needed 

formline=2

# steepness parameter for form lines. Greater value gives more and smaller value gives less form lines. 
formlinesteepness=0.37

## additional lengt of form lines in vertex points
formlineaddition=17

## shortest gap in between form line ends in vertex points
minimumgap = 30

# dash and gap parameters for form lines
dashlength = 60 
gaplength =12

# interval for index contours. Used only if form line mode is 0
indexcontours=12.5

# smoothing contrors. Bigger value smoothes contours more. Default =1. Try values about between 0.5 and 3.0
smoothing = 0.7

# curviness. How curvy contours show up. default=1. Bigger value makes more curvy/exaggerated curves (reentrants and spurs)
curviness=1.1

# knoll qualification. default =0.8. range 0.0 ... 1.0  Bigger values gives less but more distinct knolls.
knolls=0.6

# xyz factors, for feet to meter conversion etc
coordxfactor=1
coordyfactor=1
coordzfactor=1

# las/laz to xyz thinning factor. For example 0.25 leaves 25% of points
thinfactor = 1

# if water classified points, this class will be drawn with blue (uncomment to enable this)
# waterclass=9

# Water eleveation, elevation lower than this gets drawn with blue (uncomment to enable this)
# waterelevation=0.15

# if buildings classified, this class will be drawn with black (uncomment to enable this)
# buildingsclass=6

# building detection. 1=on, 0=off. These will be drawn as purple with black edges. Highly experimental.
detectbuildings=0

# batch process mode, process all laz ans las files of this directory
# off=0, on=1  
batch=0

# processes
processes=2

# batch process output folder
batchoutfolder=./out

# batch process input file folder
lazfolder=./in

# If you can't get relative paths work, try absolute paths like c:/yourfolder/lasfiles

# Karttapullautin can render vector shape files.
# Maastotietokanta by National land survey of Finland does not need configuration file. For rendering those leave this parameter empty.
# For other datasets like Fastighetskartan from Lantmateriet (Sweden) configuration file is needed.

vectorconf=
# vectorconf=osm.txt
# vectorconf=fastighetskartan.txt

# shape files should be in zip files and placed in batch input folder or zip 
# should drag-dropped on pullauta.exe

# maastotietokanta, do not render these levels, comma delimined
mtkskiplayers=

# uncomment this for no settlements color (skip these layers Pullautin usually draws with olive green)
# mtkskiplayers=32000,40200,62100,32410,32411,32412,32413,32414,32415,32416,32417,32418

# Color for vector buildings (RGB value 0,0,0 is black and 255,255,255 is white)
buildingcolor=0,0,0

# in bach mode, will we crop and copy also some temp files to output folder
#  folder.  1=on 0 = off. use this if you want to use vector contors and such for each tile.
    
savetempfiles=0

# in batch mode will we save the whole temp directory as it is
savetempfolders=0
            
# the interval of additonal dxf contour layer (raw, for mapping). 0 = disabled. Value 1.125 gives such interval contours 
basemapinterval=0 

# Experimental parameters. Dont chance these unless you feel like experimenting
scalefactor=1
zoffset=0
#skipknolldetection=0

##################################################################################################################################
# Settings specific to the rust version of karttapulautin, default values are meant to be fidel to original Perl Karttapullautin #
##################################################################################################################################
# jarkkos2019, set to 0 to fix the obvious bugs of the Perl KarttaPullautin found in the source code of the 20190203 version, 1 reproduce the buggy behaviours
jarkkos2019=1

# contour_interval sets the contours interval in meters for the output map
contour_interval=5

# depression_length sets the maximum length of the depressions to be marked. Original from Perl version is hardcoded to 181.
# set a very large number if all depressions should be marked. 
depression_length=181

# yellow_smoothing, set to 1 to apply a smoothing effect on the yellow areas matching the smoothing of the green areas
yellow_smoothing=0

# vege_bitmode, set to 1 to output a bit
vege_bitmode=0

# label_formlines_depressions, set to 1 to add a seperate label on the depressions in the formlines vector file
label_formlines_depressions=0

# vegeonly, set to 1 to only process the vegetation and skip the contour processing
vegeonly=0
".as_bytes()).expect("Cannot write file");
    }

    let conf = Ini::load_from_file("pullauta.ini").unwrap();

    let int_re = Regex::new(r"^[1-9]\d*$").unwrap();

    let mut args: Vec<String> = env::args().collect();

    args.remove(0); // program name

    if !args.is_empty() && int_re.is_match(&args[0]) {
        thread = args.remove(0);
    }

    let mut command: String = String::new();
    if !args.is_empty() {
        command = args.remove(0);
    }

    let accepted_files_re = Regex::new(r"\.(las|laz|xyz)$").unwrap();
    if command.is_empty() || accepted_files_re.is_match(&command.to_lowercase()) {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        println!(
            "Karttapullautin v{}\nThere is no warranty. Use it at your own risk!\n",
            VERSION
        );
    }

    let batch: bool = conf.general_section().get("batch").unwrap() == "1";

    let tmpfolder = format!("temp{}", thread);
    fs::create_dir_all(&tmpfolder).expect("Could not create tmp folder");
    let pnorthlinesangle: f64 = conf
        .general_section()
        .get("northlinesangle")
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    let pnorthlineswidth: usize = conf
        .general_section()
        .get("northlineswidth")
        .unwrap_or("0")
        .parse::<usize>()
        .unwrap_or(0);

    if command.is_empty() && Path::new(&format!("{}/vegetation.png", tmpfolder)).exists() && !batch
    {
        println!("Rendering png map with depressions");
        render(&thread, pnorthlinesangle, pnorthlineswidth, false).unwrap();
        println!("Rendering png map without depressions");
        render(&thread, pnorthlinesangle, pnorthlineswidth, true).unwrap();
        println!("\nAll done!");
        return;
    }

    if command.is_empty() && !batch {
        println!("USAGE:\npullauta [parameter 1] [parameter 2] [parameter 3] ... [parameter n]\nSee README.MD for more details");
        return;
    }

    if command == "cliffgeneralize" {
        println!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "ground" {
        println!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "ground2" {
        println!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "groundfix" {
        println!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "profile" {
        println!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "makecliffsold" {
        println!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "makeheight" {
        println!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "makevege" {
        println!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "xyzfixer" {
        println!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "vege" {
        println!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "blocks" {
        pullauta::blocks::blocks(&thread).unwrap();
        return;
    }

    if command == "dotknolls" {
        pullauta::knolls::dotknolls(&thread).unwrap();
        return;
    }

    if command == "dxfmerge" || command == "merge" {
        pullauta::merge::dxfmerge().unwrap();
        if command == "merge" {
            let mut scale = 1.0;
            if !args.is_empty() {
                scale = args[0].parse::<f64>().unwrap();
            }
            pullauta::merge::pngmergevege(scale).unwrap();
        }
        return;
    }

    if command == "knolldetector" {
        pullauta::knolls::knolldetector(&thread).unwrap();
        return;
    }

    if command == "makecliffs" {
        pullauta::cliffs::makecliffs(&thread).unwrap();
        return;
    }

    if command == "makevegenew" {
        pullauta::vegetation::makevegenew(&thread).unwrap();
    }

    if command == "pngmerge" || command == "pngmergedepr" {
        let mut scale = 4.0;
        if !args.is_empty() {
            scale = args[0].parse::<f64>().unwrap();
        }
        pullauta::merge::pngmerge(scale, command == "pngmergedepr").unwrap();
        return;
    }

    if command == "pngmergevege" {
        let mut scale = 1.0;
        if !args.is_empty() {
            scale = args[0].parse::<f64>().unwrap();
        }
        pullauta::merge::pngmergevege(scale).unwrap();
        return;
    }

    if command == "polylinedxfcrop" {
        let dxffilein = Path::new(&args[0]);
        let dxffileout = Path::new(&args[1]);
        let minx = args[2].parse::<f64>().unwrap();
        let miny = args[3].parse::<f64>().unwrap();
        let maxx = args[4].parse::<f64>().unwrap();
        let maxy = args[5].parse::<f64>().unwrap();
        polylinedxfcrop(dxffilein, dxffileout, minx, miny, maxx, maxy).unwrap();
        return;
    }

    if command == "pointdxfcrop" {
        let dxffilein = Path::new(&args[0]);
        let dxffileout = Path::new(&args[1]);
        let minx = args[2].parse::<f64>().unwrap();
        let miny = args[3].parse::<f64>().unwrap();
        let maxx = args[4].parse::<f64>().unwrap();
        let maxy = args[5].parse::<f64>().unwrap();
        pointdxfcrop(dxffilein, dxffileout, minx, miny, maxx, maxy).unwrap();
        return;
    }

    if command == "smoothjoin" {
        pullauta::merge::smoothjoin(&thread).unwrap();
    }

    if command == "xyzknolls" {
        pullauta::knolls::xyzknolls(&thread).unwrap();
    }

    if command == "unzipmtk" {
        unzipmtk(&thread, &args).unwrap();
    }

    if command == "mtkshaperender" {
        mtkshaperender(&thread).unwrap();
    }

    if command == "xyz2contours" {
        let cinterval: f64 = args[0].parse::<f64>().unwrap();
        let xyzfilein = args[1].clone();
        let xyzfileout = args[2].clone();
        let dxffile = args[3].clone();
        let mut ground: bool = false;
        if args.len() > 4 && args[4] == "ground" {
            ground = true;
        }
        pullauta::contours::xyz2contours(
            &thread,
            cinterval,
            &xyzfilein,
            &xyzfileout,
            &dxffile,
            ground,
        )
        .unwrap();
        return;
    }

    if command == "render" {
        let angle: f64 = args[0].parse::<f64>().unwrap();
        let nwidth: usize = args[1].parse::<usize>().unwrap();
        let nodepressions: bool = args.len() > 2 && args[2] == "nodepressions";
        render(&thread, angle, nwidth, nodepressions).unwrap();
        return;
    }

    let proc: u64 = conf
        .general_section()
        .get("processes")
        .unwrap()
        .parse::<u64>()
        .unwrap();
    if command.is_empty() && batch && proc > 1 {
        let mut handles: Vec<thread::JoinHandle<()>> = Vec::with_capacity((proc + 1) as usize);
        for i in 0..proc {
            let handle = thread::spawn(move || {
                println!("Starting thread {}", i + 1);
                batch_process(&format!("{}", i + 1));
                println!("Thread {} complete", i + 1);
            });
            thread::sleep(time::Duration::from_millis(100));
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
        return;
    }

    if (command.is_empty() && batch && proc < 2) || (command == "startthread" && batch) {
        thread = String::from("0");
        if !args.is_empty() {
            thread.clone_from(&args[0]);
        }
        if thread == "0" {
            thread = String::from("");
        }
        batch_process(&thread)
    }

    let zip_files_re = Regex::new(r"\.zip$").unwrap();
    if zip_files_re.is_match(&command.to_lowercase()) {
        let mut zips: Vec<String> = vec![command];
        zips.extend(args);
        process_zip(&thread, &zips).unwrap();
        return;
    }

    if accepted_files_re.is_match(&command.to_lowercase()) {
        let mut norender: bool = false;
        if args.len() > 1 {
            norender = args[1].clone() == "norender";
        }
        process_tile(&thread, &command, norender).unwrap();
    }
}

fn process_zip(thread: &String, filenames: &Vec<String>) -> Result<(), Box<dyn Error>> {
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let pnorthlinesangle: f64 = conf
        .general_section()
        .get("northlinesangle")
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    let pnorthlineswidth: usize = conf
        .general_section()
        .get("northlineswidth")
        .unwrap_or("0")
        .parse::<usize>()
        .unwrap_or(0);

    println!("Rendering shape files");
    unzipmtk(thread, filenames).unwrap();
    println!("Rendering png map with depressions");
    render(thread, pnorthlinesangle, pnorthlineswidth, false).unwrap();
    println!("Rendering png map without depressions");
    render(thread, pnorthlinesangle, pnorthlineswidth, true).unwrap();
    Ok(())
}

fn unzipmtk(thread: &String, filenames: &Vec<String>) -> Result<(), Box<dyn Error>> {
    if Path::new(&format!("temp{}/low.png", thread)).exists() {
        fs::remove_file(format!("temp{}/low.png", thread)).unwrap();
    }
    if Path::new(&format!("temp{}/high.png", thread)).exists() {
        fs::remove_file(format!("temp{}/high.png", thread)).unwrap();
    }

    for zip_name in filenames.iter() {
        let fname = Path::new(&zip_name);
        let file = fs::File::open(fname).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        archive
            .extract(Path::new(&format!("temp{}/", thread)))
            .unwrap();
        mtkshaperender(thread).unwrap();
    }
    Ok(())
}

fn mtkshaperender(thread: &String) -> Result<(), Box<dyn Error>> {
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let scalefactor: f64 = conf
        .general_section()
        .get("scalefactor")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);
    let buildingcolor: Vec<&str> = conf
        .general_section()
        .get("buildingcolor")
        .unwrap_or("0,0,0")
        .split(',')
        .collect();
    let vectorconf = conf.general_section().get("vectorconf").unwrap_or("");
    let mtkskip: Vec<&str> = conf
        .general_section()
        .get("mtkstip")
        .unwrap_or("")
        .split(',')
        .collect();
    let mut vectorconf_lines: Vec<String> = vec![];
    if !vectorconf.is_empty() {
        let vectorconf_data =
            fs::read_to_string(Path::new(&vectorconf)).expect("Can not read input file");
        vectorconf_lines = vectorconf_data
            .split('\n')
            .collect::<Vec<&str>>()
            .iter()
            .map(|x| x.to_string())
            .collect();
    }
    let tmpfolder = format!("temp{}", thread);
    if !Path::new(&format!("{}/vegetation.pgw", &tmpfolder)).exists() {
        println!("Could not find vegetation file");
        return Ok(());
    }

    let pgw = format!("{}/vegetation.pgw", tmpfolder);
    let input = Path::new(&pgw);
    let data = fs::read_to_string(input).expect("Can not read input file");
    let d: Vec<&str> = data.split('\n').collect();

    let x0 = d[4].trim().parse::<f64>().unwrap();
    let y0 = d[5].trim().parse::<f64>().unwrap();
    // let resvege = d[0].trim().parse::<f64>().unwrap();

    let mut img_reader =
        image::io::Reader::open(Path::new(&format!("{}/vegetation.png", tmpfolder)))
            .expect("Opening vegetation image failed");
    img_reader.no_limits();
    let img = img_reader.decode().unwrap();
    let w = img.width() as f64;
    let h = img.height() as f64;

    let outw = w * 600.0 / 254.0 / scalefactor;
    let outh = h * 600.0 / 254.0 / scalefactor;

    // let mut img2 = Canvas::new(outw as i32, outh as i32);
    let mut imgbrown = Canvas::new(outw as i32, outh as i32);
    let mut imgbrowntop = Canvas::new(outw as i32, outh as i32);
    let mut imgblack = Canvas::new(outw as i32, outh as i32);
    let mut imgblacktop = Canvas::new(outw as i32, outh as i32);
    let mut imgyellow = Canvas::new(outw as i32, outh as i32);
    let mut imgblue = Canvas::new(outw as i32, outh as i32);
    let mut imgmarsh = Canvas::new(outw as i32, outh as i32);
    let mut imgtempblack = Canvas::new(outw as i32, outh as i32);
    let mut imgtempblacktop = Canvas::new(outw as i32, outh as i32);
    let mut imgblue2 = Canvas::new(outw as i32, outh as i32);

    let white = (255, 255, 255);
    let unsetcolor = (5, 255, 255);
    let black = (0, 0, 0);
    let brown = (255, 150, 80);

    let purple = (
        buildingcolor[0].parse::<u8>().unwrap_or(0),
        buildingcolor[1].parse::<u8>().unwrap_or(0),
        buildingcolor[2].parse::<u8>().unwrap_or(0),
    );
    let yellow = (255, 184, 83);
    let blue = (29, 190, 255);
    let marsh = (0, 10, 220);
    let olive = (194, 176, 33);

    let mut shp_files: Vec<PathBuf> = Vec::new();
    for element in Path::new(&tmpfolder).read_dir().unwrap() {
        let path = element.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "shp" {
                shp_files.push(path);
            }
        }
    }

    for shp_file in shp_files.iter() {
        let file = shp_file.as_path().file_name().unwrap().to_str().unwrap();
        let file = format!("{}/{}", tmpfolder, file);

        // drawshape comes here
        let mut reader = shapefile::Reader::from_path(&file)?;
        for shape_record in reader.iter_shapes_and_records() {
            let (shape, record) = shape_record
                .unwrap_or_else(|_err: shapefile::Error| (Shape::NullShape, Record::default()));

            let mut area = false;
            let mut roadedge = 0.0;
            let mut edgeimage = "black";
            let mut image = "";
            let mut thickness = 1.0;
            let mut vari = unsetcolor;
            let mut dashedline = false;
            let mut border = 0.0;

            if vectorconf.is_empty() {
                // MML shape file
                let mut luokka = String::new();
                if let Some(fv) = record.get("LUOKKA") {
                    if let FieldValue::Numeric(Some(f_luokka)) = fv {
                        luokka = format!("{}", f_luokka);
                    }
                    if let FieldValue::Character(Some(c_luokka)) = fv {
                        luokka = c_luokka.to_string();
                    }
                }
                let mut versuh = 0.0;
                if let Some(fv) = record.get("VERSUH") {
                    if let FieldValue::Numeric(Some(f_versuh)) = fv {
                        versuh = *f_versuh;
                    }
                }
                // water streams
                if ["36311", "36312"].contains(&luokka.as_str()) {
                    thickness = 4.0;
                    vari = marsh;
                    image = "blue";
                }

                // pathes
                if luokka == "12316" && versuh != -11.0 {
                    thickness = 12.0;
                    dashedline = true;
                    image = "black";
                    vari = black;
                    if versuh > 0.0 {
                        image = "blacktop";
                    }
                }

                // large pathes
                if (luokka == "12141" || luokka == "12314") && versuh != 11.0 {
                    thickness = 12.0;
                    image = "black";
                    vari = black;
                    if versuh > 0.0 {
                        image = "blacktop";
                    }
                }

                // roads
                if ["12111", "12112", "12121", "12122", "12131", "12132"].contains(&luokka.as_str())
                    && versuh != 11.0
                {
                    imgbrown.set_line_width(20.0);
                    imgbrowntop.set_line_width(20.0);
                    thickness = 20.0;
                    vari = brown;
                    image = "brown";
                    roadedge = 26.0;
                    imgblack.set_line_width(26.0);
                    if versuh > 0.0 {
                        edgeimage = "blacktop";
                        imgbrown.set_line_width(14.0);
                        imgbrowntop.set_line_width(14.0);
                        thickness = 14.0;
                    }
                }

                // railroads
                if ["14110", "14111", "14112", "14121", "14131"].contains(&luokka.as_str())
                    && versuh != 11.0
                {
                    image = "black";
                    vari = white;
                    thickness = 3.0;
                    roadedge = 18.0;
                    if versuh > 0.0 {
                        image = "blacktop";
                        edgeimage = "blacktop";
                    }
                }

                if luokka == "12312" && versuh != 11.0 {
                    dashedline = true;
                    thickness = 6.0;
                    image = "black";
                    vari = black;
                    if versuh > 0.0 {
                        image = "blacktop";
                    }
                }

                if luokka == "12313" && versuh != 11.0 {
                    dashedline = true;
                    thickness = 5.0;
                    image = "black";
                    vari = black;
                    if versuh > 0.0 {
                        image = "blacktop";
                    }
                }

                // power line
                if ["22300", "22312", "44500", "223311"].contains(&luokka.as_str()) {
                    imgblacktop.set_line_width(5.0);
                    thickness = 5.0;
                    vari = black;
                    image = "blacktop";
                }

                // fence
                if ["44211", "44213"].contains(&luokka.as_str()) {
                    imgblacktop.set_line_width(7.0);
                    thickness = 7.0;
                    vari = black;
                    image = "blacktop";
                }

                // Next are polygons

                // fields
                if luokka == "32611" {
                    area = true;
                    vari = yellow;
                    border = 3.0;
                    image = "yellow";
                }

                // lake
                if [
                    "36200", "36211", "36313", "38700", "44300", "45111", "54112",
                ]
                .contains(&luokka.as_str())
                {
                    area = true;
                    vari = blue;
                    border = 5.0;
                    image = "blue";
                }

                // impassable marsh
                if ["35421", "38300"].contains(&luokka.as_str()) {
                    area = true;
                    vari = marsh;
                    border = 3.0;
                    image = "marsh";
                }

                // regular marsh
                if ["35400", "35411"].contains(&luokka.as_str()) {
                    area = true;
                    vari = marsh;
                    border = 0.0;
                    image = "marsh";
                }

                // marshy
                if ["35300", "35412", "35422"].contains(&luokka.as_str()) {
                    area = true;
                    vari = marsh;
                    border = 0.0;
                    image = "marsh";
                }

                // marshy
                if [
                    "42210", "42211", "42212", "42220", "42221", "42222", "42230", "42231",
                    "42232", "42240", "42241", "42242", "42270", "42250", "42251", "42252",
                    "42260", "42261", "42262",
                ]
                .contains(&luokka.as_str())
                {
                    area = true;
                    vari = purple;
                    border = 0.0;
                    image = "black";
                }

                // settlement
                if [
                    "32000", "40200", "62100", "32410", "32411", "32412", "32413", "32414",
                    "32415", "32416", "32417", "32418",
                ]
                .contains(&luokka.as_str())
                {
                    area = true;
                    vari = olive;
                    border = 0.0;
                    image = "yellow";
                }

                // airport runway, car parkings
                if ["32411", "32412", "32415", "32417", "32421"].contains(&luokka.as_str()) {
                    area = true;
                    vari = brown;
                    border = 0.0;
                    image = "yellow";
                }

                if mtkskip.contains(&luokka.as_str()) {
                    vari = unsetcolor;
                }
            } else {
                // configuration based drawing
                for conf_row in vectorconf_lines.iter() {
                    let row_data: Vec<&str> = conf_row.trim().split('|').collect();
                    if row_data.len() < 3 {
                        continue;
                    }
                    let isom = row_data[1];
                    let mut keyvals: Vec<(String, String, String)> = vec![];
                    let params: Vec<&str> = row_data[2].split('&').collect();
                    for param in params {
                        let mut operator = "=";
                        let d: Vec<&str>;
                        if param.contains("!=") {
                            d = param.splitn(2, "!=").collect();
                            operator = "!=";
                        } else {
                            d = param.splitn(2, '=').collect();
                        }
                        keyvals.push((
                            operator.to_string(),
                            d[0].trim().to_string(),
                            d[1].trim().to_string(),
                        ))
                    }
                    if vari == unsetcolor {
                        if isom == "306" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                imgblue.set_line_width(5.0);
                                thickness = 4.0;
                                vari = marsh;
                                image = "blue";
                            }
                        }

                        // small path
                        if isom == "505" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                dashedline = true;
                                thickness = 12.0;
                                vari = black;
                                image = "black";
                            }
                        }

                        // small path top
                        if isom == "505T" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                dashedline = true;
                                thickness = 12.0;
                                vari = black;
                                image = "blacktop";
                            }
                        }

                        // large path
                        if isom == "504" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                imgblack.set_line_width(12.0);
                                thickness = 12.0;
                                vari = black;
                                image = "black";
                            }
                        }

                        // large path top
                        if isom == "504T" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                imgblack.set_line_width(12.0);
                                thickness = 12.0;
                                vari = black;
                                image = "blacktop";
                            }
                        }

                        // road
                        if isom == "503" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                imgbrown.set_line_width(20.0);
                                imgbrowntop.set_line_width(20.0);
                                vari = brown;
                                image = "brown";
                                roadedge = 26.0;
                                thickness = 20.0;
                                imgblack.set_line_width(26.0);
                            }
                        }

                        // road, bridges
                        if isom == "503T" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::new();
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                edgeimage = "blacktop";
                                imgbrown.set_line_width(14.0);
                                imgbrowntop.set_line_width(14.0);
                                vari = brown;
                                image = "brown";
                                roadedge = 26.0;
                                thickness = 14.0;
                                imgblack.set_line_width(26.0);
                            }
                        }

                        // railroads
                        if isom == "515" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                vari = white;
                                image = "black";
                                roadedge = 18.0;
                                thickness = 3.0;
                            }
                        }

                        // railroads top
                        if isom == "515T" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                vari = white;
                                image = "blacktop";
                                edgeimage = "blacktop";
                                roadedge = 18.0;
                                thickness = 3.0;
                            }
                        }

                        // small path
                        if isom == "507" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                dashedline = true;
                                vari = black;
                                image = "black";
                                thickness = 6.0;
                                imgblack.set_line_width(6.0);
                            }
                        }

                        // small path top
                        if isom == "507T" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                dashedline = true;
                                vari = black;
                                image = "blacktop";
                                thickness = 6.0;
                                imgblack.set_line_width(6.0);
                            }
                        }

                        // powerline
                        if isom == "516" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                vari = black;
                                image = "blacktop";
                                thickness = 5.0;
                                imgblacktop.set_line_width(5.0);
                            }
                        }

                        // fence
                        if isom == "524" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                vari = black;
                                image = "black";
                                thickness = 7.0;
                                imgblacktop.set_line_width(7.0);
                            }
                        }

                        // blackline
                        if isom == "414" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                vari = black;
                                image = "black";
                                thickness = 4.0;
                            }
                        }

                        // areas

                        // fields
                        if isom == "401" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                area = true;
                                vari = yellow;
                                border = 3.0;
                                image = "yellow";
                            }
                        }
                        // lakes
                        if isom == "301" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                area = true;
                                vari = blue;
                                border = 5.0;
                                image = "blue";
                            }
                        }
                        // marshes
                        if isom == "310" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                area = true;
                                vari = marsh;
                                image = "marsh";
                            }
                        }
                        // buildings
                        if isom == "526" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                area = true;
                                vari = purple;
                                image = "black";
                            }
                        }
                        // settlements
                        if isom == "527" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                area = true;
                                vari = olive;
                                image = "yellow";
                            }
                        }
                        // car parkings border
                        if isom == "529.1" || isom == "301.1" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                thickness = 2.0;
                                vari = black;
                                image = "black";
                            }
                        }
                        // car park area
                        if isom == "529" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                area = true;
                                vari = brown;
                                image = "yellow";
                            }
                        }
                        // car park top
                        if isom == "529T" {
                            let mut is_ok = true;
                            for keyval in keyvals.iter() {
                                let mut r = String::from("");
                                if let Some(recordfv) = record.get(&keyval.1) {
                                    if let FieldValue::Character(Some(record_str)) = recordfv {
                                        r = record_str.to_string().trim().to_string();
                                    }
                                }
                                if keyval.0 == "=" {
                                    if r != keyval.2 {
                                        is_ok = false;
                                    }
                                } else if r == keyval.2 {
                                    is_ok = false;
                                }
                            }
                            if is_ok {
                                area = true;
                                vari = brown;
                                image = "brown";
                            }
                        }
                    }
                }
            }

            if vari != unsetcolor {
                if !area && shape.shapetype() == ShapeType::Polyline {
                    let mut poly: Vec<(f32, f32)> = vec![];
                    let polyline = shapefile::Polyline::try_from(shape).unwrap();
                    for points in polyline.parts().iter() {
                        for point in points.iter() {
                            let x = point.x;
                            let y = point.y;
                            poly.push((
                                (600.0 / 254.0 / scalefactor * (x - x0)).floor() as f32,
                                (600.0 / 254.0 / scalefactor * (y0 - y)).floor() as f32,
                            ));
                        }
                    }
                    if roadedge > 0.0 {
                        if edgeimage == "blacktop" {
                            imgblacktop.unset_stroke_cap();
                            imgblacktop.set_line_width(roadedge);
                            imgblacktop.set_color(black);
                            imgblacktop.draw_polyline(&poly);
                            imgblacktop.set_line_width(thickness);
                        } else {
                            imgblack.set_color(black);
                            imgblack.set_stroke_cap_round();
                            imgblack.set_line_width(roadedge);
                            imgblack.draw_polyline(&poly);
                            imgblack.set_line_width(thickness);
                            imgblack.unset_stroke_cap();
                        }
                    }

                    if !dashedline {
                        if image == "blacktop" {
                            imgblacktop.set_line_width(thickness);
                            imgblacktop.set_color(vari);
                            if thickness >= 9.0 {
                                imgblacktop.set_stroke_cap_round();
                            }
                            imgblacktop.draw_polyline(&poly);
                            imgblacktop.unset_stroke_cap();
                        }
                        if image == "black" {
                            imgblack.set_line_width(thickness);
                            imgblack.set_color(vari);
                            if thickness >= 9.0 {
                                imgblack.set_stroke_cap_round();
                            } else {
                                imgblack.unset_stroke_cap();
                            }
                            imgblack.draw_polyline(&poly);
                        }
                    } else {
                        if image == "blacktop" {
                            let interval_on = 1.0 + thickness * 8.0;
                            imgtempblacktop.set_dash(interval_on, thickness * 1.6);
                            if thickness >= 9.0 {
                                imgtempblacktop.set_stroke_cap_round();
                            }
                            imgtempblacktop.set_color(vari);
                            imgtempblacktop.set_line_width(thickness);
                            imgtempblacktop.draw_polyline(&poly);
                            imgtempblacktop.unset_dash();
                            imgtempblacktop.unset_stroke_cap();
                        }
                        if image == "black" {
                            let interval_on = 1.0 + thickness * 8.0;
                            imgtempblack.set_dash(interval_on, thickness * 1.6);
                            if thickness >= 9.0 {
                                imgtempblack.set_stroke_cap_round();
                            }
                            imgtempblack.set_color(vari);
                            imgtempblack.set_line_width(thickness);
                            imgtempblack.draw_polyline(&poly);
                            imgtempblack.unset_dash();
                            imgtempblack.unset_stroke_cap();
                        }
                    }

                    if image == "blue" {
                        imgblue.set_color(vari);
                        imgblue.set_line_width(thickness);
                        imgblue.draw_polyline(&poly)
                    }

                    if image == "brown" {
                        if edgeimage == "blacktop" {
                            imgbrowntop.set_line_width(thickness);
                            imgbrowntop.set_color(brown);
                            imgbrowntop.draw_polyline(&poly);
                        } else {
                            imgbrown.set_stroke_cap_round();
                            imgbrown.set_line_width(thickness);
                            imgbrown.set_color(brown);
                            imgbrown.draw_polyline(&poly);
                            imgbrown.unset_stroke_cap();
                        }
                    }
                } else if area && shape.shapetype() == ShapeType::Polygon {
                    let mut polys: Vec<Vec<(f32, f32)>> = vec![];
                    let polygon = shapefile::Polygon::try_from(shape).unwrap();
                    for ring in polygon.rings().iter() {
                        let mut poly: Vec<(f32, f32)> = vec![];
                        let mut polyborder: Vec<(f32, f32)> = vec![];
                        for point in ring.points().iter() {
                            let x = point.x;
                            let y = point.y;
                            poly.push((
                                (600.0 / 254.0 / scalefactor * (x - x0)).floor() as f32,
                                (600.0 / 254.0 / scalefactor * (y0 - y)).floor() as f32,
                            ));
                            polyborder.push((
                                (600.0 / 254.0 / scalefactor * (x - x0)).floor() as f32,
                                (600.0 / 254.0 / scalefactor * (y0 - y)).floor() as f32,
                            ));
                        }
                        polys.push(poly);
                        if border > 0.0 {
                            imgblack.set_color(black);
                            imgblack.set_line_width(border);
                            imgblack.draw_closed_polyline(&polyborder);
                        }
                    }

                    if image == "black" {
                        imgblack.set_color(vari);
                        imgblack.draw_filled_polygon(&polys)
                    }
                    if image == "blue" {
                        imgblue.set_color(vari);
                        imgblue.draw_filled_polygon(&polys)
                    }
                    if image == "yellow" {
                        imgyellow.set_color(vari);
                        imgyellow.draw_filled_polygon(&polys)
                    }
                    if image == "marsh" {
                        imgmarsh.set_color(vari);
                        imgmarsh.draw_filled_polygon(&polys)
                    }
                    if image == "brown" {
                        imgbrown.set_color(vari);
                        imgbrown.draw_filled_polygon(&polys)
                    }
                }
            }
        }

        fs::remove_file(&file).unwrap();

        let re = Regex::new(r"\.shp$").unwrap();
        for ext in [".dbf", ".sbx", ".prj", ".shx", ".sbn", ".cpg"].iter() {
            let delshp = re.replace(&file, String::from(*ext)).into_owned();
            if Path::new(&delshp).exists() {
                fs::remove_file(&delshp).unwrap();
            }
        }
    }
    imgblue2.overlay(&mut imgblue, 0.0, 0.0);
    imgblue2.overlay(&mut imgblue, 1.0, 0.0);
    imgblue2.overlay(&mut imgblue, 0.0, 1.0);
    imgblue.overlay(&mut imgblue2, 0.0, 0.0);

    let mut i = 0.0_f32;
    imgmarsh.set_transparent_color();
    while i < ((h * 600.0 / 254.0 / scalefactor + 500.0) as f32) {
        i += 14.0;
        let wd = (w * 600.0 / 254.0 / scalefactor + 2.0) as f32;
        imgmarsh.draw_filled_polygon(&vec![vec![
            (-1.0, i),
            (wd, i),
            (wd, i + 10.0),
            (-1.0, i + 10.0),
            (-1.0, i),
        ]])
    }
    imgblacktop.overlay(&mut imgtempblacktop, 0.0, 0.0);
    imgblack.overlay(&mut imgtempblack, 0.0, 0.0);

    imgyellow.overlay(&mut imgmarsh, 0.0, 0.0);

    imgblue.overlay(&mut imgblack, 0.0, 0.0);
    imgblue.overlay(&mut imgbrown, 0.0, 0.0);
    imgblue.overlay(&mut imgblacktop, 0.0, 0.0);
    imgblue.overlay(&mut imgbrowntop, 0.0, 0.0);

    if Path::new(&format!("{}/low.png", tmpfolder)).exists() {
        let mut low = Canvas::load_from(&format!("{}/low.png", tmpfolder));
        imgyellow.overlay(&mut low, 0.0, 0.0);
    }

    if Path::new(&format!("{}/high.png", tmpfolder)).exists() {
        let mut high = Canvas::load_from(&format!("{}/high.png", tmpfolder));
        imgblue.overlay(&mut high, 0.0, 0.0);
    }
    imgblue.save_as(&format!("{}/high.png", tmpfolder));
    imgyellow.save_as(&format!("{}/low.png", tmpfolder));
    Ok(())
}

fn process_tile(
    thread: &String,
    filename: &str,
    skip_rendering: bool,
) -> Result<(), Box<dyn Error>> {
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let skipknolldetection = conf
        .general_section()
        .get("skipknolldetection")
        .unwrap_or("0")
        == "1";
    let tmpfolder = format!("temp{}", thread);
    fs::create_dir_all(&tmpfolder).expect("Could not create tmp folder");
    let pnorthlinesangle: f64 = conf
        .general_section()
        .get("northlinesangle")
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    let pnorthlineswidth: usize = conf
        .general_section()
        .get("northlineswidth")
        .unwrap_or("0")
        .parse::<usize>()
        .unwrap_or(0);

    let vegemode: bool = conf.general_section().get("vegemode").unwrap_or("0") == "1";
    if vegemode {
        println!("vegemode=1 not implemented, use perl version");
        return Ok(());
    }
    let mut thread_name = String::new();
    if !thread.is_empty() {
        thread_name = format!("Thread {}: ", thread);
    }
    println!("{}Preparing input file", thread_name);
    let mut skiplaz2txt: bool = false;
    if Regex::new(r".xyz$")
        .unwrap()
        .is_match(&filename.to_lowercase())
    {
        if let Ok(lines) = read_lines(Path::new(filename)) {
            let mut i: u32 = 0;
            for line in lines {
                if i == 2 {
                    let ip = line.unwrap_or(String::new());
                    let parts = ip.split(' ');
                    let r = parts.collect::<Vec<&str>>();
                    if r.len() == 7 {
                        skiplaz2txt = true;
                        break;
                    }
                }
                i += 1;
            }
        }
    }

    if !skiplaz2txt {
        let mut thinfactor: f64 = conf
            .general_section()
            .get("thinfactor")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        if thinfactor == 0.0 {
            thinfactor = 1.0;
        }
        if thinfactor != 1.0 {
            println!("{}Using thinning factor {}", thread_name, thinfactor);
        }

        let mut rng = rand::thread_rng();
        let randdist = distributions::Bernoulli::new(thinfactor).unwrap();

        let mut xfactor: f64 = conf
            .general_section()
            .get("coordxfactor")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        let mut yfactor: f64 = conf
            .general_section()
            .get("coordyfactor")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        let mut zfactor: f64 = conf
            .general_section()
            .get("coordzfactor")
            .unwrap_or("1")
            .parse::<f64>()
            .unwrap_or(1.0);
        let zoff: f64 = conf
            .general_section()
            .get("zoffset")
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);
        if xfactor == 0.0 {
            xfactor = 1.0;
        }
        if yfactor == 0.0 {
            yfactor = 1.0;
        }
        if zfactor == 0.0 {
            zfactor = 1.0;
        }

        let tmp_filename = format!("{}/xyztemp.xyz", tmpfolder);
        let tmp_file = File::create(tmp_filename).expect("Unable to create file");
        let mut tmp_fp = BufWriter::new(tmp_file);

        let mut reader = Reader::from_path(filename).expect("Unable to open reader");
        for ptu in reader.points() {
            let pt = ptu.unwrap();
            if thinfactor == 1.0 || rng.sample(randdist) {
                write!(
                    &mut tmp_fp,
                    "{} {} {} {} {} {} {}\r\n",
                    pt.x * xfactor,
                    pt.y * yfactor,
                    pt.z * zfactor + zoff,
                    u8::from(pt.classification),
                    pt.number_of_returns,
                    pt.return_number,
                    pt.intensity
                )
                .expect("Could not write temp file");
            }
        }
        tmp_fp.flush().unwrap();
    } else {
        fs::copy(
            Path::new(filename),
            Path::new(&format!("{}/xyztemp.xyz", tmpfolder)),
        )
        .expect("Could not copy file to tmpfolder");
    }
    println!("{}Done", thread_name);
    println!("{}Knoll detection part 1", thread_name);
    let scalefactor: f64 = conf
        .general_section()
        .get("scalefactor")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);
    let vegeonly: bool = conf.general_section().get("vegeonly").unwrap_or("0") == "1";

    if !vegeonly {
        pullauta::contours::xyz2contours(
            thread,
            scalefactor * 0.3,
            "xyztemp.xyz",
            "xyz_03.xyz",
            "contours03.dxf",
            true,
        )
        .expect("contour generation failed");
    } else {
        pullauta::contours::xyz2contours(
            thread,
            scalefactor * 0.3,
            "xyztemp.xyz",
            "xyz_03.xyz",
            "null",
            true,
        )
        .expect("contour generation failed");
    }

    fs::copy(
        format!("{}/xyz_03.xyz", tmpfolder),
        format!("{}/xyz2.xyz", tmpfolder),
    )
    .expect("Could not copy file");

    let contour_interval: f64 = conf
        .general_section()
        .get("contour_interval")
        .unwrap_or("5")
        .parse::<f64>()
        .unwrap_or(5.0);
    let halfinterval = contour_interval / 2.0 * scalefactor;

    if !vegeonly {
        let basemapcontours: f64 = conf
            .general_section()
            .get("basemapinterval")
            .unwrap_or("0")
            .parse::<f64>()
            .unwrap_or(0.0);
        if basemapcontours != 0.0 {
            println!("{}Basemap contours", thread_name);
            pullauta::contours::xyz2contours(
                thread,
                basemapcontours,
                "xyz2.xyz",
                "",
                "basemap.dxf",
                false,
            )
            .expect("contour generation failed");
        }
        if !skipknolldetection {
            println!("{}Knoll detection part 2", thread_name);
            pullauta::knolls::knolldetector(thread).unwrap();
        }
        println!("{}Contour generation part 1", thread_name);
        pullauta::knolls::xyzknolls(thread).unwrap();

        println!("{}Contour generation part 2", thread_name);
        if !skipknolldetection {
            // contours 2.5
            pullauta::contours::xyz2contours(
                thread,
                halfinterval,
                "xyz_knolls.xyz",
                "null",
                "out.dxf",
                false,
            )
            .unwrap();
        } else {
            pullauta::contours::xyz2contours(
                thread,
                halfinterval,
                "xyztemp.xyz",
                "null",
                "out.dxf",
                true,
            )
            .unwrap();
        }
        println!("{}Contour generation part 3", thread_name);
        pullauta::merge::smoothjoin(thread).unwrap();
        println!("{}Contour generation part 4", thread_name);
        pullauta::knolls::dotknolls(thread).unwrap();
    }

    println!("{}Vegetation generation", thread_name);
    pullauta::vegetation::makevegenew(thread).unwrap();

    if !vegeonly {
        println!("{}Cliff generation", thread_name);
        pullauta::cliffs::makecliffs(thread).unwrap();
    }
    let detectbuildings: bool = conf.general_section().get("detectbuildings").unwrap_or("0") == "1";
    if detectbuildings {
        println!("{}Detecting buildings", thread_name);
        pullauta::blocks::blocks(thread).unwrap();
    }
    if !skip_rendering {
        println!("{}Rendering png map with depressions", thread_name);
        render(thread, pnorthlinesangle, pnorthlineswidth, false).unwrap();
        println!("{}Rendering png map without depressions", thread_name);
        render(thread, pnorthlinesangle, pnorthlineswidth, true).unwrap();
    } else {
        println!("{}Skipped rendering", thread_name);
    }
    println!("\n\n{}All done!", thread_name);
    Ok(())
}

fn render(
    thread: &String,
    angle_deg: f64,
    nwidth: usize,
    nodepressions: bool,
) -> Result<(), Box<dyn Error>> {
    println!("Rendering...");
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let scalefactor: f64 = conf
        .general_section()
        .get("scalefactor")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);
    let mut formlinesteepness: f64 = conf
        .general_section()
        .get("formlinesteepness")
        .unwrap_or("0.37")
        .parse::<f64>()
        .unwrap_or(0.37);
    formlinesteepness *= scalefactor;
    let formline: f64 = conf
        .general_section()
        .get("formline")
        .unwrap_or("2")
        .parse::<f64>()
        .unwrap_or(2.0);
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

    let tmpfolder = format!("temp{}", thread);
    let angle = -angle_deg / 180.0 * PI;

    let mut size: f64 = 0.0;
    let mut xstart: f64 = 0.0;
    let mut ystart: f64 = 0.0;
    let mut steepness: HashMap<(usize, usize), f64> = HashMap::default();
    if formline > 0.0 {
        let path = format!("{}/xyz2.xyz", tmpfolder);
        let xyz_file_in = Path::new(&path);

        if let Ok(lines) = read_lines(xyz_file_in) {
            for (i, line) in lines.enumerate() {
                let ip = line.unwrap_or(String::new());
                let mut parts = ip.split(' ');
                let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();

                if i == 0 {
                    xstart = x;
                    ystart = y;
                } else if i == 1 {
                    size = y - ystart;
                } else {
                    break;
                }
            }
        }

        let mut sxmax: usize = usize::MIN;
        let mut symax: usize = usize::MIN;

        let mut xyz: HashMap<(usize, usize), f64> = HashMap::default();

        if let Ok(lines) = read_lines(xyz_file_in) {
            for line in lines {
                let ip = line.unwrap_or(String::new());
                let mut parts = ip.split(' ');
                let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();

                let xx = ((x - xstart) / size).floor() as usize;
                let yy = ((y - ystart) / size).floor() as usize;

                xyz.insert((xx, yy), h);

                if sxmax < xx {
                    sxmax = xx;
                }
                if symax < yy {
                    symax = yy;
                }
            }
        }
        for i in 6..(sxmax - 7) {
            for j in 6..(symax - 7) {
                let mut det: f64 = 0.0;
                let mut high: f64 = f64::MIN;

                let mut temp =
                    (xyz.get(&(i - 4, j)).unwrap_or(&0.0) - xyz.get(&(i, j)).unwrap_or(&0.0)).abs()
                        / 4.0;
                let temp2 =
                    (xyz.get(&(i, j)).unwrap_or(&0.0) - xyz.get(&(i + 4, j)).unwrap_or(&0.0)).abs()
                        / 4.0;
                let det2 = (xyz.get(&(i, j)).unwrap_or(&0.0)
                    - 0.5
                        * (xyz.get(&(i - 4, j)).unwrap_or(&0.0)
                            + xyz.get(&(i + 4, j)).unwrap_or(&0.0)))
                .abs()
                    - 0.05
                        * (xyz.get(&(i - 4, j)).unwrap_or(&0.0)
                            - xyz.get(&(i + 4, j)).unwrap_or(&0.0))
                        .abs();
                let mut porr = (((xyz.get(&(i - 6, j)).unwrap_or(&0.0)
                    - xyz.get(&(i + 6, j)).unwrap_or(&0.0))
                    / 12.0)
                    .abs()
                    - ((xyz.get(&(i - 3, j)).unwrap_or(&0.0)
                        - xyz.get(&(i + 3, j)).unwrap_or(&0.0))
                        / 6.0)
                        .abs())
                .abs();

                if det2 > det {
                    det = det2;
                }
                if temp2 < temp {
                    temp = temp2;
                }
                if temp > high {
                    high = temp;
                }

                let mut temp =
                    (xyz.get(&(i, j - 4)).unwrap_or(&0.0) - xyz.get(&(i, j)).unwrap_or(&0.0)).abs()
                        / 4.0;
                let temp2 =
                    (xyz.get(&(i, j)).unwrap_or(&0.0) - xyz.get(&(i, j - 4)).unwrap_or(&0.0)).abs()
                        / 4.0;
                let det2 = (xyz.get(&(i, j)).unwrap_or(&0.0)
                    - 0.5
                        * (xyz.get(&(i, j - 4)).unwrap_or(&0.0)
                            + xyz.get(&(i, j + 4)).unwrap_or(&0.0)))
                .abs()
                    - 0.05
                        * (xyz.get(&(i, j - 4)).unwrap_or(&0.0)
                            - xyz.get(&(i, j + 4)).unwrap_or(&0.0))
                        .abs();
                let porr2 = (((xyz.get(&(i, j - 6)).unwrap_or(&0.0)
                    - xyz.get(&(i, j + 6)).unwrap_or(&0.0))
                    / 12.0)
                    .abs()
                    - ((xyz.get(&(i, j - 3)).unwrap_or(&0.0)
                        - xyz.get(&(i, j + 3)).unwrap_or(&0.0))
                        / 6.0)
                        .abs())
                .abs();

                if porr2 > porr {
                    porr = porr2;
                }
                if det2 > det {
                    det = det2;
                }
                if temp2 < temp {
                    temp = temp2;
                }
                if temp > high {
                    high = temp;
                }

                let mut temp = (xyz.get(&(i - 4, j - 4)).unwrap_or(&0.0)
                    - xyz.get(&(i, j)).unwrap_or(&0.0))
                .abs()
                    / 5.6;
                let temp2 = (xyz.get(&(i, j)).unwrap_or(&0.0)
                    - xyz.get(&(i + 4, j + 4)).unwrap_or(&0.0))
                .abs()
                    / 5.6;
                let det2 = (xyz.get(&(i, j)).unwrap_or(&0.0)
                    - 0.5
                        * (xyz.get(&(i - 4, j - 4)).unwrap_or(&0.0)
                            + xyz.get(&(i + 4, j + 4)).unwrap_or(&0.0)))
                .abs()
                    - 0.05
                        * (xyz.get(&(i - 4, j - 4)).unwrap_or(&0.0)
                            - xyz.get(&(i + 4, j + 4)).unwrap_or(&0.0))
                        .abs();
                let porr2 = (((xyz.get(&(i - 6, j - 6)).unwrap_or(&0.0)
                    - xyz.get(&(i + 6, j + 6)).unwrap_or(&0.0))
                    / 17.0)
                    .abs()
                    - ((xyz.get(&(i - 3, j - 3)).unwrap_or(&0.0)
                        - xyz.get(&(i + 3, j + 3)).unwrap_or(&0.0))
                        / 8.5)
                        .abs())
                .abs();

                if porr2 > porr {
                    porr = porr2;
                }
                if det2 > det {
                    det = det2;
                }
                if temp2 < temp {
                    temp = temp2;
                }
                if temp > high {
                    high = temp;
                }

                let mut temp = (xyz.get(&(i - 4, j + 4)).unwrap_or(&0.0)
                    - xyz.get(&(i, j)).unwrap_or(&0.0))
                .abs()
                    / 5.6;
                let temp2 = (xyz.get(&(i, j)).unwrap_or(&0.0)
                    - xyz.get(&(i + 4, j - 4)).unwrap_or(&0.0))
                .abs()
                    / 5.6;
                let det2 = (xyz.get(&(i, j)).unwrap_or(&0.0)
                    - 0.5
                        * (xyz.get(&(i + 4, j - 4)).unwrap_or(&0.0)
                            + xyz.get(&(i - 4, j + 4)).unwrap_or(&0.0)))
                .abs()
                    - 0.05
                        * (xyz.get(&(i + 4, j - 4)).unwrap_or(&0.0)
                            - xyz.get(&(i - 4, j + 4)).unwrap_or(&0.0))
                        .abs();
                let porr2 = (((xyz.get(&(i + 6, j - 6)).unwrap_or(&0.0)
                    - xyz.get(&(i - 6, j + 6)).unwrap_or(&0.0))
                    / 17.0)
                    .abs()
                    - ((xyz.get(&(i + 3, j - 3)).unwrap_or(&0.0)
                        - xyz.get(&(i - 3, j + 3)).unwrap_or(&0.0))
                        / 8.5)
                        .abs())
                .abs();

                if porr2 > porr {
                    porr = porr2;
                }
                if det2 > det {
                    det = det2;
                }
                if temp2 < temp {
                    temp = temp2;
                }
                if temp > high {
                    high = temp;
                }

                let mut val = 12.0 * high / (1.0 + 8.0 * det);
                if porr > 0.25 * 0.67 / (0.3 + formlinesteepness) {
                    val = 0.01;
                }
                if high > val {
                    val = high;
                }
                steepness.insert((i, j), val);
            }
        }
    }

    // Draw vegetation ----------
    let path = format!("{}/vegetation.pgw", tmpfolder);
    let tfw_in = Path::new(&path);

    let mut lines = read_lines(tfw_in).expect("PGW file does not exist");
    let x0 = lines
        .nth(4)
        .expect("no 4 line")
        .expect("Could not read line 5")
        .parse::<f64>()
        .unwrap();
    let y0 = lines
        .next()
        .expect("no 5 line")
        .expect("Could not read line 6")
        .parse::<f64>()
        .unwrap();

    let mut img_reader =
        image::io::Reader::open(Path::new(&format!("{}/vegetation.png", tmpfolder)))
            .expect("Opening vegetation image failed");
    img_reader.no_limits();
    let img = img_reader.decode().unwrap();

    let mut imgug_reader =
        image::io::Reader::open(Path::new(&format!("{}/undergrowth.png", tmpfolder)))
            .expect("Opening undergrowth image failed");
    imgug_reader.no_limits();
    let imgug = imgug_reader.decode().unwrap();

    let w = img.width();
    let h = img.height();

    let eastoff = -((x0 - (-angle).tan() * y0)
        - ((x0 - (-angle).tan() * y0) / (250.0 / angle.cos())).floor() * (250.0 / angle.cos()))
        / 254.0
        * 600.0;

    let new_width = (w as f64 * 600.0 / 254.0 / scalefactor) as u32;
    let new_height = (h as f64 * 600.0 / 254.0 / scalefactor) as u32;
    let mut img = image::imageops::resize(
        &img,
        new_width,
        new_height,
        image::imageops::FilterType::Nearest,
    );

    let imgug = image::imageops::resize(
        &imgug,
        new_width,
        new_height,
        image::imageops::FilterType::Nearest,
    );

    image::imageops::overlay(&mut img, &imgug, 0, 0);

    if Path::new(&format!("{}/low.png", tmpfolder)).exists() {
        let mut low_reader = image::io::Reader::open(Path::new(&format!("{}/low.png", tmpfolder)))
            .expect("Opening low image failed");
        low_reader.no_limits();
        let low = low_reader.decode().unwrap();
        let low = image::imageops::resize(
            &low,
            new_width,
            new_height,
            image::imageops::FilterType::Nearest,
        );
        image::imageops::overlay(&mut img, &low, 0, 0);
    }

    // north lines ----------------
    if angle != 999.0 {
        let mut i: f64 = eastoff - 600.0 * 250.0 / 254.0 / angle.cos() * 100.0 / scalefactor;
        while i < w as f64 * 5.0 * 600.0 / 254.0 / scalefactor {
            for m in 0..nwidth {
                draw_line_segment_mut(
                    &mut img,
                    (i as f32 + m as f32, 0.0),
                    (
                        (i as f32 + (angle.tan() * (h as f64) * 600.0 / 254.0 / scalefactor) as f32)
                            as f32
                            + m as f32,
                        (h as f32 * 600.0 / 254.0 / scalefactor as f32) as f32,
                    ),
                    Rgba([0, 0, 200, 255]),
                );
            }
            i += 600.0 * 250.0 / 254.0 / angle.cos() / scalefactor;
        }
    }

    // Drawing curves --------------
    let vegeonly: bool = conf.general_section().get("vegeonly").unwrap_or("0") == "1";

    if !vegeonly {
        let input_filename = &format!("{}/out2.dxf", tmpfolder);
        let input = Path::new(input_filename);
        let data = fs::read_to_string(input).expect("Can not read input file");
        let data: Vec<&str> = data.split("POLYLINE").collect();

        let mut formline_out = String::new();
        formline_out.push_str(data[0]);

        for (j, rec) in data.iter().enumerate() {
            let mut x = Vec::<f64>::new();
            let mut y = Vec::<f64>::new();
            let mut xline = 0;
            let mut yline = 0;
            let mut layer = "";
            if j > 0 {
                let r = rec.split("VERTEX").collect::<Vec<&str>>();
                let apu = r[1];
                let val = apu.split('\n').collect::<Vec<&str>>();
                layer = val[2].trim();
                for (i, v) in val.iter().enumerate() {
                    let vt = v.trim();
                    if vt == "10" {
                        xline = i + 1;
                    }
                    if vt == "20" {
                        yline = i + 1;
                    }
                }
                for (i, v) in r.iter().enumerate() {
                    if i > 0 {
                        let val = v.trim_end().split('\n').collect::<Vec<&str>>();
                        x.push(
                            (val[xline].trim().parse::<f64>().unwrap() - x0) * 600.0
                                / 254.0
                                / scalefactor,
                        );
                        y.push(
                            (y0 - val[yline].trim().parse::<f64>().unwrap()) * 600.0
                                / 254.0
                                / scalefactor,
                        );
                    }
                }
            }
            let mut color = Rgba([200, 0, 200, 255]); // purple
            if layer.contains("contour") {
                color = Rgba([166, 85, 43, 255]) // brown
            }
            if !nodepressions || layer.contains("contour") {
                let mut curvew = 2.0;
                if layer.contains("index") {
                    curvew = 3.0;
                }
                if formline > 0.0 {
                    if formline == 1.0 {
                        curvew = 2.5
                    }
                    if layer.contains("intermed") {
                        curvew = 1.5
                    }
                    if layer.contains("index") {
                        curvew = 3.5
                    }
                }

                let mut smallringtest = false;
                let mut help = vec![false; x.len()];
                let mut help2 = vec![false; x.len()];
                if curvew == 1.5 {
                    for i in 0..x.len() {
                        help[i] = false;
                        help2[i] = true;
                        let xx =
                            (((x[i] / 600.0 * 254.0 * scalefactor + x0) - xstart) / size).floor();
                        let yy =
                            (((-y[i] / 600.0 * 254.0 * scalefactor + y0) - ystart) / size).floor();
                        if curvew != 1.5
                            || formline == 0.0
                            || steepness.get(&(xx as usize, yy as usize)).unwrap_or(&0.0)
                                < &formlinesteepness
                            || steepness
                                .get(&(xx as usize, yy as usize + 1))
                                .unwrap_or(&0.0)
                                < &formlinesteepness
                            || steepness
                                .get(&(xx as usize + 1, yy as usize))
                                .unwrap_or(&0.0)
                                < &formlinesteepness
                            || steepness
                                .get(&(xx as usize + 1, yy as usize + 1))
                                .unwrap_or(&0.0)
                                < &formlinesteepness
                        {
                            help[i] = true;
                        }
                    }
                    for i in 5..(x.len() - 6) {
                        let mut apu = 0;
                        for j in (i - 5)..(i + 4) {
                            if help[j] {
                                apu += 1;
                            }
                        }
                        if apu < 5 {
                            help2[i] = false;
                        }
                    }
                    for i in 0..6 {
                        help2[i] = help2[6]
                    }
                    for i in (x.len() - 6)..x.len() {
                        help2[i] = help2[x.len() - 7]
                    }
                    let mut on = 0.0;
                    for i in 0..x.len() {
                        if help2[i] {
                            on = formlineaddition
                        }
                        if on > 0.0 {
                            help2[i] = true;
                            on -= 1.0;
                        }
                    }
                    if x.first() == x.last() && y.first() == y.last() && on > 0.0 {
                        let mut i = 0;
                        while i < x.len() && on > 0.0 {
                            help2[i] = true;
                            on -= 1.0;
                            i += 1;
                        }
                    }
                    let mut on = 0.0;
                    for i in 0..x.len() {
                        let ii = x.len() - i - 1;
                        if help2[ii] {
                            on = formlineaddition
                        }
                        if on > 0.0 {
                            help2[ii] = true;
                            on -= 1.0;
                        }
                    }
                    if x.first() == x.last() && y.first() == y.last() && on > 0.0 {
                        let mut i = (x.len() - 1) as i32;
                        while i > -1 && on > 0.0 {
                            help2[i as usize] = true;
                            on -= 1.0;
                            i -= 1;
                        }
                    }
                    // Let's not break small form line rings
                    smallringtest = false;
                    if x.first() == x.last() && y.first() == y.last() && x.len() < 122 {
                        for i in 1..x.len() {
                            if help2[i] {
                                smallringtest = true
                            }
                        }
                    }
                    // Let's draw short gaps together
                    if !smallringtest {
                        let mut tester = 1;
                        for i in 1..x.len() {
                            if help2[i] {
                                if tester < i && ((i - tester) as u32) < minimumgap {
                                    for j in tester..(i + 1) {
                                        help2[j] = true;
                                    }
                                }
                                tester = i;
                            }
                        }
                        // Ring handling
                        if x.first() == x.last() && y.first() == y.last() && x.len() < 2 {
                            let mut i = 1;
                            while i < x.len() && !help2[i] {
                                i += 1
                            }
                            let mut j = x.len() - 1;
                            while j > 1 && !help2[i] {
                                j -= 1
                            }
                            if ((x.len() - j + i - 1) as u32) < minimumgap && j > i {
                                for k in 0..(i + 1) {
                                    help2[k] = true
                                }
                                for k in j..x.len() {
                                    help2[k] = true
                                }
                            }
                        }
                    }
                }

                let mut linedist = 0.0;
                let mut onegapdone = false;
                let mut gap = 0.0;
                let mut formlinestart = false;

                let f_label;
                if layer.contains("depression") && label_depressions {
                    f_label = "formline_depression";
                } else {
                    f_label = "formline"
                };

                for i in 1..x.len() {
                    if curvew != 1.5 || formline == 0.0 || help2[i] || smallringtest {
                        if formline == 2.0 && !nodepressions && curvew == 1.5 {
                            if !formlinestart {
                                formline_out.push_str(
                                    format!(
                                        "POLYLINE\r\n 66\r\n1\r\n  8\r\n{}\r\n  0\r\n",
                                        f_label
                                    )
                                    .as_str(),
                                );
                                formlinestart = true;
                            }
                            formline_out.push_str(
                                format!(
                                    "VERTEX\r\n  8\r\n{}\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\n",
                                    f_label,
                                    x[i] / 600.0 * 254.0 * scalefactor + x0,
                                    -y[i] / 600.0 * 254.0 * scalefactor + y0
                                )
                                .as_str(),
                            );
                        }
                        if curvew == 1.5 && formline == 2.0 {
                            let step =
                                ((x[i - 1] - x[i]).powi(2) + (y[i - 1] - y[i]).powi(2)).sqrt();
                            if i < 4 {
                                linedist = 0.0
                            }
                            linedist += step;
                            if linedist > dashlength && i > 10 && i < x.len() - 11 {
                                let mut sum = 0.0;
                                for k in (i - 4)..(i + 6) {
                                    sum += ((x[k - 1] - x[k]).powi(2) + (y[k - 1] - y[k]).powi(2))
                                        .sqrt()
                                }
                                let mut toonearend = false;
                                for k in (i - 10)..(i + 10) {
                                    if !help2[k] {
                                        toonearend = true;
                                        break;
                                    }
                                }
                                if !toonearend
                                    && ((x[i - 5] - x[i + 5]).powi(2)
                                        + (y[i - 5] - y[i + 5]).powi(2))
                                    .sqrt()
                                        * 1.138
                                        > sum
                                {
                                    linedist = 0.0;
                                    gap = gaplength;
                                    onegapdone = true;
                                }
                            }
                            if !onegapdone && (i < x.len() - 9) && i > 6 {
                                gap = gaplength * 0.82;
                                onegapdone = true;
                                linedist = 0.0
                            }
                            if gap > 0.0 {
                                gap -= step;
                                if gap < 0.0 && onegapdone && step > 0.0 {
                                    let mut n = -curvew - 0.5;
                                    while n < curvew + 0.5 {
                                        let mut m = -curvew - 0.5;
                                        while m < curvew + 0.5 {
                                            draw_line_segment_mut(
                                                &mut img,
                                                (
                                                    ((-x[i - 1] * gap + (step + gap) * x[i]) / step
                                                        + n)
                                                        as f32,
                                                    ((-y[i - 1] * gap + (step + gap) * y[i]) / step
                                                        + m)
                                                        as f32,
                                                ),
                                                ((x[i] + n) as f32, (y[i] + m) as f32),
                                                color,
                                            );
                                            m += 1.0;
                                        }
                                        n += 1.0;
                                    }
                                    gap = 0.0;
                                }
                            } else {
                                let mut n = -curvew - 0.5;
                                while n < curvew + 0.5 {
                                    let mut m = -curvew - 0.5;
                                    while m < curvew + 0.5 {
                                        draw_line_segment_mut(
                                            &mut img,
                                            ((x[i - 1] + n) as f32, (y[i - 1] + m) as f32),
                                            ((x[i] + n) as f32, (y[i] + m) as f32),
                                            color,
                                        );
                                        m += 1.0;
                                    }
                                    n += 1.0;
                                }
                            }
                        } else {
                            let mut n = -curvew;
                            while n < curvew {
                                let mut m = -curvew;
                                while m < curvew {
                                    draw_line_segment_mut(
                                        &mut img,
                                        ((x[i - 1] + n) as f32, (y[i - 1] + m) as f32),
                                        ((x[i] + n) as f32, (y[i] + m) as f32),
                                        color,
                                    );
                                    m += 1.0;
                                }
                                n += 1.0;
                            }
                        }
                    } else if formline == 2.0 && formlinestart && !nodepressions {
                        formline_out.push_str("SEQEND\r\n  0\r\n");
                        formlinestart = false;
                    }
                }
                if formline == 2.0 && formlinestart && !nodepressions {
                    formline_out.push_str("SEQEND\r\n  0\r\n");
                }
            }
        }
        if formline == 2.0 && !nodepressions {
            formline_out.push_str("ENDSEC\r\n  0\r\nEOF\r\n");
            let filename = &format!("{}/formlines.dxf", tmpfolder);
            let output = Path::new(filename);
            let fp = File::create(output).expect("Unable to create file");
            let mut fp = BufWriter::new(fp);
            fp.write_all(formline_out.as_bytes())
                .expect("Unable to write file");
        }
        // dotknolls----------
        let input_filename = &format!("{}/dotknolls.dxf", tmpfolder);
        let input = Path::new(input_filename);
        let data = fs::read_to_string(input).expect("Can not read input file");
        let data = data.split("POINT");

        for (j, rec) in data.enumerate() {
            let mut x: f64 = 0.0;
            let mut y: f64 = 0.0;
            if j > 0 {
                let val = rec.split('\n').collect::<Vec<&str>>();
                let layer = val[2].trim();
                for (i, v) in val.iter().enumerate() {
                    let vt = v.trim();
                    if vt == "10" {
                        x = (val[i + 1].trim().parse::<f64>().unwrap() - x0) * 600.0
                            / 254.0
                            / scalefactor;
                    }
                    if vt == "20" {
                        y = (y0 - val[i + 1].trim().parse::<f64>().unwrap()) * 600.0
                            / 254.0
                            / scalefactor;
                    }
                }
                if layer == "dotknoll" {
                    let color = Rgba([166, 85, 43, 255]);

                    draw_filled_circle_mut(&mut img, (x as i32, y as i32), 7, color)
                }
            }
        }
    }
    // blocks -------------
    if Path::new(&format!("{}/blocks.png", tmpfolder)).exists() {
        let mut blockpurple_reader =
            image::io::Reader::open(Path::new(&format!("{}/blocks.png", tmpfolder)))
                .expect("Opening blocks image failed");
        blockpurple_reader.no_limits();
        let blockpurple = blockpurple_reader.decode().unwrap();
        let mut blockpurple = blockpurple.to_rgba8();
        for p in blockpurple.pixels_mut() {
            if p[0] == 255 && p[1] == 255 && p[2] == 255 {
                p[3] = 0;
            }
        }
        let blockpurple = image::imageops::crop(&mut blockpurple, 0, 0, w, h).to_image();
        let blockpurple_thumb = image::imageops::resize(
            &blockpurple,
            new_width as u32,
            new_height as u32,
            image::imageops::FilterType::Nearest,
        );

        for i in 0..3 {
            for j in 0..3 {
                image::imageops::overlay(
                    &mut img,
                    &blockpurple_thumb,
                    (i as i64 - 1) * 2,
                    (j as i64 - 1) * 2,
                );
            }
        }
        image::imageops::overlay(&mut img, &blockpurple_thumb, 0, 0);
    }
    // blueblack -------------
    if Path::new(&format!("{}/blueblack.png", tmpfolder)).exists() {
        let mut imgbb_reader =
            image::io::Reader::open(Path::new(&format!("{}/blueblack.png", tmpfolder)))
                .expect("Opening blueblack image failed");
        imgbb_reader.no_limits();
        let imgbb = imgbb_reader.decode().unwrap();
        let mut imgbb = imgbb.to_rgba8();
        for p in imgbb.pixels_mut() {
            if p[0] == 255 && p[1] == 255 && p[2] == 255 {
                p[3] = 0;
            }
        }
        let imgbb = image::imageops::crop(&mut imgbb, 0, 0, w, h).to_image();
        let imgbb_thumb = image::imageops::resize(
            &imgbb,
            new_width as u32,
            new_height as u32,
            image::imageops::FilterType::Nearest,
        );
        image::imageops::overlay(&mut img, &imgbb_thumb, 0, 0);
    }

    let cliffdebug: bool = conf.general_section().get("cliffdebug").unwrap_or("0") == "1";

    let black = Rgba([0, 0, 0, 255]);

    let mut cliffcolor =
        HashMap::from_iter([("cliff2", black), ("cliff3", black), ("cliff4", black)]);
    if cliffdebug {
        cliffcolor = HashMap::from_iter([
            ("cliff2", Rgba([100, 0, 100, 255])),
            ("cliff3", Rgba([0, 100, 100, 255])),
            ("cliff4", Rgba([100, 100, 0, 255])),
        ]);
    }
    if !vegeonly {
        let input_filename = &format!("{}/c2g.dxf", tmpfolder);
        let input = Path::new(input_filename);
        let data = fs::read_to_string(input).expect("Can not read input file");
        let data: Vec<&str> = data.split("POLYLINE").collect();

        let mut formline_out = String::new();
        formline_out.push_str(data[0]);

        for (j, rec) in data.iter().enumerate() {
            let mut x = Vec::<f64>::new();
            let mut y = Vec::<f64>::new();
            let mut xline = 0;
            let mut yline = 0;
            let mut layer = "";
            if j > 0 {
                let r = rec.split("VERTEX").collect::<Vec<&str>>();
                let apu = r[1];
                let val = apu.split('\n').collect::<Vec<&str>>();
                layer = val[2].trim();
                for (i, v) in val.iter().enumerate() {
                    let vt = v.trim();
                    if vt == "10" {
                        xline = i + 1;
                    }
                    if vt == "20" {
                        yline = i + 1;
                    }
                }
                for (i, v) in r.iter().enumerate() {
                    if i > 0 {
                        let val = v.trim_end().split('\n').collect::<Vec<&str>>();
                        x.push(
                            (val[xline].trim().parse::<f64>().unwrap() - x0) * 600.0
                                / 254.0
                                / scalefactor,
                        );
                        y.push(
                            (y0 - val[yline].trim().parse::<f64>().unwrap()) * 600.0
                                / 254.0
                                / scalefactor,
                        );
                    }
                }
            }
            let last_idx = x.len() - 1;
            if x.first() != x.last() || y.first() != y.last() {
                let dist = ((x[0] - x[last_idx]).powi(2) + (y[0] - y[last_idx]).powi(2)).sqrt();
                if dist > 0.0 {
                    let dx = x[0] - x[last_idx];
                    let dy = y[0] - y[last_idx];
                    x[0] += dx / dist * 1.5;
                    y[0] += dy / dist * 1.5;
                    x[last_idx] -= dx / dist * 1.5;
                    y[last_idx] -= dy / dist * 1.5;
                    draw_filled_circle_mut(
                        &mut img,
                        (x[0] as i32, y[0] as i32),
                        3,
                        *cliffcolor.get(&layer).unwrap_or(&black),
                    );
                    draw_filled_circle_mut(
                        &mut img,
                        (x[last_idx] as i32, y[last_idx] as i32),
                        3,
                        *cliffcolor.get(&layer).unwrap_or(&black),
                    );
                }
            }
            for i in 1..x.len() {
                for n in 0..6 {
                    for m in 0..6 {
                        draw_line_segment_mut(
                            &mut img,
                            (
                                (x[i - 1] + (n as f64) - 3.0).floor() as f32,
                                (y[i - 1] + (m as f64) - 3.0).floor() as f32,
                            ),
                            (
                                (x[i] + (n as f64) - 3.0).floor() as f32,
                                (y[i] + (m as f64) - 3.0).floor() as f32,
                            ),
                            *cliffcolor.get(&layer).unwrap_or(&black),
                        )
                    }
                }
            }
        }

        let input_filename = &format!("{}/c3g.dxf", tmpfolder);
        let input = Path::new(input_filename);
        let data = fs::read_to_string(input).expect("Can not read input file");
        let data: Vec<&str> = data.split("POLYLINE").collect();

        let mut formline_out = String::new();
        formline_out.push_str(data[0]);

        for (j, rec) in data.iter().enumerate() {
            let mut x = Vec::<f64>::new();
            let mut y = Vec::<f64>::new();
            let mut xline = 0;
            let mut yline = 0;
            let mut layer = "";
            if j > 0 {
                let r = rec.split("VERTEX").collect::<Vec<&str>>();
                let apu = r[1];
                let val = apu.split('\n').collect::<Vec<&str>>();
                layer = val[2].trim();
                for (i, v) in val.iter().enumerate() {
                    let vt = v.trim();
                    if vt == "10" {
                        xline = i + 1;
                    }
                    if vt == "20" {
                        yline = i + 1;
                    }
                }
                for (i, v) in r.iter().enumerate() {
                    if i > 0 {
                        let val = v.trim_end().split('\n').collect::<Vec<&str>>();
                        x.push(
                            (val[xline].trim().parse::<f64>().unwrap() - x0) * 600.0
                                / 254.0
                                / scalefactor,
                        );
                        y.push(
                            (y0 - val[yline].trim().parse::<f64>().unwrap()) * 600.0
                                / 254.0
                                / scalefactor,
                        );
                    }
                }
            }
            let last_idx = x.len() - 1;
            if x.first() != x.last() || y.first() != y.last() {
                let dist = ((x[0] - x[last_idx]).powi(2) + (y[0] - y[last_idx]).powi(2)).sqrt();
                if dist > 0.0 {
                    let dx = x[0] - x[last_idx];
                    let dy = y[0] - y[last_idx];
                    x[0] += dx / dist * 1.5;
                    y[0] += dy / dist * 1.5;
                    x[last_idx] -= dx / dist * 1.5;
                    y[last_idx] -= dy / dist * 1.5;

                    draw_filled_circle_mut(
                        &mut img,
                        (x[0] as i32, y[0] as i32),
                        3,
                        *cliffcolor.get(&layer).unwrap_or(&black),
                    );
                    draw_filled_circle_mut(
                        &mut img,
                        (x[last_idx] as i32, y[last_idx] as i32),
                        3,
                        *cliffcolor.get(&layer).unwrap_or(&black),
                    );
                }
            }
            for i in 1..x.len() {
                for n in 0..6 {
                    for m in 0..6 {
                        draw_line_segment_mut(
                            &mut img,
                            (
                                (x[i - 1] + (n as f64) - 3.0).floor() as f32,
                                (y[i - 1] + (m as f64) - 3.0).floor() as f32,
                            ),
                            (
                                (x[i] + (n as f64) - 3.0).floor() as f32,
                                (y[i] + (m as f64) - 3.0).floor() as f32,
                            ),
                            *cliffcolor.get(&layer).unwrap_or(&black),
                        )
                    }
                }
            }
        }
    }
    // high -------------
    if Path::new(&format!("{}/high.png", tmpfolder)).exists() {
        let mut high_reader =
            image::io::Reader::open(Path::new(&format!("{}/high.png", tmpfolder)))
                .expect("Opening high image failed");
        high_reader.no_limits();
        let high = high_reader.decode().unwrap();
        let high_thumb = image::imageops::resize(
            &high,
            new_width as u32,
            new_height as u32,
            image::imageops::FilterType::Nearest,
        );
        image::imageops::overlay(&mut img, &high_thumb, 0, 0);
    }

    let mut filename = format!("pullautus{}", thread);
    if !nodepressions {
        filename = format!("pullautus_depr{}", thread);
    }
    img.save(Path::new(&format!("{}.png", filename)))
        .expect("could not save output png");

    let path_in = format!("{}/vegetation.pgw", tmpfolder);
    let file_in = Path::new(&path_in);
    let pgw_file_out = File::create(format!("{}.pgw", filename)).expect("Unable to create file");
    let mut pgw_file_out = BufWriter::new(pgw_file_out);

    if let Ok(lines) = read_lines(file_in) {
        for (i, line) in lines.enumerate() {
            let ip = line.unwrap_or(String::new());
            let x: f64 = ip.parse::<f64>().unwrap();
            if i == 0 || i == 3 {
                write!(&mut pgw_file_out, "{}\r\n", x / 600.0 * 254.0 * scalefactor)
                    .expect("Unable to write to file");
            } else {
                write!(&mut pgw_file_out, "{}\r\n", ip).expect("Unable to write to file");
            }
        }
    }
    println!("Done");
    Ok(())
}

fn polylinedxfcrop(
    input: &Path,
    output: &Path,
    minx: f64,
    miny: f64,
    maxx: f64,
    maxy: f64,
) -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string(input).expect("Should have been able to read the file");
    let data: Vec<&str> = data.split("POLYLINE").collect();
    let dxfhead = data[0];
    let mut out = String::new();
    out.push_str(dxfhead);
    for (j, rec) in data.iter().enumerate() {
        let mut poly = String::new();
        let mut pre = "";
        let mut prex = 0.0;
        let mut prey = 0.0;
        let mut pointcount = 0;
        if j > 0 {
            if let Some((head, rec2)) = rec.split_once("VERTEX") {
                let r: Vec<&str> = rec2.split("VERTEX").collect();
                poly.push_str(&format!("POLYLINE{}", head));
                for apu in r.iter() {
                    let (apu2, _notused) = apu.split_once("SEQEND").unwrap_or((apu, ""));
                    let val: Vec<&str> = apu2.split('\n').collect();
                    let mut xline = 0;
                    let mut yline = 0;
                    for (i, v) in val.iter().enumerate() {
                        let vt = v.trim();
                        if vt == "10" {
                            xline = i + 1;
                        }
                        if vt == "20" {
                            yline = i + 1;
                        }
                    }
                    let valx = val[xline].trim().parse::<f64>().unwrap_or(0.0);
                    let valy = val[yline].trim().parse::<f64>().unwrap_or(0.0);
                    if valx >= minx && valx <= maxx && valy >= miny && valy <= maxy {
                        if !pre.is_empty() && pointcount == 0 && (prex < minx || prey < miny) {
                            poly.push_str(&format!("VERTEX{}", pre));
                            pointcount += 1;
                        }
                        poly.push_str(&format!("VERTEX{}", apu));
                        pointcount += 1;
                    } else if pointcount > 1 {
                        if valx < minx || valy < miny {
                            poly.push_str(&format!("VERTEX{}", apu));
                        }
                        if !poly.contains("SEQEND") {
                            poly.push_str("SEQEND\r\n0\r\n");
                        }
                        out.push_str(&poly);
                        poly = format!("POLYLINE{}", head);
                        pointcount = 0;
                    }
                    pre = apu2;
                    prex = valx;
                    prey = valy;
                }
                if !poly.contains("SEQEND") {
                    poly.push_str("SEQEND\r\n  0\r\n");
                }
                if pointcount > 1 {
                    out.push_str(&poly);
                }
            }
        }
    }

    if !out.contains("EOF") {
        out.push_str("ENDSEC\r\n  0\r\nEOF\r\n");
    }
    let fp = File::create(output).expect("Unable to create file");
    let mut fp = BufWriter::new(fp);
    fp.write_all(out.as_bytes()).expect("Unable to write file");
    Ok(())
}

fn pointdxfcrop(
    input: &Path,
    output: &Path,
    minx: f64,
    miny: f64,
    maxx: f64,
    maxy: f64,
) -> Result<(), Box<dyn Error>> {
    let data = fs::read_to_string(input).expect("Should have been able to read the file");
    let mut data: Vec<&str> = data.split("POINT").collect();
    let dxfhead = data[0];
    let mut out = String::new();
    out.push_str(dxfhead);
    let (d2, ending) = data[data.len() - 1]
        .split_once("ENDSEC")
        .unwrap_or((data[data.len() - 1], ""));
    let last_idx = data.len() - 1;
    let end = format!("ENDSEC{}", ending);
    data[last_idx] = d2;
    for (j, rec) in data.iter().enumerate() {
        if j > 0 {
            let val: Vec<&str> = rec.split('\n').collect();
            let val4 = val[4].trim().parse::<f64>().unwrap_or(0.0);
            let val6 = val[6].trim().parse::<f64>().unwrap_or(0.0);
            if val4 >= minx && val4 <= maxx && val6 >= miny && val6 <= maxy {
                out.push_str(&format!("POINT{}", rec));
            }
        }
    }
    out.push_str(&end);
    let fp = File::create(output).expect("Unable to create file");
    let mut fp = BufWriter::new(fp);
    fp.write_all(out.as_bytes()).expect("Unable to write file");
    Ok(())
}

fn batch_process(thread: &String) {
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let lazfolder = conf.general_section().get("lazfolder").unwrap_or("");
    let batchoutfolder = conf.general_section().get("batchoutfolder").unwrap_or("");
    let savetempfiles: bool = conf.general_section().get("savetempfiles").unwrap() == "1";
    let savetempfolders: bool = conf.general_section().get("savetempfolders").unwrap() == "1";
    let scalefactor: f64 = conf
        .general_section()
        .get("scalefactor")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);
    let vege_bitmode: bool = conf.general_section().get("vege_bitmode").unwrap_or("0") == "1";
    let zoff = conf
        .general_section()
        .get("zoffset")
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    let mut thinfactor: f64 = conf
        .general_section()
        .get("thinfactor")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);
    if thinfactor == 0.0 {
        thinfactor = 1.0;
    }

    let mut rng = rand::thread_rng();
    let randdist = distributions::Bernoulli::new(thinfactor).unwrap();

    let mut thread_name = String::new();
    if !thread.is_empty() {
        thread_name = format!("Thread {}: ", thread);
    }

    fs::create_dir_all(batchoutfolder).expect("Could not create output folder");

    let mut zip_files: Vec<String> = Vec::new();
    for element in Path::new(lazfolder).read_dir().unwrap() {
        let path = element.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "zip" {
                zip_files.push(String::from(path.to_str().unwrap()));
            }
        }
    }

    let mut laz_files: Vec<PathBuf> = Vec::new();
    for element in Path::new(lazfolder).read_dir().unwrap() {
        let path = element.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "laz" || extension == "las" {
                laz_files.push(path);
            }
        }
    }

    for laz_path in &laz_files {
        let laz = laz_path.as_path().file_name().unwrap().to_str().unwrap();
        if Path::new(&format!("{}/{}.png", batchoutfolder, laz)).exists() {
            println!("Skipping {}.png it exists already in output folder.", laz);
            continue;
        }

        println!("{}{} -> {}.png", thread_name, laz, laz);
        File::create(format!("{}/{}.png", batchoutfolder, laz)).unwrap();
        if Path::new(&format!("header{}.xyz", thread)).exists() {
            fs::remove_file(format!("header{}.xyz", thread)).unwrap();
        }

        let mut file = File::open(format!("{}/{}", lazfolder, laz)).unwrap();
        let header = Header::read_from(&mut file).unwrap();
        let minx = header.min_x;
        let miny = header.min_y;
        let maxx = header.max_x;
        let maxy = header.max_y;

        let minx2 = minx - 127.0;
        let miny2 = miny - 127.0;
        let maxx2 = maxx + 127.0;
        let maxy2 = maxy + 127.0;

        let tmp_filename = format!("temp{}.xyz", thread);
        let tmp_file = File::create(&tmp_filename).expect("Unable to create file");
        let mut tmp_fp = BufWriter::new(tmp_file);

        for laz_p in &laz_files {
            let laz = laz_p.as_path().file_name().unwrap().to_str().unwrap();
            let mut file = File::open(format!("{}/{}", lazfolder, laz)).unwrap();
            let header = Header::read_from(&mut file).unwrap();
            if header.max_x > minx2
                && header.min_x < maxx2
                && header.max_y > miny2
                && header.min_y < maxy2
            {
                let mut reader = Reader::from_path(laz_p).expect("Unable to open reader");
                for ptu in reader.points() {
                    let pt = ptu.unwrap();
                    if pt.x > minx2
                        && pt.x < maxx2
                        && pt.y > miny2
                        && pt.y < maxy2
                        && (thinfactor == 1.0 || rng.sample(randdist))
                    {
                        write!(
                            &mut tmp_fp,
                            "{} {} {} {} {} {} {}\r\n",
                            pt.x,
                            pt.y,
                            pt.z + zoff,
                            u8::from(pt.classification),
                            pt.number_of_returns,
                            pt.return_number,
                            pt.intensity
                        )
                        .expect("Could not write temp file");
                    }
                }
            }
        }
        tmp_fp.flush().unwrap();

        if zip_files.is_empty() {
            process_tile(thread, &format!("temp{}.xyz", thread), false).unwrap();
        } else {
            process_tile(thread, &format!("temp{}.xyz", thread), true).unwrap();
            process_zip(thread, &zip_files).unwrap();
        }

        // crop
        if Path::new(&format!("pullautus{}.png", thread)).exists() {
            let path = format!("pullautus{}.pgw", thread);
            let tfw_in = Path::new(&path);

            let mut lines = read_lines(tfw_in).expect("PGW file does not exist");
            let tfw0 = lines
                .next()
                .expect("no 1 line")
                .expect("Could not read line 1")
                .parse::<f64>()
                .unwrap();
            let tfw1 = lines
                .next()
                .expect("no 2 line")
                .expect("Could not read line 2")
                .parse::<f64>()
                .unwrap();
            let tfw2 = lines
                .next()
                .expect("no 3 line")
                .expect("Could not read line 3")
                .parse::<f64>()
                .unwrap();
            let tfw3 = lines
                .next()
                .expect("no 4 line")
                .expect("Could not read line 4")
                .parse::<f64>()
                .unwrap();
            let tfw4 = lines
                .next()
                .expect("no 5 line")
                .expect("Could not read line 5")
                .parse::<f64>()
                .unwrap();
            let tfw5 = lines
                .next()
                .expect("no 6 line")
                .expect("Could not read line 6")
                .parse::<f64>()
                .unwrap();

            let dx = minx - tfw4;
            let dy = -maxy + tfw5;

            let pgw_file_out = File::create(tfw_in).expect("Unable to create file");
            let mut pgw_file_out = BufWriter::new(pgw_file_out);
            write!(
                &mut pgw_file_out,
                "{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n",
                tfw0,
                tfw1,
                tfw2,
                tfw3,
                minx + tfw0 / 2.0,
                maxy - tfw0 / 2.0
            )
            .expect("Unable to write to file");

            pgw_file_out.flush().unwrap();
            fs::copy(
                Path::new(&format!("pullautus{}.pgw", thread)),
                Path::new(&format!("pullautus_depr{}.pgw", thread)),
            )
            .expect("Could not copy file");

            let orig_img = image::open(Path::new(&format!("pullautus{}.png", thread)))
                .expect("Opening image failed");
            let mut img = RgbImage::from_pixel(
                ((maxx - minx) * 600.0 / 254.0 / scalefactor + 2.0) as u32,
                ((maxy - miny) * 600.0 / 254.0 / scalefactor + 2.0) as u32,
                Rgb([255, 255, 255]),
            );
            image::imageops::overlay(
                &mut img,
                &orig_img.to_rgb8(),
                (-dx * 600.0 / 254.0 / scalefactor) as i64,
                (-dy * 600.0 / 254.0 / scalefactor) as i64,
            );
            img.save(Path::new(&format!("pullautus{}.png", thread)))
                .expect("could not save output png");

            let orig_img = image::open(Path::new(&format!("pullautus_depr{}.png", thread)))
                .expect("Opening image failed");
            let mut img = RgbImage::from_pixel(
                ((maxx - minx) * 600.0 / 254.0 / scalefactor + 2.0) as u32,
                ((maxy - miny) * 600.0 / 254.0 / scalefactor + 2.0) as u32,
                Rgb([255, 255, 255]),
            );
            image::imageops::overlay(
                &mut img,
                &orig_img.to_rgb8(),
                (-dx * 600.0 / 254.0 / scalefactor) as i64,
                (-dy * 600.0 / 254.0 / scalefactor) as i64,
            );
            img.save(Path::new(&format!("pullautus_depr{}.png", thread)))
                .expect("could not save output png");

            fs::copy(
                Path::new(&format!("pullautus{}.png", thread)),
                Path::new(&format!("{}/{}.png", batchoutfolder, laz)),
            )
            .expect("Could not copy file to output folder");
            fs::copy(
                Path::new(&format!("pullautus{}.pgw", thread)),
                Path::new(&format!("{}/{}.pgw", batchoutfolder, laz)),
            )
            .expect("Could not copy file to output folder");
            fs::copy(
                Path::new(&format!("pullautus_depr{}.png", thread)),
                Path::new(&format!("{}/{}_depr.png", batchoutfolder, laz)),
            )
            .expect("Could not copy file to output folder");
            fs::copy(
                Path::new(&format!("pullautus_depr{}.pgw", thread)),
                Path::new(&format!("{}/{}_depr.pgw", batchoutfolder, laz)),
            )
            .expect("Could not copy file to output folder");
        }

        if savetempfiles {
            let path = format!("temp{}/undergrowth.pgw", thread);
            let tfw_in = Path::new(&path);
            let mut lines = read_lines(tfw_in).expect("PGW file does not exist");
            let tfw0 = lines
                .next()
                .expect("no 1 line")
                .expect("Could not read line 1")
                .parse::<f64>()
                .unwrap();
            let tfw1 = lines
                .next()
                .expect("no 2 line")
                .expect("Could not read line 2")
                .parse::<f64>()
                .unwrap();
            let tfw2 = lines
                .next()
                .expect("no 3 line")
                .expect("Could not read line 3")
                .parse::<f64>()
                .unwrap();
            let tfw3 = lines
                .next()
                .expect("no 4 line")
                .expect("Could not read line 4")
                .parse::<f64>()
                .unwrap();
            let tfw4 = lines
                .next()
                .expect("no 5 line")
                .expect("Could not read line 5")
                .parse::<f64>()
                .unwrap();
            let tfw5 = lines
                .next()
                .expect("no 6 line")
                .expect("Could not read line 6")
                .parse::<f64>()
                .unwrap();

            let dx = minx - tfw4;
            let dy = -maxy + tfw5;

            let pgw_file_out = File::create(Path::new(&format!(
                "{}/{}_undergrowth.pgw",
                batchoutfolder, laz
            )))
            .expect("Unable to create file");
            let mut pgw_file_out = BufWriter::new(pgw_file_out);
            write!(
                &mut pgw_file_out,
                "{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n{}\r\n",
                tfw0,
                tfw1,
                tfw2,
                tfw3,
                minx + tfw0 / 2.0,
                maxy - tfw0 / 2.0
            )
            .expect("Unable to write to file");
            pgw_file_out.flush().unwrap();

            let mut orig_img_reader =
                image::io::Reader::open(Path::new(&format!("temp{}/undergrowth.png", thread)))
                    .expect("Opening undergrowth image failed");
            orig_img_reader.no_limits();
            let orig_img = orig_img_reader.decode().unwrap();
            let mut img = RgbaImage::from_pixel(
                ((maxx - minx) * 600.0 / 254.0 / scalefactor + 2.0) as u32,
                ((maxy - miny) * 600.0 / 254.0 / scalefactor + 2.0) as u32,
                Rgba([255, 255, 255, 0]),
            );
            image::imageops::overlay(
                &mut img,
                &orig_img,
                (-dx * 600.0 / 254.0 / scalefactor) as i64,
                (-dy * 600.0 / 254.0 / scalefactor) as i64,
            );
            img.save(Path::new(&format!(
                "{}/{}_undergrowth.png",
                batchoutfolder, laz
            )))
            .expect("could not save output png");

            let mut orig_img_reader =
                image::io::Reader::open(Path::new(&format!("temp{}/vegetation.png", thread)))
                    .expect("Opening vegetation image failed");
            orig_img_reader.no_limits();
            let orig_img = orig_img_reader.decode().unwrap();
            let mut img = RgbImage::from_pixel(
                ((maxx - minx) + 1.0) as u32,
                ((maxy - miny) + 1.0) as u32,
                Rgb([255, 255, 255]),
            );
            image::imageops::overlay(&mut img, &orig_img.to_rgb8(), -dx as i64, -dy as i64);
            img.save(Path::new(&format!("{}/{}_vege.png", batchoutfolder, laz)))
                .expect("could not save output png");

            let pgw_file_out = File::create(&format!("{}/{}_vege.pgw", batchoutfolder, laz))
                .expect("Unable to create file");
            let mut pgw_file_out = BufWriter::new(pgw_file_out);
            write!(
                &mut pgw_file_out,
                "1.0\r\n0.0\r\n0.0\r\n-1.0\r\n{}\r\n{}\r\n",
                minx + 0.5,
                maxy - 0.5
            )
            .expect("Unable to write to file");

            pgw_file_out.flush().unwrap();

            if vege_bitmode {
                let mut orig_img_reader = image::io::Reader::open(Path::new(&format!(
                    "temp{}/vegetation_bit.png",
                    thread
                )))
                .expect("Opening vegetation bit image failed");
                orig_img_reader.no_limits();
                let orig_img = orig_img_reader.decode().unwrap();
                let mut img = GrayImage::from_pixel(
                    ((maxx - minx) + 1.0) as u32,
                    ((maxy - miny) + 1.0) as u32,
                    Luma([0]),
                );
                image::imageops::overlay(&mut img, &orig_img.to_luma8(), -dx as i64, -dy as i64);
                img.save(Path::new(&format!(
                    "{}/{}_vege_bit.png",
                    batchoutfolder, laz
                )))
                .expect("could not save output png");

                let mut orig_img_reader = image::io::Reader::open(Path::new(&format!(
                    "temp{}/undergrowth_bit.png",
                    thread
                )))
                .expect("Opening undergrowth bit image failed");
                orig_img_reader.no_limits();
                let orig_img = orig_img_reader.decode().unwrap();
                let mut img = GrayImage::from_pixel(
                    ((maxx - minx) + 1.0) as u32,
                    ((maxy - miny) + 1.0) as u32,
                    Luma([0]),
                );
                image::imageops::overlay(&mut img, &orig_img.to_luma8(), -dx as i64, -dy as i64);
                img.save(Path::new(&format!(
                    "{}/{}_undergrowth_bit.png",
                    batchoutfolder, laz
                )))
                .expect("could not save output png");

                fs::copy(
                    Path::new(&format!("{}/{}_vege.pgw", batchoutfolder, laz)),
                    Path::new(&format!("{}/{}_vege_bit.pgw", batchoutfolder, laz)),
                )
                .expect("Could not copy file");

                fs::copy(
                    Path::new(&format!("{}/{}_vege.pgw", batchoutfolder, laz)),
                    Path::new(&format!("{}/{}_undergrowth_bit.pgw", batchoutfolder, laz)),
                )
                .expect("Could not copy file");
            }

            if Path::new(&format!("temp{}/out2.dxf", thread)).exists() {
                polylinedxfcrop(
                    Path::new(&format!("temp{}/out2.dxf", thread)),
                    Path::new(&format!("{}/{}_contours.dxf", batchoutfolder, laz)),
                    minx,
                    miny,
                    maxx,
                    maxy,
                )
                .unwrap();
            }
            let dxf_files = ["c2g", "c3g", "contours03", "detected", "formlines"];
            for dxf_file in dxf_files.iter() {
                if Path::new(&format!("temp{}/{}.dxf", thread, dxf_file)).exists() {
                    polylinedxfcrop(
                        Path::new(&format!("temp{}/{}.dxf", thread, dxf_file)),
                        Path::new(&format!("{}/{}_{}.dxf", batchoutfolder, laz, dxf_file)),
                        minx,
                        miny,
                        maxx,
                        maxy,
                    )
                    .unwrap();
                }
            }
            if Path::new(&format!("temp{}/dotknolls.dxf", thread)).exists() {
                pointdxfcrop(
                    Path::new(&format!("temp{}/dotknolls.dxf", thread)),
                    Path::new(&format!("{}/{}_dotknolls.dxf", batchoutfolder, laz)),
                    minx,
                    miny,
                    maxx,
                    maxy,
                )
                .unwrap();
            }
        }

        if Path::new(&format!("temp{}/basemap.dxf", thread)).exists() {
            polylinedxfcrop(
                Path::new(&format!("temp{}/basemap.dxf", thread)),
                Path::new(&format!("{}/{}_basemap.dxf", batchoutfolder, laz)),
                minx,
                miny,
                maxx,
                maxy,
            )
            .unwrap();
        }

        if savetempfolders {
            fs::create_dir_all(format!("temp_{}_dir", laz))
                .expect("Could not create output folder");
            for element in Path::new(&format!("temp{}", thread)).read_dir().unwrap() {
                let path = element.unwrap().path();
                if path.is_file() {
                    let filename = &path.as_path().file_name().unwrap().to_str().unwrap();
                    fs::copy(&path, Path::new(&format!("temp_{}_dir/{}", laz, filename))).unwrap();
                }
            }
        }
    }
}
