extern crate version_check;

use std::io::{self, Write};

fn main() {
    match version_check::is_min_version("1.26.0") {
        Some((true, _version)) => {
            println!("cargo:rustc-cfg=stable_i128");
        },
        Some(_) => (),
        None => {
            // TODO: Use `eprintln!` after bumping minimal Rust version to 1.19
            writeln!(io::stdout(), "couldn't query version info from rustc").unwrap();
        },
    }
}
