extern crate version_check;

fn query_rustc_print_cfg(requested_min_version: &str, cfgs: &[&str]) {
    match version_check::is_min_version(requested_min_version) {
        Some((true, actual_version)) => {
            eprintln!("{} >= {}, will enable cfgs: {:?}", actual_version, requested_min_version, cfgs);

            for cfg in cfgs {
                println!("cargo:rustc-cfg={}", cfg);
            }
        },
        Some((false, actual_version)) => {
            eprintln!("{} < {}, will not enable cfgs: {:?}", actual_version, requested_min_version, cfgs);
        },
        None => {
            eprintln!("couldn't query version info from rustc");
        },
    }
}

fn main() {
    query_rustc_print_cfg("1.33.0", &["stable_exhaustive_integer_patterns"]);
}
