use crate::vec2d::Vec2D;

use super::bytes::FromToBytes;

/// Simple container of a rectangular heightmap
#[derive(Debug, Clone)]
pub struct HeightMap {
    /// Offset to add to the x-component to get the cell coordinate.
    pub xoffset: f64,
    /// Offset to add to the y-component to get the cell coordinate.
    pub yoffset: f64,
    /// Scale to apply to get the cell coordinate.
    pub scale: f64,

    /// The actual grid data
    pub grid: Vec2D<f64>,
}

impl HeightMap {
    pub fn minx(&self) -> f64 {
        self.xoffset
    }
    pub fn miny(&self) -> f64 {
        self.yoffset
    }
    pub fn maxx(&self) -> f64 {
        self.xoffset + self.scale * self.grid.width() as f64
    }
    pub fn maxy(&self) -> f64 {
        self.yoffset + self.scale * self.grid.height() as f64
    }

    pub fn iter_values(&self) -> impl Iterator<Item = (f64, f64, f64)> + '_ {
        self.grid.iter_idx().map(|(x, y, v)| {
            (
                self.xoffset + self.scale * x as f64,
                self.yoffset + self.scale * y as f64,
                v,
            )
        })
    }
}

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
            grid: data,
        })
    }

    fn to_bytes<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.xoffset.to_bytes(writer)?;
        self.yoffset.to_bytes(writer)?;
        self.scale.to_bytes(writer)?;
        self.grid.to_bytes(writer)
    }
}
