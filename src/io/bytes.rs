/// Trait defining how to read and write a value from a byte stream.
pub trait FromToBytes: Sized {
    /// Read a value from a byte stream.
    fn from_bytes<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self>;

    /// Write a value to a byte stream.
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
        let mut buff = [0; usize::BITS as usize / 8];
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_u64() {
        let mut buff = Vec::new();
        42u64.to_bytes(&mut buff).unwrap();
        assert_eq!(u64::from_bytes(&mut buff.as_slice()).unwrap(), 42);
    }

    #[test]
    fn test_f64() {
        let mut buff = Vec::new();
        42.0f64.to_bytes(&mut buff).unwrap();
        assert_eq!(f64::from_bytes(&mut buff.as_slice()).unwrap(), 42.0);
    }

    #[test]
    fn test_usize() {
        let mut buff = Vec::new();
        42usize.to_bytes(&mut buff).unwrap();
        assert_eq!(usize::from_bytes(&mut buff.as_slice()).unwrap(), 42);
    }
}
