#![allow(dead_code)]
use std::process::Command;

fn main() {
    color_eyre::install().unwrap();

    let data = Command::new("ruby")
        .arg("-e")
        .arg(
            r#"
            ary = Array.new(128) do 
                if rand(0..1).ceil.zero?
                    rand(0..200)
                else
                    nil
                end
            end

            puts Marshal.dump(ary)
        "#,
        )
        .output()
        .unwrap()
        .stdout;

    println!("{}", pretty_hex::pretty_hex(&data));

    let result: Vec<Option<u8>> = alox_48::from_bytes(&data).unwrap();

    println!("{result:#?}")
}
