use log::info;
use pullauta::config::Config;
use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::{thread, time};

fn main() {
    // setup and configure logging, default to INFO when RUST_LOG is not set
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let mut thread: String = String::new();

    let config =
        Arc::new(Config::load_or_create_default().expect("Could not open or create config file"));

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

    let batch: bool = config.batch;

    let tmpfolder = format!("temp{}", thread);
    fs::create_dir_all(&tmpfolder).expect("Could not create tmp folder");

    let pnorthlinesangle = config.pnorthlinesangle;
    let pnorthlineswidth = config.pnorthlineswidth;

    if command.is_empty() && Path::new(&format!("{}/vegetation.png", tmpfolder)).exists() && !batch
    {
        info!("Rendering png map with depressions");
        pullauta::render::render(&config, &thread, pnorthlinesangle, pnorthlineswidth, false)
            .unwrap();
        info!("Rendering png map without depressions");
        pullauta::render::render(&config, &thread, pnorthlinesangle, pnorthlineswidth, true)
            .unwrap();
        info!("\nAll done!");
        return;
    }

    if command.is_empty() && !batch {
        println!("USAGE:\npullauta [parameter 1] [parameter 2] [parameter 3] ... [parameter n]\nSee README.MD for more details");
        return;
    }

    if command == "cliffgeneralize" {
        info!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "ground" {
        info!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "ground2" {
        info!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "groundfix" {
        info!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "profile" {
        info!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "makecliffsold" {
        info!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "makeheight" {
        info!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "xyzfixer" {
        info!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "vege" {
        info!("Not implemented in this version, use the perl version");
        return;
    }

    if command == "blocks" {
        pullauta::blocks::blocks(&thread).unwrap();
        return;
    }

    if command == "dotknolls" {
        pullauta::knolls::dotknolls(&config, &thread).unwrap();
        return;
    }

    if command == "dxfmerge" || command == "merge" {
        pullauta::merge::dxfmerge(&config).unwrap();
        if command == "merge" {
            let mut scale = 1.0;
            if !args.is_empty() {
                scale = args[0].parse::<f64>().unwrap();
            }
            pullauta::merge::pngmergevege(&config, scale).unwrap();
        }
        return;
    }

    if command == "knolldetector" {
        pullauta::knolls::knolldetector(&config, &thread).unwrap();
        return;
    }

    if command == "makecliffs" {
        pullauta::cliffs::makecliffs(&config, &thread).unwrap();
        return;
    }

    if command == "makevege" {
        pullauta::vegetation::makevege(&config, &thread).unwrap();
    }

    if command == "pngmerge" || command == "pngmergedepr" {
        let mut scale = 4.0;
        if !args.is_empty() {
            scale = args[0].parse::<f64>().unwrap();
        }
        pullauta::merge::pngmerge(&config, scale, command == "pngmergedepr").unwrap();
        return;
    }

    if command == "pngmergevege" {
        let mut scale = 1.0;
        if !args.is_empty() {
            scale = args[0].parse::<f64>().unwrap();
        }
        pullauta::merge::pngmergevege(&config, scale).unwrap();
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
        pullauta::merge::smoothjoin(&config, &thread).unwrap();
    }

    if command == "xyzknolls" {
        pullauta::knolls::xyzknolls(&config, &thread).unwrap();
    }

    if command == "unzipmtk" {
        pullauta::process::unzipmtk(&config, &thread, &args).unwrap();
    }

    if command == "mtkshaperender" {
        pullauta::render::mtkshaperender(&config, &thread).unwrap();
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
            &config,
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
        pullauta::render::render(&config, &thread, angle, nwidth, nodepressions).unwrap();
        return;
    }

    let proc = config.processes;
    if command.is_empty() && batch && proc > 1 {
        let mut handles: Vec<thread::JoinHandle<()>> = Vec::with_capacity((proc + 1) as usize);
        for i in 0..proc {
            let config = config.clone();
            let handle = thread::spawn(move || {
                info!("Starting thread {}", i + 1);
                pullauta::process::batch_process(&config, &format!("{}", i + 1));
                info!("Thread {} complete", i + 1);
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
        pullauta::process::batch_process(&config, &thread)
    }

    let zip_files_re = Regex::new(r"\.zip$").unwrap();
    if zip_files_re.is_match(&command.to_lowercase()) {
        let mut zips: Vec<String> = vec![command];
        zips.extend(args);
        pullauta::process::process_zip(&config, &thread, &zips).unwrap();
        return;
    }

    if accepted_files_re.is_match(&command.to_lowercase()) {
        let mut norender: bool = false;
        if args.len() > 1 {
            norender = args[1].clone() == "norender";
        }
        pullauta::process::process_tile(&config, &thread, &command, norender).unwrap();
    }
}
