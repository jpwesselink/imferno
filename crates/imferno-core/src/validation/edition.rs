//! Which edition of a spec is applied to a CPL or a package.
//!
//! One rule for every spec ([`resolve`]): the candidates are the editions
//! the document's namespace allows; evidence in the document that only some
//! editions' text defines narrows them. When one edition is left it is
//! applied. Otherwise the newest remaining candidate is applied and the
//! choice is reported (`IMFERNO:Package/EditionAssumed`), naming the
//! namespace seen and the edition used.

use crate::catalogue::st2067_2::St2067_2Edition;
use crate::catalogue::st2067_201::St2067_201Edition;
use crate::catalogue::st2067_3::St2067_3Edition;
use crate::catalogue::st377_1::St377_1Edition;
use crate::catalogue::st377_4::St377_4Edition;
use crate::cpl::{CompositionPlaylist, CplNamespace};
use crate::diagnostics::rule::Edition;

use super::{App2EEdition, CoreSpecTarget};

/// The edition applied to a document, and whether it was assumed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditionChoice<E> {
    /// The edition applied.
    pub edition: E,
    /// Why the edition was assumed rather than determined (the message of
    /// `IMFERNO:Package/EditionAssumed`); `None` when the namespace and the
    /// evidence leave one edition.
    pub assumed: Option<String>,
}

/// Something in a document that only some editions' text defines.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence<E> {
    /// The editions whose text defines it.
    pub editions: Vec<E>,
    /// What it is, for the report ("it declares the … namespace").
    pub what: String,
}

impl<E> Evidence<E> {
    /// Evidence for one edition.
    pub fn one(edition: E, what: impl Into<String>) -> Self {
        Self {
            editions: vec![edition],
            what: what.into(),
        }
    }
}

/// `ST2067-2` → `ST 2067-2`.
fn spec_name<E: Edition>() -> String {
    E::SPEC.id().replacen("ST", "ST ", 1)
}

fn labels<E: Edition>(editions: &[E]) -> String {
    let spec = spec_name::<E>();
    let names: Vec<String> = editions
        .iter()
        .map(|e| format!("{spec}:{}", e.label()))
        .collect();
    match names.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [init @ .., last] => format!("{} and {last}", init.join(", ")),
    }
}

/// The edition of `E`'s spec applied to `subject`.
///
/// `candidates` are the editions its namespace allows (`namespace` describes
/// that namespace for the report); an empty list allows every edition.
/// Every piece of `evidence` keeps only the candidates it names. One edition
/// left: it is applied. Several left, or the evidence contradicts itself or
/// the namespace: the newest is applied and [`EditionChoice::assumed`] says
/// why.
pub fn resolve<E: Edition + Ord>(
    subject: &str,
    namespace: &str,
    candidates: &[E],
    evidence: &[Evidence<E>],
) -> EditionChoice<E> {
    let spec = spec_name::<E>();
    // An empty list: the namespace is not one of any edition of the spec.
    let names_none = candidates.is_empty();
    let mut candidates: Vec<E> = if names_none {
        E::all().to_vec()
    } else {
        candidates.to_vec()
    };
    candidates.sort();
    candidates.dedup();
    if let [only] = candidates.as_slice() {
        // The namespace decides; evidence that names another edition is
        // reported, not followed.
        let against: Vec<String> = evidence
            .iter()
            .filter(|ev| !ev.editions.contains(only))
            .map(|ev| format!("{} ({})", ev.what, labels(&ev.editions)))
            .collect();
        return EditionChoice {
            edition: *only,
            assumed: (!against.is_empty()).then(|| {
                format!(
                    "{spec} edition of {subject} is {spec}:{} by {namespace}, but {} \
                     names another edition; {spec} rules were checked against {spec}:{} \
                     (the namespace's edition)",
                    only.label(),
                    against.join("; "),
                    only.label()
                )
            }),
        };
    }
    let left: Vec<E> = candidates
        .iter()
        .copied()
        .filter(|c| evidence.iter().all(|ev| ev.editions.contains(c)))
        .collect();
    if let [only] = left.as_slice() {
        return EditionChoice {
            edition: *only,
            assumed: None,
        };
    }
    let ns = if namespace.is_empty() {
        format!(
            "it carries no namespace, so it could be any of {}",
            labels(&candidates)
        )
    } else if names_none {
        format!("{namespace} is not the namespace of any {spec} edition")
    } else {
        format!("{namespace} is used by {}", labels(&candidates))
    };
    let (edition, why) = if left.is_empty() {
        let seen: Vec<String> = evidence
            .iter()
            .map(|ev| format!("{} ({})", ev.what, labels(&ev.editions)))
            .collect();
        (
            *candidates.last().expect("at least two candidates"),
            format!("{ns}, and the evidence does not agree: {}", seen.join("; ")),
        )
    } else if evidence.is_empty() {
        (
            *left.last().expect("at least two left"),
            format!("{ns}, and nothing in it tells them apart"),
        )
    } else {
        let seen: Vec<&str> = evidence.iter().map(|ev| ev.what.as_str()).collect();
        (
            *left.last().expect("at least two left"),
            format!(
                "{ns}, and {} does not tell {} apart",
                seen.join("; "),
                labels(&left)
            ),
        )
    };
    EditionChoice {
        edition,
        assumed: Some(format!(
            "{spec} edition of {subject} could not be determined: {why}; {spec} rules were \
             checked against {spec}:{} (the newest)",
            edition.label()
        )),
    }
}

