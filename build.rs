extern crate vergen;

use vergen::{generate_cargo_keys, ConstantsFlags};

fn main() {
    let mut flags = ConstantsFlags::all();
    flags.toggle(ConstantsFlags::SEMVER_LIGHTWEIGHT);
    flags.toggle(ConstantsFlags::SHA_SHORT);
    flags.toggle(ConstantsFlags::BUILD_DATE);
    generate_cargo_keys(flags).expect("Unable to generate the cargo keys!");
}
