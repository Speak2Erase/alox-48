#![allow(dead_code)]
use std::{collections::HashMap, process::Command};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct MyClass {
    test: Test,
    bool: bool,
}

#[derive(Debug, Deserialize)]
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
        @test = Test.new
        @bool = true
    end
end

class Test
    def initialize()
        @map = {
            "@test" => true
        }
    end
end

klass = MyClass.new
puts Marshal.dump(klass)
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

    let other: MyClass = alox_48::from_bytes(&data).unwrap();

    println!("{:?}", other);
}
