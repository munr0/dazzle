//! Checks dazzle's safe graphic sizes with real decoders: rxing (derived from ZXing) and rqrr (a
//! port of quirc).
//!
//! Table check (the default): for every version and error correction level, several payloads are
//! encoded with each of the 8 masks, using the same `qrcode` crate as zebra. Each symbol is damaged
//! in a centred square of the table size and must decode with both decoders:
//!
//! - `invert`: every data module in the square is flipped and function patterns are kept, so every
//!   touched codeword is wrong. This is the worst case the table assumes.
//! - `light`, `dark`: the whole square, function patterns included, is set light or dark, which is
//!   what a code with a graphic looks like to a camera.
//!
//! The `next size` column flips the square one size up. With a table generated with `--margin 0`
//! it should fail everywhere except where that size is limited by a function pattern (version 1),
//! which shows the table is tight.
//!
//! `--noise P` also flips each data module outside the square with probability P, to see how much
//! the margin helps against print defects. `--renders DIR` decodes the PNG pages Typst rendered
//! from renders.typ instead (see `just check-renders`).
//!
//! Usage: decode-check [--table FILE] [--level l|m|q|h]... [--version N]... [--noise P]
//!        decode-check --renders DIR

use qrcode::{
    Color, EcLevel, Version,
    bits::Bits,
    canvas::{Canvas, MaskPattern, is_functional},
    ec,
};
use rayon::prelude::*;
use std::path::Path;
use std::process::ExitCode;

const LEVELS: [(&str, EcLevel); 4] = [
    ("l", EcLevel::L),
    ("m", EcLevel::M),
    ("q", EcLevel::Q),
    ("h", EcLevel::H),
];

const MASKS: [MaskPattern; 8] = [
    MaskPattern::Checkerboard,
    MaskPattern::HorizontalLines,
    MaskPattern::VerticalLines,
    MaskPattern::DiagonalLines,
    MaskPattern::LargeCheckerboard,
    MaskPattern::Fields,
    MaskPattern::Diamonds,
    MaskPattern::Meadow,
];

/// Versions a decoder can't be relied on for, with the reason
const KNOWN_ISSUES: [(Decoder, i16, &str); 1] = [(
    Decoder::Rqrr,
    21,
    "rqrr 0.11 has the wrong alignment pattern position for version 21 (92 instead of 94)",
)];

/// Pixels per module and quiet zone modules of the synthetic test images
const SCALE: usize = 4;
const QUIET: usize = 4;

const DEFAULT_TABLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../src/safe-sizes.toml");

#[derive(Clone, Copy, PartialEq)]
enum Decoder {
    Rxing,
    Rqrr,
}

#[derive(Clone, Copy, PartialEq)]
enum Fill {
    Invert,
    Light,
    Dark,
}

/// One kind of damaged symbol, decoded for every payload and mask
struct Case {
    level: usize,
    version: i16,
    side: usize,
    fill: Fill,
    /// One size up from the table, which isn't required to decode
    next: bool,
}

/// Whether one symbol decoded with (rxing, rqrr)
type Outcome = (bool, bool);

