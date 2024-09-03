use pullauta::config::Config;
use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use std::{thread, time};

fn main() {
    let mut thread: String = String::new();

    let config = Config::load_or_create_default().expect("Could not open or create config file");

    let conf = config.conf;

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
