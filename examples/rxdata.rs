use serde::Deserialize;

mod rmxp_structs;

fn main() {
    color_eyre::install().unwrap();

    let bytes = std::fs::read("examples/Actors.rxdata").unwrap();

    println!("{}", pretty_hex::pretty_hex(&bytes));

    let mut de = alox_48::Deserializer::new(&bytes).unwrap();

    let actors = Vec::<Option<rmxp_structs::rpg::Actor>>::deserialize(&mut de);

    println!("{actors:#?}")
}
