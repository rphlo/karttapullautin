use ini::Ini;
use regex::Regex;
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::{thread, time};

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

# Note, you will need to iterate this if you use this mode. with commands 'pullauta makevege' and then 'pullauta' you can process only this part again. 
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

# vegeonly, set to 1 to only generate the vegetations related files and skip the rest
vegeonly=0
# contoursonly, set to 1 to only generate the contours related files and skip the rest
contoursonly=0
# cliffsonly, set to 1 to only generate the cliffs related files and skip the rest
cliffsonly=0
# Only one of vegeonly contoursonly and cliffsonly can be set at a time

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

    let vegeonly: bool = conf.general_section().get("vegeonly").unwrap_or("0") == "1";
    let cliffsonly: bool = conf.general_section().get("cliffsonly").unwrap_or("0") == "1";
    let contoursonly: bool = conf.general_section().get("contoursonly").unwrap_or("0") == "1";

    if (vegeonly && (cliffsonly || contoursonly))
        || (cliffsonly && (vegeonly || contoursonly))
        || (contoursonly && (vegeonly || cliffsonly))
    {
        println!("Only one of vegeonly, cliffsonly, or contoursonly can be set!\n");
        return;
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
        pullauta::render::render(&thread, pnorthlinesangle, pnorthlineswidth, false).unwrap();
        println!("Rendering png map without depressions");
        pullauta::render::render(&thread, pnorthlinesangle, pnorthlineswidth, true).unwrap();
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

    if command == "makevege" {
        pullauta::vegetation::makevege(&thread).unwrap();
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
        pullauta::crop::polylinedxfcrop(dxffilein, dxffileout, minx, miny, maxx, maxy).unwrap();
        return;
    }

    if command == "pointdxfcrop" {
        let dxffilein = Path::new(&args[0]);
        let dxffileout = Path::new(&args[1]);
        let minx = args[2].parse::<f64>().unwrap();
        let miny = args[3].parse::<f64>().unwrap();
        let maxx = args[4].parse::<f64>().unwrap();
        let maxy = args[5].parse::<f64>().unwrap();
        pullauta::crop::pointdxfcrop(dxffilein, dxffileout, minx, miny, maxx, maxy).unwrap();
        return;
    }

    if command == "smoothjoin" {
        pullauta::merge::smoothjoin(&thread).unwrap();
    }

    if command == "xyzknolls" {
        pullauta::knolls::xyzknolls(&thread).unwrap();
    }

    if command == "unzipmtk" {
        pullauta::process::unzipmtk(&thread, &args).unwrap();
    }

    if command == "mtkshaperender" {
        pullauta::render::mtkshaperender(&thread).unwrap();
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
        pullauta::render::render(&thread, angle, nwidth, nodepressions).unwrap();
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
                pullauta::process::batch_process(&format!("{}", i + 1));
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
        pullauta::process::batch_process(&thread)
    }

    let zip_files_re = Regex::new(r"\.zip$").unwrap();
    if zip_files_re.is_match(&command.to_lowercase()) {
        let mut zips: Vec<String> = vec![command];
        zips.extend(args);
        pullauta::process::process_zip(&thread, &zips).unwrap();
        return;
    }

    if accepted_files_re.is_match(&command.to_lowercase()) {
        let mut norender: bool = false;
        if args.len() > 1 {
            norender = args[1].clone() == "norender";
        }
        pullauta::process::process_tile(&thread, &command, norender).unwrap();
    }
}
