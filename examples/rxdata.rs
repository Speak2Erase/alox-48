use serde::Deserialize;

mod rmxp_structs;

fn main() {
    color_eyre::install().unwrap();

    let bytes = std::fs::read("examples/Map223.rxdata").unwrap();

    println!("{}", pretty_hex::pretty_hex(&bytes));

    let mut de = alox_48::Deserializer::new(&bytes).unwrap();

    let actors = rmxp_structs::rpg::Map::deserialize(&mut de);

    println!("{actors:#?}")
}