fn main() -> ExitCode {
    let args = Args::parse();
    let ok = match &args.renders {
        Some(dir) => check_renders(Path::new(dir)),
        None => check_table(&args),
    };
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn check_table(args: &Args) -> bool {
    let text = std::fs::read_to_string(&args.table)
        .unwrap_or_else(|e| panic!("reading {}: {e}", args.table));
    let table: toml::Table = text.parse().expect("invalid table");
    let sizes: Vec<Vec<usize>> = LEVELS
        .iter()
        .map(|(name, _)| {
            let values = table[*name].as_array().expect("missing level");
            assert_eq!(values.len(), 40, "level {name} must have 40 sizes");
            values
                .iter()
                .map(|v| v.as_integer().unwrap() as usize)
                .collect()
        })
        .collect();

    let mut cases = Vec::new();
    for (level, (name, _)) in LEVELS.iter().enumerate() {
        if !args.levels.is_empty() && !args.levels.iter().any(|l| l == name) {
            continue;
        }
        for version in 1..=40i16 {
            if !args.versions.is_empty() && !args.versions.contains(&version) {
                continue;
            }
            let side = sizes[level][version as usize - 1];
            if side == 0 {
                continue;
            }
            let width = Version::Normal(version).width() as usize;
            let fills = if args.noise > 0.0 {
                &[Fill::Light][..]
            } else {
                &[Fill::Invert, Fill::Light, Fill::Dark]
            };
            for &fill in fills {
                cases.push(Case {
                    level,
                    version,
                    side,
                    fill,
                    next: false,
                });
            }
            if args.noise == 0.0 && side + 2 <= width {
                cases.push(Case {
                    level,
                    version,
                    side: side + 2,
                    fill: Fill::Invert,
                    next: true,
                });
            }
        }
    }

    eprintln!("Checking {} with rxing and rqrr", args.table);
    let results: Vec<Vec<Outcome>> = cases.par_iter().map(|c| run_case(c, args.noise)).collect();
    let find = |level: usize, version: i16, fill: Fill, next: bool| {
        cases.iter().zip(&results).find(|(c, _)| {
            c.level == level && c.version == version && c.fill == fill && c.next == next
        })
    };

    if args.noise > 0.0 {
        let (mut total, mut rxing, mut rqrr) = (0, 0, 0);
        for outcomes in &results {
            total += outcomes.len();
            rxing += outcomes.iter().filter(|o| o.0).count();
            rqrr += outcomes.iter().filter(|o| o.1).count();
        }
        println!(
            "noise {}: decoded {rxing}/{total} with rxing, {rqrr}/{total} with rqrr",
            args.noise
        );
        return true;
    }

    let mut ok = true;
    let mut notes = Vec::new();
    println!("level version  side  invert  light   dark    next size");
    for (level, (name, _)) in LEVELS.iter().enumerate() {
        for version in 1..=40 {
            let Some((case, _)) = find(level, version, Fill::Invert, false) else {
                continue;
            };
            let mut cells = Vec::new();
            for fill in [Fill::Invert, Fill::Light, Fill::Dark] {
                let (_, outcomes) = find(level, version, fill, false).unwrap();
                let (cell, passed, note) = judge(outcomes, version);
                ok &= passed;
                notes.extend(note);
                cells.push(cell);
            }
            let next =
                find(level, version, Fill::Invert, true).map_or("-".into(), |(_, o)| count(o));
            println!(
                "{:>5} {:>7} {:>5}  {:<7} {:<7} {:<7} {}",
                name, version, case.side, cells[0], cells[1], cells[2], next
            );
        }
    }
    notes.dedup();
    for note in notes {
        println!("* {note}");
    }
    if ok {
        println!("\nOK: every symbol at the table size decoded");
    } else {
        println!("\nFAILED: some symbols at the table size did not decode");
    }
    ok
}

/// "ok" if every symbol decoded with both decoders, apart from known decoder issues
fn judge(outcomes: &[Outcome], version: i16) -> (String, bool, Option<&'static str>) {
    let known = |decoder| {
        KNOWN_ISSUES
            .iter()
            .find(|(d, v, _)| *d == decoder && *v == version)
    };
    let rxing = outcomes.iter().all(|o| o.0);
    let rqrr = outcomes.iter().all(|o| o.1);
    match (
        rxing || known(Decoder::Rxing).is_some(),
        rqrr || known(Decoder::Rqrr).is_some(),
    ) {
        (true, true) if rxing && rqrr => ("ok".into(), true, None),
        (true, true) => {
            let note = known(Decoder::Rxing).or(known(Decoder::Rqrr)).unwrap().2;
            ("ok*".into(), true, Some(note))
        }
        _ => (count(outcomes), false, None),
    }
}

/// How many symbols decoded with rxing/rqrr
fn count(outcomes: &[Outcome]) -> String {
    let rxing = outcomes.iter().filter(|o| o.0).count();
    let rqrr = outcomes.iter().filter(|o| o.1).count();
    if rxing + rqrr == 0 {
        "fails".into()
    } else {
        format!("{rxing}/{rqrr} of {}", outcomes.len())
    }
}

fn run_case(case: &Case, noise: f64) -> Vec<Outcome> {
    let level = LEVELS[case.level].1;
    let width = Version::Normal(case.version).width() as usize;
    payloads(case.version, level)
        .iter()
        .enumerate()
        .flat_map(|(i, payload)| {
            MASKS.iter().enumerate().map(move |(j, &mask)| {
                let mut colors = encode(payload, case.version, level, mask);
                damage(&mut colors, case.version, case.side, case.fill);
                if noise > 0.0 {
                    let seed =
                        (case.version as u64) << 16 | (case.level as u64) << 8 | (i * 8 + j) as u64;
                    add_noise(&mut colors, case.version, case.side, noise, seed);
                }
                let size = (width + 2 * QUIET) * SCALE;
                let luma = (0..size * size)
                    .map(|i| {
                        let (x, y) = ((i % size) / SCALE, (i / size) / SCALE);
                        let range = QUIET..QUIET + width;
                        let inside = range.contains(&x) && range.contains(&y);
                        let dark =
                            inside && colors[(y - QUIET) * width + (x - QUIET)] == Color::Dark;
                        if dark { 0 } else { 255 }
                    })
                    .collect();
                decode(luma, size, size, payload)
            })
        })
        .collect()
}

fn encode(data: &[u8], version: i16, level: EcLevel, mask: MaskPattern) -> Vec<Color> {
    let version = Version::Normal(version);
    let mut bits = Bits::new(version);
    bits.push_byte_data(data).unwrap();
    bits.push_terminator(level).unwrap();
    let (data, ec) = ec::construct_codewords(&bits.into_bytes(), version, level).unwrap();
    let mut canvas = Canvas::new(version, level);
    canvas.draw_all_functional_patterns();
    canvas.draw_data(&data, &ec);
    canvas.apply_mask(mask);
    canvas.into_colors()
}

fn damage(colors: &mut [Color], version: i16, side: usize, fill: Fill) {
    let width = Version::Normal(version).width() as usize;
    let start = (width - side) / 2;
    for y in start..start + side {
        for x in start..start + side {
            let color = &mut colors[y * width + x];
            *color = match fill {
                Fill::Light => Color::Light,
                Fill::Dark => Color::Dark,
                Fill::Invert if functional(version, x, y) => *color,
                Fill::Invert => !*color,
            };
        }
    }
}

/// Flips each data module outside the square with the given probability, like print defects
fn add_noise(colors: &mut [Color], version: i16, side: usize, probability: f64, seed: u64) {
    let width = Version::Normal(version).width() as usize;
    let square = (width - side) / 2..(width + side) / 2;
    let mut rng = Xorshift64::new(seed);
    for y in 0..width {
        for x in 0..width {
            let inside = square.contains(&x) && square.contains(&y);
            let flip = (rng.next() % 1_000_000) as f64 / 1e6 < probability;
            if flip && !inside && !functional(version, x, y) {
                colors[y * width + x] = !colors[y * width + x];
            }
        }
    }
}

fn functional(version: i16, x: usize, y: usize) -> bool {
    let version = Version::Normal(version);
    is_functional(version, version.width(), x as i16, y as i16)
}

fn decode(luma: Vec<u8>, width: usize, height: usize, expected: &[u8]) -> Outcome {
    let qr = Some(rxing::BarcodeFormat::QR_CODE);
    let rxing = rxing::helpers::detect_in_luma(luma.clone(), width as u32, height as u32, qr)
        .is_ok_and(|result| result.getText().as_bytes() == expected);

    let mut image =
        rqrr::PreparedImage::prepare_from_greyscale(width, height, |x, y| luma[y * width + x]);
    let rqrr = image.detect_grids().iter().any(|grid| {
        let mut out = Vec::new();
        grid.decode_to(&mut out).is_ok() && out == expected
    });

    (rxing, rqrr)
}

/// Decodes the pages rendered from renders.typ: DIR/*.png in name order, against the payloads in
/// DIR/payloads.json (from `typst eval`)
fn check_renders(dir: &Path) -> bool {
    let json = std::fs::read_to_string(dir.join("payloads.json")).expect("reading payloads.json");
    let payloads: Vec<String> = serde_json::from_str(&json).expect("invalid payloads.json");
    let mut pages: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "png"))
        .collect();
    pages.sort();
    assert_eq!(pages.len(), payloads.len(), "one page per payload");

    let results: Vec<Outcome> = pages
        .par_iter()
        .zip(&payloads)
        .map(|(page, payload)| {
            let image = image::open(page).unwrap().into_luma8();
            let (width, height) = image.dimensions();
            decode(
                image.into_raw(),
                width as usize,
                height as usize,
                payload.as_bytes(),
            )
        })
        .collect();

    let mut ok = true;
    for ((page, payload), (rxing, rqrr)) in pages.iter().zip(&payloads).zip(&results) {
        if !(*rxing && *rqrr) {
            ok = false;
            let name = page.file_name().unwrap().to_string_lossy();
            println!(
                "{name}: rxing {rxing}, rqrr {rqrr} ({} bytes)",
                payload.len()
            );
        }
    }
    println!(
        "{} of {} rendered codes decoded with both decoders",
        results.iter().filter(|o| o.0 && o.1).count(),
        results.len()
    );
    ok
}