/// Marker Label scope that only ST 2067-3:2020 defines (§6.14.1.2 Table 21).
pub(crate) const MARKER_SCOPE_2020: &str =
    "http://www.smpte-ra.org/schemas/2067-3/2020#standard-markers";

/// What names an ST 2067-3 edition for a CPL:
/// - the ST 2067-3 edition its ST 2067-2 edition cites: `core_pin`, else the
///   CPL's Core Constraints edition when determined ([`cpl_core_edition`]);
/// - for a CPL with IAB Virtual Tracks, the ST 2067-3 edition its
///   ST 2067-201 edition cites, when that edition is pinned or determined;
/// - a Marker Label in the `2020#standard-markers` scope, which only
///   ST 2067-3:2020 defines.
///
/// An assumed edition is not evidence.
pub fn cpl_evidence(
    cpl: &CompositionPlaylist,
    core_pin: Option<St2067_2Edition>,
    iab_pin: Option<St2067_201Edition>,
) -> Vec<Evidence<St2067_3Edition>> {
    let mut out = Vec::new();
    let core = core_pin.map(|e| (e, "the pinned")).or_else(|| {
        cpl_core_edition(cpl)
            .filter(|c| c.assumed.is_none())
            .map(|c| (c.edition, "its"))
    });
    if let Some((core, whose)) = core {
        let e = st2067_3_of_core(core);
        out.push(Evidence::one(
            e,
            format!(
                "{whose} ST 2067-2 edition, ST 2067-2:{}, cites ST 2067-3:{}",
                core.label(),
                e.label()
            ),
        ));
    }
    let has_iab = cpl
        .segment_list
        .segments
        .iter()
        .any(|s| !s.sequence_list.iab_sequences.is_empty());
    if has_iab {
        let iab = cpl_iab_edition(cpl, iab_pin);
        if iab.assumed.is_none() {
            let e = st2067_3_of_iab(iab.edition);
            out.push(Evidence::one(
                e,
                format!(
                    "its ST 2067-201 edition, ST 2067-201:{}, cites ST 2067-3:{}",
                    iab.edition.label(),
                    e.label()
                ),
            ));
        }
    }
    let uses_2020_scope = cpl.segment_list.segments.iter().any(|s| {
        s.sequence_list.marker_sequences.iter().any(|seq| {
            seq.resource_list
                .resources
                .iter()
                .flat_map(|r| r.markers.iter())
                .any(|m| m.label.effective_scope() == MARKER_SCOPE_2020)
        })
    });
    if uses_2020_scope {
        out.push(Evidence::one(
            St2067_3Edition::V2020,
            format!("it uses the Marker Label scope {MARKER_SCOPE_2020}, which only ST 2067-3:2020 defines"),
        ));
    }
    out
}

