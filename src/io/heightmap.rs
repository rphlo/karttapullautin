use crate::vec2d::Vec2D;

use super::bytes::FromToBytes;

/// Simple container of a rectangular heightmap
#[derive(Debug, Clone, PartialEq)]
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
    /// Get the maximum x-coordinate of the heightmap
    pub fn maxx(&self) -> f64 {
        self.xoffset + self.scale * (self.grid.width().saturating_sub(1)) as f64
    }

    /// Get the maximum y-coordinate of the heightmap
    pub fn maxy(&self) -> f64 {
        self.yoffset + self.scale * (self.grid.height().saturating_sub(1)) as f64
    }

    pub fn iter(&self) -> impl Iterator<Item = (f64, f64, f64)> + '_ {
        self.grid.iter().map(|(x, y, v)| {
            (
                self.xoffset + self.scale * x as f64,
                self.yoffset + self.scale * y as f64,
                v,
            )
        })
    }
}

impl HeightMap {
    /// Helper for easily reading a HeightMap from a file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        HeightMap::from_bytes(&mut reader)
    }

    /// Helper for easily writing a HeightMap to a file
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> std::io::Result<()> {
        let file = std::fs::File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        self.to_bytes(&mut writer)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bytes() {
        let mut data = Vec2D::new(2, 2, 0.0);
        data[(0, 0)] = 1.0;
        data[(1, 0)] = 2.0;
        data[(0, 1)] = 3.0;
        data[(1, 1)] = 4.0;

        let heightmap = super::HeightMap {
            xoffset: 3.0,
            yoffset: -5.0,
            scale: 1.5,
            grid: data,
        };

        let mut bytes = Vec::new();
        heightmap.to_bytes(&mut bytes).unwrap();
        let heightmap2 = super::HeightMap::from_bytes(&mut bytes.as_slice()).unwrap();

        assert_eq!(heightmap, heightmap2);
    }
}
