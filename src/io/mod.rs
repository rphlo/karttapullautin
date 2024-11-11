use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};

use log::trace;

/// The magic number that identifies a valid XYZ binary file.
const XYZ_MAGIC: &[u8] = b"XYZB";

#[derive(Debug, Clone, Copy)]
pub enum Format {
    Xyz,
    XyzMeta,
}

impl From<Format> for u8 {
    fn from(value: Format) -> u8 {
        match value {
            Format::Xyz => 1,
            Format::XyzMeta => 2,
        }
    }
}

impl TryFrom<u8> for Format {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Format::Xyz),
            2 => Ok(Format::XyzMeta),
            _ => Err(format!("unknown Format value: {}", value)),
        }
    }
}

/// A single record of an observed laser data point needed by the algorithms.
#[derive(Debug, Clone, PartialEq)]
pub struct XyzRecord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub meta: Option<XyzRecordMeta>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct XyzRecordMeta {
    pub classification: u8,
    pub number_of_returns: u8,
    pub return_number: u8,
}

impl XyzRecord {
    fn write<W: Write>(&self, writer: &mut W, format: Format) -> std::io::Result<()> {
        // write the x, y, z coordinates
        writer.write_all(&self.x.to_ne_bytes())?;
        writer.write_all(&self.y.to_ne_bytes())?;
        writer.write_all(&self.z.to_ne_bytes())?;

        // write the classification, number of returns, return number, and intensity

        match (format, &self.meta) {
            (Format::Xyz, _) => { //do nothing
            }
            (Format::XyzMeta, Some(meta)) => {
                writer.write_all(&[meta.classification])?;
                writer.write_all(&[meta.number_of_returns])?;
                writer.write_all(&[meta.return_number])?;
            }
            (Format::XyzMeta, None) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "meta data required for XyzMeta format",
                ));
            }
        }

        Ok(())
    }

    fn read<R: Read>(reader: &mut R, format: Format) -> std::io::Result<Self> {
        let mut buff = [0; 8];
        reader.read_exact(&mut buff)?;
        let x = f64::from_ne_bytes(buff);

        // let mut buff = [0; 8];
        reader.read_exact(&mut buff)?;
        let y = f64::from_ne_bytes(buff);

        // let mut buff = [0; 8];
        reader.read_exact(&mut buff)?;
        let z = f64::from_ne_bytes(buff);

        let meta = match format {
            Format::Xyz => None,
            Format::XyzMeta => {
                let mut buff = [0; 1];
                reader.read_exact(&mut buff)?;
                let classification = buff[0];

                reader.read_exact(&mut buff)?;
                let number_of_returns = buff[0];

                reader.read_exact(&mut buff)?;
                let return_number = buff[0];

                Some(XyzRecordMeta {
                    classification,
                    number_of_returns,
                    return_number,
                })
            }
        };

        Ok(Self { x, y, z, meta })
    }
}

pub struct XyzInternalWriter<W: Write> {
    inner: W,
    n_records: u64,
    records_written: u64,
    format: Format,
}

impl XyzInternalWriter<BufWriter<File>> {
    pub fn create(path: &Path, format: Format, n_records: u64) -> std::io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self::new(BufWriter::new(file), format, n_records))
    }
}

impl<W: Write> XyzInternalWriter<W> {
    pub fn new(inner: W, format: Format, n_records: u64) -> Self {
        Self {
            inner,
            records_written: 0,
            n_records,
            format,
        }
    }

    pub fn write_record(&mut self, record: &XyzRecord) -> std::io::Result<()> {
        if self.records_written >= self.n_records {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "too many records written",
            ));
        }

        // write the header (format + length) on the first write
        if self.records_written == 0 {
            self.inner.write_all(XYZ_MAGIC).unwrap();
            self.inner.write_all(&[self.format.into()]).unwrap();
            self.inner.write_all(&self.n_records.to_ne_bytes()).unwrap();
        }

        record.write(&mut self.inner, self.format)?;
        self.records_written += 1;
        Ok(())
    }

    pub fn finish(self) -> W {
        self.inner
    }
}

// // If the writer is dropped before all records are written, it will panic.
// impl<W: Write> Drop for XyzInternalWriter<W> {
//     fn drop(&mut self) {
//         if self.records_written != self.n_records {
//             panic!(
//                 "not all records written: expected {}, got {}",
//                 self.n_records, self.records_written
//             );
//         }
//     }
// }

pub struct XyzInternalReader<R: Read> {
    inner: R,
    format: Format,
    n_records: u64,
    records_read: u64,
}

impl XyzInternalReader<BufReader<File>> {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let file = File::open(path)?;
        Self::new(BufReader::new(file))
    }
}

impl<R: Read> XyzInternalReader<R> {
    pub fn new(mut inner: R) -> std::io::Result<Self> {
        // read and check the magic number
        let mut buff = [0; XYZ_MAGIC.len()];
        inner.read_exact(&mut buff)?;
        if buff != XYZ_MAGIC {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid magic number",
            ));
        }

        // read and parse the format
        let mut buff = [0; 1];
        inner.read_exact(&mut buff)?;
        let format = buff[0].try_into().expect("should have known format");

        // read the number of records, defined by the first u64
        let mut buff = [0; 8];
        inner.read_exact(&mut buff)?;
        let n_records = u64::from_ne_bytes(buff);
        trace!("reading {} records", n_records);
        Ok(Self {
            inner,
            format,
            n_records,
            records_read: 0,
        })
    }

    pub fn next(&mut self) -> std::io::Result<Option<XyzRecord>> {
        if self.records_read >= self.n_records {
            return Ok(None);
        }

        let record = XyzRecord::read(&mut self.inner, self.format)?;
        self.records_read += 1;
        Ok(Some(record))
    }

    pub fn format(&self) -> Format {
        self.format
    }

    pub fn n_records(&self) -> u64 {
        self.n_records
    }

    pub fn finish(self) -> R {
        self.inner
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::io::XyzRecord;

    use super::*;

    #[test]
    fn test_xyz_record() {
        let record = XyzRecord {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            meta: Some(XyzRecordMeta {
                classification: 4,
                number_of_returns: 5,
                return_number: 6,
            }),
        };

        let mut buff = Vec::new();
        record.write(&mut buff, Format::XyzMeta).unwrap();
        let read_record = XyzRecord::read(&mut buff.as_slice(), Format::XyzMeta).unwrap();

        assert_eq!(record, read_record);
    }

    #[test]
    fn test_writer_reader_many() {
        let cursor = Cursor::new(Vec::new());
        let mut writer = XyzInternalWriter::new(cursor, Format::XyzMeta, 3);

        let record = XyzRecord {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            meta: Some(XyzRecordMeta {
                classification: 4,
                number_of_returns: 5,
                return_number: 6,
            }),
        };

        writer.write_record(&record).unwrap();
        writer.write_record(&record).unwrap();
        writer.write_record(&record).unwrap();

        // now read the records
        let cursor = Cursor::new(writer.finish().into_inner());
        let mut reader = super::XyzInternalReader::new(cursor).unwrap();
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert_eq!(reader.next().unwrap(), None);
    }
}
