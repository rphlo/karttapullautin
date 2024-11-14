use image::{GrayImage, Luma, Rgb, RgbImage, Rgba, RgbaImage};
use las::{raw::Header, Reader};
use log::info;
use rand::distributions;
use rand::prelude::*;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::blocks;
use crate::cliffs;
use crate::config::Config;
use crate::contours;
use crate::crop;
use crate::io::xyz::{XyzInternalWriter, XyzRecordMeta};
use crate::knolls;
use crate::merge;
use crate::render;
use crate::util::read_lines;
use crate::util::read_lines_no_alloc;
use crate::util::Timing;
use crate::vegetation;

pub fn process_zip(
    config: &Config,
    thread: &String,
    tmpfolder: &Path,
    filenames: &[String],
) -> Result<(), Box<dyn Error>> {
    let mut timing = Timing::start_now("process_zip");
    let &Config {
        pnorthlineswidth,
        pnorthlinesangle,
        ..
    } = config;

    info!("Rendering shape files");
    timing.start_section("unzip and render shape files");
    unzipmtk(config, tmpfolder, filenames).unwrap();

    info!("Rendering png map with depressions");
    timing.start_section("Rendering png map with depressions");
    render::render(
        config,
        thread,
        tmpfolder,
        pnorthlinesangle,
        pnorthlineswidth,
        false,
    )
    .unwrap();

    info!("Rendering png map without depressions");
    timing.start_section("Rendering png map without depressions");
    render::render(
        config,
        thread,
        tmpfolder,
        pnorthlinesangle,
        pnorthlineswidth,
        true,
    )
    .unwrap();

    Ok(())
}

pub fn unzipmtk(
    config: &Config,
    tmpfolder: &Path,
    filenames: &[String],
) -> Result<(), Box<dyn Error>> {
    let low_file = tmpfolder.join("low.png");
    if low_file.exists() {
        fs::remove_file(low_file).unwrap();
    }

    let high_file = tmpfolder.join("high.png");
    if high_file.exists() {
        fs::remove_file(high_file).unwrap();
    }

    for zip_name in filenames.iter() {
        info!("Opening zip file {}", zip_name);
        let file = fs::File::open(zip_name).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        info!(
            "Extracting {:?} MB from {zip_name}",
            archive.decompressed_size().map(|s| s / 1024 / 1024)
        );
        archive.extract(tmpfolder).unwrap();
        render::mtkshaperender(config, tmpfolder).unwrap();
    }
    Ok(())
}

