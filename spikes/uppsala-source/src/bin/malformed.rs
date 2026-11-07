//! Criterion 7: error positions for malformed XML.
//!
//! Each case injects one fault into a real CPL at a known byte offset and
//! records where uppsala (and, for comparison, quick-xml's reader) reports
//! the error.
//!
//! Usage: cargo run --release --bin malformed

use uppsala_source_spike::corpus::repo_root;
use uppsala_source_spike::lines::LineIndex;

const BASE: &str =
    "test-data/MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml";

/// Replace the first occurrence of `find`; the fault offset is where the
/// replacement starts plus `at` bytes into it.
fn inject(src: &str, find: &str, with: &str, at: usize) -> (Vec<u8>, usize) {
    let i = src
        .find(find)
        .unwrap_or_else(|| panic!("{find} not in base"));
    let out = format!("{}{}{}", &src[..i], with, &src[i + find.len()..]);
    (out.into_bytes(), i + at)
}

fn uppsala_error(e: &uppsala::XmlError) -> (String, Option<(usize, usize)>) {
    use uppsala::XmlError as E;
    match e {
        E::WellFormedness(w) => (w.message.clone(), Some((w.line, w.column))),
        E::Namespace(n) => (n.message.clone(), Some((n.line, n.column))),
        E::Parse(p) => (p.message.clone(), Some((p.line, p.column))),
        E::UnexpectedEof => ("Unexpected end of input".into(), None),
        other => (other.to_string(), None),
    }
}

fn quick_xml_error(src: &str) -> String {
    let mut r = quick_xml::Reader::from_str(src);
    loop {
        match r.read_event() {
            Ok(quick_xml::events::Event::Eof) => return "accepted".into(),
            Ok(_) => {}
            Err(e) => return format!("byte {} ({e})", r.error_position()),
        }
    }
}

fn main() {
    let base = std::fs::read_to_string(repo_root().join(BASE)).unwrap();
    let mut cases: Vec<(&str, Vec<u8>, usize)> = Vec::new();
    let mut add = |label, (bytes, at): (Vec<u8>, usize)| cases.push((label, bytes, at));

    add(
        "unclosed element (</ContentTitle> removed)",
        inject(&base, "MERIDIAN</ContentTitle>", "MERIDIAN", 8),
    );
    add(
        "start tag missing '>'",
        inject(&base, "<ContentKind>Test", "<ContentKind Test", 13),
    );
    add(
        "unquoted attribute value",
        inject(&base, "<ContentTitle>", "<ContentTitle language=en>", 23),
    );
    add(
        "attribute without value",
        inject(&base, "<ContentTitle>", "<ContentTitle language>", 22),
    );
    add(
        "duplicate attribute",
        inject(
            &base,
            "<ContentTitle>",
            "<ContentTitle a=\"1\" a=\"2\">",
            20,
        ),
    );
    add(
        "mismatched end tag",
        inject(&base, "</ContentTitle>", "</ContentTitel>", 0),
    );
    add(
        "raw '<' in text",
        inject(&base, ">MERIDIAN<", ">MERI < DIAN<", 6),
    );
    add(
        "raw '&' in text",
        inject(&base, ">MERIDIAN<", ">MERI & DIAN<", 6),
    );
    add(
        "undefined entity",
        inject(&base, ">MERIDIAN<", ">MERI &foo; DIAN<", 6),
    );
    add(
        "undeclared prefix",
        inject(&base, "<ContentKind>", "<x:ContentKind>", 0),
    );
    add(
        "control character U+0001",
        inject(&base, ">MERIDIAN<", ">MERI\u{1}DIAN<", 5),
    );
    add("text after root", {
        let s = format!("{base}\ntrailing");
        let n = base.len() + 1;
        (s.into_bytes(), n)
    });
    add(
        "comment containing '--'",
        inject(&base, "<ContentKind>", "<!-- a -- b --><ContentKind>", 7),
    );
    add("truncated at 50%", {
        let mut n = base.len() / 2;
        while !base.is_char_boundary(n) {
            n -= 1;
        }
        (base[..n].as_bytes().to_vec(), n)
    });
    add("truncated inside a start tag", {
        let n = base.find("<ContentKind>").unwrap() + 5;
        (base[..n].as_bytes().to_vec(), n)
    });
    add(
        "declared UTF-16, content UTF-8",
        inject(&base, "encoding=\"UTF-8\"", "encoding=\"UTF-16\"", 10),
    );
    add("Latin-1 byte (0xE9) in UTF-8 text", {
        let (mut b, at) = inject(&base, ">MERIDIAN<", ">MERIXDIAN<", 5);
        b[at] = 0xE9;
        (b, at)
    });
    add("Latin-1 document declaring ISO-8859-1", {
        let (mut b, at) = inject(
            &base.replace("encoding=\"UTF-8\"", "encoding=\"ISO-8859-1\""),
            ">MERIDIAN<",
            ">MERIXDIAN<",
            5,
        );
        b[at] = 0xE9;
        (b, at)
    });
    add("UTF-16LE with BOM", {
        let mut b = vec![0xFF, 0xFE];
        for u in base.encode_utf16() {
            b.extend_from_slice(&u.to_le_bytes());
        }
        (b, 0)
    });

    println!("| case | fault at (line:col) | uppsala error | uppsala at | points at fault? | quick-xml |");
    println!("|---|---|---|---|---|---|");
    for (label, bytes, fault) in cases {
        // uppsala takes &str: decoding is the caller's job.
        let src = match String::from_utf8(bytes.clone()) {
            Ok(s) => s,
            Err(e) => {
                let lines = LineIndex::new(&String::from_utf8_lossy(&bytes));
                let at = e.utf8_error().valid_up_to();
                let (fl, fc) = lines.line_col(fault);
                let (l, c) = lines.line_col(at);
                println!(
                    "| {label} | {fl}:{fc} | (not reached) `String::from_utf8`: invalid UTF-8 | {l}:{c} (byte {at}) | {} | n/a |",
                    if at == fault { "yes (exact byte)" } else { "no" }
                );
                continue;
            }
        };
        let lines = LineIndex::new(&src);
        let (fl, fc) = lines.line_col(fault.min(src.len()));
        let qx = quick_xml_error(&src);
        match uppsala::parse(&src) {
            Ok(_) => println!("| {label} | {fl}:{fc} | **accepted** | - | - | {qx} |"),
            Err(e) => {
                let (msg, pos) = uppsala_error(&e);
                let (at, verdict) = match pos {
                    None => ("(none)".to_string(), "no position (EOF)".to_string()),
                    Some((l, c)) => {
                        let off = lines.offset(l, c).unwrap_or(usize::MAX);
                        let d = off as isize - fault as isize;
                        let v = if l == fl && d.unsigned_abs() <= 2 {
                            "yes".to_string()
                        } else if l == fl {
                            format!("same line, {d:+} bytes")
                        } else {
                            format!("no: {:+} lines", l as isize - fl as isize)
                        };
                        (format!("{l}:{c}"), v)
                    }
                };
                println!("| {label} | {fl}:{fc} | {msg} | {at} | {verdict} | {qx} |");
            }
        }
    }
}
