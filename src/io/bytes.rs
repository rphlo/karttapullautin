/// Trait defining how to read and write a value from a byte stream.
pub trait FromToBytes: Sized {
    /// Read a value from a byte stream.
    fn from_bytes<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self>;

    fn to_bytes<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()>;
}

impl FromToBytes for f64 {
    fn from_bytes<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buff = [0; 8];
        reader.read_exact(&mut buff)?;
        Ok(f64::from_ne_bytes(buff))
    }

    fn to_bytes<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.to_ne_bytes())
    }
}

impl FromToBytes for usize {
    fn from_bytes<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buff = [0; 8];
        reader.read_exact(&mut buff)?;
        Ok(usize::from_ne_bytes(buff))
    }
    fn to_bytes<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.to_ne_bytes())
    }
}

impl FromToBytes for u64 {
    fn from_bytes<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut buff = [0; 8];
        reader.read_exact(&mut buff)?;
        Ok(u64::from_ne_bytes(buff))
    }
    fn to_bytes<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&self.to_ne_bytes())
    }
}
