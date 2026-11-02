use crate::canvas::{Canvas, Rgb, rgb};
use crate::card::{Book, Cover, CoverFormat, Medium};
use crate::font::glyph;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};

const W: i32 = 300;
const H: i32 = 450;

pub fn books() -> Vec<Book> {
    vec![
        book(
            "The Lantern Keeper's Ledger",
            Medium::Comic,
            "Manga",
            "46",
            false,
            Some(art(0x1B2A49, 0xE8743B, 0xFFD166, 'L', 0)),
        ),
        book(
            "Salt & Cinder",
            Medium::Comic,
            "Manhwa",
            "112.7",
            false,
            Some(art(0x12343B, 0x2D6E7E, 0xC6E2E9, 'S', 1)),
        ),
        book(
            "Cartographer of the Seventh Floor Who Keeps Redrawing the Same Hallway",
            Medium::Comic,
            "Manhua",
            "7",
            false,
            Some(art(0x3B1F47, 0xB5446E, 0xF4C2C2, 'C', 2)),
        ),
        book(
            "夜更けの図書館",
            Medium::Novel,
            "Other",
            "32",
            true,
            Some(art(0x101820, 0x3A506B, 0xF2AA4C, '7', 3)),
        ),
        book(
            "Moonlit Tower Errands",
            Medium::Comic,
            "Other",
            "53",
            false,
            Some(art(0x23395B, 0x8EA8C3, 0xEDF2F4, 'M', 4)),
        ),
        book(
            "Tidewater Apprentice",
            Medium::Comic,
            "OEL",
            "9",
            false,
            None,
        ),
    ]
}

fn book(
    title: &str,
    medium: Medium,
    kind: &str,
    chapter: &str,
    completed: bool,
    cover: Option<Cover>,
) -> Book {
    Book {
        series_id: format!("demo-{title}"),
        title: title.to_string(),
        url: None,
        medium,
        kind: kind.to_string(),
        chapter: Some(chapter.to_string()),
        completed,
        cover,
    }
}

fn art(top: u32, bottom: u32, accent: u32, initial: char, pattern: u32) -> Cover {
    let (top, bottom, accent) = (rgb(top), rgb(bottom), rgb(accent));
    let mut c = Canvas::new(W, H, top);
    for band in 0..15 {
        c.blend_rect(0, band * 30, W, 30, bottom, band as f32 / 14.0);
    }
    match pattern {
        0 => disc(&mut c, 210, 110, 70, accent, 1.0),
        1 => {
            for i in 0..7 {
                diagonal(&mut c, i * 90 - 150, 26, accent, 0.25);
            }
        }
        2 => {
            for r in [150, 110, 70, 30] {
                disc(&mut c, 150, 170, r, accent, 0.25);
            }
        }
        3 => {
            for y in 0..6 {
                for x in 0..6 {
                    disc(&mut c, 30 + x * 48, 30 + y * 48, 9, accent, 0.75);
                }
            }
        }
        _ => {
            disc(&mut c, 80, 90, 44, accent, 1.0);
            disc(&mut c, 98, 80, 40, top, 1.0);
            ridge(&mut c, 300, top);
        }
    }
    letter(&mut c, 24, 270, initial, 20, accent);
    encode(&c)
}

fn disc(c: &mut Canvas, cx: i32, cy: i32, r: i32, color: Rgb, alpha: f32) {
    for y in cy - r..=cy + r {
        for x in cx - r..=cx + r {
            if (x - cx).pow(2) + (y - cy).pow(2) <= r * r {
                c.blend(x, y, color, alpha);
            }
        }
    }
}

fn diagonal(c: &mut Canvas, start: i32, thickness: i32, color: Rgb, alpha: f32) {
    for y in 0..H {
        c.blend_rect(start + y, y, thickness, 1, color, alpha);
    }
}

fn ridge(c: &mut Canvas, base: i32, color: Rgb) {
    for x in 0..W {
        let top = base - (x % 120 - 60).abs();
        c.rect(x, top, 1, H - top, color);
    }
}

fn letter(c: &mut Canvas, x: i32, y: i32, ch: char, scale: i32, color: Rgb) {
    let Some(rows) = glyph(ch) else { return };
    for (row, bits) in (0..).zip(rows) {
        for col in 0..3 {
            if bits & (0b100 >> col) != 0 {
                c.rect(x + col * scale, y + row * scale, scale, scale, color);
            }
        }
    }
}

fn encode(c: &Canvas) -> Cover {
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(
            &c.pixels().concat(),
            c.width() as u32,
            c.height() as u32,
            ExtendedColorType::Rgb8,
        )
        .expect("encoding a PNG in memory cannot fail");
    Cover {
        format: CoverFormat::Png,
        bytes,
    }
}
