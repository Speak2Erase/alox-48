#![allow(dead_code)]
use std::{collections::HashMap, process::Command};

#[derive(Debug, alox_48::Deserialize, Default)]
struct MyClass {
    #[marshal(default = "default_test")]
    test: Test,
    bool: bool,
}

#[derive(Debug, alox_48::Deserialize, Default)]
struct Test {
    map: HashMap<String, bool>,
}

fn main() {
    color_eyre::install().unwrap();

    let data = Command::new("ruby")
        .arg("-e")
        .arg(
            r#"
        class MyClass
            def initialize()
                @bool = true
            end
        end
        
        klass = MyClass.new
        puts Marshal.dump(klass)
      "#,
        )
        .output()
        .unwrap()
        .stdout;

    println!("{}", pretty_hex::pretty_hex(&data));

    let other: MyClass = alox_48::from_bytes(&data).unwrap();

    println!("{other:?}",);
}