/// The ST 2067-3 edition whose rules are applied to a CPL when the caller
/// pins none (see [`resolve`]). The 2013 namespace is ST 2067-3:2013; 2016
/// and 2020 share the 2016 namespace; an ST 429-7 (DCI) or unknown
/// namespace is not an ST 2067-3 namespace, so every edition is a
/// candidate and the newest is applied, announced.
pub fn cpl_edition(
    cpl: &CompositionPlaylist,
    core_pin: Option<St2067_2Edition>,
    iab_pin: Option<St2067_201Edition>,
) -> EditionChoice<St2067_3Edition> {
    use St2067_3Edition::{V2013, V2016, V2020};
    let candidates: &[St2067_3Edition] = match &cpl.namespace {
        CplNamespace::Smpte2067_3_2013 => &[V2013],
        CplNamespace::Smpte2067_3_2016 => &[V2016, V2020],
        CplNamespace::Dci429_7 | CplNamespace::Unknown(_) => &[],
    };
    let namespace = match &cpl.namespace {
        CplNamespace::Dci429_7 => format!("its namespace {} (ST 429-7)", cpl.namespace),
        CplNamespace::Unknown(ns) if ns.is_empty() => "its (empty) namespace".into(),
        ns => format!("its namespace {ns}"),
    };
    resolve(
        &format!("CPL {}", cpl.id),
        &namespace,
        candidates,
        &cpl_evidence(cpl, core_pin, iab_pin),
    )
}

/// The ST 2067-3 edition an ST 2067-2 edition cites (2013 §6.1 uses the
/// 2067-3:2013 namespace; 2016 and 2020 §3 cite ST 2067-3:2016 / :2020).
fn st2067_3_of_core(core: St2067_2Edition) -> St2067_3Edition {
    match core {
        St2067_2Edition::V2013 => St2067_3Edition::V2013,
        St2067_2Edition::V2016 => St2067_3Edition::V2016,
        St2067_2Edition::V2020 => St2067_3Edition::V2020,
    }
}

/// The ST 2067-3 edition an ST 2067-201 edition cites (2019 §3:
/// ST 2067-3:2016; 2021 and 2026: ST 2067-3:2020).
fn st2067_3_of_iab(iab: St2067_201Edition) -> St2067_3Edition {
    match iab {
        St2067_201Edition::V2019 => St2067_3Edition::V2016,
        St2067_201Edition::V2021 | St2067_201Edition::V2026 => St2067_3Edition::V2020,
    }
}

/// The ST 377-1 and ST 377-4 editions applied to an MXF track file.
///
/// MXF carries no namespace; the editions come from the dated normative
/// references of the specs whose rules apply to the file: ST 2067-2 (every
/// edition references ST 377-1:2011; 2020 references ST 377-4:2012, 2013
/// and 2016 reference "ST 377-4:2011", which is not an edition, so they name
/// none) and, for an IAB track file, ST 2067-201 (2019: 377-1:2011 and
/// 377-4:2012; 2021: 377-1:2019 and 377-4:2012; 2026: 377-1:2019 and
/// 377-4:2021). See [`resolve`].
pub fn mxf_editions(
    subject: &str,
    core: St2067_2Edition,
    iab: Option<St2067_201Edition>,
) -> (EditionChoice<St377_1Edition>, EditionChoice<St377_4Edition>) {
    use St2067_201Edition as Iab;
    let core_label = core.label();
    let mut e1 = vec![Evidence::one(
        St377_1Edition::V2011,
        format!("ST 2067-2:{core_label} references ST 377-1:2011"),
    )];
    let mut e4 = Vec::new();
    if core == St2067_2Edition::V2020 {
        e4.push(Evidence::one(
            St377_4Edition::V2012,
            "ST 2067-2:2020 references ST 377-4:2012",
        ));
    }
    if let Some(iab) = iab {
        let (one, four) = match iab {
            Iab::V2019 => (St377_1Edition::V2011, St377_4Edition::V2012),
            Iab::V2021 => (St377_1Edition::V2019, St377_4Edition::V2012),
            Iab::V2026 => (St377_1Edition::V2019, St377_4Edition::V2021),
        };
        let iab_label = iab.label();
        e1.push(Evidence::one(
            one,
            format!(
                "ST 2067-201:{iab_label} references ST 377-1:{}",
                one.label()
            ),
        ));
        e4.push(Evidence::one(
            four,
            format!(
                "ST 2067-201:{iab_label} references ST 377-4:{}",
                four.label()
            ),
        ));
    }
    let mut four = resolve(subject, "", St377_4Edition::ALL, &e4);
    if core != St2067_2Edition::V2020 {
        four.assumed = four.assumed.map(|why| {
            why.replacen(
                "could not be determined: ",
                &format!(
                    "could not be determined: ST 2067-2:{core_label} references ST 377-4:2011, \
                     which is not an ST 377-4 edition; "
                ),
                1,
            )
        });
    }
    (resolve(subject, "", St377_1Edition::ALL, &e1), four)
}

