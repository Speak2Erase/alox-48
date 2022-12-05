#![allow(dead_code)]
use std::process::Command;

fn main() {
    color_eyre::install().unwrap();

    let data = Command::new("ruby")
        .arg("-e")
        .arg(
            r#"
            puts Marshal.dump([true, false, 15, 20.0])
        "#,
        )
        .output()
        .unwrap()
        .stdout;

    assert_eq!(
        data[0..=1],
        [4, 8],
        "The data should have version number 4.8"
    );

    println!("{}", pretty_hex::pretty_hex(&data));

    let other: alox_48::Value = alox_48::from_bytes(&data).unwrap();

    println!("{:?}", other);
}
