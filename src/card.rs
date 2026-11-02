use crate::base64;
use std::str::FromStr;

const COVER_W: i32 = 150;
const COVER_H: i32 = 225;
const GAP: i32 = 14;
const PAD: i32 = 16;
const COVERS_Y: i32 = 52;
const TITLE_PX: f32 = 13.0;
const HEADING_PX: f32 = 18.0;
const PILL_H: i32 = 18;

const NEUTRAL: &str = "#6e7681";
const LIGHT_HEADING: &str = "#24292f";
const LIGHT_TITLE: &str = "#57606a";
const DARK_HEADING: &str = "#e6edf3";
const DARK_TITLE: &str = "#b4bac2";

const PLACEHOLDERS: [&str; 11] = [
    "#9E2832", "#284E8C", "#24705C", "#CC9A3E", "#6E3A84", "#7A6A4F", "#3A3A46", "#B85C3A",
    "#5C823A", "#D8768C", "#468CAC",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Medium {
    Comic,
    Novel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoverFormat {
    Png,
    Jpeg,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cover {
    pub format: CoverFormat,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Book {
    pub series_id: String,
    pub title: String,
    pub url: Option<String>,
    pub medium: Medium,
    pub kind: String,
    pub chapter: Option<String>,
    pub completed: bool,
    pub cover: Option<Cover>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Color(String);

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let hex = s.strip_prefix('#').unwrap_or("");
        let valid_hex = matches!(hex.len(), 3 | 6) && hex.chars().all(|c| c.is_ascii_hexdigit());
        if s == "transparent" || valid_hex {
            Ok(Self(s.to_string()))
        } else {
            Err(format!("{s:?} is not #rgb, #rrggbb or transparent"))
        }
    }
}

impl Color {
    fn is_dark(&self) -> Option<bool> {
        let hex = self.0.strip_prefix('#')?;
        let channel = |i: usize| -> f32 {
            let (start, len) = if hex.len() == 3 { (i, 1) } else { (i * 2, 2) };
            let value = u32::from_str_radix(&hex[start..start + len], 16).unwrap_or(0);
            (if len == 1 { value * 17 } else { value }) as f32
        };
        Some(0.299 * channel(0) + 0.587 * channel(1) + 0.114 * channel(2) < 128.0)
    }
}

#[derive(Clone, Debug)]
pub struct Options {
    pub heading: String,
    pub cards: usize,
    pub heading_color: Option<Color>,
    pub title_color: Option<Color>,
    pub background: Option<Color>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            heading: "Currently reading".to_string(),
            cards: 6,
            heading_color: None,
            title_color: None,
            background: None,
        }
    }
}

pub fn render(books: &[Book], opts: &Options) -> String {
    let books = &books[..books.len().min(opts.cards)];
    let columns = if books.is_empty() {
        2
    } else {
        books.len() as i32
    };
    let width = PAD * 2 + columns * COVER_W + (columns - 1) * GAP;
    let has_cover = books.iter().any(|b| b.cover.is_some());
    let height = match (books.is_empty(), has_cover) {
        (true, _) => 92,
        (false, true) => 345,
        (false, false) => 331,
    };

    let heading = wrap_title(
        &clean(&opts.heading),
        (width - PAD * 2) as f32,
        HEADING_PX,
        1,
    )
    .pop()
    .unwrap_or_default();
    let names: Vec<String> = books.iter().map(|b| clean(&b.title)).collect();

    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" class=\"atsu-shelf\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\" role=\"img\">\n"
    );
    svg.push_str(&format!(
        "<title>{}</title>\n",
        esc(&format!("{heading}: {}", names.join(", ")))
    ));
    svg.push_str(&style(opts));
    svg.push_str(&format!(
        "<rect class=\"bg\" width=\"{width}\" height=\"{height}\"/>\n"
    ));
    svg.push_str(&format!(
        "<text class=\"h\" x=\"{PAD}\" y=\"36\">{}</text>\n",
        esc(&heading)
    ));

    if books.is_empty() {
        svg.push_str(&format!(
            "<text class=\"t\" x=\"{PAD}\" y=\"70\">Nothing yet</text>\n"
        ));
    }
    for (i, book) in books.iter().enumerate() {
        svg.push_str(&card(book, i as i32));
    }
    if has_cover {
        svg.push_str(&format!(
            "<text class=\"c\" x=\"{}\" y=\"335\">Covers: AniList</text>\n",
            width - PAD
        ));
    }
    svg.push_str("</svg>\n");
    svg
}

