//! Rule table: SMPTE ST 2067-5 (IMF Essence Component).
//!
//! ST 2067-2 §5.1.1 requires Track Files to conform to ST 2067-5: the
//! 2013 and 2016 editions of ST 2067-2 reference ST 2067-5:2013, the 2020
//! edition references ST 2067-5:2020.
//!
//! Every row checked against the text of its edition (C1/C4).

#![allow(non_upper_case_globals)]

use crate::catalogue::st2067_2::St2067_2Edition;
use crate::diagnostics::rule::Strength::Shall;
use crate::diagnostics::rule::{checked, AnyRule, Rule, Spec};
use crate::diagnostics::Category;

crate::diagnostics::rule::editions! {
    /// Editions of SMPTE ST 2067-5 (Essence Component) imferno knows.
    pub enum St2067_5Edition for Spec::St2067_5 {
        V2013 => "2013",
        V2020 => "2020",
    }
}

impl St2067_5Edition {
    /// The ST 2067-5 edition an ST 2067-2 edition normatively references.
    pub const fn for_core(core: St2067_2Edition) -> Self {
        match core {
            St2067_2Edition::V2013 | St2067_2Edition::V2016 => Self::V2013,
            St2067_2Edition::V2020 => Self::V2020,
        }
    }
}

type E = St2067_5Edition;

/// The Operational Pattern qualifier bits are not those of an IMF Essence
/// Component (internal essence, stream file, uni-track).
pub static OperationalPatternQualifier: Rule<E> = Rule {
    name: "OperationalPatternQualifier",
    category: Category::Container,
    description: "Operational Pattern UL byte 15 qualifier bits 1-3 shall be 0 (internal essence, stream file, uni-track); with the ST 377-1 marker bit 0 set, the low nibble is 0x1.",
    example: Some("Operational Pattern UL 060e2b34.04010102.0d010201.01010900: qualifier 0x09 sets bit 3 (multi-track)"),
    editions: &[
        checked(E::V2013, "5.2.1", Shall).interpret("Bits 4-7 are reserved (ST 377-1 Table 11) and not checked.", None),
        checked(E::V2020, "5.2.1", Shall).interpret("Bits 4-7 are reserved (ST 377-1 Table 11) and not checked.", None),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[&OperationalPatternQualifier];
