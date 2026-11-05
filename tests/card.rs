use atsu_shelf::base64;
use atsu_shelf::card::{Book, Color, Cover, CoverFormat, Medium, Options, render, wrap_title};
use atsu_shelf::demo;

fn book(title: &str) -> Book {
    Book {
        series_id: format!("id-{title}"),
        title: title.to_string(),
        url: None,
        medium: Medium::Comic,
        kind: "Manga".to_string(),
        chapter: Some("7".to_string()),
        completed: false,
        cover: None,
    }
}

fn with_cover(mut b: Book) -> Book {
    b.cover = Some(Cover {
        format: CoverFormat::Png,
        bytes: b"foobar".to_vec(),
    });
    b
}

#[test]
fn base64_matches_rfc_4648_vectors() {
    let vectors: [(&str, &str); 7] = [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("foob", "Zm9vYg=="),
        ("fooba", "Zm9vYmE="),
        ("foobar", "Zm9vYmFy"),
    ];
    for (input, want) in vectors {
        assert_eq!(base64::encode(input.as_bytes()), want);
    }
}

#[test]
fn output_is_an_svg_document() {
    let svg = render(&[book("Alpha")], &Options::default());
    assert!(svg.starts_with("<svg "));
    assert!(svg.trim_end().ends_with("</svg>"));
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
}

#[test]
fn draws_one_card_per_book_up_to_the_limit() {
    let books: Vec<Book> = ["A", "B", "C", "D"].map(book).into();
    let opts = Options {
        cards: 3,
        ..Options::default()
    };
    let svg = render(&books, &opts);
    assert_eq!(svg.matches("class=\"card\"").count(), 3);
    assert!(svg.contains(">C<"));
    assert!(!svg.contains(">D<"));
}

#[test]
fn width_follows_the_number_of_cards_drawn() {
    let one = render(&[book("A")], &Options::default());
    let two = render(&[book("A"), book("B")], &Options::default());
    assert!(one.contains("width=\"182\""), "{}", &one[..120]);
    assert!(two.contains("width=\"346\""), "{}", &two[..120]);
}

#[test]
fn empty_shelf_says_nothing_yet() {
    let svg = render(&[], &Options::default());
    assert!(svg.contains("Nothing yet"));
    assert_eq!(svg.matches("class=\"card\"").count(), 0);
}

#[test]
fn heading_is_configurable() {
    let opts = Options {
        heading: "Your Bookmarks".to_string(),
        ..Options::default()
    };
    assert!(render(&[], &opts).contains(">Your Bookmarks<"));
}

#[test]
fn cover_is_embedded_as_a_data_uri() {
    let svg = render(&[with_cover(book("A"))], &Options::default());
    assert!(svg.contains("href=\"data:image/png;base64,Zm9vYmFy\""));
}

#[test]
fn missing_cover_draws_a_placeholder_with_the_first_letter() {
    let svg = render(&[book("alpha")], &Options::default());
    assert!(!svg.contains("<image"));
    assert!(svg.contains(">A<"));
}

#[test]
fn placeholder_handles_an_empty_title() {
    let svg = render(&[book("")], &Options::default());
    assert_eq!(svg.matches("class=\"card\"").count(), 1);
}

#[test]
fn credit_only_appears_with_a_cover() {
    let credit = "Covers: AniList";
    assert!(!render(&[book("A")], &Options::default()).contains(credit));
    assert!(render(&[with_cover(book("A"))], &Options::default()).contains(credit));
}

#[test]
fn pills_show_chapter_status_and_type() {
    let mut b = book("A");
    b.kind = "Manhwa".to_string();
    b.chapter = Some("112.7".to_string());
    let svg = render(&[b], &Options::default());
    assert!(svg.contains(">112.7<"));
    assert!(svg.contains(">Reading<"));
    assert!(svg.contains(">Manhwa<"));
}

#[test]
fn completed_series_says_done() {
    let mut b = book("A");
    b.completed = true;
    let svg = render(&[b], &Options::default());
    assert!(svg.contains(">Done<"));
    assert!(!svg.contains(">Reading<"));
}

#[test]
fn novel_medium_is_labelled_novel_whatever_the_type() {
    let mut b = book("A");
    b.medium = Medium::Novel;
    b.kind = "Other".to_string();
    let svg = render(&[b], &Options::default());
    assert!(svg.contains(">Novel<"));
    assert!(!svg.contains(">Other<"));
}

#[test]
fn comic_with_type_other_stays_other() {
    let mut b = book("A");
    b.kind = "Other".to_string();
    let svg = render(&[b], &Options::default());
    assert!(svg.contains(">Other<"));
    assert!(!svg.contains(">Novel<"));
}

#[test]
fn hostile_text_is_escaped() {
    let mut b = book("<script>alert('x')</script> & \"co\"");
    b.kind = "<b>".to_string();
    b.chapter = Some("</text>x".to_string());
    let opts = Options {
        heading: "<svg onload=x>".to_string(),
        ..Options::default()
    };
    let svg = render(&[b], &opts);
    assert!(!svg.contains("<script"));
    assert!(!svg.contains("<b>"));
    assert!(!svg.contains("</text>x"));
    assert!(!svg.contains("<svg onload"));
    assert!(svg.contains("&lt;script&gt;"));
    assert!(svg.contains("&amp;"));
}

#[test]
fn characters_xml_forbids_are_dropped() {
    let svg = render(&[book("Al\u{0}ph\u{1b}a\u{fffe}")], &Options::default());
    assert!(svg.contains(">Alpha<"));
    assert!(!svg.contains('\u{0}'));
    assert!(!svg.contains('\u{1b}'));
}

