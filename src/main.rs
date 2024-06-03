use image::{GrayImage, Luma, Rgb, RgbImage, Rgba, RgbaImage};
use imageproc::drawing::{draw_filled_circle_mut, draw_filled_rect_mut, draw_line_segment_mut};
use imageproc::filter::median_filter;
use imageproc::rect::Rect;
use ini::Ini;
use las::{raw::Header, Read, Reader};
use rand::distributions;
use rand::prelude::*;
use regex::Regex;
use rustc_hash::FxHashMap as HashMap;
use shapefile::dbase::{FieldValue, Record};
use shapefile::{Shape, ShapeType};
use std::env;
use std::error::Error;
use std::f32::consts::SQRT_2;
use std::f64::consts::PI;
use std::fs::{self, File};
use std::io::{self, BufRead, BufWriter, Write};
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
        blocks(&thread).unwrap();
        return;
    }

    if command == "dotknolls" {
        dotknolls(&thread).unwrap();
        return;
    }

    if command == "dxfmerge" || command == "merge" {
        dxfmerge().unwrap();
        if command == "merge" {
            let mut scale = 1.0;
            if !args.is_empty() {
                scale = args[0].parse::<f64>().unwrap();
            }
            pngmergevege(scale).unwrap();
        }
        return;
    }

    if command == "knolldetector" {
        knolldetector(&thread).unwrap();
        return;
    }

    if command == "makecliffs" {
        makecliffs(&thread).unwrap();
        return;
    }

    if command == "makevegenew" {
        makevegenew(&thread).unwrap();
    }

    if command == "pngmerge" || command == "pngmergedepr" {
        let mut scale = 4.0;
        if !args.is_empty() {
            scale = args[0].parse::<f64>().unwrap();
        }
        pngmerge(scale, command == "pngmergedepr").unwrap();
        return;
    }

    if command == "pngmergevege" {
        let mut scale = 1.0;
        if !args.is_empty() {
            scale = args[0].parse::<f64>().unwrap();
        }
        pngmergevege(scale).unwrap();
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
        smoothjoin(&thread).unwrap();
    }

    if command == "xyzknolls" {
        xyzknolls(&thread).unwrap();
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
        xyz2contours(
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

fn dxfmerge() -> Result<(), Box<dyn Error>> {
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let batchoutfolder = conf.general_section().get("batchoutfolder").unwrap_or("");

    let mut dxf_files: Vec<PathBuf> = Vec::new();
    for element in Path::new(batchoutfolder).read_dir().unwrap() {
        let path = element.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "dxf" {
                dxf_files.push(path);
            }
        }
    }

    if dxf_files.is_empty() {
        println!("No dxf files found in output directory");
        return Ok(());
    }

    let out2_file = File::create("merged.dxf").expect("Unable to create file");
    let mut out2 = BufWriter::new(out2_file);
    let out_file = File::create("merged_contours.dxf").expect("Unable to create file");
    let mut out = BufWriter::new(out_file);

    let mut headprinted = false;
    let mut footer = String::new();
    let mut headout = String::new();

    for dx in dxf_files.iter() {
        let dxf = dx.as_path().file_name().unwrap().to_str().unwrap();
        let dxf_filename = format!("{}/{}", batchoutfolder, dxf);

        if Path::new(&dxf_filename).exists() && dxf_filename.ends_with("contours.dxf") {
            let input = Path::new(&dxf_filename);
            let data = fs::read_to_string(input).expect("Can not read input file");
            if data.contains("POLYLINE") {
                let d: Vec<&str> = data.splitn(2, "POLYLINE").collect();
                let head = d[0];
                let body = d[1];
                let d: Vec<&str> = body.splitn(2, "ENDSEC").collect();
                let body = d[0];
                footer = String::from(d[1]);

                if !headprinted {
                    headout = String::from(head);
                    out.write_all(head.as_bytes())
                        .expect("Could not write to file");
                    out2.write_all(head.as_bytes())
                        .expect("Could not write to file");
                    headprinted = true;
                }

                out.write_all("POLYLINE".as_bytes())
                    .expect("Could not write to file");
                out.write_all(body.as_bytes())
                    .expect("Could not write to file");

                let plines: Vec<&str> = body.split("POLYLINE").collect();
                for pl in plines.iter() {
                    if !pl.contains("_intermed") {
                        out2.write_all("POLYLINE".as_bytes())
                            .expect("Could not write to file");
                        out2.write_all(pl.as_bytes())
                            .expect("Could not write to file");
                    }
                }
            }
        }
    }
    write!(&mut out, "ENDSEC{}", &footer).expect("Could not write to file");

    headprinted = false;

    let out_file = File::create("merged_c2f.dxf").expect("Unable to create file");
    let mut out = BufWriter::new(out_file);

    for dx in dxf_files.iter() {
        let dxf = dx.as_path().file_name().unwrap().to_str().unwrap();
        let dxf_filename = format!("{}/{}", batchoutfolder, dxf);
        if Path::new(&dxf_filename).exists() && dxf_filename.ends_with("_c2f.dxf") {
            let input = Path::new(&dxf_filename);
            let data = fs::read_to_string(input).expect("Can not read input file");
            if data.contains("POLYLINE") {
                let d: Vec<&str> = data.splitn(2, "POLYLINE").collect();
                let body = d[1];
                let d: Vec<&str> = body.splitn(2, "ENDSEC").collect();
                let body = d[0];
                footer = String::from(d[1]);

                if !headprinted {
                    out.write_all(headout.as_bytes())
                        .expect("Could not write to file");
                    headprinted = true;
                }

                out.write_all("POLYLINE".as_bytes())
                    .expect("Could not write to file");
                out.write_all(body.as_bytes())
                    .expect("Could not write to file");

                out2.write_all("POLYLINE".as_bytes())
                    .expect("Could not write to file");
                out2.write_all(body.as_bytes())
                    .expect("Could not write to file");
            }
        }
    }
    write!(&mut out, "ENDSEC{}", &footer).expect("Could not write to file");

    headprinted = false;

    let out_file = File::create("merged_c2.dxf").expect("Unable to create file");
    let mut out = BufWriter::new(out_file);

    for dx in dxf_files.iter() {
        let dxf = dx.as_path().file_name().unwrap().to_str().unwrap();
        let dxf_filename = format!("{}/{}", batchoutfolder, dxf);
        if Path::new(&dxf_filename).exists() && dxf_filename.ends_with("_c2g.dxf") {
            let input = Path::new(&dxf_filename);
            let data = fs::read_to_string(input).expect("Can not read input file");
            if data.contains("POLYLINE") {
                let d: Vec<&str> = data.splitn(2, "POLYLINE").collect();
                let body = d[1];
                let d: Vec<&str> = body.splitn(2, "ENDSEC").collect();
                let body = d[0];
                footer = String::from(d[1]);

                if !headprinted {
                    out.write_all(headout.as_bytes())
                        .expect("Could not write to file");
                    headprinted = true;
                }

                out.write_all("POLYLINE".as_bytes())
                    .expect("Could not write to file");
                out.write_all(body.as_bytes())
                    .expect("Could not write to file");

                out2.write_all("POLYLINE".as_bytes())
                    .expect("Could not write to file");
                out2.write_all(body.as_bytes())
                    .expect("Could not write to file");
            }
        }
    }

    write!(&mut out, "ENDSEC{}", &footer).expect("Could not write to file");

    headprinted = false;

    let basemapcontours: f64 = conf
        .general_section()
        .get("basemapinterval")
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);
    if basemapcontours > 0.0 {
        let out_file = File::create("merged_basemap.dxf").expect("Unable to create file");
        let mut out = BufWriter::new(out_file);

        for dx in dxf_files.iter() {
            let dxf = dx.as_path().file_name().unwrap().to_str().unwrap();
            let dxf_filename = format!("{}/{}", batchoutfolder, dxf);
            if Path::new(&dxf_filename).exists() && dxf_filename.ends_with("_basemap.dxf") {
                let input = Path::new(&dxf_filename);
                let data = fs::read_to_string(input).expect("Can not read input file");
                if data.contains("POLYLINE") {
                    let d: Vec<&str> = data.splitn(2, "POLYLINE").collect();
                    let body = d[1];
                    let d: Vec<&str> = body.splitn(2, "ENDSEC").collect();
                    let body = d[0];
                    footer = String::from(d[1]);

                    if !headprinted {
                        out.write_all(headout.as_bytes())
                            .expect("Could not write to file");
                        headprinted = true;
                    }

                    out.write_all("POLYLINE".as_bytes())
                        .expect("Could not write to file");
                    out.write_all(body.as_bytes())
                        .expect("Could not write to file");

                    out2.write_all("POLYLINE".as_bytes())
                        .expect("Could not write to file");
                    out2.write_all(body.as_bytes())
                        .expect("Could not write to file");
                }
            }
        }
        write!(&mut out, "ENDSEC{}", &footer).expect("Could not write to file");

        headprinted = false;
    }

    let out_file = File::create("merged_c3.dxf").expect("Unable to create file");
    let mut out = BufWriter::new(out_file);

    for dx in dxf_files.iter() {
        let dxf = dx.as_path().file_name().unwrap().to_str().unwrap();
        let dxf_filename = format!("{}/{}", batchoutfolder, dxf);
        if Path::new(&dxf_filename).exists() && dxf_filename.ends_with("_c3g.dxf") {
            let input = Path::new(&dxf_filename);
            let data = fs::read_to_string(input).expect("Can not read input file");
            if data.contains("POLYLINE") {
                let d: Vec<&str> = data.splitn(2, "POLYLINE").collect();
                let body = d[1];
                let d: Vec<&str> = body.splitn(2, "ENDSEC").collect();
                let body = d[0];
                footer = String::from(d[1]);

                if !headprinted {
                    out.write_all(headout.as_bytes())
                        .expect("Could not write to file");
                    headprinted = true;
                }

                out.write_all("POLYLINE".as_bytes())
                    .expect("Could not write to file");
                out.write_all(body.as_bytes())
                    .expect("Could not write to file");

                out2.write_all("POLYLINE".as_bytes())
                    .expect("Could not write to file");
                out2.write_all(body.as_bytes())
                    .expect("Could not write to file");
            }
        }
    }
    write!(&mut out, "ENDSEC{}", &footer).expect("Could not write to file");

    headprinted = false;

    let out_file = File::create("formlines.dxf").expect("Unable to create file");
    let mut out = BufWriter::new(out_file);

    for dx in dxf_files.iter() {
        let dxf = dx.as_path().file_name().unwrap().to_str().unwrap();
        let dxf_filename = format!("{}/{}", batchoutfolder, dxf);
        if Path::new(&dxf_filename).exists() && dxf_filename.ends_with("_formlines.dxf") {
            let input = Path::new(&dxf_filename);
            let data = fs::read_to_string(input).expect("Can not read input file");
            if data.contains("POLYLINE") {
                let d: Vec<&str> = data.splitn(2, "POLYLINE").collect();
                let body = d[1];
                let d: Vec<&str> = body.splitn(2, "ENDSEC").collect();
                let body = d[0];
                footer = String::from(d[1]);

                if !headprinted {
                    out.write_all(headout.as_bytes())
                        .expect("Could not write to file");
                    headprinted = true;
                }

                out.write_all("POLYLINE".as_bytes())
                    .expect("Could not write to file");
                out.write_all(body.as_bytes())
                    .expect("Could not write to file");

                out2.write_all("POLYLINE".as_bytes())
                    .expect("Could not write to file");
                out2.write_all(body.as_bytes())
                    .expect("Could not write to file");
            }
        }
    }
    write!(&mut out, "ENDSEC{}", &footer).expect("Could not write to file");

    headprinted = false;

    let out_file = File::create("merged_dotknolls.dxf").expect("Unable to create file");
    let mut out = BufWriter::new(out_file);

    for dx in dxf_files.iter() {
        let dxf = dx.as_path().file_name().unwrap().to_str().unwrap();
        let dxf_filename = format!("{}/{}", batchoutfolder, dxf);
        if Path::new(&dxf_filename).exists() && dxf_filename.ends_with("_dotknolls.dxf") {
            let input = Path::new(&dxf_filename);
            let data = fs::read_to_string(input).expect("Can not read input file");
            if data.contains("POINT") {
                let d: Vec<&str> = data.splitn(2, "POINT").collect();
                let body = d[1];
                let d: Vec<&str> = body.splitn(2, "ENDSEC").collect();
                let body = d[0];
                footer = String::from(d[1]);

                if !headprinted {
                    out.write_all(headout.as_bytes())
                        .expect("Could not write to file");
                    headprinted = true;
                }

                out.write_all("POINT".as_bytes())
                    .expect("Could not write to file");
                out.write_all(body.as_bytes())
                    .expect("Could not write to file");

                out2.write_all("POINT".as_bytes())
                    .expect("Could not write to file");
                out2.write_all(body.as_bytes())
                    .expect("Could not write to file");
            }
        }
    }
    write!(&mut out, "ENDSEC{}", &footer).expect("Could not write to file");

    headprinted = false;

    let out_file = File::create("merged_detected.dxf").expect("Unable to create file");
    let mut out = BufWriter::new(out_file);

    for dx in dxf_files.iter() {
        let dxf = dx.as_path().file_name().unwrap().to_str().unwrap();
        let dxf_filename = format!("{}/{}", batchoutfolder, dxf);
        if Path::new(&dxf_filename).exists() && dxf_filename.ends_with("_detected.dxf") {
            let input = Path::new(&dxf_filename);
            let data = fs::read_to_string(input).expect("Can not read input file");
            if data.contains("POLYLINE") {
                let d: Vec<&str> = data.splitn(2, "POLYLINE").collect();
                let body = d[1];
                let d: Vec<&str> = body.splitn(2, "ENDSEC").collect();
                let body = d[0];
                footer = String::from(d[1]);

                if !headprinted {
                    out.write_all(headout.as_bytes())
                        .expect("Could not write to file");
                    headprinted = true;
                }

                out.write_all("POLYLINE".as_bytes())
                    .expect("Could not write to file");
                out.write_all(body.as_bytes())
                    .expect("Could not write to file");
            }
        }
    }
    write!(&mut out, "ENDSEC{}", &footer).expect("Could not write to file");
    write!(&mut out2, "ENDSEC{}", &footer).expect("Could not write to file");

    Ok(())
}

fn merge_png(png_files: Vec<PathBuf>, outfilename: &str, scale: f64) -> Result<(), Box<dyn Error>> {
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let batchoutfolder = conf.general_section().get("batchoutfolder").unwrap_or("");

    let mut xmin = f64::MAX;
    let mut ymin = f64::MAX;
    let mut xmax = f64::MIN;
    let mut ymax = f64::MIN;
    let mut res = f64::NAN;
    for png in png_files.iter() {
        let filename = png.as_path().file_name().unwrap().to_str().unwrap();
        let full_filename = format!("{}/{}", batchoutfolder, filename);
        let img = image::open(Path::new(&full_filename)).expect("Opening image failed");
        let width = img.width() as f64;
        let height = img.height() as f64;
        let pgw = full_filename.replace(".png", ".pgw");
        if Path::new(&pgw).exists() {
            let input = Path::new(&pgw);
            let data = fs::read_to_string(input).expect("Can not read input file");
            let d: Vec<&str> = data.split('\n').collect();
            let tfw0 = d[0].trim().parse::<f64>().unwrap();
            let tfw4 = d[4].trim().parse::<f64>().unwrap();
            let tfw5 = d[5].trim().parse::<f64>().unwrap();

            if res.is_nan() {
                res = tfw0;
            }
            if tfw4 < xmin {
                xmin = tfw4;
            }
            if (tfw4 + width * res) > xmax {
                xmax = tfw4 + width * res;
            }
            if tfw5 > ymax {
                ymax = tfw5;
            }
            if (tfw5 - height * res) < ymin {
                ymin = tfw5 - height * res;
            }
        }
    }
    let mut im = RgbImage::from_pixel(
        ((xmax - xmin) / res / scale) as u32,
        ((ymax - ymin) / res / scale) as u32,
        Rgb([255, 255, 255]),
    );
    for png in png_files.iter() {
        let filename = png.as_path().file_name().unwrap().to_str().unwrap();
        let png = format!("{}/{}", batchoutfolder, filename);
        let pgw = png.replace(".png", ".pgw");
        let filesize = Path::new(&png).metadata().unwrap().len();
        if Path::new(&png).exists() && Path::new(&pgw).exists() && filesize > 0 {
            let img = image::open(Path::new(&png)).expect("Opening image failed");
            let width = img.width() as f64;
            let height = img.height() as f64;

            let input = Path::new(&pgw);
            let data = fs::read_to_string(input).expect("Can not read input file");
            let d: Vec<&str> = data.split('\n').collect();
            let tfw4 = d[4].trim().parse::<f64>().unwrap();
            let tfw5 = d[5].trim().parse::<f64>().unwrap();

            let img2 = image::imageops::thumbnail(
                &img.to_rgb8(),
                (width / scale + 0.5) as u32,
                (height / scale + 0.5) as u32,
            );
            image::imageops::overlay(
                &mut im,
                &img2,
                ((tfw4 - xmin) / res / scale) as i64,
                ((-tfw5 + ymax) / res / scale) as i64,
            );
        }
    }
    im.save(Path::new(&format!("{}.jpg", outfilename)))
        .expect("could not save output jpg");
    im.save(Path::new(&format!("{}.png", outfilename)))
        .expect("could not save output png");

    let tfw_file = File::create(format!("{}.pgw", outfilename)).expect("Unable to create file");
    let mut tfw_out = BufWriter::new(tfw_file);
    write!(
        &mut tfw_out,
        "{}\r\n0\r\n0\r\n{}\r\n{}\r\n{}\r\n",
        res * scale,
        -res * scale,
        xmin,
        ymax
    )
    .expect("Could not write to file");
    tfw_out.flush().expect("Cannot flush");
    fs::copy(
        Path::new(&format!("{}.pgw", outfilename)),
        Path::new(&format!("{}.jgw", outfilename)),
    )
    .expect("Could not copy file");
    Ok(())
}

