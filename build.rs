extern crate vergen;

use vergen::{ConstantsFlags, Result, Vergen};

fn main() {
    gen_constants().expect("Unable to generate vergen constants!");
}

fn gen_constants() -> Result<()> {
    let mut flags = ConstantsFlags::all();
    flags.toggle(ConstantsFlags::SEMVER_LIGHTWEIGHT);
    flags.toggle(ConstantsFlags::SHA_SHORT);
    flags.toggle(ConstantsFlags::BUILD_DATE);
    let vergen = Vergen::new(flags)?;

    for (k, v) in vergen.build_info() {
        println!("cargo:rustc-env={}={}", k.name(), v);
    }

    Ok(())
}
