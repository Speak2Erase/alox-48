fn main() {
    color_eyre::install().unwrap();

    let data = std::fs::read("examples/Actors.rxdata").unwrap();

    // println!("{}", pretty_hex::pretty_hex(&data));

    let result: alox_48::Value = alox_48::from_bytes(&data).unwrap();

    println!("{result:#?}")
}
