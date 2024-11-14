use crate::vec2d::Vec2D;

use super::bytes::FromToBytes;

/// Simple container of a rectangular heightmap
pub struct HeightMap {
    /// Offset to add to the x-component to get the cell coordinate.
    pub xoffset: f64,
    /// Offset to add to the y-component to get the cell coordinate.
    pub yoffset: f64,
    /// Scale to apply to get the cell coordinate.
    pub scale: f64,

    /// The actual grid data
    pub data: Vec2D<f64>,
}

impl HeightMap {}

impl FromToBytes for HeightMap {
    fn from_bytes<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let xoffset = f64::from_bytes(reader)?;
        let yoffset = f64::from_bytes(reader)?;
        let scale = f64::from_bytes(reader)?;

        let data = Vec2D::from_bytes(reader)?;

        Ok(HeightMap {
            xoffset,
            yoffset,
            scale,
            data,
        })
    }

    fn to_bytes<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.xoffset.to_bytes(writer)?;
        self.yoffset.to_bytes(writer)?;
        self.scale.to_bytes(writer)?;
        self.data.to_bytes(writer)
    }
}
