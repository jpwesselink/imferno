//! Rule table: XML Schema layer.
//!
//! Generated from the pre-C0 per-edition code enums; edit by hand from now on.

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::{absent, row, AnyRule, Rule, Spec};
use crate::diagnostics::{Category, Severity};

crate::diagnostics::rule::editions! {
    /// Editions of XML Schema layer imferno knows.
    pub enum XsdSchema for Spec::Xsd {
        Cpl2013 => "CPL-2013",
        Cpl2016 => "CPL-2016",
        Pkl2016 => "PKL-2016",
        DciPkl2007 => "DCI-PKL-2007",
        Scm2018 => "SCM-2018",
        Opl2014 => "OPL-2014",
        Custom => "custom",
        Unvendored => "unvendored",
    }
}

type E = XsdSchema;

/// A required element is absent (XSD minOccurs violated).
pub static ElementMissing: Rule<E> = Rule {
    name: "ElementMissing",
    category: Category::Schema,
    description: "A required element is absent (XSD minOccurs violated).",
    example: Some(
        "<CompositionPlaylist>…</CompositionPlaylist>  <!-- missing required <EditRate> -->",
    ),
    editions: &[
        row(E::Cpl2013, "", Severity::Error),
        row(E::Cpl2016, "", Severity::Error),
        row(E::Pkl2016, "", Severity::Error),
        row(E::DciPkl2007, "", Severity::Error),
        row(E::Scm2018, "", Severity::Error),
        row(E::Opl2014, "", Severity::Error),
        row(E::Custom, "", Severity::Error),
        absent(E::Unvendored),
    ],
};

/// An element appears outside its declared content model.
pub static UnexpectedElement: Rule<E> = Rule {
    name: "UnexpectedElement",
    category: Category::Schema,
    description: "An element appears outside its declared content model.",
    example: Some("<CompositionPlaylist><BogusTag/>…</CompositionPlaylist>  <!-- not in the schema sequence -->"),
    editions: &[
        row(E::Cpl2013, "", Severity::Error),
        row(E::Cpl2016, "", Severity::Error),
        row(E::Pkl2016, "", Severity::Error),
        row(E::DciPkl2007, "", Severity::Error),
        row(E::Scm2018, "", Severity::Error),
        row(E::Opl2014, "", Severity::Error),
        row(E::Custom, "", Severity::Error),
        absent(E::Unvendored),
    ],
};

/// A value violates an xs:pattern facet on its type.
pub static PatternInvalid: Rule<E> = Rule {
    name: "PatternInvalid",
    category: Category::Schema,
    description: "A value violates an xs:pattern facet on its type.",
    example: Some("<Id>not-a-uuid</Id>  <!-- dcml:UUIDType pattern expects 'urn:uuid:...' -->"),
    editions: &[
        row(E::Cpl2013, "", Severity::Error),
        row(E::Cpl2016, "", Severity::Error),
        row(E::Pkl2016, "", Severity::Error),
        row(E::DciPkl2007, "", Severity::Error),
        row(E::Scm2018, "", Severity::Error),
        row(E::Opl2014, "", Severity::Error),
        row(E::Custom, "", Severity::Error),
        absent(E::Unvendored),
    ],
};

/// A value violates its declared XSD type (built-in or restriction).
pub static TypeInvalid: Rule<E> = Rule {
    name: "TypeInvalid",
    category: Category::Schema,
    description: "A value violates its declared XSD type (built-in or restriction).",
    example: Some("<Size>not-a-number</Size>  <!-- declared xs:positiveInteger -->"),
    editions: &[
        row(E::Cpl2013, "", Severity::Error),
        row(E::Cpl2016, "", Severity::Error),
        row(E::Pkl2016, "", Severity::Error),
        row(E::DciPkl2007, "", Severity::Error),
        row(E::Scm2018, "", Severity::Error),
        row(E::Opl2014, "", Severity::Error),
        row(E::Custom, "", Severity::Error),
        absent(E::Unvendored),
    ],
};

/// An XSD constraint failed; see message for details.
pub static SchemaConstraintFailed: Rule<E> = Rule {
    name: "SchemaConstraintFailed",
    category: Category::Schema,
    description: "An XSD constraint failed; see message for details.",
    example: Some("<EditRate>24 0</EditRate>  <!-- denominator violates a derived restriction -->"),
    editions: &[
        row(E::Cpl2013, "", Severity::Error),
        row(E::Cpl2016, "", Severity::Error),
        row(E::Pkl2016, "", Severity::Error),
        row(E::DciPkl2007, "", Severity::Error),
        row(E::Scm2018, "", Severity::Error),
        row(E::Opl2014, "", Severity::Error),
        row(E::Custom, "", Severity::Error),
        absent(E::Unvendored),
    ],
};

/// XSD validation was skipped: no vendored schema covers this document's namespace.
pub static NotValidated: Rule<E> = Rule {
    name: "NotValidated",
    category: Category::Schema,
    description: "XSD validation was skipped: no vendored schema covers this document's namespace.",
    example: Some("<CompositionPlaylist xmlns=\"http://www.smpte-ra.org/schemas/429-7/2006/CPL\">  <!-- DCI-era namespace; no vendored XSD -->"),
    editions: &[
        absent(E::Cpl2013),
        absent(E::Cpl2016),
        absent(E::Pkl2016),
        absent(E::DciPkl2007),
        absent(E::Scm2018),
        absent(E::Opl2014),
        absent(E::Custom),
        row(E::Unvendored, "", Severity::Info),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[
    &ElementMissing,
    &UnexpectedElement,
    &PatternInvalid,
    &TypeInvalid,
    &SchemaConstraintFailed,
    &NotValidated,
];
