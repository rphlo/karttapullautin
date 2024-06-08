use std::{
    error::Error,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use image::{Rgb, RgbImage};
use ini::Ini;

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

pub fn pngmergevege(scale: f64) -> Result<(), Box<dyn Error>> {
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

pub fn pngmerge(scale: f64, depr: bool) -> Result<(), Box<dyn Error>> {
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

pub fn dxfmerge() -> Result<(), Box<dyn Error>> {
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
