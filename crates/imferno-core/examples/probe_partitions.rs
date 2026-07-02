fn main() {
    let path = std::env::args().nth(1).unwrap();
    let p = std::path::Path::new(&path);
    for (label, target) in [
        ("Footer", regxml::PartitionTarget::Footer),
        ("Header", regxml::PartitionTarget::Header),
    ] {
        let opts = regxml::MxfFragmentOptions {
            partition: target,
            ..Default::default()
        };
        match imferno_core::mxf::metadata::parse_mxf_to_regxml(p, opts) {
            Ok(xml) => println!("[{label}] OK — {} bytes", xml.len()),
            Err(e) => println!("[{label}] FAIL — {e:?}"),
        }
    }
}