/// The ST 2067-201 edition of the IAB rules applied to a CPL (and to the
/// track files its IAB Virtual Tracks reference): `pin` when the caller
/// gives one. Every edition (2019, 2021, 2026) uses the 2019 namespace, and
/// no App #2E or ST 2067-2 edition references a dated ST 2067-201, so
/// nothing in a CPL tells them apart: the newest is applied, announced.
pub fn cpl_iab_edition(
    cpl: &CompositionPlaylist,
    pin: Option<St2067_201Edition>,
) -> EditionChoice<St2067_201Edition> {
    if let Some(edition) = pin {
        return EditionChoice {
            edition,
            assumed: None,
        };
    }
    resolve(
        &format!("CPL {}", cpl.id),
        "its IAB namespace http://www.smpte-ra.org/ns/2067-201/2019",
        St2067_201Edition::ALL,
        &[],
    )
}

/// The ST 2067-2 editions a CPL namespace allows: ST 2067-2:2013 uses
/// ST 2067-3:2013 (2013 §6.1); 2016 and 2020 both use the ST 2067-3:2016
/// namespace. `None` for a namespace that is not an ST 2067-3 namespace (no
/// Core Constraints validator runs).
fn core_candidates(ns: &CplNamespace) -> Option<&'static [St2067_2Edition]> {
    use St2067_2Edition::{V2013, V2016, V2020};
    match ns {
        CplNamespace::Smpte2067_3_2013 => Some(&[V2013]),
        CplNamespace::Smpte2067_3_2016 => Some(&[V2016, V2020]),
        CplNamespace::Dci429_7 | CplNamespace::Unknown(_) => None,
    }
}

/// The ST 2067-2 edition an App #2E edition references: ST 2067-21:2014 and
/// :2016 apply ST 2067-20:2013 / :2016, which reference ST 2067-2:2013 /
/// :2016; ST 2067-21:2020, :2023 and :2026 reference ST 2067-2:2020.
fn core_edition_of_app2e(app: App2EEdition) -> St2067_2Edition {
    match app {
        App2EEdition::V2014 => St2067_2Edition::V2013,
        App2EEdition::V2016 => St2067_2Edition::V2016,
        App2EEdition::V2020 | App2EEdition::V2023 | App2EEdition::V2026 => St2067_2Edition::V2020,
    }
}

/// What in a CPL names an ST 2067-2 edition:
/// - the namespace of its `ApplicationIdentification` element (ST 2067-2
///   §6.1 defines it in its edition's Core Constraints namespace), else every
///   Core Constraints namespace it declares;
/// - every App #2E edition its `ApplicationIdentification` lists, by the
///   ST 2067-2 edition that edition references.
pub fn cpl_core_evidence(cpl: &CompositionPlaylist) -> Vec<Evidence<St2067_2Edition>> {
    let mut out = Vec::new();
    if let Some(e) = application_identification_core_edition(cpl) {
        out.push(Evidence::one(
            e,
            format!(
                "its ApplicationIdentification element is in the ST 2067-2:{} Core Constraints \
                 namespace",
                e.label()
            ),
        ));
    } else {
        for e in declared_core_constraints_editions(cpl) {
            out.push(Evidence::one(
                e,
                format!(
                    "it declares the ST 2067-2:{} Core Constraints namespace",
                    e.label()
                ),
            ));
        }
    }
    let app_ids = cpl
        .extension_properties
        .as_ref()
        .and_then(|e| e.application_identification.as_deref())
        .unwrap_or_default();
    for app in app_ids
        .split_whitespace()
        .filter_map(App2EEdition::from_application_identification)
    {
        let e = core_edition_of_app2e(app);
        out.push(Evidence::one(
            e,
            format!(
                "its ApplicationIdentification lists ST 2067-21:{}, which references \
                 ST 2067-2:{}",
                app.label(),
                e.label()
            ),
        ));
    }
    out
}

