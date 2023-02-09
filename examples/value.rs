fn main() {
    color_eyre::install().unwrap();

    let data = std::fs::read("examples/System.rxdata").unwrap();

    println!("{}", pretty_hex::pretty_hex(&data));

    let result: Result<alox_48::Value, _> = alox_48::from_bytes(&data);

    println!("{result:#?}")
}
