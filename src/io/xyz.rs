use crate::io::bytes::FromToBytes;
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Seek, Write},
    path::Path,
    time::Instant,
};

use log::debug;

/// The magic number that identifies a valid XYZ binary file.
const XYZ_MAGIC: &[u8] = b"XYZB";

/// A single record of an observed laser data point needed by the algorithms.
#[derive(Debug, Clone, PartialEq)]
pub struct XyzRecord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub classification: u8,
    pub number_of_returns: u8,
    pub return_number: u8,
}

impl FromToBytes for XyzRecord {
    fn from_bytes<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let x = f64::from_bytes(reader)?;
        let y = f64::from_bytes(reader)?;
        let z = f64::from_bytes(reader)?;

        let mut buff = [0; 3];
        reader.read_exact(&mut buff)?;
        let classification = buff[0];
        let number_of_returns = buff[1];
        let return_number = buff[2];
        Ok(Self {
            x,
            y,
            z,
            classification,
            number_of_returns,
            return_number,
        })
    }

    fn to_bytes<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // write the x, y, z coordinates
        self.x.to_bytes(writer)?;
        self.y.to_bytes(writer)?;
        self.z.to_bytes(writer)?;

        // write the classification, number of returns, return number, and intensity
        writer.write_all(&[
            self.classification,
            self.number_of_returns,
            self.return_number,
        ])
    }
}

pub struct XyzInternalWriter<W: Write + Seek> {
    inner: Option<W>,
    records_written: u64,
    // for stats
    start: Option<Instant>,
}

impl XyzInternalWriter<BufWriter<File>> {
    pub fn create(path: &Path) -> std::io::Result<Self> {
        debug!("Writing records to {:?}", path);
        let file = File::create(path)?;
        Ok(Self::new(BufWriter::new(file)))
    }
}

impl<W: Write + Seek> XyzInternalWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner: Some(inner),
            records_written: 0,
            start: None,
        }
    }

    pub fn write_record(&mut self, record: &XyzRecord) -> std::io::Result<()> {
        let inner = self.inner.as_mut().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "writer has already been finished",
            )
        })?;

        // write the header (format + length) on the first write
        if self.records_written == 0 {
            self.start = Some(Instant::now());

            inner.write_all(XYZ_MAGIC)?;
            // Write the temporary number of records as all FF
            u64::MAX.to_bytes(inner)?;
        }

        record.to_bytes(inner)?;
        self.records_written += 1;
        Ok(())
    }

    pub fn finish(&mut self) -> std::io::Result<W> {
        let mut inner = self.inner.take().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "writer has already been finished",
            )
        })?;

        // seek to the beginning of the file and write the number of records
        inner.seek(std::io::SeekFrom::Start(XYZ_MAGIC.len() as u64))?;
        self.records_written.to_bytes(&mut inner)?;

        // log statistics about the written records
        if let Some(start) = self.start {
            let elapsed = start.elapsed();
            debug!(
                "Wrote {} records in {:.2?} ({:.2?}/record)",
                self.records_written,
                elapsed,
                elapsed / self.records_written as u32,
            );
        }
        Ok(inner)
    }
}

impl<W: Write + Seek> Drop for XyzInternalWriter<W> {
    fn drop(&mut self) {
        if self.inner.is_some() {
            self.finish().expect("failed to finish writer in Drop");
        }
    }
}

pub struct XyzInternalReader<R: Read> {
    inner: R,
    n_records: u64,
    records_read: u64,
    // for stats
    start: Option<Instant>,
}

impl XyzInternalReader<BufReader<File>> {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        debug!("Reading records from: {:?}", path);
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

        // read the number of records, defined by the first u64
        let n_records = u64::from_bytes(&mut inner)?;
        Ok(Self {
            inner,
            n_records,
            records_read: 0,
            start: None,
        })
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> std::io::Result<Option<XyzRecord>> {
        if self.records_read >= self.n_records {
            // TODO: log statistics about the read records
            if let Some(start) = self.start {
                let elapsed = start.elapsed();
                debug!(
                    "Read {} records in {:.2?} ({:.2?}/record)",
                    self.records_read,
                    elapsed,
                    elapsed / self.records_read as u32,
                );
            }

            return Ok(None);
        }

        if self.records_read == 0 {
            self.start = Some(Instant::now());
        }

        let record = XyzRecord::from_bytes(&mut self.inner)?;
        self.records_read += 1;
        Ok(Some(record))
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::io::xyz::XyzRecord;

    use super::*;

    #[test]
    fn test_xyz_record() {
        let record = XyzRecord {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            classification: 4,
            number_of_returns: 5,
            return_number: 6,
        };

        let mut buff = Vec::new();
        record.to_bytes(&mut buff).unwrap();
        let read_record = XyzRecord::from_bytes(&mut buff.as_slice()).unwrap();

        assert_eq!(record, read_record);
    }

    #[test]
    fn test_writer_reader_many() {
        let cursor = Cursor::new(Vec::new());
        let mut writer = XyzInternalWriter::new(cursor);

        let record = XyzRecord {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            classification: 4,
            number_of_returns: 5,
            return_number: 6,
        };

        writer.write_record(&record).unwrap();
        writer.write_record(&record).unwrap();
        writer.write_record(&record).unwrap();

        // now read the records
        let data = writer.finish().unwrap().into_inner();
        let cursor = Cursor::new(data);
        let mut reader = super::XyzInternalReader::new(cursor).unwrap();
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert_eq!(reader.next().unwrap(), None);
    }
}
