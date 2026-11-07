//! Criterion 8: property tests over the range invariants of criterion 1.
//!
//! * `generated_documents`: random well-formed documents (prefixes, namespace
//!   declarations, both quote styles, entity/char references, CR LF, comments,
//!   CDATA, PIs, self-closing tags, odd whitespace). Must parse; all
//!   invariants must hold.
//! * `mutated_corpus`: real corpus files with random byte-level edits around
//!   markup. Must never panic; when uppsala accepts, invariants must hold.
//! * `arbitrary_markup`: short strings over a markup-heavy alphabet. Same.
//!
//! Default case counts are small for `cargo test`; the spike run used
//! `PROPTEST_CASES=...` (see REPORT.md).

use proptest::prelude::*;
use uppsala_source_spike::corpus::imf_corpus;
use uppsala_source_spike::invariants::{check, Options};

/// 256 cases unless PROPTEST_CASES is set; no failure persistence files.
fn cfg() -> ProptestConfig {
    let mut c = ProptestConfig::default();
    if std::env::var_os("PROPTEST_CASES").is_none() {
        c.cases = 256;
    }
    c.failure_persistence = None;
    c
}

fn assert_invariants(src: &str) -> Result<(), TestCaseError> {
    // No panic allowed on any input.
    let parsed = std::panic::catch_unwind(|| uppsala::parse(src).map(|_| ()));
    prop_assert!(parsed.is_ok(), "uppsala panicked on {:?}", src);
    if let Ok(doc) = uppsala::parse(src) {
        let (_, v) = check(
            src,
            &doc,
            &Options {
                check_line_col: true,
            },
        );
        prop_assert!(
            v.is_empty(),
            "violations {:?} for {:?}",
            v.iter().take(3).collect::<Vec<_>>(),
            src
        );
    }
    Ok(())
}

// ── Generated well-formed documents ────────────────────────────────────────

#[derive(Debug, Clone)]
enum Node {
    Elem {
        prefix: Option<usize>,
        name: String,
        attrs: Vec<(String, String, bool)>,
        decl: Option<usize>,
        kids: Vec<Node>,
        self_close: bool,
    },
    Text(String),
    Comment(String),
    CData(String),
    Pi(String),
}

const PREFIXES: [&str; 3] = ["a", "xs", "cc"];

fn name() -> impl Strategy<Value = String> {
    "[A-Za-z_][A-Za-z0-9_.-]{0,8}|[é日][a-z]{0,3}"
}

fn text() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            "[ a-zA-Z0-9\t\n]{1,10}",
            Just("\r\n".to_string()),
            Just("&amp;".to_string()),
            Just("&lt;".to_string()),
            Just("&gt;".to_string()),
            Just("&#233;".to_string()),
            Just("&#x1F600;".to_string()),
            Just("]]".to_string()),
            Just("☕".to_string()),
        ],
        1..5,
    )
    .prop_map(|v| v.concat())
}

fn attr_value() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop_oneof![
            "[ a-zA-Z0-9/:>=\t\n]{0,8}",
            Just("&quot;".to_string()),
            Just("&apos;".to_string()),
            Just("&amp;".to_string()),
            Just("&#10;".to_string()),
        ],
        0..4,
    )
    .prop_map(|v| v.concat())
}

fn node() -> impl Strategy<Value = Node> {
    let leaf = prop_oneof![
        text().prop_map(Node::Text),
        "[a-z -]{0,10}"
            .prop_map(|s| Node::Comment(s.replace("--", "- ").trim_end_matches('-').to_string())),
        "[a-z<>& ]{0,10}".prop_map(Node::CData),
        "[a-z ]{0,6}".prop_map(Node::Pi),
    ];
    leaf.prop_recursive(5, 64, 6, |inner| {
        (
            prop::option::of(0..PREFIXES.len()),
            name(),
            prop::collection::vec((name(), attr_value(), any::<bool>()), 0..4),
            prop::option::of(0..PREFIXES.len()),
            prop::collection::vec(inner, 0..6),
            any::<bool>(),
        )
            .prop_map(|(prefix, name, attrs, decl, kids, self_close)| Node::Elem {
                prefix,
                name,
                attrs,
                decl,
                kids,
                self_close,
            })
    })
}

