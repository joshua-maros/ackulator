use crate::prelude::*;

pub(crate) fn add_default_symbols(env: &mut Environment) {
    const MAX_PRECISION: u32 = std::u32::MAX;
    let unitless = CompositeUnit::unitless();
    let coulumbs = env.find(|unit: &Unit| unit.name == "Coulumbs").unwrap();
    let farads = env.find(|unit: &Unit| unit.name == "Farads").unwrap();
    let meters = env.find(|unit: &Unit| unit.name == "Meters").unwrap();
    // Doubles hold about 16 digits of data. In case there is ever support in the future for using
    // f128s, all constants have 35 digits to make sure they are precise as possible without being
    // overkill.

    let mut mc = |name: &str, value: f64, precision: u32, unit: CompositeUnit| {
        env.add_global_symbol(
            Symbol::plain(name.to_owned()),
            env.make_scalar(value, unit.clone(), precision).into(),
        );
    };

    mc(
        "Pi",
        3.1415926535897932384626433832795028,
        MAX_PRECISION,
        unitless.clone(),
    );
    mc(
        "Euler's Constant",
        2.7182818284590452353602874713526624,
        MAX_PRECISION,
        unitless.clone(),
    );
    mc(
        "Golden Ratio",
        1.6180339887498948482045868343656381,
        MAX_PRECISION,
        unitless.clone(),
    );
    mc(
        "Vacuum Permittivity",
        8.854187812813e-12,
        11,
        farads.clone() / meters.clone(),
    );
    mc(
        "Elementary Charge",
        1.602176634e-19,
        MAX_PRECISION, // SI 2019 makes the elementary charge an exact constant.
        coulumbs.clone().into(),
    );
}
