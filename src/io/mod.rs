use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use heightmap::HeightMap;

pub mod bytes;
pub mod heightmap;
pub mod xyz;

/// Helper function to convert an internal xyz file to a regular xyz file.
pub fn internal2xyz(input: &str, output: &str) -> std::io::Result<()> {
    if input.ends_with(".xyz.bin") {
        let mut reader = xyz::XyzInternalReader::open(Path::new(input))?;
        let mut writer = BufWriter::new(File::create(output)?);

        while let Some(record) = reader.next()? {
            writeln!(
                writer,
                "{} {} {} {} {} {}",
                record.x,
                record.y,
                record.z,
                record.classification,
                record.number_of_returns,
                record.return_number
            )?;
        }
    } else if input.ends_with(".hmap") {
        let hmap = HeightMap::from_file(input)?;
        let mut writer = BufWriter::new(File::create(output)?);

        for (x, y, h) in hmap.iter() {
            writeln!(writer, "{} {} {}", x, y, h)?;
        }
    } else {
        panic!("Unknown internal file format: {}", input);
    }

    Ok(())
}
