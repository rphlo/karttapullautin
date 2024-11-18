use crate::canvas::Canvas;
use crate::config::Config;
use crate::io::bytes::FromToBytes;
use crate::io::fs::FileSystem;
use crate::io::heightmap::HeightMap;
use image::ImageBuffer;
use image::Rgba;
use imageproc::drawing::{draw_filled_circle_mut, draw_line_segment_mut};
use log::info;
use rustc_hash::FxHashMap as HashMap;
use shapefile::dbase::{FieldValue, Record};
use shapefile::{Shape, ShapeType};
use std::error::Error;
use std::f64::consts::PI;
use std::io::BufRead;
use std::io::BufReader;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, PartialEq)]
enum Operator {
    Equal,
    NotEqual,
}

/// A condition that is used to filter the records in the vectorconf file
#[derive(Debug, PartialEq)]
struct Condition {
    operator: Operator,
    key: String,
    value: String,
}

/// Each mapping represents one line in the vectorconf file
#[derive(Debug, PartialEq)]
struct Mapping {
    /// The ISOM code that this mapping maps the shape to
    isom: String,
    /// The conditions that must be met for this mapping to be applied
    conditions: Vec<Condition>,
}

impl FromStr for Mapping {
    type Err = String;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let row_data: Vec<&str> = line.trim().split('|').collect();
        if row_data.len() != 3 {
            return Err(format!(
                "vectorconf line does not contain 3 sections separated by '|', it has {}: {}",
                row_data.len(),
                line
            ));
        }
        let isom = row_data[1].to_string();
        if isom.is_empty() {
            return Err(format!("ISOM code most not be empty: {line}"));
        }

        let conditions: Vec<Condition> = row_data[2]
            .split('&')
            .map(|param| {
                let (operator, d): (Operator, Vec<&str>) = if param.contains("!=") {
                    (Operator::NotEqual, param.splitn(2, "!=").collect())
                } else if param.contains("=") {
                    (Operator::Equal, param.splitn(2, "=").collect())
                } else {
                    return Err(format!(
                        "Condition does not contain a valid operator: {}",
                        param
                    ));
                };
                Ok(Condition {
                    operator,
                    key: d[0].trim().to_string(),
                    value: d[1].trim().to_string(),
                })
            })
            .collect::<Result<Vec<_>, Self::Err>>()?;

        Ok(Self { isom, conditions })
    }
}

