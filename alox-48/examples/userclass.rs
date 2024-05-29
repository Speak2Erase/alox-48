#![allow(dead_code)]
use std::process::Command;

#[derive(alox_48::Deserialize, Debug)]
#[marshal(enforce_class)]
struct Floats([f32; 3]);

fn main() {
    color_eyre::install().unwrap();

    let data = Command::new("ruby")
        .arg("-e")
        .arg(
            r#"
            class MyClass < Array
            end
            
            inst = MyClass.new
            inst << 15.0
            inst << 12.0
            inst << 13.0
            puts Marshal.dump(inst)
        "#,
        )
        .output()
        .unwrap()
        .stdout;

    println!("{}", pretty_hex::pretty_hex(&data));

    let result: Floats = alox_48::from_bytes(&data).unwrap();

    println!("{result:#?}")
}
