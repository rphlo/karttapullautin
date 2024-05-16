use skia_safe::{
    surfaces, Color, Data, EncodedImageFormat, Image, Paint, PaintCap, PaintStyle, Path,
    PathEffect, Surface,
};
use std::fs::File;
use std::io::Write;
use std::mem;

pub struct Canvas {
    surface: Surface,
    path: Path,
    paint: Paint,
}

impl Canvas {
    pub fn new(width: i32, height: i32) -> Canvas {
        let mut surface = surfaces::raster_n32_premul((width, height)).expect("surface");
        let path = Path::new();
        let mut paint = Paint::default();
        paint.set_color(Color::BLACK);
        paint.set_anti_alias(false);
        paint.set_stroke_width(1.0);
        surface.canvas().clear(0x00000000);
        Canvas {
            surface,
            path,
            paint,
        }
    }

    #[inline]
    pub fn set_line_width(&mut self, width: f32) {
        self.paint.set_stroke_width(width);
    }

    #[inline]
    pub fn set_color(&mut self, rgb: (u8, u8, u8)) {
        self.paint.set_color(Color::from_rgb(rgb.0, rgb.1, rgb.2));
    }

    #[inline]
    pub fn set_transparent_color(&mut self) {
        self.paint.set_blend_mode(skia_safe::BlendMode::SrcIn);
        self.paint.set_color(Color::TRANSPARENT);
    }

    #[inline]
    pub fn set_stroke_cap_round(&mut self) {
        self.paint.set_stroke_cap(PaintCap::Round);
    }

    #[inline]
    pub fn unset_stroke_cap(&mut self) {
        self.paint.set_stroke_cap(PaintCap::Butt);
    }

    #[inline]
    pub fn set_dash(&mut self, interval_on: f32, interval_off: f32) {
        self.paint
            .set_path_effect(PathEffect::dash(&[interval_on, interval_off], 0.0));
    }

    #[inline]
    pub fn unset_dash(&mut self) {
        self.paint.set_path_effect(None);
    }

    #[inline]
    pub fn save(&mut self) {
        self.canvas().save();
    }

    #[inline]
    pub fn draw_polyline(&mut self, pts: &[(f32, f32)]) {
        let new_path = Path::new();
        let _ = mem::replace(&mut self.path, new_path);
        self.paint.set_style(PaintStyle::Stroke);
        self.path.move_to((pts[0].0, pts[0].1));
        for pt in pts.iter() {
            self.path.line_to((pt.0, pt.1));
        }
        self.surface.canvas().draw_path(&self.path, &self.paint);
        self.save();
    }

    #[inline]
    pub fn draw_closed_polyline(&mut self, pts: &[(f32, f32)]) {
        let new_path = Path::new();
        let _ = mem::replace(&mut self.path, new_path);
        self.paint.set_style(PaintStyle::Stroke);
        self.path.move_to((pts[0].0, pts[0].1));
        for pt in pts.iter() {
            self.path.line_to((pt.0, pt.1));
        }
        self.surface.canvas().draw_path(&self.path, &self.paint);
        self.save();
    }

    #[inline]
    pub fn draw_filled_polygon(&mut self, apts: &Vec<Vec<(f32, f32)>>) {
        let new_path = Path::new();
        let _ = mem::replace(&mut self.path, new_path);
        self.paint.set_stroke_width(1.0);
        self.paint.set_style(PaintStyle::StrokeAndFill);
        for pts in apts {
            self.path.move_to((pts[0].0, pts[0].1));
            for pt in pts.iter() {
                self.path.line_to((pt.0, pt.1));
            }
        }
        self.surface.canvas().draw_path(&self.path, &self.paint);
        self.save();
    }

    #[inline]
    pub fn data(&mut self) -> Data {
        let image = self.surface.image_snapshot();
        let mut context = self.surface.direct_context();
        image
            .encode(context.as_mut(), EncodedImageFormat::PNG, None)
            .unwrap()
    }

    #[inline]
    pub fn image(&mut self) -> skia_safe::image::Image {
        self.surface.image_snapshot()
    }

    #[inline]
    fn canvas(&mut self) -> &skia_safe::Canvas {
        self.surface.canvas()
    }

    #[inline]
    pub fn save_as(&mut self, filename: &str) {
        let d = self.data();
        let mut file = File::create(filename).unwrap();
        let bytes = d.as_bytes();
        file.write_all(bytes).unwrap();
    }

    #[inline]
    pub fn load_from(filename: &str) -> Canvas {
        let data = Data::from_filename(filename).unwrap();
        let image = Image::from_encoded(data).unwrap();
        let mut c = Canvas::new(image.width(), image.height());
        c.draw_image(image);
        c
    }

    #[inline]
    pub fn draw_image(&mut self, image: Image) {
        self.surface.canvas().draw_image(image, (0, 0), None);
    }

    #[inline]
    pub fn overlay(&mut self, other_canvas: &mut Canvas, x: f32, y: f32) {
        self.surface
            .canvas()
            .draw_image(other_canvas.image(), (x, y), None);
    }
}
