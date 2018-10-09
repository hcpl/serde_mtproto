extern crate version_check;

fn query_rustc_print_cfg(requested_min_version: &str, cfgs: &[&str]) {
    match version_check::is_min_version(requested_min_version) {
        Some((true, _actual_version)) => {
            for cfg in cfgs {
                println!("cargo:rustc-cfg={}", cfg);
            }
        },
        Some(_) => (),
        None => {
            eprintln!("couldn't query version info from rustc");
        },
    }
}

fn main() {
    query_rustc_print_cfg("1.21.0", &["lints_1_21"]);
    query_rustc_print_cfg("1.24.0", &["lints_1_24"]);
    query_rustc_print_cfg("1.26.0", &["lints_1_26", "stable_i128"]);
    query_rustc_print_cfg("1.27.0", &["lints_1_27"]);
}
