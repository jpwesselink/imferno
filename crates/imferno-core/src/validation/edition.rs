//! Which edition of a spec is applied to a CPL or a package.
//!
//! One rule for every spec ([`resolve`]): the candidates are the editions
//! the document's namespace allows; evidence in the document that only some
//! editions' text defines narrows them. When one edition is left it is
//! applied. Otherwise the newest remaining candidate is applied and the
//! choice is reported (`IMFERNO:Package/EditionAssumed`), naming the
//! namespace seen and the edition used.

use crate::catalogue::st2067_2::St2067_2Edition;
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
    let mut candidates: Vec<E> = if candidates.is_empty() {
        E::all().to_vec()
    } else {
        candidates.to_vec()
    };
    candidates.sort();
    candidates.dedup();
    if let [only] = candidates.as_slice() {
        return EditionChoice {
            edition: *only,
            assumed: None,
        };
    }
    let left: Vec<E> = candidates
        .iter()
        .copied()
        .filter(|c| evidence.iter().all(|ev| ev.editions.contains(c)))
        .collect();
    let (edition, why) = match left.as_slice() {
        [only] => {
            return EditionChoice {
                edition: *only,
                assumed: None,
            }
        }
        [] => {
            let seen: Vec<String> = evidence
                .iter()
                .map(|ev| format!("{} ({})", ev.what, labels(&ev.editions)))
                .collect();
            (
                *candidates.last().expect("at least two candidates"),
                format!(
                    "{namespace} allows {}, and the evidence does not agree: {}",
                    labels(&candidates),
                    seen.join("; ")
                ),
            )
        }
        several => (
            *several.last().expect("at least two left"),
            if evidence.is_empty() {
                format!(
                    "{namespace} is used by {}, and nothing in it tells them apart",
                    labels(several)
                )
            } else {
                let seen: Vec<&str> = evidence.iter().map(|ev| ev.what.as_str()).collect();
                format!(
                    "{namespace} is used by {}, and {} does not tell {} apart",
                    labels(&candidates),
                    seen.join("; "),
                    labels(several)
                )
            },
        ),
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

    #[test]
    fn known_core_namespaces() {
        assert_eq!(
            core_edition_of_namespace(NS_2016),
            Some(St2067_2Edition::V2016)
        );
        assert_eq!(core_edition_of_namespace("urn:x"), None);
    }
}