pub fn process_tile(
    config: &Config,
    thread: &String,
    tmpfolder: &Path,
    input_file: &Path,
    skip_rendering: bool,
) -> Result<(), Box<dyn Error>> {
    let mut timing = Timing::start_now("process_tile");
    fs::create_dir_all(tmpfolder).expect("Could not create tmp folder");

    let &Config {
        pnorthlinesangle,
        pnorthlineswidth,
        skipknolldetection,
        ..
    } = config;

    let mut thread_name = String::new();
    if !thread.is_empty() {
        thread_name = format!("Thread {}: ", thread);
    }

    timing.start_section("preparing input file");
    info!("{}Preparing input file", thread_name);

    let filename = input_file
        .file_name()
        .ok_or_else(|| format!("No extension for input file {}", input_file.display()))?
        .to_string_lossy()
        .to_lowercase();

    let target_file = tmpfolder.join("xyztemp.xyz.bin");

    if filename.ends_with(".xyz") {
        // if we are here we don't know if the file has at least 6 columns, but we assume that it is in the format
        // x y z classification number_of_returns return_number

        info!(
            "{}Converting points from .xyz to internal binary format",
            thread_name,
        );

        let mut writer = XyzInternalWriter::create(&target_file, crate::io::xyz::Format::XyzMeta)
            .expect("Could not create writer");
        read_lines_no_alloc(input_file, |line| {
            let mut parts = line.split(' ');
            let x = parts.next().unwrap().parse::<f64>().unwrap();
            let y = parts.next().unwrap().parse::<f64>().unwrap();
            let z = parts.next().unwrap().parse::<f64>().unwrap();

            let classification = parts.next().unwrap().parse::<u8>().unwrap();
            let number_of_returns = parts.next().unwrap().parse::<u8>().unwrap();
            let return_number = parts.next().unwrap().parse::<u8>().unwrap();

            writer
                .write_record(&crate::io::xyz::XyzRecord {
                    x,
                    y,
                    z,
                    meta: Some(XyzRecordMeta {
                        classification,
                        number_of_returns,
                        return_number,
                    }),
                })
                .expect("Could not write record");
        })
        .expect("Could not read file");
        writer.finish().expect("Unable to finish writing");
    } else if filename.ends_with(".laz") || filename.ends_with(".las") {
        info!(
            "{}Converting points from .laz/laz to internal binary format",
            thread_name
        );
        let &Config {
            thinfactor,
            xfactor,
            yfactor,
            zfactor,
            zoff,
            ..
        } = config;

        if thinfactor != 1.0 {
            info!("{}Using thinning factor {}", thread_name, thinfactor);
        }

        let mut rng = rand::thread_rng();
        let randdist = distributions::Bernoulli::new(thinfactor).unwrap();

        let mut reader = Reader::from_path(input_file).expect("Unable to open reader");

        let mut writer =
            XyzInternalWriter::create(&target_file, crate::io::xyz::Format::XyzMeta).unwrap();

        for ptu in reader.points() {
            let pt = ptu.unwrap();
            if thinfactor == 1.0 || rng.sample(randdist) {
                writer.write_record(&crate::io::xyz::XyzRecord {
                    x: pt.x * xfactor,
                    y: pt.y * yfactor,
                    z: pt.z * zfactor + zoff,
                    meta: Some(XyzRecordMeta {
                        classification: u8::from(pt.classification),
                        number_of_returns: pt.number_of_returns,
                        return_number: pt.return_number,
                    }),
                })?;
            }
        }
        writer.finish().expect("Unable to finish writing");
    } else if filename.ends_with(".xyz.bin") {
        info!("{}Copying input file", thread_name);
        fs::copy(input_file, target_file).expect("Could not copy file");
    } else {
        return Err(format!("Unsupported input file: {}", input_file.display()).into());
    }

    info!("{}Done", thread_name);

    info!("{}Knoll detection part 1", thread_name);
    timing.start_section("knoll detection part 1");

    let &Config {
        scalefactor,
        vegeonly,
        cliffsonly,
        contoursonly,
        ..
    } = config;

    if vegeonly || cliffsonly {
        contours::xyz2contours(
            config,
            tmpfolder,
            scalefactor * 0.3,
            "xyztemp.xyz.bin", //point cloud in
            "xyz_03.xyz.bin",  // heightmap out
            "null",            // no dxf curves
            true,              // only 2 or water_class points
        )
        .expect("contour generation failed");
    } else {
        contours::xyz2contours(
            config,
            tmpfolder,
            scalefactor * 0.3,
            "xyztemp.xyz.bin", // point cloud in
            "xyz_03.xyz.bin",  // heightmap out
            "contours03.dxf",  // dxf curves generated from the heightmap
            true,              // only 2 or water_class points
        )
        .expect("contour generation failed");
    }

    // copy the generated heightmap
    fs::copy(
        tmpfolder.join("xyz_03.xyz.bin"),
        tmpfolder.join("xyz2.xyz.bin"),
    )
    .expect("Could not copy file");

    fs::copy(
        tmpfolder.join("xyz_03.xyz.bin.hmap"),
        tmpfolder.join("xyz2.xyz.bin.hmap"),
    )
    .expect("Could not copy file");

    let &Config {
        contour_interval,
        basemapcontours,
        ..
    } = config;
    let halfinterval = contour_interval / 2.0 * scalefactor;

    if !vegeonly && !cliffsonly {
        if basemapcontours != 0.0 {
            info!("{}Basemap contours", thread_name);
            contours::xyz2contours(
                config,
                tmpfolder,
                basemapcontours,
                "xyz2.xyz.bin", // heightmap in
                "",             // no heightmap out
                "basemap.dxf",  // generate dxf contours
                false,          // include all points
            )
            .expect("contour generation failed");
        }
        if !skipknolldetection {
            info!("{}Knoll detection part 2", thread_name);
            timing.start_section("knoll detection part 2");
            knolls::knolldetector(config, tmpfolder).unwrap();
        }
        info!("{}Contour generation part 1", thread_name);
        timing.start_section("contour generation part 1");
        knolls::xyzknolls(config, tmpfolder).unwrap(); // modifies the heightmap (but does not change dimensions

        info!("{}Contour generation part 2", thread_name);
        timing.start_section("contour generation part 2");
        if !skipknolldetection {
            // contours 2.5
            contours::xyz2contours(
                config,
                tmpfolder,
                halfinterval,
                "xyz_knolls.xyz.bin", // heightmap in
                "null",               // no heightmap out
                "out.dxf",            // generates dxf curves
                false,                // includes all points
            )
            .unwrap();
        } else {
            contours::xyz2contours(
                config,
                tmpfolder,
                halfinterval,
                "xyztemp.xyz.bin", // point cloud in
                "null",            // do not save the heightmap
                "out.dxf",         // generate dxf curves
                true,              // include only ground classified points or water
            )
            .unwrap();
        }
        info!("{}Contour generation part 3", thread_name);
        timing.start_section("contour generation part 3");
        merge::smoothjoin(config, tmpfolder).unwrap();

        info!("{}Contour generation part 4", thread_name);
        timing.start_section("contour generation part 4");
        knolls::dotknolls(config, tmpfolder).unwrap();
    }

    if !cliffsonly && !contoursonly {
        info!("{}Vegetation generation", thread_name);
        timing.start_section("vegetation generation");
        vegetation::makevege(config, tmpfolder).unwrap();
    }

    if !vegeonly && !contoursonly {
        info!("{}Cliff generation", thread_name);
        timing.start_section("cliff generation");
        cliffs::makecliffs(config, tmpfolder).unwrap();
    }
    if !vegeonly && !contoursonly && !cliffsonly && config.detectbuildings {
        info!("{}Detecting buildings", thread_name);
        timing.start_section("detecting buildings");
        blocks::blocks(tmpfolder).unwrap();
    }
    if !skip_rendering && !vegeonly && !contoursonly && !cliffsonly {
        info!("{}Rendering png map with depressions", thread_name);
        timing.start_section("rendering png map with depressions");
        render::render(
            config,
            thread,
            tmpfolder,
            pnorthlinesangle,
            pnorthlineswidth,
            false,
        )
        .unwrap();

        info!("{}Rendering png map without depressions", thread_name);
        timing.start_section("rendering png map without depressions");
        render::render(
            config,
            thread,
            tmpfolder,
            pnorthlinesangle,
            pnorthlineswidth,
            true,
        )
        .unwrap();
    } else if contoursonly {
        info!("{}Rendering formlines", thread_name);
        timing.start_section("rendering formlines");
        let mut img = RgbaImage::from_pixel(1, 1, Rgba([0, 0, 0, 0]));
        render::draw_curves(config, &mut img, tmpfolder, false, false).unwrap();
    } else {
        info!("{}Skipped rendering", thread_name);
    }
    info!("{}All done!", thread_name);
    Ok(())
}

