mod rmxp_structs;

fn main() {
    color_eyre::install().unwrap();

    let bytes = std::fs::read("examples/Map001.rxdata").unwrap();

    println!("{}", pretty_hex::pretty_hex(&bytes));

    let map: rmxp_structs::rpg::Map = alox_48::from_bytes(&bytes).unwrap();

    println!("{:#?}", map)
}