/// The ST 2067-2 edition a CPL's Core Constraints are checked against when
/// the caller pins none (see [`resolve`]); `None` when the CPL namespace is
/// not an ST 2067-3 namespace.
pub fn cpl_core_edition(cpl: &CompositionPlaylist) -> Option<EditionChoice<St2067_2Edition>> {
    let candidates = core_candidates(&cpl.namespace)?;
    Some(resolve(
        &format!("CPL {}", cpl.id),
        &format!("its namespace {}", cpl.namespace),
        candidates,
        &cpl_core_evidence(cpl),
    ))
}

/// The ST 2067-2 edition of a Core Constraints namespace URI.
fn core_edition_of_namespace(uri: &str) -> Option<St2067_2Edition> {
    [
        (CoreSpecTarget::St2067_2_2013, St2067_2Edition::V2013),
        (CoreSpecTarget::St2067_2_2016, St2067_2Edition::V2016),
        (CoreSpecTarget::St2067_2_2020, St2067_2Edition::V2020),
    ]
    .into_iter()
    .find(|(t, _)| t.namespace_uri() == uri)
    .map(|(_, e)| e)
}

/// The ST 2067-2 edition of the namespace of the CPL's
/// `ApplicationIdentification` element (ST 2067-2 §6.1 defines the element in
/// the Core Constraints namespace of its edition), or `None` when the element
/// is absent, its namespace cannot be resolved, or it is not a Core
/// Constraints namespace.
pub fn application_identification_core_edition(
    cpl: &CompositionPlaylist,
) -> Option<St2067_2Edition> {
    scan_namespaces(cpl.source_xml.as_deref()?)?
        .application_identification
        .as_deref()
        .and_then(core_edition_of_namespace)
}

/// The ST 2067-2 editions whose Core Constraints namespace the CPL declares
/// (`xmlns:cc="http://www.smpte-ra.org/ns/2067-2/2020"`), oldest first;
/// empty when it declares none or its source XML was not kept.
pub fn declared_core_constraints_editions(cpl: &CompositionPlaylist) -> Vec<St2067_2Edition> {
    let Some(ns) = cpl.source_xml.as_deref().and_then(scan_namespaces) else {
        return Vec::new();
    };
    let mut editions: Vec<St2067_2Edition> = ns
        .declared
        .iter()
        .filter_map(|uri| core_edition_of_namespace(uri))
        .collect();
    editions.sort();
    editions.dedup();
    editions
}

/// Namespace facts of an XML document, read with a namespace-aware parser.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct XmlNamespaces {
    /// Every namespace URI declared (`xmlns`, `xmlns:p`) in the document.
    declared: Vec<String>,
    /// The namespace of the first `ApplicationIdentification` element,
    /// resolved in that element's scope.
    application_identification: Option<String>,
}

