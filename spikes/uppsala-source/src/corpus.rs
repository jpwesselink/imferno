//! The XML corpus the spike measures against.

use std::path::{Path, PathBuf};

/// Repo root, from this crate's manifest dir (`spikes/uppsala-source`).
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>, pred: &dyn Fn(&Path) -> bool) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            walk(&p, out, pred);
        } else if pred(&p) {
            out.push(p);
        }
    }
}

fn is_xml(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("xml"))
}

/// Every `.xml` under `test-data/`, `docs/docs/public/samples/` and
/// `fuzz/seeds/parse_xml/` (the latter two do not exist on main today).
pub fn imf_corpus() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    walk(&root.join("test-data"), &mut out, &is_xml);
    walk(&root.join("docs/docs/public/samples"), &mut out, &is_xml);
    walk(&root.join("fuzz/seeds/parse_xml"), &mut out, &|_| true);
    out
}

/// Other tracked XML in the repo (XSDs, SMPTE registers, RegXML dictionaries):
/// not IMF documents, but more real-world XML for the range checks.
pub fn extra_corpus() -> Vec<PathBuf> {
    let root = repo_root();
    let mut out = Vec::new();
    walk(&root.join("crates/imferno-core/specs"), &mut out, &|p| {
        p.extension().is_some_and(|e| e == "xsd")
    });
    walk(
        &root.join("crates/imferno-core/resources"),
        &mut out,
        &is_xml,
    );
    out
}

/// Root element local name, by a cheap scan (no parser).
pub fn root_local_name(src: &str) -> Option<String> {
    let mut rest = src;
    loop {
        let i = rest.find('<')?;
        rest = &rest[i + 1..];
        if rest.starts_with('?') || rest.starts_with('!') {
            continue;
        }
        let end = rest.find(|c: char| c.is_whitespace() || c == '>' || c == '/')?;
        let q = &rest[..end];
        return Some(q.rsplit(':').next().unwrap_or(q).to_string());
    }
}

pub fn rel(p: &Path) -> String {
    p.strip_prefix(repo_root())
        .unwrap_or(p)
        .display()
        .to_string()
}