fn ws() -> impl Strategy<Value = String> {
    "[ \t\n]{0,3}|\r\n"
}

fn render(n: &Node, ws: &[String], i: &mut usize, out: &mut String) {
    fn next(ws: &[String], i: &mut usize) -> String {
        *i += 1;
        ws[*i % ws.len()].clone()
    }
    match n {
        Node::Text(t) => out.push_str(t),
        Node::Comment(c) => out.push_str(&format!("<!--{c}-->")),
        Node::CData(c) => out.push_str(&format!("<![CDATA[{c}]]>")),
        Node::Pi(d) => out.push_str(&format!("<?pi {d}?>")),
        Node::Elem {
            prefix,
            name,
            attrs,
            decl,
            kids,
            self_close,
        } => {
            // Prefixed names declare their prefix on the element itself.
            let qn = match prefix {
                Some(p) => format!("{}:{name}", PREFIXES[*p]),
                None => name.clone(),
            };
            out.push('<');
            out.push_str(&qn);
            if let Some(p) = prefix {
                out.push_str(&format!(" xmlns:{}=\"urn:{}\"", PREFIXES[*p], PREFIXES[*p]));
            }
            if let Some(d) = decl {
                if Some(*d) != *prefix {
                    out.push_str(&format!(" xmlns:{}='urn:x{}'", PREFIXES[*d], d));
                }
            }
            let mut seen = std::collections::HashSet::new();
            for (an, av, dq) in attrs {
                if an.starts_with("xml") || !seen.insert(an.clone()) {
                    continue;
                }
                let (q, v) = if *dq {
                    ('"', av.replace('"', ""))
                } else {
                    ('\'', av.replace('\'', ""))
                };
                let (w1, w2) = (next(ws, i), next(ws, i));
                out.push_str(&format!(" {an}{w1}={w2}{q}{v}{q}"));
            }
            out.push_str(&next(ws, i));
            if *self_close && kids.is_empty() {
                out.push_str("/>");
                return;
            }
            out.push('>');
            let mut prev_text = false;
            for k in kids {
                // Adjacent text nodes would merge; separate them.
                let is_text = matches!(k, Node::Text(_));
                if is_text && prev_text {
                    out.push_str("<!---->");
                }
                prev_text = is_text;
                render(k, ws, i, out);
            }
            out.push_str(&format!("</{qn}{}>", next(ws, i).replace('\r', "")));
        }
    }
}

fn document() -> impl Strategy<Value = String> {
    (
        any::<bool>(),
        any::<bool>(),
        // The root is always an element (no filter: rejects would cap long runs).
        (name(), prop::collection::vec(node(), 0..6)).prop_map(|(name, kids)| Node::Elem {
            prefix: None,
            name,
            attrs: vec![],
            decl: Some(0),
            kids,
            self_close: false,
        }),
        prop::collection::vec(ws(), 1..6),
        "[ \n]{0,3}",
    )
        .prop_map(|(bom, decl, root, ws, tail)| {
            let mut s = String::new();
            if bom {
                s.push('\u{feff}');
            }
            if decl {
                s.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
            }
            s.push_str("<!-- prolog -->\n");
            let mut i = 0;
            render(&root, &ws, &mut i, &mut s);
            s.push_str(&tail);
            s
        })
}

proptest! {
    #![proptest_config(cfg())]

    #[test]
    fn generated_documents(src in document()) {
        prop_assert!(uppsala::parse(&src).is_ok(), "rejected generated doc {:?}: {:?}", src, uppsala::parse(&src).err());
        assert_invariants(&src)?;
    }
}

// ── Mutated corpus files ───────────────────────────────────────────────────