/// Read the namespace declarations of `xml` and the namespace of its first
/// `ApplicationIdentification` element; `None` when the XML is not
/// well-formed.
fn scan_namespaces(xml: &str) -> Option<XmlNamespaces> {
    use quick_xml::events::Event;
    use quick_xml::name::ResolveResult;

    let mut reader = quick_xml::NsReader::from_str(xml);
    let mut out = XmlNamespaces::default();
    loop {
        match reader.read_resolved_event().ok()? {
            (ns, Event::Start(e)) | (ns, Event::Empty(e)) => {
                for attr in e.attributes().flatten() {
                    let key = attr.key.as_ref();
                    if key == "xmlns" || key.starts_with("xmlns:") {
                        if let Ok(v) = attr.normalized_value(quick_xml::XmlVersion::Implicit1_0) {
                            out.declared.push(v.trim().to_string());
                        }
                    }
                }
                if out.application_identification.is_none()
                    && e.local_name().as_ref() == "ApplicationIdentification"
                {
                    if let ResolveResult::Bound(n) = ns {
                        out.application_identification = Some(n.as_ref().trim().to_string());
                    }
                }
            }
            (_, Event::Eof) => return Some(out),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cpl(xml: &str) -> CompositionPlaylist {
        let mut c = crate::cpl::parse_cpl(BASE).expect("base CPL parses");
        c.source_xml = Some(xml.to_string());
        c
    }

    /// A minimal 2016-namespace CPL the parser accepts.
    const BASE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2016">
  <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
  <IssueDate>2024-01-01T00:00:00Z</IssueDate>
  <ContentTitle>T</ContentTitle>
  <EditRate>24 1</EditRate>
  <SegmentList><Segment><Id>urn:uuid:00000000-0000-0000-0000-000000000002</Id><SequenceList/></Segment></SegmentList>
</CompositionPlaylist>"#;

    const NS_2016: &str = "http://www.smpte-ra.org/schemas/2067-2/2016";
    const NS_2020: &str = "http://www.smpte-ra.org/ns/2067-2/2020";

    fn with_ext(root_attrs: &str, ext: &str) -> String {
        format!(
            r#"<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2016" {root_attrs}>
  <ExtensionProperties>{ext}</ExtensionProperties>
</CompositionPlaylist>"#
        )
    }

    #[test]
    fn app_id_namespace_from_a_double_quoted_prefix() {
        let xml = with_ext(
            &format!(r#"xmlns:cc="{NS_2020}""#),
            "<cc:ApplicationIdentification>x</cc:ApplicationIdentification>",
        );
        assert_eq!(
            application_identification_core_edition(&cpl(&xml)),
            Some(St2067_2Edition::V2020)
        );
    }

    #[test]
    fn app_id_namespace_from_a_single_quoted_prefix() {
        let xml = with_ext(
            &format!("xmlns:cc='{NS_2020}'"),
            "<cc:ApplicationIdentification>x</cc:ApplicationIdentification>",
        );
        assert_eq!(
            application_identification_core_edition(&cpl(&xml)),
            Some(St2067_2Edition::V2020)
        );
    }

    /// The prefix is resolved in the element's own scope, not from the
    /// first `xmlns:cc` in the document.
    #[test]
    fn app_id_prefix_is_resolved_in_scope() {
        let xml = with_ext(
            "",
            &format!(
                r#"<x:Other xmlns:x="urn:x" xmlns:cc="{NS_2016}"/>
<cc:ApplicationIdentification xmlns:cc="{NS_2020}">x</cc:ApplicationIdentification>"#
            ),
        );
        assert_eq!(
            application_identification_core_edition(&cpl(&xml)),
            Some(St2067_2Edition::V2020)
        );
        let xml = with_ext(
            "",
            &format!(
                r#"<x:Other xmlns:x="urn:x" xmlns:cc="{NS_2016}"><cc:Y/></x:Other>
<x:Wrap xmlns:x="urn:x" xmlns:cc="{NS_2020}"><cc:ApplicationIdentification>x</cc:ApplicationIdentification></x:Wrap>"#
            ),
        );
        assert_eq!(
            application_identification_core_edition(&cpl(&xml)),
            Some(St2067_2Edition::V2020)
        );
    }

    /// A default namespace on an ancestor applies to an unprefixed element.
    #[test]
    fn app_id_in_an_inherited_default_namespace() {
        let xml = with_ext(
            "",
            &format!(
                r#"<Wrap xmlns="{NS_2016}"><ApplicationIdentification>x</ApplicationIdentification></Wrap>"#
            ),
        );
        assert_eq!(
            application_identification_core_edition(&cpl(&xml)),
            Some(St2067_2Edition::V2016)
        );
    }

    /// Unprefixed in the CPL namespace: not a Core Constraints namespace.
    #[test]
    fn app_id_in_the_cpl_namespace_names_no_core_edition() {
        let xml = with_ext(
            "",
            "<ApplicationIdentification>x</ApplicationIdentification>",
        );
        assert_eq!(application_identification_core_edition(&cpl(&xml)), None);
    }

    #[test]
    fn declared_core_namespaces_in_either_quote_style() {
        let xml = with_ext(&format!(r#"xmlns:a='{NS_2016}' xmlns:b="{NS_2020}""#), "");
        assert_eq!(
            declared_core_constraints_editions(&cpl(&xml)),
            vec![St2067_2Edition::V2016, St2067_2Edition::V2020]
        );
    }

    fn with_namespace(ns: CplNamespace) -> CompositionPlaylist {
        let mut c = crate::cpl::parse_cpl(BASE).expect("base CPL parses");
        c.namespace = ns;
        c
    }

    /// ST 2067-2:2016 cites ST 2067-3:2016 and :2020 cites :2020: a CPL whose
    /// Core Constraints edition is determined (or pinned) determines its
    /// ST 2067-3 edition.
    #[test]
    fn st2067_3_follows_the_dated_core_reference() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-data/IAB/CompleteIMP/CPL_e0265fda-cb35-4e35-a4e4-4f44d82d2a52.xml"
        );
        let iab = crate::cpl::parse_cpl(&std::fs::read_to_string(path).unwrap()).unwrap();
        let c = cpl_edition(&iab, None, None);
        assert_eq!((c.edition, c.assumed), (St2067_3Edition::V2016, None));
        // ST 2067-201:2019 cites ST 2067-3:2016; :2021 cites :2020, against
        // the 2016 core: announced.
        let c = cpl_edition(&iab, None, Some(St2067_201Edition::V2019));
        assert_eq!((c.edition, c.assumed), (St2067_3Edition::V2016, None));
        let c = cpl_edition(&iab, None, Some(St2067_201Edition::V2021));
        assert_eq!(c.edition, St2067_3Edition::V2020);
        assert!(c.assumed.unwrap().contains("ST 2067-201:2021, cites ST 2067-3:2020"));

        let bare = with_namespace(CplNamespace::Smpte2067_3_2016);
        let c = cpl_edition(&bare, Some(St2067_2Edition::V2016), None);
        assert_eq!((c.edition, c.assumed), (St2067_3Edition::V2016, None));
    }

    /// The 2020 marker scope against a 2016 core: the conflict is announced.
    #[test]
    fn st2067_3_2020_marker_scope_against_a_2016_core_is_announced() {
        let mut bare = with_namespace(CplNamespace::Smpte2067_3_2016);
        bare.source_xml = None;
        let xml = BASE.replace(
            "<SequenceList/>",
            r#"<SequenceList><MarkerSequence><Id>urn:uuid:00000000-0000-0000-0000-000000000003</Id><TrackId>urn:uuid:00000000-0000-0000-0000-000000000004</TrackId><ResourceList><Resource><Id>urn:uuid:00000000-0000-0000-0000-000000000005</Id><IntrinsicDuration>10</IntrinsicDuration><Marker><Label scope="http://www.smpte-ra.org/schemas/2067-3/2020#standard-markers">FFEI</Label><Offset>0</Offset></Marker></Resource></ResourceList></MarkerSequence></SequenceList>"#,
        );
        let cpl = crate::cpl::parse_cpl(&xml).unwrap();
        let c = cpl_edition(&cpl, None, None);
        assert_eq!((c.edition, c.assumed), (St2067_3Edition::V2020, None));
        let c = cpl_edition(&cpl, Some(St2067_2Edition::V2016), None);
        assert_eq!(c.edition, St2067_3Edition::V2020);
        let why = c.assumed.expect("conflict announced");
        assert!(why.contains("the evidence does not agree"), "{why}");
    }

    #[test]
    fn st2067_3_2013_namespace_is_determined() {
        let c = cpl_edition(&with_namespace(CplNamespace::Smpte2067_3_2013), None, None);
        assert_eq!(c.edition, St2067_3Edition::V2013);
        assert_eq!(c.assumed, None);
    }

    /// 2016 and 2020 share the namespace; nothing tells them apart.
    #[test]
    fn st2067_3_2016_namespace_assumes_2020_and_says_so() {
        let c = cpl_edition(&with_namespace(CplNamespace::Smpte2067_3_2016), None, None);
        assert_eq!(c.edition, St2067_3Edition::V2020);
        let why = c.assumed.expect("announced");
        assert!(
            why.contains("its namespace http://www.smpte-ra.org/schemas/2067-3/2016 is used by ST 2067-3:2016 and ST 2067-3:2020"),
            "{why}"
        );
        assert!(
            why.ends_with("checked against ST 2067-3:2020 (the newest)"),
            "{why}"
        );
    }

    /// An ST 429-7 (DCI) or unknown namespace is not an ST 2067-3
    /// namespace: the newest edition, announced (was 2013 / 2016 silently).
    #[test]
    fn non_st2067_3_namespaces_assume_the_newest_and_say_so() {
        for ns in [
            CplNamespace::Dci429_7,
            CplNamespace::Unknown("urn:example".into()),
            CplNamespace::Unknown(String::new()),
        ] {
            let c = cpl_edition(&with_namespace(ns.clone()), None, None);
            assert_eq!(c.edition, St2067_3Edition::V2020, "{ns:?}");
            let why = c.assumed.expect("announced");
            assert!(
                why.contains("is not the namespace of any ST 2067-3 edition"),
                "{why}"
            );
        }
    }

    #[test]
    fn evidence_against_the_namespace_edition_is_announced() {
        use St2067_2Edition::{V2013, V2020};
        let c = resolve("X", "ns", &[V2013], &[Evidence::one(V2013, "a")]);
        assert_eq!((c.edition, c.assumed), (V2013, None));
        let c = resolve("X", "ns", &[V2013], &[Evidence::one(V2020, "b")]);
        assert_eq!(c.edition, V2013, "the namespace decides");
        let why = c.assumed.expect("the conflict is announced");
        assert!(why.contains("b (ST 2067-2:2020)"), "{why}");
        assert!(why.contains("checked against ST 2067-2:2013"), "{why}");
    }

    #[test]
    fn evidence_narrows_and_conflicts_are_announced() {
        use St2067_2Edition::{V2016, V2020};
        let c = resolve("X", "ns", &[V2016, V2020], &[Evidence::one(V2016, "a")]);
        assert_eq!((c.edition, c.assumed), (V2016, None));
        let c = resolve(
            "X",
            "ns",
            &[V2016, V2020],
            &[Evidence::one(V2016, "a"), Evidence::one(V2020, "b")],
        );
        assert_eq!(c.edition, V2020);
        let why = c.assumed.unwrap();
        assert!(
            why.contains("the evidence does not agree: a (ST 2067-2:2016); b (ST 2067-2:2020)"),
            "{why}"
        );
    }

    /// MXF editions come from the dated references of the specs that apply.
    #[test]
    fn mxf_editions_follow_the_dated_references() {
        use St2067_201Edition as Iab;
        // Every ST 2067-2 edition references ST 377-1:2011; 2020 references
        // ST 377-4:2012.
        let (one, four) = mxf_editions("f", St2067_2Edition::V2020, None);
        assert_eq!((one.edition, one.assumed), (St377_1Edition::V2011, None));
        assert_eq!((four.edition, four.assumed), (St377_4Edition::V2012, None));
        // 2016 references "ST 377-4:2011", not an edition: newest, announced.
        let (_, four) = mxf_editions("f", St2067_2Edition::V2016, None);
        assert_eq!(four.edition, St377_4Edition::V2021);
        let why = four.assumed.unwrap();
        assert!(why.contains("it carries no namespace"), "{why}");
        // IAB 2026 references ST 377-1:2019 / 377-4:2021; with ST 2067-2's
        // 377-1:2011 that conflicts: newest, announced.
        let (one, four) = mxf_editions("f", St2067_2Edition::V2020, Some(Iab::V2026));
        assert_eq!(one.edition, St377_1Edition::V2019);
        assert!(one.assumed.unwrap().contains("the evidence does not agree"));
        assert_eq!(four.edition, St377_4Edition::V2021);
        assert!(four.assumed.is_some());
        // IAB 2019 agrees with ST 2067-2:2020.
        let (one, four) = mxf_editions("f", St2067_2Edition::V2020, Some(Iab::V2019));
        assert_eq!((one.edition, one.assumed), (St377_1Edition::V2011, None));
        assert_eq!((four.edition, four.assumed), (St377_4Edition::V2012, None));
    }

    #[test]
    fn known_core_namespaces() {
        assert_eq!(
            core_edition_of_namespace(NS_2016),
            Some(St2067_2Edition::V2016)
        );
        assert_eq!(core_edition_of_namespace("urn:x"), None);
    }
}