#[test]
fn only_http_links_are_kept() {
    let mut good = book("A");
    good.url = Some("https://atsu.moe/manga/abc?a=1&b=2".to_string());
    let svg = render(&[good], &Options::default());
    assert!(svg.contains("href=\"https://atsu.moe/manga/abc?a=1&amp;b=2\""));

    for bad in [
        "javascript:alert(1)",
        "JaVaScRiPt:alert(1)",
        "data:text/html,x",
    ] {
        let mut b = book("A");
        b.url = Some(bad.to_string());
        let svg = render(&[b], &Options::default());
        assert!(!svg.contains("<a "), "kept {bad}");
    }
}

#[test]
fn wrap_keeps_short_titles_on_one_line() {
    assert_eq!(
        wrap_title("Occultic;Nine", 150.0, 13.0, 2),
        ["Occultic;Nine"]
    );
}

#[test]
fn wrap_breaks_between_words() {
    let lines = wrap_title("Steins;Gate: My Darling's Embrace", 150.0, 13.0, 2);
    assert_eq!(lines.len(), 2);
    assert_eq!(lines.join(" "), "Steins;Gate: My Darling's Embrace");
}

#[test]
fn wrap_adds_an_ellipsis_when_it_runs_out_of_lines() {
    let title = "Living Together with the Queen from My High School Who Used to Be Arrogant";
    let lines = wrap_title(title, 150.0, 13.0, 2);
    assert_eq!(lines.len(), 2);
    assert!(lines[1].ends_with('…'), "{lines:?}");
}

#[test]
fn wrap_splits_text_with_no_spaces() {
    let lines = wrap_title(&"長".repeat(40), 150.0, 13.0, 2);
    assert_eq!(lines.len(), 2);
    assert!(lines[0].chars().count() <= 12, "{lines:?}");
    assert!(lines[1].ends_with('…'));

    let word = wrap_title(&"W".repeat(200), 150.0, 13.0, 2);
    assert_eq!(word.len(), 2);
}

#[test]
fn wrap_handles_empty_and_blank_titles() {
    assert!(wrap_title("", 150.0, 13.0, 2).is_empty());
    assert!(wrap_title("   ", 150.0, 13.0, 2).is_empty());
}

#[test]
fn default_theme_is_transparent_neutral_and_ignores_the_viewers_dark_mode() {
    let svg = render(&[], &Options::default());
    assert!(!svg.contains("prefers-color-scheme"));
    assert!(svg.contains("var(--atsu-shelf-background,transparent)"));
    assert!(svg.contains("var(--atsu-shelf-heading,#6e7681)"));
    assert!(svg.contains("var(--atsu-shelf-title,#6e7681)"));
}

#[test]
fn host_page_variables_can_override_every_color() {
    let opts = Options {
        heading_color: Some("#111111".parse().unwrap()),
        title_color: Some("#ff69b4".parse().unwrap()),
        background: Some("#ffffff".parse().unwrap()),
        ..Options::default()
    };
    let svg = render(&[], &opts);
    assert!(svg.contains("var(--atsu-shelf-heading,#111111)"));
    assert!(svg.contains("var(--atsu-shelf-title,#ff69b4)"));
    assert!(svg.contains("var(--atsu-shelf-background,#ffffff)"));
}

#[test]
fn a_fixed_color_leaves_the_others_at_their_defaults() {
    let opts = Options {
        title_color: Some("#ff69b4".parse().unwrap()),
        ..Options::default()
    };
    let svg = render(&[], &opts);
    assert!(svg.contains("var(--atsu-shelf-title,#ff69b4)"));
    assert!(svg.contains("var(--atsu-shelf-heading,#6e7681)"));
}

#[test]
fn fixed_background_picks_readable_text() {
    let on = |bg: &str| {
        render(
            &[],
            &Options {
                background: Some(bg.parse().unwrap()),
                ..Options::default()
            },
        )
    };
    let white = on("#fff");
    assert!(white.contains("var(--atsu-shelf-heading,#24292f)"));
    let black = on("#000000");
    assert!(black.contains("var(--atsu-shelf-heading,#e6edf3)"));
    let clear = on("transparent");
    assert!(clear.contains("var(--atsu-shelf-heading,#6e7681)"));
}

#[test]
fn colors_accept_hex_and_transparent_only() {
    for ok in ["#fff", "#FFF", "#ff69b4", "transparent"] {
        assert!(ok.parse::<Color>().is_ok(), "{ok}");
    }
    for bad in [
        "",
        "red",
        "#ff",
        "#gggggg",
        "#ff69b4;}",
        "#fff}body{display:none",
        "url(x)",
    ] {
        assert!(bad.parse::<Color>().is_err(), "{bad}");
    }
}

#[test]
fn demo_has_six_books_with_invented_content_and_decodable_covers() {
    let books = demo::books();
    assert_eq!(books.len(), 6);
    let mut covers = 0;
    for b in &books {
        if let Some(cover) = &b.cover {
            let img = image::load_from_memory(&cover.bytes).expect("cover decodes");
            assert_eq!((img.width(), img.height()), (300, 450));
            covers += 1;
        }
    }
    assert_eq!(covers, 5, "one book shows the no-cover placeholder");
}

#[test]
fn demo_is_deterministic() {
    let render_demo = || render(&demo::books(), &Options::default());
    assert_eq!(render_demo(), render_demo());
}

#[test]
fn committed_demo_svg_is_current() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/embed/demo.svg");
    let committed = std::fs::read_to_string(path).unwrap().replace("\r\n", "\n");
    assert!(
        committed == render(&demo::books(), &Options::default()),
        "embed/demo.svg is out of date; run: cargo run -- draw --demo --out embed/demo.svg"
    );
}
