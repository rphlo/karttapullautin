use image::{Rgb, RgbImage};
use log::info;
use rustc_hash::FxHashMap as HashMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::io::bytes::FromToBytes;
use crate::io::heightmap::HeightMap;
use crate::vec2d::Vec2D;

fn merge_png(
    config: &Config,
    png_files: Vec<PathBuf>,
    outfilename: &str,
    scale: f64,
) -> Result<(), Box<dyn Error>> {
    let batchoutfolder = &config.batchoutfolder;

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

pub fn pngmergevege(config: &Config, scale: f64) -> Result<(), Box<dyn Error>> {
    let batchoutfolder = &config.batchoutfolder;

    let mut png_files: Vec<PathBuf> = Vec::new();
    for element in Path::new(batchoutfolder).read_dir().unwrap() {
        let path = element.unwrap().path();
        let filename = &path.as_path().file_name().unwrap().to_str().unwrap();
        if filename.ends_with("_vege.png") {
            png_files.push(path);
        }
    }
    if png_files.is_empty() {
        info!("No _vege.png files found in output directory");
        return Ok(());
    }
    merge_png(config, png_files, "merged_vege", scale).unwrap();
    Ok(())
}

pub fn pngmerge(config: &Config, scale: f64, depr: bool) -> Result<(), Box<dyn Error>> {
    let batchoutfolder = &config.batchoutfolder;

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
        info!("No files to merge found in output directory");
        return Ok(());
    }
    let mut outfilename = "merged";
    if depr {
        outfilename = "merged_depr";
    }
    merge_png(config, png_files, outfilename, scale).unwrap();
    Ok(())
}

pub fn dxfmerge(config: &Config) -> Result<(), Box<dyn Error>> {
    let batchoutfolder = &config.batchoutfolder;

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
        info!("No dxf files found in output directory");
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

    let basemapcontours: f64 = config.basemapcontours;

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

pub fn smoothjoin(config: &Config, tmpfolder: &Path) -> Result<(), Box<dyn Error>> {
    info!("Smooth curves...");

    let &Config {
        scalefactor,
        inidotknolls,
        smoothing,
        curviness,
        mut indexcontours,
        formline,
        depression_length,
        contour_interval,
        ..
    } = config;

    let halfinterval = contour_interval / 2.0 * scalefactor;
    if formline > 0.0 {
        indexcontours = 5.0 * contour_interval;
    }

    let interval = halfinterval;

    let heightmap_in = tmpfolder.join("xyz_knolls.xyz.bin.hmap");
    let mut reader = BufReader::new(File::open(heightmap_in)?);
    let hmap = HeightMap::from_bytes(&mut reader)?;

    // in world coordinates
    let xstart = hmap.xoffset;
    let ystart = hmap.yoffset;
    let size = hmap.scale;
    let xmax = (hmap.grid.width() - 1) as u64;
    let ymax = (hmap.grid.height() - 1) as u64;

    // Temporarily convert to HashMap for not having to go through all the logic below.
    let mut xyz: HashMap<(u64, u64), f64> = HashMap::default();
    for (x, y, h) in hmap.grid.iter_idx() {
        xyz.insert((x as u64, y as u64), h);
    }

    let mut steepness = Vec2D::new((xmax + 1) as usize, (ymax + 1) as usize, f64::NAN);

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
            steepness[(i as usize, j as usize)] = high - low;
        }
    }
    let input = tmpfolder.join("out.dxf");
    let data = fs::read_to_string(input).expect("Can not read input file");
    let data: Vec<&str> = data.split("POLYLINE").collect();
    let mut dxfheadtmp = data[0];
    dxfheadtmp = dxfheadtmp.split("ENDSEC").collect::<Vec<&str>>()[0];
    dxfheadtmp = dxfheadtmp.split("HEADER").collect::<Vec<&str>>()[1];
    let dxfhead = &format!("HEADER{}ENDSEC", dxfheadtmp);

    let output = tmpfolder.join("out2.dxf");
    let fp = File::create(output).expect("Unable to create file");
    let mut fp = BufWriter::new(fp);

    fp.write_all(b"  0\r\nSECTION\r\n  2\r\n")
        .expect("Could not write file");
    fp.write_all(dxfhead.as_bytes())
        .expect("Could not write file");
    fp.write_all(b"\r\n  0\r\nSECTION\r\n  2\r\nENTITIES\r\n  0\r\n")
        .expect("Could not write file");

    let depr_output = tmpfolder.join("depressions.txt");
    let depr_fp = File::create(depr_output).expect("Unable to create file");
    let mut depr_fp = BufWriter::new(depr_fp);

    let dotknoll_output = tmpfolder.join("dotknolls.txt");
    let dotknoll_fp = File::create(dotknoll_output).expect("Unable to create file");
    let mut dotknoll_fp = BufWriter::new(dotknoll_fp);

    let knollhead_output = tmpfolder.join("knollheads.txt");
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
                        let ss = steepness[(xx, yy)];
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
                        && el_x_len < 41
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
                        let ss = steepness[(xx, yy)];
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

                #[allow(clippy::manual_memcpy)]
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
                write!(
                    fp,
                    "POLYLINE\r\n 66\r\n1\r\n  8\r\n{}\r\n 38\r\n{}\r\n  0\r\n",
                    layer, h
                )
                .expect("Unable to write file");

                for k in 0..el_x_len {
                    write!(
                        fp,
                        "VERTEX\r\n  8\r\n{}\r\n 10\r\n{}\r\n 20\r\n{}\r\n 30\r\n{}\r\n  0\r\n",
                        layer, el_x[l][k], el_y[l][k], h
                    )
                    .expect("Unable to write file");
                }
                fp.write_all(b"SEQEND\r\n  0\r\n")
                    .expect("Unable to write file");
            } // -- if not dotkoll
        }
    }
    fp.write_all(b"ENDSEC\r\n  0\r\nEOF\r\n")
        .expect("Unable to write file");
    info!("Done");
    Ok(())
}
