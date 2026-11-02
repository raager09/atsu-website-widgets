use atsu_shelf::canvas::{Canvas, Rgb};
use atsu_shelf::font::{draw_char, draw_text, draw_text_clipped, glyph, has_glyph, text_width};
use std::collections::HashSet;

const BG: Rgb = [0, 0, 0];
const INK: Rgb = [255, 255, 255];
const REQUIRED: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-./:!?',&+()#";

#[test]
fn every_required_character_has_a_glyph() {
    for c in REQUIRED.chars() {
        assert!(has_glyph(c), "missing glyph for {c:?}");
    }
}

#[test]
fn other_characters_have_no_glyph() {
    for c in [' ', 'a', 'z', 'é', 'あ', '📚', '_', '"'] {
        assert!(!has_glyph(c), "unexpected glyph for {c:?}");
        assert_eq!(glyph(c), None);
    }
}

#[test]
fn glyphs_are_distinct_visible_and_three_bits_wide() {
    let mut seen = HashSet::new();
    for c in REQUIRED.chars() {
        let rows = glyph(c).unwrap();
        assert!(rows.iter().any(|r| *r != 0), "{c:?} is blank");
        assert!(rows.iter().all(|r| *r <= 0b111), "{c:?} is too wide");
        assert!(seen.insert(rows), "{c:?} duplicates another glyph");
    }
}

#[test]
fn text_width_is_four_per_character_minus_trailing_gap() {
    assert_eq!(text_width(""), 0);
    assert_eq!(text_width("A"), 3);
    assert_eq!(text_width("AB"), 7);
    assert_eq!(text_width("NOW READING"), 43);
}

#[test]
fn text_width_counts_characters_not_bytes() {
    assert_eq!(text_width("あé"), 7);
}

#[test]
fn draw_char_plots_the_glyph_bits() {
    let mut c = Canvas::new(5, 7, BG);
    draw_char(&mut c, 1, 1, 'T', INK);
    let rows = glyph('T').unwrap();
    for (row, bits) in rows.iter().enumerate() {
        for col in 0..3 {
            let lit = bits & (0b100 >> col) != 0;
            let want = if lit { INK } else { BG };
            assert_eq!(c.get(1 + col, 1 + row as i32), Some(want), "({col},{row})");
        }
    }
}

#[test]
fn draw_text_places_characters_four_pixels_apart() {
    let mut joined = Canvas::new(16, 5, BG);
    draw_text(&mut joined, 0, 0, "AB", INK);
    let mut parts = Canvas::new(16, 5, BG);
    draw_char(&mut parts, 0, 0, 'A', INK);
    draw_char(&mut parts, 4, 0, 'B', INK);
    assert_eq!(joined, parts);
}

#[test]
fn undrawable_characters_are_blank_but_still_advance() {
    let mut with_gap = Canvas::new(16, 5, BG);
    draw_text(&mut with_gap, 0, 0, "AあB", INK);
    let mut parts = Canvas::new(16, 5, BG);
    draw_char(&mut parts, 0, 0, 'A', INK);
    draw_char(&mut parts, 8, 0, 'B', INK);
    assert_eq!(with_gap, parts);
}

#[test]
fn clipped_draw_only_touches_columns_in_range() {
    let mut full = Canvas::new(24, 5, BG);
    draw_text(&mut full, 0, 0, "HELLO", INK);
    let mut clipped = Canvas::new(24, 5, BG);
    draw_text_clipped(&mut clipped, 0, 0, "HELLO", INK, 2..9);
    for y in 0..5 {
        for x in 0..24 {
            let want = if (2..9).contains(&x) {
                full.get(x, y)
            } else {
                Some(BG)
            };
            assert_eq!(clipped.get(x, y), want, "({x},{y})");
        }
    }
}

#[test]
fn clipped_draw_accepts_text_scrolled_off_the_left() {
    let mut scrolled = Canvas::new(8, 5, BG);
    draw_text_clipped(&mut scrolled, -4, 0, "AB", INK, 0..8);
    let mut plain = Canvas::new(8, 5, BG);
    draw_char(&mut plain, 0, 0, 'B', INK);
    assert_eq!(scrolled, plain);
}

#[test]
fn text_past_the_canvas_edge_does_not_panic() {
    let mut c = Canvas::new(4, 4, BG);
    draw_text(&mut c, 2, 2, "WWWWW", INK);
    draw_text(&mut c, -100, -100, "WWWWW", INK);
    draw_text_clipped(&mut c, 500, 500, "WWWWW", INK, 0..4);
}
