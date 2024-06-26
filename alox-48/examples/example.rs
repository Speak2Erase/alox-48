#![allow(dead_code)]
use std::{collections::HashMap, process::Command};

#[derive(Debug, alox_48::Deserialize)]
#[marshal(expecting = "An instance of MyClass")]
struct MyClass {
    test: Test,
    bool: bool,
}

#[derive(Debug, alox_48::Deserialize)]
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
            "test" => true
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

    println!("{}", pretty_hex::pretty_hex(&data));

    let other: MyClass = alox_48::from_bytes(&data).unwrap();

    println!("{other:?}",);
}
