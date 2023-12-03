#![allow(dead_code)]
use std::process::Command;

fn main() {
    color_eyre::install().unwrap();

    let data = Command::new("ruby")
        .arg("-e")
        .arg(
            r#"
            str = "hello world".encode("US-ASCII")
            
            puts Marshal.dump(str)
        "#,
        )
        .output()
        .unwrap()
        .stdout;

    println!("{}", pretty_hex::pretty_hex(&data));

    let result: String = alox_48::from_bytes(&data).unwrap();

    println!("{result:#?}")
}
