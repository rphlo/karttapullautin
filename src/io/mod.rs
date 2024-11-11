use std::io::{Read, Write};

use log::trace;

/// A single record of an observed laser data point needed by the algorithms.
#[derive(Debug, Clone, PartialEq)]
pub struct XyzRecord {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub classification: u8,
    pub number_of_returns: u8,
    pub return_number: u8,
    // pub intensity: u16,
}

impl XyzRecord {
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        // write the x, y, z coordinates
        writer.write_all(&self.x.to_ne_bytes())?;
        writer.write_all(&self.y.to_ne_bytes())?;
        writer.write_all(&self.z.to_ne_bytes())?;

        // write the classification, number of returns, return number, and intensity
        writer.write_all(&[
            self.classification,
            self.number_of_returns,
            self.return_number,
        ])?;
        // writer.write_all(&self.intensity.to_ne_bytes())?;
        Ok(())
    }

    fn read<R: Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buff = [0; 8];
        reader.read_exact(&mut buff)?;
        let x = f64::from_ne_bytes(buff);

        // let mut buff = [0; 8];
        reader.read_exact(&mut buff)?;
        let y = f64::from_ne_bytes(buff);

        // let mut buff = [0; 8];
        reader.read_exact(&mut buff)?;
        let z = f64::from_ne_bytes(buff);

        let mut buff = [0; 1];
        reader.read_exact(&mut buff)?;
        let classification = buff[0];

        reader.read_exact(&mut buff)?;
        let number_of_returns = buff[0];

        reader.read_exact(&mut buff)?;
        let return_number = buff[0];

        // let mut buff = [0; 2];
        // reader.read_exact(&mut buff)?;
        // let intensity = u16::from_ne_bytes(buff);

        Ok(Self {
            x,
            y,
            z,
            classification,
            number_of_returns,
            return_number,
            // intensity,
        })
    }
}

pub struct XyzInternalWriter<W: Write> {
    inner: W,
    n_records: u64,
    records_written: u64,
}

impl<W: Write> XyzInternalWriter<W> {
    pub fn new(inner: W, n_records: u64) -> Self {
        Self {
            inner,
            records_written: 0,
            n_records,
        }
    }

    pub fn write_record(&mut self, record: &XyzRecord) -> std::io::Result<()> {
        if self.records_written >= self.n_records {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "too many records written",
            ));
        }

        // write the header on the first write
        if self.records_written == 0 {
            self.inner.write_all(&self.n_records.to_ne_bytes()).unwrap();
        }

        record.write(&mut self.inner)?;
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
    n_records: u64,
    records_read: u64,
}

impl<R: Read> XyzInternalReader<R> {
    pub fn new(mut inner: R) -> std::io::Result<Self> {
        // read the number of records, defined by the first u64
        let mut buff = [0; 8];
        inner.read_exact(&mut buff)?;
        let n_records = u64::from_ne_bytes(buff);
        trace!("reading {} records", n_records);
        Ok(Self {
            inner,
            n_records,
            records_read: 0,
        })
    }

    pub fn next(&mut self) -> std::io::Result<Option<XyzRecord>> {
        if self.records_read >= self.n_records {
            return Ok(None);
        }

        let record = XyzRecord::read(&mut self.inner)?;
        self.records_read += 1;
        Ok(Some(record))
    }

    pub fn finish(self) -> R {
        self.inner
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::io::XyzRecord;

    use super::XyzInternalWriter;

    #[test]
    fn test_xyz_record() {
        let record = XyzRecord {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            classification: 4,
            number_of_returns: 5,
            return_number: 6,
            // intensity: 7,
        };

        let mut buff = Vec::new();
        record.write(&mut buff).unwrap();
        let read_record = XyzRecord::read(&mut buff.as_slice()).unwrap();

        assert_eq!(record, read_record);
    }

    #[test]
    fn test_writer_reader_many() {
        let cursor = Cursor::new(Vec::new());
        let mut writer = XyzInternalWriter::new(cursor, 3);

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
        let cursor = Cursor::new(writer.finish().into_inner());
        let mut reader = super::XyzInternalReader::new(cursor).unwrap();
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert_eq!(reader.next().unwrap().unwrap(), record);
        assert_eq!(reader.next().unwrap(), None);
    }
}
