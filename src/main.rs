use std::env;
use regex::Regex;
use std::path::Path;
extern crate ini;
use ini::Ini;
use std::{thread, time};
use std::error::Error;
use std::fs::File;
use std::fs;
use std::io::{self, BufRead};
use image::io::Reader as ImageReader;
use std::process::Command;
use std::io::{BufWriter, Write};
use std::fs::OpenOptions;
extern crate csv;
extern crate serde;
// This lets us write `#[derive(Deserialize)]`.
#[macro_use]
extern crate serde_derive;


fn main() {
    let mut thread: String = String::new();
    if Path::new("pullauta.ini").exists() {
        // nothing
    } else {
        // TODO: create the ini file
    }

    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    
    let int_re = Regex::new(r"^[1-9]\d*$").unwrap();

    let mut args: Vec<String> = env::args().collect();
    
    args.remove(0); // program name
    
    if args.len() > 0 && int_re.is_match(&args[0]) {
        thread = args.remove(0);
    }

    let mut command: String = String::new();
    if args.len() > 0{
        command = args.remove(0);
    }

    let accepted_files_re = Regex::new(r"\.(las|laz|xyz)$").unwrap();
    if command == "" || accepted_files_re.is_match(&command.to_lowercase())  {
        println!("This is rusty karttapulatin... There is no warranty. Use it at your own risk!\n");
    }
    
    // println!("Hello thread {}, command {}!", thread, command);

    let batch: bool = conf.general_section().get("batch").unwrap() == "1";

    let tmpfolder = format!("temp{}", thread);
    fs::create_dir_all(&tmpfolder).expect("Could not create tmp folder");

    if command == "" && Path::new(&format!("{}/vegetation.png", tmpfolder)).exists() && !batch {
        println!("Rendering png map with depressions");
        // TODO: run `pullauta render $pnorthlinesangle $pnorthlineswidth`
        println!("Rendering png map without depressions");
        // TODO: run `pullauta render $pnorthlinesangle $pnorthlineswidth  nodepressions`
        println!("\nAll done!");
        return();
    }

    if command == "" && !batch {
        println!("USAGE:\npullauta [parameter 1] [parameter 2] [parameter 3] ... [parameter n]\nSee readme.txt for more details");
    }

    if command == "groundfix" {
        println!("Not implemented");
        return();
    }

    if command == "profile" {
        println!("Not implemented");
        return();
    }
    
    if command == "ground" {
        println!("Not implemented");
        return();
    }

    if command == "ground2" {
        println!("Not implemented");
        return();
    }

    if command == "ground2" {
        println!("Not implemented");
        return();
    }

    if command == "blocks" {
        println!("Not implemented");
        return();
    }

    if command == "dxfmerge" || command == "merge" {
        println!("Not implemented");
        return();
    }

    if command == "pngmerge" || command == "pngmergedepr" {
        println!("Not implemented");
        return();
    }

    if command == "pngmergevege" {
        println!("Not implemented");
        return();
    }

    fn batch_process(thread: &String) {
        println!("Pullauta thread started  {}", thread);
        let _thread_number = thread.parse::<u64>().unwrap_or(0);
        println!("Not implemented further");
        return();
    }

    let proc: u64 = conf.general_section().get("processes").unwrap().parse::<u64>().unwrap();
    if command == "" && batch && proc > 1 {
        let handle = thread::spawn(move || {
          for i in 0..proc {
                println!("Starting thread {}", i + 1);
                batch_process(&format!("{}", i + 1)); 
                thread::sleep(time::Duration::from_millis(100));
            }
        });
        handle.join().unwrap();
        return();
    }

    if (command == "" && batch && proc < 2 ) || (command == "startthread" && batch) {
        thread = String::from("0");
        if args.len() > 0 {
            thread = args[0].clone();
        }
        if thread == "0" {
            thread = String::from("");
        }
        batch_process(&thread)
    }

    if accepted_files_re.is_match(&command) {
        println!("Preparing input file");

        let mut skiplaz2txt: bool = false;
        if Regex::new(r".xyz$").unwrap().is_match(&command.to_lowercase()) {
            println!(".xyz input file");
            if let Ok(lines) = read_lines(&command) {
                let mut i: u32 = 0;
                for line in lines {
                    if  i == 2 {
                        let ip = line.unwrap_or(String::new());
                        let parts = ip.split(" ");
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
            let out = Command::new("las2txt")
                    .arg("-version")
                    .output()
                    .expect("las2txt command failed to start");
            if out.status.success() {
                println!("Not implemented further");
                return();
            } else {
                println!("Can't find las2txt. It is needed if input file is not xyz file with xyzc data. Make sure it is in path or copy it to the same folder as pullautin");
                return();
            }
        } else {
            fs::copy(&command, format!("{}/xyztemp.xyz", tmpfolder)).expect("Could not copy file to tmpfolder");
        }
        println!("Done");
        println!("Knoll detection part 1");
        let scalefactor: f64 = conf.general_section().get("scalefactor").unwrap_or("1").parse::<f64>().unwrap_or(1.0);
        xyz2countours(&thread, scalefactor * 0.3, "xyztemp.xyz", "xyz_03.xyz", "contours03.dxf", true).expect("countour generation failed");
        
        fs::copy(format!("{}/xyz_03.xyz", tmpfolder), format!("{}/xyz2.xyz", tmpfolder)).expect("Could not copy file");
        
        let basemapcontours: f64 = conf.general_section().get("basemapinterval").unwrap_or("0").parse::<f64>().unwrap_or(0.0);

        if basemapcontours != 0.0 {
            println!("Basemap contours");
            xyz2countours(&thread, basemapcontours, "xyz2.xyz", "", "basemap.dxf", false).expect("countour generation failed");
        }
        
        if conf.general_section().get("skipknolldetection").unwrap_or("0") == "1" {
            println!("Knoll detection part 2");
            // TODO: Run `pulauta knolldetector`
        }
    
        println!("Contour generation part 1");
        // TODO: Run `pulauta xyzknolls`
    
        if conf.general_section().get("skipknolldetection").unwrap_or("0") == "1" {
            // contours 2.5
            println!("Contour generation part 2");
            xyz2countours(&thread, 2.5 * scalefactor, "xyz_knolls.xyz", "", "out.dxf", false).expect("countour generation failed");
        } else {
            xyz2countours(&thread, basemapcontours, "xyztemp.xyz", "", "out.dxf", true).expect("countour generation failed");
        }
        println!("Not implemented further");
    }
}


fn xyz2countours(thread: &String, cinterval: f64, xyzfilein: &str, xyzfileout: &str, dxffile: &str, ground: bool) -> Result<(), Box<dyn Error>> {
    println!("Running xyz2countours {} {} {} {} {} {}", thread, cinterval, xyzfilein, xyzfileout, dxffile, ground);
    println!(".");

    let conf = Ini::load_from_file("pullauta.ini").unwrap();
    let scalefactor: f64 = conf.general_section().get("scalefactor").unwrap_or("1").parse::<f64>().unwrap_or(1.0);
    let water_class: u32 = conf.general_section().get("waterclass").unwrap_or("9").parse::<u32>().unwrap_or(9);

    let tmpfolder = format!("temp{}", thread);

    let mut xmin: f64 = std::f64::MAX;
    let mut xmax: f64 = std::f64::MIN; 

    let mut ymin: f64 = std::f64::MAX;
    let mut ymax: f64 = std::f64::MIN;
    
    let mut hmin: f64 = std::f64::MAX; 
    let mut hmax: f64 = std::f64::MIN;
    
    let xyz_file_in = format!("{}/{}", tmpfolder, xyzfilein);
    
    #[derive(Debug, Deserialize)]
    struct XYZCRecord {
        x: f64,
        y: f64,
        h: f64,
        c: u32,
        _x: u32,
        _y: u32,
        _z: u32
    }

    let mut rdr = csv::ReaderBuilder::new().delimiter(b' ').has_headers(false).from_path(&xyz_file_in)?;
    let mut iter = rdr.deserialize();
    while let Some(result) = iter.next() {
        let r: XYZCRecord = result.unwrap();
        if r.c == 2 || r.c == water_class || !ground {
            let x = r.x;
            let y = r.y;
            let h = r.h;
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

    xmin = (xmin / 2.0 / scalefactor).floor() * 2.0 * scalefactor;
    ymin = (ymin / 2.0 / scalefactor).floor() * 2.0 * scalefactor;

    let w: usize = ((xmax - xmin).ceil() / 2.0 / scalefactor) as usize;
    let h: usize = ((ymax - ymin).ceil() / 2.0 / scalefactor) as usize;
    
    println!("{} {}", w, h);

    let mut list_alt = vec![vec![Vec::new(); w + 1]; h + 1];
    let mut rdr2 = csv::ReaderBuilder::new().delimiter(b' ').has_headers(false).from_path(&xyz_file_in)?;
    let mut iter2 = rdr2.deserialize();
    while let Some(result) = iter2.next() {
        let r: XYZCRecord = result.unwrap();
        if r.c == 2 || r.c == water_class || !ground {
            list_alt[((r.x - xmin).floor() / 2.0 / scalefactor) as usize][((r.y - ymin).floor() / 2.0 / scalefactor) as usize].push(r.h);
        }
    }
    let mut avg_alt = vec![vec![f64::NAN; w + 1]; h + 1];

    for x in 0..w {
        for y in 0..h {
            if !list_alt[x][y].is_empty() {
                avg_alt[x][y] = average(&list_alt[x][y]);
            }
        }
    }

    let count: u32 = 0;
    let count2: u32 = 0;
    let row: u32 = 0;

    for x in 0..w {
        for y in 0..h {
            if avg_alt[x][y].is_nan() {
                // interpolate altitude of pixel
                // Todo: optimize to first clasify area then assign values
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

                while  j1 > 0 && avg_alt[x][j1].is_nan(){
                    j1 -= 1;
                }

                while j2 < h && avg_alt[x][j2].is_nan() {
                    j2 += 1;
                }

                let mut val1 = f64::NAN;
                let mut val2 = f64::NAN;
                
                if !avg_alt[i1][y].is_nan() && !avg_alt[i2][y].is_nan() {
                    val1 = ((i2 - x) as f64 * avg_alt[i1][y] + (x - i1) as f64 * avg_alt[i2][y]) / ((i2 - i1) as f64);
                }

                if !avg_alt[x][j1].is_nan() && !avg_alt[x][j2].is_nan() {
                    val2 = ((j2 - y) as f64 * avg_alt[x][j1] + (y - j1) as f64 * avg_alt[x][j2]) / ((j2 - j1) as f64);
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


    for x in 0..w {
        for y in 0..h {
            if avg_alt[x][y].is_nan() {
                // second round of interpolation of altitude of pixel
                let mut val: f64 = 0.0;
                let mut c   = 0;
                for i in 0..3 {
                    let ii: i32 = i - 1;
                    for j in 0..3 {
                        let jj: i32 = j - 1;
                        if y as i32 + jj > 0 && x as i32 + ii > 0 {
                            let x_idx = (x as i32 + ii) as usize;
                            let y_idx = (y as i32 + jj) as usize;
                            if x_idx < w && y_idx < h && !avg_alt[x_idx][y_idx].is_nan() {
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

    for x in 0..w {
        for y in 1..h {
            if avg_alt[x][y].is_nan() { 
                avg_alt[x][y] = avg_alt[x][y - 1]; 
            }
        }
        for y in 1..h {
            let yy = h - y;
            if avg_alt[x][h].is_nan() { 
                avg_alt[x][y] = avg_alt[x][yy + 1]; 
            }
        }

    }

    xmin += 1.0;
    ymin += 1.0;

    for x in 0..w {
        for y in 0..h {
            let mut ele = avg_alt[x][y];
            let temp: f64 = (ele / cinterval + 0.5).floor() * cinterval;
            if (ele - temp).abs() < 0.02 {
                if ele - temp < 0.0 {
                    ele = temp - 0.02;
                }
                else {
                    ele = temp + 0.02;
                }
                avg_alt[x][y] = ele;
            }
        }
    }
    if xyzfileout != "" {
        let xyz_file_out = format!("{}/{}", tmpfolder, xyzfileout);
        let f = File::create(&xyz_file_out).expect("Unable to create file");
        let mut f = BufWriter::new(f);
        for x in 0..w {
            for y in 0..h {
                let ele = avg_alt[x][y];
                f.write(
                    format!(
                        "{} {} {}\n",
                        x as f64 * 2.0 * scalefactor + xmin,
                        y as f64 * 2.0 * scalefactor + ymin,
                        ele
                    ).as_bytes()
                ).expect("Cannot write to output file");
            }
        }
    }

    let v = cinterval;

    let mut progress: f64 = 0.0;
    let mut progprev: f64 = 0.0;
    let total: f64 = (hmax - hmin) / v;
    let mut level: f64 = (hmin / v).floor() * v;
    let polyline_out = format!("{}/temp_polylines.txt", tmpfolder);
    while level < hmax {
        level += v;
        progress += 1.0;
        if (progress / total * 18.0).floor() > progprev {
            progprev = (progress / total * 18.0).floor();
            println!("Generating temp polylines: {}%", (progress / total * 100.0).floor() as u32);
        }
        let obj = Vec::<String>::new();

        for i in 0..w-1 {
            for j in 0..h-1 {
                let a = avg_alt[i][j];
                let b = avg_alt[i][j + 1];
                let c = avg_alt[i + 1][j];
                let d = avg_alt[i + 1][j + 1];
                if a < level && b < level && c < level && d < level || a > level && b > level && c > level && d > level {
                    // skip
                } else {

                }
            }
        }
        let f = OpenOptions::new().create(true).append(true).open(&polyline_out).expect("Unable to create file");
        let mut f = BufWriter::new(f);
        f.write(format!("Content comming: level {}\n", level).as_bytes()).expect("Cannot write to output file");
    }

    println!("Not implemented futher");
    Ok(())
}


fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn average(numbers: &Vec<f64>) -> f64 {
    let mut sum = 0.0;
    for n in numbers {
        sum += n;
    }
    sum / numbers.len() as f64 
}

fn check_obj_in (obj: &Vec<String>, x1: f64, x2: f64, y1: f64, y2: f64) {
    let x1 = (x1 * 100.0).floor() / 100.0;
    let x2 = (x1 * 100.0).floor() / 100.0;
    let y1 = (x1 * 100.0).floor() / 100.0;
    let y2 = (x1 * 100.0).floor() / 100.0;
}