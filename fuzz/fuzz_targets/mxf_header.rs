//! MXF header parsing on arbitrary bytes: KLV keys, BER lengths and the
//! partition pack. Binary length fields are where out-of-bounds reads and
//! huge allocations come from.
#![no_main]

use std::io::Cursor;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut reader = Cursor::new(data);
    let _ = imferno_core::mxf::parse_mxf_header_info_from_reader(&mut reader);
});
