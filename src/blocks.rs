use image::{DynamicImage, Rgb, RgbImage, Rgba, RgbaImage};
use imageproc::drawing::draw_filled_rect_mut;
use imageproc::filter::median_filter;
use imageproc::rect::Rect;
use log::info;
use rustc_hash::FxHashMap as HashMap;
use std::{error::Error, path::Path};

use crate::io::xyz::XyzInternalReader;

pub fn blocks(tmpfolder: &Path) -> Result<(), Box<dyn Error>> {
    info!("Identifying blocks...");
    let xyz_file_in = tmpfolder.join("xyz2.xyz.bin");
    let mut xmax: u64 = u64::MIN;
    let mut ymax: u64 = u64::MIN;

    let mut reader = XyzInternalReader::open(&xyz_file_in)?;
    let first = reader.next()?.expect("should have record");
    let second = reader.next()?.expect("should have record");
    let (xstartxyz, ystartxyz, size) = (first.x, first.y, second.y - first.y);

    let mut xyz: HashMap<(u64, u64), f64> = HashMap::default();
    let mut reader = XyzInternalReader::open(&xyz_file_in).unwrap();
    while let Some(r) = reader.next().unwrap() {
        let (x, y, h) = (r.x, r.y, r.z);

        let xx = ((x - xstartxyz) / size).floor() as u64;
        let yy = ((y - ystartxyz) / size).floor() as u64;
        xyz.insert((xx, yy), h);

        if xmax < xx {
            xmax = xx;
        }
        if ymax < yy {
            ymax = yy;
        }
    }

    let mut img = RgbImage::from_pixel(xmax as u32 * 2, ymax as u32 * 2, Rgb([255, 255, 255]));
    let mut img2 = RgbaImage::from_pixel(xmax as u32 * 2, ymax as u32 * 2, Rgba([0, 0, 0, 0]));

    let black = Rgb([0, 0, 0]);
    let white = Rgba([255, 255, 255, 255]);

    let xyz_file_in = tmpfolder.join("xyztemp.xyz.bin");
    let mut reader = XyzInternalReader::open(&xyz_file_in).unwrap();
    while let Some(r) = reader.next().unwrap() {
        let (x, y, h) = (r.x, r.y, r.z);
        let r3 = r.classification;
        let r4 = r.number_of_returns;
        let r5 = r.return_number;

        let xx = ((x - xstartxyz) / size).floor() as u64;
        let yy = ((y - ystartxyz) / size).floor() as u64;
        if r3 != 2 && r3 != 9 && r4 == 1 && r5 == 1 && h - *xyz.get(&(xx, yy)).unwrap_or(&0.0) > 2.0
        {
            draw_filled_rect_mut(
                &mut img,
                Rect::at(
                    (x - xstartxyz - 1.0) as i32,
                    (ystartxyz + 2.0 * ymax as f64 - y - 1.0) as i32,
                )
                .of_size(3, 3),
                black,
            );
        } else {
            draw_filled_rect_mut(
                &mut img2,
                Rect::at(
                    (x - xstartxyz - 1.0) as i32,
                    (ystartxyz + 2.0 * ymax as f64 - y - 1.0) as i32,
                )
                .of_size(3, 3),
                white,
            );
        }
    }

    img2.save(tmpfolder.join("blocks2.png"))
        .expect("error saving png");

    let mut img = DynamicImage::ImageRgb8(img);

    image::imageops::overlay(&mut img, &DynamicImage::ImageRgba8(img2), 0, 0);

    let filter_size = 2;
    img = image::DynamicImage::ImageRgb8(median_filter(&img.to_rgb8(), filter_size, filter_size));

    img.save(tmpfolder.join("blocks.png"))
        .expect("error saving png");
    info!("Done");
    Ok(())
}