pub fn mtkshaperender(
    fs: &impl FileSystem,
    config: &Config,
    tmpfolder: &Path,
) -> Result<(), Box<dyn Error>> {
    let scalefactor = config.scalefactor;

    let vectorconf = &config.vectorconf;
    let mtkskip = &config.mtkskiplayers;

    let mut vectorconf_mappings: Vec<Mapping> = vec![];
    if !vectorconf.is_empty() {
        let vectorconf_lines = fs
            .read_to_string(vectorconf)
            .expect("Can not read input file");

        // parse all the lines in the vectorconf file into a list of mappings
        vectorconf_mappings = vectorconf_lines
            .lines()
            .map(Mapping::from_str)
            .collect::<Result<Vec<_>, _>>()?;
    }
    if !tmpfolder.join("vegetation.pgw").exists() {
        info!("Could not find vegetation file");
        return Ok(());
    }

    let input = tmpfolder.join("vegetation.pgw");
    let data = fs.read_to_string(input).expect("Can not read input file");
    let d: Vec<&str> = data.split('\n').collect();

    let x0 = d[4].trim().parse::<f64>().unwrap();
    let y0 = d[5].trim().parse::<f64>().unwrap();
    // let resvege = d[0].trim().parse::<f64>().unwrap();

    let mut img_reader = image::ImageReader::open(tmpfolder.join("vegetation.png"))
        .expect("Opening vegetation image failed");
    img_reader.no_limits();
    let img = img_reader.decode().unwrap();
    let w = img.width() as f64;
    let h = img.height() as f64;

    let outw = w * 600.0 / 254.0 / scalefactor;
    let outh = h * 600.0 / 254.0 / scalefactor;

    // TODO: only allocate the canvas that are actually used... in a lazy way
    let mut imgbrown = Canvas::new(outw as i32, outh as i32);
    let mut imgbrowntop = Canvas::new(outw as i32, outh as i32);
    let mut imgblack = Canvas::new(outw as i32, outh as i32);
    let mut imgblacktop = Canvas::new(outw as i32, outh as i32);
    let mut imgyellow = Canvas::new(outw as i32, outh as i32);
    let mut imgblue = Canvas::new(outw as i32, outh as i32);
    let mut imgmarsh = Canvas::new(outw as i32, outh as i32);
    let mut imgtempblack = Canvas::new(outw as i32, outh as i32);
    let mut imgtempblacktop = Canvas::new(outw as i32, outh as i32);
    let mut imgblue2 = Canvas::new(outw as i32, outh as i32);

    let white = (255, 255, 255);
    let black = (0, 0, 0);
    let brown = (255, 150, 80);

    let purple = config.buildingcolor;
    let yellow = (255, 184, 83);
    let blue = (29, 190, 255);
    let marsh = (0, 10, 220);
    let olive = (194, 176, 33);

    let mut shp_files: Vec<PathBuf> = Vec::new();
    for element in tmpfolder.read_dir().unwrap() {
        let path = element.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "shp" {
                shp_files.push(path);
            }
        }
    }

    info!("Processing shapefiles: {:?}", shp_files);

    for shp_file in shp_files.iter() {
        let file = shp_file.as_path().file_name().unwrap().to_str().unwrap();
        let mut file = tmpfolder.join(file);

        info!("Processing shapefile: {:?}", file);
        // drawshape comes here
        let mut reader = shapefile::Reader::from_path(&file)?;
        for shape_record in reader.iter_shapes_and_records() {
            let (shape, record) = shape_record
                .unwrap_or_else(|_err: shapefile::Error| (Shape::NullShape, Record::default()));

            let mut area = false;
            let mut roadedge = 0.0;
            let mut edgeimage = "black";
            let mut image = "";
            let mut thickness = 1.0;
            let mut color: Option<(u8, u8, u8)> = None;
            let mut dashedline = false;
            let mut border = 0.0;

            if vectorconf.is_empty() {
                // MML shape file
                let mut luokka = String::new();
                if let Some(fv) = record.get("LUOKKA") {
                    if let FieldValue::Numeric(Some(f_luokka)) = fv {
                        luokka = format!("{}", f_luokka);
                    }
                    if let FieldValue::Character(Some(c_luokka)) = fv {
                        luokka = c_luokka.to_string();
                    }
                }
                let mut versuh = 0.0;
                if let Some(FieldValue::Numeric(Some(f_versuh))) = record.get("VERSUH") {
                    versuh = *f_versuh;
                }
                // water streams
                if ["36311", "36312"].contains(&luokka.as_str()) {
                    thickness = 4.0;
                    color = Some(marsh);
                    image = "blue";
                }

                // pathes
                if luokka == "12316" && versuh != -11.0 {
                    thickness = 12.0;
                    dashedline = true;
                    image = "black";
                    color = Some(black);
                    if versuh > 0.0 {
                        image = "blacktop";
                    }
                }

                // large pathes
                if (luokka == "12141" || luokka == "12314") && versuh != 11.0 {
                    thickness = 12.0;
                    image = "black";
                    color = Some(black);
                    if versuh > 0.0 {
                        image = "blacktop";
                    }
                }

                // roads
                if ["12111", "12112", "12121", "12122", "12131", "12132"].contains(&luokka.as_str())
                    && versuh != 11.0
                {
                    imgbrown.set_line_width(20.0);
                    imgbrowntop.set_line_width(20.0);
                    thickness = 20.0;
                    color = Some(brown);
                    image = "brown";
                    roadedge = 26.0;
                    imgblack.set_line_width(26.0);
                    if versuh > 0.0 {
                        edgeimage = "blacktop";
                        imgbrown.set_line_width(14.0);
                        imgbrowntop.set_line_width(14.0);
                        thickness = 14.0;
                    }
                }

                // railroads
                if ["14110", "14111", "14112", "14121", "14131"].contains(&luokka.as_str())
                    && versuh != 11.0
                {
                    image = "black";
                    color = Some(white);
                    thickness = 3.0;
                    roadedge = 18.0;
                    if versuh > 0.0 {
                        image = "blacktop";
                        edgeimage = "blacktop";
                    }
                }

                if luokka == "12312" && versuh != 11.0 {
                    dashedline = true;
                    thickness = 6.0;
                    image = "black";
                    color = Some(black);
                    if versuh > 0.0 {
                        image = "blacktop";
                    }
                }

                if luokka == "12313" && versuh != 11.0 {
                    dashedline = true;
                    thickness = 5.0;
                    image = "black";
                    color = Some(black);
                    if versuh > 0.0 {
                        image = "blacktop";
                    }
                }

                // power line
                if ["22300", "22312", "44500", "223311"].contains(&luokka.as_str()) {
                    imgblacktop.set_line_width(5.0);
                    thickness = 5.0;
                    color = Some(black);
                    image = "blacktop";
                }

                // fence
                if ["44211", "44213"].contains(&luokka.as_str()) {
                    imgblacktop.set_line_width(7.0);
                    thickness = 7.0;
                    color = Some(black);
                    image = "blacktop";
                }

                // Next are polygons

                // fields
                if luokka == "32611" {
                    area = true;
                    color = Some(yellow);
                    border = 3.0;
                    image = "yellow";
                }

                // lake
                if [
                    "36200", "36211", "36313", "38700", "44300", "45111", "54112",
                ]
                .contains(&luokka.as_str())
                {
                    area = true;
                    color = Some(blue);
                    border = 5.0;
                    image = "blue";
                }

                // impassable marsh
                if ["35421", "38300"].contains(&luokka.as_str()) {
                    area = true;
                    color = Some(marsh);
                    border = 3.0;
                    image = "marsh";
                }

                // regular marsh
                if ["35400", "35411"].contains(&luokka.as_str()) {
                    area = true;
                    color = Some(marsh);
                    border = 0.0;
                    image = "marsh";
                }

                // marshy
                if ["35300", "35412", "35422"].contains(&luokka.as_str()) {
                    area = true;
                    color = Some(marsh);
                    border = 0.0;
                    image = "marsh";
                }

                // marshy
                if [
                    "42210", "42211", "42212", "42220", "42221", "42222", "42230", "42231",
                    "42232", "42240", "42241", "42242", "42270", "42250", "42251", "42252",
                    "42260", "42261", "42262",
                ]
                .contains(&luokka.as_str())
                {
                    area = true;
                    color = Some(purple);
                    border = 0.0;
                    image = "black";
                }

                // settlement
                if [
                    "32000", "40200", "62100", "32410", "32411", "32412", "32413", "32414",
                    "32415", "32416", "32417", "32418",
                ]
                .contains(&luokka.as_str())
                {
                    area = true;
                    color = Some(olive);
                    border = 0.0;
                    image = "yellow";
                }

                // airport runway, car parkings
                if ["32411", "32412", "32415", "32417", "32421"].contains(&luokka.as_str()) {
                    area = true;
                    color = Some(brown);
                    border = 0.0;
                    image = "yellow";
                }

                if mtkskip.contains(&luokka) {
                    color = None;
                }
            } else {
                // configuration based drawing, iterate over all the rules and find the one that matches
                for mapping in vectorconf_mappings.iter() {
                    // if the color is already set we have a match, skip the rest of the mappings
                    if color.is_some() {
                        break;
                    }

                    // check if the record matches the conditions
                    let mut is_ok = true;
                    for keyval in &mapping.conditions {
                        let mut r = String::from("");
                        if let Some(FieldValue::Character(Some(record_str))) =
                            record.get(&keyval.key)
                        {
                            r = record_str.trim().to_string();
                        }
                        if keyval.operator == Operator::Equal {
                            if r != keyval.value {
                                is_ok = false;
                            }
                        } else if r == keyval.value {
                            is_ok = false;
                        }
                    }

                    // no match? continue to the next mapping
                    if !is_ok {
                        continue;
                    }

                    let isom = &mapping.isom;

                    if isom == "306" {
                        imgblue.set_line_width(5.0);
                        thickness = 4.0;
                        color = Some(marsh);
                        image = "blue";
                    }

                    // small path
                    if isom == "505" {
                        dashedline = true;
                        thickness = 12.0;
                        color = Some(black);
                        image = "black";
                    }

                    // small path top
                    if isom == "505T" {
                        dashedline = true;
                        thickness = 12.0;
                        color = Some(black);
                        image = "blacktop";
                    }

                    // large path
                    if isom == "504" {
                        imgblack.set_line_width(12.0);
                        thickness = 12.0;
                        color = Some(black);
                        image = "black";
                    }

                    // large path top
                    if isom == "504T" {
                        imgblack.set_line_width(12.0);
                        thickness = 12.0;
                        color = Some(black);
                        image = "blacktop";
                    }

                    // road
                    if isom == "503" {
                        imgbrown.set_line_width(20.0);
                        imgbrowntop.set_line_width(20.0);
                        color = Some(brown);
                        image = "brown";
                        roadedge = 26.0;
                        thickness = 20.0;
                        imgblack.set_line_width(26.0);
                    }

                    // road, bridges
                    if isom == "503T" {
                        edgeimage = "blacktop";
                        imgbrown.set_line_width(14.0);
                        imgbrowntop.set_line_width(14.0);
                        color = Some(brown);
                        image = "brown";
                        roadedge = 26.0;
                        thickness = 14.0;
                        imgblack.set_line_width(26.0);
                    }

                    // railroads
                    if isom == "515" {
                        color = Some(white);
                        image = "black";
                        roadedge = 18.0;
                        thickness = 3.0;
                    }

                    // railroads top
                    if isom == "515T" {
                        color = Some(white);
                        image = "blacktop";
                        edgeimage = "blacktop";
                        roadedge = 18.0;
                        thickness = 3.0;
                    }

                    // small path
                    if isom == "507" {
                        dashedline = true;
                        color = Some(black);
                        image = "black";
                        thickness = 6.0;
                        imgblack.set_line_width(6.0);
                    }

                    // small path top
                    if isom == "507T" {
                        dashedline = true;
                        color = Some(black);
                        image = "blacktop";
                        thickness = 6.0;
                        imgblack.set_line_width(6.0);
                    }

                    // powerline
                    if isom == "516" {
                        color = Some(black);
                        image = "blacktop";
                        thickness = 5.0;
                        imgblacktop.set_line_width(5.0);
                    }

                    // fence
                    if isom == "524" {
                        color = Some(black);
                        image = "black";
                        thickness = 7.0;
                        imgblacktop.set_line_width(7.0);
                    }

                    // blackline
                    if isom == "414" {
                        color = Some(black);
                        image = "black";
                        thickness = 4.0;
                    }

                    // areas

                    // fields
                    if isom == "401" {
                        area = true;
                        color = Some(yellow);
                        border = 3.0;
                        image = "yellow";
                    }
                    // lakes
                    if isom == "301" {
                        area = true;
                        color = Some(blue);
                        border = 5.0;
                        image = "blue";
                    }
                    // marshes
                    if isom == "310" {
                        area = true;
                        color = Some(marsh);
                        image = "marsh";
                    }
                    // buildings
                    if isom == "526" {
                        area = true;
                        color = Some(purple);
                        image = "black";
                    }
                    // settlements
                    if isom == "527" {
                        area = true;
                        color = Some(olive);
                        image = "yellow";
                    }
                    // car parkings border
                    if isom == "529.1" || isom == "301.1" {
                        thickness = 2.0;
                        color = Some(black);
                        image = "black";
                    }
                    // car park area
                    if isom == "529" {
                        area = true;
                        color = Some(brown);
                        image = "yellow";
                    }
                    // car park top
                    if isom == "529T" {
                        area = true;
                        color = Some(brown);
                        image = "brown";
                    }
                }
            }

            // if there was a match, do the drawing!
            if let Some(color) = color {
                if !area && shape.shapetype() == ShapeType::Polyline {
                    let mut poly: Vec<(f32, f32)> = vec![];
                    let polyline = shapefile::Polyline::try_from(shape).unwrap();
                    for points in polyline.parts().iter() {
                        for point in points.iter() {
                            let x = point.x;
                            let y = point.y;
                            poly.push((
                                (600.0 / 254.0 / scalefactor * (x - x0)).floor() as f32,
                                (600.0 / 254.0 / scalefactor * (y0 - y)).floor() as f32,
                            ));
                        }
                    }
                    if roadedge > 0.0 {
                        if edgeimage == "blacktop" {
                            imgblacktop.unset_stroke_cap();
                            imgblacktop.set_line_width(roadedge);
                            imgblacktop.set_color(black);
                            imgblacktop.draw_polyline(&poly);
                            imgblacktop.set_line_width(thickness);
                        } else {
                            imgblack.set_color(black);
                            imgblack.set_stroke_cap_round();
                            imgblack.set_line_width(roadedge);
                            imgblack.draw_polyline(&poly);
                            imgblack.set_line_width(thickness);
                            imgblack.unset_stroke_cap();
                        }
                    }

                    if !dashedline {
                        if image == "blacktop" {
                            imgblacktop.set_line_width(thickness);
                            imgblacktop.set_color(color);
                            if thickness >= 9.0 {
                                imgblacktop.set_stroke_cap_round();
                            }
                            imgblacktop.draw_polyline(&poly);
                            imgblacktop.unset_stroke_cap();
                        }
                        if image == "black" {
                            imgblack.set_line_width(thickness);
                            imgblack.set_color(color);
                            if thickness >= 9.0 {
                                imgblack.set_stroke_cap_round();
                            } else {
                                imgblack.unset_stroke_cap();
                            }
                            imgblack.draw_polyline(&poly);
                        }
                    } else {
                        if image == "blacktop" {
                            let interval_on = 1.0 + thickness * 8.0;
                            imgtempblacktop.set_dash(interval_on, thickness * 1.6);
                            if thickness >= 9.0 {
                                imgtempblacktop.set_stroke_cap_round();
                            }
                            imgtempblacktop.set_color(color);
                            imgtempblacktop.set_line_width(thickness);
                            imgtempblacktop.draw_polyline(&poly);
                            imgtempblacktop.unset_dash();
                            imgtempblacktop.unset_stroke_cap();
                        }
                        if image == "black" {
                            let interval_on = 1.0 + thickness * 8.0;
                            imgtempblack.set_dash(interval_on, thickness * 1.6);
                            if thickness >= 9.0 {
                                imgtempblack.set_stroke_cap_round();
                            }
                            imgtempblack.set_color(color);
                            imgtempblack.set_line_width(thickness);
                            imgtempblack.draw_polyline(&poly);
                            imgtempblack.unset_dash();
                            imgtempblack.unset_stroke_cap();
                        }
                    }

                    if image == "blue" {
                        imgblue.set_color(color);
                        imgblue.set_line_width(thickness);
                        imgblue.draw_polyline(&poly)
                    }

                    if image == "brown" {
                        if edgeimage == "blacktop" {
                            imgbrowntop.set_line_width(thickness);
                            imgbrowntop.set_color(brown);
                            imgbrowntop.draw_polyline(&poly);
                        } else {
                            imgbrown.set_stroke_cap_round();
                            imgbrown.set_line_width(thickness);
                            imgbrown.set_color(brown);
                            imgbrown.draw_polyline(&poly);
                            imgbrown.unset_stroke_cap();
                        }
                    }
                } else if area && shape.shapetype() == ShapeType::Polygon {
                    let mut polys: Vec<Vec<(f32, f32)>> = vec![];
                    let polygon = shapefile::Polygon::try_from(shape).unwrap();
                    for ring in polygon.rings().iter() {
                        let mut poly: Vec<(f32, f32)> = vec![];
                        let mut polyborder: Vec<(f32, f32)> = vec![];
                        for point in ring.points().iter() {
                            let x = point.x;
                            let y = point.y;
                            poly.push((
                                (600.0 / 254.0 / scalefactor * (x - x0)).floor() as f32,
                                (600.0 / 254.0 / scalefactor * (y0 - y)).floor() as f32,
                            ));
                            polyborder.push((
                                (600.0 / 254.0 / scalefactor * (x - x0)).floor() as f32,
                                (600.0 / 254.0 / scalefactor * (y0 - y)).floor() as f32,
                            ));
                        }
                        polys.push(poly);
                        if border > 0.0 {
                            imgblack.set_color(black);
                            imgblack.set_line_width(border);
                            imgblack.draw_closed_polyline(&polyborder);
                        }
                    }

                    if image == "black" {
                        imgblack.set_color(color);
                        imgblack.draw_filled_polygon(&polys)
                    }
                    if image == "blue" {
                        imgblue.set_color(color);
                        imgblue.draw_filled_polygon(&polys)
                    }
                    if image == "yellow" {
                        imgyellow.set_color(color);
                        imgyellow.draw_filled_polygon(&polys)
                    }
                    if image == "marsh" {
                        imgmarsh.set_color(color);
                        imgmarsh.draw_filled_polygon(&polys)
                    }
                    if image == "brown" {
                        imgbrown.set_color(color);
                        imgbrown.draw_filled_polygon(&polys)
                    }
                }
            }
        }

        // remove the shapefile and all associated files
        fs.remove_file(&file).unwrap();
        for ext in ["dbf", "sbx", "prj", "shx", "sbn", "cpg", "qmd"].iter() {
            file.set_extension(ext);
            if file.exists() {
                println!("Removing file: {:?}", file);
                fs.remove_file(&file).unwrap();
            }
        }
    }
    imgblue2.overlay(&mut imgblue, 0.0, 0.0);
    imgblue2.overlay(&mut imgblue, 1.0, 0.0);
    imgblue2.overlay(&mut imgblue, 0.0, 1.0);
    imgblue.overlay(&mut imgblue2, 0.0, 0.0);

    let mut i = 0.0_f32;
    imgmarsh.set_transparent_color();
    while i < ((h * 600.0 / 254.0 / scalefactor + 500.0) as f32) {
        i += 14.0;
        let wd = (w * 600.0 / 254.0 / scalefactor + 2.0) as f32;
        imgmarsh.draw_filled_polygon(&[vec![
            (-1.0, i),
            (wd, i),
            (wd, i + 10.0),
            (-1.0, i + 10.0),
            (-1.0, i),
        ]])
    }
    imgblacktop.overlay(&mut imgtempblacktop, 0.0, 0.0);
    imgblack.overlay(&mut imgtempblack, 0.0, 0.0);

    imgyellow.overlay(&mut imgmarsh, 0.0, 0.0);

    imgblue.overlay(&mut imgblack, 0.0, 0.0);
    imgblue.overlay(&mut imgbrown, 0.0, 0.0);
    imgblue.overlay(&mut imgblacktop, 0.0, 0.0);
    imgblue.overlay(&mut imgbrowntop, 0.0, 0.0);

    let low_file = tmpfolder.join("low.png");
    if low_file.exists() {
        let mut low = Canvas::load_from(&low_file);
        imgyellow.overlay(&mut low, 0.0, 0.0);
    }

    let high_file = tmpfolder.join("high.png");
    if high_file.exists() {
        let mut high = Canvas::load_from(&high_file);
        imgblue.overlay(&mut high, 0.0, 0.0);
    }
    imgblue.save_as(&high_file);
    imgyellow.save_as(&low_file);
    Ok(())
}