fn pngmergevege(scale: f64) -> Result<(), Box<dyn Error>> {
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let batchoutfolder = conf.general_section().get("batchoutfolder").unwrap_or("");

    let mut png_files: Vec<PathBuf> = Vec::new();
    for element in Path::new(batchoutfolder).read_dir().unwrap() {
        let path = element.unwrap().path();
        let filename = &path.as_path().file_name().unwrap().to_str().unwrap();
        if filename.ends_with("_vege.png") {
            png_files.push(path);
        }
    }
    if png_files.is_empty() {
        println!("No _vege.png files found in output directory");
        return Ok(());
    }
    merge_png(png_files, "merged_vege", scale).unwrap();
    Ok(())
}

fn pngmerge(scale: f64, depr: bool) -> Result<(), Box<dyn Error>> {
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let batchoutfolder = conf.general_section().get("batchoutfolder").unwrap_or("");

    let mut png_files: Vec<PathBuf> = Vec::new();
    for element in Path::new(batchoutfolder).read_dir().unwrap() {
        let path = element.unwrap().path();
        let filename = &path.as_path().file_name().unwrap().to_str().unwrap();
        if filename.ends_with(".png")
            && !filename.ends_with("_undergrowth.png")
            && !filename.ends_with("_undergrowth_bit.png")
            && !filename.ends_with("_vege.png")
            && !filename.ends_with("_vege_bit.png")
            && ((depr && filename.ends_with("_depr.png"))
                || (!depr && !filename.ends_with("_depr.png")))
        {
            png_files.push(path);
        }
    }

    if png_files.is_empty() {
        println!("No files to merge found in output directory");
        return Ok(());
    }
    let mut outfilename = "merged";
    if depr {
        outfilename = "merged_depr";
    }
    merge_png(png_files, outfilename, scale).unwrap();
    Ok(())
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
        xyz2contours(
            thread,
            scalefactor * 0.3,
            "xyztemp.xyz",
            "xyz_03.xyz",
            "contours03.dxf",
            true,
        )
        .expect("contour generation failed");
    } else {
        xyz2contours(
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
            xyz2contours(
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
            knolldetector(thread).unwrap();
        }
        println!("{}Contour generation part 1", thread_name);
        xyzknolls(thread).unwrap();

        println!("{}Contour generation part 2", thread_name);
        if !skipknolldetection {
            // contours 2.5
            xyz2contours(
                thread,
                halfinterval,
                "xyz_knolls.xyz",
                "null",
                "out.dxf",
                false,
            )
            .unwrap();
        } else {
            xyz2contours(thread, halfinterval, "xyztemp.xyz", "null", "out.dxf", true).unwrap();
        }
        println!("{}Contour generation part 3", thread_name);
        smoothjoin(thread).unwrap();
        println!("{}Contour generation part 4", thread_name);
        dotknolls(thread).unwrap();
    }

    println!("{}Vegetation generation", thread_name);
    makevegenew(thread).unwrap();

    if !vegeonly {
        println!("{}Cliff generation", thread_name);
        makecliffs(thread).unwrap();
    }
    let detectbuildings: bool = conf.general_section().get("detectbuildings").unwrap_or("0") == "1";
    if detectbuildings {
        println!("{}Detecting buildings", thread_name);
        blocks(thread).unwrap();
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

fn smoothjoin(thread: &String) -> Result<(), Box<dyn Error>> {
    println!("Smooth curves...");
    let tmpfolder = format!("temp{}", thread);
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let scalefactor: f64 = conf
        .general_section()
        .get("scalefactor")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);
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
    let mut indexcontours: f64 = conf
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
    let jarkkos_bug: bool = conf.general_section().get("jarkkos2019").unwrap_or("0") == "1";
    let contour_interval: f64 = conf
        .general_section()
        .get("contour_interval")
        .unwrap_or("5")
        .parse::<f64>()
        .unwrap_or(5.0);
    let depression_length: usize = conf
        .general_section()
        .get("depression_length")
        .unwrap_or("181")
        .parse::<usize>()
        .unwrap_or(181);
    let halfinterval = contour_interval / 2.0 * scalefactor;
    if formline > 0.0 {
        indexcontours = 5.0 * contour_interval;
    }

    let interval = halfinterval;
    let path = format!("{}/xyz_knolls.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);
    let mut size: f64 = f64::NAN;
    let mut xstart: f64 = f64::NAN;
    let mut ystart: f64 = f64::NAN;

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

    let mut xmax: u64 = u64::MIN;
    let mut ymax: u64 = u64::MIN;
    let mut xyz: HashMap<(u64, u64), f64> = HashMap::default();
    if let Ok(lines) = read_lines(xyz_file_in) {
        for line in lines {
            let ip = line.unwrap_or(String::new());
            let mut parts = ip.split(' ');
            let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();

            let xx = ((x - xstart) / size).floor() as u64;
            let yy = ((y - ystart) / size).floor() as u64;

            xyz.insert((xx, yy), h);

            if xmax < xx {
                xmax = xx;
            }
            if ymax < yy {
                ymax = yy;
            }
        }
    }

    let mut steepness = vec![vec![f64::NAN; (ymax + 1) as usize]; (xmax + 1) as usize];
    for i in 1..xmax {
        for j in 1..ymax {
            let mut low: f64 = f64::MAX;
            let mut high: f64 = f64::MIN;
            for ii in i - 1..i + 2 {
                for jj in j - 1..j + 2 {
                    let tmp = *xyz.get(&(ii, jj)).unwrap_or(&0.0);
                    if tmp < low {
                        low = tmp;
                    }
                    if tmp > high {
                        high = tmp;
                    }
                }
            }
            steepness[i as usize][j as usize] = high - low;
        }
    }
    let input_filename = &format!("{}/out.dxf", tmpfolder);
    let input = Path::new(input_filename);
    let data = fs::read_to_string(input).expect("Can not read input file");
    let data: Vec<&str> = data.split("POLYLINE").collect();
    let mut dxfheadtmp = data[0];
    dxfheadtmp = dxfheadtmp.split("ENDSEC").collect::<Vec<&str>>()[0];
    dxfheadtmp = dxfheadtmp.split("HEADER").collect::<Vec<&str>>()[1];
    let dxfhead = &format!("HEADER{}ENDSEC", dxfheadtmp);
    let mut out = String::new();
    out.push_str("  0\r\nSECTION\r\n  2\r\n");
    out.push_str(dxfhead);
    out.push_str("\r\n  0\r\nSECTION\r\n  2\r\nENTITIES\r\n  0\r\n");

    let depr_filename = &format!("{}/depressions.txt", tmpfolder);
    let depr_output = Path::new(depr_filename);
    let depr_fp = File::create(depr_output).expect("Unable to create file");
    let mut depr_fp = BufWriter::new(depr_fp);

    let dotknoll_filename = &format!("{}/dotknolls.txt", tmpfolder);
    let dotknoll_output = Path::new(dotknoll_filename);
    let dotknoll_fp = File::create(dotknoll_output).expect("Unable to create file");
    let mut dotknoll_fp = BufWriter::new(dotknoll_fp);

    let knollhead_filename = &format!("{}/knollheads.txt", tmpfolder);
    let knollhead_output = Path::new(knollhead_filename);
    let knollhead_fp = File::create(knollhead_output).expect("Unable to create file");
    let mut knollhead_fp = BufWriter::new(knollhead_fp);

    let mut heads1: HashMap<String, usize> = HashMap::default();
    let mut heads2: HashMap<String, usize> = HashMap::default();
    let mut heads = Vec::<String>::new();
    let mut tails = Vec::<String>::new();
    let mut el_x = Vec::<Vec<f64>>::new();
    let mut el_y = Vec::<Vec<f64>>::new();
    el_x.push(vec![]);
    el_y.push(vec![]);
    heads.push(String::from("-"));
    tails.push(String::from("-"));
    for (j, rec) in data.iter().enumerate() {
        let mut x = Vec::<f64>::new();
        let mut y = Vec::<f64>::new();
        let mut xline = 0;
        let mut yline = 0;
        if j > 0 {
            let r = rec.split("VERTEX").collect::<Vec<&str>>();
            let apu = r[1];
            let val = apu.split('\n').collect::<Vec<&str>>();
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
                    x.push(val[xline].trim().parse::<f64>().unwrap());
                    y.push(val[yline].trim().parse::<f64>().unwrap());
                }
            }
            let x0 = x.first().unwrap();
            let xl = x.last().unwrap();
            let y0 = y.first().unwrap();
            let yl = y.last().unwrap();
            let head = format!("{}x{}", x0, y0);
            let tail = format!("{}x{}", xl, yl);

            heads.push(head);
            tails.push(tail);

            let head = format!("{}x{}", x0, y0);
            let tail = format!("{}x{}", xl, yl);
            el_x.push(x);
            el_y.push(y);
            if *heads1.get(&head).unwrap_or(&0) == 0 {
                heads1.insert(head, j);
            } else {
                heads2.insert(head, j);
            }
            if *heads1.get(&tail).unwrap_or(&0) == 0 {
                heads1.insert(tail, j);
            } else {
                heads2.insert(tail, j);
            }
        }
    }

    for l in 0..data.len() {
        let mut to_join = 0;
        if !el_x[l].is_empty() {
            let mut end_loop = false;
            while !end_loop {
                let tmp = *heads1.get(&heads[l]).unwrap_or(&0);
                if tmp != 0 && tmp != l && !el_x[tmp].is_empty() {
                    to_join = tmp;
                } else {
                    let tmp = *heads2.get(&heads[l]).unwrap_or(&0);
                    if tmp != 0 && tmp != l && !el_x[tmp].is_empty() {
                        to_join = tmp;
                    } else {
                        let tmp = *heads2.get(&tails[l]).unwrap_or(&0);
                        if tmp != 0 && tmp != l && !el_x[tmp].is_empty() {
                            to_join = tmp;
                        } else {
                            let tmp = *heads1.get(&tails[l]).unwrap_or(&0);
                            if tmp != 0 && tmp != l && !el_x[tmp].is_empty() {
                                to_join = tmp;
                            } else {
                                end_loop = true;
                            }
                        }
                    }
                }
                if !end_loop {
                    if tails[l] == heads[to_join] {
                        let tmp = tails[l].to_string();
                        heads2.insert(tmp, 0);
                        let tmp = tails[l].to_string();
                        heads1.insert(tmp, 0);
                        let mut to_append = el_x[to_join].to_vec();
                        el_x[l].append(&mut to_append);
                        let mut to_append = el_y[to_join].to_vec();
                        el_y[l].append(&mut to_append);
                        let tmp = tails[to_join].to_string();
                        tails[l] = tmp;
                        el_x[to_join].clear();
                    } else if tails[l] == tails[to_join] {
                        let tmp = tails[l].to_string();
                        heads2.insert(tmp, 0);
                        let tmp = tails[l].to_string();
                        heads1.insert(tmp, 0);
                        let mut to_append = el_x[to_join].to_vec();
                        to_append.reverse();
                        el_x[l].append(&mut to_append);
                        let mut to_append = el_y[to_join].to_vec();
                        to_append.reverse();
                        el_y[l].append(&mut to_append);
                        let tmp = heads[to_join].to_string();
                        tails[l] = tmp;
                        el_x[to_join].clear();
                    } else if heads[l] == tails[to_join] {
                        let tmp = heads[l].to_string();
                        heads2.insert(tmp, 0);
                        let tmp = heads[l].to_string();
                        heads1.insert(tmp, 0);
                        let to_append = el_x[to_join].to_vec();
                        el_x[l].splice(0..0, to_append);
                        let to_append = el_y[to_join].to_vec();
                        el_y[l].splice(0..0, to_append);
                        let tmp = heads[to_join].to_string();
                        heads[l] = tmp;
                        el_x[to_join].clear();
                    } else if heads[l] == heads[to_join] {
                        let tmp = heads[l].to_string();
                        heads2.insert(tmp, 0);
                        let tmp = heads[l].to_string();
                        heads1.insert(tmp, 0);
                        let mut to_append = el_x[to_join].to_vec();
                        to_append.reverse();
                        el_x[l].splice(0..0, to_append);
                        let mut to_append = el_y[to_join].to_vec();
                        to_append.reverse();
                        el_y[l].splice(0..0, to_append);
                        let tmp = tails[to_join].to_string();
                        heads[l] = tmp;
                        el_x[to_join].clear();
                    }
                }
            }
        }
    }
    for l in 0..data.len() {
        let mut el_x_len = el_x[l].len();
        if el_x_len > 0 {
            let mut skip = false;
            let mut depression = 1;
            if el_x_len < 3 {
                skip = true;
                el_x[l].clear();
            }
            let mut h = f64::NAN;
            if !skip {
                let mut mm: isize = (((el_x_len - 1) as f64) / 3.0).floor() as isize - 1;
                if mm < 0 {
                    mm = 0;
                }
                let mut m = mm as usize;
                while m < el_x_len {
                    let xm = el_x[l][m];
                    let ym = el_y[l][m];
                    if (xm - xstart) / size == ((xm - xstart) / size).floor() {
                        let xx = ((xm - xstart) / size).floor() as u64;
                        let yy = ((ym - ystart) / size).floor() as u64;
                        let h1 = *xyz.get(&(xx, yy)).unwrap_or(&0.0);
                        let h2 = *xyz.get(&(xx, yy + 1)).unwrap_or(&0.0);
                        let h3 = h1 * (yy as f64 + 1.0 - (ym - ystart) / size)
                            + h2 * ((ym - ystart) / size - yy as f64);
                        h = (h3 / interval + 0.5).floor() * interval;
                        m += el_x_len;
                    } else if m < el_x_len - 1
                        && (el_y[l][m] - ystart) / size == ((el_y[l][m] - ystart) / size).floor()
                    {
                        let xx = ((xm - xstart) / size).floor() as u64;
                        let yy = ((ym - ystart) / size).floor() as u64;
                        let h1 = *xyz.get(&(xx, yy)).unwrap_or(&0.0);
                        let h2 = *xyz.get(&(xx + 1, yy)).unwrap_or(&0.0);
                        let h3 = h1 * (xx as f64 + 1.0 - (xm - xstart) / size)
                            + h2 * ((xm - xstart) / size - xx as f64);
                        h = (h3 / interval + 0.5).floor() * interval;
                        m += el_x_len;
                    } else {
                        m += 1;
                    }
                }
            }
            if !skip
                && el_x_len < depression_length
                && el_x[l].first() == el_x[l].last()
                && el_y[l].first() == el_y[l].last()
            {
                let mut mm: isize = (((el_x_len - 1) as f64) / 3.0).floor() as isize - 1;
                if mm < 0 {
                    mm = 0;
                }
                let mut m = mm as usize;
                let mut x_avg = el_x[l][m];
                let mut y_avg = el_y[l][m];
                while m < el_x_len {
                    let xm = (el_x[l][m] - xstart) / size;
                    let ym = (el_y[l][m] - ystart) / size;
                    if m < el_x_len - 3
                        && ym == ym.floor()
                        && (xm - xm.floor()).abs() > 0.5
                        && ym.floor() != ((el_y[l][0] - ystart) / size).floor()
                        && xm.floor() != ((el_x[l][0] - xstart) / size).floor()
                    {
                        x_avg = xm.floor() * size + xstart;
                        y_avg = el_y[l][m].floor();
                        m += el_x_len;
                    }
                    m += 1;
                }
                let foo_x = ((x_avg - xstart) / size).floor() as u64;
                let foo_y = ((y_avg - ystart) / size).floor() as u64;

                let h_center = *xyz.get(&(foo_x, foo_y)).unwrap_or(&0.0);

                let mut hit = 0;

                let xtest = foo_x as f64 * size + xstart;
                let ytest = foo_y as f64 * size + ystart;

                let mut x0 = f64::NAN;
                let mut y0 = f64::NAN;
                for n in 0..el_x[l].len() {
                    let x1 = el_x[l][n];
                    let y1 = el_y[l][n];
                    if n > 0
                        && ((y0 <= ytest && ytest < y1) || (y1 <= ytest && ytest < y0))
                        && (xtest < (x1 - x0) * (ytest - y0) / (y1 - y0) + x0)
                    {
                        hit += 1;
                    }
                    x0 = x1;
                    y0 = y1;
                }
                depression = 1;
                if (h_center < h && hit % 2 == 1) || (h_center > h && hit % 2 != 1) {
                    depression = -1;
                    write!(&mut depr_fp, "{},{}", el_x[l][0], el_y[l][0])
                        .expect("Unable to write file");
                    for k in 1..el_x[l].len() {
                        write!(&mut depr_fp, "|{},{}", el_x[l][k], el_y[l][k])
                            .expect("Unable to write file");
                    }
                    writeln!(&mut depr_fp).expect("Unable to write file");
                }
                if !skip {
                    // Check if knoll is distinct enough
                    let mut steepcounter = 0;
                    let mut minele = f64::MAX;
                    let mut maxele = f64::MIN;
                    for k in 0..(el_x_len - 1) {
                        let xx = ((el_x[l][k] - xstart) / size + 0.5).floor() as usize;
                        let yy = ((el_y[l][k] - ystart) / size + 0.5).floor() as usize;
                        let ss = steepness[xx][yy];
                        if minele > h - 0.5 * ss {
                            minele = h - 0.5 * ss;
                        }
                        if maxele < h + 0.5 * ss {
                            maxele = h + 0.5 * ss;
                        }
                        if ss > 1.0 {
                            steepcounter += 1;
                        }
                    }

                    if (steepcounter as f64) < 0.4 * (el_x_len as f64 - 1.0)
                        && (jarkkos_bug || el_x_len < 41)
                        && depression as f64 * h_center - 1.9 < minele
                    {
                        if maxele - 0.45 * scalefactor * inidotknolls < minele {
                            skip = true;
                        }
                        if el_x_len < 33 && maxele - 0.75 * scalefactor * inidotknolls < minele {
                            skip = true;
                        }
                        if el_x_len < 19 && maxele - 0.9 * scalefactor * inidotknolls < minele {
                            skip = true;
                        }
                    }
                    if (steepcounter as f64) < inidotknolls * (el_x_len - 1) as f64 && el_x_len < 15
                    {
                        skip = true;
                    }
                }
            }
            if el_x_len < 5 {
                skip = true;
            }
            if !skip && el_x_len < 15 {
                // dot knoll
                let mut x_avg = 0.0;
                let mut y_avg = 0.0;
                for k in 0..(el_x_len - 1) {
                    x_avg += el_x[l][k];
                    y_avg += el_y[l][k];
                }
                x_avg /= (el_x_len - 1) as f64;
                y_avg /= (el_x_len - 1) as f64;
                write!(&mut dotknoll_fp, "{} {} {}\r\n", depression, x_avg, y_avg)
                    .expect("Unable to write to file");
                skip = true;
            }

            if !skip {
                // not skipped, lets save first coordinate pair for later form line knoll PIP analysis
                write!(&mut knollhead_fp, "{} {}\r\n", el_x[l][0], el_y[l][0])
                    .expect("Unable to write to file");
                // adaptive generalization
                if el_x_len > 101 {
                    let mut newx: Vec<f64> = vec![];
                    let mut newy: Vec<f64> = vec![];
                    let mut xpre = el_x[l][0];
                    let mut ypre = el_y[l][0];

                    newx.push(el_x[l][0]);
                    newy.push(el_y[l][0]);

                    for k in 1..(el_x_len - 1) {
                        let xx = ((el_x[l][k] - xstart) / size + 0.5).floor() as usize;
                        let yy = ((el_y[l][k] - ystart) / size + 0.5).floor() as usize;
                        let ss = steepness[xx][yy];
                        if ss.is_nan() || ss < 0.5 {
                            if ((xpre - el_x[l][k]).powi(2) + (ypre - el_y[l][k]).powi(2)).sqrt()
                                >= 4.0
                            {
                                newx.push(el_x[l][k]);
                                newy.push(el_y[l][k]);
                                xpre = el_x[l][k];
                                ypre = el_y[l][k];
                            }
                        } else {
                            newx.push(el_x[l][k]);
                            newy.push(el_y[l][k]);
                            xpre = el_x[l][k];
                            ypre = el_y[l][k];
                        }
                    }
                    newx.push(el_x[l][el_x_len - 1]);
                    newy.push(el_y[l][el_x_len - 1]);

                    el_x[l].clear();
                    el_x[l].append(&mut newx);
                    el_y[l].clear();
                    el_y[l].append(&mut newy);
                    el_x_len = el_x[l].len();
                }
                // Smoothing
                let mut dx: Vec<f64> = vec![f64::NAN; el_x_len];
                let mut dy: Vec<f64> = vec![f64::NAN; el_x_len];

                for k in 2..(el_x_len - 3) {
                    dx[k] = (el_x[l][k - 2]
                        + el_x[l][k - 1]
                        + el_x[l][k]
                        + el_x[l][k + 1]
                        + el_x[l][k + 2]
                        + el_x[l][k + 3])
                        / 6.0;
                    dy[k] = (el_y[l][k - 2]
                        + el_y[l][k - 1]
                        + el_y[l][k]
                        + el_y[l][k + 1]
                        + el_y[l][k + 2]
                        + el_y[l][k + 3])
                        / 6.0;
                }

                let mut xa: Vec<f64> = vec![f64::NAN; el_x_len];
                let mut ya: Vec<f64> = vec![f64::NAN; el_x_len];
                for k in 1..(el_x_len - 1) {
                    xa[k] = (el_x[l][k - 1] + el_x[l][k] / (0.01 + smoothing) + el_x[l][k + 1])
                        / (2.0 + 1.0 / (0.01 + smoothing));
                    ya[k] = (el_y[l][k - 1] + el_y[l][k] / (0.01 + smoothing) + el_y[l][k + 1])
                        / (2.0 + 1.0 / (0.01 + smoothing));
                }

                if el_x[l].first() == el_x[l].last() && el_y[l].first() == el_y[l].last() {
                    let vx = (el_x[l][1] + el_x[l][0] / (0.01 + smoothing) + el_x[l][el_x_len - 2])
                        / (2.0 + 1.0 / (0.01 + smoothing));
                    let vy = (el_y[l][1] + el_y[l][0] / (0.01 + smoothing) + el_y[l][el_x_len - 2])
                        / (2.0 + 1.0 / (0.01 + smoothing));
                    xa[0] = vx;
                    ya[0] = vy;
                    xa[el_x_len - 1] = vx;
                    ya[el_x_len - 1] = vy;
                } else {
                    xa[0] = el_x[l][0];
                    ya[0] = el_y[l][0];
                    xa[el_x_len - 1] = el_x[l][el_x_len - 1];
                    ya[el_x_len - 1] = el_y[l][el_x_len - 1];
                }
                for k in 1..(el_x_len - 1) {
                    el_x[l][k] = (xa[k - 1] + xa[k] / (0.01 + smoothing) + xa[k + 1])
                        / (2.0 + 1.0 / (0.01 + smoothing));
                    el_y[l][k] = (ya[k - 1] + ya[k] / (0.01 + smoothing) + ya[k + 1])
                        / (2.0 + 1.0 / (0.01 + smoothing));
                }
                if xa.first() == xa.last() && ya.first() == ya.last() {
                    let vx = (xa[1] + xa[0] / (0.01 + smoothing) + xa[el_x_len - 2])
                        / (2.0 + 1.0 / (0.01 + smoothing));
                    let vy = (ya[1] + ya[0] / (0.01 + smoothing) + ya[el_x_len - 2])
                        / (2.0 + 1.0 / (0.01 + smoothing));
                    el_x[l][0] = vx;
                    el_y[l][0] = vy;
                    el_x[l][el_x_len - 1] = vx;
                    el_y[l][el_x_len - 1] = vy;
                } else {
                    el_x[l][0] = xa[0];
                    el_y[l][0] = ya[0];
                    el_x[l][el_x_len - 1] = xa[el_x_len - 1];
                    el_y[l][el_x_len - 1] = ya[el_x_len - 1];
                }

                for k in 1..(el_x_len - 1) {
                    xa[k] = (el_x[l][k - 1] + el_x[l][k] / (0.01 + smoothing) + el_x[l][k + 1])
                        / (2.0 + 1.0 / (0.01 + smoothing));
                    ya[k] = (el_y[l][k - 1] + el_y[l][k] / (0.01 + smoothing) + el_y[l][k + 1])
                        / (2.0 + 1.0 / (0.01 + smoothing));
                }

                if el_x[l].first() == el_x[l].last() && el_y[l].first() == el_y[l].last() {
                    let vx = (el_x[l][1] + el_x[l][0] / (0.01 + smoothing) + el_x[l][el_x_len - 2])
                        / (2.0 + 1.0 / (0.01 + smoothing));
                    let vy = (el_y[l][1] + el_y[l][0] / (0.01 + smoothing) + el_y[l][el_x_len - 2])
                        / (2.0 + 1.0 / (0.01 + smoothing));
                    xa[0] = vx;
                    ya[0] = vy;
                    xa[el_x_len - 1] = vx;
                    ya[el_x_len - 1] = vy;
                } else {
                    xa[0] = el_x[l][0];
                    ya[0] = el_y[l][0];
                    xa[el_x_len - 1] = el_x[l][el_x_len - 1];
                    ya[el_x_len - 1] = el_y[l][el_x_len - 1];
                }
                for k in 0..el_x_len {
                    el_x[l][k] = xa[k];
                    el_y[l][k] = ya[k];
                }

                let mut dx2: Vec<f64> = vec![f64::NAN; el_x_len];
                let mut dy2: Vec<f64> = vec![f64::NAN; el_x_len];
                for k in 2..(el_x_len - 3) {
                    dx2[k] = (el_x[l][k - 2]
                        + el_x[l][k - 1]
                        + el_x[l][k]
                        + el_x[l][k + 1]
                        + el_x[l][k + 2]
                        + el_x[l][k + 3])
                        / 6.0;
                    dy2[k] = (el_y[l][k - 2]
                        + el_y[l][k - 1]
                        + el_y[l][k]
                        + el_y[l][k + 1]
                        + el_y[l][k + 2]
                        + el_y[l][k + 3])
                        / 6.0;
                }
                for k in 3..(el_x_len - 3) {
                    let vx = el_x[l][k] + (dx[k] - dx2[k]) * curviness;
                    let vy = el_y[l][k] + (dy[k] - dy2[k]) * curviness;
                    el_x[l][k] = vx;
                    el_y[l][k] = vy;
                }

                let mut layer = String::from("contour");
                if depression == -1 {
                    layer = String::from("depression");
                }
                if indexcontours != 0.0
                    && (((h / interval + 0.5).floor() * interval) / indexcontours).floor()
                        - ((h / interval + 0.5).floor() * interval) / indexcontours
                        == 0.0
                {
                    layer.push_str("_index");
                }
                if formline > 0.0
                    && (((h / interval + 0.5).floor() * interval) / (2.0 * interval)).floor()
                        - ((h / interval + 0.5).floor() * interval) / (2.0 * interval)
                        != 0.0
                {
                    layer.push_str("_intermed");
                }
                out.push_str(
                    format!(
                        "POLYLINE\r\n 66\r\n1\r\n  8\r\n{}\r\n 38\r\n{}\r\n  0\r\n",
                        layer, h
                    )
                    .as_str(),
                );
                for k in 0..el_x_len {
                    out.push_str(
                        format!(
                            "VERTEX\r\n  8\r\n{}\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\n",
                            layer, el_x[l][k], el_y[l][k]
                        )
                        .as_str(),
                    );
                }
                out.push_str("SEQEND\r\n  0\r\n");
            } // -- if not dotkoll
        }
    }
    out.push_str("ENDSEC\r\n  0\r\nEOF\r\n");
    let output_filename = &format!("{}/out2.dxf", tmpfolder);
    let output = Path::new(output_filename);
    let fp = File::create(output).expect("Unable to create file");
    let mut fp = BufWriter::new(fp);
    fp.write_all(out.as_bytes()).expect("Unable to write file");
    println!("Done");
    Ok(())
}

