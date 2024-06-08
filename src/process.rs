use image::{GrayImage, Luma, Rgb, RgbImage, Rgba, RgbaImage};
use ini::Ini;
use las::{raw::Header, Read, Reader};
use rand::distributions;
use rand::prelude::*;
use regex::Regex;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::blocks;
use crate::cliffs;
use crate::contours;
use crate::crop;
use crate::knolls;
use crate::merge;
use crate::render;
use crate::util::read_lines;
use crate::vegetation;

pub fn process_zip(thread: &String, filenames: &Vec<String>) -> Result<(), Box<dyn Error>> {
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
    render::render(thread, pnorthlinesangle, pnorthlineswidth, false).unwrap();
    println!("Rendering png map without depressions");
    render::render(thread, pnorthlinesangle, pnorthlineswidth, true).unwrap();
    Ok(())
}

pub fn unzipmtk(thread: &String, filenames: &Vec<String>) -> Result<(), Box<dyn Error>> {
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
        render::mtkshaperender(thread).unwrap();
    }
    Ok(())
}

pub fn process_tile(
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
        contours::xyz2contours(
            thread,
            scalefactor * 0.3,
            "xyztemp.xyz",
            "xyz_03.xyz",
            "contours03.dxf",
            true,
        )
        .expect("contour generation failed");
    } else {
        contours::xyz2contours(
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
            contours::xyz2contours(
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
            knolls::knolldetector(thread).unwrap();
        }
        println!("{}Contour generation part 1", thread_name);
        knolls::xyzknolls(thread).unwrap();

        println!("{}Contour generation part 2", thread_name);
        if !skipknolldetection {
            // contours 2.5
            contours::xyz2contours(
                thread,
                halfinterval,
                "xyz_knolls.xyz",
                "null",
                "out.dxf",
                false,
            )
            .unwrap();
        } else {
            contours::xyz2contours(thread, halfinterval, "xyztemp.xyz", "null", "out.dxf", true)
                .unwrap();
        }
        println!("{}Contour generation part 3", thread_name);
        merge::smoothjoin(thread).unwrap();
        println!("{}Contour generation part 4", thread_name);
        knolls::dotknolls(thread).unwrap();
    }

    println!("{}Vegetation generation", thread_name);
    vegetation::makevegenew(thread).unwrap();

    if !vegeonly {
        println!("{}Cliff generation", thread_name);
        cliffs::makecliffs(thread).unwrap();
    }
    let detectbuildings: bool = conf.general_section().get("detectbuildings").unwrap_or("0") == "1";
    if detectbuildings {
        println!("{}Detecting buildings", thread_name);
        blocks::blocks(thread).unwrap();
    }
    if !skip_rendering {
        println!("{}Rendering png map with depressions", thread_name);
        render::render(thread, pnorthlinesangle, pnorthlineswidth, false).unwrap();
        println!("{}Rendering png map without depressions", thread_name);
        render::render(thread, pnorthlinesangle, pnorthlineswidth, true).unwrap();
    } else {
        println!("{}Skipped rendering", thread_name);
    }
    println!("\n\n{}All done!", thread_name);
    Ok(())
}

pub fn batch_process(thread: &String) {
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
                crop::polylinedxfcrop(
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
                    crop::polylinedxfcrop(
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
                crop::pointdxfcrop(
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
            crop::polylinedxfcrop(
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
