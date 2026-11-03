use crate::card::{Cover, CoverFormat};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::{self, FilterType};
use image::{DynamicImage, ImageReader, Limits, Rgb, RgbImage};
use std::io::Cursor;

const W: u32 = 300;
const H: u32 = 450;
const JPEG_QUALITY: u8 = 75;
const MAX_SIDE: u32 = 4096;

pub fn prepare(bytes: &[u8]) -> Option<Cover> {
    let flat = flatten(&centre_crop(decode(bytes)?));
    let resized = imageops::resize(&flat, W, H, FilterType::Triangle);
    let mut out = Vec::new();
    JpegEncoder::new_with_quality(&mut out, JPEG_QUALITY)
        .encode_image(&resized)
        .ok()?;
    Some(Cover {
        format: CoverFormat::Jpeg,
        bytes: out,
    })
}

fn decode(bytes: &[u8]) -> Option<DynamicImage> {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    reader.limits(limits);
    reader.decode().ok()
}

fn centre_crop(img: DynamicImage) -> DynamicImage {
    let (w, h) = (img.width(), img.height());
    let (crop_w, crop_h) = if w * H > h * W {
        ((h * W / H).max(1), h)
    } else {
        (w, (w * H / W).max(1))
    };
    img.crop_imm((w - crop_w) / 2, (h - crop_h) / 2, crop_w, crop_h)
}

fn flatten(img: &DynamicImage) -> RgbImage {
    let rgba = img.to_rgba8();
    RgbImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let [r, g, b, a] = rgba.get_pixel(x, y).0;
        let over_white = |c: u8| ((c as u32 * a as u32 + 255 * (255 - a as u32)) / 255) as u8;
        Rgb([over_white(r), over_white(g), over_white(b)])
    })
}
