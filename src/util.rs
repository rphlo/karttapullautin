use std::{
    fs::File,
    mem,
    slice,
    io::{self, BufRead, Read, ErrorKind, Error, Cursor},
    path::Path,
};

pub struct XYZPoint {
    pub x: f64,
    pub y: f64,
    pub h: f64,
    pub classification: u8,
    pub number_of_returns: u8,
    pub return_number: u8,
}

const PT_SIZE: usize = mem::size_of::<XYZPoint>();

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}

impl XYZPoint {
    pub fn to_bytes(&self) -> &[u8] {
        let bytes: &[u8] = unsafe { any_as_u8_slice(self) };
        bytes
    }
}

fn xyz_point_from_bytes(bytes: &Vec<u8>) -> XYZPoint {
    let mut c = Cursor::new(bytes);
    let mut pt: XYZPoint = unsafe { mem::zeroed() };
    unsafe {
        let pt_slice = slice::from_raw_parts_mut(&mut pt as *mut _ as *mut u8, PT_SIZE);
        // `read_exact()` comes from `Read` impl for `&[u8]`
        c.read_exact(pt_slice).unwrap();
    }
    pt
}

pub fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn read_n_points<P>(filename: P, n: u8) -> std::result::Result<Vec<XYZPoint>, Error>
where
    P: AsRef<Path>,
{
    let mut file = File::open(filename)?;
    let mut out = vec![];
    let mut line_buffer = vec![0u8; PT_SIZE];
    let mut i = 0;
    loop {
      match file.read_exact(&mut line_buffer) {
        Ok(()) => {
            let pt = xyz_point_from_bytes(&line_buffer);
            out.push(pt);
            i += 1;
            if i == n {
                break;
            }
        },
        Err(e) => {
            if e.kind() == ErrorKind::UnexpectedEof {
                break;
            }
            panic!("Unexpected error reading file.");
        }  
      }
    }
    Ok(out)
}

/// Iterates over the lines in a file and calls the callback with a &str reference to each line.
/// This function does not allocate new strings for each line, as opposed to using
/// [`io::BufReader::lines()`].
pub fn read_bytes_no_alloc<P>(filename: P, mut line_callback: impl FnMut(&XYZPoint)) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let mut file = File::open(filename)?;
    let mut pt: XYZPoint = unsafe { mem::zeroed() };
    unsafe {
        let pt_slice = slice::from_raw_parts_mut(&mut pt as *mut _ as *mut u8, PT_SIZE);
  
    loop {
      match file.read_exact(pt_slice) {
        Ok(()) => {
            line_callback(&pt);
        },
        Err(e) => {
            if e.kind() == ErrorKind::UnexpectedEof {
                break;
            }
            panic!("Unexpected error reading file.");
        }  
        }
    }}
   Ok(())
}

pub fn read_lines_no_alloc<P>(filename: P, mut line_callback: impl FnMut(&str)) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    let mut reader = io::BufReader::new(file);

    let mut line_buffer = String::new();
    while reader.read_line(&mut line_buffer)? > 0 {
        // the read line contains the newline delimiter, so we need to trim it off
        let line = line_buffer.trim_end();
        line_callback(line);
        line_buffer.clear();
    }

    Ok(())
}
