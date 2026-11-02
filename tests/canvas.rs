use atsu_shelf::canvas::{Canvas, Rgb, rgb};
use std::collections::HashSet;

const BLACK: Rgb = [0, 0, 0];
const WHITE: Rgb = [255, 255, 255];
const RED: Rgb = [255, 0, 0];

#[test]
fn rgb_splits_hex_into_channels() {
    assert_eq!(rgb(0x16122A), [0x16, 0x12, 0x2A]);
}

#[test]
fn new_fills_every_pixel() {
    let c = Canvas::new(4, 3, RED);
    assert_eq!((c.width(), c.height()), (4, 3));
    assert_eq!(c.pixels().len(), 12);
    assert!(c.pixels().iter().all(|p| *p == RED));
}

#[test]
fn set_then_get_round_trips() {
    let mut c = Canvas::new(4, 3, BLACK);
    c.set(3, 2, RED);
    assert_eq!(c.get(3, 2), Some(RED));
    assert_eq!(c.get(2, 2), Some(BLACK));
}

#[test]
fn out_of_bounds_is_ignored() {
    let mut c = Canvas::new(4, 3, BLACK);
    let before = c.clone();
    for (x, y) in [(-1, 0), (0, -1), (4, 0), (0, 3)] {
        c.set(x, y, RED);
        assert_eq!(c.get(x, y), None);
    }
    assert_eq!(c, before);
}

#[test]
fn rect_fills_exactly_its_area() {
    let mut c = Canvas::new(4, 4, BLACK);
    c.rect(1, 1, 2, 2, RED);
    let reds = c.pixels().iter().filter(|p| **p == RED).count();
    assert_eq!(reds, 4);
    assert_eq!(c.get(1, 1), Some(RED));
    assert_eq!(c.get(2, 2), Some(RED));
    assert_eq!(c.get(3, 3), Some(BLACK));
}

#[test]
fn rect_clips_to_the_canvas() {
    let mut c = Canvas::new(4, 4, BLACK);
    c.rect(-2, -2, 3, 3, RED);
    let reds = c.pixels().iter().filter(|p| **p == RED).count();
    assert_eq!(reds, 1);
    assert_eq!(c.get(0, 0), Some(RED));
}

#[test]
fn blend_endpoints_leave_or_replace() {
    let mut c = Canvas::new(1, 1, BLACK);
    c.blend(0, 0, WHITE, 0.0);
    assert_eq!(c.get(0, 0), Some(BLACK));
    c.blend(0, 0, WHITE, 1.0);
    assert_eq!(c.get(0, 0), Some(WHITE));
}

#[test]
fn blend_half_is_the_midpoint() {
    let mut c = Canvas::new(1, 1, BLACK);
    c.blend(0, 0, WHITE, 0.5);
    assert_eq!(c.get(0, 0), Some([128, 128, 128]));
}

#[test]
fn blend_snaps_alpha_to_eighths() {
    let mut exact = Canvas::new(1, 1, BLACK);
    exact.blend(0, 0, WHITE, 0.5);
    let mut near = Canvas::new(1, 1, BLACK);
    near.blend(0, 0, WHITE, 0.52);
    assert_eq!(exact, near);
}

#[test]
fn blend_sweep_yields_at_most_nine_colors() {
    let mut seen = HashSet::new();
    for i in 0..=1000 {
        let mut c = Canvas::new(1, 1, BLACK);
        c.blend(0, 0, WHITE, i as f32 / 1000.0);
        seen.insert(c.get(0, 0).unwrap());
    }
    assert_eq!(seen.len(), 9);
}

#[test]
fn blend_clamps_alpha_outside_zero_to_one() {
    let mut c = Canvas::new(2, 1, BLACK);
    c.blend(0, 0, WHITE, -3.0);
    c.blend(1, 0, WHITE, 3.0);
    assert_eq!(c.get(0, 0), Some(BLACK));
    assert_eq!(c.get(1, 0), Some(WHITE));
}

#[test]
fn blend_rect_clips_and_blends() {
    let mut c = Canvas::new(2, 2, BLACK);
    c.blend_rect(1, 1, 5, 5, WHITE, 1.0);
    assert_eq!(c.get(1, 1), Some(WHITE));
    assert_eq!(c.get(0, 0), Some(BLACK));
}