pub fn render(
    fs: &impl FileSystem,
    config: &Config,
    thread: &String,
    tmpfolder: &Path,
    angle_deg: f64,
    nwidth: usize,
    nodepressions: bool,
) -> Result<(), Box<dyn Error>> {
    info!("Rendering...");

    let scalefactor = config.scalefactor;

    let angle = -angle_deg / 180.0 * PI;

    // Draw vegetation ----------
    let tfw_in = tmpfolder.join("vegetation.pgw");
    let mut lines = BufReader::new(fs.open(tfw_in).expect("PGW file does not exist")).lines();
    let x0 = lines
        .nth(4)
        .expect("no 4 line")
        .expect("Could not read line 5")
        .parse::<f64>()
        .unwrap();
    let y0 = lines
        .next()
        .expect("no 5 line")
        .expect("Could not read line 6")
        .parse::<f64>()
        .unwrap();

    let mut img_reader = image::ImageReader::open(tmpfolder.join("vegetation.png"))
        .expect("Opening vegetation image failed");
    img_reader.no_limits();
    let img = img_reader.decode().unwrap();

    let mut imgug_reader = image::ImageReader::open(tmpfolder.join("undergrowth.png"))
        .expect("Opening undergrowth image failed");
    imgug_reader.no_limits();
    let imgug = imgug_reader.decode().unwrap();

    let w = img.width();
    let h = img.height();

    let eastoff = -((x0 - (-angle).tan() * y0)
        - ((x0 - (-angle).tan() * y0) / (250.0 / angle.cos())).floor() * (250.0 / angle.cos()))
        / 254.0
        * 600.0;

    let new_width = (w as f64 * 600.0 / 254.0 / scalefactor) as u32;
    let new_height = (h as f64 * 600.0 / 254.0 / scalefactor) as u32;
    let mut img = image::imageops::resize(
        &img,
        new_width,
        new_height,
        image::imageops::FilterType::Nearest,
    );

    let imgug = image::imageops::resize(
        &imgug,
        new_width,
        new_height,
        image::imageops::FilterType::Nearest,
    );

    image::imageops::overlay(&mut img, &imgug, 0, 0);

    let low_file = tmpfolder.join("low.png");
    if low_file.exists() {
        let mut low_reader = image::ImageReader::open(low_file).expect("Opening low image failed");
        low_reader.no_limits();
        let low = low_reader.decode().unwrap();
        let low = image::imageops::resize(
            &low,
            new_width,
            new_height,
            image::imageops::FilterType::Nearest,
        );
        image::imageops::overlay(&mut img, &low, 0, 0);
    }

    // north lines ----------------
    if angle != 999.0 {
        let mut i: f64 = eastoff - 600.0 * 250.0 / 254.0 / angle.cos() * 100.0 / scalefactor;
        while i < w as f64 * 5.0 * 600.0 / 254.0 / scalefactor {
            for m in 0..nwidth {
                draw_line_segment_mut(
                    &mut img,
                    (i as f32 + m as f32, 0.0),
                    (
                        (i as f32
                            + (angle.tan() * (h as f64) * 600.0 / 254.0 / scalefactor) as f32)
                            + m as f32,
                        (h as f32 * 600.0 / 254.0 / scalefactor as f32),
                    ),
                    Rgba([0, 0, 200, 255]),
                );
            }
            i += 600.0 * 250.0 / 254.0 / angle.cos() / scalefactor;
        }
    }

    draw_curves(fs, config, &mut img, tmpfolder, nodepressions, true).unwrap();

    // dotknolls----------
    let input = tmpfolder.join("dotknolls.dxf");
    let data = fs.read_to_string(input).expect("Can not read input file");
    let data = data.split("POINT");

    for (j, rec) in data.enumerate() {
        let mut x: f64 = 0.0;
        let mut y: f64 = 0.0;
        if j > 0 {
            let val = rec.split('\n').collect::<Vec<&str>>();
            let layer = val[2].trim();
            for (i, v) in val.iter().enumerate() {
                let vt = v.trim();
                if vt == "10" {
                    x = (val[i + 1].trim().parse::<f64>().unwrap() - x0) * 600.0
                        / 254.0
                        / scalefactor;
                }
                if vt == "20" {
                    y = (y0 - val[i + 1].trim().parse::<f64>().unwrap()) * 600.0
                        / 254.0
                        / scalefactor;
                }
            }
            if layer == "dotknoll" {
                let color = Rgba([166, 85, 43, 255]);

                draw_filled_circle_mut(&mut img, (x as i32, y as i32), 7, color)
            }
        }
    }
    // blocks -------------
    let blocks_file = tmpfolder.join("blocks.png");
    if blocks_file.exists() {
        let mut blockpurple_reader =
            image::ImageReader::open(blocks_file).expect("Opening blocks image failed");
        blockpurple_reader.no_limits();
        let blockpurple = blockpurple_reader.decode().unwrap();
        let mut blockpurple = blockpurple.to_rgba8();
        for p in blockpurple.pixels_mut() {
            if p[0] == 255 && p[1] == 255 && p[2] == 255 {
                p[3] = 0;
            }
        }
        let blockpurple = image::imageops::crop(&mut blockpurple, 0, 0, w, h).to_image();
        let blockpurple_thumb = image::imageops::resize(
            &blockpurple,
            new_width,
            new_height,
            image::imageops::FilterType::Nearest,
        );

        for i in 0..3 {
            for j in 0..3 {
                image::imageops::overlay(
                    &mut img,
                    &blockpurple_thumb,
                    (i as i64 - 1) * 2,
                    (j as i64 - 1) * 2,
                );
            }
        }
        image::imageops::overlay(&mut img, &blockpurple_thumb, 0, 0);
    }
    // blueblack -------------
    let blueblack_file = tmpfolder.join("blueblack.png");
    if blueblack_file.exists() {
        let mut imgbb_reader =
            image::ImageReader::open(blueblack_file).expect("Opening blueblack image failed");
        imgbb_reader.no_limits();
        let imgbb = imgbb_reader.decode().unwrap();
        let mut imgbb = imgbb.to_rgba8();
        for p in imgbb.pixels_mut() {
            if p[0] == 255 && p[1] == 255 && p[2] == 255 {
                p[3] = 0;
            }
        }
        let imgbb = image::imageops::crop(&mut imgbb, 0, 0, w, h).to_image();
        let imgbb_thumb = image::imageops::resize(
            &imgbb,
            new_width,
            new_height,
            image::imageops::FilterType::Nearest,
        );
        image::imageops::overlay(&mut img, &imgbb_thumb, 0, 0);
    }

    let black = Rgba([0, 0, 0, 255]);

    let mut cliffcolor =
        HashMap::from_iter([("cliff2", black), ("cliff3", black), ("cliff4", black)]);
    if config.cliffdebug {
        cliffcolor = HashMap::from_iter([
            ("cliff2", Rgba([100, 0, 100, 255])),
            ("cliff3", Rgba([0, 100, 100, 255])),
            ("cliff4", Rgba([100, 100, 0, 255])),
        ]);
    }
    let input = tmpfolder.join("c2g.dxf");
    let data = fs.read_to_string(input).expect("Can not read input file");
    let data: Vec<&str> = data.split("POLYLINE").collect();

    for (j, rec) in data.iter().enumerate() {
        let mut x = Vec::<f64>::new();
        let mut y = Vec::<f64>::new();
        let mut xline = 0;
        let mut yline = 0;
        let mut layer = "";
        if j > 0 {
            let r = rec.split("VERTEX").collect::<Vec<&str>>();
            let apu = r[1];
            let val = apu.split('\n').collect::<Vec<&str>>();
            layer = val[2].trim();
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
                    x.push(
                        (val[xline].trim().parse::<f64>().unwrap() - x0) * 600.0
                            / 254.0
                            / scalefactor,
                    );
                    y.push(
                        (y0 - val[yline].trim().parse::<f64>().unwrap()) * 600.0
                            / 254.0
                            / scalefactor,
                    );
                }
            }
        }
        let last_idx = x.len() - 1;
        if x.first() != x.last() || y.first() != y.last() {
            let dist = ((x[0] - x[last_idx]).powi(2) + (y[0] - y[last_idx]).powi(2)).sqrt();
            if dist > 0.0 {
                let dx = x[0] - x[last_idx];
                let dy = y[0] - y[last_idx];
                x[0] += dx / dist * 1.5;
                y[0] += dy / dist * 1.5;
                x[last_idx] -= dx / dist * 1.5;
                y[last_idx] -= dy / dist * 1.5;
                draw_filled_circle_mut(
                    &mut img,
                    (x[0] as i32, y[0] as i32),
                    3,
                    *cliffcolor.get(&layer).unwrap_or(&black),
                );
                draw_filled_circle_mut(
                    &mut img,
                    (x[last_idx] as i32, y[last_idx] as i32),
                    3,
                    *cliffcolor.get(&layer).unwrap_or(&black),
                );
            }
        }
        for i in 1..x.len() {
            for n in 0..6 {
                for m in 0..6 {
                    draw_line_segment_mut(
                        &mut img,
                        (
                            (x[i - 1] + (n as f64) - 3.0).floor() as f32,
                            (y[i - 1] + (m as f64) - 3.0).floor() as f32,
                        ),
                        (
                            (x[i] + (n as f64) - 3.0).floor() as f32,
                            (y[i] + (m as f64) - 3.0).floor() as f32,
                        ),
                        *cliffcolor.get(&layer).unwrap_or(&black),
                    )
                }
            }
        }
    }

    let input = &tmpfolder.join("c3g.dxf");
    let data = fs.read_to_string(input).expect("Can not read input file");
    let data: Vec<&str> = data.split("POLYLINE").collect();

    for (j, rec) in data.iter().enumerate() {
        let mut x = Vec::<f64>::new();
        let mut y = Vec::<f64>::new();
        let mut xline = 0;
        let mut yline = 0;
        let mut layer = "";
        if j > 0 {
            let r = rec.split("VERTEX").collect::<Vec<&str>>();
            let apu = r[1];
            let val = apu.split('\n').collect::<Vec<&str>>();
            layer = val[2].trim();
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
                    x.push(
                        (val[xline].trim().parse::<f64>().unwrap() - x0) * 600.0
                            / 254.0
                            / scalefactor,
                    );
                    y.push(
                        (y0 - val[yline].trim().parse::<f64>().unwrap()) * 600.0
                            / 254.0
                            / scalefactor,
                    );
                }
            }
        }
        let last_idx = x.len() - 1;
        if x.first() != x.last() || y.first() != y.last() {
            let dist = ((x[0] - x[last_idx]).powi(2) + (y[0] - y[last_idx]).powi(2)).sqrt();
            if dist > 0.0 {
                let dx = x[0] - x[last_idx];
                let dy = y[0] - y[last_idx];
                x[0] += dx / dist * 1.5;
                y[0] += dy / dist * 1.5;
                x[last_idx] -= dx / dist * 1.5;
                y[last_idx] -= dy / dist * 1.5;

                draw_filled_circle_mut(
                    &mut img,
                    (x[0] as i32, y[0] as i32),
                    3,
                    *cliffcolor.get(&layer).unwrap_or(&black),
                );
                draw_filled_circle_mut(
                    &mut img,
                    (x[last_idx] as i32, y[last_idx] as i32),
                    3,
                    *cliffcolor.get(&layer).unwrap_or(&black),
                );
            }
        }
        for i in 1..x.len() {
            for n in 0..6 {
                for m in 0..6 {
                    draw_line_segment_mut(
                        &mut img,
                        (
                            (x[i - 1] + (n as f64) - 3.0).floor() as f32,
                            (y[i - 1] + (m as f64) - 3.0).floor() as f32,
                        ),
                        (
                            (x[i] + (n as f64) - 3.0).floor() as f32,
                            (y[i] + (m as f64) - 3.0).floor() as f32,
                        ),
                        *cliffcolor.get(&layer).unwrap_or(&black),
                    )
                }
            }
        }
    }
    // high -------------
    let high_file = tmpfolder.join("high.png");
    if high_file.exists() {
        let mut high_reader =
            image::ImageReader::open(high_file).expect("Opening high image failed");
        high_reader.no_limits();
        let high = high_reader.decode().unwrap();
        let high_thumb = image::imageops::resize(
            &high,
            new_width,
            new_height,
            image::imageops::FilterType::Nearest,
        );
        image::imageops::overlay(&mut img, &high_thumb, 0, 0);
    }

    let filename = if nodepressions {
        format!("pullautus{}", thread)
    } else {
        format!("pullautus_depr{}", thread)
    };

    img.save(&format!("{}.png", filename))
        .expect("could not save output png");

    let file_in = tmpfolder.join("vegetation.pgw");
    let pgw_file_out = fs
        .create(format!("{}.pgw", filename))
        .expect("Unable to create file");
    let mut pgw_file_out = BufWriter::new(pgw_file_out);

    if let Ok(lines) = fs.open(file_in) {
        for (i, line) in BufReader::new(lines).lines().enumerate() {
            let ip = line.unwrap_or(String::new());
            let x: f64 = ip.parse::<f64>().unwrap();
            if i == 0 || i == 3 {
                write!(&mut pgw_file_out, "{}\r\n", x / 600.0 * 254.0 * scalefactor)
                    .expect("Unable to write to file");
            } else {
                write!(&mut pgw_file_out, "{}\r\n", ip).expect("Unable to write to file");
            }
        }
    }
    info!("Done");
    Ok(())
}

