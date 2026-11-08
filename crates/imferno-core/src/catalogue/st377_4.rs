//! Rule table: SMPTE ST 377-4 (MXF Multichannel Audio Labeling).
//!
//! Every row checked against the text of its edition (C1/C4; 2021 rows C17).

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::Strength::Shall;
use crate::diagnostics::rule::{checked, AnyRule, Rule, Spec};
use crate::diagnostics::Category;

crate::diagnostics::rule::editions! {
    /// Editions of SMPTE ST 377-4 (MXF Multichannel Audio Labeling) imferno knows.
    pub enum St377_4Edition for Spec::St377_4 {
        V2012 => "2012",
        V2021 => "2021",
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
        checked(E::V2012, "6.3", Shall).interpret("ST 2067-2:2013 and :2016 cite \"ST 377-4:2011\", which never existed (ST 377-4 was approved on 14 March 2012); the citation is read as this edition.", None),
        checked(E::V2021, "6.3", Shall),
    ],
};

/// AudioChannelLabelSubDescriptor SoundfieldGroupLinkID shall equal the SoundfieldGroup's MCALinkID (§6.4.1).
pub static SoundfieldGroupLinkIDMismatch: Rule<E> = Rule {
    name: "SoundfieldGroupLinkIDMismatch",
    category: Category::Audio,
    description: "AudioChannelLabelSubDescriptor SoundfieldGroupLinkID shall equal the SoundfieldGroup's MCALinkID (§6.4.1).",
    example: Some("<AudioChannelLabelSubDescriptor><SoundfieldGroupLinkID>urn:uuid:abc…</SoundfieldGroupLinkID></AudioChannelLabelSubDescriptor>  <!-- but the parent SoundfieldGroupLabelSubDescriptor MCALinkID = urn:uuid:def… -->"),
    editions: &[
        checked(E::V2012, "6.4.1", Shall).interpret("ST 2067-2:2013 and :2016 cite \"ST 377-4:2011\", which never existed (ST 377-4 was approved on 14 March 2012); the citation is read as this edition.", None),
        checked(E::V2021, "6.4.1", Shall),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[&MCALinkIDMissing, &SoundfieldGroupLinkIDMismatch];
