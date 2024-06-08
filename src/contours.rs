use ini::Ini;
use rustc_hash::FxHashMap as HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::util::read_lines_no_alloc;

pub fn xyz2contours(
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