pub fn draw_curves(
    fs: &impl FileSystem,
    config: &Config,
    canvas: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    tmpfolder: &Path,
    nodepressions: bool,
    draw_image: bool,
) -> Result<(), Box<dyn Error>> {
    // Drawing curves --------------
    let &Config {
        scalefactor,
        mut formlinesteepness,
        formline,
        formlineaddition,
        dashlength,
        gaplength,
        minimumgap,
        label_depressions,
        ..
    } = config;
    formlinesteepness *= scalefactor;

    let mut size: f64 = 0.0;
    let mut xstart: f64 = 0.0;
    let mut ystart: f64 = 0.0;
    let mut x0: f64 = 0.0;
    let mut y0: f64 = 0.0;
    let mut steepness: HashMap<(usize, usize), f64> = HashMap::default();

    if formline > 0.0 {
        let heightmap_in = tmpfolder.join("xyz2.hmap");
        let mut reader = BufReader::new(fs.open(heightmap_in)?);
        let hmap = HeightMap::from_bytes(&mut reader)?;

        xstart = hmap.xoffset;
        ystart = hmap.yoffset;
        size = hmap.scale;

        x0 = xstart;

        // Temporarily convert to HashMap for not having to go through all the logic below.
        let mut xyz: HashMap<(usize, usize), f64> = HashMap::default();
        for (x, y, h) in hmap.grid.iter() {
            xyz.insert((x, y), h);
        }
        y0 = hmap.maxy();

        let sxmax = hmap.grid.width() - 1;
        let symax = hmap.grid.height() - 1;

        for i in 6..(sxmax - 7) {
            for j in 6..(symax - 7) {
                let mut det: f64 = 0.0;
                let mut high: f64 = f64::MIN;

                let mut temp =
                    (xyz.get(&(i - 4, j)).unwrap_or(&0.0) - xyz.get(&(i, j)).unwrap_or(&0.0)).abs()
                        / 4.0;
                let temp2 =
                    (xyz.get(&(i, j)).unwrap_or(&0.0) - xyz.get(&(i + 4, j)).unwrap_or(&0.0)).abs()
                        / 4.0;
                let det2 = (xyz.get(&(i, j)).unwrap_or(&0.0)
                    - 0.5
                        * (xyz.get(&(i - 4, j)).unwrap_or(&0.0)
                            + xyz.get(&(i + 4, j)).unwrap_or(&0.0)))
                .abs()
                    - 0.05
                        * (xyz.get(&(i - 4, j)).unwrap_or(&0.0)
                            - xyz.get(&(i + 4, j)).unwrap_or(&0.0))
                        .abs();
                let mut porr = (((xyz.get(&(i - 6, j)).unwrap_or(&0.0)
                    - xyz.get(&(i + 6, j)).unwrap_or(&0.0))
                    / 12.0)
                    .abs()
                    - ((xyz.get(&(i - 3, j)).unwrap_or(&0.0)
                        - xyz.get(&(i + 3, j)).unwrap_or(&0.0))
                        / 6.0)
                        .abs())
                .abs();

                if det2 > det {
                    det = det2;
                }
                if temp2 < temp {
                    temp = temp2;
                }
                if temp > high {
                    high = temp;
                }

                let mut temp =
                    (xyz.get(&(i, j - 4)).unwrap_or(&0.0) - xyz.get(&(i, j)).unwrap_or(&0.0)).abs()
                        / 4.0;
                let temp2 =
                    (xyz.get(&(i, j)).unwrap_or(&0.0) - xyz.get(&(i, j - 4)).unwrap_or(&0.0)).abs()
                        / 4.0;
                let det2 = (xyz.get(&(i, j)).unwrap_or(&0.0)
                    - 0.5
                        * (xyz.get(&(i, j - 4)).unwrap_or(&0.0)
                            + xyz.get(&(i, j + 4)).unwrap_or(&0.0)))
                .abs()
                    - 0.05
                        * (xyz.get(&(i, j - 4)).unwrap_or(&0.0)
                            - xyz.get(&(i, j + 4)).unwrap_or(&0.0))
                        .abs();
                let porr2 = (((xyz.get(&(i, j - 6)).unwrap_or(&0.0)
                    - xyz.get(&(i, j + 6)).unwrap_or(&0.0))
                    / 12.0)
                    .abs()
                    - ((xyz.get(&(i, j - 3)).unwrap_or(&0.0)
                        - xyz.get(&(i, j + 3)).unwrap_or(&0.0))
                        / 6.0)
                        .abs())
                .abs();

                if porr2 > porr {
                    porr = porr2;
                }
                if det2 > det {
                    det = det2;
                }
                if temp2 < temp {
                    temp = temp2;
                }
                if temp > high {
                    high = temp;
                }

                let mut temp = (xyz.get(&(i - 4, j - 4)).unwrap_or(&0.0)
                    - xyz.get(&(i, j)).unwrap_or(&0.0))
                .abs()
                    / 5.6;
                let temp2 = (xyz.get(&(i, j)).unwrap_or(&0.0)
                    - xyz.get(&(i + 4, j + 4)).unwrap_or(&0.0))
                .abs()
                    / 5.6;
                let det2 = (xyz.get(&(i, j)).unwrap_or(&0.0)
                    - 0.5
                        * (xyz.get(&(i - 4, j - 4)).unwrap_or(&0.0)
                            + xyz.get(&(i + 4, j + 4)).unwrap_or(&0.0)))
                .abs()
                    - 0.05
                        * (xyz.get(&(i - 4, j - 4)).unwrap_or(&0.0)
                            - xyz.get(&(i + 4, j + 4)).unwrap_or(&0.0))
                        .abs();
                let porr2 = (((xyz.get(&(i - 6, j - 6)).unwrap_or(&0.0)
                    - xyz.get(&(i + 6, j + 6)).unwrap_or(&0.0))
                    / 17.0)
                    .abs()
                    - ((xyz.get(&(i - 3, j - 3)).unwrap_or(&0.0)
                        - xyz.get(&(i + 3, j + 3)).unwrap_or(&0.0))
                        / 8.5)
                        .abs())
                .abs();

                if porr2 > porr {
                    porr = porr2;
                }
                if det2 > det {
                    det = det2;
                }
                if temp2 < temp {
                    temp = temp2;
                }
                if temp > high {
                    high = temp;
                }

                let mut temp = (xyz.get(&(i - 4, j + 4)).unwrap_or(&0.0)
                    - xyz.get(&(i, j)).unwrap_or(&0.0))
                .abs()
                    / 5.6;
                let temp2 = (xyz.get(&(i, j)).unwrap_or(&0.0)
                    - xyz.get(&(i + 4, j - 4)).unwrap_or(&0.0))
                .abs()
                    / 5.6;
                let det2 = (xyz.get(&(i, j)).unwrap_or(&0.0)
                    - 0.5
                        * (xyz.get(&(i + 4, j - 4)).unwrap_or(&0.0)
                            + xyz.get(&(i - 4, j + 4)).unwrap_or(&0.0)))
                .abs()
                    - 0.05
                        * (xyz.get(&(i + 4, j - 4)).unwrap_or(&0.0)
                            - xyz.get(&(i - 4, j + 4)).unwrap_or(&0.0))
                        .abs();
                let porr2 = (((xyz.get(&(i + 6, j - 6)).unwrap_or(&0.0)
                    - xyz.get(&(i - 6, j + 6)).unwrap_or(&0.0))
                    / 17.0)
                    .abs()
                    - ((xyz.get(&(i + 3, j - 3)).unwrap_or(&0.0)
                        - xyz.get(&(i - 3, j + 3)).unwrap_or(&0.0))
                        / 8.5)
                        .abs())
                .abs();

                if porr2 > porr {
                    porr = porr2;
                }
                if det2 > det {
                    det = det2;
                }
                if temp2 < temp {
                    temp = temp2;
                }
                if temp > high {
                    high = temp;
                }

                let mut val = 12.0 * high / (1.0 + 8.0 * det);
                if porr > 0.25 * 0.67 / (0.3 + formlinesteepness) {
                    val = 0.01;
                }
                if high > val {
                    val = high;
                }
                steepness.insert((i, j), val);
            }
        }
    }

    let input = &tmpfolder.join("out2.dxf");
    let data = fs.read_to_string(input).expect("Can not read input file");
    let data: Vec<&str> = data.split("POLYLINE").collect();

    // only create the file if condition is met
    let mut fp = if formline == 2.0 && !nodepressions {
        let output = tmpfolder.join("formlines.dxf");
        let fp = fs.create(output).expect("Unable to create file");
        let mut fp = BufWriter::new(fp);
        fp.write_all(data[0].as_bytes())
            .expect("Could not write file");

        Some(fp)
    } else {
        None
    };

    for (j, rec) in data.iter().enumerate() {
        let mut x = Vec::<f64>::new();
        let mut y = Vec::<f64>::new();
        let mut xline = 0;
        let mut yline = 0;
        let mut layer = "";
        if j > 0 {
            let r = rec.split("VERTEX").collect::<Vec<&str>>();
            let apu = r[1];
            let val = apu.split('\n').collect::<Vec<&str>>();
            layer = val[2].trim();
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
                    x.push(
                        (val[xline].trim().parse::<f64>().unwrap() - x0) * 600.0
                            / 254.0
                            / scalefactor,
                    );
                    y.push(
                        (y0 - val[yline].trim().parse::<f64>().unwrap()) * 600.0
                            / 254.0
                            / scalefactor,
                    );
                }
            }
        }
        let mut color = Rgba([200, 0, 200, 255]); // purple
        if layer.contains("contour") {
            color = Rgba([166, 85, 43, 255]) // brown
        }

        if !nodepressions || layer.contains("contour") {
            let mut curvew = 2.0;
            if layer.contains("index") {
                curvew = 3.0;
            }
            if formline > 0.0 {
                if formline == 1.0 {
                    curvew = 2.5
                }
                if layer.contains("intermed") {
                    curvew = 1.5
                }
                if layer.contains("index") {
                    curvew = 3.5
                }
            }

            let mut smallringtest = false;
            let mut help = vec![false; x.len()];
            let mut help2 = vec![false; x.len()];
            if curvew == 1.5 {
                for i in 0..x.len() {
                    help[i] = false;
                    help2[i] = true;
                    let xx = (((x[i] / 600.0 * 254.0 * scalefactor + x0) - xstart) / size).floor();
                    let yy = (((-y[i] / 600.0 * 254.0 * scalefactor + y0) - ystart) / size).floor();
                    if curvew != 1.5
                        || formline == 0.0
                        || steepness.get(&(xx as usize, yy as usize)).unwrap_or(&0.0)
                            < &formlinesteepness
                        || steepness
                            .get(&(xx as usize, yy as usize + 1))
                            .unwrap_or(&0.0)
                            < &formlinesteepness
                        || steepness
                            .get(&(xx as usize + 1, yy as usize))
                            .unwrap_or(&0.0)
                            < &formlinesteepness
                        || steepness
                            .get(&(xx as usize + 1, yy as usize + 1))
                            .unwrap_or(&0.0)
                            < &formlinesteepness
                    {
                        help[i] = true;
                    }
                }
                for i in 5..(x.len() - 6) {
                    let mut apu = 0;
                    for j in (i - 5)..(i + 4) {
                        if help[j] {
                            apu += 1;
                        }
                    }
                    if apu < 5 {
                        help2[i] = false;
                    }
                }
                for i in 0..6 {
                    help2[i] = help2[6]
                }
                for i in (x.len() - 6)..x.len() {
                    help2[i] = help2[x.len() - 7]
                }
                let mut on = 0.0;
                for i in 0..x.len() {
                    if help2[i] {
                        on = formlineaddition
                    }
                    if on > 0.0 {
                        help2[i] = true;
                        on -= 1.0;
                    }
                }
                if x.first() == x.last() && y.first() == y.last() && on > 0.0 {
                    let mut i = 0;
                    while i < x.len() && on > 0.0 {
                        help2[i] = true;
                        on -= 1.0;
                        i += 1;
                    }
                }
                let mut on = 0.0;
                for i in 0..x.len() {
                    let ii = x.len() - i - 1;
                    if help2[ii] {
                        on = formlineaddition
                    }
                    if on > 0.0 {
                        help2[ii] = true;
                        on -= 1.0;
                    }
                }
                if x.first() == x.last() && y.first() == y.last() && on > 0.0 {
                    let mut i = (x.len() - 1) as i32;
                    while i > -1 && on > 0.0 {
                        help2[i as usize] = true;
                        on -= 1.0;
                        i -= 1;
                    }
                }
                // Let's not break small form line rings
                smallringtest = false;
                if x.first() == x.last() && y.first() == y.last() && x.len() < 122 {
                    for i in 1..x.len() {
                        if help2[i] {
                            smallringtest = true
                        }
                    }
                }
                // Let's draw short gaps together
                if !smallringtest {
                    let mut tester = 1;
                    for i in 1..x.len() {
                        if help2[i] {
                            if tester < i && ((i - tester) as u32) < minimumgap {
                                for j in tester..(i + 1) {
                                    help2[j] = true;
                                }
                            }
                            tester = i;
                        }
                    }
                    // Ring handling
                    if x.first() == x.last() && y.first() == y.last() && x.len() < 2 {
                        let mut i = 1;
                        while i < x.len() && !help2[i] {
                            i += 1
                        }
                        let mut j = x.len() - 1;
                        while j > 1 && !help2[i] {
                            j -= 1
                        }
                        if ((x.len() - j + i - 1) as u32) < minimumgap && j > i {
                            for k in 0..(i + 1) {
                                help2[k] = true
                            }
                            for k in j..x.len() {
                                help2[k] = true
                            }
                        }
                    }
                }
            }

            let mut linedist = 0.0;
            let mut onegapdone = false;
            let mut gap = 0.0;
            let mut formlinestart = false;

            let f_label = if layer.contains("depression") && label_depressions {
                "formline_depression"
            } else {
                "formline"
            };

            for i in 1..x.len() {
                if curvew != 1.5 || formline == 0.0 || help2[i] || smallringtest {
                    if let (Some(fp), true) = (fp.as_mut(), curvew == 1.5) {
                        if !formlinestart {
                            write!(fp, "POLYLINE\r\n 66\r\n1\r\n  8\r\n{}\r\n  0\r\n", f_label)
                                .expect("Could not write file");
                            formlinestart = true;
                        }
                        write!(
                            fp,
                            "VERTEX\r\n  8\r\n{}\r\n 10\r\n{}\r\n 20\r\n{}\r\n  0\r\n",
                            f_label,
                            x[i] / 600.0 * 254.0 * scalefactor + x0,
                            -y[i] / 600.0 * 254.0 * scalefactor + y0
                        )
                        .expect("Could not write file");
                    }

                    if draw_image {
                        if curvew == 1.5 && formline == 2.0 {
                            let step =
                                ((x[i - 1] - x[i]).powi(2) + (y[i - 1] - y[i]).powi(2)).sqrt();
                            if i < 4 {
                                linedist = 0.0
                            }
                            linedist += step;
                            if linedist > dashlength && i > 10 && i < x.len() - 11 {
                                let mut sum = 0.0;
                                for k in (i - 4)..(i + 6) {
                                    sum += ((x[k - 1] - x[k]).powi(2) + (y[k - 1] - y[k]).powi(2))
                                        .sqrt()
                                }
                                let mut toonearend = false;
                                for k in (i - 10)..(i + 10) {
                                    if !help2[k] {
                                        toonearend = true;
                                        break;
                                    }
                                }
                                if !toonearend
                                    && ((x[i - 5] - x[i + 5]).powi(2)
                                        + (y[i - 5] - y[i + 5]).powi(2))
                                    .sqrt()
                                        * 1.138
                                        > sum
                                {
                                    linedist = 0.0;
                                    gap = gaplength;
                                    onegapdone = true;
                                }
                            }
                            if !onegapdone && (i < x.len() - 9) && i > 6 {
                                gap = gaplength * 0.82;
                                onegapdone = true;
                                linedist = 0.0
                            }
                            if gap > 0.0 {
                                gap -= step;
                                if gap < 0.0 && onegapdone && step > 0.0 {
                                    let mut n = -curvew - 0.5;
                                    while n < curvew + 0.5 {
                                        let mut m = -curvew - 0.5;
                                        while m < curvew + 0.5 {
                                            draw_line_segment_mut(
                                                canvas,
                                                (
                                                    ((-x[i - 1] * gap + (step + gap) * x[i]) / step
                                                        + n)
                                                        as f32,
                                                    ((-y[i - 1] * gap + (step + gap) * y[i]) / step
                                                        + m)
                                                        as f32,
                                                ),
                                                ((x[i] + n) as f32, (y[i] + m) as f32),
                                                color,
                                            );
                                            m += 1.0;
                                        }
                                        n += 1.0;
                                    }
                                    gap = 0.0;
                                }
                            } else {
                                let mut n = -curvew - 0.5;
                                while n < curvew + 0.5 {
                                    let mut m = -curvew - 0.5;
                                    while m < curvew + 0.5 {
                                        draw_line_segment_mut(
                                            canvas,
                                            ((x[i - 1] + n) as f32, (y[i - 1] + m) as f32),
                                            ((x[i] + n) as f32, (y[i] + m) as f32),
                                            color,
                                        );
                                        m += 1.0;
                                    }
                                    n += 1.0;
                                }
                            }
                        } else {
                            let mut n = -curvew;
                            while n < curvew {
                                let mut m = -curvew;
                                while m < curvew {
                                    draw_line_segment_mut(
                                        canvas,
                                        ((x[i - 1] + n) as f32, (y[i - 1] + m) as f32),
                                        ((x[i] + n) as f32, (y[i] + m) as f32),
                                        color,
                                    );
                                    m += 1.0;
                                }
                                n += 1.0;
                            }
                        }
                    }
                } else if let (Some(fp), true) = (fp.as_mut(), formlinestart) {
                    fp.write_all(b"SEQEND\r\n  0\r\n")
                        .expect("Could not write file");
                    formlinestart = false;
                }
            }
            if let (Some(fp), true) = (fp.as_mut(), formlinestart) {
                fp.write_all(b"SEQEND\r\n  0\r\n")
                    .expect("Could not write file");
            }
        }
    }
    if let Some(fp) = fp.as_mut() {
        fp.write_all(b"ENDSEC\r\n  0\r\nEOF\r\n")
            .expect("Could not write file");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_mapping_from_str_invalid_no_isom() {
        let line = "description||key1=value1";
        let mapping = Mapping::from_str(line);
        assert!(mapping.is_err());
    }

    #[test]
    fn test_mapping_from_str_invalid_no_conditions() {
        let line = "description|306|";
        let mapping = Mapping::from_str(line);
        assert!(mapping.is_err());
    }

    #[test]
    fn test_mapping_from_str_valid_single() {
        let line = "description|306|key1=value1";
        let mapping = Mapping::from_str(line).unwrap();
        let expected = Mapping {
            isom: "306".to_string(),
            conditions: vec![Condition {
                operator: Operator::Equal,
                key: "key1".to_string(),
                value: "value1".to_string(),
            }],
        };
        assert_eq!(mapping, expected);
    }

    #[test]
    fn test_mapping_from_str_valid_two() {
        let line = "description|306|key1=value1&key2!=value2";
        let mapping = Mapping::from_str(line).unwrap();
        let expected = Mapping {
            isom: "306".to_string(),
            conditions: vec![
                Condition {
                    operator: Operator::Equal,
                    key: "key1".to_string(),
                    value: "value1".to_string(),
                },
                Condition {
                    operator: Operator::NotEqual,
                    key: "key2".to_string(),
                    value: "value2".to_string(),
                },
            ],
        };
        assert_eq!(mapping, expected);
    }

    #[test]
    fn test_mapping_from_str_valid_more() {
        let line = "description|306|key1=value1&key2!=value2&key3=value3";
        let mapping = Mapping::from_str(line).unwrap();
        let expected = Mapping {
            isom: "306".to_string(),
            conditions: vec![
                Condition {
                    operator: Operator::Equal,
                    key: "key1".to_string(),
                    value: "value1".to_string(),
                },
                Condition {
                    operator: Operator::NotEqual,
                    key: "key2".to_string(),
                    value: "value2".to_string(),
                },
                Condition {
                    operator: Operator::Equal,
                    key: "key3".to_string(),
                    value: "value3".to_string(),
                },
            ],
        };
        assert_eq!(mapping, expected);
    }

    #[test]
    fn test_mapping_from_str_invalid() {
        let line = "306|key1=value1&key2!=value2";
        let result = Mapping::from_str(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_mapping_from_str_invalid_operator() {
        let line = "description|306|key1=value1&key2>value2";
        let result = Mapping::from_str(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_mapping_from_str_missing_sections() {
        let line = "306|key1=value1";
        let result = Mapping::from_str(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_mapping_from_str_empty_line() {
        let line = "";
        let result = Mapping::from_str(line);
        assert!(result.is_err());
    }

    #[test]
    fn test_mapping_from_str_extra_sections() {
        let line = "description|306|key1=value1&key2!=value2|extra";
        let result = Mapping::from_str(line);
        assert!(result.is_err());
    }

    /// Make sure the bundled osm.txt file can be parsed
    #[test]
    fn test_mapping_from_str_osm() {
        let lines = include_str!("../osm.txt");
        for line in lines.lines() {
            Mapping::from_str(line).unwrap();
        }
    }

    /// Make sure the bundled fastighetskartan.txt file can be parsed
    #[test]
    fn test_mapping_from_str_fastighetskartan() {
        let lines = include_str!("../fastighetskartan.txt");
        for line in lines.lines() {
            Mapping::from_str(line).unwrap();
        }
    }
}