fn makecliffs(thread: &String) -> Result<(), Box<dyn Error>> {
    println!("Identifying cliffs...");
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let jarkkos_bug: bool = conf.general_section().get("jarkkos2019").unwrap_or("0") == "1";

    let c1_limit: f64 = conf
        .general_section()
        .get("cliff1")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);
    let c2_limit: f64 = conf
        .general_section()
        .get("cliff2")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);

    let cliff_thin: f64 = conf
        .general_section()
        .get("cliffthin")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);

    let steep_factor: f64 = conf
        .general_section()
        .get("cliffsteepfactor")
        .unwrap_or("0.33")
        .parse::<f64>()
        .unwrap_or(0.33);

    let flat_place: f64 = conf
        .general_section()
        .get("cliffflatplace")
        .unwrap_or("6.6")
        .parse::<f64>()
        .unwrap_or(6.6);

    let mut no_small_ciffs: f64 = conf
        .general_section()
        .get("cliffnosmallciffs")
        .unwrap_or("0")
        .parse::<f64>()
        .unwrap_or(0.0);

    if no_small_ciffs == 0.0 {
        no_small_ciffs = 6.0;
    } else {
        no_small_ciffs -= flat_place;
    }

    let mut xmin: f64 = std::f64::MAX;
    let mut xmax: f64 = std::f64::MIN;

    let mut ymin: f64 = std::f64::MAX;
    let mut ymax: f64 = std::f64::MIN;

    let mut hmin: f64 = std::f64::MAX;
    let mut hmax: f64 = std::f64::MIN;

    let tmpfolder = format!("temp{}", thread);

    let path = format!("{}/xyztemp.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);

    if let Ok(lines) = read_lines(xyz_file_in) {
        for line in lines {
            let ip = line.unwrap_or(String::new());
            let mut parts = ip.split(' ');
            let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();

            if xmin > x {
                xmin = x;
            }

            if xmax < x {
                xmax = x;
            }

            if ymin > y {
                ymin = y;
            }

            if ymax < y {
                ymax = y;
            }

            if hmin > h {
                hmin = h;
            }

            if hmax < h {
                hmax = h;
            }
        }
    }
    let path = format!("{}/xyz2.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);
    let mut size: f64 = f64::NAN;
    let mut xstart: f64 = f64::NAN;
    let mut ystart: f64 = f64::NAN;
    let mut sxmax: usize = usize::MIN;
    let mut symax: usize = usize::MIN;
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

    let mut xyz = vec![
        vec![f64::NAN; ((ymax - ystart) / size).ceil() as usize + 1];
        ((xmax - xstart) / size).ceil() as usize + 1
    ];
    if let Ok(lines) = read_lines(xyz_file_in) {
        for line in lines {
            let ip = line.unwrap_or(String::new());
            let mut parts = ip.split(' ');
            let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();

            let xx = ((x - xstart) / size).floor() as usize;
            let yy = ((y - ystart) / size).floor() as usize;

            xyz[xx][yy] = h;

            if sxmax < xx {
                sxmax = xx;
            }
            if symax < yy {
                symax = yy;
            }
        }
    }

    let mut steepness = vec![vec![f64::NAN; symax + 1]; sxmax + 1];
    for i in 3..sxmax - 4 {
        for j in 3..symax - 4 {
            let mut low: f64 = f64::MAX;
            let mut high: f64 = f64::MIN;
            for ii in i - 3..i + 4 {
                for jj in j - 3..j + 4 {
                    if xyz[ii][jj] < low {
                        low = xyz[ii][jj];
                    }
                    if xyz[ii][jj] > high {
                        high = xyz[ii][jj];
                    }
                }
            }
            steepness[i][j] = high - low;
        }
    }

    let mut img = RgbImage::from_pixel(
        (xmax - xmin).floor() as u32,
        (ymax - ymin).floor() as u32,
        Rgb([255, 255, 255]),
    );

    xmin = (xmin / 3.0).floor() * 3.0;
    ymin = (ymin / 3.0).floor() * 3.0;

    let mut list_alt =
        vec![
            vec![Vec::<(f64, f64, f64)>::new(); (((ymax - ymin) / 3.0).ceil() + 1.0) as usize];
            (((xmax - xmin) / 3.0).ceil() + 1.0) as usize
        ];

    let path = format!("{}/xyztemp.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);

    let mut rng = rand::thread_rng();
    let randdist = distributions::Bernoulli::new(cliff_thin).unwrap();

    if let Ok(lines) = read_lines(xyz_file_in) {
        for line in lines {
            if cliff_thin == 1.0 || rng.sample(randdist) {
                let ip = line.unwrap_or(String::new());
                let mut parts = ip.split(' ');
                let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                let r3 = parts.next().unwrap();

                if r3 == "2" {
                    list_alt[((x - xmin).floor() / 3.0) as usize]
                        [((y - ymin).floor() / 3.0) as usize]
                        .push((x, y, h));
                }
            }
        }
    }
    let w = ((xmax - xmin).floor() / 3.0) as usize;
    let h = ((ymax - ymin).floor() / 3.0) as usize;

    let f2 =
        File::create(Path::new(&format!("{}/c2g.dxf", tmpfolder))).expect("Unable to create file");
    let mut f2 = BufWriter::new(f2);

    write!(&mut f2,"  0\r\nSECTION\r\n  2\r\nHEADER\r\n  9\r\n$EXTMIN\r\n 10\r\n{}\r\n 20\r\n{}\r\n  9\r\n$EXTMAX\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nENDSEC\r\n  0\r\nSECTION\r\n  2\r\nENTITIES\r\n  0\r\n", xmin, ymin, xmax, ymax).expect("Cannot write dxf file");

    let f3 =
        File::create(Path::new(&format!("{}/c3g.dxf", tmpfolder))).expect("Unable to create file");
    let mut f3 = BufWriter::new(f3);

    write!(&mut f3, "  0\r\nSECTION\r\n  2\r\nHEADER\r\n  9\r\n$EXTMIN\r\n 10\r\n{}\r\n 20\r\n{}\r\n  9\r\n$EXTMAX\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nENDSEC\r\n  0\r\nSECTION\r\n  2\r\nENTITIES\r\n  0\r\n",
            xmin, ymin, xmax, ymax
    ).expect("Cannot write dxf file");

    for x in 0..w + 1 {
        for y in 0..h + 1 {
            if !list_alt[x][y].is_empty() {
                let mut t = Vec::<(f64, f64, f64)>::new();
                if x >= 1 {
                    if y >= 1 {
                        t.extend(&list_alt[x - 1][y - 1]);
                    }
                    t.extend(&list_alt[x - 1][y]);
                    if y < h {
                        t.extend(&list_alt[x - 1][y + 1]);
                    }
                }
                if y >= 1 {
                    t.extend(&list_alt[x][y - 1]);
                }
                t.extend(&list_alt[x][y]);
                if y < h {
                    t.extend(&list_alt[x][y + 1]);
                }
                if x < h {
                    if y >= 1 {
                        t.extend(&list_alt[x + 1][y - 1]);
                    }
                    t.extend(&list_alt[x + 1][y]);
                    if y < h {
                        t.extend(&list_alt[x + 1][y + 1]);
                    }
                }
                let mut d = Vec::<(f64, f64, f64)>::new();
                d.extend(&list_alt[x][y]);

                if d.len() > 31 {
                    let b = ((d.len() - 1) as f64 / 30.0).floor() as usize;
                    let mut i: usize = 0;
                    while i < d.len() {
                        let mut e = i + b;
                        if e > d.len() {
                            e = d.len();
                        }
                        let _: Vec<_> = d.drain(i..e).collect();
                        i += 1;
                    }
                }
                if t.len() > 301 {
                    let b = ((t.len() - 1) as f64 / 300.0).floor() as usize;
                    let mut i: usize = 0;
                    while i < t.len() {
                        let mut e = i + b;
                        if e > t.len() {
                            e = t.len();
                        }
                        let _: Vec<_> = t.drain(i..e).collect();
                        i += 1;
                    }
                }
                let mut temp_max: f64 = f64::MIN;
                let mut temp_min: f64 = f64::MAX;
                for rec in t.iter() {
                    let h0 = rec.2;
                    if temp_max < h0 {
                        temp_max = h0;
                    }
                    if temp_min > h0 || jarkkos_bug {
                        temp_min = h0;
                    }
                }
                if temp_max - temp_min < c1_limit * 0.999 {
                    d.clear();
                }

                for rec in d.iter() {
                    let x0 = rec.0;
                    let y0 = rec.1;
                    let h0 = rec.2;

                    let cliff_length = 1.47;
                    let mut steep = steepness[((x0 - xstart) / size + 0.5).floor() as usize]
                        [((y0 - ystart) / size + 0.5).floor() as usize]
                        - flat_place;
                    if steep.is_nan() {
                        steep = -flat_place;
                    }
                    if steep < 0.0 {
                        steep = 0.0;
                    }
                    if steep > 17.0 {
                        steep = 17.0;
                    }
                    let bonus =
                        (c2_limit - c1_limit) * (1.0 - (no_small_ciffs - steep) / no_small_ciffs);
                    let limit = c1_limit + bonus;
                    let mut bonus = c2_limit * steep_factor * (steep - no_small_ciffs);
                    if bonus < 0.0 {
                        bonus = 0.0;
                    }
                    let limit2 = c2_limit + bonus;
                    for rec2 in t.iter() {
                        let xt = rec2.0;
                        let yt = rec2.1;
                        let ht = rec2.2;

                        let temp = h0 - ht;
                        let dist = ((x0 - xt).powi(2) + (y0 - yt).powi(2)).sqrt();
                        if dist > 0.0 {
                            if steep < no_small_ciffs
                                && temp > limit
                                && temp > (limit + (dist - limit) * 0.85)
                                && (((x0 + xt) / 2.0 - xmin + 0.5).floor() as u32) < img.width()
                                && (((y0 + yt) / 2.0 - ymin + 0.5).floor() as u32) < img.height()
                            {
                                let p = img.get_pixel(
                                    ((x0 + xt) / 2.0 - xmin + 0.5).floor() as u32,
                                    ((y0 + yt) / 2.0 - ymin + 0.5).floor() as u32,
                                );
                                if p[0] == 255 {
                                    img.put_pixel(
                                        ((x0 + xt) / 2.0 - xmin + 0.5).floor() as u32,
                                        ((y0 + yt) / 2.0 - ymin + 0.5).floor() as u32,
                                        Rgb([0, 0, 0]),
                                    );
                                    f2.write_all(
                                        "POLYLINE\r\n 66\r\n1\r\n  8\r\ncliff2\r\n  0\r\n"
                                            .as_bytes(),
                                    )
                                    .expect("Cannot write dxf file");
                                    write!(
                                        &mut f2,
                                        "VERTEX\r\n  8\r\ncliff2\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nVERTEX\r\n  8\r\ncliff2\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nSEQEND\r\n  0\r\n",
                                        (x0 + xt) / 2.0 + cliff_length * (y0 - yt) / dist,
                                        (y0 + yt) / 2.0 - cliff_length * (x0 - xt) / dist,
                                        (x0 + xt) / 2.0 - cliff_length * (y0 - yt) / dist,
                                        (y0 + yt) / 2.0 + cliff_length * (x0 - xt) / dist,
                                    ).expect("Cannot write dxf file");
                                }
                            }

                            if temp > limit2 && temp > (limit2 + (dist - limit2) * 0.85) {
                                f3.write_all(
                                    "POLYLINE\r\n 66\r\n1\r\n  8\r\ncliff3\r\n  0\r\n".as_bytes(),
                                )
                                .expect("Cannot write dxf file");
                                write!(
                                    &mut f3,
                                    "VERTEX\r\n  8\r\ncliff3\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nVERTEX\r\n  8\r\ncliff3\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nSEQEND\r\n  0\r\n",
                                    (x0 + xt) / 2.0 + cliff_length * (y0 - yt) / dist,
                                    (y0 + yt) / 2.0 - cliff_length * (x0 - xt) / dist,
                                    (x0 + xt) / 2.0 - cliff_length * (y0 - yt) / dist,
                                    (y0 + yt) / 2.0 + cliff_length * (x0 - xt) / dist,
                                ).expect("Cannot write dxf file");
                            }
                        }
                    }
                }
            }
        }
    }

    f2.write_all("ENDSEC\r\n  0\r\nEOF\r\n".as_bytes())
        .expect("Cannot write dxf file");
    let c2_limit = 2.6 * 2.75;
    let path = format!("{}/xyz2.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);
    let mut list_alt =
        vec![
            vec![Vec::<(f64, f64, f64)>::new(); (((ymax - ymin) / 3.0).ceil() + 1.0) as usize];
            (((xmax - xmin) / 3.0).ceil() + 1.0) as usize
        ];

    if let Ok(lines) = read_lines(xyz_file_in) {
        for line in lines {
            if cliff_thin == 1.0 || rng.sample(randdist) {
                let ip = line.unwrap_or(String::new());
                let mut parts = ip.split(' ');
                let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();

                list_alt[((x - xmin).floor() / 3.0) as usize][((y - ymin).floor() / 3.0) as usize]
                    .push((x, y, h));
            }
        }
    }

    for x in 0..w + 1 {
        for y in 0..h + 1 {
            if !list_alt[x][y].is_empty() {
                let mut t = Vec::<(f64, f64, f64)>::new();
                if x >= 1 {
                    if y >= 1 {
                        t.extend(&list_alt[x - 1][y - 1]);
                    }
                    t.extend(&list_alt[x - 1][y]);
                    if y < h {
                        t.extend(&list_alt[x - 1][y + 1]);
                    }
                }
                if y >= 1 {
                    t.extend(&list_alt[x][y - 1]);
                }
                t.extend(&list_alt[x][y]);
                if y < h {
                    t.extend(&list_alt[x][y + 1]);
                }
                if x < h {
                    if y >= 1 {
                        t.extend(&list_alt[x + 1][y - 1]);
                    }
                    t.extend(&list_alt[x + 1][y]);
                    if y < h {
                        t.extend(&list_alt[x + 1][y + 1]);
                    }
                }
                let mut d = Vec::<(f64, f64, f64)>::new();
                d.extend(&list_alt[x][y]);

                for rec in d.iter() {
                    let x0 = rec.0;
                    let y0 = rec.1;
                    let h0 = rec.2;
                    let cliff_length = 1.47;
                    let limit = c2_limit;
                    for rec2 in t.iter() {
                        let xt = rec2.0;
                        let yt = rec2.1;
                        let ht = rec2.2;
                        let temp = h0 - ht;
                        let dist = ((x0 - xt).powi(2) + (y0 - yt).powi(2)).sqrt();
                        if dist > 0.0 && temp > limit && temp > (limit + (dist - limit) * 0.85) {
                            f3.write_all(
                                "POLYLINE\r\n 66\r\n1\r\n  8\r\ncliff4\r\n  0\r\n".as_bytes(),
                            )
                            .expect("Cannot write dxf file");
                            write!(
                                &mut f3,
                                "VERTEX\r\n  8\r\ncliff4\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nVERTEX\r\n  8\r\ncliff4\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nSEQEND\r\n  0\r\n",
                                (x0 + xt) / 2.0 + cliff_length * (y0 - yt) / dist,
                                (y0 + yt) / 2.0 - cliff_length * (x0 - xt) / dist,
                                (x0 + xt) / 2.0 - cliff_length * (y0 - yt) / dist,
                                (y0 + yt) / 2.0 + cliff_length * (x0 - xt) / dist,
                            ).expect("Cannot write dxf file");
                        }
                    }
                }
            }
        }
    }

    f3.write_all("ENDSEC\r\n  0\r\nEOF\r\n".as_bytes())
        .expect("Cannot write dxf file");
    img.save(Path::new(&format!("{}/c2.png", tmpfolder)))
        .expect("could not save output png");
    println!("Done");
    Ok(())
}

fn blocks(thread: &String) -> Result<(), Box<dyn Error>> {
    println!("Identifying blocks...");
    let tmpfolder = format!("temp{}", thread);
    let path = format!("{}/xyz2.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);
    let mut size: f64 = f64::NAN;
    let mut xstartxyz: f64 = f64::NAN;
    let mut ystartxyz: f64 = f64::NAN;
    let mut xmax: u64 = u64::MIN;
    let mut ymax: u64 = u64::MIN;
    if let Ok(lines) = read_lines(xyz_file_in) {
        for (i, line) in lines.enumerate() {
            let ip = line.unwrap_or(String::new());
            let mut parts = ip.split(' ');
            let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();

            if i == 0 {
                xstartxyz = x;
                ystartxyz = y;
            } else if i == 1 {
                size = y - ystartxyz;
            } else {
                break;
            }
        }
    }
    let mut xyz: HashMap<(u64, u64), f64> = HashMap::default();

    if let Ok(lines) = read_lines(xyz_file_in) {
        for line in lines {
            let ip = line.unwrap_or(String::new());
            let mut parts = ip.split(' ');
            let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();

            let xx = ((x - xstartxyz) / size).floor() as u64;
            let yy = ((y - ystartxyz) / size).floor() as u64;
            xyz.insert((xx, yy), h);

            if xmax < xx {
                xmax = xx;
            }
            if ymax < yy {
                ymax = yy;
            }
        }
    }
    let mut img = RgbImage::from_pixel(xmax as u32 * 2, ymax as u32 * 2, Rgb([255, 255, 255]));
    let mut img2 = RgbaImage::from_pixel(xmax as u32 * 2, ymax as u32 * 2, Rgba([0, 0, 0, 0]));

    let black = Rgb([0, 0, 0]);
    let white = Rgba([255, 255, 255, 255]);

    let path = format!("{}/xyztemp.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);
    if let Ok(lines) = read_lines(xyz_file_in) {
        for line in lines {
            let ip = line.unwrap_or(String::new());
            let mut parts = ip.split(' ');
            let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let r3 = parts.next().unwrap();
            let r4 = parts.next().unwrap();
            let r5 = parts.next().unwrap();

            let xx = ((x - xstartxyz) / size).floor() as u64;
            let yy = ((y - ystartxyz) / size).floor() as u64;
            if r3 != "2"
                && r3 != "9"
                && r4 == "1"
                && r5 == "1"
                && h - *xyz.get(&(xx, yy)).unwrap_or(&0.0) > 2.0
            {
                draw_filled_rect_mut(
                    &mut img,
                    Rect::at(
                        (x - xstartxyz - 1.0) as i32,
                        (ystartxyz + 2.0 * ymax as f64 - y - 1.0) as i32,
                    )
                    .of_size(3, 3),
                    black,
                );
            } else {
                draw_filled_rect_mut(
                    &mut img2,
                    Rect::at(
                        (x - xstartxyz - 1.0) as i32,
                        (ystartxyz + 2.0 * ymax as f64 - y - 1.0) as i32,
                    )
                    .of_size(3, 3),
                    white,
                );
            }
        }
    }
    let filter_size = 2;
    img.save(Path::new(&format!("{}/blocks.png", tmpfolder)))
        .expect("error saving png");
    img2.save(Path::new(&format!("{}/blocks2.png", tmpfolder)))
        .expect("error saving png");
    let mut img =
        image::open(Path::new(&format!("{}/blocks.png", tmpfolder))).expect("Opening image failed");
    let img2 = image::open(Path::new(&format!("{}/blocks2.png", tmpfolder)))
        .expect("Opening image failed");

    image::imageops::overlay(&mut img, &img2, 0, 0);

    img = image::DynamicImage::ImageRgb8(median_filter(&img.to_rgb8(), filter_size, filter_size));

    img.save(Path::new(&format!("{}/blocks.png", tmpfolder)))
        .expect("error saving png");
    println!("Done");
    Ok(())
}

fn dotknolls(thread: &String) -> Result<(), Box<dyn Error>> {
    println!("Identifying dotknolls...");
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let scalefactor: f64 = conf
        .general_section()
        .get("scalefactor")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);

    let tmpfolder = format!("temp{}", thread);

    let path = format!("{}/xyz_knolls.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);

    let mut xstart: f64 = 0.0;
    let mut ystart: f64 = 0.0;
    let mut size: f64 = 0.0;

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
    let mut xmax = 0.0;
    let mut ymax = 0.0;

    if let Ok(lines) = read_lines(xyz_file_in) {
        for line in lines {
            let ip = line.unwrap_or(String::new());
            let mut parts = ip.split(' ');

            // make sure we have at least 2 items
            if let (Some(r0), Some(r1)) = (parts.next(), parts.next()) {
                let x: f64 = r0.parse::<f64>().unwrap();
                let y: f64 = r1.parse::<f64>().unwrap();

                let xx = ((x - xstart) / size).floor();
                let yy = ((y - ystart) / size).floor();

                if xmax < xx {
                    xmax = xx;
                }

                if ymax < yy {
                    ymax = yy;
                }
            }
        }
    }

    let mut im = GrayImage::from_pixel(
        (xmax * size / scalefactor) as u32,
        (ymax * size / scalefactor) as u32,
        Luma([0xff]),
    );

    let f = File::create(Path::new(&format!("{}/dotknolls.dxf", tmpfolder)))
        .expect("Unable to create file");
    let mut f = BufWriter::new(f);
    write!(&mut f,
        "  0\r\nSECTION\r\n  2\r\nHEADER\r\n  9\r\n$EXTMIN\r\n 10\r\n{}\r\n 20\r\n{}\r\n  9\r\n$EXTMAX\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nENDSEC\r\n  0\r\nSECTION\r\n  2\r\nENTITIES\r\n  0\r\n",
        xstart, ystart, xmax * size + xstart, ymax * size + ystart
    ).expect("Cannot write dxf file");

    let input_filename = &format!("{}/out2.dxf", tmpfolder);
    let input = Path::new(input_filename);
    let data = fs::read_to_string(input).expect("Can not read input file");
    let data: Vec<&str> = data.split("POLYLINE").collect();

    for (j, rec) in data.iter().enumerate() {
        let mut x = Vec::<f64>::new();
        let mut y = Vec::<f64>::new();
        let mut xline = 0;
        let mut yline = 0;
        if j > 0 {
            let r = rec.split("VERTEX").collect::<Vec<&str>>();
            let apu = r[1];
            let val = apu.split('\n').collect::<Vec<&str>>();
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
                    x.push(val[xline].trim().parse::<f64>().unwrap());
                    y.push(val[yline].trim().parse::<f64>().unwrap());
                }
            }
        }
        for i in 1..x.len() {
            draw_line_segment_mut(
                &mut im,
                (
                    ((x[i - 1] - xstart) / scalefactor).floor() as f32,
                    ((y[i - 1] - ystart) / scalefactor).floor() as f32,
                ),
                (
                    ((x[i] - xstart) / scalefactor).floor() as f32,
                    ((y[i] - ystart) / scalefactor).floor() as f32,
                ),
                Luma([0x0]),
            )
        }
    }

    let input_filename = &format!("{}/dotknolls.txt", tmpfolder);
    let input = Path::new(input_filename);
    if let Ok(lines) = read_lines(input) {
        for line in lines {
            let ip = line.unwrap_or(String::new());
            let parts = ip.split(' ');
            let r = parts.collect::<Vec<&str>>();
            if r.len() >= 3 {
                let depression: bool = r[0] == "1";
                let x: f64 = r[1].parse::<f64>().unwrap();
                let y: f64 = r[2].parse::<f64>().unwrap();
                let mut ok = true;
                let mut i = (x - xstart) / scalefactor - 3.0;
                let mut layer = String::new();
                while i < (x - xstart) / scalefactor + 4.0 && ok {
                    let mut j = (y - ystart) / scalefactor - 3.0;
                    while j < (y - ystart) / scalefactor + 4.0 && ok {
                        if (i as u32) >= im.width() || (j as u32) >= im.height() {
                            ok = false;
                            break;
                        }
                        let pix = im.get_pixel(i as u32, j as u32);
                        if pix[0] == 0 {
                            ok = false;
                            break;
                        }
                        j += 1.0;
                    }
                    i += 1.0;
                }
                if !ok {
                    layer = String::from("ugly");
                }
                if depression {
                    layer.push_str("dotknoll")
                } else {
                    layer.push_str("udepression")
                }
                write!(
                    &mut f,
                    "POINT\r\n  8\r\n{}\r\n 10\r\n{}\r\n 20\r\n{}\r\n 50\r\n0\r\n  0\r\n",
                    layer, x, y
                )
                .expect("Can not write to file");
            }
        }
    }
    f.write_all("ENDSEC\r\n  0\r\nEOF\r\n".as_bytes())
        .expect("Can not write to file");
    println!("Done");
    Ok(())
}

fn xyz2contours(
    thread: &String,
    cinterval: f64,
    xyzfilein: &str,
    xyzfileout: &str,
    dxffile: &str,
    ground: bool,
) -> Result<(), Box<dyn Error>> {
    println!("Generating curves...");

    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let jarkkos_bug: bool = conf.general_section().get("jarkkos2019").unwrap_or("0") == "1";

    let scalefactor: f64 = conf
        .general_section()
        .get("scalefactor")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);
    let water_class = conf.general_section().get("waterclass").unwrap_or("9");

    let tmpfolder = format!("temp{}", thread);

    let mut xmin: f64 = std::f64::MAX;
    let mut xmax: f64 = std::f64::MIN;

    let mut ymin: f64 = std::f64::MAX;
    let mut ymax: f64 = std::f64::MIN;

    let mut hmin: f64 = std::f64::MAX;
    let mut hmax: f64 = std::f64::MIN;

    let path = format!("{}/{}", tmpfolder, xyzfilein);
    let xyz_file_in = Path::new(&path);

    read_lines_no_alloc(xyz_file_in, |line| {
        let mut parts = line.trim().split(' ');

        let p0 = parts.next().unwrap();
        let p1 = parts.next().unwrap();
        let p2 = parts.next().unwrap();
        let p3 = parts.next();

        if p3.is_some_and(|p3| p3 == "2" || p3 == water_class) || !ground {
            let x: f64 = p0.parse::<f64>().unwrap();
            let y: f64 = p1.parse::<f64>().unwrap();
            let h: f64 = p2.parse::<f64>().unwrap();

            if xmin > x {
                xmin = x;
            }

            if xmax < x {
                xmax = x;
            }

            if ymin > y {
                ymin = y;
            }

            if ymax < y {
                ymax = y;
            }

            if hmin > h {
                hmin = h;
            }

            if hmax < h {
                hmax = h;
            }
        }
    })
    .expect("could not read file");

    xmin = (xmin / 2.0 / scalefactor).floor() * 2.0 * scalefactor;
    ymin = (ymin / 2.0 / scalefactor).floor() * 2.0 * scalefactor;

    let w: usize = ((xmax - xmin).ceil() / 2.0 / scalefactor) as usize;
    let h: usize = ((ymax - ymin).ceil() / 2.0 / scalefactor) as usize;

    // a two-dimensional vector of (sum, count) pairs for computing averages
    let mut list_alt = vec![vec![(0f64, 0usize); h + 2]; w + 2];

    read_lines_no_alloc(xyz_file_in, |line| {
        let mut parts = line.trim().split(' ');

        let p0 = parts.next().unwrap();
        let p1 = parts.next().unwrap();
        let p2 = parts.next().unwrap();
        let p3 = parts.next();

        if p3.is_some_and(|p3| p3 == "2" || p3 == water_class) || !ground {
            let x: f64 = p0.parse::<f64>().unwrap();
            let y: f64 = p1.parse::<f64>().unwrap();
            let h: f64 = p2.parse::<f64>().unwrap();

            let (sum, count) = &mut list_alt[((x - xmin).floor() / 2.0 / scalefactor) as usize]
                [((y - ymin).floor() / 2.0 / scalefactor) as usize];
            *sum += h;
            *count += 1;
        }
    })
    .expect("could not read file");

    let mut avg_alt = vec![vec![f64::NAN; h + 2]; w + 2];

    for x in 0..w + 1 {
        for y in 0..h + 1 {
            let (sum, count) = &list_alt[x][y];

            if *count > 0 {
                avg_alt[x][y] = *sum / *count as f64;
            }
        }
    }

    for x in 0..w + 1 {
        for y in 0..h + 1 {
            if avg_alt[x][y].is_nan() {
                // interpolate altitude of pixel
                // TODO: optimize to first clasify area then assign values
                let mut i1 = x;
                let mut i2 = x;
                let mut j1 = y;
                let mut j2 = y;

                while i1 > 0 && avg_alt[i1][y].is_nan() {
                    i1 -= 1;
                }

                while i2 < w && avg_alt[i2][y].is_nan() {
                    i2 += 1;
                }

                while j1 > 0 && avg_alt[x][j1].is_nan() {
                    j1 -= 1;
                }

                while j2 < h && avg_alt[x][j2].is_nan() {
                    j2 += 1;
                }

                let mut val1 = f64::NAN;
                let mut val2 = f64::NAN;

                if !avg_alt[i1][y].is_nan() && !avg_alt[i2][y].is_nan() {
                    val1 = ((i2 - x) as f64 * avg_alt[i1][y] + (x - i1) as f64 * avg_alt[i2][y])
                        / ((i2 - i1) as f64);
                }

                if !avg_alt[x][j1].is_nan() && !avg_alt[x][j2].is_nan() {
                    val2 = ((j2 - y) as f64 * avg_alt[x][j1] + (y - j1) as f64 * avg_alt[x][j2])
                        / ((j2 - j1) as f64);
                }

                if !val1.is_nan() && !val2.is_nan() {
                    avg_alt[x][y] = (val1 + val2) / 2.0;
                } else if !val1.is_nan() {
                    avg_alt[x][y] = val1;
                } else if !val2.is_nan() {
                    avg_alt[x][y] = val2;
                }
            }
        }
    }

    for x in 0..w + 1 {
        for y in 0..h + 1 {
            if avg_alt[x][y].is_nan() {
                // second round of interpolation of altitude of pixel
                let mut val: f64 = 0.0;
                let mut c = 0;
                for i in 0..3 {
                    let ii: i32 = i - 1;
                    for j in 0..3 {
                        let jj: i32 = j - 1;
                        if y as i32 + jj >= 0 && x as i32 + ii >= 0 {
                            let x_idx = (x as i32 + ii) as usize;
                            let y_idx = (y as i32 + jj) as usize;
                            if x_idx <= w && y_idx <= h && !avg_alt[x_idx][y_idx].is_nan() {
                                c += 1;
                                val += avg_alt[x_idx][y_idx];
                            }
                        }
                    }
                }
                if c > 0 {
                    avg_alt[x][y] = val / c as f64;
                }
            }
        }
    }

    for x in 0..w + 1 {
        for y in 1..h + 1 {
            if avg_alt[x][y].is_nan() {
                avg_alt[x][y] = avg_alt[x][y - 1];
            }
        }
        for yy in 1..h + 1 {
            let y = h - yy;
            if avg_alt[x][y].is_nan() {
                avg_alt[x][y] = avg_alt[x][y + 1];
            }
        }
    }

    xmin += 1.0;
    ymin += 1.0;

    for x in 0..w + 1 {
        for y in 0..h + 1 {
            let mut ele = avg_alt[x][y];
            let temp: f64 = (ele / cinterval + 0.5).floor() * cinterval;
            if (ele - temp).abs() < 0.02 {
                if ele - temp < 0.0 || (jarkkos_bug && -temp < 0.0) {
                    ele = temp - 0.02;
                } else {
                    ele = temp + 0.02;
                }
                avg_alt[x][y] = ele;
            }
        }
    }

    if !xyzfileout.is_empty() && xyzfileout != "null" {
        let path = format!("{}/{}", tmpfolder, xyzfileout);
        let xyz_file_out = Path::new(&path);
        let f = File::create(xyz_file_out).expect("Unable to create file");
        let mut f = BufWriter::new(f);
        for x in 0..w + 1 {
            for y in 0..h + 1 {
                let ele = avg_alt[x][y];
                let xx = x as f64 * 2.0 * scalefactor + xmin;
                let yy = y as f64 * 2.0 * scalefactor + ymin;
                write!(&mut f, "{} {} {}\r\n", xx, yy, ele).expect("Cannot write to output file");
            }
        }
    }
    if !dxffile.is_empty() && dxffile != "null" {
        let v = cinterval;

        let mut level: f64 = (hmin / v).floor() * v;
        let path = format!("{}/temp_polylines.txt", tmpfolder);
        let polyline_out = Path::new(&path);

        let f = File::create(polyline_out).expect("Unable to create file");
        let mut f = BufWriter::new(f);

        loop {
            if level >= hmax {
                break;
            }

            let mut obj = Vec::<(i64, i64, u8)>::new();
            let mut curves: HashMap<(i64, i64, u8), (i64, i64)> = HashMap::default();

            for i in 1..(w - 1) {
                for j in 2..(h - 1) {
                    let mut a = avg_alt[i][j];
                    let mut b = avg_alt[i][j + 1];
                    let mut c = avg_alt[i + 1][j];
                    let mut d = avg_alt[i + 1][j + 1];

                    if a < level && b < level && c < level && d < level
                        || a > level && b > level && c > level && d > level
                    {
                        // skip
                    } else {
                        let temp: f64 = (a / v + 0.5).floor() * v;
                        if (a - temp).abs() < 0.05 {
                            if a - temp < 0.0 {
                                a = temp - 0.05;
                            } else {
                                a = temp + 0.05;
                            }
                        }

                        let temp: f64 = (b / v + 0.5).floor() * v;
                        if (b - temp).abs() < 0.05 {
                            if b - temp < 0.0 {
                                b = temp - 0.05;
                            } else {
                                b = temp + 0.05;
                            }
                        }

                        let temp: f64 = (c / v + 0.5).floor() * v;
                        if (c - temp).abs() < 0.05 {
                            if c - temp < 0.0 {
                                c = temp - 0.05;
                            } else {
                                c = temp + 0.05;
                            }
                        }

                        let temp: f64 = (d / v + 0.5).floor() * v;
                        if (d - temp).abs() < 0.05 {
                            if d - temp < 0.0 {
                                d = temp - 0.05;
                            } else {
                                d = temp + 0.05;
                            }
                        }

                        if a < b {
                            if level < b && level > a {
                                let x1: f64 = i as f64;
                                let y1: f64 = j as f64 + (level - a) / (b - a);
                                if level > c {
                                    let x2: f64 = i as f64 + (b - level) / (b - c);
                                    let y2: f64 = j as f64 + (level - c) / (b - c);
                                    check_obj_in(&mut obj, &mut curves, x1, x2, y1, y2);
                                } else if level < c {
                                    let x2: f64 = i as f64 + (level - a) / (c - a);
                                    let y2: f64 = j as f64;
                                    check_obj_in(&mut obj, &mut curves, x1, x2, y1, y2);
                                }
                            }
                        } else if b < a && level < a && level > b {
                            let x1: f64 = i as f64;
                            let y1: f64 = j as f64 + (a - level) / (a - b);
                            if level < c {
                                let x2: f64 = i as f64 + (level - b) / (c - b);
                                let y2: f64 = j as f64 + (c - level) / (c - b);
                                check_obj_in(&mut obj, &mut curves, x1, x2, y1, y2);
                            } else if level > c {
                                let x2: f64 = i as f64 + (a - level) / (a - c);
                                let y2: f64 = j as f64;
                                check_obj_in(&mut obj, &mut curves, x1, x2, y1, y2);
                            }
                        }

                        if a < c {
                            if level < c && level > a {
                                let x1: f64 = i as f64 + (level - a) / (c - a);
                                let y1: f64 = j as f64;
                                if level > b {
                                    let x2: f64 = i as f64 + (level - b) / (c - b);
                                    let y2: f64 = j as f64 + (c - level) / (c - b);
                                    check_obj_in(&mut obj, &mut curves, x1, x2, y1, y2);
                                }
                            }
                        } else if a > c && level < a && level > c {
                            let x1: f64 = i as f64 + (a - level) / (a - c);
                            let y1: f64 = j as f64;
                            if level < b {
                                let x2: f64 = i as f64 + (b - level) / (b - c);
                                let y2: f64 = j as f64 + (level - c) / (b - c);
                                check_obj_in(&mut obj, &mut curves, x1, x2, y1, y2);
                            }
                        }

                        if c < d {
                            if level < d && level > c {
                                let x1: f64 = i as f64 + 1.0;
                                let y1: f64 = j as f64 + (level - c) / (d - c);
                                if level < b {
                                    let x2: f64 = i as f64 + (b - level) / (b - c);
                                    let y2: f64 = j as f64 + (level - c) / (b - c);
                                    check_obj_in(&mut obj, &mut curves, x1, x2, y1, y2);
                                } else if level > b {
                                    let x2: f64 = i as f64 + (level - b) / (d - b);
                                    let y2: f64 = j as f64 + 1.0;
                                    check_obj_in(&mut obj, &mut curves, x1, x2, y1, y2);
                                }
                            }
                        } else if c > d && level < c && level > d {
                            let x1: f64 = i as f64 + 1.0;
                            let y1: f64 = j as f64 + (c - level) / (c - d);
                            if level > b {
                                let x2: f64 = i as f64 + (level - b) / (c - b);
                                let y2: f64 = j as f64 + (c - level) / (c - b);
                                check_obj_in(&mut obj, &mut curves, x1, x2, y1, y2);
                            } else if level < b {
                                let x2: f64 = i as f64 + (b - level) / (b - d);
                                let y2: f64 = j as f64 + 1.0;
                                check_obj_in(&mut obj, &mut curves, x1, x2, y1, y2);
                            }
                        }

                        if d < b {
                            if level < b && level > d {
                                let x1: f64 = i as f64 + (b - level) / (b - d);
                                let y1: f64 = j as f64 + 1.0;
                                if level > c {
                                    let x2: f64 = i as f64 + (b - level) / (b - c);
                                    let y2: f64 = j as f64 + (level - c) / (b - c);
                                    check_obj_in(&mut obj, &mut curves, x1, x2, y1, y2);
                                }
                            }
                        } else if b < d && level < d && level > b {
                            let x1: f64 = i as f64 + (level - b) / (d - b);
                            let y1: f64 = j as f64 + 1.0;
                            if level < c {
                                let x2: f64 = i as f64 + (level - b) / (c - b);
                                let y2: f64 = j as f64 + (c - level) / (c - b);
                                check_obj_in(&mut obj, &mut curves, x1, x2, y1, y2);
                            }
                        }
                    }
                }
            }

            for k in obj.iter() {
                if curves.contains_key(k) {
                    let (x, y, _) = *k;
                    write!(&mut f, "{},{};", x as f64 / 100.0, y as f64 / 100.0)
                        .expect("Cannot write to output file");
                    let mut res = (x, y);

                    let (x, y) = *curves.get(&k).unwrap();
                    write!(&mut f, "{},{};", x as f64 / 100.0, y as f64 / 100.0)
                        .expect("Cannot write to output file");
                    curves.remove(&k);

                    let mut head = (x, y);

                    if curves.get(&(head.0, head.1, 1)).is_some_and(|v| *v == res) {
                        curves.remove(&(head.0, head.1, 1));
                    }
                    if curves.get(&(head.0, head.1, 2)).is_some_and(|v| *v == res) {
                        curves.remove(&(head.0, head.1, 2));
                    }
                    loop {
                        if curves.get(&(head.0, head.1, 1)).is_some_and(|v| *v != res) {
                            res = head;

                            let (x, y) = *curves.get(&(head.0, head.1, 1)).unwrap();
                            write!(&mut f, "{},{};", x as f64 / 100.0, y as f64 / 100.0)
                                .expect("Cannot write to output file");
                            curves.remove(&(head.0, head.1, 1));

                            head = (x, y);
                            if curves.get(&(head.0, head.1, 1)).is_some_and(|v| *v == res) {
                                curves.remove(&(head.0, head.1, 1));
                            }
                            if curves.get(&(head.0, head.1, 2)).is_some_and(|v| *v == res) {
                                curves.remove(&(head.0, head.1, 2));
                            }
                        } else if curves.get(&(head.0, head.1, 2)).is_some_and(|v| *v != res) {
                            res = head;

                            let (x, y) = *curves.get(&(head.0, head.1, 2)).unwrap();
                            write!(&mut f, "{},{};", x as f64 / 100.0, y as f64 / 100.0)
                                .expect("Cannot write to output file");
                            curves.remove(&(head.0, head.1, 2));

                            head = (x, y);
                            if curves.get(&(head.0, head.1, 1)).is_some_and(|v| *v == res) {
                                curves.remove(&(head.0, head.1, 1));
                            }
                            if curves.get(&(head.0, head.1, 2)).is_some_and(|v| *v == res) {
                                curves.remove(&(head.0, head.1, 2));
                            }
                        } else {
                            f.write_all("\r\n".as_bytes())
                                .expect("Cannot write to output file");
                            break;
                        }
                    }
                }
            }
            level += v;
        }
        // explicitly flush and drop to close the file
        drop(f);

        let f = File::create(Path::new(&format!("{}/{}", tmpfolder, dxffile)))
            .expect("Unable to create file");
        let mut f = BufWriter::new(f);

        write!(
            &mut f,
            "  0\r\nSECTION\r\n  2\r\nHEADER\r\n  9\r\n$EXTMIN\r\n 10\r\n{}\r\n 20\r\n{}\r\n  9\r\n$EXTMAX\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nENDSEC\r\n  0\r\nSECTION\r\n  2\r\nENTITIES\r\n  0\r\n",
            xmin, ymin, xmax, ymax,
        ).expect("Cannot write dxf file");

        read_lines_no_alloc(polyline_out, |line| {
            let parts = line.trim().split(';');
            let r = parts.collect::<Vec<&str>>();
            f.write_all("POLYLINE\r\n 66\r\n1\r\n  8\r\ncont\r\n  0\r\n".as_bytes())
                .expect("Cannot write dxf file");
            for (i, d) in r.iter().enumerate() {
                if d != &"" {
                    let ii = i + 1;
                    let ldata = r.len() - 2;
                    if ii > 5 && ii < ldata - 5 && ldata > 12 && ii % 2 == 0 {
                        continue;
                    }
                    let mut xy_raw = d.split(',');
                    let x: f64 =
                        xy_raw.next().unwrap().parse::<f64>().unwrap() * 2.0 * scalefactor + xmin;
                    let y: f64 =
                        xy_raw.next().unwrap().parse::<f64>().unwrap() * 2.0 * scalefactor + ymin;
                    write!(
                        &mut f,
                        "VERTEX\r\n  8\r\ncont\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\n",
                        x, y
                    )
                    .expect("Cannot write dxf file");
                }
            }
            f.write_all("SEQEND\r\n  0\r\n".as_bytes())
                .expect("Cannot write dxf file");
        })
        .expect("Cannot read file");
        f.write_all("ENDSEC\r\n  0\r\nEOF\r\n".as_bytes())
            .expect("Cannot write dxf file");
        println!("Done");
    }
    Ok(())
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Iterates over the lines in a file and calls the callback with a &str reference to each line.
/// This function does not allocate new strings for each line, as opposed to using
/// [`io::BufReader::lines()`].
fn read_lines_no_alloc<P>(filename: P, mut line_callback: impl FnMut(&str)) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let mut reader = io::BufReader::new(file);

    let mut line_buffer = String::new();
    while reader.read_line(&mut line_buffer)? > 0 {
        // the read line contains the newline delimiter, so we need to trim it off
        let line = line_buffer.trim_end();
        line_callback(&line);
        line_buffer.clear();
    }

    Ok(())
}

fn check_obj_in(
    obj: &mut Vec<(i64, i64, u8)>,
    curves: &mut HashMap<(i64, i64, u8), (i64, i64)>,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
) {
    // convert the coordinates to integers with 2 decimal places for use as keys
    let x1 = (x1 * 100.0).floor() as i64;
    let x2 = (x2 * 100.0).floor() as i64;
    let y1 = (y1 * 100.0).floor() as i64;
    let y2 = (y2 * 100.0).floor() as i64;

    if x1 != x2 || y1 != y2 {
        let key = (x1, y1, 1);
        if !curves.contains_key(&key) {
            curves.insert(key, (x2, y2));
            obj.push(key);
        } else {
            let key = (x1, y1, 2);
            curves.insert(key, (x2, y2));
            obj.push(key);
        }
        let key = (x2, y2, 1);
        if !curves.contains_key(&key) {
            curves.insert(key, (x1, y1));
            obj.push(key);
        } else {
            let key = (x2, y2, 2);
            curves.insert(key, (x1, y1));
            obj.push(key);
        }
    }
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

fn knolldetector(thread: &String) -> Result<(), Box<dyn Error>> {
    println!("Detecting knolls...");
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let scalefactor: f64 = conf
        .general_section()
        .get("scalefactor")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);
    let contour_interval: f64 = conf
        .general_section()
        .get("contour_interval")
        .unwrap_or("5")
        .parse::<f64>()
        .unwrap_or(5.0);
    let halfinterval = contour_interval / 2.0 * scalefactor;

    let interval = 0.3 * scalefactor;
    let tmpfolder = format!("temp{}", thread);
    let path = format!("{}/xyz_03.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);

    let mut size: f64 = f64::NAN;
    let mut xstart: f64 = f64::NAN;
    let mut ystart: f64 = f64::NAN;

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

    let mut xmax: u64 = u64::MIN;
    let mut ymax: u64 = u64::MIN;
    let mut xmin: u64 = u64::MAX;
    let mut ymin: u64 = u64::MAX;
    let mut xyz: HashMap<(u64, u64), f64> = HashMap::default();
    read_lines_no_alloc(xyz_file_in, |line| {
        let mut parts = line.split(' ');
        let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
        let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
        let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();

        let xx = ((x - xstart) / size).floor() as u64;
        let yy = ((y - ystart) / size).floor() as u64;

        xyz.insert((xx, yy), h);

        if xmax < xx {
            xmax = xx;
        }
        if ymax < yy {
            ymax = yy;
        }
        if xmin > xx {
            xmin = xx;
        }
        if ymin > yy {
            ymin = yy;
        }
    })
    .expect("Could not read file");

    let data = fs::read_to_string(Path::new(&format!("{}/contours03.dxf", tmpfolder)))
        .expect("Should have been able to read the file");
    let data: Vec<&str> = data.split("POLYLINE").collect();
    let f = File::create(Path::new(&format!("{}/detected.dxf", tmpfolder)))
        .expect("Unable to create file");
    let mut f = BufWriter::new(f);
    write!(&mut f,
        "  0\r\nSECTION\r\n  2\r\nHEADER\r\n  9\r\n$EXTMIN\r\n 10\r\n{}\r\n 20\r\n{}\r\n  9\r\n$EXTMAX\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nENDSEC\r\n  0\r\nSECTION\r\n  2\r\nENTITIES\r\n  0\r\n",
        xmin, ymin, xmax, ymax
    ).expect("Cannot write dxf file");

    let mut heads1: HashMap<String, usize> = HashMap::default();
    let mut heads2: HashMap<String, usize> = HashMap::default();
    let mut heads = Vec::<String>::with_capacity(data.len());
    let mut tails = Vec::<String>::with_capacity(data.len());
    let mut el_x = Vec::<Vec<f64>>::with_capacity(data.len());
    let mut el_y = Vec::<Vec<f64>>::with_capacity(data.len());
    el_x.push(vec![]);
    el_y.push(vec![]);
    heads.push(String::from("-"));
    tails.push(String::from("-"));
    for (j, rec) in data.iter().enumerate() {
        let mut x = Vec::<f64>::new();
        let mut y = Vec::<f64>::new();
        let mut xline = 0;
        let mut yline = 0;
        if j > 0 {
            let r = rec.split("VERTEX").collect::<Vec<&str>>();
            let apu = r[1];
            let val = apu.split('\n').collect::<Vec<&str>>();
            for (i, v) in val.iter().enumerate() {
                let vt = v.trim();
                if vt == "10" {
                    xline = i + 1;
                }
                if vt == "20" {
                    yline = i + 1;
                }
            }
            if r.len() < 201 {
                for (i, v) in r.iter().enumerate() {
                    if i > 0 {
                        let val = v.trim_end().split('\n').collect::<Vec<&str>>();
                        x.push(val[xline].trim().parse::<f64>().unwrap());
                        y.push(val[yline].trim().parse::<f64>().unwrap());
                    }
                }
                let x0 = x.first().unwrap();
                let xl = x.last().unwrap();

                let y0 = y.first().unwrap();
                let yl = y.last().unwrap();

                let head = format!("{}x{}", x0, y0);
                let tail = format!("{}x{}", xl, yl);

                heads.push(head);
                tails.push(tail);

                let head = format!("{}x{}", x0, y0);
                let tail = format!("{}x{}", xl, yl);

                el_x.push(x);
                el_y.push(y);
                if *heads1.get(&head).unwrap_or(&0) == 0 {
                    heads1.insert(head, j);
                } else {
                    heads2.insert(head, j);
                }
                if *heads1.get(&tail).unwrap_or(&0) == 0 {
                    heads1.insert(tail, j);
                } else {
                    heads2.insert(tail, j);
                }
            } else {
                heads.push(String::from("-"));
                tails.push(String::from("-"));
                el_x.push(vec![]);
                el_y.push(vec![]);
            }
        }
    }

    for l in 0..data.len() {
        let mut to_join = 0;
        if !el_x[l].is_empty() {
            let mut end_loop = false;
            while !end_loop {
                let tmp = *heads1.get(&heads[l]).unwrap_or(&0);
                if tmp != 0 && tmp != l && !el_x[tmp].is_empty() {
                    to_join = tmp;
                } else {
                    let tmp = *heads2.get(&heads[l]).unwrap_or(&0);
                    if tmp != 0 && tmp != l && !el_x[tmp].is_empty() {
                        to_join = tmp;
                    } else {
                        let tmp = *heads2.get(&tails[l]).unwrap_or(&0);
                        if tmp != 0 && tmp != l && !el_x[tmp].is_empty() {
                            to_join = tmp;
                        } else {
                            let tmp = *heads1.get(&tails[l]).unwrap_or(&0);
                            if tmp != 0 && tmp != l && !el_x[tmp].is_empty() {
                                to_join = tmp;
                            } else {
                                end_loop = true;
                            }
                        }
                    }
                }
                if !end_loop {
                    if tails[l] == heads[to_join] {
                        let tmp = tails[l].to_string();
                        heads2.insert(tmp, 0);
                        let tmp = tails[l].to_string();
                        heads1.insert(tmp, 0);
                        let mut to_append = el_x[to_join].to_vec();
                        el_x[l].append(&mut to_append);
                        let mut to_append = el_y[to_join].to_vec();
                        el_y[l].append(&mut to_append);
                        let tmp = tails[to_join].to_string();
                        tails[l] = tmp;
                        el_x[to_join].clear();
                        el_y[to_join].clear();
                    } else if tails[l] == tails[to_join] {
                        let tmp = tails[l].to_string();
                        heads2.insert(tmp, 0);
                        let tmp = tails[l].to_string();
                        heads1.insert(tmp, 0);
                        let mut to_append = el_x[to_join].to_vec();
                        to_append.reverse();
                        el_x[l].append(&mut to_append);
                        let mut to_append = el_y[to_join].to_vec();
                        to_append.reverse();
                        el_y[l].append(&mut to_append);
                        let tmp = heads[to_join].to_string();
                        tails[l] = tmp;
                        el_x[to_join].clear();
                        el_y[to_join].clear();
                    } else if heads[l] == tails[to_join] {
                        let tmp = heads[l].to_string();
                        heads2.insert(tmp, 0);
                        let tmp = heads[l].to_string();
                        heads1.insert(tmp, 0);
                        let to_append = el_x[to_join].to_vec();
                        el_x[l].splice(0..0, to_append);
                        let to_append = el_y[to_join].to_vec();
                        el_y[l].splice(0..0, to_append);
                        let tmp = heads[to_join].to_string();
                        heads[l] = tmp;
                        el_x[to_join].clear();
                        el_y[to_join].clear();
                    } else if heads[l] == heads[to_join] {
                        let tmp = heads[l].to_string();
                        heads2.insert(tmp, 0);
                        let tmp = heads[l].to_string();
                        heads1.insert(tmp, 0);
                        let mut to_append = el_x[to_join].to_vec();
                        to_append.reverse();
                        el_x[l].splice(0..0, to_append);
                        let mut to_append = el_y[to_join].to_vec();
                        to_append.reverse();
                        el_y[l].splice(0..0, to_append);
                        let tmp = tails[to_join].to_string();
                        heads[l] = tmp;
                        el_x[to_join].clear();
                        el_y[to_join].clear();
                    }
                }
            }
        }
    }

    let mut elevation: HashMap<u64, f64> = HashMap::default();
    for l in 0..data.len() {
        let mut skip = false;
        let el_x_len = el_x[l].len();
        if el_x_len > 0 {
            if el_x_len > 121 {
                skip = true;
                el_x[l].clear();
                el_y[l].clear();
            }
            if el_x_len < 9 {
                let mut p = 0;
                let mut dist = 0.0;
                while p < el_x_len - 1 {
                    dist += ((el_x[l][p] - el_x[l][p + 1]).powi(2)
                        + (el_y[l][p] - el_y[l][p + 1]).powi(2))
                    .sqrt();
                    p += 1;
                }
                if dist < 5.0 || el_x_len < 3 {
                    skip = true;
                    el_x[l].clear();
                    el_y[l].clear();
                }
            }
            if el_x[l].first() != el_x[l].last() || el_y[l].first() != el_y[l].last() {
                skip = true;
                el_x[l].clear();
                el_y[l].clear();
            }
            if !skip
                && el_x_len < 122
                && el_x[l].first() == el_x[l].last()
                && el_y[l].first() == el_y[l].last()
            {
                let tailx = *el_x[l].first().unwrap();
                let mut xl = el_x[l].to_vec();
                xl.push(tailx);
                let taily = *el_y[l].first().unwrap();
                let mut yl = el_y[l].to_vec();
                yl.push(taily);
                let mut mm = ((el_x_len as f64 / 3.0).floor() - 1.0) as i32;
                if mm < 0 {
                    mm = 0;
                }
                let mut m = mm as usize;
                let mut h = 0.0;
                while m < xl.len() {
                    let xm = xl[m];
                    let ym = yl[m];
                    let xo = (xm - xstart) / size;
                    let yo = (ym - ystart) / size;
                    if xo == xo.floor() {
                        let h1 = *xyz
                            .get(&(xo.floor() as u64, yo.floor() as u64))
                            .unwrap_or(&0.0);
                        let h2 = *xyz
                            .get(&(xo.floor() as u64, yo.floor() as u64 + 1))
                            .unwrap_or(&0.0);
                        h = h1 * (yo.floor() + 1.0 - yo) + h2 * (yo - yo.floor());
                        h = (h / interval + 0.5).floor() * interval;
                        break;
                    } else if m < (el_x_len - 3) && yo == yo.floor() {
                        let h1 = *xyz
                            .get(&(xo.floor() as u64, yo.floor() as u64))
                            .unwrap_or(&0.0);
                        let h2 = *xyz
                            .get(&(xo.floor() as u64 + 1, yo.floor() as u64))
                            .unwrap_or(&0.0);
                        h = h1 * (xo.floor() + 1.0 - xo) + h2 * (xo - xo.floor());
                        h = (h / interval + 0.5).floor() * interval;
                    }
                    m += 1;
                }
                elevation.insert(l as u64, h);

                let mut mm = ((el_x_len as f64 / 3.0).floor() - 1.0) as i32;
                if mm < 0 {
                    mm = 0;
                }
                let mut m = mm as usize;
                let mut xa = xl[m];
                let mut ya = yl[m];
                while m < xl.len() {
                    let xm = xl[m];
                    let ym = yl[m];
                    let xo = (xm - xstart) / size;
                    let yo = (ym - ystart) / size;
                    if m < xl.len() - 3 && yo == yo.floor() && xo != xo.floor() {
                        xa = xo.floor() * size + xstart;
                        ya = ym.floor();
                        break;
                    }
                    m += 1;
                }
                let h_center = *xyz
                    .get(&(
                        ((xa - xstart) / size).floor() as u64,
                        ((ya - ystart) / size).floor() as u64,
                    ))
                    .unwrap_or(&0.0);
                let mut hit = 0;
                let xtest = ((xa - xstart) / size).floor() * size + xstart + 0.000000001;
                let ytest = ((ya - ystart) / size).floor() * size + ystart + 0.000000001;

                let mut n = 0;
                let mut y0 = 0.0;
                let mut x0 = 0.0;
                while n < (el_x_len - 1) {
                    let x1 = el_x[l][n];
                    let y1 = el_y[l][n];
                    if n > 0
                        && ((y0 <= ytest && ytest < y1) || (y1 <= ytest && ytest < y0))
                        && (xtest < ((x1 - x0) * (ytest - y0) / (y1 - y0) + x0))
                    {
                        hit += 1;
                    }
                    n += 1;
                    x0 = x1;
                    y0 = y1;
                }

                if (h_center < h) && (hit % 2 == 1) || (h_center > h) && (hit % 2 != 1) {
                    skip = true;
                    el_x[l].clear();
                    el_y[l].clear();
                }
            }
        }
        if skip {
            el_x[l].clear();
            el_y[l].clear();
        }
    }

    struct Head {
        id: u64,
        xtest: f64,
        ytest: f64,
    }
    let mut heads = Vec::<Head>::new();
    for l in 0..data.len() {
        if !el_x[l].is_empty() {
            if el_x[l].first() == el_x[l].last() && el_y[l].first() == el_y[l].last() {
                heads.push(Head {
                    id: l as u64,
                    xtest: el_x[l][0],
                    ytest: el_y[l][0],
                });
            } else {
                el_x[l].clear();
                el_y[l].clear();
            }
        }
    }
    struct Top {
        id: u64,
        xtest: f64,
        ytest: f64,
    }
    let mut tops = Vec::<Top>::new();
    struct BoundingBox {
        minx: f64,
        maxx: f64,
        miny: f64,
        maxy: f64,
    }
    let mut bb: HashMap<usize, BoundingBox> = HashMap::default();
    for l in 0..data.len() {
        let mut skip = false;
        if !el_x[l].is_empty() {
            let mut x = el_x[l].to_vec();
            let tailx = *el_x[l].first().unwrap();
            x.push(tailx);

            let mut y = el_y[l].to_vec();
            let taily = *el_y[l].first().unwrap();
            y.push(taily);

            let mut minx = f64::MAX;
            let mut miny = f64::MAX;
            let mut maxx = f64::MIN;
            let mut maxy = f64::MIN;

            for k in 0..x.len() {
                if x[k] > maxx {
                    maxx = x[k]
                }
                if x[k] < minx {
                    minx = x[k]
                }
                if y[k] > maxy {
                    maxy = y[k]
                }
                if y[k] < miny {
                    miny = y[k]
                }
            }
            bb.insert(
                l,
                BoundingBox {
                    minx,
                    maxx,
                    miny,
                    maxy,
                },
            );

            for head in heads.iter() {
                let &Head { id, xtest, ytest } = head;

                if !skip
                    && *elevation.get(&id).unwrap() > *elevation.get(&(l as u64)).unwrap()
                    && id != (l as u64)
                    && xtest < maxx
                    && xtest > minx
                    && ytest < maxy
                    && ytest > miny
                {
                    let mut hit = 0;
                    let mut n = 0;
                    let mut x0 = 0.0;
                    let mut y0 = 0.0;
                    while n < x.len() {
                        let x1 = x[n];
                        let y1 = y[n];

                        if n > 0
                            && ((y0 <= ytest && ytest < y1) || (y1 <= ytest && ytest < y0))
                            && (xtest < ((x1 - x0) * (ytest - y0) / (y1 - y0) + x0))
                        {
                            hit += 1;
                        }
                        x0 = x1;
                        y0 = y1;
                        n += 1;
                    }
                    if hit % 2 == 1 {
                        skip = true;
                    }
                }
            }
            if !skip {
                tops.push(Top {
                    id: l as u64,
                    xtest: x[0],
                    ytest: y[0],
                });
            }
        }
    }
    struct Candidate {
        id: u64,
        xtest: f64,
        ytest: f64,
        topid: u64,
    }
    let mut canditates = Vec::<Candidate>::new();

    for l in 0..data.len() {
        let mut skip = true;
        if !el_x[l].is_empty() {
            let mut x = el_x[l].to_vec();
            let tailx = *el_x[l].first().unwrap();
            x.push(tailx);

            let mut y = el_y[l].to_vec();
            let taily = *el_y[l].first().unwrap();
            y.push(taily);

            let &BoundingBox {
                minx,
                maxx,
                miny,
                maxy,
            } = bb.get(&l).unwrap();

            let mut topid = 0;
            for head in tops.iter() {
                let &Top { id, xtest, ytest } = head;
                let ll = l as u64;

                if *elevation.get(&ll).unwrap() < (*elevation.get(&id).unwrap() - 0.1)
                    && *elevation.get(&ll).unwrap() > (*elevation.get(&id).unwrap() - 4.6)
                    && skip
                    && xtest < maxx
                    && xtest > minx
                    && ytest < maxy
                    && ytest > miny
                {
                    let mut hit = 0;
                    let mut n = 0;

                    let mut x0 = 0.0;
                    let mut y0 = 0.0;
                    while n < x.len() {
                        let x1 = x[n];
                        let y1 = y[n];

                        if n > 0
                            && ((y0 <= ytest && ytest < y1) || (y1 <= ytest && ytest < y0))
                            && (xtest < ((x1 - x0) * (ytest - y0) / (y1 - y0) + x0))
                        {
                            hit += 1;
                        }
                        x0 = x1;
                        y0 = y1;

                        n += 1;
                    }
                    if hit % 2 == 1 {
                        skip = false;
                        topid = id;
                    }
                }
            }
            if !skip {
                canditates.push(Candidate {
                    id: l as u64,
                    xtest: x[0],
                    ytest: y[0],
                    topid,
                });
            } else {
                el_x[l].clear();
                el_y[l].clear();
            }
        }
    }

    let mut best: HashMap<u64, u64> = HashMap::default();
    let mut mov: HashMap<u64, f64> = HashMap::default();

    for head in canditates.iter() {
        let &Candidate { id, topid, .. } = head;
        let el = *elevation.get(&id).unwrap();
        let test = (el / halfinterval + 1.0).floor() * halfinterval - el;

        if !best.contains_key(&topid) {
            best.insert(topid, id);
            mov.insert(id, test);
        } else {
            let tid = *best.get(&topid).unwrap();
            if *mov.get(&tid).unwrap() < 1.75
                && (*elevation.get(&topid).unwrap() - *elevation.get(&tid).unwrap() - 0.6).abs()
                    < 0.2
            {
                // no action
            } else if *mov.get(&tid).unwrap() > test {
                best.insert(topid, id);
                mov.insert(id, test);
            }
        }
    }
    let mut new_candidates = Vec::<Candidate>::new();
    for head in canditates.iter() {
        let &Candidate {
            id,
            xtest,
            ytest,
            topid,
        } = head;

        let x = el_x[id as usize].to_vec();
        if *best.get(&topid).unwrap() == id
            && (x.len() < 13
                || (*elevation.get(&topid).unwrap() > (*elevation.get(&id).unwrap() + 0.45)
                    || (*elevation.get(&id).unwrap()
                        - 2.5 * (*elevation.get(&id).unwrap() / 2.5).floor())
                        > 0.45))
        {
            new_candidates.push(Candidate {
                id,
                xtest,
                ytest,
                topid,
            });
        } else {
            el_x[id as usize].clear();
            el_y[id as usize].clear();
        }
    }

    let canditates = new_candidates;

    let file_pins =
        File::create(Path::new(&format!("{}/pins.txt", tmpfolder))).expect("Unable to create file");
    let mut file_pins = BufWriter::new(file_pins);

    for l in 0..data.len() {
        let mut skip = false;
        let ll = l as u64;
        let mut ltopid = 0;
        if !el_x[l].is_empty() {
            let mut x = el_x[l].to_vec();
            let tailx = *el_x[l].first().unwrap();
            x.push(tailx);

            let mut y = el_y[l].to_vec();
            let taily = *el_y[l].first().unwrap();
            y.push(taily);

            let &BoundingBox {
                minx,
                maxx,
                miny,
                maxy,
            } = bb.get(&l).unwrap();

            for head in canditates.iter() {
                let &Candidate {
                    id,
                    xtest,
                    ytest,
                    topid,
                } = head;

                ltopid = topid;
                if id != ll && !skip && xtest < maxx && xtest > minx && ytest < maxy && ytest > miny
                {
                    let mut hit = 0;
                    let mut n = 0;

                    let mut x0 = 0.0;
                    let mut y0 = 0.0;
                    while n < x.len() {
                        let x1 = x[n];
                        let y1 = y[n];

                        if n > 0
                            && ((y0 <= ytest && ytest < y1) || (y1 <= ytest && ytest < y0))
                            && (xtest < ((x1 - x0) * (ytest - y0) / (y1 - y0) + x0))
                        {
                            hit += 1;
                        }
                        x0 = x1;
                        y0 = y1;

                        n += 1;
                    }
                    if hit % 2 == 1 {
                        skip = true;
                    }
                }
            }

            if !skip {
                f.write_all("POLYLINE\r\n 66\r\n1\r\n  8\r\n1010\r\n  0\r\n".as_bytes())
                    .expect("Can not write to file");
                let mut xa = 0.0;
                let mut ya = 0.0;
                for k in 0..x.len() {
                    xa += x[k];
                    ya += y[k];
                }
                let xlen = x.len() as f64;
                xa /= xlen;
                ya /= xlen;

                write!(
                    &mut file_pins,
                    "{},{},{},{},{},{},{},{}\r\n",
                    x[0],
                    y[0],
                    *elevation.get(&ll).unwrap(),
                    xa,
                    ya,
                    *elevation.get(&ltopid).unwrap(),
                    x.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(" "),
                    y.iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                )
                .expect("Could not write to file");

                for k in 0..x.len() {
                    write!(
                        &mut f,
                        "VERTEX\r\n  8\r\n1010\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\n",
                        x[k], y[k]
                    )
                    .expect("Can not write to file");
                }
                f.write_all("SEQEND\r\n  0\r\n".as_bytes())
                    .expect("Can not write to file");
            } else {
                el_x[l].clear();
                el_y[l].clear();
            }
        }
    }
    f.write_all("ENDSEC\r\n  0\r\nEOF\r\n".as_bytes())
        .expect("Can not write to file");

    println!("Done");
    Ok(())
}

fn xyzknolls(thread: &String) -> Result<(), Box<dyn Error>> {
    println!("Identifying knolls...");
    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let scalefactor: f64 = conf
        .general_section()
        .get("scalefactor")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);

    let interval = 2.5 * scalefactor;

    let tmpfolder = format!("temp{}", thread);

    let path = format!("{}/xyz_03.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);

    let mut xstart: f64 = 0.0;
    let mut ystart: f64 = 0.0;
    let mut size: f64 = 0.0;

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
    let mut xmax: u64 = 0;
    let mut ymax: u64 = 0;
    let mut xyz: HashMap<(u64, u64), f64> = HashMap::default();
    let mut xyz2: HashMap<(u64, u64), f64> = HashMap::default();
    read_lines_no_alloc(xyz_file_in, |line| {
        let mut parts = line.split(' ');
        let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
        let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
        let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();

        let xx = ((x - xstart) / size).floor() as u64;
        let yy = ((y - ystart) / size).floor() as u64;
        xyz.insert((xx, yy), h);
        xyz2.insert((xx, yy), h);
        if xmax < xx {
            xmax = xx;
        }
        if ymax < yy {
            ymax = yy;
        }
    })
    .expect("could not read file");

    for i in 2..(xmax as usize - 1) {
        for j in 2..(ymax as usize - 1) {
            let mut low = f64::MAX;
            let mut high = f64::MIN;
            let mut val = 0.0;
            let mut count = 0;
            for ii in (i - 2)..(i + 3) {
                for jj in (j - 2)..(j + 3) {
                    let tmp = *xyz.get(&(ii as u64, jj as u64)).unwrap_or(&0.0);
                    if tmp < low {
                        low = tmp;
                    }
                    if tmp > high {
                        high = tmp;
                    }
                    count += 1;
                    val += tmp;
                }
            }
            let steepness = high - low;
            if steepness < 1.25 {
                let tmp = (1.25 - steepness) * (val - low - high) / (count as f64 - 2.0) / 1.25
                    + steepness * (*xyz2.get(&(i as u64, j as u64)).unwrap_or(&0.0)) / 1.25;
                xyz2.insert((i as u64, j as u64), tmp);
            }
        }
    }

    let path = format!("{}/pins.txt", tmpfolder);
    let pins_file_in = Path::new(&path);

    struct Pin {
        xx: f64,
        yy: f64,
        ele: f64,
        ele2: f64,
        xlist: Vec<f64>,
        ylist: Vec<f64>,
    }
    let mut pins: Vec<Pin> = Vec::new();

    read_lines_no_alloc(pins_file_in, |line| {
        let mut r = line.trim().split(',');
        let ele = r.nth(2).unwrap().parse::<f64>().unwrap();
        let xx = r.next().unwrap().parse::<f64>().unwrap();
        let yy = r.next().unwrap().parse::<f64>().unwrap();
        let ele2 = r.next().unwrap().parse::<f64>().unwrap();
        let xlist = r.next().unwrap();
        let ylist = r.next().unwrap();
        let mut x: Vec<f64> = xlist
            .split(' ')
            .map(|s| s.parse::<f64>().unwrap())
            .collect();
        let mut y: Vec<f64> = ylist
            .split(' ')
            .map(|s| s.parse::<f64>().unwrap())
            .collect();
        x.push(x[0]);
        y.push(y[0]);

        pins.push(Pin {
            xx,
            yy,
            ele,
            ele2,
            xlist: x,
            ylist: y,
        });
    })
    .expect("could not read pins file");

    // compute closest distance from each pin to another pin
    let mut dist: HashMap<usize, f64> = HashMap::default();
    for (l, pin) in pins.iter().enumerate() {
        let mut min = f64::MAX;
        let xx = ((pin.xx - xstart) / size).floor();
        let yy = ((pin.yy - ystart) / size).floor();
        for (k, pin2) in pins.iter().enumerate() {
            if k != l {
                continue;
            }

            let xx2 = ((pin2.xx - xstart) / size).floor();
            let yy2 = ((pin2.yy - ystart) / size).floor();
            let mut dis = (xx2 - xx).abs();
            let disy = (yy2 - yy).abs();
            if disy > dis {
                dis = disy;
            }
            if dis < min {
                min = dis;
            }
        }
        dist.insert(l, min);
    }

    for (l, line) in pins.into_iter().enumerate() {
        let Pin {
            xx,
            yy,
            ele,
            ele2,
            xlist: mut x,
            ylist: mut y,
        } = line;

        let elenew = ((ele - 0.09) / interval + 1.0).floor() * interval;
        let mut move1 = elenew - ele + 0.15;
        let mut move2 = move1 * 0.4;
        if move1 > 0.66 * interval {
            move2 = move1 * 0.6;
        }
        if move1 < 0.25 * interval {
            move2 = 0.0;
            move1 += 0.3;
        }
        move1 += 0.5;
        if ele2 + move1 > ((ele - 0.09) / interval + 2.0).floor() * interval {
            move1 -= 0.4;
        }
        if elenew - ele > 1.5 * scalefactor && x.len() > 21 {
            for k in 0..x.len() {
                x[k] = xx + (x[k] - xx) * 0.8;
                y[k] = yy + (y[k] - yy) * 0.8;
            }
        }
        let mut touched: HashMap<String, bool> = HashMap::default();
        let mut minx = u64::MAX;
        let mut miny = u64::MAX;
        let mut maxx = u64::MIN;
        let mut maxy = u64::MIN;
        for k in 0..x.len() {
            x[k] = ((x[k] - xstart) / size + 0.5).floor();
            y[k] = ((y[k] - ystart) / size + 0.5).floor();
            let xk = x[k] as u64;
            let yk = y[k] as u64;
            if xk > maxx {
                maxx = xk;
            }
            if yk > maxy {
                maxy = yk;
            }
            if xk < minx {
                minx = xk;
            }
            if yk < miny {
                miny = yk;
            }
        }

        let xx = ((xx - xstart) / size).floor();
        let yy = ((yy - ystart) / size).floor();

        let mut x0 = 0.0;
        let mut y0 = 0.0;

        for ii in minx as usize..(maxx as usize + 1) {
            for jj in miny as usize..(maxy as usize + 1) {
                let mut hit = 0;
                let xtest = ii as f64;
                let ytest = jj as f64;
                for n in 0..x.len() {
                    let x1 = x[n];
                    let y1 = y[n];
                    if n > 1
                        && ((y0 <= ytest && ytest < y1) || (y1 <= ytest && ytest < y0))
                        && xtest < (x1 - x0) * (ytest - y0) / (y1 - y0) + x0
                    {
                        hit += 1;
                    }
                    x0 = x1;
                    y0 = y1;
                }
                if hit % 2 == 1 {
                    let tmp = *xyz2.get(&(ii as u64, jj as u64)).unwrap_or(&0.0) + move1;
                    xyz2.insert((ii as u64, jj as u64), tmp);
                    let coords = format!("{}_{}", ii, jj);
                    touched.insert(coords, true);
                }
            }
        }
        let mut range = *dist.get(&l).unwrap_or(&0.0) * 0.8 - 1.0;
        if range < 1.0 {
            range = 1.0;
        }
        if range > 12.0 {
            range = 12.0;
        }
        for iii in 0..((range * 2.0 + 1.0) as usize) {
            for jjj in 0..((range * 2.0 + 1.0) as usize) {
                let ii: f64 = xx - range + iii as f64;
                let jj: f64 = yy - range + jjj as f64;
                if ii > 0.0 && ii < xmax as f64 + 1.0 && jj > 0.0 && jj < ymax as f64 + 1.0 {
                    let coords = format!("{}_{}", ii, jj);
                    if !*touched.get(&coords).unwrap_or(&false) {
                        let tmp = *xyz2
                            .get(&(ii.floor() as u64, jj.floor() as u64))
                            .unwrap_or(&0.0)
                            + (range - (xx - ii).abs()) / range * (range - (yy - jj).abs()) / range
                                * move2;
                        xyz2.insert((ii.floor() as u64, jj.floor() as u64), tmp);
                    }
                }
            }
        }
    }

    let f2 = File::create(Path::new(&format!("{}/xyz_knolls.xyz", tmpfolder)))
        .expect("Unable to create file");
    let mut f2 = BufWriter::new(f2);

    read_lines_no_alloc(xyz_file_in, |line| {
        let parts = line.split(' ');
        let mut r = parts.collect::<Vec<&str>>();
        let x: f64 = r[0].parse::<f64>().unwrap();
        let y: f64 = r[1].parse::<f64>().unwrap();
        let mut h = *xyz2
            .get(&(
                ((x - xstart) / size).floor() as u64,
                ((y - ystart) / size).floor() as u64,
            ))
            .unwrap_or(&0.0);
        let tmp = (h / interval + 0.5).floor() * interval;
        if (tmp - h).abs() < 0.02 {
            if h - tmp < 0.0 {
                h = tmp - 0.02;
            } else {
                h = tmp + 0.02;
            }
        }
        let new_val = format!("{}", h);
        r[2] = &new_val;
        let out = r.join(" ");
        f2.write_all(out.as_bytes()).expect("cannot write to file");
        f2.write_all("\n".as_bytes()).expect("cannot write to file");
    })
    .expect("could not read file");

    println!("Done");
    Ok(())
}

fn makevegenew(thread: &String) -> Result<(), Box<dyn Error>> {
    println!("Generating vegetation...");

    let tmpfolder = format!("temp{}", thread);

    let path = format!("{}/xyz2.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);

    let mut xstart: f64 = 0.0;
    let mut ystart: f64 = 0.0;
    let mut size: f64 = 0.0;

    if let Ok(lines) = read_lines(xyz_file_in) {
        for (i, line) in lines.enumerate() {
            let ip = line.unwrap_or(String::new());
            let mut parts = ip.split(' ');
            let x = parts.next().unwrap().parse::<f64>().unwrap();
            let y = parts.next().unwrap().parse::<f64>().unwrap();

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

    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let block: f64 = conf
        .general_section()
        .get("greendetectsize")
        .unwrap_or("3")
        .parse::<f64>()
        .unwrap_or(3.0);

    let mut xyz: HashMap<(u64, u64), f64> = HashMap::default();
    let mut top: HashMap<(u64, u64), f64> = HashMap::default();

    read_lines_no_alloc(xyz_file_in, |line| {
        let mut parts = line.trim().split(' ');

        let x = parts.next().unwrap().parse::<f64>().unwrap();
        let y = parts.next().unwrap().parse::<f64>().unwrap();
        let h = parts.next().unwrap().parse::<f64>().unwrap();

        let xx = ((x - xstart) / size).floor() as u64;
        let yy = ((y - ystart) / size).floor() as u64;
        xyz.insert((xx, yy), h);
        let xxx = ((x - xstart) / block).floor() as u64;
        let yyy = ((y - ystart) / block).floor() as u64;
        if top.contains_key(&(xxx, yyy)) && h > *top.get(&(xxx, yyy)).unwrap() {
            top.insert((xxx, yyy), h);
        }
    })
    .expect("Can not read file");

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
        zones.push(last_zone);
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

    let vege_bitmode: bool = conf.general_section().get("vege_bitmode").unwrap_or("0") == "1";

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

    let path = format!("{}/xyztemp.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);

    let xmin = xstart;
    let ymin = ystart;
    let mut xmax: f64 = f64::MIN;
    let mut ymax: f64 = f64::MIN;

    let mut hits: HashMap<(u64, u64), u64> = HashMap::default();
    let mut yhit: HashMap<(u64, u64), u64> = HashMap::default();
    let mut noyhit: HashMap<(u64, u64), u64> = HashMap::default();

    let mut i = 0;
    read_lines_no_alloc(xyz_file_in, |line| {
        if vegethin == 0 || ((i + 1) as u32) % vegethin == 0 {
            let mut parts = line.trim().split(' ');
            let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let r3 = parts.next().unwrap();
            let r4 = parts.next().unwrap();
            let r5 = parts.next().unwrap();

            if xmax < x {
                xmax = x;
            }
            if ymax < y {
                ymax = y;
            }
            if x > xmin && y > ymin {
                let xx = ((x - xmin) / block).floor() as u64;
                let yy = ((y - ymin) / block).floor() as u64;
                if h > *top.get(&(xx, yy)).unwrap_or(&0.0) {
                    top.insert((xx, yy), h);
                }
                let xx = ((x - xmin) / 3.0).floor() as u64;
                let yy = ((y - ymin) / 3.0).floor() as u64;
                *hits.entry((xx, yy)).or_insert(0) += 1;

                if r3 == "2"
                    || h < yellowheight
                        + *xyz
                            .get(&(
                                ((x - xmin) / size).floor() as u64,
                                ((y - ymin) / size).floor() as u64,
                            ))
                            .unwrap_or(&0.0)
                {
                    *yhit.entry((xx, yy)).or_insert(0) += 1;
                } else if r4 == "1" && r5 == "1" {
                    *noyhit.entry((xx, yy)).or_insert(0) += yellowfirstlast;
                } else {
                    *noyhit.entry((xx, yy)).or_insert(0) += 1;
                }
            }
        }

        i += 1;
    })
    .expect("Can not read file");

    let mut firsthit: HashMap<(u64, u64), u64> = HashMap::default();
    let mut ugg: HashMap<(u64, u64), f64> = HashMap::default();
    let mut ug: HashMap<(u64, u64), u64> = HashMap::default();
    let mut ghit: HashMap<(u64, u64), u64> = HashMap::default();
    let mut greenhit: HashMap<(u64, u64), f64> = HashMap::default();
    let mut highit: HashMap<(u64, u64), u64> = HashMap::default();
    let step: f32 = 6.0;

    let mut i = 0;
    read_lines_no_alloc(xyz_file_in, |line| {
        if vegethin == 0 || ((i + 1) as u32) % vegethin == 0 {
            let mut parts = line.trim().split(' ');

            // parse the parts of the line
            let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let h: f64 = parts.next().unwrap().parse::<f64>().unwrap() - zoffset;
            let r3 = parts.next().unwrap();
            let r4 = parts.next().unwrap();
            let r5 = parts.next().unwrap();

            if x > xmin && y > ymin {
                if r5 == "1" {
                    let xx = ((x - xmin) / block + 0.5).floor() as u64;
                    let yy = ((y - ymin) / block + 0.5).floor() as u64;
                    *firsthit.entry((xx, yy)).or_insert(0) += 1;
                }

                let xx = ((x - xmin) / size).floor() as u64;
                let yy = ((y - ymin) / size).floor() as u64;
                let a = *xyz.get(&(xx, yy)).unwrap_or(&0.0);
                let b = *xyz.get(&(xx + 1, yy)).unwrap_or(&0.0);
                let c = *xyz.get(&(xx, yy + 1)).unwrap_or(&0.0);
                let d = *xyz.get(&(xx + 1, yy + 1)).unwrap_or(&0.0);

                let distx = (x - xmin) / size - xx as f64;
                let disty = (y - ymin) / size - yy as f64;

                let ab = a * (1.0 - distx) + b * distx;
                let cd = c * (1.0 - distx) + d * distx;
                let thelele = ab * (1.0 - disty) + cd * disty;
                let xx = ((x - xmin) / block / (step as f64) + 0.5).floor() as u64;
                let yy = (((y - ymin) / block / (step as f64)).floor() + 0.5).floor() as u64;
                let hh = h - thelele;
                if hh <= 1.2 {
                    if r3 == "2" {
                        *ugg.entry((xx, yy)).or_insert(0.0) += 1.0;
                    } else if hh > 0.25 {
                        *ug.entry((xx, yy)).or_insert(0) += 1;
                    } else {
                        *ugg.entry((xx, yy)).or_insert(0.0) += 1.0;
                    }
                } else {
                    *ugg.entry((xx, yy)).or_insert(0.0) += 0.05;
                }

                let xx = ((x - xmin) / block + 0.5).floor() as u64;
                let yy = ((y - ymin) / block + 0.5).floor() as u64;
                let yyy = ((y - ymin) / block).floor() as u64; // necessary due to bug in perl version
                if r3 == "2" || greenground >= hh {
                    if r4 == "1" && r5 == "1" {
                        *ghit.entry((xx, yyy)).or_insert(0) += firstandlastreturnasground;
                    } else {
                        *ghit.entry((xx, yyy)).or_insert(0) += 1;
                    }
                } else {
                    let mut last = 1.0;
                    if r4 == r5 {
                        last = lastfactor;
                        if hh < 5.0 {
                            last = firstandlastfactor;
                        }
                    }
                    for zone in zones.iter() {
                        let mut parts = zone.split('|');
                        let low: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                        let high: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                        let roof: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                        let factor: f64 = parts.next().unwrap().parse::<f64>().unwrap();
                        if hh >= low
                            && hh < high
                            && *top.get(&(xx, yy)).unwrap_or(&0.0) - thelele < roof
                        {
                            let offset = factor * last;
                            *greenhit.entry((xx, yy)).or_insert(0.0) += offset;
                            break;
                        }
                    }

                    if greenhigh < hh {
                        *highit.entry((xx, yy)).or_insert(0) += 1;
                    }
                }
            }
        }

        i += 1;
    })
    .expect("Can not read file");

    let w = (xmax - xmin).floor() / block;
    let h = (ymax - ymin).floor() / block;
    let wy = (xmax - xmin).floor() / 3.0;
    let hy = (ymax - ymin).floor() / 3.0;

    let scalefactor: f64 = conf
        .general_section()
        .get("scalefactor")
        .unwrap_or("1")
        .parse::<f64>()
        .unwrap_or(1.0);

    let mut imgug = RgbaImage::from_pixel(
        (w * block * 600.0 / 254.0 / scalefactor) as u32,
        (h * block * 600.0 / 254.0 / scalefactor) as u32,
        Rgba([255, 255, 255, 0]),
    );
    let mut img_ug_bit = GrayImage::from_pixel(
        (w * block * 600.0 / 254.0 / scalefactor) as u32,
        (h * block * 600.0 / 254.0 / scalefactor) as u32,
        Luma([0x00]),
    );
    let mut imggr1 =
        RgbImage::from_pixel((w * block) as u32, (h * block) as u32, Rgb([255, 255, 255]));
    let mut imggr1b =
        RgbImage::from_pixel((w * block) as u32, (h * block) as u32, Rgb([255, 255, 255]));
    let mut imgye2 = RgbaImage::from_pixel(
        (w * block) as u32,
        (h * block) as u32,
        Rgba([255, 255, 255, 0]),
    );
    let mut imgye2b = RgbaImage::from_pixel(
        (w * block) as u32,
        (h * block) as u32,
        Rgba([255, 255, 255, 0]),
    );
    let mut imgwater =
        RgbImage::from_pixel((w * block) as u32, (h * block) as u32, Rgb([255, 255, 255]));

    let mut greens = Vec::new();
    for i in 0..greenshades.len() {
        greens.push(Rgb([
            (greentone - greentone / (greenshades.len() - 1) as f64 * i as f64) as u8,
            (254.0 - (74.0 / (greenshades.len() - 1) as f64) * i as f64) as u8,
            (greentone - greentone / (greenshades.len() - 1) as f64 * i as f64) as u8,
        ]))
    }

    let mut aveg = 0;
    let mut avecount = 0;

    for x in 1..(h as usize) {
        for y in 1..(h as usize) {
            let xx = x as u64;
            let yy = y as u64;
            if *ghit.get(&(xx, yy)).unwrap_or(&0) > 1 {
                aveg += *firsthit.get(&(xx, yy)).unwrap_or(&0);
                avecount += 1;
            }
        }
    }
    let aveg = aveg as f64 / avecount as f64;
    let ye2 = Rgba([255, 219, 166, 255]);
    for x in 4..(wy as usize - 3) {
        for y in 4..(hy as usize - 3) {
            let mut ghit2 = 0;
            let mut highhit2 = 0;

            for i in x..x + 2 {
                for j in y..y + 2 {
                    ghit2 += *yhit.get(&(i as u64, j as u64)).unwrap_or(&0);
                    highhit2 += *noyhit.get(&(i as u64, j as u64)).unwrap_or(&0);
                }
            }
            if ghit2 as f64 / (highhit2 as f64 + ghit2 as f64 + 0.01) > yellowthreshold {
                draw_filled_rect_mut(
                    &mut imgye2,
                    Rect::at(x as i32 * 3 + 2, (hy as i32 - y as i32) * 3 - 3).of_size(3, 3),
                    ye2,
                );
            }
        }
    }

    for x in 2..w as usize {
        for y in 2..h as usize {
            let mut ghit2 = 0;
            let mut highit2 = 0;
            let roof = *top.get(&(x as u64, y as u64)).unwrap_or(&0.0)
                - *xyz
                    .get(&(
                        (x as f64 * block / size).floor() as u64,
                        (y as f64 * block / size).floor() as u64,
                    ))
                    .unwrap_or(&0.0);

            let greenhit2 = *greenhit.get(&(x as u64, y as u64)).unwrap_or(&0.0);
            let mut firsthit2 = *firsthit.get(&(x as u64, y as u64)).unwrap_or(&0);
            for i in (x - 2)..x + 3_usize {
                for j in (y - 2)..y + 3_usize {
                    if firsthit2 > *firsthit.get(&(i as u64, j as u64)).unwrap_or(&0) {
                        firsthit2 = *firsthit.get(&(i as u64, j as u64)).unwrap_or(&0);
                    }
                }
            }
            highit2 += *highit.get(&(x as u64, y as u64)).unwrap_or(&0);
            ghit2 += *ghit.get(&(x as u64, y as u64)).unwrap_or(&0);

            let mut greenlimit = 9999.0;
            for &(v0, v1, v2) in thresholds.iter() {
                if roof >= v0 && roof < v1 {
                    greenlimit = v2;
                    break;
                }
            }

            let mut greenshade = 0;

            let thevalue = greenhit2 / (ghit2 as f64 + greenhit2 + 1.0)
                * (1.0 - topweight
                    + topweight * highit2 as f64
                        / (ghit2 as f64 + greenhit2 + highit2 as f64 + 1.0))
                * (1.0 - pointvolumefactor * firsthit2 as f64 / (aveg + 0.00001))
                    .powf(pointvolumeexponent);
            if thevalue > 0.0 {
                for (i, &shade) in greenshades.iter().enumerate() {
                    if thevalue > greenlimit * shade {
                        greenshade = i + 1;
                    }
                }
                if greenshade > 0 {
                    draw_filled_rect_mut(
                        &mut imggr1,
                        Rect::at(
                            ((x as f64 + 0.5) * block) as i32 - addition,
                            (((h - y as f64) - 0.5) * block) as i32 - addition,
                        )
                        .of_size(
                            (block as i32 + addition) as u32,
                            (block as i32 + addition) as u32,
                        ),
                        *greens.get(greenshade - 1).unwrap(),
                    );
                }
            }
        }
    }
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
    if med > 0 {
        imggr1b = median_filter(&imggr1, med / 2, med / 2);
        if proceed_yellows {
            imgye2b = median_filter(&imgye2, med / 2, med / 2);
        }
    }
    let med2: u32 = conf
        .general_section()
        .get("medianboxsize2")
        .unwrap_or("0")
        .parse::<u32>()
        .unwrap_or(0);
    if med2 > 0 {
        imggr1 = median_filter(&imggr1b, med2 / 2, med2 / 2);
        if proceed_yellows {
            imgye2 = median_filter(&imgye2, med / 2, med / 2);
        }
    } else {
        imggr1 = imggr1b;
        if proceed_yellows {
            imgye2 = imgye2b;
        }
    }

    imgye2
        .save(Path::new(&format!("{}/yellow.png", tmpfolder)))
        .expect("could not save output png");
    imggr1
        .save(Path::new(&format!("{}/greens.png", tmpfolder)))
        .expect("could not save output png");

    let mut img =
        image::open(Path::new(&format!("{}/greens.png", tmpfolder))).expect("Opening image failed");
    let img2 =
        image::open(Path::new(&format!("{}/yellow.png", tmpfolder))).expect("Opening image failed");
    image::imageops::overlay(&mut img, &img2, 0, 0);
    img.save(Path::new(&format!("{}/vegetation.png", tmpfolder)))
        .expect("could not save output png");

    if vege_bitmode {
        let g_img = image::open(Path::new(&format!("{}/greens.png", tmpfolder)))
            .expect("Opening image failed");
        let mut g_img = g_img.to_rgb8();
        for pixel in g_img.pixels_mut() {
            let mut found = false;
            for (idx, color) in greens.iter().enumerate() {
                let c = idx as u8 + 2;
                if pixel[0] == color[0] && pixel[1] == color[1] && pixel[2] == color[2] {
                    *pixel = Rgb([c, c, c]);
                    found = true;
                }
            }
            if !found {
                *pixel = Rgb([0, 0, 0]);
            }
        }
        g_img
            .save(Path::new(&format!("{}/greens_bit.png", tmpfolder)))
            .expect("could not save output png");
        let g_img = image::open(Path::new(&format!("{}/greens_bit.png", tmpfolder)))
            .expect("Opening image failed");
        let g_img = g_img.to_luma8();
        g_img
            .save(Path::new(&format!("{}/greens_bit.png", tmpfolder)))
            .expect("could not save output png");

        let y_img = image::open(Path::new(&format!("{}/yellow.png", tmpfolder)))
            .expect("Opening image failed");
        let mut y_img = y_img.to_rgba8();
        for pixel in y_img.pixels_mut() {
            if pixel[0] == ye2[0] && pixel[1] == ye2[1] && pixel[2] == ye2[2] && pixel[3] == ye2[3]
            {
                *pixel = Rgba([1, 1, 1, 255]);
            } else {
                *pixel = Rgba([0, 0, 0, 0]);
            }
        }
        y_img
            .save(Path::new(&format!("{}/yellow_bit.png", tmpfolder)))
            .expect("could not save output png");
        let y_img = image::open(Path::new(&format!("{}/yellow_bit.png", tmpfolder)))
            .expect("Opening image failed");
        let y_img = y_img.to_luma_alpha8();
        y_img
            .save(Path::new(&format!("{}/yellow_bit.png", tmpfolder)))
            .expect("could not save output png");

        let mut img_bit = image::open(Path::new(&format!("{}/greens_bit.png", tmpfolder)))
            .expect("Opening image failed");
        let img_bit2 = image::open(Path::new(&format!("{}/yellow_bit.png", tmpfolder)))
            .expect("Opening image failed");
        image::imageops::overlay(&mut img_bit, &img_bit2, 0, 0);
        img_bit
            .save(Path::new(&format!("{}/vegetation_bit.png", tmpfolder)))
            .expect("could not save output png");
    }

    let black = Rgb([0, 0, 0]);
    let blue = Rgb([29, 190, 255]);
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
    if buildings > 0 || water > 0 {
        read_lines_no_alloc(xyz_file_in, |line| {
            let mut parts = line.split(' ');
            let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let c: u64 = parts.next().unwrap().parse::<u64>().unwrap();

            if c == buildings {
                draw_filled_rect_mut(
                    &mut imgwater,
                    Rect::at((x - xmin) as i32 - 1, (ymax - y) as i32 - 1).of_size(3, 3),
                    black,
                );
            }
            if c == water {
                draw_filled_rect_mut(
                    &mut imgwater,
                    Rect::at((x - xmin) as i32 - 1, (ymax - y) as i32 - 1).of_size(3, 3),
                    blue,
                );
            }
        })
        .expect("Can not read file");
    }
    let waterele = conf
        .general_section()
        .get("waterelevation")
        .unwrap_or("")
        .parse::<f64>()
        .unwrap_or(-999999.0);
    let path = format!("{}/xyz2.xyz", tmpfolder);
    let xyz_file_in = Path::new(&path);

    read_lines_no_alloc(xyz_file_in, |line| {
        let mut parts = line.split(' ');
        let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
        let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
        let hh: f64 = parts.next().unwrap().parse::<f64>().unwrap();

        if hh < waterele {
            draw_filled_rect_mut(
                &mut imgwater,
                Rect::at((x - xmin) as i32 - 1, (ymax - y) as i32 - 1).of_size(3, 3),
                blue,
            );
        }
    })
    .expect("Can not read file");

    imgwater
        .save(Path::new(&format!("{}/blueblack.png", tmpfolder)))
        .expect("could not save output png");

    let underg = Rgba([64, 121, 0, 255]);
    let tmpfactor = (600.0 / 254.0 / scalefactor) as f32;

    let bf32 = block as f32;
    let hf32 = h as f32;
    let ww = w as f32 * bf32;
    let hh = hf32 * bf32;
    let mut x = 0.0_f32;

    loop {
        if x >= ww {
            break;
        }
        let mut y = 0.0_f32;
        loop {
            if y >= hh {
                break;
            }
            let xx = ((x / bf32 / step).floor()) as u64;
            let yy = ((y / bf32 / step).floor()) as u64;
            let foo = *ug.get(&(xx, yy)).unwrap_or(&0) as f64
                / (*ug.get(&(xx, yy)).unwrap_or(&0) as f64
                    + { *ugg.get(&(xx, yy)).unwrap_or(&0.0) }
                    + 0.01);
            if foo > uglimit {
                draw_line_segment_mut(
                    &mut imgug,
                    (
                        tmpfactor * (x + bf32 * 3.0),
                        tmpfactor * (hf32 * bf32 - y - bf32 * 3.0),
                    ),
                    (
                        tmpfactor * (x + bf32 * 3.0),
                        tmpfactor * (hf32 * bf32 - y + bf32 * 3.0),
                    ),
                    underg,
                );
                draw_line_segment_mut(
                    &mut imgug,
                    (
                        tmpfactor * (x + bf32 * 3.0) + 1.0,
                        tmpfactor * (hf32 * bf32 - y - bf32 * 3.0),
                    ),
                    (
                        tmpfactor * (x + bf32 * 3.0) + 1.0,
                        tmpfactor * (hf32 * bf32 - y + bf32 * 3.0),
                    ),
                    underg,
                );
                draw_line_segment_mut(
                    &mut imgug,
                    (
                        tmpfactor * (x - bf32 * 3.0),
                        tmpfactor * (hf32 * bf32 - y - bf32 * 3.0),
                    ),
                    (
                        tmpfactor * (x - bf32 * 3.0),
                        tmpfactor * (hf32 * bf32 - y + bf32 * 3.0),
                    ),
                    underg,
                );
                draw_line_segment_mut(
                    &mut imgug,
                    (
                        tmpfactor * (x - bf32 * 3.0) + 1.0,
                        tmpfactor * (hf32 * bf32 - y - bf32 * 3.0),
                    ),
                    (
                        tmpfactor * (x - bf32 * 3.0) + 1.0,
                        tmpfactor * (hf32 * bf32 - y + bf32 * 3.0),
                    ),
                    underg,
                );

                if vege_bitmode {
                    draw_filled_circle_mut(
                        &mut img_ug_bit,
                        (
                            (tmpfactor * (x)) as i32,
                            (tmpfactor * (hf32 * bf32 - y)) as i32,
                        ),
                        (bf32 * 9.0 * SQRT_2) as i32,
                        Luma([0x01]),
                    )
                }
            }
            if foo > uglimit2 {
                draw_line_segment_mut(
                    &mut imgug,
                    (tmpfactor * x, tmpfactor * (hf32 * bf32 - y - bf32 * 3.0)),
                    (tmpfactor * x, tmpfactor * (hf32 * bf32 - y + bf32 * 3.0)),
                    underg,
                );
                draw_line_segment_mut(
                    &mut imgug,
                    (
                        tmpfactor * x + 1.0,
                        tmpfactor * (hf32 * bf32 - y - bf32 * 3.0),
                    ),
                    (
                        tmpfactor * x + 1.0,
                        tmpfactor * (hf32 * bf32 - y + bf32 * 3.0),
                    ),
                    underg,
                );

                if vege_bitmode {
                    draw_filled_circle_mut(
                        &mut img_ug_bit,
                        (
                            (tmpfactor * (x)) as i32,
                            (tmpfactor * (hf32 * bf32 - y)) as i32,
                        ),
                        (bf32 * 9.0 * SQRT_2) as i32,
                        Luma([0x02]),
                    )
                }
            }

            y += bf32 * step;
        }
        x += bf32 * step;
    }
    imgug
        .save(Path::new(&format!("{}/undergrowth.png", tmpfolder)))
        .expect("could not save output png");
    let img_ug_bit_b = median_filter(&img_ug_bit, (bf32 * step) as u32, (bf32 * step) as u32);
    img_ug_bit_b
        .save(Path::new(&format!("{}/undergrowth_bit.png", tmpfolder)))
        .expect("could not save output png");

    let ugpgw = File::create(Path::new(&format!("{}/undergrowth.pgw", tmpfolder)))
        .expect("Unable to create file");
    let mut ugpgw = BufWriter::new(ugpgw);
    write!(
        &mut ugpgw,
        "{}\r\n0.0\r\n0.0\r\n{}\r\n{}\r\n{}\r\n",
        1.0 / tmpfactor,
        -1.0 / tmpfactor,
        xmin,
        ymax,
    )
    .expect("Cannot write pgw file");

    let vegepgw = File::create(Path::new(&format!("{}/vegetation.pgw", tmpfolder)))
        .expect("Unable to create file");
    let mut vegepgw = BufWriter::new(vegepgw);
    write!(
        &mut vegepgw,
        "1.0\r\n0.0\r\n0.0\r\n-1.0\r\n{}\r\n{}\r\n",
        xmin, ymax
    )
    .expect("Cannot write pgw file");

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
