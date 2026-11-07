//! Rule table: SMPTE ST 429-8 (Packing List).
//!
//! The Packing List of an ST 2067-2:2013 IMP (2013 §7.1). Later IMF
//! editions report the same checks under ST 2067-2 (ingest, §8.3).

#![allow(non_upper_case_globals)]

use crate::diagnostics::rule::Strength::Shall;
use crate::diagnostics::rule::{checked, AnyRule, Rule, Spec};
use crate::diagnostics::Category;

crate::diagnostics::rule::editions! {
    /// Editions of SMPTE ST 429-8 (Packing List) imferno knows.
    pub enum St429_8Edition for Spec::St429_8 {
        V2007 => "2007",
    }
}

type E = St429_8Edition;

/// The asset's size differs from the Size the Packing List declares.
pub static SizeMismatch: Rule<E> = Rule {
    name: "SizeMismatch",
    category: Category::Asset,
    description: "The asset's size differs from the Size element the Packing List declares for it.",
    example: Some("<Size>1048576</Size> but the file is 1048000 bytes"),
    editions: &[
        checked(E::V2007, "6.4", Shall).interpret("§6.4 defines Size declaratively (it \"contains the size of the asset\"); a Size that differs from the asset's size makes the element wrong, so imferno treats it as a requirement.", None),
    ],
};

/// The asset's SHA-1 digest differs from the Hash the Packing List declares.
pub static ChecksumMismatch: Rule<E> = Rule {
    name: "ChecksumMismatch",
    category: Category::Asset,
    description: "The asset's SHA-1 message digest differs from the Hash element the Packing List declares for it.",
    example: Some("<Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash> but the file's SHA-1 differs"),
    editions: &[
        checked(E::V2007, "6.3", Shall),
    ],
};

/// Every rule of this spec.
pub static ALL: &[&dyn AnyRule] = &[&SizeMismatch, &ChecksumMismatch];
