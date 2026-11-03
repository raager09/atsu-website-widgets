use atsu_shelf::card::{Book, Cover, CoverFormat, Medium, Options, render};
use atsu_shelf::cover::prepare;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat, Rgb, RgbImage, Rgba, RgbaImage};
use std::io::Cursor;

const FIXTURE: &[u8] = include_bytes!("fixtures/cover.png");
const RED: [u8; 3] = [220, 30, 30];
const BLUE: [u8; 3] = [30, 30, 220];
const SVG_BUDGET: usize = 400 * 1024;

fn png(img: DynamicImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .unwrap();
    bytes
}

fn rgb_png(w: u32, h: u32, f: impl Fn(u32, u32) -> [u8; 3]) -> Vec<u8> {
    png(RgbImage::from_fn(w, h, |x, y| Rgb(f(x, y))).into())
}

fn decode(cover: &Cover) -> RgbImage {
    image::load_from_memory(&cover.bytes).unwrap().to_rgb8()
}

fn assert_close(got: [u8; 3], want: [u8; 3]) {
    for (g, w) in got.iter().zip(want) {
        assert!(g.abs_diff(w) <= 24, "got {got:?}, want {want:?}");
    }
}

#[test]
fn output_is_a_300_by_450_jpeg() {
    let cover = prepare(FIXTURE).unwrap();
    assert_eq!(cover.format, CoverFormat::Jpeg);
    assert_eq!(
        image::guess_format(&cover.bytes).unwrap(),
        ImageFormat::Jpeg
    );
    assert_eq!(decode(&cover).dimensions(), (300, 450));
}

#[test]
fn jpeg_input_is_accepted() {
    let img = RgbImage::from_pixel(230, 345, Rgb(BLUE));
    let mut bytes = Vec::new();
    JpegEncoder::new(&mut bytes).encode_image(&img).unwrap();
    let cover = prepare(&bytes).unwrap();
    assert_eq!(decode(&cover).dimensions(), (300, 450));
}

#[test]
fn wide_images_are_centre_cropped_not_stretched() {
    let bytes = rgb_png(
        900,
        450,
        |x, _| {
            if (300..600).contains(&x) { BLUE } else { RED }
        },
    );
    let out = decode(&prepare(&bytes).unwrap());
    for x in [8, 150, 291] {
        assert_close(out.get_pixel(x, 225).0, BLUE);
    }
}

#[test]
fn tall_images_are_centre_cropped_not_stretched() {
    let bytes = rgb_png(
        300,
        1350,
        |_, y| {
            if (450..900).contains(&y) { BLUE } else { RED }
        },
    );
    let out = decode(&prepare(&bytes).unwrap());
    for y in [8, 225, 441] {
        assert_close(out.get_pixel(150, y).0, BLUE);
    }
}

#[test]
fn shrinking_averages_fine_detail() {
    let bytes = rgb_png(1200, 1800, |x, y| {
        if (x + y) % 2 == 0 {
            [255, 255, 255]
        } else {
            [0, 0, 0]
        }
    });
    let out = decode(&prepare(&bytes).unwrap());
    for (x, y) in [(150, 225), (40, 60), (260, 400)] {
        assert_close(out.get_pixel(x, y).0, [128, 128, 128]);
    }
}

#[test]
fn transparency_becomes_white() {
    let img = RgbaImage::from_pixel(300, 450, Rgba([0, 0, 0, 0]));
    let out = decode(&prepare(&png(img.into())).unwrap());
    assert_close(out.get_pixel(150, 225).0, [255, 255, 255]);
}

#[test]
fn extreme_shapes_still_give_a_full_size_cover() {
    for (w, h) in [(1, 1), (1, 3000), (3000, 1)] {
        let cover = prepare(&rgb_png(w, h, |_, _| BLUE)).expect("a cover");
        assert_eq!(decode(&cover).dimensions(), (300, 450), "{w}x{h}");
    }
}

#[test]
fn bad_bytes_give_none() {
    assert_eq!(prepare(b""), None);
    assert_eq!(prepare(b"<html>not an image</html>"), None);
    assert_eq!(prepare(&FIXTURE[..200]), None);
}

#[test]
fn oversized_images_are_refused() {
    assert_eq!(prepare(&rgb_png(5000, 1, |_, _| BLUE)), None);
    assert_eq!(prepare(&rgb_png(1, 5000, |_, _| BLUE)), None);
}

#[test]
fn six_covers_stay_under_the_svg_budget() {
    let books: Vec<Book> = (0..6)
        .map(|i| Book {
            series_id: i.to_string(),
            title: format!("Series {i}"),
            url: None,
            medium: Medium::Comic,
            kind: "Manga".to_string(),
            chapter: Some("7".to_string()),
            completed: false,
            cover: Some(prepare(FIXTURE).unwrap()),
        })
        .collect();
    let size = render(&books, &Options::default()).len();
    assert!(size < SVG_BUDGET, "{size} bytes");
}