pub fn wrap_title(title: &str, max_px: f32, font_px: f32, max_lines: usize) -> Vec<String> {
    let fits = |s: &str| text_px(s, font_px) <= max_px;
    let mut lines = Vec::new();
    let mut line = String::new();

    for word in title.split_whitespace() {
        let joined = if line.is_empty() {
            word.to_string()
        } else {
            format!("{line} {word}")
        };
        if fits(&joined) {
            line = joined;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        for c in word.chars() {
            line.push(c);
            if !fits(&line) && line.chars().count() > 1 {
                line.pop();
                lines.push(std::mem::take(&mut line));
                line.push(c);
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }

    if lines.len() > max_lines {
        lines.truncate(max_lines);
        if let Some(last) = lines.last_mut() {
            while !last.is_empty() && !fits(&format!("{last}…")) {
                last.pop();
            }
            last.truncate(last.trim_end().len());
            last.push('…');
        }
    }
    lines
}

fn style(opts: &Options) -> String {
    let (base_heading, base_title) = match opts.background.as_ref().and_then(Color::is_dark) {
        Some(true) => (DARK_HEADING, DARK_TITLE),
        Some(false) => (LIGHT_HEADING, LIGHT_TITLE),
        None => (NEUTRAL, NEUTRAL),
    };
    let chosen = |c: &Option<Color>, fallback: &str| {
        c.as_ref().map_or(fallback.to_string(), |c| c.0.clone())
    };

    format!(
        "<style>\n\
         .atsu-shelf{{--h:var(--atsu-shelf-heading,{});--t:var(--atsu-shelf-title,{});--b:var(--atsu-shelf-background,{});font-family:-apple-system,BlinkMacSystemFont,\"Segoe UI\",\"Noto Sans\",Helvetica,Arial,\"Noto Sans CJK JP\",sans-serif}}\n\
         .atsu-shelf .bg{{fill:var(--b)}}\n\
         .atsu-shelf .h{{fill:var(--h);font-size:{HEADING_PX}px;font-weight:600}}\n\
         .atsu-shelf .t{{fill:var(--t);font-size:{TITLE_PX}px}}\n\
         .atsu-shelf .c{{fill:var(--t);font-size:9px;opacity:.7;text-anchor:end}}\n\
         .atsu-shelf .p{{fill:#fff;font-size:11px;font-weight:600;text-anchor:middle}}\n\
         .atsu-shelf .l{{fill:#fff;font-size:64px;font-weight:700;text-anchor:middle}}\n\
         </style>\n",
        chosen(&opts.heading_color, base_heading),
        chosen(&opts.title_color, base_title),
        chosen(&opts.background, "transparent"),
    )
}

fn card(book: &Book, index: i32) -> String {
    let x = PAD + index * (COVER_W + GAP);
    let title = clean(&book.title);
    let mut body = cover(book, &title, index);
    body.push_str(&status_pill(book));
    body.push_str(&kind_pill(book));
    for (i, line) in (0..).zip(wrap_title(&title, COVER_W as f32, TITLE_PX, 2)) {
        body.push_str(&format!(
            "<text class=\"t\" y=\"{}\">{}</text>\n",
            COVER_H + 20 + i * 18,
            esc(&line)
        ));
    }
    if let Some(url) = book.url.as_deref().filter(|u| is_http(u)) {
        body = format!("<a href=\"{}\">\n{body}</a>\n", esc(url));
    }
    format!("<g class=\"card\" transform=\"translate({x} {COVERS_Y})\">\n{body}</g>\n")
}

fn cover(book: &Book, title: &str, index: i32) -> String {
    match &book.cover {
        Some(cover) => {
            let mime = match cover.format {
                CoverFormat::Png => "image/png",
                CoverFormat::Jpeg => "image/jpeg",
            };
            format!(
                "<clipPath id=\"atsu-shelf-c{index}\"><rect width=\"{COVER_W}\" height=\"{COVER_H}\" rx=\"12\"/></clipPath>\n\
                 <image width=\"{COVER_W}\" height=\"{COVER_H}\" preserveAspectRatio=\"xMidYMid slice\" clip-path=\"url(#atsu-shelf-c{index})\" href=\"data:{mime};base64,{}\"/>\n",
                base64::encode(&cover.bytes)
            )
        }
        None => {
            let color = PLACEHOLDERS[fnv1a(&book.series_id) as usize % PLACEHOLDERS.len()];
            let letter: String = title
                .chars()
                .find(|c| c.is_alphanumeric())
                .map_or("?".to_string(), |c| c.to_uppercase().collect());
            format!(
                "<rect width=\"{COVER_W}\" height=\"{COVER_H}\" rx=\"12\" fill=\"{color}\"/>\n\
                 <text class=\"l\" x=\"{}\" y=\"{}\">{}</text>\n",
                COVER_W / 2,
                COVER_H / 2 + 22,
                esc(&letter)
            )
        }
    }
}

fn status_pill(book: &Book) -> String {
    let (status, color) = if book.completed {
        ("Done", "#1f8a4c")
    } else {
        ("Reading", "#0f5f96")
    };
    let status_w = pill_width(status);
    let chapter = book.chapter.as_deref().map(clean).unwrap_or_default();
    if chapter.is_empty() {
        return pill(8, 8, status_w, color, status);
    }
    let chapter_w = pill_width(&chapter);
    let mut out = pill_rect(8, 8, chapter_w + status_w, "#26262b");
    out.push_str(&pill_label(8 + chapter_w / 2, 8, &chapter));
    out.push_str(&pill(8 + chapter_w, 8, status_w, color, status));
    out
}

fn kind_pill(book: &Book) -> String {
    let label = match book.medium {
        Medium::Novel => "Novel".to_string(),
        Medium::Comic => clean(&book.kind),
    };
    if label.is_empty() {
        return String::new();
    }
    let color = match label.as_str() {
        "Manga" => "#7c5ce0",
        "Manhwa" => "#1e7fd6",
        "Manhua" => "#d9622b",
        "OEL" => "#1f8f7d",
        "Novel" => "#b84f76",
        _ => "#5c6370",
    };
    pill(8, COVER_H - 8 - PILL_H, pill_width(&label), color, &label)
}

fn pill(x: i32, y: i32, w: i32, fill: &str, label: &str) -> String {
    pill_rect(x, y, w, fill) + &pill_label(x + w / 2, y, label)
}

fn pill_rect(x: i32, y: i32, w: i32, fill: &str) -> String {
    format!(
        "<rect x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{PILL_H}\" rx=\"9\" fill=\"{fill}\"/>\n"
    )
}

fn pill_label(center_x: i32, y: i32, label: &str) -> String {
    format!(
        "<text class=\"p\" x=\"{center_x}\" y=\"{}\">{}</text>\n",
        y + 13,
        esc(label)
    )
}

fn pill_width(label: &str) -> i32 {
    text_px(label, 11.0).ceil() as i32 + 14
}

fn is_http(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.starts_with("https://") || lower.starts_with("http://")
}

fn clean(s: &str) -> String {
    s.chars()
        .filter(|c| {
            let c = *c as u32;
            matches!(c, 0x9 | 0xA | 0xD | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x10000..)
        })
        .collect()
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in clean(s).chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

fn text_px(s: &str, font_px: f32) -> f32 {
    s.chars().map(char_em).sum::<f32>() * font_px
}

fn char_em(c: char) -> f32 {
    match c as u32 {
        0x1100..=0x115F
        | 0x2E80..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE6F
        | 0xFF00..=0xFF60
        | 0x1F300..=0x1FAFF
        | 0x20000..=0x3FFFF => 1.0,
        _ => match c {
            ' ' => 0.28,
            'i' | 'j' | 'l' | 'r' | 't' | 'f' | '.' | ',' | ';' | ':' | '\'' | '!' | '|' => 0.33,
            'm' | 'w' => 0.8,
            'M' | 'W' => 0.9,
            c if c.is_uppercase() => 0.68,
            _ => 0.56,
        },
    }
}

fn fnv1a(s: &str) -> u32 {
    s.bytes().fold(0x811C_9DC5, |hash, byte| {
        (hash ^ byte as u32).wrapping_mul(0x0100_0193)
    })
}