/// Printable ASCII payloads for one version and level: two at full capacity, one at half
/// capacity and a short URL, so blocks see both data and padding
fn payloads(version: i16, level: EcLevel) -> Vec<Vec<u8>> {
    let max = capacity(version, level);
    let mut rng = Xorshift64::new(version as u64 * 4 + level as u64);
    let mut random = |len: usize| (0..len).map(|_| b' ' + (rng.next() % 95) as u8).collect();
    let url = b"https://typst.app".to_vec();
    let short = if url.len() <= max {
        url
    } else {
        random(max / 2 + 1)
    };
    vec![random(max), random(max), random(max / 2 + 1), short]
}

/// Longest byte-mode payload that fits the version at the level
fn capacity(version: i16, level: EcLevel) -> usize {
    let fits = |len: usize| {
        let mut bits = Bits::new(Version::Normal(version));
        bits.push_byte_data(&vec![b'a'; len]).is_ok() && bits.push_terminator(level).is_ok()
    };
    let (mut low, mut high) = (1, 4096);
    while low + 1 < high {
        let mid = (low + high) / 2;
        if fits(mid) { low = mid } else { high = mid }
    }
    low
}

struct Xorshift64(u64);

impl Xorshift64 {
    fn new(seed: u64) -> Self {
        let s = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        Self(if s == 0 { 1 } else { s })
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

struct Args {
    table: String,
    levels: Vec<String>,
    versions: Vec<i16>,
    noise: f64,
    renders: Option<String>,
}

impl Args {
    fn parse() -> Self {
        let mut args = Args {
            table: DEFAULT_TABLE.to_string(),
            levels: vec![],
            versions: vec![],
            noise: 0.0,
            renders: None,
        };
        let mut iter = std::env::args().skip(1);
        while let Some(arg) = iter.next() {
            let mut value = || iter.next().unwrap_or_else(|| panic!("{arg} needs a value"));
            match arg.as_str() {
                "--table" => args.table = value(),
                "--level" => args.levels.push(value()),
                "--version" => args
                    .versions
                    .push(value().parse().expect("invalid version")),
                "--noise" => args.noise = value().parse().expect("invalid probability"),
                "--renders" => args.renders = Some(value()),
                _ => panic!("unknown argument {arg}"),
            }
        }
        args
    }
}
