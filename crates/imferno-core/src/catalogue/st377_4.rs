//! Rule table: SMPTE ST 377-4 (MXF Multichannel Audio Labeling).
//!
//! Generated from the pre-C0 per-edition code enums; edit by hand from now on.

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::{row, AnyRule, Rule, Spec};
use crate::diagnostics::{Category, Severity};

crate::diagnostics::rule::editions! {
    /// Editions of SMPTE ST 377-4 (MXF Multichannel Audio Labeling) imferno knows.
    pub enum St377_4Edition for Spec::St377_4 {
        V2012 => "2012",
    }
}

type E = St377_4Edition;

/// Every MCA sub-descriptor shall carry an MCALinkID (§6.3 Table 3, Req).
pub static MCALinkIDMissing: Rule<E> = Rule {
    name: "MCALinkIDMissing",
    category: Category::Audio,
    description: "Every MCA sub-descriptor shall carry an MCALinkID (§6.3 Table 3, Req).",
    example: Some("<AudioChannelLabelSubDescriptor>…</AudioChannelLabelSubDescriptor>  <!-- no <MCALinkID> child -->"),
    editions: &[
        row(E::V2012, "6.3", Severity::Error),
    ],
};

/// AudioChannelLabelSubDescriptor SoundfieldGroupLinkID shall equal the SoundfieldGroup's MCALinkID (§6.4.1).
pub static SoundfieldGroupLinkIDMismatch: Rule<E> = Rule {
    name: "SoundfieldGroupLinkIDMismatch",
    category: Category::Audio,
    description: "AudioChannelLabelSubDescriptor SoundfieldGroupLinkID shall equal the SoundfieldGroup's MCALinkID (§6.4.1).",
    example: Some("<AudioChannelLabelSubDescriptor><SoundfieldGroupLinkID>urn:uuid:abc…</SoundfieldGroupLinkID></AudioChannelLabelSubDescriptor>  <!-- but the parent SoundfieldGroupLabelSubDescriptor MCALinkID = urn:uuid:def… -->"),
    editions: &[
        row(E::V2012, "6.4.1", Severity::Error),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[&MCALinkIDMissing, &SoundfieldGroupLinkIDMismatch];
