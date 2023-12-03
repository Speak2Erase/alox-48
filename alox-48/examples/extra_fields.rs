#![allow(dead_code)]
use std::process::Command;

fn main() {
    color_eyre::install().unwrap();

    let data = Command::new("ruby")
        .arg("-e")
        .arg(
            r#"
            str = "hello world"
            str.instance_variable_set("@abc", 123)
            
            puts Marshal.dump(str)
        "#,
        )
        .output()
        .unwrap()
        .stdout;

    println!("{}", pretty_hex::pretty_hex(&data));

    let result: alox_48::RbString = alox_48::from_bytes(&data).unwrap();

    println!("{result:#?}")
}
