use std::ops::Range;

pub type Rgb = [u8; 3];

pub const fn rgb(hex: u32) -> Rgb {
    [(hex >> 16) as u8, (hex >> 8) as u8, hex as u8]
}

/// An RGB pixel buffer. Drawing outside it is silently clipped, so callers can
/// position things partly off-canvas without checking.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas {
    width: i32,
    height: i32,
    pixels: Vec<Rgb>,
}

impl Canvas {
    pub fn new(width: i32, height: i32, fill: Rgb) -> Self {
        let (width, height) = (width.max(0), height.max(0));
        Self {
            width,
            height,
            pixels: vec![fill; width as usize * height as usize],
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn pixels(&self) -> &[Rgb] {
        &self.pixels
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Rgb> {
        self.index(x, y).map(|i| self.pixels[i])
    }

    pub fn set(&mut self, x: i32, y: i32, color: Rgb) {
        if let Some(i) = self.index(x, y) {
            self.pixels[i] = color;
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgb) {
        let (xs, ys) = self.clip(x, y, w, h);
        for py in ys {
            for px in xs.clone() {
                self.set(px, py, color);
            }
        }
    }

    /// Blends `color` over the pixel. Alpha snaps to eighths, so any one pair
    /// of colors can only produce nine results, which keeps the palette small.
    pub fn blend(&mut self, x: i32, y: i32, color: Rgb, alpha: f32) {
        let Some(i) = self.index(x, y) else { return };
        let level = (alpha.clamp(0.0, 1.0) * 8.0).round() as u16;
        let dst = &mut self.pixels[i];
        for (d, s) in dst.iter_mut().zip(color) {
            *d = ((*d as u16 * (8 - level) + s as u16 * level + 4) / 8) as u8;
        }
    }

    pub fn blend_rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: Rgb, alpha: f32) {
        let (xs, ys) = self.clip(x, y, w, h);
        for py in ys {
            for px in xs.clone() {
                self.blend(px, py, color, alpha);
            }
        }
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        let in_bounds = (0..self.width).contains(&x) && (0..self.height).contains(&y);
        in_bounds.then(|| y as usize * self.width as usize + x as usize)
    }

    fn clip(&self, x: i32, y: i32, w: i32, h: i32) -> (Range<i32>, Range<i32>) {
        (
            x.max(0)..x.saturating_add(w).min(self.width),
            y.max(0)..y.saturating_add(h).min(self.height),
        )
    }
}
