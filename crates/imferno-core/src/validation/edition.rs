//! Which edition of a spec is applied to a CPL or a package.

use crate::catalogue::st2067_2::St2067_2Edition;
use crate::cpl::CompositionPlaylist;

use super::CoreSpecTarget;

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
