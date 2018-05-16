extern crate version_check;

use std::io::{self, Write};

fn query_rustc_print_cfg(requested_min_version: &str, cfgs: &[&str]) {
    match version_check::is_min_version(requested_min_version) {
        Some((true, _actual_version)) => {
            for cfg in cfgs {
                println!("cargo:rustc-cfg={}", cfg);
            }
        },
        Some(_) => (),
        None => {
            // TODO: Use `eprintln!` after bumping minimal Rust version to 1.19
            writeln!(io::stdout(), "couldn't query version info from rustc").unwrap();
        },
    }
}

fn main() {
    query_rustc_print_cfg("1.19.0", &["lints_1_19"]);
    query_rustc_print_cfg("1.24.0", &["lints_1_24"]);
    query_rustc_print_cfg("1.26.0", &["lints_1_26", "stable_i128"]);
}
