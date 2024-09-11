use std::{
    fmt::Debug,
    fs::File,
    io::{self, BufRead},
    path::Path,
    time::Instant,
};

use log::debug;

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Iterates over the lines in a file and calls the callback with a &str reference to each line.
/// This function does not allocate new strings for each line, as opposed to using
/// [`io::BufReader::lines()`] as in [`read_lines`].
pub fn read_lines_no_alloc<P>(filename: P, mut line_callback: impl FnMut(&str)) -> io::Result<()>
where
    P: AsRef<Path> + Debug,
{
    debug!("Reading lines from {filename:?}");
    let start = Instant::now();

    let file = File::open(filename)?;
    let mut reader = io::BufReader::new(file);

    let mut line_buffer = String::new();
    let mut line_count: u32 = 0;
    let mut byte_count: usize = 0;
    loop {
        let bytes_read = reader.read_line(&mut line_buffer)?;

        if bytes_read == 0 {
            break;
        }

        line_count += 1;
        byte_count += bytes_read;

        // the read line contains the newline delimiter, so we need to trim it off
        let line = line_buffer.trim_end();
        line_callback(line);
        line_buffer.clear();
    }

    let elapsed = start.elapsed();
    debug!(
        "Read {} lines in {:?} ({:?}/line), total {} bytes ({:.2} bytes / second, {:?} / byte, {:.2} bytes / line)",
        line_count,
        elapsed,
        elapsed / line_count,
        byte_count,
        byte_count as f64 / elapsed.as_secs_f64(),
        elapsed / byte_count as u32,
        byte_count as f64 / line_count as f64,
    );

    Ok(())
}