pub fn batch_process(conf: &Config, thread: &String) {
    let &Config {
        vegeonly,
        cliffsonly,
        contoursonly,
        savetempfolders,
        savetempfiles,
        scalefactor,
        vege_bitmode,
        zoff,
        thinfactor,
        ..
    } = conf;

    let Config {
        lazfolder,
        batchoutfolder,
        ..
    } = conf;

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
            info!("Skipping {}.png it exists already in output folder.", laz);
            continue;
        }

        info!("{}{} -> {}.png", thread_name, laz, laz);
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

        let tmp_filename = PathBuf::from(format!("temp{}.xyz.bin", thread));
        let mut writer = XyzInternalWriter::create(&tmp_filename, crate::io::xyz::Format::XyzMeta)
            .expect("Could not create writer");

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
                        writer
                            .write_record(&crate::io::xyz::XyzRecord {
                                x: pt.x,
                                y: pt.y,
                                z: pt.z + zoff,
                                meta: Some(XyzRecordMeta {
                                    classification: u8::from(pt.classification),
                                    number_of_returns: pt.number_of_returns,
                                    return_number: pt.return_number,
                                }),
                            })
                            .expect("Could not write record");
                    }
                }
            }
        }
        writer.finish().expect("Unable to finish writing");

        let tmpfolder = PathBuf::from(format!("temp{}", thread));
        if zip_files.is_empty() {
            process_tile(conf, thread, &tmpfolder, &tmp_filename, false).unwrap();
        } else {
            process_tile(conf, thread, &tmpfolder, &tmp_filename, true).unwrap();
            if !vegeonly && !cliffsonly && !contoursonly {
                process_zip(conf, thread, &tmpfolder, &zip_files).unwrap();
            }
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
            if !contoursonly && !cliffsonly {
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
                    image::ImageReader::open(Path::new(&format!("temp{}/undergrowth.png", thread)))
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
                    image::ImageReader::open(Path::new(&format!("temp{}/vegetation.png", thread)))
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

                let pgw_file_out = File::create(format!("{}/{}_vege.pgw", batchoutfolder, laz))
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
                    let mut orig_img_reader = image::ImageReader::open(Path::new(&format!(
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
                    image::imageops::overlay(
                        &mut img,
                        &orig_img.to_luma8(),
                        -dx as i64,
                        -dy as i64,
                    );
                    img.save(Path::new(&format!(
                        "{}/{}_vege_bit.png",
                        batchoutfolder, laz
                    )))
                    .expect("could not save output png");

                    let mut orig_img_reader = image::ImageReader::open(Path::new(&format!(
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
                    image::imageops::overlay(
                        &mut img,
                        &orig_img.to_luma8(),
                        -dx as i64,
                        -dy as i64,
                    );
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
