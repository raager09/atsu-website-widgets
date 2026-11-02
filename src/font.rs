use crate::canvas::{Canvas, Rgb};
use std::ops::Range;

const ADVANCE: i32 = 4;

/// The 3×5 bitmap for `c`, one `u8` per row with bit 2 as the leftmost column.
/// Only uppercase letters are covered, so callers uppercase their text first.
pub fn glyph(c: char) -> Option<[u8; 5]> {
    Some(match c {
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b110, 0b101, 0b101, 0b101, 0b101],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        'Q' => [0b010, 0b101, 0b101, 0b110, 0b011],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b110, 0b001, 0b010, 0b100, 0b111],
        '3' => [0b110, 0b001, 0b010, 0b001, 0b110],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b110, 0b001, 0b110],
        '6' => [0b011, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b110],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
        ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        '!' => [0b010, 0b010, 0b010, 0b000, 0b010],
        '?' => [0b110, 0b001, 0b010, 0b000, 0b010],
        '\'' => [0b010, 0b010, 0b000, 0b000, 0b000],
        ',' => [0b000, 0b000, 0b000, 0b010, 0b100],
        '&' => [0b010, 0b101, 0b010, 0b101, 0b011],
        '+' => [0b000, 0b010, 0b111, 0b010, 0b000],
        '(' => [0b001, 0b010, 0b010, 0b010, 0b001],
        ')' => [0b100, 0b010, 0b010, 0b010, 0b100],
        '#' => [0b101, 0b111, 0b101, 0b111, 0b101],
        _ => return None,
    })
}

pub fn has_glyph(c: char) -> bool {
    glyph(c).is_some()
}

/// Width in pixels, without the gap after the last character.
pub fn text_width(text: &str) -> i32 {
    (text.chars().count() as i32 * ADVANCE - 1).max(0)
}

pub fn draw_char(canvas: &mut Canvas, x: i32, y: i32, c: char, color: Rgb) {
    plot(canvas, x, y, c, color, &(i32::MIN..i32::MAX));
}

pub fn draw_text(canvas: &mut Canvas, x: i32, y: i32, text: &str, color: Rgb) {
    draw_text_clipped(canvas, x, y, text, color, i32::MIN..i32::MAX);
}

/// Draws `text`, touching only the columns in `clip`. Characters without a
/// glyph (including spaces) are blank but still advance.
pub fn draw_text_clipped(
    canvas: &mut Canvas,
    x: i32,
    y: i32,
    text: &str,
    color: Rgb,
    clip: Range<i32>,
) {
    let mut pen = x;
    for c in text.chars() {
        plot(canvas, pen, y, c, color, &clip);
        pen = pen.saturating_add(ADVANCE);
    }
}

fn plot(canvas: &mut Canvas, x: i32, y: i32, c: char, color: Rgb, clip: &Range<i32>) {
    let Some(rows) = glyph(c) else { return };
    for (dy, bits) in (0..).zip(rows) {
        for dx in 0..3 {
            let px = x.saturating_add(dx);
            if bits & (0b100 >> dx) != 0 && clip.contains(&px) {
                canvas.set(px, y.saturating_add(dy), color);
            }
        }
    }
}