fn corpus() -> &'static Vec<String> {
    static C: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        imf_corpus()
            .into_iter()
            .filter(|p| {
                std::fs::metadata(p)
                    .map(|m| m.len() < 12_000)
                    .unwrap_or(false)
            })
            .map(|p| std::fs::read_to_string(p).unwrap())
            .collect()
    })
}

#[derive(Debug, Clone)]
enum Mutation {
    Insert(usize, &'static str),
    Delete(usize, usize),
    Replace(usize, &'static str),
}

const SNIPPETS: [&str; 16] = [
    "<",
    ">",
    "/",
    "&",
    ";",
    "\"",
    "'",
    "=",
    "<!--",
    "-->",
    "<![CDATA[",
    "]]>",
    "</x>",
    "<x>",
    "&amp;",
    "\r",
];

fn mutation() -> impl Strategy<Value = Mutation> {
    prop_oneof![
        (any::<usize>(), prop::sample::select(SNIPPETS.to_vec()))
            .prop_map(|(a, s)| Mutation::Insert(a, s)),
        (any::<usize>(), 1usize..12).prop_map(|(a, n)| Mutation::Delete(a, n)),
        (any::<usize>(), prop::sample::select(SNIPPETS.to_vec()))
            .prop_map(|(a, s)| Mutation::Replace(a, s)),
    ]
}

fn apply(src: &str, m: &Mutation) -> String {
    let at = |a: usize| {
        let mut i = a % (src.len() + 1);
        while !src.is_char_boundary(i) {
            i -= 1;
        }
        i
    };
    if src.is_empty() {
        return src.to_string();
    }
    match m {
        Mutation::Insert(a, s) => {
            let i = at(*a);
            format!("{}{s}{}", &src[..i], &src[i..])
        }
        Mutation::Delete(a, n) => {
            let i = at(*a);
            let mut j = (i + n).min(src.len());
            while !src.is_char_boundary(j) {
                j += 1;
            }
            format!("{}{}", &src[..i], &src[j..])
        }
        Mutation::Replace(a, s) => {
            let i = at(*a);
            let mut j = (i + 1).min(src.len());
            while !src.is_char_boundary(j) {
                j += 1;
            }
            format!("{}{s}{}", &src[..i], &src[j..])
        }
    }
}

proptest! {
    #![proptest_config(cfg())]

    #[test]
    fn mutated_corpus(file in any::<prop::sample::Index>(), muts in prop::collection::vec(mutation(), 1..4)) {
        let c = corpus();
        let mut src = c[file.index(c.len())].clone();
        for m in &muts {
            src = apply(&src, m);
        }
        assert_invariants(&src)?;
    }

    #[test]
    fn arbitrary_markup(src in ARBITRARY) {
        assert_invariants(&src)?;
        assert_invariants(&format!("<r>{src}</r>"))?;
    }
}

const ARBITRARY: &str =
    "(<|>|/|=|\"|'|&|;|!|\\?|-|\\[|\\]|a|b:c|xmlns|\\s|é|&amp;|<!--|-->|CDATA){0,40}";

/// How often the mutation strategies still produce well-formed XML (so the
/// invariant check, not just the no-panic check, is exercised).
#[test]
fn acceptance_rates() {
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;
    let mut runner = TestRunner::deterministic();
    let c = corpus();
    let strat = (
        any::<prop::sample::Index>(),
        prop::collection::vec(mutation(), 1..4),
    );
    let (mut ok, n) = (0, 2000);
    for _ in 0..n {
        let (file, muts) = strat.new_tree(&mut runner).unwrap().current();
        let mut src = c[file.index(c.len())].clone();
        for m in &muts {
            src = apply(&src, m);
        }
        ok += uppsala::parse(&src).is_ok() as usize;
    }
    let (mut ok2, n2) = (0, 2000);
    for _ in 0..n2 {
        let s = ARBITRARY.new_tree(&mut runner).unwrap().current();
        ok2 += uppsala::parse(&format!("<r>{s}</r>")).is_ok() as usize;
    }
    eprintln!("ACCEPTANCE mutated_corpus {ok}/{n}, arbitrary_markup wrapped in <r> {ok2}/{n2}");
}
