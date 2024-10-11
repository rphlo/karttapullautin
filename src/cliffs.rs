use image::{Rgb, RgbImage};
use log::info;
use rand::distributions;
use rand::prelude::*;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::config::Config;
use crate::util::read_lines;
use crate::util::read_lines_no_alloc;
use crate::vec2d::Vec2D;

pub fn makecliffs(config: &Config, tmpfolder: &Path) -> Result<(), Box<dyn Error>> {
    info!("Identifying cliffs...");

    let &Config {
        c1_limit,
        c2_limit,
        cliff_thin,
        steep_factor,
        flat_place,
        mut no_small_ciffs,
        ..
    } = config;

    if no_small_ciffs == 0.0 {
        no_small_ciffs = 6.0;
    } else {
        no_small_ciffs -= flat_place;
    }

    let mut xmin: f64 = f64::MAX;
    let mut xmax: f64 = f64::MIN;

    let mut ymin: f64 = f64::MAX;
    let mut ymax: f64 = f64::MIN;

    let mut hmin: f64 = f64::MAX;
    let mut hmax: f64 = f64::MIN;

    let xyz_file_in = tmpfolder.join("xyztemp.xyz");

    read_lines_no_alloc(xyz_file_in, |line| {
        let mut parts = line.split(' ');
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
    })
    .expect("Could not read input file");

    let xyz_file_in = tmpfolder.join("xyz2.xyz");
    let mut size: f64 = f64::NAN;
    let mut xstart: f64 = f64::NAN;
    let mut ystart: f64 = f64::NAN;
    let mut sxmax: usize = usize::MIN;
    let mut symax: usize = usize::MIN;
    if let Ok(lines) = read_lines(&xyz_file_in) {
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

    let mut xyz = Vec2D::new(
        ((xmax - xstart) / size).ceil() as usize + 1,
        ((ymax - ystart) / size).ceil() as usize + 1,
        f64::NAN,
    );

    read_lines_no_alloc(xyz_file_in, |line| {
        let mut parts = line.split(' ');
        let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
        let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
        let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();

        let xx = ((x - xstart) / size).floor() as usize;
        let yy = ((y - ystart) / size).floor() as usize;

        xyz[(xx, yy)] = h;

        if sxmax < xx {
            sxmax = xx;
        }
        if symax < yy {
            symax = yy;
        }
    })
    .expect("Could not read input file");

    let mut steepness = Vec2D::new(sxmax + 1, symax + 1, f64::NAN);

    for i in 3..sxmax - 4 {
        for j in 3..symax - 4 {
            let mut low: f64 = f64::MAX;
            let mut high: f64 = f64::MIN;
            for ii in i - 3..i + 4 {
                for jj in j - 3..j + 4 {
                    let value = xyz[(ii, jj)];

                    if value < low {
                        low = value;
                    }
                    if value > high {
                        high = value;
                    }
                }
            }
            steepness[(i, j)] = high - low;
        }
    }

    let mut img = RgbImage::from_pixel(
        (xmax - xmin).floor() as u32,
        (ymax - ymin).floor() as u32,
        Rgb([255, 255, 255]),
    );

    xmin = (xmin / 3.0).floor() * 3.0;
    ymin = (ymin / 3.0).floor() * 3.0;

    let mut list_alt = Vec2D::new(
        (((xmax - xmin) / 3.0).ceil() + 1.0) as usize,
        (((ymax - ymin) / 3.0).ceil() + 1.0) as usize,
        Vec::<(f64, f64, f64)>::new(),
    );

    let xyz_file_in = tmpfolder.join("xyztemp.xyz");

    let mut rng = rand::thread_rng();
    let randdist = distributions::Bernoulli::new(cliff_thin).unwrap();

    read_lines_no_alloc(&xyz_file_in, |line| {
        if cliff_thin == 1.0 || rng.sample(randdist) {
            let mut parts = line.split(' ');
            let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let r3 = parts.next().unwrap();

            if r3 == "2" {
                list_alt[(
                    ((x - xmin).floor() / 3.0) as usize,
                    ((y - ymin).floor() / 3.0) as usize,
                )]
                    .push((x, y, h));
            }
        }
    })
    .expect("Could not read input file");

    let w = ((xmax - xmin).floor() / 3.0) as usize;
    let h = ((ymax - ymin).floor() / 3.0) as usize;

    let f2 = File::create(tmpfolder.join("c2g.dxf")).expect("Unable to create file");
    let mut f2 = BufWriter::new(f2);

    write!(&mut f2,"  0\r\nSECTION\r\n  2\r\nHEADER\r\n  9\r\n$EXTMIN\r\n 10\r\n{}\r\n 20\r\n{}\r\n  9\r\n$EXTMAX\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nENDSEC\r\n  0\r\nSECTION\r\n  2\r\nENTITIES\r\n  0\r\n", xmin, ymin, xmax, ymax).expect("Cannot write dxf file");

    let f3 = File::create(tmpfolder.join("c3g.dxf")).expect("Unable to create file");
    let mut f3 = BufWriter::new(f3);

    write!(&mut f3, "  0\r\nSECTION\r\n  2\r\nHEADER\r\n  9\r\n$EXTMIN\r\n 10\r\n{}\r\n 20\r\n{}\r\n  9\r\n$EXTMAX\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\nENDSEC\r\n  0\r\nSECTION\r\n  2\r\nENTITIES\r\n  0\r\n",
            xmin, ymin, xmax, ymax
    ).expect("Cannot write dxf file");

    for x in 0..w + 1 {
        for y in 0..h + 1 {
            if !list_alt[(x, y)].is_empty() {
                let mut t = Vec::<(f64, f64, f64)>::new();
                if x >= 1 {
                    if y >= 1 {
                        t.extend(&list_alt[(x - 1, y - 1)]);
                    }
                    t.extend(&list_alt[(x - 1, y)]);
                    if y < h {
                        t.extend(&list_alt[(x - 1, y + 1)]);
                    }
                }
                if y >= 1 {
                    t.extend(&list_alt[(x, y - 1)]);
                }
                t.extend(&list_alt[(x, y)]);
                if y < h {
                    t.extend(&list_alt[(x, y + 1)]);
                }
                if x < w {
                    if y >= 1 {
                        t.extend(&list_alt[(x + 1, y - 1)]);
                    }
                    t.extend(&list_alt[(x + 1, y)]);
                    if y < h {
                        t.extend(&list_alt[(x + 1, y + 1)]);
                    }
                }
                let mut d = Vec::<(f64, f64, f64)>::new();
                d.extend(&list_alt[(x, y)]);

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
                    if temp_min > h0 {
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
                    let mut steep = steepness[(
                        ((x0 - xstart) / size + 0.5).floor() as usize,
                        ((y0 - ystart) / size + 0.5).floor() as usize,
                    )] - flat_place;
                    if steep.is_nan() {
                        steep = -flat_place;
                    }

                    steep = steep.clamp(0.0, 17.0);

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

    let mut list_alt = Vec2D::new(
        (((xmax - xmin) / 3.0).ceil() + 1.0) as usize,
        (((ymax - ymin) / 3.0).ceil() + 1.0) as usize,
        Vec::<(f64, f64, f64)>::new(),
    );

    let xyz_file_in = tmpfolder.join("xyz2.xyz");
    read_lines_no_alloc(&xyz_file_in, |line| {
        if cliff_thin == 1.0 || rng.sample(randdist) {
            let mut parts = line.split(' ');
            let x: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let y: f64 = parts.next().unwrap().parse::<f64>().unwrap();
            let h: f64 = parts.next().unwrap().parse::<f64>().unwrap();

            list_alt[(
                ((x - xmin).floor() / 3.0) as usize,
                ((y - ymin).floor() / 3.0) as usize,
            )]
                .push((x, y, h));
        }
    })
    .expect("Could not read input file");

    for x in 0..w + 1 {
        for y in 0..h + 1 {
            if !list_alt[(x, y)].is_empty() {
                let mut t = Vec::<(f64, f64, f64)>::new();
                if x >= 1 {
                    if y >= 1 {
                        t.extend(&list_alt[(x - 1, y - 1)]);
                    }
                    t.extend(&list_alt[(x - 1, y)]);
                    if y < h {
                        t.extend(&list_alt[(x - 1, y + 1)]);
                    }
                }
                if y >= 1 {
                    t.extend(&list_alt[(x, y - 1)]);
                }
                t.extend(&list_alt[(x, y)]);
                if y < h {
                    t.extend(&list_alt[(x, y + 1)]);
                }
                if x < w {
                    if y >= 1 {
                        t.extend(&list_alt[(x + 1, y - 1)]);
                    }
                    t.extend(&list_alt[(x + 1, y)]);
                    if y < h {
                        t.extend(&list_alt[(x + 1, y + 1)]);
                    }
                }
                let mut d = Vec::<(f64, f64, f64)>::new();
                d.extend(&list_alt[(x, y)]);

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
    img.save(tmpfolder.join("c2.png"))
        .expect("could not save output png");
    info!("Done");
    Ok(())
}
