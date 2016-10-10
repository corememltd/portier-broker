// Stable Rust doesn't yet support the custom_derive feature, so we generate
// code for structures which #[derive] Serde's Serialize/Deserialize traits.
//
// See https://serde.rs/codegen-stable.html for more information.

extern crate serde_codegen;

use std::env;
use std::path::Path;

pub fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let src = Path::new("src/serde_types.in.rs");
    let dst = Path::new(&out_dir).join("serde_types.rs");

    serde_codegen::expand(&src, &dst).unwrap();
}
